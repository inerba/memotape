//! DeepFilterNet3 standard, con il runtime libDF originale fissato nel manifest.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::mpsc;
use std::thread::JoinHandle;

use df::tract::{DfParams, DfTract, RuntimeParams};
use sha2::{Digest, Sha256};

use super::processing::{AudioProcessor, Boundary, Format, PcmBlock};
use super::resample::Resampler;
use crate::error::AppError;

#[cfg(test)]
mod diagnostic_tests;
mod hop;
#[cfg(test)]
mod performance_tests;
mod prepared;
use hop::Hop;
pub(crate) use prepared::PreparedFilter;

pub const MODEL_FILE: &str = "resources/models/DeepFilterNet3_onnx.tar.gz";
const MODEL_SIZE: usize = 7_983_136;
const MODEL_SHA256: &str = "c94d91f70911001c946e0fabb4aa9adc37045f45a03b56008cb0c8244cb63616";
const RATE: u32 = 48_000;
const HOP: usize = 480;

pub fn validate(path: &Path) -> Result<(), AppError> {
    let bytes = std::fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::AudioCleaningMissing
        } else {
            AppError::AudioCleaningIncompatible(error.to_string())
        }
    })?;
    if bytes.len() != MODEL_SIZE || format!("{:x}", Sha256::digest(&bytes)) != MODEL_SHA256 {
        return Err(AppError::AudioCleaningIncompatible(
            "dimensione o SHA-256 non corrispondono a DFN3 standard".into(),
        ));
    }
    Ok(())
}

fn failed(error: impl std::fmt::Display) -> AppError {
    AppError::AudioCleaningFailed(error.to_string())
}

/// Stato di un segmento continuo nello stesso formato della Sorgente. Ogni canale conserva
/// inferenza e STFT indipendenti: nessuna conversione mono dell'audio salvato.
struct State {
    /// Piano ottimizzato senza audio precedente: i confini non ricaricano né ricompilano Tract.
    template: DfTract,
    format: Format,
    channels: Vec<Hop>,
    to_48: Resampler,
    from_48: Resampler,
    pending: Vec<f32>,
    input_48_frames: usize,
    processed_48_frames: usize,
    delay: usize,
    input_frames: usize,
    wet: VecDeque<f32>,
    emitted: usize,
}

impl State {
    fn new(path: &Path, format: Format) -> Result<Self, AppError> {
        validate(path)?;
        let params = DfParams::new(path.to_path_buf()).map_err(failed)?;
        // Uscita DFN3 standard, senza miscela aggiuntiva con l'originale, post-filter o AGC.
        let model = DfTract::new(params, &RuntimeParams::default().with_atten_lim(100.0))
            .map_err(failed)?;
        if model.sr != RATE as usize
            || model.hop_size != HOP
            || model.fft_size != 960
            || model.lookahead != 2
        {
            return Err(AppError::AudioCleaningIncompatible(
                "configurazione DFN3 standard inattesa".into(),
            ));
        }
        let delay = model.fft_size - model.hop_size + model.lookahead * model.hop_size;
        Self::from_model(model, format, delay)
    }

    fn from_model(model: DfTract, format: Format, delay: usize) -> Result<Self, AppError> {
        let channels = (0..format.channels)
            .map(|_| Hop::new(model.clone()))
            .collect();
        Ok(Self {
            template: model,
            format,
            channels,
            to_48: Resampler::new(format.rate, RATE, format.channels)?,
            from_48: Resampler::new(RATE, format.rate, format.channels)?,
            pending: Vec::new(),
            input_48_frames: 0,
            processed_48_frames: 0,
            delay,
            input_frames: 0,
            wet: VecDeque::new(),
            emitted: 0,
        })
    }

    fn infer(&mut self, samples: &[f32], finish: bool) -> Result<(), AppError> {
        self.input_48_frames += samples.len() / self.format.channels;
        self.pending.extend_from_slice(samples);
        let size = HOP * self.format.channels;
        if finish {
            self.pending.resize(
                (self.pending.len() / self.format.channels + self.delay).div_ceil(HOP) * size,
                0.0,
            );
        }
        let mut consumed = 0;
        while self.pending.len() - consumed >= size {
            let mut interleaved = vec![0.0; size];
            for (channel, state) in self.channels.iter_mut().enumerate() {
                let input: Vec<f32> = self.pending[consumed..consumed + size]
                    .chunks_exact(self.format.channels)
                    .map(|frame| frame[channel])
                    .collect();
                let mut output = vec![0.0; HOP];
                state.process(&input, &mut output)?;
                for (frame, value) in interleaved
                    .chunks_exact_mut(self.format.channels)
                    .zip(output)
                {
                    frame[channel] = value;
                }
            }
            let start = self.processed_48_frames;
            let useful_start = self.delay.saturating_sub(start).min(HOP);
            let useful_end = (self.input_48_frames + self.delay)
                .saturating_sub(start)
                .min(HOP);
            if useful_end > useful_start {
                let mut returned = Vec::new();
                self.from_48.push(
                    &interleaved
                        [useful_start * self.format.channels..useful_end * self.format.channels],
                    &mut returned,
                );
                self.wet.extend(returned);
            }
            self.processed_48_frames += HOP;
            consumed += size;
        }
        self.pending.drain(..consumed);
        Ok(())
    }

    fn emit(&mut self, finish: bool, out: &mut Vec<f32>) -> Result<(), AppError> {
        let channels = self.format.channels;
        // Il doppio arrotondamento del ricampionamento può produrre un frame oltre
        // la durata originale. Si consegna soltanto l'intervallo realmente acquisito.
        let frames = (self.wet.len() / channels).min(self.input_frames - self.emitted);
        for value in self.wet.drain(..frames * channels) {
            if !value.is_finite() {
                return Err(failed("campioni non finiti"));
            }
            out.push(value);
        }
        self.emitted += frames;
        if finish && self.emitted != self.input_frames {
            return Err(failed("coda PCM incompleta"));
        }
        if finish {
            self.wet.clear();
        }
        Ok(())
    }
}

impl State {
    fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
        if block.format != self.format {
            return Err(failed("il formato PCM cambia"));
        }
        self.input_frames += block.samples.len() / self.format.channels;
        let mut samples = Vec::new();
        self.to_48.push(block.samples, &mut samples);
        self.infer(&samples, false)?;
        self.emit(false, out)
    }

    fn flush(&mut self, _boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
        let mut samples = Vec::new();
        self.to_48.finish(&mut samples);
        self.infer(&samples, true)?;
        let mut returned = Vec::new();
        self.from_48.finish(&mut returned);
        self.wet.extend(returned);
        self.emit(true, out)
    }

    fn reset(&mut self) -> Result<(), AppError> {
        *self = Self::from_model(self.template.clone(), self.format, self.delay)?;
        Ok(())
    }
}

enum Request {
    Process(Vec<f32>, mpsc::SyncSender<Result<Vec<f32>, AppError>>),
    Flush(Boundary, mpsc::SyncSender<Result<Vec<f32>, AppError>>),
    Reset(mpsc::SyncSender<Result<Vec<f32>, AppError>>),
}

/// Tract 0.19.16 ha stato `Rc` non Send: resta sul thread che lo crea. Il contratto PCM
/// trasferisce soltanto campioni e risultati; nessun `unsafe impl Send` sul runtime.
pub struct DeepFilter {
    format: Format,
    sender: Option<mpsc::SyncSender<Request>>,
    worker: Option<JoinHandle<()>>,
    return_to: Option<std::sync::Weak<prepared::Slot>>,
    pristine: bool,
    finished: bool,
}

impl DeepFilter {
    pub fn new(path: &Path, format: Format) -> Result<Self, AppError> {
        Self::load(path, format, false)
    }

    fn load(path: &Path, format: Format, reusable: bool) -> Result<Self, AppError> {
        let path = path.to_path_buf();
        let (sender, receiver) = mpsc::sync_channel(1);
        let (ready, initialized) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("memotape-dfn3".into())
            .spawn(move || {
                let mut state = match State::new(&path, format) {
                    Ok(state) => {
                        let _ = ready.send(Ok(()));
                        state
                    }
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                while let Ok(request) = receiver.recv() {
                    let mut out = Vec::new();
                    let result = match request {
                        Request::Process(samples, response) => {
                            let result = state
                                .process(
                                    PcmBlock {
                                        format,
                                        start_frame: 0,
                                        samples: &samples,
                                    },
                                    &mut out,
                                )
                                .map(|()| out);
                            let failed = result.is_err();
                            let _ = response.send(result);
                            failed
                        }
                        Request::Flush(boundary, response) => {
                            let result = state.flush(boundary, &mut out).and_then(|()| {
                                if boundary != Boundary::Finish || reusable {
                                    state.reset()?;
                                }
                                Ok(out)
                            });
                            let failed = result.is_err();
                            let _ = response.send(result);
                            failed || (boundary == Boundary::Finish && !reusable)
                        }
                        Request::Reset(response) => {
                            let result = state.reset().map(|()| Vec::new());
                            let failed = result.is_err();
                            let _ = response.send(result);
                            failed
                        }
                    };
                    if result {
                        break;
                    }
                }
            })
            .map_err(failed)?;
        if let Err(error) = initialized.recv().map_err(failed).and_then(|result| result) {
            let _ = worker.join();
            return Err(error);
        }
        Ok(Self {
            format,
            sender: Some(sender),
            worker: Some(worker),
            return_to: None,
            pristine: true,
            finished: false,
        })
    }

    fn exchange(
        &self,
        make: impl FnOnce(mpsc::SyncSender<Result<Vec<f32>, AppError>>) -> Request,
        out: &mut Vec<f32>,
    ) -> Result<(), AppError> {
        let (response, returned) = mpsc::sync_channel(1);
        self.sender
            .as_ref()
            .ok_or_else(|| failed("processore chiuso"))?
            .send(make(response))
            .map_err(failed)?;
        out.extend(returned.recv().map_err(failed)??);
        Ok(())
    }
}

impl AudioProcessor for DeepFilter {
    fn max_pending_frames(&self) -> usize {
        self.format.rate as usize / 2 + 8192
    }
    fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
        if self.finished {
            return Err(failed("sessione del processore chiusa"));
        }
        if block.format != self.format {
            return Err(failed("il formato PCM cambia"));
        }
        self.pristine = false;
        self.exchange(
            |response| Request::Process(block.samples.to_vec(), response),
            out,
        )
    }
    fn flush(&mut self, boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
        self.exchange(|response| Request::Flush(boundary, response), out)?;
        self.pristine = true;
        self.finished = boundary == Boundary::Finish;
        Ok(())
    }
}

impl Drop for DeepFilter {
    fn drop(&mut self) {
        if let Some(slot) = self.return_to.take().and_then(|slot| slot.upgrade()) {
            let reset = if self.pristine {
                Ok(())
            } else {
                self.exchange(Request::Reset, &mut Vec::new())
            };
            if reset.is_ok() {
                slot.release(Ok(Self {
                    format: self.format,
                    sender: self.sender.take(),
                    worker: self.worker.take(),
                    return_to: None,
                    pristine: true,
                    finished: false,
                }));
                return;
            }
            if let Err(error) = reset {
                slot.release(Err(error));
            }
        }
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "diagnosi DFN3 reale su ticchettio: eseguire separatamente"]
    fn dfn3_ticchettio_attenuazione_effettiva() {
        let format = Format {
            rate: RATE,
            channels: 1,
        };
        let model_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(MODEL_FILE);
        let mut seed = 43_u32;
        let input: Vec<f32> = (0..RATE as usize * 6)
            .map(|i| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let phase = (i + RATE as usize / 2) % (RATE as usize / 3);
                let envelope = if phase < RATE as usize / 12 {
                    (-(phase as f32) / (RATE as f32 * 0.012)).exp()
                } else {
                    0.0
                };
                (seed as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32 * 0.35 * envelope
            })
            .collect();
        let mut app = DeepFilter::new(&model_path, format).unwrap();
        let mut actual = Vec::new();
        for block in input.chunks(960) {
            app.process(
                PcmBlock {
                    format,
                    start_frame: 0,
                    samples: block,
                },
                &mut actual,
            )
            .unwrap();
        }
        app.flush(Boundary::Finish, &mut actual).unwrap();
        let mut native = State::new(&model_path, format).unwrap();
        native.infer(&input, true).unwrap();
        let mut tail = Vec::new();
        native.from_48.finish(&mut tail);
        native.wet.extend(tail);
        let wet: Vec<f32> = native.wet.iter().copied().collect();
        assert_eq!(actual.len(), input.len());
        assert_eq!(wet.len(), input.len());
        let difference = actual
            .iter()
            .zip(&wet)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            difference <= 1e-6,
            "il processore pubblico deve restituire DFN3 diretto: scarto {difference}"
        );
        let energy = |samples: &[f32]| {
            samples[RATE as usize..RATE as usize * 5]
                .iter()
                .map(|&v| f64::from(v).powi(2))
                .sum::<f64>()
        };
        let original_db = 10.0 * (energy(&input) / energy(&wet)).log10();
        let app_db = 10.0 * (energy(&input) / energy(&actual)).log10();
        println!("TICCHETTIO DFN3={original_db:.3}dB integrazione={app_db:.3}dB");
        assert!(
            original_db >= 6.0,
            "il campione deve essere attenuabile da DFN3"
        );
        assert!(
            app_db >= 2.9,
            "pulizia attiva quasi inefficace: {app_db:.3}dB contro DFN3 {original_db:.3}dB"
        );
    }
    use crate::audio_toolkit::processing::{Boundary, Format, PcmStream};

    #[test]
    #[ignore = "DFN3 reale: misura riproducibile del rumore, eseguire separatamente"]
    fn dfn3_riduce_il_rumore_e_misura_l_onset_della_voce_debole() {
        use crate::audio_toolkit::decode::Decoder;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut decoder = Decoder::open(&root.join("tests/fixtures/parlato-it.wav")).unwrap();
        let mut voice = Vec::new();
        while let Some(block) = decoder.next_block().unwrap() {
            voice.extend(block.mono());
        }
        let mut seed = 42_u32;
        let noisy: Vec<_> = voice
            .iter()
            .map(|&x| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                x + (seed as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32 * 0.04
            })
            .collect();
        let format = Format {
            rate: 16_000,
            channels: 1,
        };
        let mut state = State::new(&root.join(MODEL_FILE), format).unwrap();
        let mut output = Vec::new();
        let mut snrs = Vec::new();
        for block in noisy.chunks(1024) {
            state
                .process(
                    PcmBlock {
                        format,
                        start_frame: 0,
                        samples: block,
                    },
                    &mut output,
                )
                .unwrap();
            snrs.push(state.channels[0].snr);
        }
        state.flush(Boundary::Finish, &mut output).unwrap();
        assert_eq!(output.len(), noisy.len());
        // Il riferimento pulito noto rende la misura sensibile anche alla distorsione della voce.
        let error = |samples: &[f32]| {
            samples
                .iter()
                .zip(&voice)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
        };
        let reduction = 10.0 * (error(&noisy) / error(&output)).log10();
        println!(
            "RUMORE seed=42 uniforme +/-0.04, errore rispetto al parlato noto: {reduction}dB; SNR modello min={} max={}",
            snrs.iter().copied().fold(f32::INFINITY, f32::min),
            snrs.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        );
        let weak: Vec<_> = voice.iter().map(|x| x * 0.031622776).collect();
        let mut weak_state = State::new(&root.join(MODEL_FILE), format).unwrap();
        let mut weak_output = Vec::new();
        for block in weak.chunks(1024) {
            weak_state
                .process(
                    PcmBlock {
                        format,
                        start_frame: 0,
                        samples: block,
                    },
                    &mut weak_output,
                )
                .unwrap();
        }
        weak_state
            .flush(Boundary::Finish, &mut weak_output)
            .unwrap();
        let onset = |samples: &[f32]| {
            use crate::audio_toolkit::resample::FrameResampler;
            use crate::audio_toolkit::vad::{Silero, VoiceDetector};
            let mut resampler = FrameResampler::new(16_000).unwrap();
            let mut frames = resampler.push(samples);
            frames.extend(resampler.finish());
            let mut silero = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
            let probabilities: Vec<_> = frames
                .iter()
                .map(|frame| silero.probability(frame).unwrap())
                .collect();
            probabilities
                .windows(2)
                .position(|p| {
                    p.iter()
                        .all(|&p| p >= crate::audio_toolkit::segmenter::Params::default().threshold)
                })
                .map(|frame| frame * 30)
        };
        let dry_onset = onset(&weak);
        let wet_onset = onset(&weak_output);
        println!(
            "VOCE -30dB onset Silero: originale={dry_onset:?}ms pulito={wet_onset:?}ms, frame {} -> {}",
            weak.len(),
            weak_output.len()
        );
        assert_eq!(weak_output.len(), weak.len());
        assert!(weak_output.iter().all(|x| x.is_finite()));
        // La scelta esplicita di DFN3 diretto sostituisce la conservazione dell'onset
        // tramite miscela. Si misura lo scostamento, senza promettere voce debole invariata.
        assert!(
            dry_onset.is_some(),
            "la fixture deve contenere parlato debole"
        );
        assert!(
            reduction >= 1.0,
            "riduzione misurabile e utile del rumore: {reduction}dB"
        );
    }

    #[test]
    #[ignore = "DFN3 e corpus breve reali; richiede MEMOTAPE_DFN3_SHORT, eseguire separatamente"]
    fn dfn3_corpus_breve_conserva_durata_e_round_trip() {
        use crate::audio_toolkit::decode::Decoder;
        let path = std::env::var("MEMOTAPE_DFN3_SHORT").unwrap();
        let mut decoder = Decoder::open(Path::new(&path)).unwrap();
        let mut input = Vec::new();
        while let Some(block) = decoder.next_block().unwrap() {
            assert_eq!(block.rate, 16_000);
            input.extend(block.mono());
        }
        let format = Format {
            rate: 16_000,
            channels: 1,
        };
        let mut state = State::new(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(MODEL_FILE),
            format,
        )
        .unwrap();
        let mut output = Vec::new();
        for block in input.chunks(512) {
            state
                .process(
                    PcmBlock {
                        format,
                        start_frame: 0,
                        samples: block,
                    },
                    &mut output,
                )
                .unwrap();
        }
        state.flush(Boundary::Finish, &mut output).unwrap();
        assert_eq!(output.len(), input.len());
        assert!(output.iter().all(|x| x.is_finite()));
        let mut up = Resampler::new(16_000, RATE, 1).unwrap();
        let mut at_48 = Vec::new();
        up.push(&input, &mut at_48);
        up.finish(&mut at_48);
        let mut down = Resampler::new(RATE, 16_000, 1).unwrap();
        let mut round_trip = Vec::new();
        down.push(&at_48, &mut round_trip);
        down.finish(&mut round_trip);
        assert_eq!(round_trip.len(), input.len());
        let span = 19_200..23_680; // No, 1200..1480 ms, indipendente dagli onset Silero.
        let error: f32 = span
            .clone()
            .map(|i| (input[i] - round_trip[i]).powi(2))
            .sum();
        let (lag, _) = (-200_isize..=200)
            .map(|lag| {
                let cross: f32 = span
                    .clone()
                    .map(|i| input[i] * round_trip[(i as isize + lag) as usize])
                    .sum();
                (lag, cross)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        println!(
            "DFN3 DIRETTO, {} frame. ROUND-TRIP 16/48/16 kHz sul No: lag={lag} campioni, errore quadratico={error}",
            input.len()
        );
        assert_eq!(
            lag, 0,
            "il round-trip non introduce uno spostamento temporale"
        );
        assert!(error > 0.0, "il round-trip non è un riferimento PCM neutro");
    }

    #[test]
    fn modello_assente_o_diverso_non_diventa_bypass() {
        let missing = std::env::temp_dir().join("memotape-dfn3-mancante.tar.gz");
        assert!(matches!(
            validate(&missing),
            Err(AppError::AudioCleaningMissing)
        ));
        let path = std::env::temp_dir().join("memotape-dfn3-errato.tar.gz");
        std::fs::write(&path, b"altro modello").unwrap();
        assert!(matches!(
            validate(&path),
            Err(AppError::AudioCleaningIncompatible(_))
        ));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    #[ignore = "DFN3 reale: eseguire separatamente, senza altri motori nativi"]
    fn dfn3_conserva_durata_silenzio_e_coda_a_stop() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources/models/DeepFilterNet3_onnx.tar.gz");
        let format = Format {
            rate: 48_000,
            channels: 2,
        };
        let mut stream =
            PcmStream::new(format, Box::new(DeepFilter::new(&path, format).unwrap())).unwrap();
        let mut input = vec![0.0; 4800 * 2];
        for (i, frame) in input.chunks_exact_mut(2).enumerate().skip(3600) {
            frame[0] = 0.1 * (i as f32 * 0.05).sin();
            frame[1] = 0.00001 * (i as f32 * 0.08).sin();
        }
        let mut output = Vec::new();
        for block in input.chunks(137 * 2) {
            output.extend(stream.push(block).unwrap().samples);
        }
        output.extend(stream.boundary(Boundary::Finish).unwrap().samples);
        assert_eq!(output.len(), input.len());
        assert!(output.iter().all(|x| x.is_finite()));
        assert!(output[..3000 * 2].iter().all(|x| x.abs() < 1e-8));
        assert!(
            output[4400 * 2..]
                .iter()
                .step_by(2)
                .any(|x| x.abs() > 0.001),
            "Stop conserva la voce della coda"
        );
        let mut native = State::new(&path, format).unwrap();
        native.infer(&input, true).unwrap();
        let mut tail = Vec::new();
        native.from_48.finish(&mut tail);
        native.wet.extend(tail);
        let expected: Vec<_> = native.wet.into_iter().collect();
        // Il segnale debolissimo può essere soppresso dal modello stesso. L'adattatore
        // deve conservarne esattamente l'uscita, non reinserire l'originale per farlo udire.
        assert_eq!(
            output, expected,
            "anche i canali deboli e la coda usano DFN3 diretto"
        );
    }

    #[test]
    #[ignore = "confronto DFN3 reale con API originale libDF; eseguire separatamente"]
    fn dfn3_percorso_spettrale_equivale_al_runtime_originale_su_audio_non_silenzioso() {
        use ndarray::Array2;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MODEL_FILE);
        let mut original = DfTract::new(
            DfParams::new(path.clone()).unwrap(),
            &RuntimeParams::default().with_atten_lim(100.0),
        )
        .unwrap();
        let mut adapted = Hop::new(
            DfTract::new(
                DfParams::new(path.clone()).unwrap(),
                &RuntimeParams::default().with_atten_lim(100.0),
            )
            .unwrap(),
        );
        let input: Vec<f32> = (0..480 * 30)
            .map(|i| 0.1 * (i as f32 * 0.071).sin())
            .collect();
        let mut expected = Vec::new();
        let mut output_hop = Array2::<f32>::zeros((1, 480));
        for hop in input.chunks_exact(480) {
            let hop = Array2::from_shape_vec((1, 480), hop.to_vec()).unwrap();
            original.process(hop.view(), output_hop.view_mut()).unwrap();
            expected.extend(output_hop.iter().copied());
        }
        let mut result = Vec::new();
        for hop in input.chunks_exact(HOP) {
            let mut output = vec![0.0; HOP];
            adapted.process(hop, &mut output).unwrap();
            result.extend(output);
        }
        let error = result
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(error < 1e-6, "differenza dal runtime originale: {error}");
    }

    #[test]
    #[ignore = "DFN3 reale a frequenze e canali diversi; eseguire separatamente"]
    fn dfn3_ricampionamento_pause_e_sessioni_conservano_formato_e_durata() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MODEL_FILE);
        for (rate, channels) in [(8000, 1), (16000, 2), (24000, 1), (44100, 2), (48000, 6)] {
            let format = Format { rate, channels };
            let input: Vec<f32> = (0..(rate / 10 + 13) as usize)
                .flat_map(|i| {
                    (0..channels).map(move |ch| {
                        if ch == channels - 1 {
                            0.0
                        } else {
                            0.05 * (i as f32 * 0.05 + ch as f32).sin()
                        }
                    })
                })
                .collect();
            let mut stream =
                PcmStream::new(format, Box::new(DeepFilter::new(&path, format).unwrap())).unwrap();
            for boundary in [Boundary::Pause, Boundary::Configuration, Boundary::Finish] {
                let mut result = Vec::new();
                for block in input.chunks(channels * 137) {
                    result.extend(stream.push(block).unwrap().samples);
                }
                result.extend(stream.boundary(boundary).unwrap().samples);
                assert_eq!(result.len(), input.len(), "{rate} Hz, {channels} canali");
                assert!(result.iter().all(|x| x.is_finite()));
                assert!(
                    result
                        .chunks_exact(channels)
                        .all(|f| f[channels - 1].abs() < 1e-8)
                );
            }
        }
    }
}
