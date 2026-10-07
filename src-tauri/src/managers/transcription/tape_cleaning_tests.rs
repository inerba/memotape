use super::tests::{temp_dir, tone_ogg};
use super::*;
use crate::audio_toolkit::cleaning::TrattoPulizia;
use crate::audio_toolkit::processing::{AudioProcessor, tests::DelayedScale};
use crate::engine::pipeline::tests::{EnergyDetector, FakeEngine};

#[test]
fn protezione_tape_separato_fissa_profili_in_preparazione_e_conserva_audio_e_consumatori() {
    use crate::audio_toolkit::processing::Bypass;
    use crate::audio_toolkit::protection::{ProtectionTimeline, Sensibilita};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct WeakDetector;
    impl VoiceDetector for WeakDetector {
        fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError> {
            // Il detector finto segue le annotazioni, ignorando residui numerici di Opus.
            Ok(if frame.iter().any(|s| s.abs() > 1e-10) {
                0.45
            } else {
                0.0
            })
        }
        fn reset(&mut self) {}
    }
    let (path, mut document) = mixed_tape("memotape-protection-separate");
    let voice = path.parent().unwrap().join("voice.ogg");
    let silence = path.parent().unwrap().join("silence.ogg");
    tone_ogg(&silence, &[(3.0, false)]);
    document.modalita = tape::Modalita::IngressiSeparati;
    std::fs::remove_file(&path).unwrap();
    tape::write(
        &path,
        &[
            (Ingresso::Mix, &voice),
            (Ingresso::Microfono, &silence),
            (Ingresso::Sistema, &voice),
        ],
        &document,
        Some(&[0.5]),
    )
    .unwrap();
    let before: Vec<_> = [Ingresso::Mix, Ingresso::Microfono, Ingresso::Sistema]
        .into_iter()
        .map(|i| samples(&path, i))
        .collect();
    let timelines = [ProtectionTimeline::default(), ProtectionTimeline::default()];
    let changed = Arc::new(AtomicBool::new(false));
    let prepared = TapeAudio::prepare(
        &path,
        &document,
        96,
        &CleaningLog::default(),
        &CancelToken::new(),
        |ingresso| {
            let changed = changed.clone();
            let index = usize::from(ingresso == Ingresso::Sistema);
            timelines[index].capture(Box::new(Bypass), move || {
                if changed.load(Ordering::SeqCst) {
                    Sensibilita::Selettivo
                } else {
                    Sensibilita::Bilanciato
                }
            })
        },
    )
    .unwrap();
    // Queste richieste successive non possono reinterpretare l'audio preparato.
    changed.store(true, Ordering::SeqCst);
    let mut phrases = Vec::new();
    for (index, ingresso) in [Ingresso::Microfono, Ingresso::Sistema]
        .into_iter()
        .enumerate()
    {
        let mut engine = FakeEngine {
            result: Some(String::from("Grazie. Thank you.").into()),
            ..FakeEngine::default()
        };
        let duration = transcribe_decoded(
            Decoder::open_ingresso(prepared.source(ingresso), ingresso).unwrap(),
            &mut engine,
            &mut WeakDetector,
            Some("it"),
            None,
            pipeline::FileAudio {
                copy: None,
                processor: Box::new(Bypass),
                protection: timelines[index].clone(),
            },
            &CancelToken::new(),
            &mut |event| {
                if let PipelineEvent::Phrase {
                    id,
                    inizio_ms,
                    fine_ms,
                    text,
                    tempi,
                } = event
                {
                    phrases.push(tape::Frase {
                        id,
                        inizio_ms,
                        fine_ms,
                        testo: text,
                        tempi,
                        ingresso,
                        parlante: None,
                        parlante_non_determinato: false,
                        parlante_provvisorio: false,
                        testo_corretto: false,
                        parlante_corretto: false,
                    });
                }
            },
        )
        .unwrap();
        assert_eq!(duration, 3000);
        assert_eq!(engine.samples.is_empty(), ingresso == Ingresso::Microfono);
    }
    assert_eq!(phrases.len(), 1);
    assert_eq!(phrases[0].ingresso, Ingresso::Sistema);
    document.frasi = phrases;
    prepared
        .commit(&path, &document, &CancelToken::new())
        .unwrap();
    for (index, ingresso) in [Ingresso::Mix, Ingresso::Microfono, Ingresso::Sistema]
        .into_iter()
        .enumerate()
    {
        assert_eq!(samples(&path, ingresso), before[index]);
    }
    let opened = open_tape(&path).unwrap();
    assert_eq!(opened.phrases.len(), 1);
    assert_eq!(opened.phrases[0].text, "Grazie. Thank you.");
    assert_eq!(tape::forma_onda(&path), Some(vec![0.5]));
    let mut library = Library::open(
        path.parent().unwrap(),
        &path.parent().unwrap().join("indice.sqlite"),
    )
    .unwrap();
    library.sync().unwrap();
    assert_eq!(library.search("Grazie", None).unwrap().len(), 1);
    let (_, audio) = crate::player::read(&path, None).unwrap();
    assert_eq!(audio, std::fs::read(&voice).unwrap());
}

fn samples(path: &Path, ingresso: Ingresso) -> Vec<f32> {
    let mut decoder = Decoder::open_ingresso(path, ingresso).unwrap();
    let mut pcm = Vec::new();
    while let Some(block) = decoder.next_block().unwrap() {
        pcm.extend(block.samples);
    }
    pcm
}

fn mixed_tape(name: &str) -> (PathBuf, tape::Document) {
    let dir = temp_dir(name);
    let ogg = dir.join("voice.ogg");
    tone_ogg(&ogg, &[(3.0, true)]);
    let path = dir.join("Documento.tape");
    let document = tape::Document::new(
        "2026-10-06T12:00:00+02:00".into(),
        3000,
        tape::Modalita::Mix,
        None,
        SpeechLanguage::from("it"),
        false,
        &[],
    );
    tape::write(&path, &[(Ingresso::Mix, &ogg)], &document, None).unwrap();
    (path, document)
}

fn prepare(
    path: &Path,
    document: &tape::Document,
    enabled: bool,
    cancel: &CancelToken,
) -> Result<TapeAudio, AppError> {
    let log = CleaningLog::default();
    TapeAudio::prepare(path, document, 96, &log, cancel, |ingresso| {
        Box::new(
            ConfiguredCleaning::with_factory(
                Box::new(move || enabled),
                ingresso,
                log.clone(),
                Box::new(|_| Ok(Box::new(DelayedScale::new(160)))),
            )
            .reuse(document.pulizia_audio.clone()),
        )
    })
}

fn transcribe_prepared(
    prepared: &TapeAudio,
    old: &tape::Document,
    ingresso: Ingresso,
    engine: &mut FakeEngine,
    cancel: &CancelToken,
) -> Result<(tape::Document, Vec<f32>), AppError> {
    let mut document = old.clone();
    document.frasi.clear();
    document.correzioni_testo.clear();
    document.parlanti.clear();
    let mut pcm = Vec::new();
    transcribe_decoded(
        Decoder::open_ingresso(prepared.source(ingresso), ingresso)?,
        engine,
        &mut EnergyDetector,
        Some("it"),
        Some(&mut pcm),
        None,
        cancel,
        &mut |event| {
            if let PipelineEvent::Phrase {
                id,
                inizio_ms,
                fine_ms,
                text,
                tempi,
            } = event
            {
                document.frasi.push(tape::Frase {
                    id,
                    inizio_ms,
                    fine_ms,
                    testo: text,
                    tempi,
                    ingresso,
                    parlante: None,
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                    testo_corretto: false,
                    parlante_corretto: false,
                });
            }
        },
    )?;
    document.completa = true;
    Ok((document, pcm))
}

fn rmse(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    (a.iter().zip(b).map(|(a, b)| (a - b).powi(2)).sum::<f32>() / a.len() as f32).sqrt()
}

#[test]
fn protezione_bilanciata_senza_frasi_sostituisce_vecchio_testo_copia_e_ricerca() {
    use crate::audio_toolkit::decode::Block;
    use crate::audio_toolkit::processing::Bypass;
    use crate::audio_toolkit::protection::{ProtectionTimeline, Sensibilita};
    struct NoiseDetector;
    impl VoiceDetector for NoiseDetector {
        fn probability(&mut self, _: &[f32]) -> Result<f32, AppError> {
            Ok(0.6)
        }
        fn reset(&mut self) {}
    }
    let (path, mut document) = mixed_tape("memotape-protection-no-speech");
    let ogg = path.parent().unwrap().join("noise.ogg");
    let mut copy = OggCopy::new(std::fs::File::create_new(&ogg).unwrap(), 16000, 1, 96).unwrap();
    copy.push(&Block {
        rate: 16000,
        channels: 1,
        samples: (0..48000)
            .map(|i| 0.08 * (i as f32 * std::f32::consts::TAU * 6000.0 / 16000.0).sin())
            .collect(),
    })
    .unwrap();
    copy.finish().unwrap();
    document.frasi = vec![tape::Frase {
        id: 0,
        inizio_ms: 0,
        fine_ms: 3000,
        testo: "falsafraseprecedente".into(),
        ingresso: Ingresso::Mix,
        parlante: None,
        parlante_non_determinato: false,
        parlante_provvisorio: false,
        tempi: vec![],
        testo_corretto: false,
        parlante_corretto: false,
    }];
    std::fs::remove_file(&path).unwrap();
    tape::write(&path, &[(Ingresso::Mix, &ogg)], &document, Some(&[0.08])).unwrap();
    let mut library = Library::open(
        path.parent().unwrap(),
        &path.parent().unwrap().join("indice.sqlite"),
    )
    .unwrap();
    library.sync().unwrap();
    assert_eq!(
        library.search("falsafraseprecedente", None).unwrap().len(),
        1
    );
    let timeline = ProtectionTimeline::default();
    let prepared = TapeAudio::prepare(
        &path,
        &document,
        96,
        &CleaningLog::default(),
        &CancelToken::new(),
        |_| timeline.capture(Box::new(Bypass), || Sensibilita::Bilanciato),
    )
    .unwrap();
    let mut engine = FakeEngine::default();
    let mut accepted = Vec::new();
    let duration = transcribe_decoded(
        Decoder::open_ingresso(prepared.source(Ingresso::Mix), Ingresso::Mix).unwrap(),
        &mut engine,
        &mut NoiseDetector,
        Some("it"),
        None,
        pipeline::FileAudio {
            copy: None,
            processor: Box::new(Bypass),
            protection: timeline,
        },
        &CancelToken::new(),
        &mut |event| {
            if matches!(event, PipelineEvent::Phrase { .. }) {
                accepted.push(event)
            }
        },
    )
    .unwrap();
    assert!(accepted.is_empty());
    assert!(engine.samples.is_empty());
    assert_eq!(duration, 3000);
    document.frasi.clear();
    document.completa = true;
    prepared
        .commit(&path, &document, &CancelToken::new())
        .unwrap();
    let reopened = open_tape(&path).unwrap();
    assert!(reopened.phrases.is_empty());
    assert!(tape::read(&path).unwrap().frasi.is_empty());
    assert!(
        !tape_text(&path, &Settings::default(), CopiaCome::Testo)
            .unwrap()
            .contains("falsafraseprecedente")
    );
    assert!(
        !tape_text(&path, &Settings::default(), CopiaCome::Markdown)
            .unwrap()
            .contains("falsafraseprecedente")
    );
    library.sync().unwrap();
    assert!(
        library
            .search("falsafraseprecedente", None)
            .unwrap()
            .is_empty()
    );
    let (_, audio) = crate::player::read(&path, None).unwrap();
    assert_eq!(audio, std::fs::read(&ogg).unwrap());
    assert_eq!(tape::forma_onda(&path), Some(vec![0.08]));
}

#[test]
fn tape_misto_parziale_pulisce_solo_i_tratti_liberi_e_aggiorna_i_consumatori() {
    let (path, mut document) = mixed_tape("memotape-partial-mix");
    document.origine = Some("vecchia.wav".into());
    document.frasi = vec![tape::Frase {
        id: 0,
        inizio_ms: 0,
        fine_ms: 3000,
        testo: "precedente".into(),
        ingresso: Ingresso::Mix,
        parlante: None,
        parlante_non_determinato: false,
        parlante_provvisorio: false,
        tempi: vec![],
        testo_corretto: false,
        parlante_corretto: false,
    }];
    document.pulizia_audio = vec![TrattoPulizia {
        ingresso: Ingresso::Mix,
        algoritmo: "altro".into(),
        versione: "storica".into(),
        frequenza: 16000,
        inizio_frame: 14403,
        fine_frame: 25607,
    }];
    tape::rewrite(&path, &document).unwrap();
    let before = std::fs::read(&path).unwrap();
    let original = samples(&path, Ingresso::Mix);
    let mut library = Library::open(
        path.parent().unwrap(),
        &path.parent().unwrap().join("indice.sqlite"),
    )
    .unwrap();
    library.sync().unwrap();
    assert_eq!(library.search("precedente", None).unwrap().len(), 1);
    let prepared = prepare(&path, &document, true, &CancelToken::new()).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let expected: Vec<_> = original
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            if (43209..76821).contains(&i) {
                s
            } else {
                s * 0.5
            }
        })
        .collect();
    let processed = samples(prepared.source(Ingresso::Mix), Ingresso::Mix);
    assert!(rmse(&processed, &expected) < 0.015);
    let mut engine = FakeEngine {
        result: Some(String::from("nuovorisultato").into()),
        ..FakeEngine::default()
    };
    let (new, analyzed) = transcribe_prepared(
        &prepared,
        &document,
        Ingresso::Mix,
        &mut engine,
        &CancelToken::new(),
    )
    .unwrap();
    assert!(!analyzed.is_empty());
    assert!(engine.pcm.iter().any(|s| s.abs() > 0.2));
    prepared.commit(&path, &new, &CancelToken::new()).unwrap();
    assert_eq!(samples(&path, Ingresso::Mix), processed);
    let reopened = tape::read(&path).unwrap();
    assert_eq!(reopened.origine, document.origine);
    assert_eq!(reopened.creato, document.creato);
    assert_eq!(reopened.durata_ms, 3000);
    assert_eq!(reopened.pulizia_audio.len(), 3);
    assert_eq!(reopened.pulizia_audio[0], document.pulizia_audio[0]);
    assert_eq!(
        (
            reopened.pulizia_audio[1].inizio_frame,
            reopened.pulizia_audio[1].fine_frame
        ),
        (0, 43209)
    );
    assert_eq!(
        (
            reopened.pulizia_audio[2].inizio_frame,
            reopened.pulizia_audio[2].fine_frame
        ),
        (76821, 144000)
    );
    assert!(tape::forma_onda(&path).unwrap().iter().any(|p| *p > 0.45));
    assert!(
        open_tape(&path).unwrap().phrases[0]
            .text
            .contains("nuovorisultato")
    );
    assert!(
        tape_text(&path, &Settings::default(), CopiaCome::Markdown)
            .unwrap()
            .contains("nuovorisultato")
    );
    library.sync().unwrap();
    assert!(library.search("precedente", None).unwrap().is_empty());
    assert_eq!(library.search("nuovorisultato", None).unwrap().len(), 1);
    let (_, played) = crate::player::read(&path, None).unwrap();
    assert!(!played.is_empty());
    // Ogni frame è ormai coperto, anche con frequenze diverse nei metadati.
    let second = prepare(&path, &reopened, true, &CancelToken::new()).unwrap();
    assert_eq!(second.source(Ingresso::Mix), path);
    second
        .commit(&path, &reopened, &CancelToken::new())
        .unwrap();
    assert_eq!(samples(&path, Ingresso::Mix), processed);
    assert_eq!(
        tape::read(&path).unwrap().pulizia_audio,
        reopened.pulizia_audio
    );
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
}

#[test]
fn ritrascrizione_tape_pulito_riusa_ogg_e_sostituisce_testo_atomicamente() {
    let dir = temp_dir("memotape-reuse-tape");
    let ogg = dir.join("voice.ogg");
    tone_ogg(&ogg, &[(3.0, true)]);
    let path = dir.join("Documento.tape");
    let mut document = tape::Document::new(
        "2026-10-06T12:00:00+02:00".into(),
        3000,
        tape::Modalita::Mix,
        None,
        SpeechLanguage::from("it"),
        false,
        &[],
    );
    document.origine = Some("provenienza.wav".into());
    document.pulizia_audio = vec![crate::audio_toolkit::cleaning::TrattoPulizia {
        ingresso: Ingresso::Microfono,
        algoritmo: "deepfilternet3".into(),
        versione: "precedente".into(),
        frequenza: 16000,
        inizio_frame: 0,
        fine_frame: 48000,
    }];
    tape::write(&path, &[(Ingresso::Mix, &ogg)], &document, Some(&[0.2])).unwrap();
    let before = std::fs::read(&path).unwrap();
    let log = CleaningLog::default();
    let prepared = TapeAudio::prepare(
        &path,
        &document,
        96,
        &log,
        &CancelToken::new(),
        |ingresso| {
            Box::new(
                ConfiguredCleaning::with_factory(
                    Box::new(|| true),
                    ingresso,
                    log.clone(),
                    Box::new(|_| Ok(Box::new(DelayedScale::new(160)))),
                )
                .reuse(document.pulizia_audio.clone()),
            ) as Box<dyn AudioProcessor>
        },
    )
    .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(log.intervals().is_empty());
    assert_eq!(prepared.source(Ingresso::Mix), path);
    let mut new = document.clone();
    new.completa = true;
    prepared.commit(&path, &new, &CancelToken::new()).unwrap();
    let reread = tape::read(&path).unwrap();
    assert!(reread.completa);
    assert_eq!(reread.origine, document.origine);
    assert_eq!(reread.pulizia_audio, document.pulizia_audio);
    assert_eq!(tape::forma_onda(&path), Some(vec![0.2]));
    let mut mix = tape::Mix::open(&path).unwrap();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut mix, &mut bytes).unwrap();
    assert_eq!(bytes, std::fs::read(&ogg).unwrap());
    assert!(!dir.join(TEMP_FOLDER).exists());
}

#[test]
fn tape_separato_pulisce_il_microfono_riusa_il_sistema_e_ricostruisce_il_mix() {
    let (path, mut document) = mixed_tape("memotape-clean-separate");
    let ogg = path.parent().unwrap().join("voice.ogg");
    document.modalita = tape::Modalita::IngressiSeparati;
    std::fs::remove_file(&path).unwrap();
    tape::write(
        &path,
        &[
            (Ingresso::Mix, &ogg),
            (Ingresso::Microfono, &ogg),
            (Ingresso::Sistema, &ogg),
        ],
        &document,
        None,
    )
    .unwrap();
    let mic = samples(&path, Ingresso::Microfono);
    let sys = samples(&path, Ingresso::Sistema);
    let original_system = {
        let mut input = tape::Mix::open_ingresso(&path, Ingresso::Sistema).unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut input, &mut bytes).unwrap();
        bytes
    };
    let log = CleaningLog::default();
    let prepared = TapeAudio::prepare(
        &path,
        &document,
        96,
        &log,
        &CancelToken::new(),
        |ingresso| {
            Box::new(
                ConfiguredCleaning::with_factory(
                    Box::new(move || ingresso == Ingresso::Microfono),
                    ingresso,
                    log.clone(),
                    Box::new(|_| Ok(Box::new(DelayedScale::new(160)))),
                )
                .reuse(document.pulizia_audio.clone()),
            )
        },
    )
    .unwrap();
    let treated = samples(prepared.source(Ingresso::Microfono), Ingresso::Microfono);
    let mixed = samples(prepared.source(Ingresso::Mix), Ingresso::Mix);
    assert!(rmse(&treated, &mic.iter().map(|s| s * 0.5).collect::<Vec<_>>()) < 0.015);
    assert_eq!(prepared.source(Ingresso::Sistema), path);
    let expected: Vec<_> = mic
        .iter()
        .zip(&sys)
        .map(|(a, b)| (a * 0.5 + b).clamp(-1.0, 1.0))
        .collect();
    assert!(rmse(&mixed, &expected) < 0.025);
    let (mic_document, mic_analysis) = transcribe_prepared(
        &prepared,
        &document,
        Ingresso::Microfono,
        &mut FakeEngine::default(),
        &CancelToken::new(),
    )
    .unwrap();
    let (sys_document, _) = transcribe_prepared(
        &prepared,
        &document,
        Ingresso::Sistema,
        &mut FakeEngine::default(),
        &CancelToken::new(),
    )
    .unwrap();
    assert!(mic_analysis.len() >= 48000);
    document.frasi = mic_document.frasi;
    document.frasi.extend(sys_document.frasi);
    document.completa = true;
    prepared
        .commit(&path, &document, &CancelToken::new())
        .unwrap();
    assert_eq!(samples(&path, Ingresso::Mix), mixed);
    assert_eq!(samples(&path, Ingresso::Microfono), treated);
    assert_eq!(samples(&path, Ingresso::Sistema), sys);
    let (_, played) = crate::player::read(&path, None).unwrap();
    assert!(!played.is_empty());
    let mut input = tape::Mix::open_ingresso(&path, Ingresso::Sistema).unwrap();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut input, &mut bytes).unwrap();
    assert_eq!(bytes, original_system);
    assert_eq!(tape::read(&path).unwrap().pulizia_audio.len(), 1);
    assert_eq!(
        tape::read(&path).unwrap().pulizia_audio[0].ingresso,
        Ingresso::Microfono
    );
    assert!(open_tape(&path).unwrap().info.ingressi_separati);
    assert!(tape::forma_onda(&path).unwrap().iter().all(|s| *s > 0.65));
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
}

#[test]
fn annulla_e_guasti_in_decodifica_asr_e_sostituzione_conservano_il_tape() {
    let (path, document) = mixed_tape("memotape-clean-failures");
    let before = std::fs::read(&path).unwrap();
    let cancel = CancelToken::new();
    cancel.cancel();
    assert!(matches!(
        prepare(&path, &document, true, &cancel),
        Err(AppError::Cancelled)
    ));
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
    let log = CleaningLog::default();
    assert!(matches!(
        TapeAudio::prepare(&path, &document, 96, &log, &CancelToken::new(), |_| {
            Box::new(super::cleaning_tests::FailedProcessor)
        }),
        Err(AppError::AudioCleaningFailed(_))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
    let prepared = prepare(&path, &document, true, &CancelToken::new()).unwrap();
    let cancel = CancelToken::new();
    let mut engine = FakeEngine {
        cancel_at: Some((1, cancel.clone())),
        ..FakeEngine::default()
    };
    assert!(matches!(
        transcribe_prepared(&prepared, &document, Ingresso::Mix, &mut engine, &cancel),
        Err(AppError::Cancelled)
    ));
    drop(prepared);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
    let prepared = prepare(&path, &document, true, &CancelToken::new()).unwrap();
    assert_eq!(
        prepared.commit(&path, &document, &cancel),
        Err(AppError::Cancelled)
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
    // Errore reale del filesystem alla sostituzione su Windows: nessuna cancellazione del precedente.
    use std::os::windows::fs::OpenOptionsExt;
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&path)
        .unwrap();
    let prepared = prepare(&path, &document, true, &CancelToken::new()).unwrap();
    assert!(matches!(
        prepared.commit(&path, &document, &CancelToken::new()),
        Err(AppError::UnwritableFolder(_))
    ));
    drop(held);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
    assert_eq!(
        std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|f| f.path().extension().is_some_and(|s| s == "tmp"))
            .count(),
        0
    );
}

#[test]
fn un_tape_vecchio_spento_o_senza_parlato_non_inventa_tracce_e_non_perde_dati_aggiuntivi() {
    let (path, document) = mixed_tape("memotape-clean-old-tape");
    let original_audio = std::fs::read(path.parent().unwrap().join("voice.ogg")).unwrap();
    let mut json = serde_json::to_value(&document).unwrap();
    json.as_object_mut().unwrap().remove("pulizia_audio");
    json["dati_futuri"] = serde_json::json!({ "titolo": "da conservare" });
    {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mix.ogg", options).unwrap();
        zip.write_all(&original_audio).unwrap();
        zip.start_file("allegato.txt", options).unwrap();
        zip.write_all(b"dato aggiuntivo").unwrap();
        zip.start_file("trascrizione.json", options).unwrap();
        serde_json::to_writer(&mut zip, &json).unwrap();
        zip.finish().unwrap();
    }
    let original = std::fs::read(&path).unwrap();
    open_tape(&path).unwrap();
    crate::player::forma_onda(&path, 50, &Activity::default()).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let prepared = prepare(&path, &document, false, &CancelToken::new()).unwrap();
    let mut engine = FakeEngine {
        result: Some(String::new().into()),
        ..FakeEngine::default()
    };
    let (new, _) = transcribe_prepared(
        &prepared,
        &document,
        Ingresso::Mix,
        &mut engine,
        &CancelToken::new(),
    )
    .unwrap();
    prepared.commit(&path, &new, &CancelToken::new()).unwrap();
    assert!(tape::read(&path).unwrap().frasi.is_empty());
    assert!(tape::read(&path).unwrap().pulizia_audio.is_empty());
    assert!(!tape::has_ingressi(&path).unwrap());
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let names: Vec<_> = zip.file_names().collect();
    assert_eq!(names.len(), 3);
    let json: serde_json::Value =
        serde_json::from_reader(zip.by_name("trascrizione.json").unwrap()).unwrap();
    assert_eq!(json["dati_futuri"]["titolo"], "da conservare");
    let mut extra = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("allegato.txt").unwrap(), &mut extra).unwrap();
    assert_eq!(extra, "dato aggiuntivo");
    assert!(!path.parent().unwrap().join(TEMP_FOLDER).exists());
}

#[test]
#[ignore = "DFN3 e Nemotron reali, eseguire separatamente in sequenza"]
fn dfn3_ritrascrive_tape_parziale_con_parole_ai_confini_e_riusa_gli_ogg() {
    use crate::audio_toolkit::processing::{Boundary, Format, PcmStream};
    use crate::engine::pipeline::tests::fixture;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = root.join(deepfilter::MODEL_FILE);
    // Prima di Opus: tratti a frequenza non intera, stereo, con contesto reale e fase globale.
    let format = Format {
        rate: 44100,
        channels: 2,
    };
    let input: Vec<_> = (0..88200)
        .flat_map(|i| [0.2 * (i as f32 * 0.05).sin(), 0.1 * (i as f32 * 0.09).cos()])
        .collect();
    let interval = TrattoPulizia {
        ingresso: Ingresso::Mix,
        algoritmo: "dfn3".into(),
        versione: "precedente".into(),
        frequenza: 48000,
        inizio_frame: 12017,
        fine_frame: 36239,
    };
    let protected = 11040..33295; // floor(12017*441/480)..ceil(36239*441/480)
    let run = |chunk: usize| {
        let log = CleaningLog::default();
        let processor = ConfiguredCleaning::new(model.clone(), || true, Ingresso::Mix, log.clone())
            .reuse(vec![interval.clone()]);
        let mut stream = PcmStream::new(format, Box::new(processor)).unwrap();
        let mut out = Vec::new();
        for part in input.chunks(chunk * 2) {
            out.extend(stream.push(part).unwrap().samples);
        }
        out.extend(stream.boundary(Boundary::Finish).unwrap().samples);
        assert_eq!(out.len(), input.len());
        assert_eq!(
            out[protected.start * 2..protected.end * 2],
            input[protected.start * 2..protected.end * 2]
        );
        out
    };
    let a = run(997);
    let b = run(4410);
    assert!(
        rmse(&a, &b) < 1e-6,
        "i blocchi non devono cambiare fase o contesto"
    );
    println!(
        "DFN3 PCM stereo 44.1 kHz: 88200 frame, tratti riusati invariati, RMSE fra partizioni={}",
        rmse(&a, &b)
    );

    let dir = temp_dir("memotape-dfn3-retranscribe");
    let settings = Settings::default();
    let path = file_to_tape(&dir, &dir, &fixture("parlato-it.wav"), &settings, |copy| {
        let mut decoder = Decoder::open(&fixture("parlato-it.wav"))?;
        let mut copy = copy;
        while let Some(block) = decoder.next_block()? {
            copy.push(&block)?;
        }
        copy.finish()?;
        let mut document = tape::Document::new(
            "2026-10-06T12:00:00+02:00".into(),
            8960,
            tape::Modalita::Mix,
            None,
            SpeechLanguage::from("it"),
            false,
            &[],
        );
        document.frasi.push(tape::Frase {
            id: 0,
            inizio_ms: 0,
            fine_ms: 8960,
            testo: "precedente".into(),
            ingresso: Ingresso::Mix,
            parlante: None,
            parlante_non_determinato: false,
            parlante_provvisorio: false,
            tempi: vec![],
            testo_corretto: false,
            parlante_corretto: false,
        });
        Ok(document)
    })
    .unwrap()
    .unwrap();
    let old = tape::read(&path).unwrap();
    let log = CleaningLog::default();
    let first = TapeAudio::prepare(&path, &old, 96, &log, &CancelToken::new(), |ingresso| {
        let mut decoded_blocks = 0;
        Box::new(
            ConfiguredCleaning::new(
                model.clone(),
                move || {
                    decoded_blocks += 1;
                    decoded_blocks < 83
                },
                ingresso,
                log.clone(),
            )
            .reuse(vec![]),
        )
    })
    .unwrap();
    first.commit(&path, &old, &CancelToken::new()).unwrap();
    let partial = tape::read(&path).unwrap();
    assert!(!partial.pulizia_audio.is_empty());
    assert!(partial.pulizia_audio.last().unwrap().fine_frame < 430080);
    let log = CleaningLog::default();
    let prepared = TapeAudio::prepare(&path, &partial, 96, &log, &CancelToken::new(), |ingresso| {
        Box::new(
            ConfiguredCleaning::new(model.clone(), || true, ingresso, log.clone())
                .reuse(partial.pulizia_audio.clone()),
        )
    })
    .unwrap();
    let model_info = models::default_model();
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let mut engine = TranscribeCpp::load(&model_info.path(&models_dir)).unwrap();
    let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
    let mut new = partial.clone();
    new.frasi.clear();
    transcribe_decoded(
        Decoder::open(prepared.source(Ingresso::Mix)).unwrap(),
        &mut engine,
        &mut detector,
        Some("it"),
        None,
        None,
        &CancelToken::new(),
        &mut |event| {
            if let PipelineEvent::Phrase {
                id,
                inizio_ms,
                fine_ms,
                text,
                tempi,
            } = event
            {
                new.frasi.push(tape::Frase {
                    id,
                    inizio_ms,
                    fine_ms,
                    testo: text,
                    tempi,
                    ingresso: Ingresso::Mix,
                    parlante: None,
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                    testo_corretto: false,
                    parlante_corretto: false,
                });
            }
        },
    )
    .unwrap();
    let text = new
        .frasi
        .iter()
        .map(|f| f.testo.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let words: String = text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    assert_eq!(
        words,
        "buongiornoatuttioggiparliamoditrascrizioneilcomputertrasformalavoceintesto"
    );
    println!("DFN3 Tape parziale → completo, {}: {text}", model_info.id);
    prepared.commit(&path, &new, &CancelToken::new()).unwrap();
    let before = crate::player::read(&path, None).unwrap().1;
    let complete = tape::read(&path).unwrap();
    let log = CleaningLog::default();
    let second = TapeAudio::prepare(
        &path,
        &complete,
        96,
        &log,
        &CancelToken::new(),
        |ingresso| {
            Box::new(
                ConfiguredCleaning::new(model.clone(), || true, ingresso, log.clone())
                    .reuse(complete.pulizia_audio.clone()),
            )
        },
    )
    .unwrap();
    assert!(log.intervals().is_empty());
    second
        .commit(&path, &complete, &CancelToken::new())
        .unwrap();
    assert_eq!(crate::player::read(&path, None).unwrap().1, before);
    assert_eq!(
        tape::read(&path).unwrap().pulizia_audio,
        complete.pulizia_audio
    );
    assert!(!dir.join(TEMP_FOLDER).exists());
    println!("Seconda Trascrizione: Ogg byte-identico, nessun nuovo tratto DFN3");
}
