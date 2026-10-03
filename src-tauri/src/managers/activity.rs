//! Una sola Attività alla volta, con il token per annullarla.

use std::sync::{Mutex, MutexGuard, PoisonError};

use transcribe_cpp::CancelToken;

use crate::error::AppError;

/// L'Attività in corso, se c'è. Registrata in `tauri::State`.
#[derive(Default)]
pub struct Activity {
    current: Mutex<Option<CancelToken>>,
}

impl Activity {
    /// Avvia un'Attività, che dura finché vive il guard restituito.
    /// Se ce n'è già una restituisce `AppError::ActivityInProgress`.
    pub fn begin(&self) -> Result<ActivityGuard<'_>, AppError> {
        let mut current = self.current();
        if current.is_some() {
            return Err(AppError::ActivityInProgress);
        }
        let cancel = CancelToken::new();
        *current = Some(cancel.clone());
        Ok(ActivityGuard {
            activity: self,
            cancel,
        })
    }

    /// Annulla l'Attività in corso; senza Attività non fa nulla.
    pub fn cancel(&self) {
        if let Some(cancel) = &*self.current() {
            cancel.cancel();
        }
    }

    fn current(&self) -> MutexGuard<'_, Option<CancelToken>> {
        // Il dato è un `Option` sempre valido: un panic altrove non lo lascia a metà.
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Un'Attività in corso: finisce al drop.
pub struct ActivityGuard<'a> {
    activity: &'a Activity,
    /// Premuto da `Activity::cancel`.
    pub cancel: CancelToken,
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
        let first = activity.begin().unwrap();
        assert!(matches!(
            activity.begin(),
            Err(AppError::ActivityInProgress)
        ));
        drop(first);
        assert!(activity.begin().is_ok());
    }

    #[test]
    fn annulla_preme_il_token_dell_attivita_in_corso() {
        let activity = Activity::default();
        // Senza Attività Annulla non fa nulla e non tocca quella successiva.
        activity.cancel();
        let guard = activity.begin().unwrap();
        assert!(!guard.cancel.is_cancelled());
        activity.cancel();
        assert!(guard.cancel.is_cancelled());
        drop(guard);
        assert!(!activity.begin().unwrap().cancel.is_cancelled());
    }
}
