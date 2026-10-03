//! Il modello tenuto caricato tra una Trascrizione e l'altra.
//!
//! Il motore si ricarica solo se cambia il modello o se il modello caricato viene eliminato. Chi
//! lo usa lo prende in prestito (`Lease`); chi arriva intanto aspetta, così due caricamenti non
//! si sovrappongono e un modello non si elimina mentre lo si usa.

use std::ops::{Deref, DerefMut};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

use crate::error::AppError;

pub struct LoadedModel<E> {
    slot: Mutex<Slot<E>>,
    /// Segnala che il motore è tornato libero.
    released: Condvar,
}

struct Slot<E> {
    /// Il motore tenuto caricato, con l'id del suo modello.
    engine: Option<(&'static str, E)>,
    /// Il modello in caricamento o in uso.
    in_use: Option<&'static str>,
}

impl<E> Default for LoadedModel<E> {
    fn default() -> Self {
        Self {
            slot: Mutex::new(Slot {
                engine: None,
                in_use: None,
            }),
            released: Condvar::new(),
        }
    }
}

impl<E> LoadedModel<E> {
    /// Prende in prestito il motore del modello `id()`, letto quando il motore è libero; se non è
    /// quello tenuto, scarica il precedente e lo carica con `load`. Fino al rilascio del prestito il
    /// modello è in uso.
    pub fn take(
        &self,
        id: impl FnOnce() -> &'static str,
        load: impl FnOnce(&'static str) -> Result<E, AppError>,
    ) -> Result<Lease<'_, E>, AppError> {
        let mut slot = self.slot();
        while slot.in_use.is_some() {
            slot = self
                .released
                .wait(slot)
                .unwrap_or_else(PoisonError::into_inner);
        }
        let id = id();
        slot.in_use = Some(id);
        let kept = slot.engine.take();
        drop(slot);
        let engine = match kept {
            Some((kept_id, engine)) if kept_id == id => Ok(engine),
            // Il modello precedente si libera prima di caricare il nuovo: mai due in memoria.
            other => {
                drop(other);
                load(id)
            }
        };
        let mut lease = Lease {
            owner: self,
            id,
            engine: None,
        };
        lease.engine = Some(engine?);
        Ok(lease)
    }

    /// Il modello in caricamento o in uso.
    pub fn in_use(&self) -> Option<&'static str> {
        self.slot().in_use
    }

    /// Scarica il motore di `id`, prima di eliminarne il file. Se è in uso restituisce
    /// `AppError::ModelInUse`.
    pub fn evict(&self, id: &str) -> Result<(), AppError> {
        let mut slot = self.slot();
        if slot.in_use == Some(id) {
            return Err(AppError::ModelInUse(id.into()));
        }
        let evicted = slot.engine.take_if(|(kept, _)| *kept == id);
        drop(slot);
        drop(evicted);
        Ok(())
    }

    fn slot(&self) -> MutexGuard<'_, Slot<E>> {
        // Lo slot resta coerente anche se un panic altrove avvelena il lock.
        self.slot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Il motore in prestito: torna a `LoadedModel` al drop.
pub struct Lease<'a, E> {
    owner: &'a LoadedModel<E>,
    id: &'static str,
    engine: Option<E>,
}

impl<E> Lease<'_, E> {
    /// Il modello del motore.
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// Non restituisce il motore, che si ricaricherà al prossimo uso: dopo un guasto nativo non
    /// è detto che sia ancora sano.
    pub fn discard(mut self) {
        self.engine = None;
    }
}

impl<E> Deref for Lease<'_, E> {
    type Target = E;

    fn deref(&self) -> &E {
        self.engine
            .as_ref()
            .expect("il motore c'è finché vive il prestito")
    }
}

impl<E> DerefMut for Lease<'_, E> {
    fn deref_mut(&mut self) -> &mut E {
        self.engine
            .as_mut()
            .expect("il motore c'è finché vive il prestito")
    }
}

impl<E> Drop for Lease<'_, E> {
    fn drop(&mut self) {
        let mut slot = self.owner.slot();
        slot.engine = self.engine.take().map(|engine| (self.id, engine));
        slot.in_use = None;
        drop(slot);
        self.owner.released.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    /// Un motore finto che ricorda di quale modello è.
    #[derive(Debug, PartialEq)]
    struct Engine(&'static str);

    /// Prende il motore di `id` contando i caricamenti.
    fn take<'a>(
        loaded: &'a LoadedModel<Engine>,
        id: &'static str,
        loads: &Cell<u32>,
    ) -> Result<Lease<'a, Engine>, AppError> {
        loaded.take(
            || id,
            |id| {
                loads.set(loads.get() + 1);
                Ok(Engine(id))
            },
        )
    }

    #[test]
    fn la_seconda_trascrizione_con_lo_stesso_modello_non_lo_ricarica() {
        let loaded = LoadedModel::default();
        let loads = Cell::new(0);
        assert_eq!(
            *take(&loaded, "nemotron", &loads).unwrap(),
            Engine("nemotron")
        );
        assert_eq!(
            *take(&loaded, "nemotron", &loads).unwrap(),
            Engine("nemotron")
        );
        assert_eq!(loads.get(), 1);
    }

    #[test]
    fn cambiare_modello_carica_il_nuovo() {
        let loaded = LoadedModel::default();
        let loads = Cell::new(0);
        drop(take(&loaded, "nemotron", &loads).unwrap());
        assert_eq!(
            *take(&loaded, "whisper", &loads).unwrap(),
            Engine("whisper")
        );
        assert_eq!(
            *take(&loaded, "nemotron", &loads).unwrap(),
            Engine("nemotron")
        );
        assert_eq!(loads.get(), 3);
    }

    #[test]
    fn un_modello_in_uso_non_si_scarica_e_uno_eliminato_si_ricarica() {
        let loaded = LoadedModel::default();
        let loads = Cell::new(0);
        let lease = take(&loaded, "nemotron", &loads).unwrap();
        assert_eq!(loaded.in_use(), Some("nemotron"));
        assert!(matches!(
            loaded.evict("nemotron"),
            Err(AppError::ModelInUse(_))
        ));
        // Un altro modello si elimina anche durante la Trascrizione.
        loaded.evict("whisper").unwrap();
        drop(lease);
        assert_eq!(loaded.in_use(), None);
        loaded.evict("nemotron").unwrap();
        drop(take(&loaded, "nemotron", &loads).unwrap());
        assert_eq!(loads.get(), 2);
    }

    #[test]
    fn un_caricamento_fallito_o_un_motore_scartato_non_restano() {
        let loaded = LoadedModel::<Engine>::default();
        let failed = loaded.take(|| "nemotron", |_| Err(AppError::ModelMissing("x".into())));
        assert!(matches!(failed, Err(AppError::ModelMissing(_))));
        assert_eq!(loaded.in_use(), None);
        let loads = Cell::new(0);
        take(&loaded, "nemotron", &loads).unwrap().discard();
        assert_eq!(loaded.in_use(), None);
        drop(take(&loaded, "nemotron", &loads).unwrap());
        assert_eq!(loads.get(), 2);
    }

    #[test]
    fn chi_arriva_secondo_aspetta_e_trova_il_modello_gia_caricato() {
        let loaded = LoadedModel::default();
        let loads = std::sync::atomic::AtomicU32::new(0);
        let load = |id| {
            loads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::thread::sleep(std::time::Duration::from_millis(50));
            Ok(Engine(id))
        };
        std::thread::scope(|scope| {
            for _ in 0..3 {
                scope.spawn(|| drop(loaded.take(|| "nemotron", load).unwrap()));
            }
        });
        assert_eq!(loads.into_inner(), 1);
    }
}
