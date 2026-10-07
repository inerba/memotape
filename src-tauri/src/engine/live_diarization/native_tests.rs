//! Smoke esplicito su WASAPI: riproduce la fixture sull'uscita predefinita, non cambia Impostazioni.
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use crate::audio_toolkit::capture::{self, Capture, Kind};
use crate::audio_toolkit::decode::Decoder;
use crate::audio_toolkit::mixer::Mixer;
use crate::audio_toolkit::ogg_opus::{OggOpusWriter, tests::temp_dir};
use crate::audio_toolkit::vad::Silero;
use crate::engine::{
    diarize, live, pipeline,
    transcribe_cpp::{OfflineDiarizer, TranscribeCpp},
};
use crate::managers::{
    models,
    settings::{Diarizer, SpeechLanguage},
    transcription,
};
use crate::{tape, transcript::EsitoDiarizzazione};

#[test]
#[ignore = "WASAPI, riproduzione audio e modelli reali; eseguire in sequenza"]
fn un_ingresso_nativo_con_guasto_dei_parlanti_conserva_audio_testo_e_tape() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let folder = temp_dir("parlanti-dal-vivo-nativo");
    let ogg = folder.join("Registrazione.ogg");
    let model = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let mut engine = TranscribeCpp::load(&models::default_model().path(&models_dir)).unwrap();
    let mut diarizer = OfflineDiarizer::load_nemotron3(&model).unwrap();
    // Il riscaldamento GPU è fuori dalla misura/coda della cattura.
    diarizer
        .diarize(&[0.0; 16_000], &CancelToken::new())
        .unwrap();
    let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
    let (blocks, captured) = capture::channel(1);
    let capture = Capture::open(Kind::System, None, 0, blocks).unwrap();
    let mut mixer = Mixer::new(
        capture.now(),
        &[(capture.rate, capture.channels)],
        (16_000, 1),
    )
    .unwrap();
    let (mut feed, mut frames) = live::channels(16_000, 1, 1).unwrap().remove(0);
    let (tx, mut audio) = channel();
    feed.set_diarization(tx);
    let state = Mutex::new(LiveTranscript::default());
    let partial_speaker = AtomicBool::new(false);
    let timed_partial = AtomicBool::new(false);
    let divided = AtomicBool::new(false);
    let observed = std::sync::atomic::AtomicU32::new(0);
    let inspect = |state: &LiveTranscript| {
        let previous = observed.swap(state.revision, Ordering::Relaxed);
        assert!(state.revision >= previous);
        let ids = state
            .phrases
            .iter()
            .chain(&state.partials)
            .map(|(id, _)| *id)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), state.phrases.len() + state.partials.len());
        assert_eq!(
            state
                .phrases
                .iter()
                .map(|(_, p)| p.text.as_str())
                .collect::<String>(),
            state
                .originals()
                .map(|p| p.text.as_str())
                .collect::<String>()
        );
        assert_eq!(
            state
                .partials
                .iter()
                .map(|(_, p)| p.text.as_str())
                .collect::<String>(),
            state
                .partial
                .as_ref()
                .map(|(_, p)| p.text.clone())
                .unwrap_or_default()
        );
        if state.partials.iter().any(|(_, p)| !p.tempi.is_empty()) {
            timed_partial.store(true, Ordering::Relaxed);
        }
        if state.partials.len() > 1 || state.phrases.len() > state.originals().count() {
            divided.store(true, Ordering::Relaxed);
        }
    };
    let cancel = CancelToken::new();
    let mut writer =
        OggOpusWriter::new(std::fs::File::create(&ogg).unwrap(), 16_000, 1, 64).unwrap();
    let fixture = root.join("tests/fixtures/parlato-due-voci.wav");
    let player_script = format!(
        "$p = New-Object System.Media.SoundPlayer '{}'; $p.PlaySync()",
        fixture.display().to_string().replace('\'', "''")
    );
    let mut player = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &player_script])
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();
    let (asr, analyzed) = std::thread::scope(|scope| {
        let transcription = scope.spawn(|| {
            pipeline::transcribe(
                &mut frames,
                &mut engine,
                &mut detector,
                Some("it"),
                &cancel,
                &mut |event| {
                    let mut state = state.lock().unwrap();
                    state.on_asr(Ingresso::Mix, event);
                    inspect(&state);
                    if state.partials.iter().any(|(_, p)| p.parlante.is_some()) {
                        partial_speaker.store(true, Ordering::Relaxed);
                    }
                },
            )
        });
        let recognition = scope.spawn(|| {
            let mut delayed = false;
            diarizer.diarize_live(&mut audio, &cancel, &mut |turns| {
                let end = turns.iter().map(|t| t.fine_ms).max().unwrap_or(0);
                let mut state = state.lock().unwrap();
                state.on_turns(turns);
                inspect(&state);
                if state.partials.iter().any(|(_, p)| p.parlante.is_some()) {
                    partial_speaker.store(true, Ordering::Relaxed);
                }
                drop(state);
                if end >= 9000 && !delayed {
                    delayed = true;
                    // Guasto riproducibile: la coda satura mentre cattura e ASR continuano.
                    std::thread::sleep(Duration::from_secs(4));
                }
            })
        });
        let start = Instant::now();
        let mut out = Vec::new();
        while start.elapsed() < Duration::from_secs(29) {
            let paused = (14..15).contains(&start.elapsed().as_secs());
            mixer.advance(capture.now(), paused, &mut out).unwrap();
            while let Ok(block) = captured.try_recv() {
                mixer
                    .push(block.input, block.capture_ns, &block.samples, &mut out)
                    .unwrap();
                capture.recycle(block.samples);
            }
            writer.write(&out).unwrap();
            feed.push(&out, paused);
            out.clear();
            std::thread::sleep(Duration::from_millis(10));
        }
        mixer.finish(capture.now(), &mut out).unwrap();
        writer.write(&out).unwrap();
        feed.push(&out, false);
        feed.finish();
        (transcription.join().unwrap(), recognition.join().unwrap())
    });
    assert!(player.wait().unwrap().success());
    drop(capture);
    let waveform = writer.forma_onda();
    writer.finish().unwrap();
    assert!(asr.is_ok(), "{asr:?}");
    assert_eq!(analyzed, Err(AppError::LiveDiarizationLagging));
    assert!(
        partial_speaker.load(Ordering::Relaxed),
        "nessun Parziale ha ricevuto un Parlante durante la cattura"
    );
    assert!(
        timed_partial.load(Ordering::Relaxed),
        "nessun Parziale con tempi affidabili"
    );
    assert!(
        divided.load(Ordering::Relaxed),
        "nessuna divisione dal vivo osservata"
    );
    let state = state.into_inner().unwrap();
    let originals: Vec<_> = state.originals().cloned().collect();
    let mut phrases: Vec<_> = state.phrases.into_iter().map(|(_, p)| p).collect();
    assert!(!phrases.is_empty());
    let text: String = phrases.iter().map(|p| p.text.as_str()).collect();
    assert!(text.to_lowercase().contains("riunione"), "{text}");
    assert!(
        text.to_lowercase().contains("vendite"),
        "manca il testo dopo il guasto: {text}"
    );
    assert!(
        phrases.iter().any(|p| p.fine_ms > 20_000),
        "ASR non ha proseguito dopo il guasto"
    );
    let duration = mixer.elapsed_ms();
    assert!((27_800..28_200).contains(&duration), "{duration}");
    let final_cancel = CancelToken::new();
    final_cancel.cancel();
    let unfinished = diarize::finalize(
        &mut phrases,
        Diarizer::Nemotron3,
        &final_cancel,
        Err(AppError::Cancelled),
    )
    .unwrap();
    assert_eq!(unfinished.esito, EsitoDiarizzazione::Annullata);
    let mut document = tape::Document::new(
        tape::creato(chrono::Local::now()),
        duration,
        tape::Modalita::Mix,
        Some(models::default_model().id.clone()),
        SpeechLanguage::from("it"),
        true,
        &phrases,
    );
    document.diarizzazione = Some(unfinished);
    let tape_path = folder.join("Registrazione.tape");
    tape::write(
        &tape_path,
        &[(Ingresso::Mix, &ogg)],
        &document,
        Some(&waveform),
    )
    .unwrap();
    assert_eq!(tape::read(&tape_path).unwrap(), document);
    assert!(transcription::open_tape(&tape_path).unwrap().info.completa);
    let mut saved_audio = Vec::new();
    tape::Mix::open(&tape_path)
        .unwrap()
        .read_to_end(&mut saved_audio)
        .unwrap();
    assert_eq!(saved_audio, std::fs::read(&ogg).unwrap());
    let mut decoder = Decoder::open(&tape_path).unwrap();
    let mut decoded_ms = 0.0;
    let mut peak = 0.0_f32;
    while let Some(block) = decoder.next_block().unwrap() {
        decoded_ms +=
            block.samples.len() as f64 * 1000.0 / f64::from(block.rate) / block.channels as f64;
        peak = block.samples.iter().fold(peak, |max, s| max.max(s.abs()));
    }
    assert!(peak > 0.01);
    assert!(
        (decoded_ms - f64::from(duration)).abs() < 100.0,
        "audio={decoded_ms} ms, timer={duration} ms"
    );
    // La sessione fallita non impedisce il tentativo finale sull'audio salvato.
    let mut final_model = OfflineDiarizer::load_nemotron3(&model).unwrap();
    let final_turns = final_model
        .diarize_saved(Decoder::open(&ogg).unwrap(), &CancelToken::new())
        .unwrap();
    let mut transcript = crate::transcript::Transcript {
        title: "Registrazione".into(),
        date: "2026-10-06 00:00".into(),
        durata_ms: Some(duration),
        model: "Nemotron".into(),
        speech_language: SpeechLanguage::from("it"),
        phrases,
        live_asr: originals,
        parlanti: Default::default(),
        diarizzazione: None,
    };
    diarize::finalize_live_ingressi(
        &mut transcript,
        Diarizer::Nemotron3,
        vec![(Ingresso::Mix, Ok(final_turns))],
    );
    assert_eq!(
        transcript.diarizzazione.as_ref().unwrap().esito,
        EsitoDiarizzazione::Completata
    );
    let phrases = &transcript.phrases;
    assert!(phrases.iter().all(|p| !p.parlante_provvisorio));
    assert_eq!(
        phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
        text
    );
    let mut final_document = tape::Document::new(
        document.creato.clone(),
        duration,
        tape::Modalita::Mix,
        document.modello.clone(),
        SpeechLanguage::from("it"),
        true,
        phrases,
    );
    final_document.diarizzazione = transcript.diarizzazione;
    let final_path = folder.join("Finale.tape");
    tape::write(
        &final_path,
        &[(Ingresso::Mix, &ogg)],
        &final_document,
        Some(&waveform),
    )
    .unwrap();
    assert_eq!(tape::read(&final_path).unwrap(), final_document);
    let reopened = transcription::open_tape(&final_path).unwrap();
    assert_eq!(reopened.phrases.len(), phrases.len());
    for (id, (visible, saved)) in phrases.iter().zip(&reopened.phrases).enumerate() {
        assert_eq!(saved.phrase_id, id as u32);
        assert_eq!(saved.text, visible.text);
        assert_eq!(
            (saved.inizio_ms, saved.fine_ms),
            (visible.inizio_ms, visible.fine_ms)
        );
        assert_eq!(saved.parlante, visible.parlante);
        assert_eq!(
            saved.parlante_non_determinato,
            visible.parlante_non_determinato
        );
        assert!(!saved.parlante_provvisorio);
    }
    eprintln!(
        "ticket06: {} revisioni, Parziali con tempi, divisioni dal vivo, Frasi/Tape/riapertura coerenti",
        observed.load(Ordering::Relaxed)
    );
    eprintln!(
        "smoke WASAPI: {} ms, {} Frasi, audio identico nel Tape, Parziale attribuito, errore isolato e analisi finale riuscita",
        duration,
        phrases.len()
    );
}
