//! Un worker preparato, con prestito esclusivo per sessione. Lo stato Tract resta sul suo thread.

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::DeepFilter;
use crate::audio_toolkit::processing::Format;
use crate::error::AppError;

enum Availability {
    Loading,
    Ready(Result<DeepFilter, AppError>),
    Leased,
}

pub(super) struct Slot {
    availability: Mutex<Availability>,
    changed: Condvar,
}

impl Slot {
    pub(super) fn release(&self, filter: Result<DeepFilter, AppError>) {
        *self
            .availability
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Availability::Ready(filter);
        self.changed.notify_all();
    }
}

#[derive(Clone)]
pub(crate) struct PreparedFilter(Arc<Slot>);

impl PreparedFilter {
    pub(crate) fn failed(&self) -> bool {
        matches!(
            *self
                .0
                .availability
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
            Availability::Ready(Err(_))
        )
    }
    pub(crate) fn new(path: PathBuf, format: Format) -> Self {
        let slot = Arc::new(Slot {
            availability: Mutex::new(Availability::Loading),
            changed: Condvar::new(),
        });
        let returned = Arc::downgrade(&slot);
        let spawned = std::thread::Builder::new()
            .name("memotape-prepara-dfn3".into())
            .spawn(move || {
                if returned.strong_count() == 0 {
                    return;
                }
                let start = Instant::now();
                let filter = DeepFilter::load(&path, format, true);
                log::info!("DFN3 preparato ({format:?}) in {:?}", start.elapsed());
                if let Some(slot) = returned.upgrade() {
                    slot.release(filter);
                }
            });
        if let Err(error) = spawned {
            slot.release(Err(AppError::AudioCleaningFailed(error.to_string())));
        }
        Self(slot)
    }

    /// Non aspetta il modello: l'audio con pulizia spenta può partire subito.
    pub(crate) fn take_ready(&self) -> Result<Option<DeepFilter>, AppError> {
        let mut availability = self
            .0
            .availability
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Availability::Ready(Err(error)) = &*availability {
            return Err(error.clone());
        }
        if matches!(*availability, Availability::Ready(Ok(_))) {
            let Availability::Ready(Ok(mut filter)) =
                std::mem::replace(&mut *availability, Availability::Leased)
            else {
                unreachable!()
            };
            filter.return_to = Some(Arc::downgrade(&self.0));
            return Ok(Some(filter));
        }
        Ok(None)
    }

    /// Attesa solo per pulizia richiesta, con confini di annullamento fra le attese.
    pub(crate) fn wait(&self, cancelled: impl Fn() -> bool) -> Result<DeepFilter, AppError> {
        loop {
            if cancelled() {
                return Err(AppError::Cancelled);
            }
            if let Some(filter) = self.take_ready()? {
                return Ok(filter);
            }
            let availability = self
                .0
                .availability
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if !matches!(*availability, Availability::Ready(_)) {
                drop(
                    self.0
                        .changed
                        .wait_timeout(availability, Duration::from_millis(25))
                        .unwrap_or_else(PoisonError::into_inner),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::processing::{AudioProcessor, Boundary, PcmBlock};

    #[test]
    fn annulla_non_aspetta_un_filtro_ancora_in_preparazione() {
        let filter = PreparedFilter(Arc::new(Slot {
            availability: Mutex::new(Availability::Loading),
            changed: Condvar::new(),
        }));
        assert!(matches!(filter.wait(|| true), Err(AppError::Cancelled)));
        assert!(filter.take_ready().unwrap().is_none());
    }

    #[test]
    #[ignore = "DFN3 reale: riuso, isolamento e tempi; eseguire da solo"]
    fn dfn3_preparato_riusa_piano_e_azzera_audio_fra_sessioni() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(super::super::MODEL_FILE);
        for (rate, channels, inputs) in [(48_000, 1, 1), (48_000, 2, 2), (16_000, 1, 2)] {
            let format = Format { rate, channels };
            let cold = Instant::now();
            let slots: Vec<_> = (0..inputs)
                .map(|_| PreparedFilter::new(path.clone(), format))
                .collect();
            let mut filters: Vec<_> = slots
                .iter()
                .map(|slot| slot.wait(|| false).unwrap())
                .collect();
            let initialization = cold.elapsed();
            println!("PREPARED {rate} Hz {channels} canali {inputs} ingressi: {initialization:?}");
            std::thread::sleep(Duration::from_millis(150));
            assert!(slots[0].take_ready().unwrap().is_none());
            let input: Vec<_> = (0..rate as usize / 10 * channels)
                .map(|i| 0.1 * (i as f32 * 0.03).sin())
                .collect();
            let render = |filter: &mut DeepFilter| {
                let mut out = Vec::new();
                filter
                    .process(
                        PcmBlock {
                            format,
                            start_frame: 0,
                            samples: &input,
                        },
                        &mut out,
                    )
                    .unwrap();
                filter.flush(Boundary::Finish, &mut out).unwrap();
                assert_eq!(out.len(), input.len());
                out
            };
            let original: Vec<_> = filters.iter_mut().map(render).collect();
            drop(filters);
            let reuse = Instant::now();
            let mut filters: Vec<_> = slots
                .iter()
                .map(|slot| slot.wait(|| false).unwrap())
                .collect();
            println!(
                "REUSED {rate} Hz {channels} canali {inputs} ingressi: {:?}",
                reuse.elapsed()
            );
            for (filter, previous) in filters.iter_mut().zip(&original) {
                assert_eq!(
                    &render(filter),
                    previous,
                    "stato audio della sessione precedente"
                );
            }
            drop(filters);
            // Anche un prestito lasciato a metà, senza Finish, torna con stato audio nuovo.
            let mut abandoned = slots[0].wait(|| false).unwrap();
            abandoned
                .process(
                    PcmBlock {
                        format,
                        start_frame: 0,
                        samples: &input,
                    },
                    &mut Vec::new(),
                )
                .unwrap();
            drop(abandoned);
            let mut fresh = slots[0].wait(|| false).unwrap();
            assert_eq!(render(&mut fresh), original[0]);
        }
    }
}
