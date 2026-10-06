//! Banco Windows esplicito: fixture sintetica ripetuta, stessa pipeline, nessuna UI.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::json;
use transcribe_cpp::{Backend, ModelOptions};

use super::*;
use crate::audio_toolkit::{decode::Decoder, ogg_opus::OggOpusWriter, vad::Silero};
use crate::engine::{
    diarize, live, live_diarization,
    pipeline::{self, PipelineEvent},
};
use crate::managers::{
    models,
    settings::{Diarizer, SpeechLanguage},
};
use crate::tape;
use crate::transcript::{Ingresso, Phrase, Transcript};

type Span = (u32, u32, String);
#[derive(Default)]
struct Seen {
    text_ms: f64,
    label_ms: Option<f64>,
}
#[derive(Default)]
struct Observations {
    state: live_diarization::LiveTranscript,
    spans: BTreeMap<Span, Seen>,
    horizons: Vec<(u32, f64)>,
    revisions: usize,
}
fn spans(phrase: &Phrase) -> impl Iterator<Item = Span> + '_ {
    phrase.tempi.iter().map(|t| {
        (
            t.inizio_ms,
            t.fine_ms,
            phrase.text[t.inizio_byte..t.fine_byte].to_string(),
        )
    })
}
impl Observations {
    fn asr(&mut self, input: Ingresso, event: PipelineEvent, now: f64) {
        match &event {
            PipelineEvent::Partial { text, tempi, .. }
            | PipelineEvent::Phrase { text, tempi, .. } => {
                for t in tempi {
                    let key = (
                        t.inizio_ms,
                        t.fine_ms,
                        text[t.inizio_byte..t.fine_byte].to_string(),
                    );
                    self.spans.entry(key).or_insert(Seen {
                        text_ms: now,
                        label_ms: None,
                    });
                }
            }
            PipelineEvent::Progress(_) => {}
        }
        self.state.on_asr(input, event);
        self.inspect(now);
    }
    fn turns(&mut self, turns: Vec<diarize::Turn>, now: f64) {
        let horizon = turns.iter().map(|t| t.fine_ms).max().unwrap_or(0);
        if self.horizons.last().is_none_or(|(end, _)| horizon > *end) {
            self.horizons.push((horizon, now));
        }
        self.state.on_turns(turns);
        self.inspect(now);
    }
    fn inspect(&mut self, now: f64) {
        self.revisions += 1;
        let assigned: BTreeSet<_> = self
            .state
            .phrases
            .iter()
            .chain(&self.state.partials)
            .filter(|(_, p)| p.parlante.is_some() && !p.parlante_non_determinato)
            .flat_map(|(_, p)| spans(p))
            .collect();
        for (key, seen) in &mut self.spans {
            if assigned.contains(key) && seen.label_ms.is_none() {
                seen.label_ms = Some(now);
            }
        }
    }
}
fn distribution(mut values: Vec<f64>) -> serde_json::Value {
    if values.is_empty() {
        return json!({"n": 0});
    }
    values.sort_by(f64::total_cmp);
    let quantile = |p: f64| values[((values.len() as f64 * p).ceil() as usize).saturating_sub(1)];
    json!({"n": values.len(), "min_ms": values[0], "p50_ms": quantile(0.5),
        "p95_ms": quantile(0.95), "p99_ms": quantile(0.99), "max_ms": values[values.len()-1]})
}
// Fine audio -> istante effettivo di consegna del frame contenente quel campione.
fn delivered(end: u32, arrivals: &[f64]) -> Option<f64> {
    end.checked_sub(1)
        .and_then(|t| arrivals.get(t as usize / 30))
        .copied()
}
#[test]
fn metriche_distinguono_audio_testo_e_casi_non_attribuiti() {
    let mut observations = Observations::default();
    let phrase = Phrase {
        inizio_ms: 0,
        fine_ms: 300,
        text: "Ciao".into(),
        tempi: vec![crate::transcript::TempoTesto {
            inizio_byte: 0,
            fine_byte: 4,
            inizio_ms: 60,
            fine_ms: 120,
        }],
        ingresso: Ingresso::Mix,
        parlante: None,
        parlante_non_determinato: false,
        parlante_provvisorio: false,
    };
    observations.asr(
        Ingresso::Mix,
        PipelineEvent::Phrase {
            id: 0,
            inizio_ms: 0,
            fine_ms: 300,
            text: phrase.text,
            tempi: phrase.tempi,
        },
        200.0,
    );
    assert!(observations.spans.values().all(|s| s.label_ms.is_none()));
    observations.turns(
        vec![diarize::Turn {
            inizio_ms: 0,
            fine_ms: 300,
            parlante: 1,
        }],
        350.0,
    );
    let seen = observations.spans.values().next().unwrap();
    assert_eq!(seen.label_ms.unwrap() - seen.text_ms, 150.0);
    assert_eq!(
        seen.label_ms.unwrap() - delivered(120, &[30.0, 60.0, 90.0, 120.0]).unwrap(),
        230.0
    );
    assert_eq!(delivered(121, &[30.0, 60.0, 90.0, 120.0]), None);
    assert_eq!(distribution(vec![10.0, 20.0, 30.0, 40.0])["p95_ms"], 40.0);
    assert_eq!(distribution(Vec::new())["n"], 0);
}

fn load_asr(path: &Path, backend: Backend) -> TranscribeCpp {
    let model = Model::load_with(
        path,
        &ModelOptions {
            backend,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        model.backend(),
        if backend == Backend::Cpu {
            "CPU"
        } else {
            "Vulkan0"
        }
    );
    let capabilities = model.capabilities();
    let mut engine = TranscribeCpp {
        session: model.session().unwrap(),
        languages: capabilities.languages,
        streaming: capabilities.supports_streaming,
        fallback: model.supports(Feature::TemperatureFallback),
        timestamps: capabilities.max_timestamp_kind,
    };
    engine.warm_up();
    engine
}
fn load_diarizer(path: &Path, backend: Backend) -> OfflineDiarizer {
    let model = Model::load_with(
        path,
        &ModelOptions {
            backend,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        model.backend(),
        if backend == Backend::Cpu {
            "CPU"
        } else {
            "Vulkan0"
        }
    );
    OfflineDiarizer {
        session: model.session().unwrap(),
        nemotron3: true,
    }
}
#[test]
#[ignore = "benchmark esplicito Windows CPU/Vulkan con modelli reali, eseguire in sequenza"]
fn ticket08_windows_benchmark() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let folder = PathBuf::from(std::env::var("MEMOTAPE_BENCH_OUTPUT").unwrap());
    std::fs::create_dir_all(&folder).unwrap();
    let inputs_count: usize = std::env::var("MEMOTAPE_BENCH_INPUTS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=2).contains(&inputs_count));
    let backend_name = std::env::var("MEMOTAPE_BENCH_BACKEND").unwrap();
    let backend = match backend_name.as_str() {
        "cpu" => Backend::Cpu,
        "vulkan" => Backend::Vulkan,
        _ => panic!("backend invalido"),
    };
    let seconds: usize = std::env::var("MEMOTAPE_BENCH_SECONDS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((10..=3600).contains(&seconds));
    let model_path = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
    crate::engine::local_diarizer::validate(&model_path).unwrap();
    init_backends().unwrap();
    assert_eq!(transcribe_cpp::version_commit(), "e6672a8");
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let asr_path = models::default_model().path(&models_dir);
    let mut decoder = Decoder::open(&root.join("tests/fixtures/parlato-due-voci.wav")).unwrap();
    let mut fixture = Vec::new();
    while let Some(block) = decoder.next_block().unwrap() {
        fixture.extend(block.mono());
    }
    let audio_ms = (seconds * 16000 / 480 * 30) as u32;
    let loaded_at = Instant::now();
    let mut engines: Vec<_> = (0..inputs_count)
        .map(|_| load_asr(&asr_path, backend))
        .collect();
    let mut diarizers: Vec<_> = (0..inputs_count)
        .map(|_| load_diarizer(&model_path, backend))
        .collect();
    for model in &mut diarizers {
        model.diarize(&[0.0; 16000], &CancelToken::new()).unwrap();
    }
    let load_ms = loaded_at.elapsed().as_millis();
    let inputs = if inputs_count == 1 {
        vec![Ingresso::Mix]
    } else {
        vec![Ingresso::Microfono, Ingresso::Sistema]
    };
    let mut detectors: Vec<_> = (0..inputs_count)
        .map(|_| Silero::new(&root.join("resources/silero_vad.onnx")).unwrap())
        .collect();
    let (mut feeds, frames): (Vec<_>, Vec<_>) = live::channels(16000, 1, inputs_count)
        .unwrap()
        .into_iter()
        .unzip();
    let mut audio = Vec::new();
    for feed in &mut feeds {
        let (tx, rx) = live_diarization::channel();
        feed.set_diarization(tx);
        audio.push(rx);
    }
    let states: Vec<_> = (0..inputs_count)
        .map(|_| Mutex::new(Observations::default()))
        .collect();
    let arrivals = Mutex::new(Vec::new());
    let mut paths: Vec<_> = (0..inputs_count)
        .map(|i| folder.join(format!("ingresso-{i}.ogg")))
        .collect();
    let mut writers: Vec<_> = paths
        .iter()
        .map(|p| OggOpusWriter::new(std::fs::File::create(p).unwrap(), 16000, 1, 64).unwrap())
        .collect();
    let cancel = CancelToken::new();
    let start = Instant::now();
    let now = || start.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "ticket08: ready pid={} backend={backend_name} ingressi={inputs_count} audio={seconds}s load={load_ms}ms",
        std::process::id()
    );
    let (asr, recognized, stop_ms) = std::thread::scope(|scope| {
        let pipelines: Vec<_> = frames
            .into_iter()
            .zip(&mut engines)
            .zip(&mut detectors)
            .enumerate()
            .map(|(index, ((mut frames, engine), detector))| {
                let (states, inputs, cancel, now) = (&states, &inputs, &cancel, &now);
                scope.spawn(move || {
                    pipeline::transcribe(
                        &mut frames,
                        engine,
                        detector,
                        Some("it"),
                        cancel,
                        &mut |event| {
                            states[index]
                                .lock()
                                .unwrap()
                                .asr(inputs[index], event, now());
                        },
                    )
                })
            })
            .collect();
        let recognitions: Vec<_> = audio
            .into_iter()
            .zip(&mut diarizers)
            .enumerate()
            .map(|(index, (mut frames, model))| {
                let (states, cancel, now) = (&states, &cancel, &now);
                scope.spawn(move || {
                    model.diarize_live(&mut frames, cancel, &mut |turns| {
                        states[index].lock().unwrap().turns(turns, now());
                    })
                })
            })
            .collect();
        let total_frames = seconds * 16000 / 480;
        let pause_frame = total_frames / 2;
        let mut pause = Duration::ZERO;
        for index in 0..total_frames {
            if index == pause_frame {
                for feed in &mut feeds {
                    feed.push(&[], true);
                }
                std::thread::sleep(Duration::from_secs(1));
                pause = Duration::from_secs(1);
            }
            let deadline = Duration::from_millis((index * 30) as u64) + pause;
            if let Some(wait) = deadline.checked_sub(start.elapsed()) {
                std::thread::sleep(wait);
            }
            let offset = index * 480;
            let chunk: Vec<_> = (offset..offset + 480)
                .map(|i| fixture[i % fixture.len()])
                .collect();
            arrivals.lock().unwrap().push(now());
            for (feed, writer) in feeds.iter_mut().zip(&mut writers) {
                writer.write(&chunk).unwrap();
                feed.push(&chunk, false);
            }
        }
        for feed in feeds {
            feed.finish();
        }
        let stopped = now();
        (
            pipelines
                .into_iter()
                .map(|p| p.join().unwrap())
                .collect::<Vec<_>>(),
            recognitions
                .into_iter()
                .map(|r| r.join().unwrap())
                .collect::<Vec<_>>(),
            stopped,
        )
    });
    let drained_ms = now();
    let arrivals = arrivals.into_inner().unwrap();
    for writer in writers {
        writer.finish().unwrap();
    }
    assert!(asr.iter().all(Result::is_ok), "{asr:?}");
    let mut measures = Vec::new();
    let mut transcript = Transcript {
        title: "Banco Windows".into(),
        date: "2026-10-06 00:00".into(),
        durata_ms: Some(audio_ms),
        model: "Nemotron".into(),
        speech_language: SpeechLanguage::from("it"),
        phrases: Vec::new(),
        live_asr: Vec::new(),
        parlanti: Default::default(),
        diarizzazione: None,
    };
    for (index, observations) in states.into_iter().enumerate() {
        let observations = observations.into_inner().unwrap();
        let final_spans: BTreeSet<_> = observations.state.originals().flat_map(spans).collect();
        let mut audio_to_text = Vec::new();
        let mut audio_to_label = Vec::new();
        let mut text_to_label = Vec::new();
        let mut rows = Vec::new();
        for span in &final_spans {
            let seen = observations.spans.get(span).unwrap();
            let arrival = delivered(span.1, &arrivals);
            if let Some(t) = arrival {
                audio_to_text.push(seen.text_ms - t);
            }
            if let Some(label) = seen.label_ms {
                text_to_label.push(label - seen.text_ms);
                if let Some(t) = arrival {
                    audio_to_label.push(label - t);
                }
            }
            rows.push(json!({"start_ms": span.0, "end_ms": span.1, "text": span.2,
                "audio_delivered_ms": arrival, "text_available_ms": seen.text_ms, "label_available_ms": seen.label_ms}));
        }
        let frontier: Vec<_> = observations
            .horizons
            .iter()
            .filter_map(|(end, observed)| delivered(*end, &arrivals).map(|sent| observed - sent))
            .collect();
        measures.push(json!({"ingresso": inputs[index], "final_timed_units": final_spans.len(),
            "unlabeled_units": final_spans.len()-text_to_label.len(), "audio_to_text": distribution(audio_to_text),
            "audio_to_label": distribution(audio_to_label), "text_to_label": distribution(text_to_label),
            "diarizer_frontier": distribution(frontier), "revisions": observations.revisions, "units": rows}));
        transcript
            .live_asr
            .extend(observations.state.originals().cloned());
        for (_, phrase) in observations.state.phrases {
            transcript.insert(phrase);
        }
    }
    drop(engines);
    drop(diarizers);
    let final_started = Instant::now();
    let mut results = Vec::new();
    for (input, path) in inputs.iter().zip(&paths) {
        let mut diarizer = load_diarizer(&model_path, backend);
        results.push((
            *input,
            diarizer.diarize_saved(Decoder::open(path).unwrap(), &CancelToken::new()),
        ));
    }
    let final_ms = final_started.elapsed().as_millis();
    let original_text: String = transcript
        .live_asr
        .iter()
        .map(|p| p.text.as_str())
        .collect();
    let original_by_input: Vec<String> = inputs
        .iter()
        .map(|input| {
            transcript
                .live_asr
                .iter()
                .filter(|p| p.ingresso == *input)
                .map(|p| p.text.as_str())
                .collect()
        })
        .collect();
    diarize::finalize_live_ingressi(&mut transcript, Diarizer::Nemotron3, results);
    for (input, original) in inputs.iter().zip(&original_by_input) {
        assert_eq!(
            transcript
                .phrases
                .iter()
                .filter(|p| p.ingresso == *input)
                .map(|p| p.text.as_str())
                .collect::<String>(),
            *original
        );
    }
    // Mix: prima traccia duplicata nel banco; non è una cattura di due dispositivi.
    if inputs_count == 2 {
        paths.insert(0, paths[0].clone());
    }
    let tape_audio: Vec<_> = if inputs_count == 1 {
        vec![(Ingresso::Mix, paths[0].as_path())]
    } else {
        vec![
            (Ingresso::Mix, paths[0].as_path()),
            (Ingresso::Microfono, paths[1].as_path()),
            (Ingresso::Sistema, paths[2].as_path()),
        ]
    };
    let mut document = tape::Document::new(
        tape::creato(chrono::Local::now()),
        audio_ms,
        if inputs_count == 1 {
            tape::Modalita::Mix
        } else {
            tape::Modalita::IngressiSeparati
        },
        Some(models::default_model().id.clone()),
        SpeechLanguage::from("it"),
        true,
        &transcript.phrases,
    );
    document.diarizzazione = transcript.diarizzazione;
    let tape_path = folder.join("Banco.tape");
    tape::write(&tape_path, &tape_audio, &document, None).unwrap();
    assert_eq!(tape::read(&tape_path).unwrap(), document);
    for (input, path) in &tape_audio {
        use std::io::Read;
        let mut saved = Vec::new();
        tape::Mix::open_ingresso(&tape_path, *input)
            .unwrap()
            .read_to_end(&mut saved)
            .unwrap();
        assert_eq!(saved, std::fs::read(path).unwrap());
        let mut decoder = Decoder::open_ingresso(&tape_path, *input).unwrap();
        let mut decoded_ms = 0.0;
        while let Some(block) = decoder.next_block().unwrap() {
            decoded_ms +=
                block.samples.len() as f64 * 1000.0 / f64::from(block.rate) / block.channels as f64;
        }
        assert!(
            (decoded_ms - f64::from(audio_ms)).abs() < 0.1,
            "decoded={decoded_ms} expected={audio_ms}"
        );
    }
    let reopened = crate::managers::transcription::open_tape(&tape_path).unwrap();
    assert_eq!(reopened.phrases.len(), transcript.phrases.len());
    let report = json!({"fixture_kind":"synthetic_repeated", "runtime_commit":transcribe_cpp::version_commit(),
        "backend": backend_name, "inputs":inputs_count, "audio_ms":audio_ms, "pause_wall_ms":1000,
        "load_warmup_ms":load_ms, "stop_ms":stop_ms, "drained_ms":drained_ms, "stop_drain_ms":drained_ms-stop_ms,
        "final_analysis_ms":final_ms, "live_results":recognized.iter().map(|r| format!("{r:?}")).collect::<Vec<_>>(),
        "final_outcome":document.diarizzazione, "text_bytes":original_text.len(),"measures":measures,
        "integrity":{"asr":true,"audio_bytes":true,"decoded_duration":true,"text_final":true,"reopen":true}});
    std::fs::write(
        folder.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ticket08 report={}", folder.join("report.json").display());
}

#[test]
#[ignore = "confronto locale con ASR congelata e due diarizer reali, eseguire in sequenza"]
fn ticket08_confronto_stessa_asr() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = PathBuf::from(std::env::var("MEMOTAPE_COMPARE_AUDIO").unwrap());
    let folder = PathBuf::from(std::env::var("MEMOTAPE_COMPARE_OUTPUT").unwrap());
    std::fs::create_dir_all(&folder).unwrap();
    let backend = match std::env::var("MEMOTAPE_COMPARE_BACKEND").unwrap().as_str() {
        "cpu" => Backend::Cpu,
        "vulkan" => Backend::Vulkan,
        _ => panic!("backend invalido"),
    };
    init_backends().unwrap();
    assert_eq!(transcribe_cpp::version_commit(), "e6672a8");
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let mut engine = load_asr(&models::default_model().path(&models_dir), backend);
    let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
    let mut originals = Vec::new();
    let mut pcm = Vec::new();
    pipeline::transcribe_file(
        &source,
        &mut engine,
        &mut detector,
        Some("it"),
        Some(&mut pcm),
        None,
        &CancelToken::new(),
        &mut |event| {
            if let PipelineEvent::Phrase {
                inizio_ms,
                fine_ms,
                text,
                tempi,
                ..
            } = event
            {
                originals.push(Phrase {
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
    )
    .unwrap();
    assert!(!originals.is_empty(), "nessun testo riconosciuto");
    drop(engine);
    let nemotron = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
    crate::engine::local_diarizer::validate(&nemotron).unwrap();
    let sortformer = models::diarizer().path(&models_dir);
    for (name, path, selected) in [
        ("sortformer", sortformer, Diarizer::Sortformer),
        ("nemotron3", nemotron, Diarizer::Nemotron3),
    ] {
        let model = Model::load_with(
            &path,
            &ModelOptions {
                backend,
                ..Default::default()
            },
        )
        .unwrap();
        let mut diarizer = OfflineDiarizer {
            session: model.session().unwrap(),
            nemotron3: selected == Diarizer::Nemotron3,
        };
        let started = Instant::now();
        let turns = diarizer.diarize(&pcm, &CancelToken::new()).unwrap();
        let elapsed_ms = started.elapsed().as_millis();
        let rttm: String = turns
            .iter()
            .map(|t| {
                format!(
                    "SPEAKER sample 1 {:.3} {:.3} <NA> <NA> {} <NA> <NA>\n",
                    f64::from(t.inizio_ms) / 1000.0,
                    f64::from(t.fine_ms - t.inizio_ms) / 1000.0,
                    t.parlante
                )
            })
            .collect();
        std::fs::write(folder.join(format!("{name}.rttm")), rttm).unwrap();
        let mut phrases = originals.clone();
        diarize::assign_configured(&mut phrases, Ingresso::Mix, &turns, selected);
        assert_eq!(
            phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
            originals
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>()
        );
        let units: Vec<_> = originals.iter().flat_map(spans).map(|(start,end,text)| {
            let assigned = phrases.iter().find(|p| p.inizio_ms <= start && p.fine_ms >= end && p.parlante.is_some() && !p.parlante_non_determinato);
            json!({"start_ms":start,"end_ms":end,"text":text,"speaker":assigned.and_then(|p| p.parlante).map(|n| n.to_string())})
        }).collect();
        std::fs::write(
            folder.join(format!("{name}-units.json")),
            serde_json::to_vec_pretty(&units).unwrap(),
        )
        .unwrap();
        std::fs::write(
            folder.join(format!("{name}-phrases.json")),
            serde_json::to_vec_pretty(&phrase_rows(&phrases)).unwrap(),
        )
        .unwrap();
        std::fs::write(folder.join(format!("{name}-inference.json")),serde_json::to_vec_pretty(&json!({"milliseconds":elapsed_ms,"backend":format!("{backend:?}"),"audio_ms":pcm.len()/16,"no_reference_scored":true})).unwrap()).unwrap();
    }
    std::fs::write(
        folder.join("frozen-asr.json"),
        serde_json::to_vec_pretty(&phrase_rows(&originals)).unwrap(),
    )
    .unwrap();
}

fn phrase_rows(phrases: &[Phrase]) -> Vec<serde_json::Value> {
    phrases
        .iter()
        .map(|p| {
            json!({"start_ms":p.inizio_ms,"end_ms":p.fine_ms,"text":p.text,
        "speaker":p.parlante,"unknown":p.parlante_non_determinato,"tempi":p.tempi})
        })
        .collect()
}

#[test]
#[ignore = "Tape reale prodotto dal banco, copia/esportazione/player e Annulla nativo"]
fn ticket08_flusso_tape_e_annulla_nativo() {
    use crate::managers::settings::{CopiaCome, Language, Settings};
    use crate::managers::transcription;
    use std::sync::atomic::{AtomicBool, Ordering};
    let source = PathBuf::from(std::env::var("MEMOTAPE_FLOW_TAPE").unwrap());
    let folder = PathBuf::from(std::env::var("MEMOTAPE_FLOW_OUTPUT").unwrap());
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Corretto.tape");
    std::fs::copy(&source, &path).unwrap();
    let original = tape::read(&path).unwrap();
    let first = original
        .frasi
        .iter()
        .find(|p| p.parlante.is_some() && !p.parlante_non_determinato)
        .unwrap();
    let (_, audio_before) = crate::player::read(&path, None).unwrap();
    let (window, range) = crate::player::read(&path, Some("bytes=100-199")).unwrap();
    assert_eq!(window.status, 206);
    assert_eq!(range, audio_before[100..200]);
    tape::edit_frase(
        &path,
        first.ingresso,
        first.id,
        "Correzione umana di prova.",
    )
    .unwrap();
    let corrected = tape::read(&path).unwrap();
    assert!(
        corrected
            .frasi
            .iter()
            .find(|p| p.id == first.id && p.ingresso == first.ingresso)
            .unwrap()
            .tempi
            .is_empty()
    );
    assert_eq!(
        corrected
            .frasi
            .iter()
            .find(|p| p.id == first.id && p.ingresso == first.ingresso)
            .unwrap()
            .inizio_ms,
        first.inizio_ms
    );
    assert_eq!(
        corrected
            .frasi
            .iter()
            .find(|p| p.id == first.id && p.ingresso == first.ingresso)
            .unwrap()
            .fine_ms,
        first.fine_ms
    );
    tape::rename_parlante(&path, first.ingresso, first.parlante, "Voce verificata").unwrap();
    for (name, language) in [
        ("it", Language::It),
        ("en", Language::En),
        ("fr", Language::Fr),
        ("es", Language::Es),
        ("de", Language::De),
        ("pl", Language::Pl),
    ] {
        let settings = Settings {
            interface_language: Some(language),
            ..Default::default()
        };
        for (extension, format) in [("txt", CopiaCome::Testo), ("md", CopiaCome::Markdown)] {
            let text = transcription::tape_text(&path, &settings, format).unwrap();
            assert!(text.contains("Correzione umana di prova."));
            assert!(text.contains("Voce verificata"));
            assert!(!text.contains("{{"));
            if original.frasi.iter().any(|p| p.parlante_non_determinato) {
                assert!(text.contains(&crate::transcript::Labels::of(language).unknown_speaker()));
            }
            std::fs::write(folder.join(format!("corretto-{name}.{extension}")), text).unwrap();
        }
    }
    assert_eq!(crate::player::read(&path, None).unwrap().1, audio_before);
    let reopened = transcription::open_tape(&path).unwrap();
    assert!(
        reopened
            .phrases
            .iter()
            .any(|p| p.text == "Correzione umana di prova.")
    );
    let model = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
    init_backends().unwrap();
    let mut diarizer = load_diarizer(&model, Backend::Vulkan);
    let cancel = CancelToken::new();
    let started = AtomicBool::new(false);
    let result = std::thread::scope(|scope| {
        let run = scope.spawn(|| {
            started.store(true, Ordering::Release);
            diarizer.diarize_saved(Decoder::open(&source).unwrap(), &cancel)
        });
        while !started.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        std::thread::sleep(Duration::from_millis(50));
        cancel.cancel();
        run.join().unwrap()
    });
    assert_eq!(result, Err(AppError::Cancelled));
    let mut incomplete = corrected.clone();
    for phrase in &mut incomplete.frasi {
        phrase.parlante_provvisorio = true;
    }
    let mut snapshot: Vec<Phrase> = incomplete
        .frasi
        .iter()
        .map(|p| Phrase {
            inizio_ms: p.inizio_ms,
            fine_ms: p.fine_ms,
            text: p.testo.clone(),
            tempi: p.tempi.clone(),
            ingresso: p.ingresso,
            parlante: p.parlante,
            parlante_provvisorio: p.parlante_provvisorio,
            parlante_non_determinato: p.parlante_non_determinato,
        })
        .collect();
    let before: String = snapshot.iter().map(|p| p.text.as_str()).collect();
    incomplete.diarizzazione = diarize::finalize(
        &mut snapshot,
        Diarizer::Nemotron3,
        &CancelToken::new(),
        result.map(|turns| vec![(Ingresso::Mix, turns)]),
    );
    assert_eq!(
        snapshot.iter().map(|p| p.text.as_str()).collect::<String>(),
        before
    );
    assert!(snapshot.iter().all(|p| p.parlante_provvisorio));
    tape::rewrite(&path, &incomplete).unwrap();
    assert!(transcription::open_tape(&path).unwrap().info.completa);
    for language in [
        Language::It,
        Language::En,
        Language::Fr,
        Language::Es,
        Language::De,
        Language::Pl,
    ] {
        let settings = Settings {
            interface_language: Some(language),
            ..Default::default()
        };
        let text = transcription::tape_text(&path, &settings, CopiaCome::Markdown).unwrap();
        let labels = crate::transcript::Labels::of(language);
        let speaker = incomplete.frasi.iter().find_map(|p| p.parlante).unwrap();
        // Accetta il nome rinominato o il numero nella stessa etichetta provvisoria tradotta.
        assert!(
            text.contains(&labels.provisional_speaker(&labels.parlante(speaker)))
                || text.contains(&labels.provisional_speaker("Voce verificata"))
        );
    }
    assert_eq!(crate::player::read(&path, None).unwrap().1, audio_before);
    // Schema precedente: nessuno dei nuovi campi di diarizzazione/tempi per il Tape v1.
    let mut legacy = original.clone();
    legacy.diarizzazione = None;
    for phrase in &mut legacy.frasi {
        phrase.tempi.clear();
        phrase.parlante_provvisorio = false;
        phrase.parlante_non_determinato = false;
    }
    let old_path = folder.join("Precedente.tape");
    let audio_path = folder.join("mix.ogg");
    std::fs::write(&audio_path, &audio_before).unwrap();
    tape::write(
        &old_path,
        &[(Ingresso::Mix, audio_path.as_path())],
        &legacy,
        None,
    )
    .unwrap();
    let old_bytes = std::fs::read(&old_path).unwrap();
    let old = transcription::open_tape(&old_path).unwrap();
    assert!(!old.info.ingressi_separati);
    assert!(old.info.diarizzazione.is_none());
    assert_eq!(std::fs::read(&old_path).unwrap(), old_bytes);
    assert_eq!(
        crate::player::read(&old_path, None).unwrap().1,
        audio_before
    );
    eprintln!(
        "ticket08 flusso: Annulla nativo, player Range, correzione/rinomina, 6 lingue copia/export, provvisorio, Tape v1 senza riscrittura; UI non attestata"
    );
}

#[test]
#[ignore = "risultati del comparatore e audio corpus locale: integrazione writer/Tape/manager"]
fn ticket08_tape_del_corpus() {
    let folder = PathBuf::from(std::env::var("MEMOTAPE_CORPUS_CASE").unwrap());
    let comparison = folder.join("comparison-vulkan");
    let projected: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(comparison.join("nemotron3-phrases.json")).unwrap())
            .unwrap();
    let frozen: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(comparison.join("frozen-asr.json")).unwrap())
            .unwrap();
    let phrases: Vec<Phrase> = projected
        .iter()
        .map(|row| Phrase {
            inizio_ms: u32::try_from(row["start_ms"].as_u64().unwrap()).unwrap(),
            fine_ms: u32::try_from(row["end_ms"].as_u64().unwrap()).unwrap(),
            text: row["text"].as_str().unwrap().to_string(),
            tempi: serde_json::from_value(row["tempi"].clone()).unwrap(),
            ingresso: Ingresso::Mix,
            parlante: row["speaker"].as_u64().map(|s| u32::try_from(s).unwrap()),
            parlante_provvisorio: false,
            parlante_non_determinato: row["unknown"].as_bool().unwrap(),
        })
        .collect();
    assert_eq!(
        phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
        frozen
            .iter()
            .map(|p| p["text"].as_str().unwrap())
            .collect::<String>()
    );
    assert!(phrases.iter().any(|p| p.parlante_non_determinato));
    let output = folder.join("flow-source");
    std::fs::create_dir_all(&output).unwrap();
    let ogg = output.join("mix.ogg");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ogg)
        .unwrap();
    let mut writer = OggOpusWriter::new(file, 16000, 1, 64).unwrap();
    let mut decoder = Decoder::open(&folder.join("audio.wav")).unwrap();
    let mut samples = 0;
    while let Some(block) = decoder.next_block().unwrap() {
        assert_eq!((block.rate, block.channels), (16000, 1));
        samples += block.samples.len();
        writer.write(&block.samples).unwrap();
    }
    writer.finish().unwrap();
    let mut document = tape::Document::new(
        tape::creato(chrono::Local::now()),
        u32::try_from(samples / 16).unwrap(),
        tape::Modalita::Mix,
        Some(models::default_model().id.clone()),
        SpeechLanguage::from("it"),
        true,
        &phrases,
    );
    document.diarizzazione = Some(crate::transcript::Diarizzazione {
        modello: Diarizer::Nemotron3,
        esito: crate::transcript::EsitoDiarizzazione::Completata,
        ingressi: vec![crate::transcript::DiarizzazioneIngresso {
            ingresso: Ingresso::Mix,
            esito: crate::transcript::EsitoDiarizzazione::Completata,
        }],
    });
    let path = output.join("Corpus.tape");
    tape::write(&path, &[(Ingresso::Mix, ogg.as_path())], &document, None).unwrap();
    assert_eq!(tape::read(&path).unwrap(), document);
    let opened = crate::managers::transcription::open_tape(&path).unwrap();
    assert_eq!(opened.phrases.len(), phrases.len());
    assert!(opened.phrases.iter().any(|p| p.parlante_non_determinato));
    assert_eq!(
        crate::player::read(&path, None).unwrap().1,
        std::fs::read(&ogg).unwrap()
    );
    eprintln!(
        "ticket08 corpus Tape: writer Opus/Tape/riapertura/unknown dal confronto reale costruito, {} Frasi; UI non attestata",
        phrases.len()
    );
}

// Ricava il namespace richiamando l'assegnazione di produzione, senza riferimenti gold.
// Ancore fuori dall'audio rendono osservabile ogni ID raw dopo le Frasi originali.
fn projection_namespace(
    times: &[(u32, u32)],
    turns: &[diarize::Turn],
    selected: Diarizer,
) -> BTreeMap<u32, u32> {
    let ids: BTreeSet<u32> = turns.iter().map(|t| t.parlante).collect();
    let mut expanded = turns.to_vec();
    let mut probes = times.to_vec();
    let mut end = times
        .iter()
        .map(|t| t.1)
        .chain(turns.iter().map(|t| t.fine_ms))
        .max()
        .unwrap_or(0)
        + 100;
    for id in &ids {
        probes.push((end, end + 30));
        expanded.push(diarize::Turn {
            inizio_ms: end,
            fine_ms: end + 30,
            parlante: *id,
        });
        end += 60;
    }
    let assigned = if selected == Diarizer::Sortformer {
        diarize::assign(&probes, &expanded)
    } else {
        diarize::assign_unambiguous(&probes, &expanded)
    };
    ids.into_iter()
        .zip(assigned.into_iter().skip(times.len()))
        .map(|(raw, projected)| (projected.unwrap(), raw))
        .collect()
}

#[test]
fn namespace_della_proiezione_resiste_alla_permutazione_degli_id_raw() {
    let turns = vec![
        diarize::Turn {
            inizio_ms: 0,
            fine_ms: 100,
            parlante: 3,
        },
        diarize::Turn {
            inizio_ms: 100,
            fine_ms: 1000,
            parlante: 4,
        },
        diarize::Turn {
            inizio_ms: 1000,
            fine_ms: 2000,
            parlante: 3,
        },
    ];
    let times = [(200, 900), (1100, 1900)];
    let changed: Vec<_> = turns
        .iter()
        .map(|t| diarize::Turn {
            parlante: if t.parlante == 3 { 4 } else { 3 },
            ..*t
        })
        .collect();
    for selected in [Diarizer::Sortformer, Diarizer::Nemotron3] {
        let before = projection_namespace(&times, &turns, selected);
        let after = projection_namespace(&times, &changed, selected);
        let assigned_before = if selected == Diarizer::Sortformer {
            diarize::assign(&times, &turns)
        } else {
            diarize::assign_unambiguous(&times, &turns)
        };
        let assigned_after = if selected == Diarizer::Sortformer {
            diarize::assign(&times, &changed)
        } else {
            diarize::assign_unambiguous(&times, &changed)
        };
        assert_eq!(assigned_before, assigned_after);
        for id in assigned_before.into_iter().flatten() {
            assert_eq!(before[&id] == 4, after[&id] == 3);
        }
    }
    assert_eq!(
        projection_namespace(&times, &turns, Diarizer::Sortformer),
        BTreeMap::from([(1, 4), (2, 3)])
    );
    assert_eq!(
        projection_namespace(&times, &turns, Diarizer::Nemotron3),
        BTreeMap::from([(1, 3), (2, 4)])
    );
}

#[test]
#[ignore = "RTTM/ASR del corpus già eseguito: replay assegnazione vera, nessuna nuova inferenza"]
fn ticket08_namespace_del_corpus() {
    let root = PathBuf::from(std::env::var("MEMOTAPE_CORPUS_ROOT").unwrap());
    for entry in std::fs::read_dir(root.join("cases")).unwrap() {
        let comparison = entry.unwrap().path().join("comparison-vulkan");
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(&std::fs::read(comparison.join("frozen-asr.json")).unwrap())
                .unwrap();
        let originals: Vec<Phrase> = rows
            .iter()
            .map(|row| Phrase {
                inizio_ms: u32::try_from(row["start_ms"].as_u64().unwrap()).unwrap(),
                fine_ms: u32::try_from(row["end_ms"].as_u64().unwrap()).unwrap(),
                text: row["text"].as_str().unwrap().to_string(),
                tempi: serde_json::from_value(row["tempi"].clone()).unwrap(),
                ingresso: Ingresso::Mix,
                parlante: None,
                parlante_provvisorio: false,
                parlante_non_determinato: false,
            })
            .collect();
        let times: Vec<_> = originals.iter().map(|p| (p.inizio_ms, p.fine_ms)).collect();
        for (name, selected) in [
            ("sortformer", Diarizer::Sortformer),
            ("nemotron3", Diarizer::Nemotron3),
        ] {
            let turns: Vec<diarize::Turn> =
                std::fs::read_to_string(comparison.join(format!("{name}.rttm")))
                    .unwrap()
                    .lines()
                    .map(|line| {
                        let fields: Vec<_> = line.split_whitespace().collect();
                        assert_eq!(
                            (fields.len(), fields[0], fields[1]),
                            (10, "SPEAKER", "sample")
                        );
                        let start: f64 = fields[3].parse().unwrap();
                        let duration: f64 = fields[4].parse().unwrap();
                        diarize::Turn {
                            inizio_ms: (start * 1000.0).round() as u32,
                            fine_ms: ((start + duration) * 1000.0).round() as u32,
                            parlante: fields[7].parse().unwrap(),
                        }
                    })
                    .collect();
            let mut replay = originals.clone();
            diarize::assign_configured(&mut replay, Ingresso::Mix, &turns, selected);
            let expected: Vec<serde_json::Value> = serde_json::from_slice(
                &std::fs::read(comparison.join(format!("{name}-phrases.json"))).unwrap(),
            )
            .unwrap();
            assert_eq!(
                phrase_rows(&replay),
                expected,
                "replay numerazione/proiezione diverso dal risultato nativo originale"
            );
            let namespace = projection_namespace(&times, &turns, selected);
            assert!(
                replay
                    .iter()
                    .filter_map(|p| p.parlante)
                    .all(|id| namespace.contains_key(&id))
            );
            let path = comparison.join(format!("{name}-speaker-namespace.json"));
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap();
            serde_json::to_writer_pretty(file,&json!({"source":"production assign/assign_unambiguous with probes after real timeline", "projected_to_raw":namespace})).unwrap();
        }
    }
    eprintln!(
        "ticket08 namespace: replay identico a entrambe le proiezioni per tutti5casi, namespace ricavato dalla produzione senza reference"
    );
}
