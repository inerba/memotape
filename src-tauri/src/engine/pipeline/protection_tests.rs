use super::tests::FakeEngine;
use super::*;
use crate::audio_toolkit::protection::{ProtectionTimeline, Sensibilita};
use crate::audio_toolkit::vad::Silero;
use std::path::{Path, PathBuf};

fn cuts(
    frames: Vec<Feed>,
    detector: &mut dyn VoiceDetector,
    timeline: &ProtectionTimeline,
) -> Vec<(usize, Vec<Vec<f32>>)> {
    let shared = Shared {
        progress: AtomicU8::new(0),
        stopped: AtomicBool::new(false),
    };
    let (sender, receiver) = mpsc::sync_channel(1000);
    cut_phrases(
        frames.into_iter().map(Ok),
        detector,
        None,
        &shared,
        &CancelToken::new(),
        &sender,
        timeline,
    );
    drop(sender);
    receiver
        .into_iter()
        .filter_map(|c| match c.unwrap() {
            Cut::Phrase { start, frames } => Some((start, frames)),
            Cut::End(_) => None,
        })
        .collect()
}

struct Probability(f32);
impl VoiceDetector for Probability {
    fn probability(&mut self, _: &[f32]) -> Result<f32, AppError> {
        Ok(self.0)
    }
    fn reset(&mut self) {}
}

#[test]
fn protezione_preserva_parole_deboli_scartando_rumore_senza_tagliare_pcm() {
    let noise = vec![
        Feed::Frame(
            (0..480)
                .map(|i| if i % 2 == 0 { 0.08 } else { -0.08 })
                .collect()
        );
        12
    ];
    let weak = vec![Feed::Frame((0..480).map(|i| (i as f32 * 0.1).sin() * 0.00001).collect()); 2];
    for level in [
        Sensibilita::Spento,
        Sensibilita::Sensibile,
        Sensibilita::Bilanciato,
        Sensibilita::Selettivo,
    ] {
        let timeline = ProtectionTimeline::default();
        timeline.record(0, level);
        assert_eq!(
            cuts(noise.clone(), &mut Probability(0.7), &timeline).len(),
            usize::from(level == Sensibilita::Spento)
        );
        assert_eq!(
            cuts(weak.clone(), &mut Probability(0.45), &timeline).len(),
            usize::from(level != Sensibilita::Selettivo)
        );
        assert_eq!(
            cuts(
                vec![Feed::Frame(vec![0.0; 480]); 20],
                &mut Probability(0.7),
                &timeline
            )
            .len(),
            usize::from(level == Sensibilita::Spento)
        );
    }
    // La destinazione audio riceve ogni campione anche quando nessuna candidata viene ammessa.
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parlato-it.wav");
    let timeline = ProtectionTimeline::default();
    timeline.record(0, Sensibilita::Selettivo);
    let mut pcm = Vec::new();
    let mut engine = FakeEngine::default();
    let duration = transcribe_decoded(
        Decoder::open(&source).unwrap(),
        &mut engine,
        &mut Probability(0.45),
        Some("it"),
        Some(&mut pcm),
        FileAudio {
            copy: None,
            processor: Box::new(Bypass),
            protection: timeline,
        },
        &CancelToken::new(),
        &mut |_| {},
    )
    .unwrap();
    assert!(engine.samples.is_empty());
    assert_eq!(duration, 8970);
    assert_eq!(pcm.len(), 143520);
}

#[test]
fn cambio_sensibilita_non_revoca_parola_in_corso_ne_frasi_accodate() {
    let timeline = ProtectionTimeline::default();
    timeline.record(0, Sensibilita::Sensibile);
    timeline.record(480 * 2, Sensibilita::Selettivo);
    let frame: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.00001).collect();
    let selected = cuts(
        vec![Feed::Frame(frame); 10],
        &mut Probability(0.45),
        &timeline,
    );
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].1.len(), 10);
    timeline.record(480 * 10, Sensibilita::Spento);
    let shared = Shared {
        progress: AtomicU8::new(0),
        stopped: AtomicBool::new(false),
    };
    let (sender, receiver) = mpsc::sync_channel(4);
    for (start, frames) in selected {
        sender.send(Ok(Cut::Phrase { start, frames })).unwrap();
    }
    sender.send(Ok(Cut::End(10))).unwrap();
    drop(sender);
    let mut engine = FakeEngine::default();
    let mut events = Vec::new();
    let duration = transcribe_phrases(
        receiver,
        &mut engine,
        Some("it"),
        Some(0),
        &shared,
        &CancelToken::new(),
        &mut |e| events.push(e),
    )
    .unwrap();
    assert_eq!(duration, 300);
    assert_eq!(engine.samples, [4800]);
    assert!(matches!(
        &events[0],
        PipelineEvent::Phrase {
            inizio_ms: 0,
            fine_ms: 300,
            ..
        }
    ));
}

#[test]
fn revisione_catturata_sul_blocco_prima_del_ritardo_del_processore() {
    use crate::audio_toolkit::processing::{Format, PcmBlock};
    struct Delay(Vec<f32>);
    impl AudioProcessor for Delay {
        fn max_pending_frames(&self) -> usize {
            480
        }
        fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
            out.append(&mut self.0);
            self.0.extend_from_slice(block.samples);
            Ok(())
        }
        fn flush(&mut self, _: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
            out.append(&mut self.0);
            Ok(())
        }
    }
    let timeline = ProtectionTimeline::default();
    let mut levels = [Sensibilita::Sensibile, Sensibilita::Selettivo].into_iter();
    let mut stream = PcmStream::new(
        Format {
            rate: 16000,
            channels: 1,
        },
        timeline.capture(Box::new(Delay(Vec::new())), move || levels.next().unwrap()),
    )
    .unwrap();
    assert!(stream.push(&[0.1; 480]).unwrap().samples.is_empty());
    assert_eq!(stream.push(&[0.2; 480]).unwrap().samples, [0.1; 480]);
    assert_eq!(timeline.level(0), Sensibilita::Sensibile);
    assert_eq!(timeline.level(480), Sensibilita::Selettivo);
    assert_eq!(
        stream.boundary(Boundary::Finish).unwrap().samples,
        [0.2; 480]
    );
}

#[test]
#[ignore = "corpus DFN3/Silero/tre ASR, release sequenziale, senza Impostazioni o Libreria"]
fn corpus_protezione_quattro_livelli_tre_asr() {
    use crate::audio_toolkit::cleaning::{CleaningLog, ConfiguredCleaning};
    use crate::audio_toolkit::deepfilter;
    use crate::engine::transcribe_cpp::TranscribeCpp;
    use crate::managers::models;
    use crate::transcript::Ingresso;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let corpus = root.join("../.scratch/pulizia-audio/verification-05/corpus");
    let levels = [
        Sensibilita::Spento,
        Sensibilita::Sensibile,
        Sensibilita::Bilanciato,
        Sensibilita::Selettivo,
    ];
    let mut cases = Vec::new();
    for name in [
        "silenzio",
        "rumore-continuo",
        "rumore-intermittente",
        "soffio-sintetico",
        "voce-attenuata-30db",
        "si-no",
    ] {
        for cleaning in [false, true] {
            let processor: Box<dyn AudioProcessor> = if cleaning {
                Box::new(ConfiguredCleaning::new(
                    root.join(deepfilter::MODEL_FILE),
                    || true,
                    Ingresso::Mix,
                    CleaningLog::default(),
                ))
            } else {
                Box::new(Bypass)
            };
            let frames: Vec<_> = FileFrames::with_processor(
                Decoder::open(&corpus.join(format!("{name}.wav"))).unwrap(),
                None,
                processor,
            )
            .map(Result::unwrap)
            .collect();
            let mut selections = Vec::new();
            for level in levels {
                let timeline = ProtectionTimeline::default();
                timeline.record(0, level);
                let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
                selections.push(cuts(frames.clone(), &mut detector, &timeline));
            }
            if ["voce-attenuata-30db", "si-no"].contains(&name) {
                assert_eq!(
                    selections[0], selections[1],
                    "regressione sensibile {name} pulizia={cleaning}"
                );
                assert_eq!(
                    selections[0], selections[2],
                    "regressione bilanciato {name} pulizia={cleaning}"
                );
            }
            cases.push((name, cleaning, selections));
        }
    }
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let mut report = Vec::new();
    let mut false_off = 0;
    let mut false_balanced = 0;
    for model in models::catalog()
        .iter()
        .filter(|m| m.kind == models::ModelKind::Trascrizione)
    {
        let mut engine = TranscribeCpp::load(&model.path(&models_dir)).unwrap();
        for (name, cleaning, selections) in &cases {
            let mut results = Vec::new();
            for (start, frames) in &selections[0] {
                let result = engine
                    .transcribe(&mut frames.clone().into_iter(), Some("it"), None)
                    .unwrap();
                results.push((*start, result.text));
            }
            for (index, level) in levels.iter().enumerate() {
                let texts: Vec<_> = selections[index]
                    .iter()
                    .filter_map(|(start, _)| {
                        results
                            .iter()
                            .find(|(s, _)| s == start)
                            .map(|(_, text)| text.as_str())
                    })
                    .filter(|t| !t.is_empty())
                    .collect();
                let speech = ["voce-attenuata-30db", "si-no"].contains(name);
                if !speech {
                    if index == 0 {
                        false_off += texts.len();
                    }
                    if index == 2 {
                        false_balanced += texts.len();
                    }
                }
                let lost = if speech {
                    results
                        .iter()
                        .filter(|(start, text)| {
                            !text.is_empty() && !selections[index].iter().any(|(s, _)| s == start)
                        })
                        .map(|(_, text)| text.split_whitespace().count())
                        .sum::<usize>()
                } else {
                    0
                };
                for ingresso in ["microfono", "sistema", "fileMisto"] {
                    println!(
                        "PROTECTION {} {name} pulizia={cleaning} {ingresso} {level:?}: candidate={} false={} perse={lost} {}",
                        model.id,
                        selections[index].len(),
                        if speech { 0 } else { texts.len() },
                        texts.join(" ")
                    );
                    report.push(serde_json::json!({"model": model.id, "case":name, "cleaning":cleaning, "ingresso":ingresso, "level":format!("{level:?}"), "candidates":selections[index].len(), "falseStarts":if speech {0} else {selections[index].len()}, "falsePhrases":if speech {0} else {texts.len()}, "lostBaselineWords":lost, "text":texts.join(" "), "equivalentProfileMeasurement":true}));
                }
            }
        }
    }
    std::fs::write(
        corpus.parent().unwrap().join("matrix.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(
        false_balanced < false_off,
        "false Frasi: spento={false_off}, bilanciato={false_balanced}"
    );
}

#[test]
#[ignore = "misura Silero sul corpus annotato locale, senza ASR"]
fn misura_evidenza_corpus_protezione() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let corpus = root.join("../.scratch/pulizia-audio/verification-05/corpus");
    for name in [
        "silenzio",
        "rumore-continuo",
        "rumore-intermittente",
        "soffio-sintetico",
        "voce-attenuata-30db",
        "si-no",
    ] {
        let source = corpus.join(format!("{name}.wav"));
        let mut vad = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
        let mut segmenter = Segmenter::new(Params::default());
        let mut evidence = Vec::new();
        let mut starts = Vec::new();
        for feed in FileFrames::new(Decoder::open(&source).unwrap(), None) {
            if let Feed::Frame(frame) = feed.unwrap() {
                let p = vad.probability(&frame).unwrap();
                let crossings = frame
                    .windows(2)
                    .filter(|w| w[0].is_sign_positive() != w[1].is_sign_positive())
                    .count() as f32
                    / frame.len() as f32;
                evidence.push((p, crossings));
                for event in segmenter.push(frame, p) {
                    match event {
                        Event::PhraseStart(s) => starts.push(s),
                        Event::PhraseEnd => {
                            report(name, *starts.last().unwrap(), evidence.len(), &evidence)
                        }
                        Event::Audio(_) => {}
                    }
                }
            }
        }
        if !segmenter.close_phrase().is_empty() {
            report(name, *starts.last().unwrap(), evidence.len(), &evidence);
        }
        println!("MEASURE {name}: candidates={}", starts.len());
    }
}

fn report(name: &str, start: usize, end: usize, evidence: &[(f32, f32)]) {
    let voiced: Vec<_> = evidence[start..end]
        .iter()
        .filter(|(p, _)| *p >= 0.4)
        .collect();
    let peak = voiced.iter().map(|e| e.0).fold(0.0_f32, f32::max);
    let mean = voiced.iter().map(|e| e.0).sum::<f32>() / voiced.len() as f32;
    let zcr = voiced.iter().map(|e| e.1).sum::<f32>() / voiced.len() as f32;
    println!(
        "MEASURE {name} {start}..{end}: voiced={} peak={peak:.6} mean={mean:.6} zcr={zcr:.6}",
        voiced.len()
    );
}
