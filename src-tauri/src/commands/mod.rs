//! Comandi Tauri: validano gli argomenti e delegano ai manager.

/// Versione dell'app, dal `Cargo.toml`.
#[tauri::command]
#[specta::specta]
pub fn app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}
