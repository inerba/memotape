//! VAD: il trait `VoiceDetector` e l'implementazione Silero v4 (vad-rs su ONNX Runtime).

use std::path::Path;

use super::resample::TARGET_RATE;
use crate::error::AppError;

/// Probabilità di parlato di un frame da 480 campioni (30 ms a 16 kHz).
pub trait VoiceDetector {
    fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError>;
    fn reset(&mut self);
}

pub struct Silero(vad_rs::Vad);

impl Silero {
    pub fn new(model: &Path) -> Result<Self, AppError> {
        vad_rs::Vad::new(model, TARGET_RATE)
            .map(Self)
            .map_err(|e| AppError::Internal(format!("Silero ({}): {e}", model.display())))
    }
}

impl VoiceDetector for Silero {
    fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError> {
        self.0
            .compute(frame)
            .map(|r| r.prob)
            .map_err(|e| AppError::Internal(format!("Silero: {e}")))
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}
