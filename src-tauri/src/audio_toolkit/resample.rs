//! Normalizzazione: audio mono a qualsiasi frequenza → frame da 30 ms a 16 kHz.

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};

use crate::error::AppError;

pub const TARGET_RATE: usize = 16_000;
/// 30 ms a 16 kHz, la dimensione che chiede Silero.
pub const FRAME_SAMPLES: usize = 480;
const CHUNK: usize = 1024;

pub struct FrameResampler {
    /// `None` se l'ingresso è già a 16 kHz.
    resampler: Option<Fft<f32>>,
    in_rate: usize,
    input: Vec<f32>,
    output: Vec<f32>,
    chunk_out: Vec<f32>,
    /// Campioni iniziali di ritardo del resampler ancora da scartare.
    delay_left: usize,
    in_total: usize,
    out_total: usize,
}

impl FrameResampler {
    pub fn new(in_rate: u32) -> Result<Self, AppError> {
        let in_rate = in_rate as usize;
        let resampler = if in_rate == TARGET_RATE {
            None
        } else {
            Some(
                Fft::<f32>::new(in_rate, TARGET_RATE, CHUNK, 1, FixedSync::Input)
                    .map_err(|e| AppError::Internal(format!("resampler: {e}")))?,
            )
        };
        let (chunk_out, delay_left) = resampler.as_ref().map_or((Vec::new(), 0), |r| {
            (vec![0.0; r.output_frames_max()], r.output_delay())
        });
        Ok(Self {
            resampler,
            in_rate,
            input: Vec::new(),
            output: Vec::new(),
            chunk_out,
            delay_left,
            in_total: 0,
            out_total: 0,
        })
    }

    /// Accoda campioni mono e restituisce i frame completi.
    pub fn push(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        self.in_total += samples.len();
        match self.resampler {
            None => self.output.extend_from_slice(samples),
            Some(_) => {
                self.input.extend_from_slice(samples);
                let mut consumed = 0;
                while self.input.len() - consumed >= self.input_needed() {
                    let n = self.input_needed();
                    consumed += self.process(consumed, n, None);
                }
                self.input.drain(..consumed);
            }
        }
        self.take_frames(false)
    }

    /// Svuota il resampler; l'ultimo frame è completato con zeri.
    pub fn finish(&mut self) -> Vec<Vec<f32>> {
        if self.resampler.is_some() {
            let expected = (self.in_total * TARGET_RATE).div_ceil(self.in_rate);
            let rest = self.input.len();
            self.process(0, rest, Some(rest));
            self.input.clear();
            // Il ritardo trattiene campioni: si spinge silenzio finché escono tutti.
            while self.out_total < expected {
                self.process(0, 0, Some(0));
            }
            let excess = self.out_total - expected;
            self.output
                .truncate(self.output.len().saturating_sub(excess));
        } else {
            self.out_total = self.in_total;
        }
        self.take_frames(true)
    }

    fn input_needed(&self) -> usize {
        self.resampler
            .as_ref()
            .map_or(0, Resampler::input_frames_next)
    }

    /// Ricampiona `len` campioni di `input` da `offset`; restituisce i campioni consumati.
    fn process(&mut self, offset: usize, len: usize, partial: Option<usize>) -> usize {
        let Some(resampler) = self.resampler.as_mut() else {
            return 0;
        };
        let needed = resampler.input_frames_next();
        // Con `partial_len` l'adapter deve comunque coprire un chunk intero.
        let mut padded;
        let chunk: &[f32] = if partial.is_some() {
            padded = vec![0.0; needed];
            padded[..len].copy_from_slice(&self.input[offset..offset + len]);
            &padded
        } else {
            &self.input[offset..offset + len]
        };
        let out_len = self.chunk_out.len();
        let input = InterleavedSlice::new(chunk, 1, chunk.len()).expect("buffer mono valido");
        let mut output =
            InterleavedSlice::new_mut(&mut self.chunk_out, 1, out_len).expect("buffer mono valido");
        let indexing = partial.map(|p| Indexing::new().partial_len(p));
        let (_, produced) = resampler
            .process_into_buffer(&input, &mut output, indexing.as_ref())
            .expect("dimensioni dei buffer coerenti con il resampler");
        let skip = self.delay_left.min(produced);
        self.delay_left -= skip;
        self.output
            .extend_from_slice(&self.chunk_out[skip..produced]);
        self.out_total += produced - skip;
        len
    }

    fn take_frames(&mut self, pad_last: bool) -> Vec<Vec<f32>> {
        let mut frames: Vec<Vec<f32>> = self
            .output
            .chunks_exact(FRAME_SAMPLES)
            .map(<[f32]>::to_vec)
            .collect();
        let rest = self.output.len() % FRAME_SAMPLES;
        let tail = self.output.split_off(self.output.len() - rest);
        self.output.clear();
        if pad_last && !tail.is_empty() {
            let mut last = tail;
            last.resize(FRAME_SAMPLES, 0.0);
            frames.push(last);
        } else {
            self.output = tail;
        }
        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resample(rate: u32, seconds: f32, push_size: usize) -> Vec<f32> {
        let n = (rate as f32 * seconds) as usize;
        let sine: Vec<f32> = (0..n)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 0.5)
            .collect();
        let mut r = FrameResampler::new(rate).unwrap();
        let mut frames = Vec::new();
        for chunk in sine.chunks(push_size) {
            frames.extend(r.push(chunk));
        }
        frames.extend(r.finish());
        assert!(frames.iter().all(|f| f.len() == FRAME_SAMPLES));
        frames.concat()
    }

    #[test]
    fn un_secondo_a_qualsiasi_frequenza_diventa_16000_campioni_in_frame_da_480() {
        for rate in [8_000, 16_000, 22_050, 44_100, 48_000] {
            let out = resample(rate, 1.0, 777);
            // 16 000 campioni, completati a 34 frame.
            assert_eq!(out.len(), 34 * FRAME_SAMPLES, "{rate} Hz");
            assert!(out[16_000..].iter().all(|&s| s == 0.0), "{rate} Hz");
        }
    }

    #[test]
    fn il_ritardo_del_resampler_non_diventa_silenzio_iniziale() {
        let out = resample(44_100, 1.0, 4096);
        // Una sinusoide da 440 Hz a 0,5 di ampiezza: nei primi 5 ms c'è già segnale pieno.
        let peak = out[..80].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.4, "picco iniziale {peak}");
    }
}
