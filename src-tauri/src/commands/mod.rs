//! Comandi Tauri: validano gli argomenti e delegano ai manager.

use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::audio_toolkit::capture::{self, AudioDevice};
use crate::error::AppError;
use crate::managers;
use crate::managers::activity::Activity;
use crate::managers::models::{ModelInfo, Models};
use crate::managers::recording::{Recorder, RecordingSaved};
use crate::managers::settings::{Language, Settings, SettingsStore};

/// Estensioni accettate da Sfoglia (spec, storia 2).
const SOURCE_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "m4a", "flac", "ogg", "opus", "webm", "mpga", "mpeg", "aiff", "mp4", "mkv",
    "mov", "m4v",
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
    .inspect_err(|e| log::error!("dialog di Sfoglia: {e}"))
    .ok()??;
    picked.into_path().ok().map(|p| p.display().to_string())
}

/// Apre la Sorgente con il programma associato. Accetta solo le estensioni di Sfoglia, così
/// non diventa un modo per lanciare eseguibili.
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
    app.opener()
        .open_path(source, None::<&str>)
        .map_err(|e| AppError::Internal(e.to_string()))
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
    managers::transcription::transcribe(app, &activity, PathBuf::from(source)).await
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

/// Valida e salva le impostazioni. Se cambia il modello scelto, lo carica in background.
#[tauri::command]
#[specta::specta]
pub fn set_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    settings: Settings,
) -> Result<(), AppError> {
    let model = settings.model.clone();
    if store.set(settings)?.model != model {
        managers::transcription::preload(&app);
    }
    Ok(())
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
/// arriva `stop_recording` o un dispositivo si scollega; poi il file diventa la Sorgente. Durata e
/// livelli arrivano con `recording-tick`. `prefix` è il prefisso tradotto del nome del file. Rifiuta
/// con `activityInProgress` se un'Attività è già in corso.
#[tauri::command]
#[specta::specta]
pub async fn record(
    app: AppHandle,
    activity: State<'_, Activity>,
    recorder: State<'_, Recorder>,
    prefix: String,
) -> Result<RecordingSaved, AppError> {
    let reserved = |c: char| c.is_control() || r#"<>:"/\|?*"#.contains(c);
    if prefix.trim().is_empty() || prefix.contains(reserved) {
        return Err(AppError::Internal(format!("prefisso non valido: {prefix}")));
    }
    managers::recording::record(app, &activity, &recorder, prefix).await
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

/// La Cartella predefinita in uso: quella delle impostazioni o `Documenti\Sbobino`.
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
