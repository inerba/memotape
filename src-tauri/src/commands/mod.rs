//! Comandi Tauri: validano gli argomenti e delegano ai manager.

use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::error::AppError;
use crate::managers;

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
    if !accepted || !path.is_file() {
        return Err(AppError::UnreadableFile(source));
    }
    app.opener()
        .open_path(source, None::<&str>)
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Trascrive la Sorgente: progresso e Frasi arrivano come eventi, poi il testo si salva nel TXT.
#[tauri::command]
#[specta::specta]
pub async fn transcribe(
    app: AppHandle,
    source: String,
) -> Result<managers::transcription::TranscriptionFinished, AppError> {
    managers::transcription::transcribe(app, PathBuf::from(source)).await
}
