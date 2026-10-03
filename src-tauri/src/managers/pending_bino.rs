//! Il Bino arrivato con un avvio (doppio clic in Esplora file) che la finestra non ha ancora preso.
//! Resta qui finché il frontend lo chiede, così non si perde se la pagina sta ancora caricando.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::bino;

/// In `tauri::State`.
#[derive(Default)]
pub struct PendingBino(Mutex<Option<String>>);

impl PendingBino {
    pub fn take(&self) -> Option<String> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

/// È arrivato un Bino da aprire: la finestra lo prende con `take_pending_bino`.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct BinoRequested;

/// Tiene il Bino tra gli argomenti `args` di un avvio, se c'è, e avvisa la finestra.
pub fn request(app: &AppHandle, args: impl IntoIterator<Item = String>, cwd: &Path) {
    let Some(path) = bino::from_args(args, cwd) else {
        return;
    };
    *app.state::<PendingBino>()
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some(path.display().to_string());
    if let Err(e) = BinoRequested.emit(app) {
        log::error!("bino-requested: {e}");
    }
}
