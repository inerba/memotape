//! Una sola Attività alla volta, con il modo di fermarla.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::error::AppError;
use crate::library::inside;
use crate::tape;

/// Come si ferma l'Attività in corso: Annulla per la Trascrizione, Stop per la Registrazione.
type Stop = Box<dyn Fn() + Send>;

struct Current {
    stop: Stop,
    /// Il Tape, o la cartella del Tape che nascerà, su cui lavora: lì le scritture si rifiutano.
    target: Option<PathBuf>,
    /// Il lock dell'Attività su `target`, per il server MCP in un altro processo (ADR-0031).
    _lock: Option<File>,
}

/// L'Attività in corso, se c'è. Registrata in `tauri::State`.
#[derive(Default)]
pub struct Activity {
    current: Mutex<Option<Current>>,
}

impl Activity {
    /// Avvia un'Attività su `target`, che dura finché vive il guard restituito; `stop` la ferma.
    /// Se ce n'è già una, o se il server MCP sta scrivendo `target`, restituisce
    /// `AppError::ActivityInProgress`.
    pub fn begin(
        &self,
        target: Option<PathBuf>,
        stop: impl Fn() + Send + 'static,
    ) -> Result<ActivityGuard<'_>, AppError> {
        let mut current = self.current();
        if current.is_some() {
            return Err(AppError::ActivityInProgress);
        }
        // Non aspetta una scrittura del server MCP su `target`, che può durare (un Taglio): con il
        // mutex preso fermerebbe anche Annulla sul thread principale. Risponde `activityInProgress`.
        let lock = match &target {
            Some(path) => tape::prova_attivita(path)?,
            None => None,
        };
        *current = Some(Current {
            stop: Box::new(stop),
            target,
            _lock: lock,
        });
        Ok(ActivityGuard { activity: self })
    }

    /// Per scrivere `path`, un Tape o una Raccolta: senza Attività la prende per il tempo della
    /// scrittura; con un'Attività su un altro Tape si scrive senza; su `path` o dentro `path`
    /// risponde `activityInProgress`.
    pub fn write(&self, path: &Path) -> Result<Option<ActivityGuard<'_>>, AppError> {
        match &*self.current() {
            None => {}
            Some(current) if current.target.as_deref().is_some_and(|t| inside(t, path)) => {
                return Err(AppError::ActivityInProgress);
            }
            Some(_) => return Ok(None),
        }
        self.begin(Some(path.to_path_buf()), || {}).map(Some)
    }

    /// Ferma l'Attività in corso. Restituisce `false` se non ce n'era una.
    pub fn cancel(&self) -> bool {
        let current = self.current();
        if let Some(current) = &*current {
            (current.stop)();
        }
        current.is_some()
    }

    fn current(&self) -> MutexGuard<'_, Option<Current>> {
        // Il dato è un `Option` sempre valido: un panic altrove non lo lascia a metà.
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Un'Attività in corso: finisce al drop.
pub struct ActivityGuard<'a> {
    activity: &'a Activity,
}

impl Drop for ActivityGuard<'_> {
    fn drop(&mut self) {
        *self.activity.current() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn una_seconda_attivita_e_rifiutata_finche_la_prima_e_in_corso() {
        let activity = Activity::default();
        let first = activity.begin(None, || {}).unwrap();
        assert!(matches!(
            activity.begin(None, || {}),
            Err(AppError::ActivityInProgress)
        ));
        drop(first);
        assert!(activity.begin(None, || {}).is_ok());
    }

    #[test]
    fn annulla_ferma_solo_l_attivita_in_corso() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        let activity = Activity::default();
        // Senza Attività Annulla non fa nulla e non tocca quella successiva.
        assert!(!activity.cancel());
        let stops = Arc::new(AtomicU32::new(0));
        let counter = Arc::clone(&stops);
        let guard = activity
            .begin(None, move || {
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .unwrap();
        assert_eq!(stops.load(Ordering::Relaxed), 0);
        assert!(activity.cancel());
        assert_eq!(stops.load(Ordering::Relaxed), 1);
        drop(guard);
        assert!(!activity.cancel());
        assert_eq!(stops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn mentre_l_assistente_scrive_il_tape_l_attivita_si_rifiuta_senza_aspettare() {
        use std::sync::mpsc;
        use std::time::Duration;
        let root = crate::audio_toolkit::ogg_opus::tests::temp_dir("attivita-assistente");
        crate::library::Library::open(&root, &root.join("indice.db")).unwrap();
        let path = root.join("Call.tape");
        // Il server MCP sta scrivendo il Tape.
        let assistente = tape::prova_attivita(&path).unwrap();
        assert!(assistente.is_some());
        let activity = Activity::default();
        let (sent, received) = mpsc::channel();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                let begun = activity.begin(Some(path.clone()), || {}).map(drop);
                // Annulla, sul thread principale dell'app, non aspetta l'Assistente.
                let cancelled = activity.cancel();
                sent.send((begun, cancelled)).unwrap();
            });
            let result = received.recv_timeout(Duration::from_secs(2));
            drop(assistente);
            let (begun, cancelled) = result.expect("ha aspettato l'Assistente");
            assert!(matches!(begun, Err(AppError::ActivityInProgress)));
            assert!(!cancelled);
        });
        // Finita la scrittura dell'Assistente, l'Attività parte.
        assert!(activity.begin(Some(path), || {}).is_ok());
    }

    #[test]
    fn le_scritture_si_rifiutano_solo_sul_tape_dell_attivita() {
        let raccolta = Path::new(r"C:\Memotape\Acme");
        let tape = raccolta.join("Call.tape");
        let altro = Path::new(r"C:\Memotape\Altro.tape");
        // Senza Attività la scrittura la prende.
        assert!(Activity::default().write(altro).unwrap().is_some());
        let activity = Activity::default();
        let guard = activity.begin(Some(tape.clone()), || {}).unwrap();
        assert!(matches!(
            activity.write(&tape),
            Err(AppError::ActivityInProgress)
        ));

        // Windows non distingue maiuscole e minuscole.
        let upper = PathBuf::from(tape.to_string_lossy().to_uppercase());
        assert!(matches!(
            activity.write(&upper),
            Err(AppError::ActivityInProgress)
        ));
        // Nemmeno la Raccolta che lo contiene.
        assert!(matches!(
            activity.write(raccolta),
            Err(AppError::ActivityInProgress)
        ));
        assert!(activity.write(altro).unwrap().is_none());
        drop(guard);
        let writing = activity.write(altro).unwrap();
        assert!(writing.is_some());
        assert!(matches!(
            activity.begin(None, || {}),
            Err(AppError::ActivityInProgress)
        ));
    }
}
