//! Pipeline di Trascrizione: frame a 16 kHz → VAD → segmentatore → motore. La fonte dei frame può
//! essere un file (`transcribe_file`) o qualunque altro flusso. Gira alla velocità del calcolo e non
//! sa nulla di Tauri né dei file TXT.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::Path;

use transcribe_cpp::CancelToken;

use super::TranscriptionEngine;
use crate::audio_toolkit::decode::Decoder;
use crate::audio_toolkit::resample::FrameResampler;
use crate::audio_toolkit::segmenter::{Event, FRAME_MS, Params, Segmenter};
use crate::audio_toolkit::vad::VoiceDetector;
use crate::error::AppError;

/// Cosa la pipeline comunica a chi la esegue. `inizio_ms` e `fine_ms` sono sulla linea del tempo
/// della Sorgente, contati nei frame da 30 ms ricevuti dalla fonte (fine esclusa). Comprendono il
/// prefill e l'hangover del segmentatore: sono l'audio dato al motore.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineEvent {
    /// Percentuale decodificata, `None` se la durata non è nota. Si emette all'inizio e a ogni
    /// cambio di punto percentuale.
    Progress(Option<u8>),
    /// Il testo provvisorio della Frase in corso, con l'id che avrà la Frase e l'audio letto
    /// finora. Se la Frase finisce vuota arriva un Parziale vuoto e l'id passa alla successiva.
    Partial {
        id: u32,
        inizio_ms: u32,
        fine_ms: u32,
        text: String,
    },
    /// Una Frase non vuota, con id progressivo da 0.
    Phrase {
        id: u32,
        inizio_ms: u32,
        fine_ms: u32,
        text: String,
    },
}

/// Quello che la fonte di frame dà alla pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum Feed {
    /// Un frame da 30 ms (`FRAME_SAMPLES` campioni) mono a 16 kHz in [-1, 1].
    Frame(Vec<f32>),
    /// Chiude la Frase in corso come a fine parlato (la Pausa): il frame successivo può solo
    /// aprirne una nuova.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "la userà la Trascrizione dal vivo")
    )]
    ClosePhrase,
    /// Avanzamento da inoltrare com'è in `PipelineEvent::Progress`.
    Progress(Option<u8>),
}

/// Trascrive la fonte `frames` chiamando `on_event` per il progresso, i Parziali e ogni Frase non vuota, in
/// ordine. `language` è la Lingua del parlato (`None`: automatica). Con `cancel` premuto si ferma
/// al prossimo frame letto o alla fine della Frase in corso, senza emettere quella Frase, e
/// restituisce `AppError::Cancelled`. La fonte si legge solo quando servono frame, e un suo errore
/// ferma la pipeline.
pub fn transcribe(
    frames: &mut dyn Iterator<Item = Result<Feed, AppError>>,
    engine: &mut dyn TranscriptionEngine,
    detector: &mut dyn VoiceDetector,
    language: Option<&str>,
    cancel: &CancelToken,
    on_event: &mut dyn FnMut(PipelineEvent),
) -> Result<(), AppError> {
    detector.reset();
    // Lo usano sia la lettura dell'audio (progresso) sia il motore (Parziali), mai insieme.
    let on_event = RefCell::new(on_event);
    let emit = |event| (on_event.borrow_mut())(event);
    let mut events = SegmentedSource {
        frames,
        detector,
        segmenter: Segmenter::new(Params::default()),
        cancel,
        queue: VecDeque::new(),
        ended: false,
        error: None,
        emit: &emit,
    };
    let mut phrase_id = 0;
    while let Some(event) = events.next()? {
        // Fuori da una Frase arrivano solo inizi: l'audio lo consuma `PhraseAudio`.
        let Event::PhraseStart(start) = event else {
            continue;
        };
        let inizio_ms = to_ms(start);
        let end = Cell::new(start);
        let mut audio = PhraseAudio {
            events: &mut events,
            end: &end,
            ended: false,
        };
        let mut partial_shown = false;
        let mut on_partial = |text: &str| {
            partial_shown = !text.is_empty();
            emit(PipelineEvent::Partial {
                id: phrase_id,
                inizio_ms,
                fine_ms: to_ms(end.get()),
                text: text.to_string(),
            });
        };
        let text = engine.transcribe(&mut audio, language, &mut on_partial);
        // Il motore può fermarsi prima della fine della Frase: il resto si scarta.
        audio.for_each(drop);
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        if let Some(error) = events.error.take() {
            return Err(error);
        }
        let text = text?;
        let fine_ms = to_ms(end.get());
        if text.is_empty() && partial_shown {
            // Il Parziale di una Frase finita vuota non resta a schermo.
            emit(PipelineEvent::Partial {
                id: phrase_id,
                inizio_ms,
                fine_ms,
                text: String::new(),
            });
        }
        if !text.is_empty() {
            emit(PipelineEvent::Phrase {
                id: phrase_id,
                inizio_ms,
                fine_ms,
                text,
            });
            phrase_id += 1;
        }
    }
    Ok(())
}

/// Trascrive il file `source` come `transcribe`, con il progresso della decodifica.
pub fn transcribe_file(
    source: &Path,
    engine: &mut dyn TranscriptionEngine,
    detector: &mut dyn VoiceDetector,
    language: Option<&str>,
    cancel: &CancelToken,
    on_event: &mut dyn FnMut(PipelineEvent),
) -> Result<(), AppError> {
    let mut frames = FileFrames::open(source)?;
    on_event(PipelineEvent::Progress(frames.progress));
    transcribe(&mut frames, engine, detector, language, cancel, on_event)
}

/// Millisecondi dall'inizio della Sorgente a `frames` frame.
fn to_ms(frames: usize) -> u32 {
    u32::try_from(frames * FRAME_MS as usize).unwrap_or(u32::MAX)
}

/// I frame di un file, decodificati e ricampionati un blocco alla volta.
struct FileFrames {
    decoder: Decoder,
    resampler: Option<FrameResampler>,
    /// L'ultimo progresso emesso.
    progress: Option<u8>,
    ready: VecDeque<Feed>,
    decoded_all: bool,
}

impl FileFrames {
    fn open(path: &Path) -> Result<Self, AppError> {
        let decoder = Decoder::open(path)?;
        Ok(Self {
            progress: decoder.progress(),
            decoder,
            resampler: None,
            ready: VecDeque::new(),
            decoded_all: false,
        })
    }

    /// Decodifica il blocco successivo in `ready`.
    fn decode(&mut self) -> Result<(), AppError> {
        let frames = match self.decoder.next_mono()? {
            Some((mono, rate)) => {
                let progress = self.decoder.progress();
                if progress != self.progress {
                    self.progress = progress;
                    self.ready.push_back(Feed::Progress(progress));
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
        self.ready.extend(frames.into_iter().map(Feed::Frame));
        Ok(())
    }
}

impl Iterator for FileFrames {
    type Item = Result<Feed, AppError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(input) = self.ready.pop_front() {
                return Some(Ok(input));
            }
            if self.decoded_all {
                return None;
            }
            if let Err(error) = self.decode() {
                self.decoded_all = true;
                return Some(Err(error));
            }
        }
    }
}

/// Gli eventi del segmentatore, prodotti leggendo la fonte solo quando servono.
struct SegmentedSource<'a> {
    frames: &'a mut dyn Iterator<Item = Result<Feed, AppError>>,
    detector: &'a mut dyn VoiceDetector,
    segmenter: Segmenter,
    cancel: &'a CancelToken,
    queue: VecDeque<Event>,
    ended: bool,
    /// Errore incontrato mentre il motore leggeva l'audio della Frase.
    error: Option<AppError>,
    emit: &'a dyn Fn(PipelineEvent),
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
            if self.ended {
                return Ok(None);
            }
            match self.frames.next().transpose()? {
                Some(Feed::Frame(frame)) => {
                    let probability = self.detector.probability(&frame)?;
                    self.queue.extend(self.segmenter.push(frame, probability));
                }
                Some(Feed::ClosePhrase) => self.queue.extend(self.segmenter.close_phrase()),
                Some(Feed::Progress(progress)) => (self.emit)(PipelineEvent::Progress(progress)),
                None => {
                    self.ended = true;
                    self.queue.extend(self.segmenter.close_phrase());
                }
            }
        }
    }
}

/// L'audio di una Frase, frame per frame, fino alla sua fine. `end` è la posizione dopo l'ultimo
/// frame letto: a Frase finita, la sua fine sulla linea del tempo, esclusa.
struct PhraseAudio<'e, 'a> {
    events: &'e mut SegmentedSource<'a>,
    end: &'e Cell<usize>,
    ended: bool,
}

impl Iterator for PhraseAudio<'_, '_> {
    type Item = Vec<f32>;

    fn next(&mut self) -> Option<Vec<f32>> {
        if self.ended {
            return None;
        }
        match self.events.next() {
            Ok(Some(Event::Audio(frame))) => {
                self.end.set(self.end.get() + 1);
                return Some(frame);
            }
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
    use crate::engine::EngineError;
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

    /// Motore finto: restituisce "frase N" e ricorda quanti campioni e quale lingua ha ricevuto
    /// ogni Frase. Con `cancel_at` preme Annulla mentre trascrive la Frase N (da 1). Con
    /// `streaming` manda un Parziale ogni 10 frame; con `empty_at` la Frase N finisce vuota; con
    /// `busy` risponde `Busy`.
    #[derive(Default)]
    struct FakeEngine {
        samples: Vec<usize>,
        languages: Vec<Option<String>>,
        cancel_at: Option<(usize, CancelToken)>,
        streaming: bool,
        empty_at: Option<usize>,
        busy: bool,
    }

    impl TranscriptionEngine for FakeEngine {
        fn transcribe(
            &mut self,
            frames: &mut dyn Iterator<Item = Vec<f32>>,
            language: Option<&str>,
            on_partial: &mut dyn FnMut(&str),
        ) -> Result<String, EngineError> {
            if self.busy {
                return Err(EngineError::Busy);
            }
            let n = self.samples.len() + 1;
            let mut samples = 0;
            for (i, frame) in frames.enumerate() {
                samples += frame.len();
                if self.streaming && i % 10 == 9 {
                    on_partial(&format!("frase {n} ({} frame)", i + 1));
                }
            }
            self.samples.push(samples);
            self.languages.push(language.map(String::from));
            if let Some((at, cancel)) = &self.cancel_at
                && *at == n
            {
                cancel.cancel();
            }
            if self.empty_at == Some(n) {
                return Ok(String::new());
            }
            Ok(format!("frase {n}"))
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
        transcribe_file(
            path,
            engine,
            &mut EnergyDetector,
            None,
            &cancel,
            &mut |event| {
                events.push(event);
            },
        )?;
        Ok(events)
    }

    fn run(path: &Path, engine: &mut FakeEngine) -> Result<Vec<(u32, String)>, AppError> {
        Ok(events(path, engine)?
            .into_iter()
            .filter_map(|event| match event {
                PipelineEvent::Phrase { id, text, .. } => Some((id, text)),
                _ => None,
            })
            .collect())
    }

    fn progress(path: &Path) -> Vec<Option<u8>> {
        events(path, &mut FakeEngine::default())
            .unwrap()
            .into_iter()
            .filter_map(|event| match event {
                PipelineEvent::Progress(percent) => Some(percent),
                _ => None,
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
    fn i_parziali_arrivano_prima_della_frase_con_lo_stesso_id() {
        let path = tre_frasi("parziali");
        let mut engine = FakeEngine {
            streaming: true,
            ..FakeEngine::default()
        };
        let mut events = Vec::new();
        transcribe_file(
            &path,
            &mut engine,
            &mut EnergyDetector,
            Some("it"),
            &CancelToken::new(),
            &mut |event| events.push(event),
        )
        .unwrap();
        // Per ogni Frase, in ordine: almeno un Parziale con il suo id, poi la Frase.
        let (mut next_id, mut partials) = (0, 0);
        for event in &events {
            match event {
                PipelineEvent::Partial { id, text, .. } => {
                    assert_eq!(*id, next_id, "{events:?}");
                    assert!(text.starts_with(&format!("frase {} (", id + 1)), "{text}");
                    partials += 1;
                }
                PipelineEvent::Phrase { id, .. } => {
                    assert_eq!(*id, next_id, "{events:?}");
                    assert!(partials > 0, "Frase {id} senza Parziali: {events:?}");
                    (next_id, partials) = (next_id + 1, 0);
                }
                PipelineEvent::Progress(_) => {}
            }
        }
        assert_eq!(next_id, 3);
        // La Lingua del parlato arriva al motore a ogni Frase.
        assert_eq!(engine.languages, vec![Some("it".to_string()); 3]);
    }

    #[test]
    fn il_parziale_di_una_frase_finita_vuota_si_cancella_e_l_id_passa_alla_successiva() {
        let path = tre_frasi("frase-vuota");
        let mut engine = FakeEngine {
            streaming: true,
            empty_at: Some(2),
            ..FakeEngine::default()
        };
        let events = events(&path, &mut engine).unwrap();
        // Ogni evento come (id, testo), con i Parziali ridotti alla Frase che trascrivono.
        let mut ids: Vec<(u32, &str)> = events
            .iter()
            .filter_map(|event| match event {
                PipelineEvent::Partial { id, text, .. } => {
                    Some((*id, text.split(" (").next().unwrap_or_default()))
                }
                PipelineEvent::Phrase { id, .. } => Some((*id, "FRASE")),
                PipelineEvent::Progress(_) => None,
            })
            .collect();
        ids.dedup();
        assert_eq!(
            ids,
            [
                (0, "frase 1"),
                (0, "FRASE"),
                (1, "frase 2"),
                (1, ""),
                (1, "frase 3"),
                (1, "FRASE"),
            ]
        );
    }

    #[test]
    fn un_motore_occupato_e_un_modello_in_uso_non_un_guasto() {
        let path = tre_frasi("occupato");
        let mut engine = FakeEngine {
            busy: true,
            ..FakeEngine::default()
        };
        let error = run(&path, &mut engine).unwrap_err();
        assert!(matches!(error, AppError::ModelInUse(_)), "{error:?}");
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
            None,
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
            None,
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

    /// `n` frame di parlato (`true`) o di silenzio, in sequenza.
    fn frames(parts: &[(bool, usize)]) -> Vec<Result<Feed, AppError>> {
        parts
            .iter()
            .flat_map(|&(speech, n)| {
                let sample = if speech { 0.3 } else { 0.0 };
                std::iter::repeat_n(Ok(Feed::Frame(vec![sample; FRAME_SAMPLES])), n)
            })
            .collect()
    }

    fn run_feed(
        feed: Vec<Result<Feed, AppError>>,
        engine: &mut FakeEngine,
    ) -> Result<Vec<PipelineEvent>, AppError> {
        let mut events = Vec::new();
        transcribe(
            &mut feed.into_iter(),
            engine,
            &mut EnergyDetector,
            None,
            &CancelToken::new(),
            &mut |event| events.push(event),
        )?;
        Ok(events)
    }

    /// Le Frasi come `(id, inizio_ms, fine_ms)`.
    fn times(events: &[PipelineEvent]) -> Vec<(u32, u32, u32)> {
        events
            .iter()
            .filter_map(|event| match event {
                PipelineEvent::Phrase {
                    id,
                    inizio_ms,
                    fine_ms,
                    ..
                } => Some((*id, *inizio_ms, *fine_ms)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn le_frasi_e_i_parziali_portano_i_tempi_sulla_linea_del_tempo_della_sorgente() {
        let feed = frames(&[
            (false, 20),
            (true, 30),
            (false, 50),
            (true, 40),
            (false, 30),
        ]);
        let mut engine = FakeEngine {
            streaming: true,
            ..FakeEngine::default()
        };
        let events = run_feed(feed, &mut engine).unwrap();
        // Prima Frase: 300 ms di prefill prima del parlato (frame 20) e 24 frame di hangover dopo
        // l'ultimo (49). Seconda: parlato dal frame 100 al 139.
        assert_eq!(times(&events), [(0, 300, 2220), (1, 2700, 4920)]);
        // Il primo Parziale arriva dopo 10 frame letti dal motore.
        let first_partial = events.iter().find_map(|event| match event {
            PipelineEvent::Partial {
                inizio_ms, fine_ms, ..
            } => Some((*inizio_ms, *fine_ms)),
            _ => None,
        });
        assert_eq!(first_partial, Some((300, 600)));
    }

    #[test]
    fn la_chiusura_della_frase_la_chiude_come_a_fine_parlato() {
        let mut feed = frames(&[(false, 10), (true, 30)]);
        feed.push(Ok(Feed::ClosePhrase));
        feed.push(Ok(Feed::Progress(Some(50))));
        feed.extend(frames(&[(true, 30), (false, 50)]));
        let mut engine = FakeEngine::default();
        let events = run_feed(feed, &mut engine).unwrap();
        // La prima Frase finisce alla chiusura, la seconda riparte da lì senza prefill.
        assert_eq!(times(&events), [(0, 0, 1200), (1, 1200, 2820)]);
        assert_eq!(engine.samples[0], 40 * FRAME_SAMPLES);
        // Il progresso della fonte passa com'è.
        assert!(events.contains(&PipelineEvent::Progress(Some(50))));
    }

    #[test]
    fn un_errore_della_fonte_ferma_la_pipeline() {
        let mut feed = frames(&[(true, 20)]);
        feed.push(Err(AppError::Internal("fonte".into())));
        feed.extend(frames(&[(true, 20)]));
        let error = run_feed(feed, &mut FakeEngine::default()).unwrap_err();
        assert!(matches!(error, AppError::Internal(_)), "{error:?}");
    }

    /// Smoke test con Silero e i tre modelli veri, in `%APPDATA%\it.sbobino.desktop\models`.
    #[test]
    #[ignore = "richiede i tre modelli scaricati (Impostazioni → Trascrizione)"]
    fn i_tre_modelli_trascrivono_il_parlato_italiano() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir =
            PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.sbobino.desktop/models");
        let mut detector =
            crate::audio_toolkit::vad::Silero::new(&root.join("resources/silero_vad.onnx"))
                .unwrap();
        for model in crate::managers::models::catalog() {
            let mut engine =
                super::super::transcribe_cpp::TranscribeCpp::load(&model.path(&dir)).unwrap();
            println!("{}: lingue {:?}", model.id, engine.languages());
            // Lo stesso parlato in WAV e nel video MP4/AAC, una volta con la lingua indicata.
            for (name, language) in [("parlato-it.wav", Some("it")), ("parlato-it.mp4", None)] {
                let cancel = CancelToken::new();
                engine.set_cancel_token(&cancel);
                let (mut phrases, mut partials) = (Vec::new(), Vec::new());
                transcribe_file(
                    &fixture(name),
                    &mut engine,
                    &mut detector,
                    language,
                    &cancel,
                    &mut |event| match event {
                        PipelineEvent::Phrase { id, text, .. } => {
                            // I Parziali arrivano prima della loro Frase, mai dopo.
                            assert!(partials.iter().all(|(p, _)| *p <= id), "{partials:?}");
                            phrases.push(text);
                        }
                        PipelineEvent::Partial { id, text, .. } => {
                            assert_eq!(id as usize, phrases.len(), "{text}");
                            partials.push((id, text));
                        }
                        PipelineEvent::Progress(_) => {}
                    },
                )
                .unwrap();
                println!("{} {name}: Parziali {partials:?}", model.id);
                // Solo Nemotron è in streaming: gli altri trascrivono la Frase intera.
                assert_eq!(
                    !partials.is_empty(),
                    model.mode == crate::managers::models::Mode::Stream,
                    "{} {name}",
                    model.id
                );
                let text = phrases.join(" ").to_lowercase();
                println!("{} {name}: {text}", model.id);
                assert_eq!(phrases.len(), 2, "{} {name}: {phrases:?}", model.id);
                assert!(
                    text.contains("trascrizione") && text.contains("testo"),
                    "{} {name}: {text}",
                    model.id
                );
            }
        }
    }
}
