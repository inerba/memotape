//! Ricampionamento con rubato: `Resampler` da una frequenza all'altra (mono o stereo
//! interleaved) e, per la Trascrizione, `FrameResampler` che produce frame da 30 ms a 16 kHz.

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler as _};

use crate::error::AppError;

pub const TARGET_RATE: usize = 16_000;
/// 30 ms a 16 kHz, la dimensione che chiede Silero.
pub const FRAME_SAMPLES: usize = 480;
const CHUNK: usize = 1024;

/// Ricampiona audio interleaved a `channels` canali da `in_rate` a `out_rate`, senza il ritardo
/// iniziale del filtro e con la lunghezza esatta in uscita dopo `finish`.
pub struct Resampler {
    /// `None` se le frequenze coincidono: i campioni passano invariati.
    fft: Option<Fft<f32>>,
    in_rate: usize,
    out_rate: usize,
    channels: usize,
    /// Campioni interleaved in attesa di un chunk intero.
    input: Vec<f32>,
    chunk_out: Vec<f32>,
    /// Frame iniziali di ritardo del filtro ancora da scartare.
    delay_left: usize,
    in_frames: usize,
    out_frames: usize,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32, channels: usize) -> Result<Self, AppError> {
        let (in_rate, out_rate) = (in_rate as usize, out_rate as usize);
        let fft = if in_rate == out_rate {
            None
        } else {
            Some(
                Fft::<f32>::new(in_rate, out_rate, CHUNK, channels, FixedSync::Input)
                    .map_err(|e| AppError::Internal(format!("resampler: {e}")))?,
            )
        };
        let (chunk_out, delay_left) = fft.as_ref().map_or((Vec::new(), 0), |r| {
            (
                vec![0.0; r.output_frames_max() * channels],
                r.output_delay(),
            )
        });
        Ok(Self {
            fft,
            in_rate,
            out_rate,
            channels,
            input: Vec::new(),
            chunk_out,
            delay_left,
            in_frames: 0,
            out_frames: 0,
        })
    }

    /// Ricampiona `samples` (interleaved) e accoda in `out` quanto è pronto.
    pub fn push(&mut self, samples: &[f32], out: &mut Vec<f32>) {
        self.in_frames += samples.len() / self.channels;
        if self.fft.is_none() {
            out.extend_from_slice(samples);
            return;
        }
        self.input.extend_from_slice(samples);
        let mut consumed = 0;
        loop {
            let needed = self.frames_needed() * self.channels;
            if self.input.len() - consumed < needed {
                break;
            }
            self.process(consumed, needed, None, out);
            consumed += needed;
        }
        self.input.drain(..consumed);
    }

    /// Svuota il filtro: in `out` arrivano gli ultimi campioni, fino alla durata esatta
    /// dell'ingresso.
    pub fn finish(&mut self, out: &mut Vec<f32>) {
        if self.fft.is_none() {
            return;
        }
        let expected = (self.in_frames * self.out_rate).div_ceil(self.in_rate);
        let start = out.len();
        let rest = self.input.len();
        self.process(0, rest, Some(rest / self.channels), out);
        self.input.clear();
        // Il ritardo trattiene campioni: si spinge silenzio finché escono tutti.
        while self.out_frames < expected {
            self.process(0, 0, Some(0), out);
        }
        let excess = (self.out_frames - expected) * self.channels;
        out.truncate(out.len().saturating_sub(excess).max(start));
        self.out_frames = expected;
    }

    fn frames_needed(&self) -> usize {
        self.fft.as_ref().map_or(0, |r| r.input_frames_next())
    }

    /// Ricampiona `len` campioni di `input` da `offset` e li accoda in `out`.
    fn process(&mut self, offset: usize, len: usize, partial: Option<usize>, out: &mut Vec<f32>) {
        let Some(fft) = self.fft.as_mut() else {
            return;
        };
        let channels = self.channels;
        let needed = fft.input_frames_next() * channels;
        // Con `partial_len` l'adapter deve comunque coprire un chunk intero.
        let mut padded;
        let chunk: &[f32] = if partial.is_some() {
            padded = vec![0.0; needed];
            padded[..len].copy_from_slice(&self.input[offset..offset + len]);
            &padded
        } else {
            &self.input[offset..offset + len]
        };
        let out_frames = self.chunk_out.len() / channels;
        let input = InterleavedSlice::new(chunk, channels, chunk.len() / channels)
            .expect("buffer interleaved valido");
        let mut output = InterleavedSlice::new_mut(&mut self.chunk_out, channels, out_frames)
            .expect("buffer interleaved valido");
        let indexing = partial.map(|p| Indexing::new().partial_len(p));
        let (_, produced) = fft
            .process_into_buffer(&input, &mut output, indexing.as_ref())
            .expect("dimensioni dei buffer coerenti con il resampler");
        let skip = self.delay_left.min(produced);
        self.delay_left -= skip;
        out.extend_from_slice(&self.chunk_out[skip * channels..produced * channels]);
        self.out_frames += produced - skip;
    }
}

/// Audio mono a qualsiasi frequenza → frame da 30 ms a 16 kHz.
pub struct FrameResampler {
    resampler: Resampler,
    output: Vec<f32>,
}

impl FrameResampler {
    pub fn new(in_rate: u32) -> Result<Self, AppError> {
        Ok(Self {
            resampler: Resampler::new(in_rate, TARGET_RATE as u32, 1)?,
            output: Vec::new(),
        })
    }

    /// Accoda campioni mono e restituisce i frame completi.
    pub fn push(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        self.resampler.push(samples, &mut self.output);
        self.take_frames(false)
    }

    /// Svuota il resampler; l'ultimo frame è completato con zeri.
    pub fn finish(&mut self) -> Vec<Vec<f32>> {
        self.resampler.finish(&mut self.output);
        self.take_frames(true)
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
