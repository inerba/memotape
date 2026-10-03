//! Pipeline di Trascrizione di un file: decodifica → frame a 16 kHz → VAD → segmentatore → motore.
//! Gira alla velocità del calcolo e non sa nulla di Tauri né dei file TXT.

use std::collections::VecDeque;
use std::path::Path;

use super::TranscriptionEngine;
use crate::audio_toolkit::decode::Decoder;
use crate::audio_toolkit::resample::FrameResampler;
use crate::audio_toolkit::segmenter::{Event, Params, Segmenter};
use crate::audio_toolkit::vad::VoiceDetector;
use crate::error::AppError;

/// Trascrive `source` chiamando `on_phrase(id, testo)` per ogni Frase non vuota, in ordine.
pub fn transcribe_file(
    source: &Path,
    engine: &mut dyn TranscriptionEngine,
    detector: &mut dyn VoiceDetector,
    on_phrase: &mut dyn FnMut(u32, String),
) -> Result<(), AppError> {
    detector.reset();
    let mut events = Events {
        decoder: Decoder::open(source)?,
        resampler: None,
        detector,
        segmenter: Segmenter::new(Params::default()),
        queue: VecDeque::new(),
        decoded_all: false,
        error: None,
    };
    let mut phrase_id = 0;
    while let Some(event) = events.next()? {
        // Fuori da una Frase arrivano solo inizi: l'audio lo consuma `PhraseAudio`.
        if event != Event::PhraseStart {
            continue;
        }
        let mut audio = PhraseAudio {
            events: &mut events,
            ended: false,
        };
        let text = engine.transcribe(&mut audio);
        // Il motore può fermarsi prima della fine della Frase: il resto si scarta.
        audio.for_each(drop);
        if let Some(error) = events.error.take() {
            return Err(error);
        }
        let text = text?;
        if !text.is_empty() {
            on_phrase(phrase_id, text);
            phrase_id += 1;
        }
    }
    Ok(())
}

/// Gli eventi del segmentatore, prodotti decodificando il file solo quando servono.
struct Events<'a> {
    decoder: Decoder,
    resampler: Option<FrameResampler>,
    detector: &'a mut dyn VoiceDetector,
    segmenter: Segmenter,
    queue: VecDeque<Event>,
    decoded_all: bool,
    /// Errore incontrato mentre il motore leggeva l'audio della Frase.
    error: Option<AppError>,
}

impl Events<'_> {
    fn next(&mut self) -> Result<Option<Event>, AppError> {
        loop {
            if let Some(event) = self.queue.pop_front() {
                return Ok(Some(event));
            }
            if self.decoded_all {
                return Ok(None);
            }
            let frames = match self.decoder.next_mono()? {
                Some((mono, rate)) => {
                    let resampler = match &mut self.resampler {
                        Some(resampler) => resampler,
                        // ponytail: frequenza fissata dal primo blocco; i file che la cambiano a metà non sono gestiti.
                        None => self.resampler.insert(FrameResampler::new(rate)?),
                    };
                    resampler.push(&mono)
                }
                None => {
                    self.decoded_all = true;
                    self.resampler
                        .as_mut()
                        .map(FrameResampler::finish)
                        .unwrap_or_default()
                }
            };
            for frame in frames {
                let probability = self.detector.probability(&frame)?;
                self.queue.extend(self.segmenter.push(frame, probability));
            }
            if self.decoded_all {
                self.queue.extend(self.segmenter.finish());
            }
        }
    }
}

/// L'audio di una Frase, frame per frame, fino alla sua fine.
struct PhraseAudio<'e, 'a> {
    events: &'e mut Events<'a>,
    ended: bool,
}

impl Iterator for PhraseAudio<'_, '_> {
    type Item = Vec<f32>;

    fn next(&mut self) -> Option<Vec<f32>> {
        if self.ended {
            return None;
        }
        match self.events.next() {
            Ok(Some(Event::Audio(frame))) => return Some(frame),
            Ok(_) => {}
            Err(error) => self.events.error = Some(error),
        }
        self.ended = true;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::resample::FRAME_SAMPLES;
    use std::path::PathBuf;

    /// Detector finto guidato dall'energia del frame.
    struct EnergyDetector;

    impl VoiceDetector for EnergyDetector {
        fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError> {
            let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();
            Ok(if rms > 0.01 { 1.0 } else { 0.0 })
        }

        fn reset(&mut self) {}
    }

    /// Motore finto: restituisce "frase N" e ricorda quanti campioni ha ricevuto ogni Frase.
    #[derive(Default)]
    struct FakeEngine {
        samples: Vec<usize>,
    }

    impl TranscriptionEngine for FakeEngine {
        fn transcribe(
            &mut self,
            frames: &mut dyn Iterator<Item = Vec<f32>>,
        ) -> Result<String, AppError> {
            self.samples.push(frames.map(|f| f.len()).sum());
            Ok(format!("frase {}", self.samples.len()))
        }
    }

    /// WAV PCM 16 bit: `(secondi, tono?)` in sequenza.
    fn wav(name: &str, rate: u32, channels: u16, parts: &[(f32, bool)]) -> PathBuf {
        let mut samples = Vec::new();
        for &(seconds, tone) in parts {
            for i in 0..(seconds * rate as f32) as usize {
                let t = i as f32 / rate as f32;
                let s = if tone {
                    (t * 300.0 * std::f32::consts::TAU).sin() * 0.3
                } else {
                    0.0
                };
                for _ in 0..channels {
                    samples.push((s * i16::MAX as f32) as i16);
                }
            }
        }
        let data_len = (samples.len() * 2) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&rate.to_le_bytes());
        bytes.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
        bytes.extend_from_slice(&(channels * 2).to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for s in samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        let path = std::env::temp_dir().join(format!("sbobino-test-{name}.wav"));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn run(path: &Path, engine: &mut FakeEngine) -> Result<Vec<(u32, String)>, AppError> {
        let mut phrases = Vec::new();
        transcribe_file(path, engine, &mut EnergyDetector, &mut |id, text| {
            phrases.push((id, text));
        })?;
        Ok(phrases)
    }

    #[test]
    fn due_tratti_di_parlato_diventano_due_frasi_in_ordine() {
        let path = wav(
            "due-frasi",
            44_100,
            2,
            &[
                (0.5, false),
                (1.5, true),
                (1.5, false),
                (2.0, true),
                (0.5, false),
            ],
        );
        let mut engine = FakeEngine::default();
        let phrases = run(&path, &mut engine).unwrap();
        assert_eq!(
            phrases,
            vec![(0, "frase 1".to_string()), (1, "frase 2".to_string())]
        );
        // Ogni Frase ha il suo parlato più prefill e hangover, in frame interi da 30 ms.
        let seconds: Vec<f32> = engine
            .samples
            .iter()
            .map(|&n| n as f32 / 16_000.0)
            .collect();
        assert!((2.0..3.0).contains(&seconds[0]), "{seconds:?}");
        assert!((2.5..3.5).contains(&seconds[1]), "{seconds:?}");
        assert!(engine.samples.iter().all(|n| n % FRAME_SAMPLES == 0));
    }

    #[test]
    fn il_parlato_continuo_si_taglia_in_frasi_da_al_massimo_18_secondi() {
        let path = wav("taglio", 16_000, 1, &[(40.0, true)]);
        let mut engine = FakeEngine::default();
        let phrases = run(&path, &mut engine).unwrap();
        assert_eq!(phrases.len(), 3);
        assert_eq!(engine.samples[..2], [18 * 16_000, 18 * 16_000]);
        // 40 s = 1333,3 frame: l'ultimo è completato con zeri.
        assert_eq!(engine.samples.iter().sum::<usize>(), 1334 * FRAME_SAMPLES);
    }

    #[test]
    fn un_file_senza_parlato_non_produce_frasi() {
        let path = wav("silenzio", 22_050, 1, &[(3.0, false)]);
        assert!(run(&path, &mut FakeEngine::default()).unwrap().is_empty());
    }

    #[test]
    fn un_file_che_non_e_audio_da_codec_non_supportato() {
        let path = std::env::temp_dir().join("sbobino-test-non-audio.mp3");
        std::fs::write(&path, b"questo non e' audio, solo testo qualunque").unwrap();
        let error = run(&path, &mut FakeEngine::default()).unwrap_err();
        assert!(matches!(error, AppError::UnsupportedCodec(_)), "{error:?}");
    }

    #[test]
    fn un_file_mancante_e_illeggibile() {
        let path = std::env::temp_dir().join("sbobino-test-non-esiste.wav");
        let error = run(&path, &mut FakeEngine::default()).unwrap_err();
        assert!(matches!(error, AppError::UnreadableFile(_)), "{error:?}");
    }

    /// Smoke test con Silero e Nemotron veri: richiede il modello in `%APPDATA%\sbobino\models`.
    #[test]
    #[ignore = "richiede Nemotron scaricato a mano (vedi AGENTS.md)"]
    fn nemotron_trascrive_il_parlato_italiano() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let model = PathBuf::from(std::env::var("APPDATA").unwrap())
            .join("sbobino/models")
            .join(crate::managers::transcription::NEMOTRON_FILE);
        let mut engine = super::super::transcribe_cpp::TranscribeCpp::load(&model).unwrap();
        let mut detector =
            crate::audio_toolkit::vad::Silero::new(&root.join("resources/silero_vad.onnx"))
                .unwrap();
        let mut phrases = Vec::new();
        transcribe_file(
            &root.join("tests/fixtures/parlato-it.wav"),
            &mut engine,
            &mut detector,
            &mut |_, text| phrases.push(text),
        )
        .unwrap();
        let text = phrases.join("\n").to_lowercase();
        println!("{text}");
        assert_eq!(phrases.len(), 2, "{phrases:?}");
        assert!(
            text.contains("trascrizione") && text.contains("testo"),
            "{text}"
        );
    }
}
