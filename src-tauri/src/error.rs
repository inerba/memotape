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
    /// Riconosci i parlanti è attiva ma il modello di diarizzazione, di cui porta il nome, non è
    /// scaricato.
    #[error("modello di diarizzazione assente: {0}")]
    DiarizerMissing(String),
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
    /// Il modello scelto, di cui porta il nome, non è scaricato o non si carica: la Registrazione
    /// continua senza Trascrizione dal vivo.
    #[error("Trascrizione dal vivo non disponibile: {0}")]
    LiveTranscriptionUnavailable(String),
    /// Il Tape è stato scritto da una versione più nuova dell'app, con uno schema che non conosce.
    #[error("Tape di una versione più nuova")]
    UnsupportedBino,
    /// Il nome di una Raccolta o il titolo di un Tape non è ammesso da Windows.
    #[error("nome non valido: {0}")]
    InvalidName(String),
    /// C'è già una Raccolta, o un Tape nella stessa cartella, con questo nome.
    #[error("nome già usato: {0}")]
    NameTaken(String),
    /// Si elimina solo una Raccolta vuota.
    #[error("Raccolta non vuota: {0}")]
    RaccoltaNotEmpty(String),
    /// Il Tape non è più dov'era: spostato, rinominato o cancellato fuori dall'app.
    #[error("Tape non trovato: {0}")]
    BinoNotFound(String),
    #[error("Attività in corso")]
    ActivityInProgress,
    // L'utente ha premuto Annulla (Trascrizione o download): non è un guasto.
    #[error("annullato")]
    Cancelled,
    #[error("errore interno: {0}")]
    Internal(String),
}
