use super::tests::FakeEngine;
use super::*;
use crate::audio_toolkit::protection::Sensibilita;
use crate::audio_toolkit::resample::FRAME_SAMPLES;

struct Detector;

#[test]
fn protezione_live_resta_attiva_sul_bypass_del_solo_ingresso_guasto() {
    use crate::audio_toolkit::{
        cleaning::{CleaningLog, ConfiguredCleaning},
        mixer::Mixer,
        processing::{AudioProcessor, Boundary, PcmBlock},
    };
    use crate::engine::live;
    use crate::transcript::Ingresso;
    struct Failure;
    impl AudioProcessor for Failure {
        fn max_pending_frames(&self) -> usize {
            960
        }
        fn process(&mut self, block: PcmBlock<'_>, _: &mut Vec<f32>) -> Result<(), AppError> {
            if block.start_frame == 0 {
                Ok(())
            } else {
                Err(AppError::AudioCleaningFailed("prova".into()))
            }
        }
        fn flush(&mut self, _: Boundary, _: &mut Vec<f32>) -> Result<(), AppError> {
            unreachable!()
        }
    }
    let warnings = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let notified = warnings.clone();
    let log = CleaningLog::default();
    let processor = ConfiguredCleaning::with_factory(
        Box::new(|| true),
        Ingresso::Microfono,
        log.clone(),
        Box::new(|_| Ok(Box::new(Failure))),
    )
    .recovering(move |error| {
        notified
            .lock()
            .unwrap()
            .push(("sessione06", Ingresso::Microfono, error))
    });
    let pairs = live::channels(16_000, 1, 2).unwrap();
    let timelines: Vec<_> = pairs.iter().map(|(f, _)| f.protection()).collect();
    let (mut feeds, sources): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    let mut mixer = Mixer::with_processors(
        0,
        &[(16_000, 1), (16_000, 1)],
        (16_000, 1),
        vec![Box::new(processor), Box::new(Bypass)],
    )
    .unwrap()
    .with_tracks();
    let noise: Vec<_> = (0..480)
        .map(|i| if i % 2 == 0 { 0.1 } else { -0.1 })
        .collect();
    let weak: Vec<_> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.00001).collect();
    let mut mix = Vec::new();
    let mut saved = [Vec::new(), Vec::new()];
    for i in 0..4 {
        for (k, timeline) in timelines.iter().enumerate() {
            mixer.protect(k, timeline, Sensibilita::Bilanciato);
        }
        mixer.push(0, i * 30_000_000, &noise, &mut mix).unwrap();
        mixer.push(1, i * 30_000_000, &weak, &mut mix).unwrap();
        for ((feed, pcm), all) in feeds.iter_mut().zip(mixer.tracks()).zip(&mut saved) {
            all.extend_from_slice(pcm);
            feed.push(pcm, false);
            pcm.clear();
        }
    }
    mixer.finish(120_000_000, &mut mix).unwrap();
    for ((feed, pcm), all) in feeds.iter_mut().zip(mixer.tracks()).zip(&mut saved) {
        all.extend_from_slice(pcm);
        feed.push(pcm, false);
    }
    for feed in feeds {
        feed.finish();
    }
    assert_eq!(saved[0], noise.repeat(4));
    assert_eq!(saved[1], weak.repeat(4));
    assert_eq!(warnings.lock().unwrap().len(), 1);
    assert_eq!(warnings.lock().unwrap()[0].1, Ingresso::Microfono);
    assert!(log.intervals().is_empty());
    for (i, mut frames) in sources.into_iter().enumerate() {
        let timeline = frames.protection();
        let mut engine = FakeEngine::default();
        transcribe_protected(
            &mut frames,
            &mut engine,
            &mut Detector,
            None,
            &CancelToken::new(),
            &mut |_| {},
            &timeline,
        )
        .unwrap();
        assert_eq!(engine.samples.len(), i);
        if i == 1 {
            assert_eq!(engine.pcm, saved[1]);
        }
    }
}

#[test]
fn protezione_live_revisione_prima_del_ricampionamento_senza_alterare_audio() {
    use crate::audio_toolkit::mixer::Mixer;
    let timeline = ProtectionTimeline::default();
    let mut mixer = Mixer::new(0, &[(44_100, 1)], (48_000, 1)).unwrap();
    let mut reference = Mixer::new(0, &[(44_100, 1)], (48_000, 1)).unwrap();
    let mut out = Vec::new();
    let mut expected = Vec::new();
    mixer.protect(0, &timeline, Sensibilita::Sensibile);
    mixer.push(0, 0, &[0.2; 441], &mut out).unwrap();
    reference.push(0, 0, &[0.2; 441], &mut expected).unwrap();
    mixer.protect(0, &timeline, Sensibilita::Selettivo);
    mixer.push(0, 10_000_000, &[0.1; 441], &mut out).unwrap();
    reference
        .push(0, 10_000_000, &[0.1; 441], &mut expected)
        .unwrap();
    mixer.finish(20_000_000, &mut out).unwrap();
    reference.finish(20_000_000, &mut expected).unwrap();
    assert_eq!(out, expected);
    assert_eq!(out.len(), 960);
    assert_eq!(timeline.level(159), Sensibilita::Sensibile);
    assert_eq!(timeline.level(160), Sensibilita::Selettivo);
}
impl VoiceDetector for Detector {
    fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError> {
        Ok(if frame.iter().any(|s| *s != 0.0) {
            0.45
        } else {
            0.0
        })
    }
    fn reset(&mut self) {}
}

#[test]
fn protezione_live_ammissione_irrevocabile_e_parziale_prima_di_fine_parlato() {
    let timeline = ProtectionTimeline::default();
    timeline.record(0, Sensibilita::Bilanciato);
    timeline.record(10 * 480, Sensibilita::Selettivo);
    let weak: Vec<_> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.00001).collect();
    let noise: Vec<_> = (0..480)
        .map(|i| if i % 2 == 0 { 0.1 } else { -0.1 })
        .collect();
    let source: Vec<_> = (0..100)
        .map(|i| if i < 10 { weak.clone() } else { noise.clone() })
        .collect();
    let read = Cell::new(0);
    let mut frames = source.iter().map(|frame| {
        read.set(read.get() + 1);
        Ok(Feed::Frame(frame.clone()))
    });
    let mut engine = FakeEngine {
        streaming: true,
        ..Default::default()
    };
    let mut partials = Vec::new();
    let mut phrases = Vec::new();
    transcribe_protected(
        &mut frames,
        &mut engine,
        &mut Detector,
        None,
        &CancelToken::new(),
        &mut |e| match e {
            PipelineEvent::Partial { fine_ms, text, .. } => {
                assert!(!text.is_empty());
                partials.push((read.get(), fine_ms));
            }
            PipelineEvent::Phrase {
                inizio_ms, fine_ms, ..
            } => phrases.push((inizio_ms, fine_ms)),
            _ => {}
        },
        &timeline,
    )
    .unwrap();
    assert_eq!(partials[0], (10, 300));
    assert_eq!(phrases, [(0, 3000)]);
    assert_eq!(engine.pcm, source.concat());
}

#[test]
fn protezione_live_coda_arretrata_ingressi_pause_stop_e_sessioni_indipendenti() {
    use crate::audio_toolkit::processing::{Boundary, Format, PcmStream, tests::DelayedScale};
    use crate::engine::live;
    let mut pairs = live::channels(16_000, 1, 2).unwrap();
    let (mut system, mut system_frames) = pairs.pop().unwrap();
    let (mut mic, mut mic_frames) = pairs.pop().unwrap();
    let level = std::sync::Arc::new(std::sync::Mutex::new(Sensibilita::Sensibile));
    let read = level.clone();
    let timeline = mic.protection();
    let mut pcm = PcmStream::new(
        Format {
            rate: 16_000,
            channels: 1,
        },
        timeline.capture(Box::new(DelayedScale::new(480)), move || {
            *read.lock().unwrap()
        }),
    )
    .unwrap();
    let weak: Vec<_> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.00001).collect();
    let mut saved: Vec<f32> = Vec::new();
    // Il PCM attende nel processore; nessun motore ha ancora consumato la coda.
    for _ in 0..4 {
        let out = pcm.push(&weak).unwrap().samples;
        saved.extend(&out);
        mic.push(&out, false);
    }
    let tail = pcm.boundary(Boundary::Pause).unwrap().samples;
    saved.extend(&tail);
    mic.push(&tail, true);
    *level.lock().unwrap() = Sensibilita::Selettivo; // Cambio durante Pausa.
    for _ in 0..4 {
        let out = pcm.push(&weak).unwrap().samples;
        saved.extend(&out);
        mic.push(&out, false);
    }
    let tail = pcm.boundary(Boundary::Finish).unwrap().samples;
    saved.extend(&tail);
    mic.push(&tail, false);
    mic.finish();
    system.protection().record(0, Sensibilita::Sensibile);
    system.push(&weak.repeat(8), false);
    system.finish();
    // Un cambio successivo a Stop non rilegge e non riscrive il PCM già in coda.
    *level.lock().unwrap() = Sensibilita::Spento;
    let mut mic_engine = FakeEngine::default();
    let mut system_engine = FakeEngine::default();
    let mut mic_phrases = Vec::new();
    let mic_timeline = mic_frames.protection();
    let system_timeline = system_frames.protection();
    let duration = transcribe_protected(
        &mut mic_frames,
        &mut mic_engine,
        &mut Detector,
        None,
        &CancelToken::new(),
        &mut |e| {
            if let PipelineEvent::Phrase {
                inizio_ms, fine_ms, ..
            } = e
            {
                mic_phrases.push((inizio_ms, fine_ms));
            }
        },
        &mic_timeline,
    )
    .unwrap();
    transcribe_protected(
        &mut system_frames,
        &mut system_engine,
        &mut Detector,
        None,
        &CancelToken::new(),
        &mut |_| {},
        &system_timeline,
    )
    .unwrap();
    assert_eq!(duration, 240);
    assert_eq!(mic_phrases, [(0, 120)]);
    assert_eq!(mic_engine.pcm, saved[..1920]);
    assert_eq!(
        saved,
        weak.repeat(8).iter().map(|s| s * 0.5).collect::<Vec<_>>()
    );
    assert_eq!(system_engine.pcm, weak.repeat(8));
    let (_, fresh) = live::channels(16_000, 1, 1).unwrap().remove(0);
    assert_eq!(fresh.protection().level(0), Sensibilita::Spento);
}

#[test]
fn protezione_live_cambio_sull_ultimo_frame_del_taglio_massimo_conserva_tutta_la_frase() {
    let timeline = ProtectionTimeline::default();
    timeline.record(0, Sensibilita::Bilanciato);
    timeline.record(599 * 480, Sensibilita::Spento);
    let frame: Vec<_> = (0..FRAME_SAMPLES)
        .map(|i| if i % 2 == 0 { 0.1 } else { -0.1 })
        .collect();
    let mut frames = (0..600).map(|_| Ok(Feed::Frame(frame.clone())));
    let mut engine = FakeEngine::default();
    let mut phrases = Vec::new();
    transcribe_protected(
        &mut frames,
        &mut engine,
        &mut Detector,
        None,
        &CancelToken::new(),
        &mut |e| {
            if let PipelineEvent::Phrase {
                inizio_ms, fine_ms, ..
            } = e
            {
                phrases.push((inizio_ms, fine_ms));
            }
        },
        &timeline,
    )
    .unwrap();
    assert_eq!(phrases, [(0, 18_000)]);
    assert_eq!(engine.pcm, frame.repeat(600));
}

#[test]
fn protezione_live_scarto_prima_di_asr_e_parziali_senza_cambiare_durata() {
    let timeline = ProtectionTimeline::default();
    timeline.record(0, Sensibilita::Bilanciato);
    let mut frames = (0..70).map(|i| {
        Ok(Feed::Frame(if i < 30 {
            (0..FRAME_SAMPLES)
                .map(|s| if s % 2 == 0 { 0.1 } else { -0.1 })
                .collect()
        } else {
            vec![0.0; FRAME_SAMPLES]
        }))
    });
    let mut engine = FakeEngine {
        streaming: true,
        ..Default::default()
    };
    let mut events = Vec::new();
    let duration = transcribe_protected(
        &mut frames,
        &mut engine,
        &mut Detector,
        None,
        &CancelToken::new(),
        &mut |e| events.push(e),
        &timeline,
    )
    .unwrap();
    assert_eq!(duration, 2100);
    assert!(engine.samples.is_empty());
    assert!(!events.iter().any(|e| matches!(
        e,
        PipelineEvent::Partial { .. } | PipelineEvent::Phrase { .. }
    )));
}
