//! La Libreria della Cartella della Libreria corrente, in `tauri::State`: la riapre quando la
//! cartella cambia ed emette `library-changed` dopo ogni allineamento e operazione.

use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::error::AppError;
use crate::library::{self, Library};
use crate::managers::recording::recordings_folder;

/// In `tauri::State`.
#[derive(Default)]
pub struct LibraryState(Mutex<Option<Library>>);

/// L'elenco della Libreria è cambiato (o può essere cambiato): il frontend lo rilegge.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct LibraryChanged;

/// Esegue `f` sulla Libreria della cartella delle impostazioni, aprendola se è cambiata.
pub fn with<T>(
    app: &AppHandle,
    f: impl FnOnce(&mut Library) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let root = recordings_folder(app)?;
    let state = app.state::<LibraryState>();
    let mut current = state.0.lock().unwrap_or_else(PoisonError::into_inner);
    if current.as_ref().is_none_or(|l| l.root() != root) {
        let dir = app
            .path()
            .app_local_data_dir()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .join("libreria");
        *current = Some(Library::open(&root, &library::db_path(&dir, &root))?);
    }
    f(current.as_mut().expect("aperta qui sopra"))
}

/// Esegue un'operazione che cambia la Libreria e poi emette `library-changed`, anche se fallisce:
/// può aver cambiato la cartella a metà.
pub fn change<T>(
    app: &AppHandle,
    f: impl FnOnce(&mut Library) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let result = with(app, f);
    if let Err(e) = LibraryChanged.emit(app) {
        log::error!("library-changed: {e}");
    }
    result
}

/// Allinea l'indice alla cartella; un errore va nel log.
pub fn sync(app: &AppHandle) {
    if let Err(e) = change(app, Library::sync) {
        log::warn!("allineamento della Libreria: {e}");
    }
}

/// `sync` in un thread, per l'avvio e il ritorno in primo piano della finestra.
pub fn sync_in_background(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || sync(&app));
}
