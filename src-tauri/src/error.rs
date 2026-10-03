//! L'unico enum degli errori applicativi. Il frontend mappa `code` a un messaggio tradotto.

#[derive(Debug, Clone, PartialEq, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "code", content = "detail", rename_all = "camelCase")]
pub enum AppError {
    #[error("file illeggibile: {0}")]
    UnreadableFile(String),
    #[error("codec non supportato: {0}")]
    UnsupportedCodec(String),
    #[error("cartella non scrivibile: {0}")]
    UnwritableFolder(String),
    #[error("modello assente: {0}")]
    ModelMissing(String),
    #[error("download fallito: {0}")]
    DownloadFailed(String),
    /// Dimensione o SHA-256 del modello scaricato non corrispondono al catalogo.
    #[error("verifica fallita: {0}")]
    VerificationFailed(String),
    #[error("Attività in corso")]
    ActivityInProgress,
    // L'utente ha premuto Annulla (Trascrizione o download): non è un guasto.
    #[error("annullato")]
    Cancelled,
    #[error("errore interno: {0}")]
    Internal(String),
}
