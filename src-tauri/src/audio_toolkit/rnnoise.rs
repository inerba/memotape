//! RNNoise 0.2 di xiph (`vendor/rnnoise`), modello `0b50c45`, con i pesi tra le risorse.

use std::collections::VecDeque;
use std::ffi::{c_int, c_void};
use std::path::Path;
use std::ptr::NonNull;

use sha2::{Digest, Sha256};

use super::processing::{AudioProcessor, Boundary, Format, PcmBlock};
use super::resample::Resampler;
use crate::error::AppError;

/// Prodotto da `write_weights` di RNNoise dal tarball del modello: la procedura per rigenerarlo
/// è in `docs/sviluppo/trascrizione.md`.
pub const WEIGHTS_FILE: &str = "resources/models/rnnoise-0b50c45.bin";
const WEIGHTS_SIZE: usize = 1_401_600;
const WEIGHTS_SHA256: &str = "47edcad7baeffb6442d9bfe8ea3b3ae728b50055c3abca1d69b4e31a806da27d";
pub const ALGORITMO: &str = "rnnoise";
/// Tag, commit, modello e uscita diretta del modello, senza miscela con l'originale.
pub const VERSIONE: &str = "v0.2/904a876dce1f9ab8860c0a5000ed151f9f6eef58/0b50c45/direct-v1";
const RATE: u32 = 48_000;
const FRAME: usize = 480;
/// Un frame di sovrapposizione della finestra più uno della 0.2, che sintetizza lo spettro del
/// frame precedente (`delayed_X`): la 0.1 e OBS compensano un frame solo.
const DELAY: usize = 2 * FRAME;
/// RNNoise lavora sulla scala dei campioni int16.
const SCALE: f32 = 32_768.0;

#[repr(C)]
struct DenoiseState {
    _private: [u8; 0],
}

#[repr(C)]
struct RNNModel {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn rnnoise_model_from_buffer(ptr: *const c_void, len: c_int) -> *mut RNNModel;
    fn rnnoise_model_free(model: *mut RNNModel);
    fn rnnoise_create(model: *mut RNNModel) -> *mut DenoiseState;
    fn rnnoise_destroy(st: *mut DenoiseState);
    fn rnnoise_process_frame(st: *mut DenoiseState, out: *mut f32, input: *const f32) -> f32;
}

fn failed(error: impl std::fmt::Display) -> AppError {
    AppError::AudioCleaningFailed(error.to_string())
}

/// Legge i pesi e ne verifica dimensione e SHA-256 prima di consegnarli al codice C.
fn read_weights(path: &Path) -> Result<Vec<u8>, AppError> {
    let bytes = std::fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::AudioCleaningMissing
        } else {
            AppError::AudioCleaningIncompatible(error.to_string())
        }
    })?;
    if bytes.len() != WEIGHTS_SIZE || format!("{:x}", Sha256::digest(&bytes)) != WEIGHTS_SHA256 {
        return Err(AppError::AudioCleaningIncompatible(
            "dimensione o SHA-256 non corrispondono ai pesi di RNNoise 0.2".into(),
        ));
    }
    Ok(bytes)
}

/// I pesi in memoria: gli stati creati dal modello puntano dentro `blob`, che deve sopravvivere
/// a tutti. Si caricano dai byte già verificati e non con `rnnoise_model_from_filename`, che apre
/// il percorso con `fopen` e non regge i caratteri non ASCII di una cartella utente.
struct Model {
    raw: NonNull<RNNModel>,
    _blob: Vec<u8>,
}

impl Model {
    fn load(path: &Path) -> Result<Self, AppError> {
        let blob = read_weights(path)?;
        // SAFETY: `blob` resta in `Model` e non cambia finché il modello esiste.
        let raw = unsafe { rnnoise_model_from_buffer(blob.as_ptr().cast(), blob.len() as c_int) };
        let raw = NonNull::new(raw).ok_or_else(|| failed("modello RNNoise non allocato"))?;
        Ok(Self { raw, _blob: blob })
    }

    fn state(&self) -> Result<State, AppError> {
        // SAFETY: il modello è valido; lo stato legge i pesi in `blob`, che vive più di lui
        // perché `Session` si distrugge prima di `Model` (ordine dei campi di `RNNoise`).
        let raw = unsafe { rnnoise_create(self.raw.as_ptr()) };
        NonNull::new(raw).map(State).ok_or_else(|| {
            AppError::AudioCleaningIncompatible("pesi non compatibili con RNNoise 0.2".into())
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        // SAFETY: creato da `rnnoise_model_from_buffer`, che non possiede `blob`.
        unsafe { rnnoise_model_free(self.raw.as_ptr()) };
    }
}

struct State(NonNull<DenoiseState>);

impl State {
    fn process(&mut self, out: &mut [f32; FRAME], input: &[f32; FRAME]) {
        // SAFETY: frame da `rnnoise_get_frame_size()` = 480 campioni, buffer distinti.
        unsafe { rnnoise_process_frame(self.0.as_ptr(), out.as_mut_ptr(), input.as_ptr()) };
    }
}

impl Drop for State {
    fn drop(&mut self) {
        // SAFETY: creato da `rnnoise_create` e distrutto una volta sola.
        unsafe { rnnoise_destroy(self.0.as_ptr()) };
    }
}

// SAFETY: `DenoiseState` e `RNNModel` sono memoria del processo senza stato legato al thread
// (niente thread-local, niente globali mutabili: la scelta SIMD è un `cpuid` puro) e l'adattatore
// li usa solo da `&mut self`, quindi spostarli su un altro thread è sicuro.
unsafe impl Send for Model {}
unsafe impl Send for State {}

/// Un tratto continuo nel formato della Sorgente: stato e ricampionamento ripartono a ogni
/// confine, dopo lo scarico della coda.
struct Session {
    channels: Vec<State>,
    to_48: Resampler,
    from_48: Resampler,
    pending: Vec<f32>,
    input_48_frames: usize,
    processed_48_frames: usize,
    input_frames: usize,
    wet: VecDeque<f32>,
    emitted: usize,
}

impl Session {
    fn new(model: &Model, format: Format) -> Result<Self, AppError> {
        Ok(Self {
            channels: (0..format.channels)
                .map(|_| model.state())
                .collect::<Result<_, _>>()?,
            to_48: Resampler::new(format.rate, RATE, format.channels)?,
            from_48: Resampler::new(RATE, format.rate, format.channels)?,
            pending: Vec::new(),
            input_48_frames: 0,
            processed_48_frames: 0,
            input_frames: 0,
            wet: VecDeque::new(),
            emitted: 0,
        })
    }

    /// Elabora i frame interi a 48 kHz; alla fine aggiunge silenzio fino a far uscire il ritardo.
    /// Del risultato tiene solo la parte allineata all'ingresso, senza il ritardo iniziale.
    fn infer(&mut self, samples: &[f32], finish: bool) {
        let count = self.channels.len();
        self.input_48_frames += samples.len() / count;
        self.pending.extend_from_slice(samples);
        let size = FRAME * count;
        if finish {
            self.pending.resize(
                (self.pending.len() / count + DELAY).div_ceil(FRAME) * size,
                0.0,
            );
        }
        let (mut input, mut output) = ([0.0; FRAME], [0.0; FRAME]);
        let mut interleaved = vec![0.0; size];
        let mut consumed = 0;
        while self.pending.len() - consumed >= size {
            let frame = &self.pending[consumed..consumed + size];
            for (channel, state) in self.channels.iter_mut().enumerate() {
                for (sample, value) in input
                    .iter_mut()
                    .zip(frame.iter().skip(channel).step_by(count))
                {
                    *sample = value * SCALE;
                }
                state.process(&mut output, &input);
                for (value, sample) in interleaved
                    .iter_mut()
                    .skip(channel)
                    .step_by(count)
                    .zip(output)
                {
                    *value = sample / SCALE;
                }
            }
            let start = self.processed_48_frames;
            let useful_start = DELAY.saturating_sub(start).min(FRAME);
            let useful_end = (self.input_48_frames + DELAY)
                .saturating_sub(start)
                .min(FRAME);
            if useful_end > useful_start {
                let mut returned = Vec::new();
                self.from_48.push(
                    &interleaved[useful_start * count..useful_end * count],
                    &mut returned,
                );
                self.wet.extend(returned);
            }
            self.processed_48_frames += FRAME;
            consumed += size;
        }
        self.pending.drain(..consumed);
    }

    fn emit(&mut self, finish: bool, out: &mut Vec<f32>) -> Result<(), AppError> {
        let count = self.channels.len();
        // Il doppio arrotondamento del ricampionamento può produrre un frame oltre la durata
        // originale: si consegna soltanto l'intervallo ricevuto.
        let frames = (self.wet.len() / count).min(self.input_frames - self.emitted);
        for value in self.wet.drain(..frames * count) {
            if !value.is_finite() {
                return Err(failed("campioni non finiti"));
            }
            out.push(value);
        }
        self.emitted += frames;
        if finish && self.emitted != self.input_frames {
            return Err(failed("coda PCM incompleta"));
        }
        Ok(())
    }

    fn process(&mut self, samples: &[f32], out: &mut Vec<f32>) -> Result<(), AppError> {
        self.input_frames += samples.len() / self.channels.len();
        let mut at_48 = Vec::new();
        self.to_48.push(samples, &mut at_48);
        self.infer(&at_48, false);
        self.emit(false, out)
    }

    fn flush(&mut self, out: &mut Vec<f32>) -> Result<(), AppError> {
        let mut at_48 = Vec::new();
        self.to_48.finish(&mut at_48);
        self.infer(&at_48, true);
        let mut returned = Vec::new();
        self.from_48.finish(&mut returned);
        self.wet.extend(returned);
        self.emit(true, out)
    }
}

/// Ricampiona a 48 kHz e torna al formato della Sorgente, uno stato per canale, uscita diretta
/// del modello. La probabilità di voce di RNNoise non si usa: il VAD resta Silero.
pub struct RNNoise {
    format: Format,
    /// Prima di `model`: gli stati leggono i suoi pesi.
    session: Session,
    model: Model,
    finished: bool,
}

impl RNNoise {
    pub fn new(path: &Path, format: Format) -> Result<Self, AppError> {
        let model = Model::load(path)?;
        Ok(Self {
            format,
            session: Session::new(&model, format)?,
            model,
            finished: false,
        })
    }
}

impl AudioProcessor for RNNoise {
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
        self.session.process(block.samples, out)
    }

    fn flush(&mut self, boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
        self.session.flush(out)?;
        self.finished = boundary == Boundary::Finish;
        if !self.finished {
            self.session = Session::new(&self.model, self.format)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
