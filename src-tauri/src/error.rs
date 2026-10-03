//! L'unico enum degli errori applicativi. Il frontend mappa `code` a un messaggio tradotto.

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
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
    #[error("Attività in corso")]
    ActivityInProgress,
    // L'utente ha premuto Annulla: non è un guasto, la status bar lo dice senza allarme.
    #[error("Trascrizione annullata")]
    Cancelled,
    #[error("errore interno: {0}")]
    Internal(String),
}
