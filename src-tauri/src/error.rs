//! L'unico enum degli errori applicativi. Il frontend mappa `code` a un messaggio tradotto.

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "code", content = "detail", rename_all = "camelCase")]
pub enum AppError {
    #[error("file illeggibile: {0}")]
    UnreadableFile(String),
    #[error("codec non supportato: {0}")]
    UnsupportedCodec(String),
    #[error("modello assente: {0}")]
    ModelMissing(String),
    #[error("errore interno: {0}")]
    Internal(String),
}
