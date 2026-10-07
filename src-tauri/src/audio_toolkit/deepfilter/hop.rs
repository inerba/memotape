//! Adattamento del percorso spettrale di libDF 0.5.6 (Hendrik Schröter, MIT).
//! `process_raw`, STFT, maschere, pesi e Tract restano quelli originali. Diversamente dalla
//! scorciatoia di `DfTract::process`, ogni hop avanza anche sul silenzio: nessun segnale pilota,
//! amplificazione o campione artificiale, e la coda si scarica con padding zero reale.

use std::collections::VecDeque;

use df::Complex32;
use df::tract::{DfTract, as_slice_complex, as_slice_mut_real};

use super::{HOP, failed};
use crate::error::AppError;

pub(super) struct Hop {
    model: DfTract,
    noisy: VecDeque<Vec<Complex32>>,
    enhanced: VecDeque<Vec<Complex32>>,
    pub(super) snr: f32,
}

impl Hop {
    pub(super) fn new(model: DfTract) -> Self {
        Self {
            snr: -15.0,
            noisy: vec![
                vec![Complex32::default(); model.n_freqs];
                model.df_order.max(model.lookahead)
            ]
            .into(),
            enhanced: vec![
                vec![Complex32::default(); model.n_freqs];
                model.df_order + model.conv_lookahead
            ]
            .into(),
            model,
        }
    }

    pub(super) fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), AppError> {
        debug_assert_eq!(input.len(), HOP);
        let mut spectrum = vec![Complex32::default(); self.model.n_freqs];
        self.model.df_states[0].analysis(input, &mut spectrum);
        self.model
            .spec_buf
            .to_array_view_mut::<f32>()
            .map_err(failed)?
            .as_slice_mut()
            .ok_or_else(|| failed("buffer Tract non contiguo"))?
            .copy_from_slice(as_slice_mut_real(&mut spectrum));
        self.noisy.pop_front();
        self.noisy.push_back(spectrum.clone());
        self.enhanced.pop_front();
        self.enhanced.push_back(spectrum);

        let (snr, gains, coefs) = self.model.process_raw().map_err(failed)?;
        if !snr.is_finite() {
            return Err(failed("stima SNR DFN3 non finita"));
        }
        self.snr = snr;
        let masked = &mut self.enhanced[self.model.df_order - 1];
        if let Some(gains) = gains {
            let gains = gains.to_array_view::<f32>().map_err(failed)?;
            self.model.df_states[0].apply_mask(
                masked,
                gains
                    .as_slice()
                    .ok_or_else(|| failed("maschera Tract non contigua"))?,
            );
        }
        let mut result = masked.clone();
        if let Some(coefs) = coefs {
            let coefs = coefs.to_array_view::<f32>().map_err(failed)?;
            let coefs = as_slice_complex(
                coefs
                    .as_slice()
                    .ok_or_else(|| failed("coefficienti Tract non contigui"))?,
            );
            for (frequency, value) in result.iter_mut().enumerate().take(self.model.nb_df) {
                *value = Complex32::default();
                for (time, spectrum) in self.noisy.iter().enumerate().take(self.model.df_order) {
                    *value += spectrum[frequency] * coefs[frequency * self.model.df_order + time];
                }
            }
        }
        if let Some(limit) = self.model.atten_lim {
            let dry = &self.noisy[self.noisy.len() - self.model.lookahead - 1];
            for (wet, &dry) in result.iter_mut().zip(dry) {
                *wet = *wet * (1.0 - limit) + dry * limit;
            }
        }
        self.model.df_states[0].synthesis(&mut result, output);
        if !output.iter().all(|value| value.is_finite()) {
            return Err(failed("output DFN3 non finito"));
        }
        Ok(())
    }
}
