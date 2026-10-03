//! `TranscriptionEngine` su transcribe-cpp: `Session::stream` per i modelli in streaming
//! (Nemotron), con un Parziale a ogni cambio del testo, e `Session::run` sulla Frase intera per
//! gli altri (Whisper, Parakeet).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use transcribe_cpp::{CancelToken, Feature, Model, RunOptions, Session, StreamOptions};

use super::{EngineError, TranscriptionEngine};
use crate::error::AppError;

/// Un modello caricato, riusabile tra una Trascrizione e l'altra.
pub struct TranscribeCpp {
    session: Session,
    /// Le lingue di `capabilities().languages` (codici o locale, es. `it-IT`).
    languages: Vec<String>,
    /// `capabilities().supports_streaming`: fra i modelli del catalogo, solo Nemotron.
    streaming: bool,
}

impl TranscribeCpp {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        let (session, capabilities) = catch_native(|| {
            let model = Model::load(path)?;
            if !model.supports(Feature::Cancellation) {
                log::info!(
                    "il modello non supporta la cancellazione: Annulla aspetta la fine della Frase"
                );
            }
            Ok((model.session()?, model.capabilities()))
        })?;
        Ok(Self {
            session,
            languages: capabilities.languages,
            streaming: capabilities.supports_streaming,
        })
    }

    /// Le lingue che il modello accetta come indicazione, lette dal modello.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Installa il token di Annulla della prossima Trascrizione: interrompe anche la Frase in
    /// corso se il modello supporta la cancellazione, altrimenti la pipeline si ferma alla fine
    /// della Frase.
    pub fn set_cancel_token(&mut self, cancel: &CancelToken) {
        self.session.set_cancel_token(cancel);
    }
}

impl TranscriptionEngine for TranscribeCpp {
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
        language: Option<&str>,
        on_partial: &mut dyn FnMut(&str),
    ) -> Result<String, EngineError> {
        let run = RunOptions {
            language: language.and_then(|l| super::resolve_language(l, &self.languages)),
            ..RunOptions::default()
        };
        if !self.streaming {
            let pcm: Vec<f32> = frames.flatten().collect();
            let session = &mut self.session;
            let transcript = catch_native(|| session.run(&pcm, &run))?;
            return Ok(transcript.text.trim().to_string());
        }
        // Uno stream per Frase. Le opzioni di default usano l'attenzione a destra predefinita del
        // modello, la più accurata: per Nemotron R=13, che dà lo stesso testo di `run`. Il lease
        // del modello si libera a `finalize`, o al drop dello stream se si esce prima.
        let session = &mut self.session;
        let mut stream = catch_native(|| session.stream(&run, &StreamOptions::default()))?;
        let mut shown = String::new();
        for frame in frames {
            let update = catch_native(|| stream.feed(&frame))?;
            if update.committed_changed || update.tentative_changed {
                let partial = stream.text().display().trim().to_string();
                if partial != shown {
                    on_partial(&partial);
                    shown = partial;
                }
            }
        }
        catch_native(|| stream.finalize())?;
        Ok(stream.text().full.trim().to_string())
    }
}

/// Chiamata nativa protetta: un panic non deve abbattere il thread della pipeline senza errore.
fn catch_native<T>(call: impl FnOnce() -> transcribe_cpp::Result<T>) -> Result<T, EngineError> {
    catch_unwind(AssertUnwindSafe(call))
        .map_err(|_| EngineError::Internal("panic in transcribe-cpp".into()))?
        .map_err(|e| match e {
            transcribe_cpp::Error::Aborted { .. } => EngineError::Cancelled,
            transcribe_cpp::Error::Busy(_) => EngineError::Busy,
            e => EngineError::Internal(format!("transcribe-cpp: {e}")),
        })
}
