//! Motore di Trascrizione (`TranscriptionEngine`) e pipeline di un file, senza Tauri.

pub mod asr;
pub mod diarize;
pub mod live;
pub mod local_diarizer;
pub mod pipeline;
pub mod transcribe_cpp;
pub mod vocabolario;

use crate::error::AppError;

/// Trascrive una Frase per chiamata.
pub trait TranscriptionEngine {
    /// `frames`: l'audio della Frase, frame f32 mono a 16 kHz in [-1, 1], letti man mano che
    /// la pipeline li produce. `language`: la Lingua del parlato con un codice dell'app (`it`…),
    /// `None` per il riconoscimento automatico. `on_partial` riceve testo e tempi relativi del Parziale ogni volta che
    /// cambia il risultato; i motori che trascrivono la Frase intera non lo chiamano mai. Senza `on_partial`
    /// (un file, che non mostra Parziali) anche un motore in streaming trascrive la Frase intera.
    /// Restituisce testo e tempi ASR relativi alla Frase, se disponibili.
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
        language: Option<&str>,
        on_partial: Option<&mut dyn FnMut(&asr::AsrResult)>,
    ) -> Result<asr::AsrResult, EngineError>;
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

/// Se `text` ha lettere di una scrittura che la Lingua del parlato `language` (`it`) non usa:
/// l'alfabeto latino vale sempre (nomi e termini stranieri), le altre scritture solo se sono quelle
/// della lingua (per le ICU di Windows: `ja` → kana e han, `ru` → cirillico). Sull'audio
/// incomprensibile i modelli inventano frasi in altre lingue (cirillico in una Trascrizione in
/// italiano): Parakeet non riceve la lingua, Whisper la riceve ma non ne è vincolato.
pub fn foreign_script(text: &str, language: &str) -> bool {
    use windows_sys::Win32::Globalization::{
        USCRIPT_COMMON, USCRIPT_INHERITED, USCRIPT_LATIN, USCRIPT_UNKNOWN, uloc_addLikelySubtags,
        uscript_getCode, uscript_getScript,
    };
    let Ok(code) = std::ffi::CString::new(language) else {
        return false;
    };
    // `ja` → `ja_Jpan_JP`: con il separatore `uscript_getCode` lo legge come locale e non come
    // nome di scrittura (`yi`, lo yiddish, è anche la scrittura Yi).
    let mut likely = [0u8; 64];
    let mut status = 0;
    // SAFETY: stringa terminata da zero e un buffer della capacità dichiarata.
    unsafe {
        uloc_addLikelySubtags(
            code.as_ptr().cast(),
            likely.as_mut_ptr(),
            likely.len() as i32 - 1,
            &mut status,
        );
    }
    let mut scripts = [0; 8];
    // SAFETY: `likely` è terminato da zero (l'ultimo byte non si scrive mai).
    let n = unsafe {
        uscript_getCode(
            likely.as_ptr(),
            scripts.as_mut_ptr(),
            scripts.len() as i32,
            &mut status,
        )
    };
    if status > 0 || n <= 0 {
        // Una lingua di cui le ICU non conoscono la scrittura: niente da confrontare.
        return false;
    }
    let scripts = &scripts[..n as usize];
    text.chars().filter(|c| c.is_alphabetic()).any(|c| {
        let mut status = 0;
        // SAFETY: nessun puntatore oltre a `status`.
        let script = unsafe { uscript_getScript(c as i32, &mut status) };
        ![
            USCRIPT_COMMON,
            USCRIPT_INHERITED,
            USCRIPT_UNKNOWN,
            USCRIPT_LATIN,
        ]
        .contains(&script)
            && !scripts.contains(&script)
    })
}

#[cfg(test)]
mod tests {
    use super::{foreign_script, resolve_language};

    #[test]
    fn una_frase_in_un_altra_scrittura_e_estranea_alla_lingua_del_parlato() {
        // Le Frasi inventate da Parakeet e Whisper su audio incomprensibile in italiano.
        assert!(foreign_script("Остріє на ве.", "it"));
        assert!(foreign_script("Cottелю e vertaggio.", "it"));
        assert!(foreign_script("Meine prima oggesministra 이 Channel", "it"));
        assert!(foreign_script("lui中a", "it"));
        // Accenti, apostrofi, cifre e punteggiatura non contano.
        assert!(!foreign_script("Perché l'ho già detto: 3,5 €… «sì»!", "it"));
        assert!(!foreign_script("Żółć, gęślą jaźń", "pl"));
        // Il latino vale sempre; le altre scritture solo quelle della lingua.
        assert!(!foreign_script("Привет, OK", "ru"));
        assert!(!foreign_script("今日はAIの話です、カタカナも", "ja"));
        assert!(foreign_script("今日は Привет", "ja"));
        assert!(!foreign_script("שלום", "yi"));
        assert!(!foreign_script("粤语", "yue"));
        // Senza parole non c'è niente da scartare.
        assert!(!foreign_script("", "it"));
    }

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
