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
    /// Il modello si sta caricando o lo usa una Trascrizione: non si elimina.
    #[error("modello in uso: {0}")]
    ModelInUse(String),
    #[error("download fallito: {0}")]
    DownloadFailed(String),
    /// Dimensione o SHA-256 del modello scaricato non corrispondono al catalogo.
    #[error("verifica fallita: {0}")]
    VerificationFailed(String),
    /// Il dispositivo si è scollegato durante la Registrazione, che si è salvata: porta il nome.
    #[error("dispositivo scollegato: {0}")]
    DeviceDisconnected(String),
    /// Nessun microfono, o quello scelto in Impostazioni non è collegato.
    #[error("microfono assente")]
    MicrophoneMissing,
    /// Nessun dispositivo di uscita per l'audio di sistema, o quello scelto non è collegato.
    #[error("dispositivo di uscita assente")]
    OutputDeviceMissing,
    /// `settings.json` esiste ma non si legge: valgono i predefiniti.
    #[error("impostazioni illeggibili: {0}")]
    UnreadableSettings(String),
    #[error("Attività in corso")]
    ActivityInProgress,
    // L'utente ha premuto Annulla (Trascrizione o download): non è un guasto.
    #[error("annullato")]
    Cancelled,
    #[error("errore interno: {0}")]
    Internal(String),
}
