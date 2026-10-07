use super::*;
use crate::audio_toolkit::ogg_opus::tests::{decoded_seconds, temp_dir};
use crate::audio_toolkit::processing::{AudioProcessor, Boundary, PcmBlock, tests::DelayedScale};
use crate::engine::pipeline::tests::{EnergyDetector, FakeEngine, fixture};

fn import(
    library: &Path,
    source: &Path,
    processor: Box<dyn AudioProcessor>,
    log: &CleaningLog,
    engine: &mut dyn TranscriptionEngine,
    detector: &mut dyn VoiceDetector,
    cancel: &CancelToken,
) -> Result<Option<PathBuf>, AppError> {
    let settings = Settings::default();
    file_to_tape(library, library, source, &settings, |copy| {
        let mut phrases = Vec::new();
        let duration = transcribe_decoded(
            Decoder::open(source)?,
            engine,
            detector,
            Some("it"),
            None,
            pipeline::FileAudio {
                copy: Some(copy),
                processor,
                protection: Default::default(),
            },
            cancel,
            &mut |event| {
                if let PipelineEvent::Phrase {
                    inizio_ms,
                    fine_ms,
                    text,
                    tempi,
                    ..
                } = event
                {
                    phrases.push(Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        tempi,
                        ingresso: Ingresso::Mix,
                        parlante: None,
                        parlante_non_determinato: false,
                        parlante_provvisorio: false,
                    });
                }
            },
        )?;
        let mut document = tape::Document::new(
            "2026-10-06T12:00:00+02:00".into(),
            duration,
            tape::Modalita::Mix,
            None,
            SpeechLanguage::from("it"),
            true,
            &phrases,
        );
        document.pulizia_audio = log.intervals();
        Ok(document)
    })
}

#[test]
fn importazione_elaborata_riapre_un_solo_audio_con_metadati_testo_e_forma_onda() {
    let library = temp_dir("memotape-cleaning-import");
    let source = fixture("parlato-it.wav");
    let original = std::fs::read(&source).unwrap();
    let log = CleaningLog::default();
    let processor = ConfiguredCleaning::with_factory(
        Box::new(|| true),
        Ingresso::Mix,
        log.clone(),
        Box::new(|_| Ok(Box::new(DelayedScale::new(160)))),
    );
    let mut engine = FakeEngine::default();
    let path = import(
        &library,
        &source,
        Box::new(processor),
        &log,
        &mut engine,
        &mut EnergyDetector,
        &CancelToken::new(),
    )
    .unwrap()
    .unwrap();
    let document = tape::read(&path).unwrap();
    assert!(!document.frasi.is_empty());
    assert_eq!(document.pulizia_audio, log.intervals());
    assert_eq!(document.pulizia_audio.len(), 1);
    assert_eq!(document.pulizia_audio[0].inizio_frame, 0);
    assert_eq!(document.pulizia_audio[0].ingresso, Ingresso::Mix);
    assert!((decoded_seconds(&path) - decoded_seconds(&source)).abs() < 0.0001);
    assert!(!open_tape(&path).unwrap().phrases.is_empty());
    assert!(tape::forma_onda(&path).is_some());
    let zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut names: Vec<_> = zip.file_names().collect();
    names.sort_unstable();
    assert_eq!(names, ["forma-onda.json", "mix.ogg", "trascrizione.json"]);
    assert!(!library.join(TEMP_FOLDER).exists());
    assert_eq!(std::fs::read(&source).unwrap(), original);
    let mut old = serde_json::to_value(document).unwrap();
    old.as_object_mut().unwrap().remove("pulizia_audio");
    assert!(
        serde_json::from_value::<tape::Document>(old)
            .unwrap()
            .pulizia_audio
            .is_empty()
    );
}

pub(super) struct FailedProcessor;
impl AudioProcessor for FailedProcessor {
    fn max_pending_frames(&self) -> usize {
        0
    }
    fn process(&mut self, _block: PcmBlock<'_>, _out: &mut Vec<f32>) -> Result<(), AppError> {
        Err(AppError::AudioCleaningFailed("guasto di prova".into()))
    }
    fn flush(&mut self, _boundary: Boundary, _out: &mut Vec<f32>) -> Result<(), AppError> {
        Ok(())
    }
}

#[test]
fn pulizia_guasta_annullata_o_senza_parlato_non_lascia_tape_o_temporanei() {
    let source = fixture("parlato-it.wav");
    let library = temp_dir("memotape-cleaning-failed");
    let log = CleaningLog::default();
    let error = import(
        &library,
        &source,
        Box::new(FailedProcessor),
        &log,
        &mut FakeEngine::default(),
        &mut EnergyDetector,
        &CancelToken::new(),
    )
    .unwrap_err();
    assert!(matches!(error, AppError::AudioCleaningFailed(_)));
    assert_eq!(std::fs::read_dir(&library).unwrap().count(), 0);
    let cancel = CancelToken::new();
    let mut engine = FakeEngine {
        cancel_at: Some((1, cancel.clone())),
        ..FakeEngine::default()
    };
    assert_eq!(
        import(
            &library,
            &source,
            Box::new(DelayedScale::new(160)),
            &log,
            &mut engine,
            &mut EnergyDetector,
            &cancel
        ),
        Err(AppError::Cancelled)
    );
    assert_eq!(std::fs::read_dir(&library).unwrap().count(), 0);
    let mut engine = FakeEngine {
        result: Some(String::new().into()),
        ..FakeEngine::default()
    };
    assert_eq!(
        import(
            &library,
            &source,
            Box::new(DelayedScale::new(160)),
            &log,
            &mut engine,
            &mut EnergyDetector,
            &CancelToken::new()
        )
        .unwrap(),
        None
    );
    assert_eq!(std::fs::read_dir(&library).unwrap().count(), 0);
}

#[test]
#[ignore = "DFN3, Silero e ASR reali; eseguire separatamente e in sequenza"]
fn dfn3_importa_audio_e_video_con_tutti_i_motori_e_riapre_i_tape() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    for model in models::catalog()
        .iter()
        .filter(|m| m.kind == models::ModelKind::Trascrizione)
    {
        let mut engine = TranscribeCpp::load(&model.path(&models_dir)).unwrap();
        for name in ["parlato-it.wav", "parlato-it.mp4"] {
            let source = fixture(name);
            let library = temp_dir(&format!("memotape-dfn3-{}-{name}", model.id));
            let log = CleaningLog::default();
            let processor = ConfiguredCleaning::new(
                std::env::var_os("MEMOTAPE_DFN3_BUNDLE")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root.join(deepfilter::MODEL_FILE)),
                || true,
                Ingresso::Mix,
                log.clone(),
            );
            let mut silero = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
            let path = import(
                &library,
                &source,
                Box::new(processor),
                &log,
                &mut engine,
                &mut silero,
                &CancelToken::new(),
            )
            .unwrap()
            .unwrap();
            let document = tape::read(&path).unwrap();
            assert!(!document.frasi.is_empty());
            assert!(!document.pulizia_audio.is_empty());
            assert!(tape::forma_onda(&path).is_some());
            assert!((decoded_seconds(&path) - decoded_seconds(&source)).abs() < 0.0001);
            println!(
                "DFN3 {} {name}: {}",
                model.id,
                document
                    .frasi
                    .iter()
                    .map(|f| f.testo.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            assert!(!open_tape(&path).unwrap().phrases.is_empty());
        }
    }
}

#[test]
#[ignore = "corpus sintetico DFN3/Silero/tre ASR: richiede MEMOTAPE_DFN3_SHORT, eseguire in sequenza"]
fn dfn3_corpus_prudente_confronta_pulizia_accesa_e_spenta() {
    use crate::audio_toolkit::processing::Bypass;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = temp_dir("memotape-dfn3-corpus");
    let mut decoder = Decoder::open(&fixture("parlato-it.wav")).unwrap();
    let mut voice = Vec::new();
    while let Some(block) = decoder.next_block().unwrap() {
        assert_eq!(block.rate, 16_000);
        voice.extend(block.mono());
    }
    let mut seed = 42_u32;
    let noise: Vec<f32> = (0..32_000)
        .map(|_| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32 * 0.08
        })
        .collect();
    let mut cases = Vec::new();
    for (name, samples) in [
        ("silenzio", vec![0.0; 32_000]),
        ("rumore-continuo", noise.clone()),
        (
            "rumore-intermittente",
            noise
                .iter()
                .enumerate()
                .map(|(i, &x)| if i % 8000 < 2000 { x } else { 0.0 })
                .collect(),
        ),
        // Surrogato esplicitamente sintetico; non sostituisce un respiro umano registrato.
        (
            "soffio-sintetico",
            noise
                .iter()
                .enumerate()
                .map(|(i, &x)| x * (std::f32::consts::PI * i as f32 / 32_000.0).sin().powi(4))
                .collect(),
        ),
        (
            "voce-attenuata-30db",
            voice.iter().map(|x| x * 0.031622776).collect(),
        ),
    ] {
        let path = directory.join(format!("{name}.wav"));
        let count = (samples.len() * 4) as u32;
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&(36 + count).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&3_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&16_000_u32.to_le_bytes());
        bytes.extend_from_slice(&64_000_u32.to_le_bytes());
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&32_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&count.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        std::fs::write(&path, bytes).unwrap();
        cases.push((name, path));
    }
    cases.push((
        "si-no",
        PathBuf::from(
            std::env::var("MEMOTAPE_DFN3_SHORT")
                .expect("generare si-no.wav con verification-02/generate-corpus.ps1"),
        ),
    ));
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    for model in models::catalog()
        .iter()
        .filter(|m| m.kind == models::ModelKind::Trascrizione)
    {
        let mut engine = TranscribeCpp::load(&model.path(&models_dir)).unwrap();
        for (name, source) in &cases {
            if std::env::var("MEMOTAPE_DFN3_ONLY")
                .ok()
                .as_deref()
                .is_some_and(|only| only != *name)
            {
                continue;
            }
            for enabled in [false, true] {
                let log = CleaningLog::default();
                let processor: Box<dyn AudioProcessor> = if enabled {
                    Box::new(ConfiguredCleaning::new(
                        root.join(deepfilter::MODEL_FILE),
                        || true,
                        Ingresso::Mix,
                        log.clone(),
                    ))
                } else {
                    Box::new(Bypass)
                };
                let library = temp_dir(&format!("memotape-corpus-{}-{name}-{enabled}", model.id));
                let mut silero = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
                let path = import(
                    &library,
                    source,
                    processor,
                    &log,
                    &mut engine,
                    &mut silero,
                    &CancelToken::new(),
                )
                .unwrap();
                let text = if let Some(path) = path {
                    assert!((decoded_seconds(&path) - decoded_seconds(source)).abs() < 0.0001);
                    tape::read(&path)
                        .unwrap()
                        .frasi
                        .iter()
                        .map(|f| f.testo.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    "[nessun parlato]".into()
                };
                println!("CORPUS {} {name} pulizia={enabled}: {text}", model.id);
                if *name == "voce-attenuata-30db" {
                    let words: String = text
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .flat_map(char::to_lowercase)
                        .collect();
                    assert_eq!(
                        words,
                        "buongiornoatuttioggiparliamoditrascrizioneilcomputertrasformalavoceintesto",
                        "voce bassa {} pulizia={enabled}: {text}",
                        model.id
                    );
                }
                if *name == "si-no" {
                    let words: String = text
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .flat_map(char::to_lowercase)
                        .collect::<String>()
                        .replace('ì', "i");
                    if model.id.starts_with("parakeet") {
                        // Il motore trascrive già l'originale come "C No"; conservarne i due termini.
                        assert!(
                            words == "cno" || words == "sino",
                            "risposte brevi {} pulizia={enabled}: {text}",
                            model.id
                        );
                    } else {
                        assert_eq!(
                            words, "sino",
                            "risposte brevi {} pulizia={enabled}: {text}",
                            model.id
                        );
                    }
                }
                if *name == "silenzio" {
                    assert_eq!(text, "[nessun parlato]");
                }
            }
        }
    }
}
