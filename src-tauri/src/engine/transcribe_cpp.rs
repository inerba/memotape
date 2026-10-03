//! `TranscriptionEngine` su transcribe-cpp, in modalità `run` (una Frase intera per chiamata).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use transcribe_cpp::{CancelToken, Feature, Model, RunOptions, Session};

use super::TranscriptionEngine;
use crate::error::AppError;

pub struct TranscribeCpp {
    session: Session,
}

impl TranscribeCpp {
    /// Carica il modello e ci installa `cancel`: se il modello supporta la cancellazione, Annulla
    /// interrompe anche la Frase in corso; altrimenti la pipeline si ferma alla fine della Frase.
    pub fn load(path: &Path, cancel: &CancelToken) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        let mut session = catch_native(|| {
            let model = Model::load(path)?;
            if !model.supports(Feature::Cancellation) {
                log::info!(
                    "il modello non supporta la cancellazione: Annulla aspetta la fine della Frase"
                );
            }
            model.session()
        })?;
        session.set_cancel_token(cancel);
        Ok(Self { session })
    }
}

impl TranscriptionEngine for TranscribeCpp {
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
    ) -> Result<String, AppError> {
        let pcm: Vec<f32> = frames.flatten().collect();
        let session = &mut self.session;
        let transcript = catch_native(|| session.run(&pcm, &RunOptions::default()))?;
        Ok(transcript.text.trim().to_string())
    }
}

/// Chiamata nativa protetta: un panic non deve abbattere il thread della pipeline senza errore.
fn catch_native<T>(call: impl FnOnce() -> transcribe_cpp::Result<T>) -> Result<T, AppError> {
    catch_unwind(AssertUnwindSafe(call))
        .map_err(|_| AppError::Internal("panic in transcribe-cpp".into()))?
        .map_err(|e| match e {
            transcribe_cpp::Error::Aborted { .. } => AppError::Cancelled,
            e => AppError::Internal(format!("transcribe-cpp: {e}")),
        })
}
