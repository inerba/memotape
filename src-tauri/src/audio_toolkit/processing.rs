//! Un solo contratto PCM prima delle destinazioni, con stato indipendente per Ingresso.
//! Il processore restituisce campioni utili nello stesso formato e ordine dell'audio ricevuto:
//! il ritardo è attesa di calcolo, mai silenzio aggiunto alla linea del tempo.

use crate::error::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    pub rate: u32,
    pub channels: usize,
}

#[derive(Clone, Copy)]
pub struct PcmBlock<'a> {
    pub format: Format,
    /// Frame dall'origine dell'Ingresso, pause escluse (non campioni interleaved).
    pub start_frame: u64,
    pub samples: &'a [f32],
}

pub struct ProcessedBlock {
    pub format: Format,
    pub start_frame: u64,
    pub samples: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    Pause,
    Configuration,
    Finish,
}

/// Unica seam nuova: l'adattatore possiede anche ricampionamento interno, stato e compensazione
/// del proprio ritardo. `flush` deve consegnare tutta la coda utile senza azzerare l'origine;
/// Pausa e Configurazione permettono altro audio, Finish chiude definitivamente la sessione.
/// Non si deve inferire che inviare zeri a un runtime equivalga a scaricarne la coda.
pub trait AudioProcessor: Send {
    fn max_pending_frames(&self) -> usize;
    fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError>;
    fn flush(&mut self, boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError>;
}

pub struct Bypass;

impl AudioProcessor for Bypass {
    fn max_pending_frames(&self) -> usize {
        0
    }

    fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
        out.extend_from_slice(block.samples);
        Ok(())
    }

    fn flush(&mut self, _boundary: Boundary, _out: &mut Vec<f32>) -> Result<(), AppError> {
        Ok(())
    }
}

pub struct PcmStream {
    format: Format,
    processor: Box<dyn AudioProcessor>,
    received: u64,
    emitted: u64,
    closed: bool,
}

impl PcmStream {
    pub fn new(format: Format, processor: Box<dyn AudioProcessor>) -> Result<Self, AppError> {
        if format.rate == 0 || format.channels == 0 {
            return Err(AppError::Internal("formato PCM non valido".into()));
        }
        Ok(Self {
            format,
            processor,
            received: 0,
            emitted: 0,
            closed: false,
        })
    }

    pub fn push(&mut self, samples: &[f32]) -> Result<ProcessedBlock, AppError> {
        if self.closed || !samples.len().is_multiple_of(self.format.channels) {
            return Err(AppError::Internal(
                "blocco PCM non valido o sessione chiusa".into(),
            ));
        }
        let block = PcmBlock {
            format: self.format,
            start_frame: self.received,
            samples,
        };
        let mut out = Vec::new();
        if let Err(error) = self.processor.process(block, &mut out) {
            self.closed = true;
            return Err(error);
        }
        self.received = block.start_frame + (samples.len() / block.format.channels) as u64;
        self.output(out, false)
    }

    pub fn boundary(&mut self, boundary: Boundary) -> Result<ProcessedBlock, AppError> {
        if self.closed {
            return Err(AppError::Internal("sessione PCM chiusa".into()));
        }
        let mut out = Vec::new();
        if let Err(error) = self.processor.flush(boundary, &mut out) {
            self.closed = true;
            return Err(error);
        }
        let block = self.output(out, true)?;
        self.closed = boundary == Boundary::Finish;
        Ok(block)
    }

    /// Il chiamante consegna la coda alle destinazioni prima di usare il nuovo processore.
    #[allow(
        dead_code,
        reason = "contratto preparatorio del ticket 01; nessun controllo utente in questo ticket"
    )]
    pub fn reconfigure(
        &mut self,
        processor: Box<dyn AudioProcessor>,
    ) -> Result<ProcessedBlock, AppError> {
        let tail = self.boundary(Boundary::Configuration)?;
        self.processor = processor;
        Ok(tail)
    }

    fn output(&mut self, samples: Vec<f32>, drained: bool) -> Result<ProcessedBlock, AppError> {
        let frames = (samples.len() / self.format.channels) as u64;
        let end = self.emitted + frames;
        if !samples.len().is_multiple_of(self.format.channels)
            || end > self.received
            || self.received - end > self.processor.max_pending_frames() as u64
            || (drained && end != self.received)
        {
            self.closed = true;
            return Err(AppError::Internal(
                "il processore PCM non conserva durata o canali".into(),
            ));
        }
        let block = ProcessedBlock {
            format: self.format,
            start_frame: self.emitted,
            samples,
        };
        self.emitted = end;
        Ok(block)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Trattiene una coda finita e dimezza il PCM, senza cambiare canali o tempi.
    pub(crate) struct DelayedScale {
        pending: Vec<f32>,
        delay: usize,
    }

    impl DelayedScale {
        pub(crate) fn new(delay: usize) -> Self {
            Self {
                pending: Vec::new(),
                delay,
            }
        }
    }

    impl AudioProcessor for DelayedScale {
        fn max_pending_frames(&self) -> usize {
            self.delay
        }

        fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
            self.pending.extend(block.samples.iter().map(|s| s * 0.5));
            let ready = self
                .pending
                .len()
                .saturating_sub(self.delay * block.format.channels);
            out.extend(self.pending.drain(..ready));
            Ok(())
        }

        fn flush(&mut self, _boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
            out.append(&mut self.pending);
            Ok(())
        }
    }

    #[test]
    fn una_coda_persa_non_puo_diventare_una_sessione_completata() {
        struct MissingTail;
        impl AudioProcessor for MissingTail {
            fn max_pending_frames(&self) -> usize {
                2
            }
            fn process(
                &mut self,
                _block: PcmBlock<'_>,
                _out: &mut Vec<f32>,
            ) -> Result<(), AppError> {
                Ok(())
            }
            fn flush(&mut self, _boundary: Boundary, _out: &mut Vec<f32>) -> Result<(), AppError> {
                Ok(())
            }
        }
        let mut stream = PcmStream::new(
            Format {
                rate: 16_000,
                channels: 1,
            },
            Box::new(MissingTail),
        )
        .unwrap();
        stream.push(&[0.8, -0.4]).unwrap();
        assert!(stream.boundary(Boundary::Finish).is_err());
        assert!(stream.push(&[]).is_err());
    }

    #[test]
    fn cambio_di_processore_scarica_il_vecchio_audio_e_vale_solo_sul_successivo() {
        let mut stream = PcmStream::new(
            Format {
                rate: 16_000,
                channels: 1,
            },
            Box::new(DelayedScale::new(2)),
        )
        .unwrap();
        assert!(stream.push(&[0.8, -0.4]).unwrap().samples.is_empty());
        let before = stream.reconfigure(Box::new(Bypass)).unwrap();
        assert_eq!(before.start_frame, 0);
        assert_eq!(before.samples, [0.4, -0.2]);
        let after = stream.push(&[0.8, -0.4]).unwrap();
        assert_eq!(after.start_frame, 2);
        assert_eq!(after.samples, [0.8, -0.4]);
        assert!(
            stream
                .boundary(Boundary::Finish)
                .unwrap()
                .samples
                .is_empty()
        );
    }

    #[test]
    fn ritardo_pause_e_cambi_conservano_posizione_canali_e_coda() {
        let mut stream = PcmStream::new(
            Format {
                rate: 48_000,
                channels: 2,
            },
            Box::new(DelayedScale::new(2)),
        )
        .unwrap();
        let first = stream.push(&[0.8, -0.4, 0.6, -0.2, 0.4, -0.8]).unwrap();
        assert_eq!(first.start_frame, 0);
        assert_eq!(first.samples, [0.4, -0.2]);
        let tail = stream.boundary(Boundary::Pause).unwrap();
        assert_eq!(tail.start_frame, 1);
        assert_eq!(tail.samples, [0.3, -0.1, 0.2, -0.4]);
        assert!(
            stream
                .boundary(Boundary::Configuration)
                .unwrap()
                .samples
                .is_empty()
        );
        assert!(stream.push(&[1.0, -1.0]).unwrap().samples.is_empty());
        let end = stream.boundary(Boundary::Finish).unwrap();
        assert_eq!(end.start_frame, 3);
        assert_eq!(end.samples, [0.5, -0.5]);
        assert!(stream.push(&[0.0, 0.0]).is_err());
    }
}
