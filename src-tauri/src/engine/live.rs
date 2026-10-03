//! Fonte di frame della Trascrizione dal vivo. La Registrazione spinge l'uscita del mixer (il mix, o
//! con gli Ingressi separati ogni Ingresso) in un canale senza limite (`LiveFeed`) e una pipeline la
//! legge in un altro thread (`LiveFrames`): la Registrazione non aspetta mai il motore, che se resta
//! indietro accoda i frame e dopo Stop smaltisce la coda con il progresso.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

use super::pipeline::Feed;
use crate::audio_toolkit::resample::FrameResampler;
use crate::error::AppError;

/// Il progresso dello smaltimento, comune a tutti i canali di una Registrazione.
struct Shared {
    /// Frame da 30 ms mandati.
    sent: AtomicUsize,
    /// Frame letti dalle pipeline.
    consumed: AtomicUsize,
    /// I canali la cui Registrazione non è ancora finita: a zero non arrivano altri frame.
    running: AtomicUsize,
    /// La percentuale più alta emessa, più uno (zero: nessuna): con due pipeline il progresso non
    /// torna indietro.
    emitted: AtomicUsize,
}

/// I canali tra la Registrazione e le pipeline, uno per audio trascritto: il mix, o ogni Ingresso.
/// Il progresso dello smaltimento conta i frame di tutti. `rate` e `channels` sono quelli
/// dell'uscita del mixer.
pub fn channels(
    rate: u32,
    channels: usize,
    n: usize,
) -> Result<Vec<(LiveFeed, LiveFrames)>, AppError> {
    let shared = Arc::new(Shared {
        sent: AtomicUsize::new(0),
        consumed: AtomicUsize::new(0),
        running: AtomicUsize::new(n),
        emitted: AtomicUsize::new(0),
    });
    (0..n)
        .map(|_| {
            let (tx, rx) = mpsc::channel();
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
                shared: Arc::clone(&shared),
            };
            Ok((feed, frames))
        })
        .collect()
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
        self.shared.running.fetch_sub(1, Ordering::Release);
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
        // continua lo stesso, e i suoi frame non contano nel progresso delle altre.
        if let Err(mpsc::SendError(Feed::Frame(_))) = self.tx.send(input) {
            self.shared.sent.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// Il lato della pipeline: i frame nell'ordine in cui la Registrazione li ha mandati. Finita la
/// Registrazione, prima di ogni frame emette il progresso dello smaltimento quando cambia.
pub struct LiveFrames {
    rx: Receiver<Feed>,
    shared: Arc<Shared>,
}

impl LiveFrames {
    /// La percentuale di frame consumati da tutte le pipeline, se la Registrazione è finita e supera
    /// quelle già emesse.
    fn progress_changed(&self) -> Option<u8> {
        if self.shared.running.load(Ordering::Acquire) > 0 {
            return None;
        }
        let sent = self.shared.sent.load(Ordering::Relaxed).max(1);
        let consumed = self.shared.consumed.load(Ordering::Relaxed);
        let percent = u8::try_from(consumed * 100 / sent).map_or(100, |p| p.min(100));
        let next = usize::from(percent) + 1;
        (self.shared.emitted.fetch_max(next, Ordering::Relaxed) < next).then_some(percent)
    }
}

impl Drop for LiveFrames {
    /// Una pipeline uscita prima della fine lascia frame che nessuno consumerà: non contano nel
    /// progresso delle altre.
    fn drop(&mut self) {
        let left = self
            .rx
            .try_iter()
            .filter(|input| matches!(input, Feed::Frame(_)))
            .count();
        self.shared.sent.fetch_sub(left, Ordering::Relaxed);
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
            self.shared.consumed.fetch_add(1, Ordering::Relaxed);
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
    use crate::managers::settings::SpeechLanguage;
    use crate::transcript::{Ingresso, Phrase, Transcript};

    /// Un solo canale, come per il mix.
    fn channel(rate: u32, channels: usize) -> Result<(LiveFeed, LiveFrames), AppError> {
        Ok(super::channels(rate, channels, 1)?.remove(0))
    }

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
    ) -> Result<u32, AppError> {
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
        assert_eq!(frames.shared.consumed.load(Ordering::Relaxed), 220);
    }

    #[test]
    fn con_due_ingressi_le_frasi_restano_in_ordine_di_inizio_e_il_progresso_le_conta_entrambe() {
        let mut pairs = channels(16_000, 1, 2).unwrap();
        let (mut system_feed, system_frames) = pairs.pop().unwrap();
        let (mut mic_feed, mic_frames) = pairs.pop().unwrap();
        // Il microfono parla per primo e alla fine, l'audio di sistema in mezzo; i due Ingressi sono
        // allineati, lunghi uguali.
        let mic = audio(
            16_000,
            1,
            &[(true, 30), (false, 100), (true, 30), (false, 40)],
        );
        let system = audio(16_000, 1, &[(false, 60), (true, 30), (false, 110)]);
        for (mic, system) in mic.chunks(160).zip(system.chunks(160)) {
            mic_feed.push(mic, false);
            system_feed.push(system, false);
        }
        mic_feed.finish();
        system_feed.finish();
        let transcript = std::sync::Mutex::new(Transcript {
            title: String::new(),
            date: String::new(),
            durata_ms: None,
            model: String::new(),
            speech_language: SpeechLanguage::Auto,
            phrases: Vec::new(),
        });
        let progress = std::sync::Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            // Il motore del microfono è lento: la Frase dell'audio di sistema arriva prima della
            // sua prima Frase, che però è iniziata prima.
            for (ingresso, mut frames, delay) in [
                (Ingresso::Microfono, mic_frames, 300),
                (Ingresso::Sistema, system_frames, 0),
            ] {
                let (transcript, progress) = (&transcript, &progress);
                scope.spawn(move || {
                    let mut engine = FakeEngine {
                        delay: Duration::from_millis(delay),
                        ..FakeEngine::default()
                    };
                    run(
                        &mut frames,
                        &mut engine,
                        &CancelToken::new(),
                        &mut |event| match event {
                            PipelineEvent::Phrase {
                                inizio_ms,
                                fine_ms,
                                text,
                                ..
                            } => transcript.lock().unwrap().insert(Phrase {
                                inizio_ms,
                                fine_ms,
                                text,
                                ingresso,
                                parlante: None,
                            }),
                            PipelineEvent::Progress(percent) => {
                                progress.lock().unwrap().extend(percent);
                            }
                            PipelineEvent::Partial { .. } => {}
                        },
                    )
                    .unwrap();
                });
            }
        });
        let phrases: Vec<(Ingresso, u32)> = transcript
            .into_inner()
            .unwrap()
            .phrases
            .iter()
            .map(|p| (p.ingresso, p.inizio_ms))
            .collect();
        assert_eq!(
            phrases,
            [
                (Ingresso::Microfono, 0),
                (Ingresso::Sistema, 1500),
                (Ingresso::Microfono, 3600)
            ]
        );
        // Il 100 arriva quando entrambe le code sono smaltite.
        let progress = progress.into_inner().unwrap();
        assert_eq!(progress.iter().max(), Some(&100), "{progress:?}");
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
    fn se_una_pipeline_esce_prima_il_progresso_delle_altre_arriva_a_100() {
        let mut pairs = channels(16_000, 1, 2).unwrap();
        let (mut dead_feed, dead_frames) = pairs.pop().unwrap();
        let (mut feed, mut frames) = pairs.pop().unwrap();
        let block = audio(16_000, 1, &[(true, 20)]);
        // La pipeline del secondo Ingresso esce con dei frame ancora in coda, poi ne arrivano altri.
        dead_feed.push(&block, false);
        drop(dead_frames);
        dead_feed.push(&block, false);
        feed.push(&block, false);
        dead_feed.finish();
        feed.finish();
        let percents: Vec<u8> = frames
            .by_ref()
            .filter_map(|input| match input.unwrap() {
                Feed::Progress(percent) => percent,
                _ => None,
            })
            .collect();
        assert_eq!(percents.last(), Some(&100), "{percents:?}");
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
