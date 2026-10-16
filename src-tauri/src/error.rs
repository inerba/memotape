//! L'unico enum degli errori applicativi. Il frontend mappa `code` a un messaggio tradotto.

#[derive(Debug, Clone, PartialEq, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "code", content = "detail", rename_all = "camelCase")]
pub enum AppError {
    #[error("modello DeepFilterNet3 distribuito con l'app assente")]
    AudioCleaningMissing,
    #[error("modello DeepFilterNet3 incompatibile: {0}")]
    AudioCleaningIncompatible(String),
    #[error("pulizia audio interrotta: {0}")]
    AudioCleaningFailed(String),
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
    /// La coda ASR ha raggiunto il limite: l'audio continua a essere salvato.
    #[error("la Trascrizione dal vivo non tiene il passo")]
    LiveTranscriptionLagging,
    /// Il Tape è stato scritto da una versione più nuova dell'app, con uno schema che non conosce.
    #[error("Tape di una versione più nuova")]
    UnsupportedTape,
    /// Il nome di una Raccolta o il titolo di un Tape non è ammesso da Windows.
    #[error("nome non valido: {0}")]
    InvalidName(String),
    /// Un Termine del Vocabolario contiene `<|` o `|>`, che nel prompt di Whisper farebbero fallire
    /// la Trascrizione: porta il Termine.
    #[error("Termine non valido: {0}")]
    InvalidTermine(String),
    /// C'è già una Raccolta, o un Tape nella stessa cartella, con questo nome.
    #[error("nome già usato: {0}")]
    NameTaken(String),
    /// Si elimina solo una Raccolta vuota.
    #[error("Raccolta non vuota: {0}")]
    RaccoltaNotEmpty(String),
    /// Il Tape non è più dov'era: spostato, rinominato o cancellato fuori dall'app.
    #[error("Tape non trovato: {0}")]
    TapeNotFound(String),
    /// Il volume del Tape non ha un Cestino: l'originale non si potrebbe recuperare.
    #[error("il volume non ha un Cestino: {0}")]
    NoRecycleBin(String),
    /// La Nota, di cui porta il nome, non ha più il testo da cui partiva la modifica: l'ha cambiata
    /// qualcun altro (un Assistente, un'altra finestra).
    #[error("Nota cambiata da fuori: {0}")]
    NoteChanged(String),
    /// Il Tape non ha una Nota con questo nome: l'ha eliminata o rinominata qualcun altro.
    #[error("Nota non trovata: {0}")]
    NoteNotFound(String),
    /// Il Tape ha già una Nota con questo nome.
    #[error("nome di Nota già usato: {0}")]
    NoteNameTaken(String),
    /// Il nome di una Nota è vuoto o supera `tape::NOME_NOTA_MAX` caratteri.
    #[error("nome di Nota non valido: {0}")]
    NoteNameInvalid(String),
    /// Il link di una Nota non è un indirizzo web o email, l'unico che si apre: porta il link.
    #[error("link non apribile: {0}")]
    LinkNotAllowed(String),
    #[error("Attività in corso")]
    ActivityInProgress,
    // L'utente ha premuto Annulla (Trascrizione o download): non è un guasto.
    #[error("annullato")]
    Cancelled,
    #[error("errore interno: {0}")]
    Internal(String),
}
