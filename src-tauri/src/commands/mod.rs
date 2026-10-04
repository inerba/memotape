//! Comandi Tauri: validano gli argomenti e delegano ai manager.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::audio_toolkit::capture::{self, AudioDevice};
use crate::bino;
use crate::error::AppError;
use crate::library::LibraryList;
use crate::managers;
use crate::managers::activity::Activity;
use crate::managers::models::{ModelInfo, Models};
use crate::managers::pending_bino::PendingBino;
use crate::managers::recording::{Recorder, RecordingSaved};
use crate::managers::settings::{Language, Settings, SettingsStore};
use crate::transcript::Ingresso;

/// Estensioni accettate da Apri file (spec, storia 2), Bino compresi.
const SOURCE_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "m4a", "flac", "ogg", "opus", "webm", "mpga", "mpeg", "aiff", "mp4", "mkv",
    "mov", "m4v", "bino",
];

/// Versione dell'app, dal `Cargo.toml`.
#[tauri::command]
#[specta::specta]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Apre il dialog di sistema sui file accettati. `filter_name` è l'etichetta tradotta del filtro.
/// Restituisce il percorso scelto, o `null` se l'utente annulla.
#[tauri::command]
#[specta::specta]
pub async fn pick_source(app: AppHandle, filter_name: String) -> Option<String> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter(filter_name, SOURCE_EXTENSIONS)
            .blocking_pick_file()
    })
    .await
    .inspect_err(|e| log::error!("dialog di Apri file: {e}"))
    .ok()??;
    picked.into_path().ok().map(|p| p.display().to_string())
}

/// Apre la Sorgente con il programma associato; un Bino lo mostra nella cartella. Accetta solo le
/// estensioni di Apri file, così non diventa un modo per lanciare eseguibili.
#[tauri::command]
#[specta::specta]
pub fn open_source(app: AppHandle, source: String) -> Result<(), AppError> {
    let path = PathBuf::from(&source);
    let accepted = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SOURCE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
    if !accepted {
        return Err(AppError::Internal(format!(
            "estensione non accettata: {source}"
        )));
    }
    if !path.is_file() {
        return Err(AppError::UnreadableFile(source));
    }
    let opened = if bino::is_bino(&path) {
        app.opener().reveal_item_in_dir(path)
    } else {
        app.opener().open_path(source, None::<&str>)
    };
    opened.map_err(|e| AppError::Internal(e.to_string()))
}

/// Apre un Bino scelto come Sorgente: restituisce le sue Frasi, che diventano l'ultima
/// Trascrizione, e i nomi dei Parlanti. `unsupportedBino` se viene da una versione più nuova
/// dell'app.
#[tauri::command]
#[specta::specta]
pub fn open_bino(
    app: AppHandle,
    source: String,
) -> Result<managers::transcription::OpenedBino, AppError> {
    let path = Path::new(&source);
    if !bino::is_bino(path) {
        return Err(AppError::Internal(format!("non è un Bino: {source}")));
    }
    // Un Bino sparito o cambiato in Esplora file si vede anche nella barra laterale.
    managers::library::sync(&app);
    managers::transcription::open_bino(&app, path)
}

/// Il Bino arrivato con un avvio (doppio clic in Esplora file) e non ancora aperto, se c'è; dopo
/// la chiamata non c'è più. `bino-requested` avvisa quando ne arriva uno con l'app già aperta.
#[tauri::command]
#[specta::specta]
pub fn take_pending_bino(pending: State<'_, PendingBino>) -> Option<String> {
    pending.take()
}

/// Trascrive la Sorgente: progresso e Frasi arrivano come eventi, poi il testo si salva nel
/// Markdown.
/// Rifiuta con `activityInProgress` se un'Attività è già in corso, e finisce con `cancelled` dopo
/// `cancel_transcription`.
#[tauri::command]
#[specta::specta]
pub async fn transcribe(
    app: AppHandle,
    activity: State<'_, Activity>,
    source: String,
) -> Result<managers::transcription::TranscriptionOutcome, AppError> {
    let outcome =
        managers::transcription::transcribe(app.clone(), &activity, PathBuf::from(source)).await;
    // Su un Bino la Trascrizione l'ha riscritto.
    managers::library::sync(&app);
    outcome
}

/// Annulla la Trascrizione in corso. Restituisce `false` se non è (ancora) partita.
#[tauri::command]
#[specta::specta]
pub fn cancel_transcription(activity: State<'_, Activity>) -> bool {
    activity.cancel()
}

/// Il testo di Copia testo: l'ultima Trascrizione in testo semplice o Markdown, secondo
/// `copiaCome`. `null` se non c'è ancora stata una Trascrizione.
#[tauri::command]
#[specta::specta]
pub fn transcript_text(
    last: State<'_, managers::transcription::LastTranscript>,
    settings: State<'_, SettingsStore>,
) -> Option<String> {
    managers::transcription::transcript_text(&last, &settings.get())
}

/// Rinomina il Parlante `parlante` di `ingresso` nell'ultima Trascrizione, nel Bino che la contiene
/// e nel Markdown che ha prodotto. Rifiuta un nome vuoto, e con `activityInProgress` durante
/// un'Attività.
#[tauri::command]
#[specta::specta]
pub fn rename_parlante(
    app: AppHandle,
    activity: State<'_, Activity>,
    last: State<'_, managers::transcription::LastTranscript>,
    settings: State<'_, SettingsStore>,
    ingresso: Ingresso,
    parlante: u32,
    nome: String,
) -> Result<(), AppError> {
    let _activity = activity.begin(None, || {})?;
    let renamed = managers::transcription::rename_parlante(
        &last,
        ingresso,
        parlante,
        &nome,
        &managers::transcription::labels(&settings.get()),
    );
    managers::library::sync(&app);
    renamed
}

/// I modelli del catalogo con il loro stato.
#[tauri::command]
#[specta::specta]
pub fn list_models(models: State<'_, Models>) -> Vec<ModelInfo> {
    models.list()
}

/// Avvia il download di un modello in background, o lo riprende da un parziale. Avanzamento ed
/// esito arrivano con `model-download-progress` e `model-state-changed`.
#[tauri::command]
#[specta::specta]
pub fn download_model(
    app: AppHandle,
    models: State<'_, Models>,
    id: String,
) -> Result<(), AppError> {
    models.start_download(&app, &id)
}

/// Annulla il download e cancella il parziale. Restituisce `false` se non c'era un download.
#[tauri::command]
#[specta::specta]
pub fn cancel_model_download(models: State<'_, Models>, id: String) -> bool {
    models.cancel(&id)
}

/// Le impostazioni lette all'avvio. `unreadableSettings` se il file non si è letto: allora il
/// backend usa i predefiniti.
#[tauri::command]
#[specta::specta]
pub fn get_settings(settings: State<'_, SettingsStore>) -> Result<Settings, AppError> {
    settings.loaded()
}

/// La lingua di Windows se è tra le sei, altrimenti l'inglese: la Lingua dell'interfaccia quando
/// le impostazioni non ne scelgono una.
#[tauri::command]
#[specta::specta]
pub fn system_language() -> Language {
    Language::system()
}

/// Valida e salva le impostazioni e restituisce quelle salvate: se all'avvio il file non si è letto,
/// sono le sue con sopra le modifiche. Se cambia il modello scelto, lo carica in background.
#[tauri::command]
#[specta::specta]
pub fn set_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    settings: Settings,
) -> Result<Settings, AppError> {
    let previous = store.set(settings)?;
    let saved = store.get();
    if previous.model != saved.model {
        managers::transcription::preload(&app);
    }
    if previous.recordings_folder != saved.recordings_folder {
        managers::library::sync_in_background(&app);
    }
    Ok(saved)
}

/// Elimina il modello scaricato. Rifiuta con `modelInUse` se si sta caricando o lo usa una
/// Trascrizione.
#[tauri::command]
#[specta::specta]
pub fn delete_model(app: AppHandle, models: State<'_, Models>, id: String) -> Result<(), AppError> {
    models.delete(&app, &id)
}

/// I microfoni rilevati, per la scelta in Impostazioni.
#[tauri::command]
#[specta::specta]
pub async fn list_microphones() -> Result<Vec<AudioDevice>, AppError> {
    // WASAPI usa COM: fuori dal thread principale, che ha già il suo apartment.
    tauri::async_runtime::spawn_blocking(capture::microphones)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// I dispositivi di uscita rilevati, per la scelta dell'audio di sistema in Impostazioni.
#[tauri::command]
#[specta::specta]
pub async fn list_output_devices() -> Result<Vec<AudioDevice>, AppError> {
    tauri::async_runtime::spawn_blocking(capture::output_devices)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Registra dagli ingressi delle impostazioni (microfono, audio di sistema o entrambi) finché
/// arriva `stop_recording` o un dispositivo si scollega; poi il Bino, nella Raccolta `raccolta`
/// (`null` o `""`: la radice della Libreria), diventa la Sorgente. Durata e livelli arrivano con
/// `recording-tick`. `prefix` è il prefisso tradotto del nome del file. Rifiuta con
/// `activityInProgress` se un'Attività è già in corso.
#[tauri::command]
#[specta::specta]
pub async fn record(
    app: AppHandle,
    activity: State<'_, Activity>,
    recorder: State<'_, Recorder>,
    prefix: String,
    raccolta: Option<String>,
) -> Result<RecordingSaved, AppError> {
    let reserved = |c: char| c.is_control() || r#"<>:"/\|?*"#.contains(c);
    if prefix.trim().is_empty() || prefix.contains(reserved) {
        return Err(AppError::Internal(format!("prefisso non valido: {prefix}")));
    }
    let saved =
        managers::recording::record(app.clone(), &activity, &recorder, prefix, raccolta).await;
    managers::library::sync(&app);
    saved
}

/// Mette in pausa (`true`) o riprende la Registrazione. Restituisce `false` se non è in corso.
#[tauri::command]
#[specta::specta]
pub fn pause_recording(recorder: State<'_, Recorder>, paused: bool) -> bool {
    recorder.set_paused(paused)
}

/// Ferma e salva la Registrazione: l'esito arriva come risultato di `record`. Restituisce
/// `false` se non è in corso.
#[tauri::command]
#[specta::specta]
pub fn stop_recording(recorder: State<'_, Recorder>) -> bool {
    recorder.stop()
}

/// La Cartella della Libreria in uso: quella delle impostazioni o `Documenti\Sbobino`.
#[tauri::command]
#[specta::specta]
pub fn recordings_folder(app: AppHandle) -> Result<String, AppError> {
    managers::recording::recordings_folder(&app).map(|p| p.display().to_string())
}

/// Apre il dialog di sistema per scegliere una cartella. `null` se l'utente annulla.
#[tauri::command]
#[specta::specta]
pub async fn pick_folder(app: AppHandle) -> Option<String> {
    let picked =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
            .await
            .inspect_err(|e| log::error!("dialog della cartella: {e}"))
            .ok()??;
    picked.into_path().ok().map(|p| p.display().to_string())
}

/// Esegue `f` fuori dal thread principale: legge e scrive sul disco.
async fn blocking<T: Send + 'static>(
    app: AppHandle,
    f: impl FnOnce(&AppHandle) -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(move || f(&app))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Le Raccolte e i Bini della Libreria, dall'indice: `library-changed` avvisa quando cambiano.
#[tauri::command]
#[specta::specta]
pub async fn library_list(app: AppHandle) -> Result<LibraryList, AppError> {
    blocking(app, |app| {
        managers::library::with(app, |library| library.list())
    })
    .await
}

/// Crea la Raccolta `nome`. `invalidName` per un nome che Windows non ammette, `nameTaken` se c'è
/// già.
#[tauri::command]
#[specta::specta]
pub async fn create_raccolta(app: AppHandle, nome: String) -> Result<(), AppError> {
    blocking(app, move |app| {
        managers::library::change(app, |library| library.create_raccolta(&nome))
    })
    .await
}

/// Rinomina la Raccolta `nome` e la sua cartella in `nuovo`, e restituisce la cartella nuova. Rifiuta
/// con `activityInProgress` se l'Attività in corso lavora su un suo Bino.
#[tauri::command]
#[specta::specta]
pub async fn rename_raccolta(
    app: AppHandle,
    activity: State<'_, Activity>,
    nome: String,
    nuovo: String,
) -> Result<String, AppError> {
    let _writing = activity.write(&raccolta_path(&app, &nome)?)?;
    blocking(app, move |app| {
        let (from, to) =
            managers::library::change(app, |library| library.rename_raccolta(&nome, &nuovo))?;
        Ok(moved(app, &from, &to))
    })
    .await
}

/// Elimina la Raccolta `nome`, solo se vuota (`raccoltaNotEmpty`).
#[tauri::command]
#[specta::specta]
pub async fn delete_raccolta(
    app: AppHandle,
    activity: State<'_, Activity>,
    nome: String,
) -> Result<(), AppError> {
    let _writing = activity.write(&raccolta_path(&app, &nome)?)?;
    blocking(app, move |app| {
        managers::library::change(app, |library| library.delete_raccolta(&nome))
    })
    .await
}

/// Rinomina il file del Bino in `<titolo>.bino` e restituisce il percorso nuovo. Rifiuta con
/// `activityInProgress` il Bino su cui lavora l'Attività in corso.
#[tauri::command]
#[specta::specta]
pub async fn rename_bino(
    app: AppHandle,
    activity: State<'_, Activity>,
    path: String,
    titolo: String,
) -> Result<String, AppError> {
    let from = PathBuf::from(path);
    let _writing = activity.write(&from)?;
    blocking(app, move |app| {
        let to = managers::library::change(app, |library| library.rename_bino(&from, &titolo))?;
        Ok(moved(app, &from, &to))
    })
    .await
}

/// Sposta il Bino nella Raccolta `raccolta` (`null` o `""`: la radice), anche da fuori della
/// Libreria (Aggiungi alla Libreria…), e restituisce il percorso nuovo.
#[tauri::command]
#[specta::specta]
pub async fn move_bino(
    app: AppHandle,
    activity: State<'_, Activity>,
    path: String,
    raccolta: Option<String>,
) -> Result<String, AppError> {
    let from = PathBuf::from(path);
    let _writing = activity.write(&from)?;
    blocking(app, move |app| {
        let to = managers::library::change(app, |library| {
            library.move_bino(&from, raccolta.as_deref())
        })?;
        Ok(moved(app, &from, &to))
    })
    .await
}

/// Manda il Bino nel Cestino di Windows.
#[tauri::command]
#[specta::specta]
pub async fn trash_bino(
    app: AppHandle,
    activity: State<'_, Activity>,
    path: String,
) -> Result<(), AppError> {
    let path = PathBuf::from(path);
    let _writing = activity.write(&path)?;
    blocking(app, move |app| {
        managers::library::change(app, |library| library.trash_bino(&path))
    })
    .await
}

/// Il Bino o la Raccolta `from` ora è `to`: lo segue anche l'ultima Trascrizione. Restituisce `to`.
fn moved(app: &AppHandle, from: &Path, to: &Path) -> String {
    app.state::<managers::transcription::LastTranscript>()
        .moved(from, to);
    to.display().to_string()
}

fn raccolta_path(app: &AppHandle, nome: &str) -> Result<PathBuf, AppError> {
    crate::library::validate_name(nome)?;
    Ok(managers::recording::recordings_folder(app)?.join(nome))
}
