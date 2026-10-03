//! Una sola Attività alla volta, con il modo di fermarla.

use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::error::AppError;

/// Come si ferma l'Attività in corso: Annulla per la Trascrizione, Stop per la Registrazione.
type Stop = Box<dyn Fn() + Send>;

/// L'Attività in corso, se c'è. Registrata in `tauri::State`.
#[derive(Default)]
pub struct Activity {
    current: Mutex<Option<Stop>>,
}

impl Activity {
    /// Avvia un'Attività, che dura finché vive il guard restituito; `stop` la ferma.
    /// Se ce n'è già una restituisce `AppError::ActivityInProgress`.
    pub fn begin(&self, stop: impl Fn() + Send + 'static) -> Result<ActivityGuard<'_>, AppError> {
        let mut current = self.current();
        if current.is_some() {
            return Err(AppError::ActivityInProgress);
        }
        *current = Some(Box::new(stop));
        Ok(ActivityGuard { activity: self })
    }

    /// Ferma l'Attività in corso. Restituisce `false` se non ce n'era una.
    pub fn cancel(&self) -> bool {
        let current = self.current();
        if let Some(stop) = &*current {
            stop();
        }
        current.is_some()
    }

    fn current(&self) -> MutexGuard<'_, Option<Stop>> {
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
        let first = activity.begin(|| {}).unwrap();
        assert!(matches!(
            activity.begin(|| {}),
            Err(AppError::ActivityInProgress)
        ));
        drop(first);
        assert!(activity.begin(|| {}).is_ok());
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
            .begin(move || {
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
}
