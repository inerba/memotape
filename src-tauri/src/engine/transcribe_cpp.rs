//! `TranscriptionEngine` su transcribe-cpp, in modalità `run` (una Frase intera per chiamata).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use transcribe_cpp::{Model, RunOptions, Session};

use super::TranscriptionEngine;
use crate::error::AppError;

pub struct TranscribeCpp {
    session: Session,
}

impl TranscribeCpp {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        let session = native(|| Model::load(path)?.session())?;
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
        let transcript = native(|| session.run(&pcm, &RunOptions::default()))?;
        Ok(transcript.text.trim().to_string())
    }
}

/// Chiamata nativa protetta: un panic non deve abbattere il thread della pipeline senza errore.
fn native<T>(call: impl FnOnce() -> transcribe_cpp::Result<T>) -> Result<T, AppError> {
    catch_unwind(AssertUnwindSafe(call))
        .map_err(|_| AppError::Internal("panic in transcribe-cpp".into()))?
        .map_err(|e| AppError::Internal(format!("transcribe-cpp: {e}")))
}
