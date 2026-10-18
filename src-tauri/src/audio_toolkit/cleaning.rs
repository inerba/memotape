//! Configurazione della pulizia osservata prima delle code ASR; metadati del PCM elaborato.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use super::processing::{AudioProcessor, Boundary, Format, PcmBlock};
use super::rnnoise::{self, RNNoise};
use crate::error::AppError;
use crate::transcript::Ingresso;

/// Intervallo semiaperto sulla linea del tempo dell'Ingresso. La frequenza è l'unità dei frame,
/// anche se la copia Ogg è ricampionata: la posizione temporale non cambia.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrattoPulizia {
    pub ingresso: Ingresso,
    pub algoritmo: String,
    pub versione: String,
    pub frequenza: u32,
    pub inizio_frame: u64,
    pub fine_frame: u64,
}

#[derive(Clone, Default)]
pub struct CleaningLog(Arc<Mutex<Vec<TrattoPulizia>>>);

impl CleaningLog {
    pub fn intervals(&self) -> Vec<TrattoPulizia> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

type ProcessorFactory = Box<dyn FnMut(Format) -> Result<Box<dyn AudioProcessor>, AppError> + Send>;

struct ActiveCleaning {
    processor: Box<dyn AudioProcessor>,
    interval: TrattoPulizia,
    /// Copia transitoria e limitata della sola coda non ancora consegnata, per il bypass.
    pending: VecDeque<f32>,
    channels: usize,
}

/// Osserva il profilo all'arrivo del blocco, prima del VAD e di tutte le destinazioni.
pub struct ConfiguredCleaning {
    enabled: Box<dyn FnMut() -> bool + Send>,
    factory: ProcessorFactory,
    ingresso: Ingresso,
    active: Option<ActiveCleaning>,
    log: CleaningLog,
    on_failure: Option<Box<dyn FnMut(AppError) + Send>>,
    failed: bool,
    /// Il processore scaricato a un confine, riusato quando la pulizia riparte.
    prepared: Option<Box<dyn AudioProcessor>>,
}

impl ConfiguredCleaning {
    pub fn new(
        path: PathBuf,
        enabled: impl FnMut() -> bool + Send + 'static,
        ingresso: Ingresso,
        log: CleaningLog,
    ) -> Self {
        Self::with_factory(
            Box::new(enabled),
            ingresso,
            log,
            Box::new(move |format| Ok(Box::new(RNNoise::new(&path, format)?))),
        )
    }

    pub(crate) fn with_factory(
        enabled: Box<dyn FnMut() -> bool + Send>,
        ingresso: Ingresso,
        log: CleaningLog,
        factory: ProcessorFactory,
    ) -> Self {
        Self {
            enabled,
            factory,
            ingresso,
            active: None,
            log,
            on_failure: None,
            failed: false,
            prepared: None,
        }
    }

    pub(crate) fn recovering(mut self, on_failure: impl FnMut(AppError) + Send + 'static) -> Self {
        self.on_failure = Some(Box::new(on_failure));
        self
    }

    fn record_interval(&self, interval: TrattoPulizia) {
        if interval.fine_frame > interval.inizio_frame {
            self.log
                .0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(interval);
        }
    }

    fn recover(&mut self, error: AppError, out: &mut Vec<f32>) -> Result<(), AppError> {
        let Some(on_failure) = &mut self.on_failure else {
            return Err(error);
        };
        self.failed = true;
        on_failure(error);
        if let Some(active) = self.active.take() {
            out.extend(active.pending);
            self.record_interval(active.interval);
        }
        self.prepared = None;
        Ok(())
    }

    fn commit(
        active: &mut ActiveCleaning,
        samples: Vec<f32>,
        channels: usize,
        out: &mut Vec<f32>,
    ) -> Result<(), AppError> {
        if !samples.len().is_multiple_of(channels)
            || samples.len() > active.pending.len()
            || samples.iter().any(|s| !s.is_finite())
            || active.pending.len() - samples.len()
                > active.processor.max_pending_frames() * channels
        {
            return Err(AppError::AudioCleaningFailed("coda PCM non valida".into()));
        }
        active.pending.drain(..samples.len());
        active.interval.fine_frame += (samples.len() / channels) as u64;
        out.extend(samples);
        Ok(())
    }

    fn drain(&mut self, boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
        if let Some(active) = &mut self.active {
            let mut tail = Vec::new();
            let keep = self.on_failure.is_some() && boundary != Boundary::Finish;
            let result = active.processor.flush(
                if keep {
                    Boundary::Configuration
                } else {
                    Boundary::Finish
                },
                &mut tail,
            );
            if let Err(error) = result {
                return self.recover(error, out);
            }
            // La coda completa deve contenere esattamente tutti i campioni rimasti.
            if tail.len() != active.pending.len() || tail.iter().any(|s| !s.is_finite()) {
                return self.recover(
                    AppError::AudioCleaningFailed("coda PCM incompleta".into()),
                    out,
                );
            }
            let mut active = self.active.take().unwrap();
            active.interval.fine_frame += (tail.len() / active.channels) as u64;
            out.extend(tail);
            self.record_interval(active.interval);
            if keep {
                self.prepared = Some(active.processor);
            }
        }
        Ok(())
    }
}

impl ConfiguredCleaning {
    fn segment(
        &mut self,
        block: PcmBlock<'_>,
        enabled: bool,
        out: &mut Vec<f32>,
    ) -> Result<(), AppError> {
        if !enabled {
            self.drain(Boundary::Configuration, out)?;
        }
        if enabled && self.active.is_none() {
            let processor = match self.prepared.take() {
                Some(processor) => Ok(processor),
                None => (self.factory)(block.format),
            };
            let processor = match processor {
                Ok(processor) => processor,
                Err(error) => {
                    self.recover(error, out)?;
                    out.extend_from_slice(block.samples);
                    return Ok(());
                }
            };
            self.active = Some(ActiveCleaning {
                processor,
                pending: VecDeque::new(),
                channels: block.format.channels,
                interval: TrattoPulizia {
                    ingresso: self.ingresso,
                    algoritmo: rnnoise::ALGORITMO.into(),
                    versione: rnnoise::VERSIONE.into(),
                    frequenza: block.format.rate,
                    inizio_frame: block.start_frame,
                    fine_frame: block.start_frame,
                },
            });
        }
        match &mut self.active {
            Some(active) => {
                active.pending.extend(block.samples);
                let mut processed = Vec::new();
                let result = active
                    .processor
                    .process(block, &mut processed)
                    .and_then(|()| Self::commit(active, processed, block.format.channels, out));
                if let Err(error) = result {
                    self.recover(error, out)?;
                }
            }
            None => out.extend_from_slice(block.samples),
        }
        Ok(())
    }
}

impl AudioProcessor for ConfiguredCleaning {
    fn max_pending_frames(&self) -> usize {
        self.active
            .as_ref()
            .map_or(0, |active| active.processor.max_pending_frames())
    }

    fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
        if block.samples.is_empty() {
            return Ok(());
        }
        let enabled = !self.failed && (self.enabled)();
        self.segment(block, enabled, out)
    }

    fn flush(&mut self, boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
        self.drain(boundary, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::audio_toolkit::processing::{PcmStream, tests::DelayedScale};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn guasto_con_coda_conserva_pcm_e_segnala_una_volta_senza_dichiarare_pulita_la_coda() {
        struct FailsWithTail;
        impl AudioProcessor for FailsWithTail {
            fn max_pending_frames(&self) -> usize {
                2
            }
            fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
                if block.start_frame == 0 {
                    out.push(block.samples[0] * 0.5);
                    Ok(())
                } else {
                    out.push(123.0); // Anche un output parziale di un'elaborazione fallita si scarta.
                    Err(AppError::AudioCleaningFailed("prova".into()))
                }
            }
            fn flush(&mut self, _: Boundary, _: &mut Vec<f32>) -> Result<(), AppError> {
                unreachable!()
            }
        }
        let warnings = Arc::new(Mutex::new(Vec::new()));
        let notified = warnings.clone();
        let log = CleaningLog::default();
        let processor = ConfiguredCleaning::with_factory(
            Box::new(|| true),
            Ingresso::Microfono,
            log.clone(),
            Box::new(|_| Ok(Box::new(FailsWithTail))),
        )
        .recovering(move |error| notified.lock().unwrap().push(error));
        let mut stream = super::super::processing::PcmStream::new(
            Format {
                rate: 8000,
                channels: 1,
            },
            Box::new(processor),
        )
        .unwrap();
        let mut output = stream.push(&[0.8, 0.6, 0.4]).unwrap().samples;
        output.extend(stream.push(&[0.2]).unwrap().samples);
        output.extend(stream.push(&[-0.3]).unwrap().samples);
        output.extend(stream.boundary(Boundary::Finish).unwrap().samples);
        assert_eq!(output, [0.4, 0.6, 0.4, 0.2, -0.3]);
        assert_eq!(warnings.lock().unwrap().len(), 1);
        let intervals = log.intervals();
        assert_eq!(intervals.len(), 1);
        assert_eq!((intervals[0].inizio_frame, intervals[0].fine_frame), (0, 1));
    }

    #[test]
    fn guasto_nello_scarico_preserva_la_coda_e_il_modello_assente_avvisa_solo_se_richiesto() {
        struct FailsAtStop;
        impl AudioProcessor for FailsAtStop {
            fn max_pending_frames(&self) -> usize {
                2
            }
            fn process(&mut self, _: PcmBlock<'_>, _: &mut Vec<f32>) -> Result<(), AppError> {
                Ok(())
            }
            fn flush(&mut self, _: Boundary, _: &mut Vec<f32>) -> Result<(), AppError> {
                Err(AppError::AudioCleaningFailed("scarico".into()))
            }
        }
        let log = CleaningLog::default();
        let processor = ConfiguredCleaning::with_factory(
            Box::new(|| true),
            Ingresso::Sistema,
            log.clone(),
            Box::new(|_| Ok(Box::new(FailsAtStop))),
        )
        .recovering(|_| {});
        let format = Format {
            rate: 8000,
            channels: 2,
        };
        let mut stream = PcmStream::new(format, Box::new(processor)).unwrap();
        assert!(
            stream
                .push(&[0.4, -0.8, 0.6, -0.2])
                .unwrap()
                .samples
                .is_empty()
        );
        assert_eq!(
            stream.boundary(Boundary::Finish).unwrap().samples,
            [0.4, -0.8, 0.6, -0.2]
        );
        assert!(log.intervals().is_empty());
        let enabled = Arc::new(AtomicBool::new(false));
        let control = enabled.clone();
        let warnings = Arc::new(Mutex::new(Vec::new()));
        let notified = warnings.clone();
        // Come nella Registrazione: il modello si carica al primo blocco con la pulizia accesa.
        let processor = ConfiguredCleaning::with_factory(
            Box::new(move || control.load(Ordering::Relaxed)),
            Ingresso::Microfono,
            log,
            Box::new(|_| Err(AppError::AudioCleaningMissing)),
        )
        .recovering(move |error| notified.lock().unwrap().push(error));
        let mut stream = PcmStream::new(format, Box::new(processor)).unwrap();
        assert_eq!(stream.push(&[0.2, -0.2]).unwrap().samples, [0.2, -0.2]);
        assert!(warnings.lock().unwrap().is_empty());
        enabled.store(true, Ordering::Relaxed);
        assert_eq!(stream.push(&[0.3, -0.3]).unwrap().samples, [0.3, -0.3]);
        assert_eq!(stream.push(&[0.4, -0.4]).unwrap().samples, [0.4, -0.4]);
        stream.boundary(Boundary::Finish).unwrap();
        assert_eq!(*warnings.lock().unwrap(), [AppError::AudioCleaningMissing]);
    }

    #[test]
    fn un_cambio_vale_sul_blocco_successivo_e_scarica_la_coda_con_metadati_esatti() {
        let enabled = Arc::new(AtomicBool::new(false));
        let control = enabled.clone();
        let log = CleaningLog::default();
        let processor = ConfiguredCleaning::with_factory(
            Box::new(move || control.load(Ordering::Relaxed)),
            Ingresso::Mix,
            log.clone(),
            Box::new(|_| Ok(Box::new(DelayedScale::new(1)))),
        );
        let mut stream = PcmStream::new(
            Format {
                rate: 8000,
                channels: 2,
            },
            Box::new(processor),
        )
        .unwrap();
        let mut output = stream.push(&[0.2, 0.4]).unwrap().samples;
        enabled.store(true, Ordering::Relaxed);
        output.extend(stream.push(&[0.4, 0.8, 0.6, 0.2]).unwrap().samples);
        enabled.store(false, Ordering::Relaxed);
        output.extend(stream.push(&[0.8, 0.4]).unwrap().samples);
        output.extend(stream.boundary(Boundary::Finish).unwrap().samples);
        assert_eq!(output, [0.2, 0.4, 0.2, 0.4, 0.3, 0.1, 0.8, 0.4]);
        let intervals = log.intervals();
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0].ingresso, Ingresso::Mix);
        assert_eq!(intervals[0].algoritmo, "rnnoise");
        assert!(intervals[0].versione.ends_with("/direct-v1"));
        assert_eq!(
            (
                intervals[0].frequenza,
                intervals[0].inizio_frame,
                intervals[0].fine_frame
            ),
            (8000, 1, 3)
        );
    }
}
