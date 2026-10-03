//! Motore di Trascrizione (`TranscriptionEngine`) e pipeline di un file, senza Tauri.

pub mod pipeline;
pub mod transcribe_cpp;

use crate::error::AppError;

/// Trascrive una Frase per chiamata.
pub trait TranscriptionEngine {
    /// `frames`: l'audio della Frase, frame f32 mono a 16 kHz in [-1, 1], letti man mano che
    /// la pipeline li produce. Restituisce il testo della Frase.
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
    ) -> Result<String, AppError>;
}
