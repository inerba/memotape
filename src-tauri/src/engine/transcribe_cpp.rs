//! `TranscriptionEngine` su transcribe-cpp, in modalità `run` (una Frase intera per chiamata).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use transcribe_cpp::{CancelToken, Feature, Model, RunOptions, Session};

use super::TranscriptionEngine;
use crate::error::AppError;

/// Un modello caricato, riusabile tra una Trascrizione e l'altra.
pub struct TranscribeCpp {
    session: Session,
    /// Le lingue di `capabilities().languages` (codici o locale, es. `it-IT`).
    languages: Vec<String>,
    run: RunOptions,
}

impl TranscribeCpp {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        let (session, languages) = catch_native(|| {
            let model = Model::load(path)?;
            if !model.supports(Feature::Cancellation) {
                log::info!(
                    "il modello non supporta la cancellazione: Annulla aspetta la fine della Frase"
                );
            }
            Ok((model.session()?, model.capabilities().languages))
        })?;
        Ok(Self {
            session,
            languages,
            run: RunOptions::default(),
        })
    }

    /// Le lingue che il modello accetta come indicazione, lette dal modello.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Prepara la prossima Trascrizione. `cancel` interrompe anche la Frase in corso se il modello
    /// supporta la cancellazione, altrimenti la pipeline si ferma alla fine della Frase.
    /// `language` è un codice tra quelli dell'app (`it`…), `None` per il riconoscimento automatico.
    pub fn prepare(&mut self, cancel: &CancelToken, language: Option<&str>) {
        self.session.set_cancel_token(cancel);
        self.run.language = language.and_then(|l| super::resolve_language(l, &self.languages));
    }
}

impl TranscriptionEngine for TranscribeCpp {
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
    ) -> Result<String, AppError> {
        let pcm: Vec<f32> = frames.flatten().collect();
        let (session, run) = (&mut self.session, &self.run);
        let transcript = catch_native(|| session.run(&pcm, run))?;
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
