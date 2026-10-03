//! Fonte di frame della Trascrizione dal vivo. La Registrazione spinge l'uscita del mixer in un
//! canale senza limite (`LiveFeed`) e la pipeline la legge in un altro thread (`LiveFrames`): la
//! Registrazione non aspetta mai il motore, che se resta indietro accoda i frame e dopo Stop smaltisce
//! la coda con il progresso.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

use super::pipeline::Feed;
use crate::audio_toolkit::resample::FrameResampler;
use crate::error::AppError;

/// Quello che la Registrazione dice alla pipeline per il progresso dello smaltimento.
#[derive(Default)]
struct Shared {
    /// Frame da 30 ms mandati.
    sent: AtomicUsize,
    /// La Registrazione è finita: non arrivano altri frame.
    finished: AtomicBool,
}

/// Il canale tra la Registrazione e la pipeline. `rate` e `channels` sono quelli dell'uscita del
/// mixer.
pub fn channel(rate: u32, channels: usize) -> Result<(LiveFeed, LiveFrames), AppError> {
    let (tx, rx) = mpsc::channel();
    let shared = Arc::new(Shared::default());
    let feed = LiveFeed {
        tx,
        shared: Arc::clone(&shared),
        channels,
        resampler: FrameResampler::new(rate)?,
        mono: Vec::new(),
        paused: false,
    };
    let frames = LiveFrames {
        rx,
        shared,
        consumed: 0,
        progress: None,
    };
    Ok((feed, frames))
}

/// Il lato della Registrazione: scende in mono, ricampiona a 16 kHz e manda i frame.
pub struct LiveFeed {
    tx: Sender<Feed>,
    shared: Arc<Shared>,
    channels: usize,
    resampler: FrameResampler,
    mono: Vec<f32>,
    paused: bool,
}

impl LiveFeed {
    /// `samples`: l'uscita del mixer, interleaved. All'inizio di una pausa è l'audio che il mixer
    /// ha svuotato fino a lì, e dopo di esso la Frase in corso si chiude; poi, in pausa, è vuota.
    pub fn push(&mut self, samples: &[f32], paused: bool) {
        let channels = self.channels;
        self.mono.clear();
        self.mono.extend(
            samples
                .chunks_exact(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32),
        );
        let frames = self.resampler.push(&self.mono);
        self.send_frames(frames);
        if paused && !self.paused {
            self.send(Feed::ClosePhrase);
        }
        self.paused = paused;
    }

    /// Fine della Registrazione: manda l'ultimo frame, completato con zeri, e chiude il canale.
    pub fn finish(mut self) {
        let frames = self.resampler.finish();
        self.send_frames(frames);
        self.shared.finished.store(true, Ordering::Release);
    }

    fn send_frames(&mut self, frames: Vec<Vec<f32>>) {
        for frame in frames {
            // Contato prima di mandarlo: la pipeline non ne consuma mai più di quelli mandati.
            self.shared.sent.fetch_add(1, Ordering::Relaxed);
            self.send(Feed::Frame(frame));
        }
    }

    fn send(&self, input: Feed) {
        // Una pipeline già uscita (modello assente, errore, Annulla) non riceve più: la Registrazione
        // continua lo stesso.
        let _ = self.tx.send(input);
    }
}

/// Il lato della pipeline: i frame nell'ordine in cui la Registrazione li ha mandati. Finita la
/// Registrazione, prima di ogni frame emette il progresso dello smaltimento quando cambia.
pub struct LiveFrames {
    rx: Receiver<Feed>,
    shared: Arc<Shared>,
    consumed: usize,
    progress: Option<u8>,
}

impl LiveFrames {
    /// La percentuale di frame consumati, se la Registrazione è finita e non è già stata emessa.
    fn progress_changed(&mut self) -> Option<u8> {
        if !self.shared.finished.load(Ordering::Acquire) {
            return None;
        }
        let sent = self.shared.sent.load(Ordering::Relaxed).max(1);
        let percent = u8::try_from(self.consumed * 100 / sent).map_or(100, |p| p.min(100));
        (self.progress != Some(percent)).then(|| *self.progress.insert(percent))
    }
}

impl Iterator for LiveFrames {
    type Item = Result<Feed, AppError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(percent) = self.progress_changed() {
            return Some(Ok(Feed::Progress(Some(percent))));
        }
        let input = self.rx.recv().ok()?;
        if matches!(input, Feed::Frame(_)) {
            self.consumed += 1;
        }
        Some(Ok(input))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    use transcribe_cpp::CancelToken;

    use super::*;
    use crate::audio_toolkit::resample::FRAME_SAMPLES;
    use crate::engine::pipeline::tests::{EnergyDetector, FakeEngine};
    use crate::engine::pipeline::{PipelineEvent, transcribe};

    /// `n` frame da 30 ms di parlato (`true`) o silenzio a `rate`, con `channels` canali.
    fn audio(rate: u32, channels: usize, parts: &[(bool, usize)]) -> Vec<f32> {
        let per_frame = rate as usize * 30 / 1000;
        parts
            .iter()
            .flat_map(|&(speech, n)| (0..n * per_frame).map(move |i| (speech, i)))
            .flat_map(|(speech, i)| {
                let t = i as f32 / rate as f32;
                let s = if speech {
                    (t * 300.0 * std::f32::consts::TAU).sin() * 0.3
                } else {
                    0.0
                };
                std::iter::repeat_n(s, channels)
            })
            .collect()
    }

    fn run(
        frames: &mut LiveFrames,
        engine: &mut FakeEngine,
        cancel: &CancelToken,
        on_event: &mut dyn FnMut(PipelineEvent),
    ) -> Result<(), AppError> {
        transcribe(frames, engine, &mut EnergyDetector, None, cancel, on_event)
    }

    /// Le Frasi come `(id, inizio_ms, fine_ms)`.
    fn phrase(event: &PipelineEvent) -> Option<(u32, u32, u32)> {
        match event {
            PipelineEvent::Phrase {
                id,
                inizio_ms,
                fine_ms,
                ..
            } => Some((*id, *inizio_ms, *fine_ms)),
            _ => None,
        }
    }

    const TRE_FRASI: &[(bool, usize)] = &[
        (true, 30),
        (false, 50),
        (true, 30),
        (false, 50),
        (true, 30),
        (false, 30),
    ];

    #[test]
    fn con_un_motore_piu_lento_dell_audio_le_frasi_arrivano_tutte_smaltendo_la_coda_dopo_stop() {
        let (mut feed, mut frames) = channel(48_000, 2).unwrap();
        let emitted = Arc::new(AtomicU32::new(0));
        let feeder = std::thread::spawn({
            let emitted = Arc::clone(&emitted);
            move || {
                // Blocchi da 10 ms, come l'uscita del mixer, senza mai aspettare la pipeline.
                for block in audio(48_000, 2, TRE_FRASI).chunks(960) {
                    feed.push(block, false);
                }
                feed.finish();
                emitted.load(Ordering::SeqCst)
            }
        });
        let mut engine = FakeEngine {
            delay: Duration::from_millis(300),
            ..FakeEngine::default()
        };
        let mut events = Vec::new();
        run(
            &mut frames,
            &mut engine,
            &CancelToken::new(),
            &mut |event| {
                if matches!(event, PipelineEvent::Phrase { .. }) {
                    emitted.fetch_add(1, Ordering::SeqCst);
                }
                events.push(event);
            },
        )
        .unwrap();
        // La Registrazione è finita prima che il motore avesse trascritto tutto.
        assert!(feeder.join().unwrap() < 3);
        let ids: Vec<u32> = events.iter().filter_map(phrase).map(|p| p.0).collect();
        assert_eq!(ids, [0, 1, 2]);
        // Il progresso dello smaltimento cresce fino a 100, dove ogni frame è stato consumato.
        let percents: Vec<u8> = events
            .iter()
            .filter_map(|event| match event {
                PipelineEvent::Progress(percent) => *percent,
                _ => None,
            })
            .collect();
        assert!(percents.windows(2).all(|w| w[0] < w[1]), "{percents:?}");
        assert_eq!(percents.last(), Some(&100));
        // 6,6 s a 48 kHz stereo diventano 220 frame a 16 kHz mono.
        assert_eq!(engine.samples.len(), 3);
        assert_eq!(frames.consumed, 220);
    }

    #[test]
    fn la_pausa_chiude_la_frase_e_il_tempo_esclude_la_pausa() {
        let (mut feed, mut frames) = channel(16_000, 1).unwrap();
        feed.push(&audio(16_000, 1, &[(false, 10), (true, 20)]), false);
        // All'inizio della pausa arriva l'audio che il mixer tratteneva, poi più nulla.
        feed.push(&audio(16_000, 1, &[(true, 10)]), true);
        for _ in 0..5 {
            feed.push(&[], true);
        }
        feed.push(&audio(16_000, 1, &[(true, 30), (false, 50)]), false);
        feed.finish();
        let mut engine = FakeEngine::default();
        let mut phrases = Vec::new();
        run(
            &mut frames,
            &mut engine,
            &CancelToken::new(),
            &mut |event| {
                phrases.extend(phrase(&event));
            },
        )
        .unwrap();
        // La prima Frase finisce alla Pausa, la seconda riparte da lì senza prefill.
        assert_eq!(phrases, [(0, 0, 1200), (1, 1200, 2820)]);
        assert_eq!(engine.samples[0], 40 * FRAME_SAMPLES);
    }

    #[test]
    fn annulla_durante_lo_smaltimento_ferma_la_pipeline() {
        let (mut feed, mut frames) = channel(16_000, 1).unwrap();
        feed.push(&audio(16_000, 1, TRE_FRASI), false);
        feed.finish();
        let cancel = CancelToken::new();
        let mut engine = FakeEngine {
            cancel_at: Some((2, cancel.clone())),
            ..FakeEngine::default()
        };
        let mut phrases = Vec::new();
        let error = run(&mut frames, &mut engine, &cancel, &mut |event| {
            phrases.extend(phrase(&event));
        })
        .unwrap_err();
        assert!(matches!(error, AppError::Cancelled), "{error:?}");
        assert_eq!(phrases.len(), 1);
        assert_eq!(engine.samples.len(), 2);
    }

    #[test]
    fn senza_pipeline_la_registrazione_continua() {
        let (mut feed, frames) = channel(16_000, 1).unwrap();
        drop(frames);
        feed.push(&audio(16_000, 1, &[(true, 10)]), true);
        feed.push(&audio(16_000, 1, &[(true, 10)]), false);
        feed.finish();
    }
}
