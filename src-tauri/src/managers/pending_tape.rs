//! Il Tape arrivato con un avvio (doppio clic in Esplora file) che la finestra non ha ancora preso.
//! Resta qui finché il frontend lo chiede, così non si perde se la pagina sta ancora caricando.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::tape;

/// In `tauri::State`.
#[derive(Default)]
pub struct PendingTape(Mutex<Option<String>>);

impl PendingTape {
    pub fn take(&self) -> Option<String> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

/// È arrivato un Tape da aprire: la finestra lo prende con `take_pending_tape`.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct TapeRequested;

/// Tiene il Tape tra gli argomenti `args` di un avvio, se c'è, e avvisa la finestra.
pub fn request(app: &AppHandle, args: impl IntoIterator<Item = String>, cwd: &Path) {
    let Some(path) = tape::from_args(args, cwd) else {
        return;
    };
    *app.state::<PendingTape>()
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some(path.display().to_string());
    if let Err(e) = TapeRequested.emit(app) {
        log::error!("tape-requested: {e}");
    }
}
