//! Corpus sintetico passato davvero dal percorso a trazione e dai Parziali nativi.
use super::*;
use crate::audio_toolkit::{
    cleaning::{CleaningLog, ConfiguredCleaning},
    deepfilter,
    protection::Sensibilita,
    vad::Silero,
};
use crate::engine::{live, transcribe_cpp::TranscribeCpp};
use crate::managers::models;
use crate::transcript::Ingresso;
use std::path::{Path, PathBuf};

fn corpus() -> Vec<(String, bool, Vec<Feed>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let corpus = root.join("../.scratch/pulizia-audio/verification-05/corpus");
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
            let frames = FileFrames::with_processor(
                Decoder::open(&corpus.join(format!("{name}.wav"))).unwrap(),
                None,
                processor,
            )
            .map(Result::unwrap)
            .collect();
            cases.push((name.to_string(), cleaning, frames));
        }
    }
    cases
}

const LEVELS: [Sensibilita; 4] = [
    Sensibilita::Spento,
    Sensibilita::Sensibile,
    Sensibilita::Bilanciato,
    Sensibilita::Selettivo,
];

fn speech(name: &str) -> bool {
    ["voce-attenuata-30db", "si-no"].contains(&name)
}

#[test]
#[ignore = "DFN3/Silero veri, ammissione sul prefisso live del corpus sintetico"]
fn protezione_live_prefissi_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut noise = [0; 4];
    for (name, cleaning, feeds) in corpus() {
        let mut selections = Vec::new();
        for (i, level) in LEVELS.into_iter().enumerate() {
            let timeline = ProtectionTimeline::default();
            timeline.record(0, level);
            let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
            let mut engine = tests::FakeEngine::default();
            transcribe_protected(
                &mut feeds.clone().into_iter().map(Ok),
                &mut engine,
                &mut detector,
                Some("it"),
                &CancelToken::new(),
                &mut |_| {},
                &timeline,
            )
            .unwrap();
            println!(
                "PREFIX {name} cleaning={cleaning} {level:?} starts={}",
                engine.samples.len()
            );
            if !speech(&name) {
                noise[i] += engine.samples.len();
            }
            selections.push(engine.pcm);
        }
        if speech(&name) {
            assert_eq!(
                selections[0], selections[1],
                "sensibile {name} cleaning={cleaning}"
            );
            assert_eq!(
                selections[0], selections[2],
                "bilanciato {name} cleaning={cleaning}"
            );
        }
    }
    assert!(noise[2] < noise[0], "avvii rumore {noise:?}");
}

struct MeasuredEngine<'a> {
    engine: &'a mut TranscribeCpp,
    starts: usize,
}
impl TranscriptionEngine for MeasuredEngine<'_> {
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
        language: Option<&str>,
        partial: Option<&mut dyn FnMut(&crate::engine::asr::AsrResult)>,
    ) -> Result<crate::engine::asr::AsrResult, super::super::EngineError> {
        self.starts += 1;
        self.engine.transcribe(frames, language, partial)
    }
}

#[test]
#[ignore = "release DFN3 + Silero + 3 ASR sequenziali, 2 Ingressi sintetici attraverso LiveFeed"]
fn protezione_live_corpus_tre_asr() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases = corpus();
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let mut report = Vec::new();
    let mut false_phrases = [0; 4];
    for model in models::catalog()
        .iter()
        .filter(|m| m.kind == models::ModelKind::Trascrizione)
    {
        let mut engine = TranscribeCpp::load(&model.path(&models_dir)).unwrap();
        for (name, cleaning, source) in &cases {
            for ingresso in [Ingresso::Microfono, Ingresso::Sistema] {
                let mut baseline = String::new();
                let mut baseline_partials = 0;
                for (index, level) in LEVELS.into_iter().enumerate() {
                    let (mut feed, frames) = live::channels(16_000, 1, 1).unwrap().remove(0);
                    feed.protection().record(0, level);
                    for input in source {
                        if let Feed::Frame(f) = input {
                            feed.push(f, false);
                        }
                    }
                    feed.finish();
                    let timeline = frames.protection();
                    let read = Cell::new(0);
                    let mut source = frames.inspect(|f| {
                        if matches!(f, Ok(Feed::Frame(_))) {
                            read.set(read.get() + 1);
                        }
                    });
                    let mut detector =
                        Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
                    let mut measured = MeasuredEngine {
                        engine: &mut engine,
                        starts: 0,
                    };
                    let mut phrases = Vec::new();
                    let mut partials = Vec::new();
                    let mut ends = Vec::new();
                    transcribe_protected(
                        &mut source,
                        &mut measured,
                        &mut detector,
                        Some("it"),
                        &CancelToken::new(),
                        &mut |e| match e {
                            PipelineEvent::Phrase { text, fine_ms, .. } => {
                                phrases.push(text);
                                ends.push(fine_ms);
                            }
                            PipelineEvent::Partial { text, fine_ms, .. } if !text.is_empty() => {
                                partials.push((read.get(), fine_ms))
                            }
                            _ => {}
                        },
                        &timeline,
                    )
                    .unwrap();
                    let text = phrases.join(" ");
                    if index == 0 {
                        baseline = text.clone();
                        baseline_partials = partials.len();
                    }
                    let false_count = if speech(name) { 0 } else { phrases.len() };
                    false_phrases[index] += false_count;
                    let lost = if speech(name) {
                        baseline
                            .split_whitespace()
                            .count()
                            .saturating_sub(text.split_whitespace().count())
                    } else {
                        0
                    };
                    if speech(name) && (index == 1 || index == 2) {
                        assert_eq!(
                            text, baseline,
                            "regressione live {} {name} cleaning={cleaning} {level:?}",
                            model.id
                        );
                    }
                    if model.id.contains("nemotron") && speech(name) && index <= 2 {
                        assert_eq!(partials.len(), baseline_partials, "Parziali persi {name}");
                        if name == "voce-attenuata-30db" {
                            assert!(
                                partials.iter().any(|(_, partial_end)| ends
                                    .iter()
                                    .any(|end| partial_end < end)),
                                "mancano Parziali durante parlato lungo"
                            );
                        }
                    }
                    println!(
                        "LIVE {} {name} cleaning={cleaning} {ingresso:?} {level:?} starts={} false={false_count} lost={lost} partials={} {text}",
                        model.id,
                        measured.starts,
                        partials.len()
                    );
                    report.push(serde_json::json!({"model":model.id,"case":name,"cleaning":cleaning,"ingresso":format!("{ingresso:?}"),"level":format!("{level:?}"),"starts":measured.starts,"falsePhrases":false_count,"lostBaselineWords":lost,"text":text,"partials":partials,"phraseEnds":ends,"syntheticQueuedLive":true}));
                }
            }
        }
    }
    std::fs::write(
        root.join("../.scratch/pulizia-audio/verification-06/matrix.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(
        false_phrases[2] < false_phrases[0],
        "false Frasi live {false_phrases:?}"
    );
}
