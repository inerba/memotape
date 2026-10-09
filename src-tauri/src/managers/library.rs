//! La Libreria della Cartella della Libreria corrente, in `tauri::State`: la riapre quando la
//! cartella cambia ed emette `library-changed` dopo ogni operazione e dopo gli allineamenti che
//! cambiano l'indice.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::error::AppError;
use crate::library::{self, Library};
use crate::managers::recording::recordings_folder;

/// In `tauri::State`.
#[derive(Default)]
pub struct LibraryState {
    library: Mutex<Option<Library>>,
    /// Un allineamento in background è in attesa e non ha ancora letto la cartella: quelli chiesti
    /// intanto (focus ripetuti della finestra) li copre lui.
    queued: AtomicBool,
}

/// L'elenco della Libreria è cambiato (o può essere cambiato): il frontend lo rilegge.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct LibraryChanged;

/// La Libreria della cartella delle impostazioni, aperta se è cambiata (`true`).
fn current(app: &AppHandle) -> Result<(MutexGuard<'_, Option<Library>>, bool), AppError> {
    let root = recordings_folder(app)?;
    let state = app.state::<LibraryState>();
    let mut current = state
        .inner()
        .library
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let reopened = current.as_ref().is_none_or(|l| l.root() != root);
    if reopened {
        let dir = app
            .path()
            .app_local_data_dir()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .join("libreria");
        *current = Some(Library::open(&root, &library::db_path(&dir, &root))?);
    }
    Ok((current, reopened))
}

/// Esegue `f` sulla Libreria della cartella delle impostazioni, aprendola se è cambiata.
pub fn with<T>(
    app: &AppHandle,
    f: impl FnOnce(&mut Library) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let (mut current, _) = current(app)?;
    f(current.as_mut().expect("aperta da current"))
}

/// Esegue un'operazione che cambia la Libreria e poi emette `library-changed`, anche se fallisce:
/// può aver cambiato la cartella a metà.
pub fn change<T>(
    app: &AppHandle,
    f: impl FnOnce(&mut Library) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let result = with(app, f);
    emit(app);
    result
}

fn emit(app: &AppHandle) {
    if let Err(e) = LibraryChanged.emit(app) {
        log::error!("library-changed: {e}");
    }
}

/// Allinea l'indice alla cartella ed emette `library-changed` solo se l'elenco può essere
/// cambiato: Tape riletti o tolti, un'altra Cartella della Libreria o un errore. Un errore va nel
/// log.
pub fn sync(app: &AppHandle) {
    let synced = current(app).and_then(|(mut current, reopened)| {
        // Da qui la cartella si legge di nuovo: un allineamento chiesto ora non è più coperto.
        app.state::<LibraryState>()
            .queued
            .store(false, Ordering::Release);
        let changed = current.as_mut().expect("aperta da current").sync()?;
        Ok(reopened || changed > 0)
    });
    match synced {
        Ok(false) => {}
        Ok(true) => emit(app),
        Err(e) => {
            // Anche se la Libreria non si è aperta: gli allineamenti successivi devono partire.
            app.state::<LibraryState>()
                .queued
                .store(false, Ordering::Release);
            log::warn!("allineamento della Libreria: {e}");
            emit(app);
        }
    }
}

/// `sync` in un thread, per l'avvio e il ritorno in primo piano della finestra. Se uno è già in
/// attesa del suo turno non ne parte un altro: leggerà la cartella dopo questa richiesta.
pub fn sync_in_background(app: &AppHandle) {
    if app
        .state::<LibraryState>()
        .queued
        .swap(true, Ordering::AcqRel)
    {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || sync(&app));
}
