//! Pipeline di Trascrizione di un file: decodifica → frame a 16 kHz → VAD → segmentatore → motore.
//! Gira alla velocità del calcolo e non sa nulla di Tauri né dei file TXT.

use std::collections::VecDeque;
use std::path::Path;

use transcribe_cpp::CancelToken;

use super::TranscriptionEngine;
use crate::audio_toolkit::decode::Decoder;
use crate::audio_toolkit::resample::FrameResampler;
use crate::audio_toolkit::segmenter::{Event, Params, Segmenter};
use crate::audio_toolkit::vad::VoiceDetector;
use crate::error::AppError;

/// Cosa la pipeline comunica a chi la esegue.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineEvent {
    /// Percentuale decodificata, `None` se la durata non è nota. Si emette all'inizio e a ogni
    /// cambio di punto percentuale.
    Progress(Option<u8>),
    /// Una Frase non vuota, con id progressivo da 0.
    Phrase { id: u32, text: String },
}

/// Trascrive `source` chiamando `on_event` per il progresso e per ogni Frase non vuota, in ordine.
/// Con `cancel` premuto si ferma al prossimo blocco decodificato o alla fine della Frase in corso,
/// senza emettere quella Frase, e restituisce `AppError::Cancelled`.
pub fn transcribe_file(
    source: &Path,
    engine: &mut dyn TranscriptionEngine,
    detector: &mut dyn VoiceDetector,
    cancel: &CancelToken,
    on_event: &mut dyn FnMut(PipelineEvent),
) -> Result<(), AppError> {
    detector.reset();
    let decoder = Decoder::open(source)?;
    let progress = decoder.progress();
    on_event(PipelineEvent::Progress(progress));
    let mut events = SegmentedSource {
        decoder,
        resampler: None,
        detector,
        segmenter: Segmenter::new(Params::default()),
        cancel,
        queue: VecDeque::new(),
        decoded_all: false,
        error: None,
        progress,
        on_event,
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
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        if let Some(error) = events.error.take() {
            return Err(error);
        }
        let text = text?;
        if !text.is_empty() {
            (events.on_event)(PipelineEvent::Phrase {
                id: phrase_id,
                text,
            });
            phrase_id += 1;
        }
    }
    Ok(())
}

/// Gli eventi del segmentatore, prodotti decodificando il file solo quando servono.
struct SegmentedSource<'a> {
    decoder: Decoder,
    resampler: Option<FrameResampler>,
    detector: &'a mut dyn VoiceDetector,
    segmenter: Segmenter,
    cancel: &'a CancelToken,
    queue: VecDeque<Event>,
    decoded_all: bool,
    /// Errore incontrato mentre il motore leggeva l'audio della Frase.
    error: Option<AppError>,
    /// L'ultimo progresso emesso.
    progress: Option<u8>,
    on_event: &'a mut dyn FnMut(PipelineEvent),
}

impl SegmentedSource<'_> {
    fn next(&mut self) -> Result<Option<Event>, AppError> {
        loop {
            if self.cancel.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            if let Some(event) = self.queue.pop_front() {
                return Ok(Some(event));
            }
            if self.decoded_all {
                return Ok(None);
            }
            let frames = match self.decoder.next_mono()? {
                Some((mono, rate)) => {
                    let progress = self.decoder.progress();
                    if progress != self.progress {
                        self.progress = progress;
                        (self.on_event)(PipelineEvent::Progress(progress));
                    }
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
    events: &'e mut SegmentedSource<'a>,
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
    /// Con `cancel_at` preme Annulla mentre trascrive la Frase N (da 1).
    #[derive(Default)]
    struct FakeEngine {
        samples: Vec<usize>,
        cancel_at: Option<(usize, CancelToken)>,
    }

    impl TranscriptionEngine for FakeEngine {
        fn transcribe(
            &mut self,
            frames: &mut dyn Iterator<Item = Vec<f32>>,
        ) -> Result<String, AppError> {
            self.samples.push(frames.map(|f| f.len()).sum());
            if let Some((n, cancel)) = &self.cancel_at
                && *n == self.samples.len()
            {
                cancel.cancel();
            }
            Ok(format!("frase {}", self.samples.len()))
        }
    }

    /// WAV PCM 16 bit: `(secondi, tono?)` in sequenza.
    fn wav_bytes(rate: u32, channels: u16, parts: &[(f32, bool)]) -> Vec<u8> {
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
        bytes
    }

    fn write(name: &str, bytes: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(format!("sbobino-test-{name}"));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn wav(name: &str, rate: u32, channels: u16, parts: &[(f32, bool)]) -> PathBuf {
        write(&format!("{name}.wav"), &wav_bytes(rate, channels, parts))
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Gli eventi della pipeline: Frasi e progresso, nell'ordine di emissione.
    fn events(path: &Path, engine: &mut FakeEngine) -> Result<Vec<PipelineEvent>, AppError> {
        let cancel = engine
            .cancel_at
            .as_ref()
            .map_or_else(CancelToken::new, |(_, cancel)| cancel.clone());
        let mut events = Vec::new();
        transcribe_file(path, engine, &mut EnergyDetector, &cancel, &mut |event| {
            events.push(event);
        })?;
        Ok(events)
    }

    fn run(path: &Path, engine: &mut FakeEngine) -> Result<Vec<(u32, String)>, AppError> {
        Ok(events(path, engine)?
            .into_iter()
            .filter_map(|event| match event {
                PipelineEvent::Phrase { id, text } => Some((id, text)),
                PipelineEvent::Progress(_) => None,
            })
            .collect())
    }

    fn progress(path: &Path) -> Vec<Option<u8>> {
        events(path, &mut FakeEngine::default())
            .unwrap()
            .into_iter()
            .filter_map(|event| match event {
                PipelineEvent::Progress(percent) => Some(percent),
                PipelineEvent::Phrase { .. } => None,
            })
            .collect()
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

    /// Tre tratti di parlato separati da silenzio.
    fn tre_frasi(name: &str) -> PathBuf {
        wav(
            name,
            16_000,
            1,
            &[
                (1.0, true),
                (1.5, false),
                (1.0, true),
                (1.5, false),
                (1.0, true),
                (0.5, false),
            ],
        )
    }

    #[test]
    fn annulla_ferma_la_pipeline_tra_una_frase_e_l_altra() {
        let path = tre_frasi("annulla");
        let cancel = CancelToken::new();
        let mut engine = FakeEngine {
            cancel_at: Some((2, cancel.clone())),
            ..FakeEngine::default()
        };
        let mut phrases = Vec::new();
        let error = transcribe_file(
            &path,
            &mut engine,
            &mut EnergyDetector,
            &cancel,
            &mut |event| {
                if let PipelineEvent::Phrase { id, .. } = event {
                    phrases.push(id);
                }
            },
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Cancelled), "{error:?}");
        // Resta solo la Frase già comparsa: quella annullata non si emette e la terza non parte.
        assert_eq!(phrases, vec![0]);
        assert_eq!(engine.samples.len(), 2);
    }

    #[test]
    fn annullata_prima_di_partire_non_trascrive_nulla() {
        let path = tre_frasi("annulla-subito");
        let cancel = CancelToken::new();
        cancel.cancel();
        let mut engine = FakeEngine::default();
        let mut emitted = 0;
        let error = transcribe_file(
            &path,
            &mut engine,
            &mut EnergyDetector,
            &cancel,
            &mut |_| {
                emitted += 1;
            },
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Cancelled), "{error:?}");
        assert!(engine.samples.is_empty());
        // Solo il progresso iniziale.
        assert_eq!(emitted, 1);
    }

    #[test]
    fn un_file_senza_parlato_non_produce_frasi() {
        let path = wav("silenzio", 22_050, 1, &[(3.0, false)]);
        assert!(run(&path, &mut FakeEngine::default()).unwrap().is_empty());
    }

    #[test]
    fn con_la_durata_nota_il_progresso_sale_da_0_a_100() {
        let path = wav("progresso", 16_000, 1, &[(1.0, true), (2.0, false)]);
        let percents = progress(&path);
        assert_eq!(percents.first(), Some(&Some(0)));
        assert_eq!(percents.last(), Some(&Some(100)));
        assert!(
            percents.windows(2).all(|w| w[0] < w[1]),
            "il progresso cresce e non si ripete: {percents:?}"
        );
    }

    #[test]
    fn senza_durata_nota_il_progresso_e_indeterminato() {
        // Un WAV "in streaming", come quelli di ffmpeg su stdout: lunghezze 0xFFFFFFFF.
        let mut bytes = wav_bytes(16_000, 1, &[(1.0, true), (1.0, false)]);
        bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        let path = write("streaming.wav", &bytes);
        assert_eq!(progress(&path), vec![None]);
    }

    #[test]
    fn un_video_mp4_con_audio_aac_si_trascrive() {
        let path = fixture("parlato-it.mp4");
        let mut engine = FakeEngine::default();
        let events = events(&path, &mut engine).unwrap();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, PipelineEvent::Phrase { .. })),
            "{events:?}"
        );
        // Tutto l'audio (circa 9 s di parlato) arriva al motore e il progresso arriva a 100.
        let seconds = engine.samples.iter().sum::<usize>() as f32 / 16_000.0;
        assert!(seconds > 3.0, "{seconds}");
        assert_eq!(events.last(), Some(&PipelineEvent::Progress(Some(100))));
    }

    #[test]
    fn un_file_che_non_e_audio_da_codec_non_supportato() {
        let path = write(
            "non-audio.mp3",
            b"questo non e' audio, solo testo qualunque",
        );
        let error = run(&path, &mut FakeEngine::default()).unwrap_err();
        assert!(matches!(error, AppError::UnsupportedCodec(_)), "{error:?}");
    }

    #[test]
    fn un_codec_non_supportato_in_un_contenitore_valido_da_errore_dedicato() {
        // WAV con format tag 0x2000: AC-3, che Symphonia non decodifica.
        let mut bytes = wav_bytes(48_000, 2, &[(0.5, true)]);
        bytes[20..22].copy_from_slice(&0x2000u16.to_le_bytes());
        let path = write("ac3.wav", &bytes);
        let error = run(&path, &mut FakeEngine::default()).unwrap_err();
        assert!(matches!(error, AppError::UnsupportedCodec(_)), "{error:?}");
    }

    #[test]
    fn un_file_mancante_e_illeggibile() {
        let path = std::env::temp_dir().join("sbobino-test-non-esiste.wav");
        let error = run(&path, &mut FakeEngine::default()).unwrap_err();
        assert!(matches!(error, AppError::UnreadableFile(_)), "{error:?}");
    }

    /// Smoke test con Silero e Nemotron veri: richiede il modello in `%APPDATA%\it.sbobino.desktop\models`.
    #[test]
    #[ignore = "richiede Nemotron scaricato (Impostazioni → Trascrizione)"]
    fn nemotron_trascrive_il_parlato_italiano() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let model = crate::managers::models::default_model().path(
            &PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.sbobino.desktop/models"),
        );
        let mut engine =
            super::super::transcribe_cpp::TranscribeCpp::load(&model, &CancelToken::new()).unwrap();
        let mut detector =
            crate::audio_toolkit::vad::Silero::new(&root.join("resources/silero_vad.onnx"))
                .unwrap();
        // Lo stesso parlato in WAV e nel video MP4/AAC.
        for name in ["parlato-it.wav", "parlato-it.mp4"] {
            let mut phrases = Vec::new();
            let cancel = CancelToken::new();
            transcribe_file(
                &fixture(name),
                &mut engine,
                &mut detector,
                &cancel,
                &mut |event| {
                    if let PipelineEvent::Phrase { text, .. } = event {
                        phrases.push(text);
                    }
                },
            )
            .unwrap();
            let text = phrases
                .join(
                    "
",
                )
                .to_lowercase();
            println!("{name}: {text}");
            assert_eq!(phrases.len(), 2, "{name}: {phrases:?}");
            assert!(
                text.contains("trascrizione") && text.contains("testo"),
                "{name}: {text}"
            );
        }
    }
}
