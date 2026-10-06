//! Smoke nativo deterministico: due tracce dalla fixture, due ASR, uno o due diarizer.
//! Non attesta hardware di cattura, qualità sul parlato reale o latenza della vista.
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::Mutex;

use super::*;
use crate::audio_toolkit::{
    decode::Decoder,
    ogg_opus::{OggOpusWriter, tests::temp_dir},
    vad::Silero,
};
use crate::engine::{
    live, pipeline,
    transcribe_cpp::{OfflineDiarizer, TranscribeCpp},
};
use crate::managers::{
    models,
    settings::{Diarizer, SpeechLanguage},
};
use crate::{
    tape,
    transcript::{EsitoDiarizzazione, Transcript},
};

// Get-Process osserva il solo processo del test. CPU cumulativa; memoria host, non VRAM.
fn process_stats() -> (u64, u64, f64) {
    let script = format!(
        "$p = Get-Process -Id {}; Write-Output ($p.WorkingSet64.ToString() + ',' + $p.PrivateMemorySize64.ToString() + ',' + $p.CPU.ToString([System.Globalization.CultureInfo]::InvariantCulture))",
        std::process::id()
    );
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    let values: Vec<_> = output.trim().split(',').collect();
    (
        values[0].parse().unwrap(),
        values[1].parse().unwrap(),
        values[2].parse().unwrap(),
    )
}

#[test]
#[ignore = "due ASR e modelli Nemotron 3 reali; smoke sequenziale con memoria e carico"]
fn due_ingressi_nativi_una_e_due_istanze_con_guasto_isolato() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let fixture = root.join("tests/fixtures/parlato-due-voci.wav");
    let mut decoder = Decoder::open(&fixture).unwrap();
    let mut pcm = Vec::new();
    while let Some(block) = decoder.next_block().unwrap() {
        pcm.extend(block.mono());
    }
    for (count, fault) in [(1, false), (2, false), (2, true)] {
        let folder = temp_dir(&format!("ticket07-native-{count}"));
        let before = process_stats();
        let load = Instant::now();
        let mut engines: Vec<_> = (0..2)
            .map(|_| TranscribeCpp::load(&models::default_model().path(&models_dir)).unwrap())
            .collect();
        let mut diarizers: Vec<_> = (0..count)
            .map(|_| OfflineDiarizer::load_nemotron3(&model).unwrap())
            .collect();
        for model in &mut diarizers {
            model.diarize(&[0.0; 16000], &CancelToken::new()).unwrap();
        }
        let loaded = process_stats();
        eprintln!(
            "ticket07 {count} diarizer + 2 ASR (guasto={fault}): caricamento={} ms, WS prima={} MiB/caricati={} MiB, private prima={} MiB/caricati={} MiB",
            load.elapsed().as_millis(),
            before.0 / 1048576,
            loaded.0 / 1048576,
            before.1 / 1048576,
            loaded.1 / 1048576
        );
        let mut detectors: Vec<_> = (0..2)
            .map(|_| Silero::new(&root.join("resources/silero_vad.onnx")).unwrap())
            .collect();
        let channels = live::channels(16000, 1, 2).unwrap();
        let (mut feeds, frames): (Vec<_>, Vec<_>) = channels.into_iter().unzip();
        let inputs = [Ingresso::Microfono, Ingresso::Sistema];
        let mut audio = Vec::new();
        // Casella spenta: solo Sistema; casella accesa: modello e stream distinti per entrambi.
        for index in if count == 1 { vec![1] } else { vec![0, 1] } {
            let (tx, rx) = channel();
            feeds[index].set_diarization(tx);
            audio.push((index, rx));
        }
        assert_eq!(audio.len(), count);
        let states = [
            Mutex::new(LiveTranscript::default()),
            Mutex::new(LiveTranscript::default()),
        ];
        let plain = Mutex::new(Vec::new());
        let mut peak = loaded;
        let start = Instant::now();
        let mut writers: Vec<_> = ["mix.ogg", "microfono.ogg", "sistema.ogg"]
            .into_iter()
            .map(|name| {
                OggOpusWriter::new(
                    std::fs::File::create(folder.join(name)).unwrap(),
                    16000,
                    1,
                    64,
                )
                .unwrap()
            })
            .collect();
        let cancel = CancelToken::new();
        let (asr, analyzed) = std::thread::scope(|scope| {
            let pipelines: Vec<_> = frames
                .into_iter()
                .zip(&mut engines)
                .zip(&mut detectors)
                .enumerate()
                .map(|(index, ((mut frames, engine), detector))| {
                    let (states, plain, cancel) = (&states, &plain, &cancel);
                    scope.spawn(move || {
                        pipeline::transcribe(
                            &mut frames,
                            engine,
                            detector,
                            Some("it"),
                            cancel,
                            &mut |event| {
                                if count == 2 || index == 1 {
                                    states[index].lock().unwrap().on_asr(inputs[index], event);
                                } else if let PipelineEvent::Phrase {
                                    inizio_ms,
                                    fine_ms,
                                    text,
                                    tempi,
                                    ..
                                } = event
                                {
                                    plain.lock().unwrap().push(Phrase {
                                        inizio_ms,
                                        fine_ms,
                                        text,
                                        tempi,
                                        ingresso: inputs[index],
                                        parlante: None,
                                        parlante_non_determinato: false,
                                        parlante_provvisorio: false,
                                    });
                                }
                            },
                        )
                    })
                })
                .collect();
            let recognitions: Vec<_> = audio
                .into_iter()
                .zip(&mut diarizers)
                .map(|((index, mut audio), model)| {
                    let (states, cancel) = (&states, &cancel);
                    scope.spawn(move || {
                        let mut delayed = false;
                        let result = model.diarize_live(&mut audio, cancel, &mut |turns| {
                            let end = turns.iter().map(|t| t.fine_ms).max().unwrap_or(0);
                            states[index].lock().unwrap().on_turns(turns);
                            if fault && index == 0 && end >= 9000 && !delayed {
                                delayed = true;
                                std::thread::sleep(Duration::from_secs(4));
                            }
                        });
                        (inputs[index], result)
                    })
                })
                .collect();
            let mut frame = 0;
            let total = pcm.len().div_ceil(480);
            let mut next_sample = 5;
            while frame < total {
                let elapsed = start.elapsed();
                // Stessa sessione: pausa da 1 s, non inserita nell'audio delle tracce.
                if (12..13).contains(&elapsed.as_secs()) {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                let active = elapsed.saturating_sub(if elapsed.as_secs() >= 13 {
                    Duration::from_secs(1)
                } else {
                    Duration::ZERO
                });
                if active.as_millis() >= (frame * 30) as u128 {
                    let chunk = &pcm[frame * 480..((frame + 1) * 480).min(pcm.len())];
                    // Le tracce hanno la stessa fixture sintetica. Il mix ne è la somma con clamp.
                    let mixed: Vec<_> = chunk.iter().map(|v| (v * 2.0).clamp(-1.0, 1.0)).collect();
                    writers[0].write(&mixed).unwrap();
                    for (feed, writer) in feeds.iter_mut().zip(&mut writers[1..]) {
                        writer.write(chunk).unwrap();
                        feed.push(chunk, false);
                    }
                    frame += 1;
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
                if elapsed.as_secs() >= next_sample {
                    let sampled = process_stats();
                    peak.0 = peak.0.max(sampled.0);
                    peak.1 = peak.1.max(sampled.1);
                    next_sample += 5;
                }
            }
            for feed in feeds {
                feed.finish();
            }
            (
                pipelines
                    .into_iter()
                    .map(|p| p.join().unwrap())
                    .collect::<Vec<_>>(),
                recognitions
                    .into_iter()
                    .map(|r| r.join().unwrap())
                    .collect::<Vec<_>>(),
            )
        });
        let wall = start.elapsed().as_secs_f64();
        let after = process_stats();
        eprintln!(
            "ticket07 {count} diarizer (guasto={fault}): audio={} ms, wall={wall:.2} s, CPU={:.2} s/{:.2} core equivalenti, WS picco campionato={} MiB, private picco campionato={} MiB; esiti={analyzed:?}",
            pcm.len() * 1000 / 16000,
            after.2 - loaded.2,
            (after.2 - loaded.2) / wall,
            peak.0 / 1048576,
            peak.1 / 1048576
        );
        assert!(asr.iter().all(Result::is_ok), "{asr:?}");
        assert!(
            analyzed
                .iter()
                .any(|(i, r)| *i == Ingresso::Sistema && r.is_ok())
        );
        if fault {
            assert!(
                analyzed.iter().any(|(i, r)| *i == Ingresso::Microfono
                    && *r == Err(AppError::LiveDiarizationLagging))
            );
        }
        if !fault {
            assert!(
                analyzed.iter().all(|(_, result)| result.is_ok()),
                "{analyzed:?}"
            );
        }
        for writer in writers {
            writer.finish().unwrap();
        }
        let mut transcript = Transcript {
            title: "Due Ingressi".into(),
            date: "2026-10-06 00:00".into(),
            durata_ms: Some((pcm.len() * 1000 / 16000) as u32),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::from("it"),
            phrases: plain.into_inner().unwrap(),
            live_asr: Vec::new(),
            parlanti: Default::default(),
            diarizzazione: None,
        };
        for state in states {
            let state = state.into_inner().unwrap();
            transcript.live_asr.extend(state.originals().cloned());
            for (_, p) in state.phrases {
                transcript.insert(p);
            }
        }
        for ingresso in inputs {
            assert!(
                transcript
                    .phrases
                    .iter()
                    .any(|p| p.ingresso == ingresso && p.fine_ms > 20000)
            );
        }
        if count == 1 {
            assert!(
                transcript
                    .phrases
                    .iter()
                    .filter(|p| p.ingresso == Ingresso::Microfono)
                    .all(|p| p.parlante.is_none()
                        && !p.parlante_non_determinato
                        && !p.parlante_provvisorio)
            );
        }
        drop(engines);
        drop(diarizers);
        let mut final_model = OfflineDiarizer::load_nemotron3(&model).unwrap();
        let system_turns = final_model
            .diarize_saved(
                Decoder::open(&folder.join("sistema.ogg")).unwrap(),
                &CancelToken::new(),
            )
            .unwrap();
        let mut final_results = vec![(Ingresso::Sistema, Ok(system_turns))];
        if count == 2 {
            let result = if fault {
                Err(AppError::Internal(
                    "guasto finale isolato nella prova".into(),
                ))
            } else {
                let mut model = OfflineDiarizer::load_nemotron3(&model).unwrap();
                model.diarize_saved(
                    Decoder::open(&folder.join("microfono.ogg")).unwrap(),
                    &CancelToken::new(),
                )
            };
            final_results.push((Ingresso::Microfono, result));
        }
        crate::engine::diarize::finalize_live_ingressi(
            &mut transcript,
            Diarizer::Nemotron3,
            final_results,
        );
        assert_eq!(
            transcript.diarizzazione.as_ref().unwrap().ingressi[0].esito,
            EsitoDiarizzazione::Completata
        );
        let audio = [
            (Ingresso::Mix, folder.join("mix.ogg")),
            (Ingresso::Microfono, folder.join("microfono.ogg")),
            (Ingresso::Sistema, folder.join("sistema.ogg")),
        ];
        let mut document = tape::Document::new(
            tape::creato(chrono::Local::now()),
            transcript.durata_ms.unwrap(),
            tape::Modalita::IngressiSeparati,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            true,
            &transcript.phrases,
        );
        document.diarizzazione = transcript.diarizzazione;
        let path = folder.join("Due.tape");
        tape::write(
            &path,
            &audio
                .iter()
                .map(|(i, p)| (*i, p.as_path()))
                .collect::<Vec<_>>(),
            &document,
            None,
        )
        .unwrap();
        assert_eq!(tape::read(&path).unwrap(), document);
        let reopened = crate::managers::transcription::open_tape(&path).unwrap();
        assert_eq!(reopened.info.diarizzazione, document.diarizzazione);
        for ingresso in inputs {
            assert!(reopened.phrases.iter().any(|p| p.ingresso == ingresso));
        }
        for (ingresso, original) in audio {
            let mut saved = Vec::new();
            use std::io::Read;
            tape::Mix::open_ingresso(&path, ingresso)
                .unwrap()
                .read_to_end(&mut saved)
                .unwrap();
            assert_eq!(saved, std::fs::read(original).unwrap());
        }
    }
}
