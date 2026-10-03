//! Motore di Trascrizione (`TranscriptionEngine`) e pipeline di un file, senza Tauri.

pub mod diarize;
pub mod live;
pub mod pipeline;
pub mod transcribe_cpp;

use crate::error::AppError;

/// Trascrive una Frase per chiamata.
pub trait TranscriptionEngine {
    /// `frames`: l'audio della Frase, frame f32 mono a 16 kHz in [-1, 1], letti man mano che
    /// la pipeline li produce. `language`: la Lingua del parlato con un codice dell'app (`it`…),
    /// `None` per il riconoscimento automatico. `on_partial` riceve il Parziale ogni volta che
    /// cambia; i motori che trascrivono la Frase intera non lo chiamano mai. Senza `on_partial`
    /// (un file, che non mostra Parziali) anche un motore in streaming trascrive la Frase intera.
    /// Restituisce il testo della Frase.
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
        language: Option<&str>,
        on_partial: Option<&mut dyn FnMut(&str)>,
    ) -> Result<String, EngineError>;
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EngineError {
    /// Il modello sta già calcolando per un'altra sessione: non è un guasto, il motore resta sano e
    /// la Trascrizione si può riprovare. Con una sola Attività alla volta non capita (ADR-0003).
    #[error("modello occupato")]
    Busy,
    #[error("annullato")]
    Cancelled,
    #[error("{0}")]
    Internal(String),
}

impl From<EngineError> for AppError {
    fn from(error: EngineError) -> Self {
        match error {
            EngineError::Busy => Self::ModelInUse(error.to_string()),
            EngineError::Cancelled => Self::Cancelled,
            EngineError::Internal(e) => Self::Internal(e),
        }
    }
}

/// Il codice che il modello accetta per la Lingua del parlato `language` (`it`): lo stesso codice
/// o un locale che inizia così (`it-IT` per Nemotron). `None` se il modello non la offre.
pub fn resolve_language(language: &str, model_languages: &[String]) -> Option<String> {
    model_languages
        .iter()
        .find(|m| {
            m.eq_ignore_ascii_case(language)
                || m.split_once('-')
                    .is_some_and(|(code, _)| code.eq_ignore_ascii_case(language))
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::resolve_language;

    fn languages(codes: &[&str]) -> Vec<String> {
        codes.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn la_lingua_del_parlato_diventa_il_codice_offerto_dal_modello() {
        assert_eq!(
            resolve_language("it", &languages(&["en", "it", "de"])),
            Some("it".into())
        );
        // Nemotron vuole il locale.
        assert_eq!(
            resolve_language("it", &languages(&["en-US", "it-IT"])),
            Some("it-IT".into())
        );
        // Un prefisso che non è il codice non vale.
        assert_eq!(resolve_language("it", &languages(&["ita", "en"])), None);
        assert_eq!(
            resolve_language("pl", &languages(&["en-US", "it-IT"])),
            None
        );
        assert_eq!(resolve_language("it", &[]), None);
    }
}
