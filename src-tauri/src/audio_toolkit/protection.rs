//! Protezione delle candidate Frasi, indipendente dal trattamento del PCM.
use std::sync::{Arc, Mutex, PoisonError};

use super::processing::{AudioProcessor, Boundary, PcmBlock};
use crate::error::AppError;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum Sensibilita {
    Spento,
    Sensibile,
    #[default]
    Bilanciato,
    Selettivo,
}

/// Revisioni catturate sul PCM decodificato, prima dei buffer del filtro e dell'ASR.
/// Coordinate a 16 kHz; una revisione vale dal primo campione del blocco.
#[derive(Clone, Default)]
pub struct ProtectionTimeline(Arc<Mutex<Vec<(u64, Sensibilita)>>>);

impl ProtectionTimeline {
    pub fn record(&self, sample: u64, level: Sensibilita) {
        let mut revisions = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if revisions
            .last()
            .is_none_or(|(_, previous)| *previous != level)
        {
            revisions.push((sample, level));
        }
    }

    pub fn level(&self, sample: u64) -> Sensibilita {
        let revisions = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let index = revisions.partition_point(|(start, _)| *start <= sample);
        index
            .checked_sub(1)
            .map_or(Sensibilita::Spento, |i| revisions[i].1)
    }

    pub fn capture(
        &self,
        processor: Box<dyn AudioProcessor>,
        read: impl FnMut() -> Sensibilita + Send + 'static,
    ) -> Box<dyn AudioProcessor> {
        Box::new(CaptureProtection {
            processor,
            read: Box::new(read),
            timeline: self.clone(),
        })
    }
}

struct CaptureProtection {
    processor: Box<dyn AudioProcessor>,
    read: Box<dyn FnMut() -> Sensibilita + Send>,
    timeline: ProtectionTimeline,
}

impl AudioProcessor for CaptureProtection {
    fn max_pending_frames(&self) -> usize {
        self.processor.max_pending_frames()
    }
    fn process(&mut self, block: PcmBlock<'_>, out: &mut Vec<f32>) -> Result<(), AppError> {
        self.timeline.record(
            block.start_frame * 16_000 / u64::from(block.format.rate),
            (self.read)(),
        );
        self.processor.process(block, out)
    }
    fn flush(&mut self, boundary: Boundary, out: &mut Vec<f32>) -> Result<(), AppError> {
        self.processor.flush(boundary, out)
    }
}

/// Evidenza distinta per revisione: cambiare il livello non revoca una parte già ammessa.
#[derive(Default)]
pub struct PhraseEvidence {
    groups: Vec<(Sensibilita, Evidence)>,
}

#[derive(Default)]
struct Evidence {
    voiced: usize,
    probability: f32,
    peak: f32,
    crossings: f32,
}

impl PhraseEvidence {
    pub fn push(&mut self, frame: &[f32], probability: f32, level: Sensibilita) {
        // Silero può conservare uno stato alto durante zeri: non sono evidenza acustica.
        if probability < 0.4 || (level != Sensibilita::Spento && !frame.iter().any(|s| *s != 0.0)) {
            return;
        }
        if self
            .groups
            .last()
            .is_none_or(|(previous, _)| *previous != level)
        {
            self.groups.push((level, Evidence::default()));
        }
        let evidence = &mut self.groups.last_mut().expect("gruppo inserito").1;
        evidence.voiced += 1;
        evidence.probability += probability;
        evidence.peak = evidence.peak.max(probability);
        evidence.crossings += frame
            .windows(2)
            .filter(|w| w[0].is_sign_positive() != w[1].is_sign_positive())
            .count() as f32
            / frame.len() as f32;
    }

    pub fn accepts(&self) -> bool {
        self.groups.iter().any(|(level, e)| {
            if *level == Sensibilita::Spento {
                return true;
            }
            let (crossings, mean) = (
                e.crossings / e.voiced as f32,
                e.probability / e.voiced as f32,
            );
            // Taratura sul corpus annotato del ticket 05. ZCR relativa, senza soglia energetica
            // o durata minima. L'evidenza molto forte conserva consonanti brevi non periodiche.
            let (limit, probability) = match level {
                Sensibilita::Sensibile => (0.48, 0.4),
                Sensibilita::Bilanciato => (0.40, 0.4),
                Sensibilita::Selettivo => (0.20, 0.6),
                Sensibilita::Spento => unreachable!(),
            };
            (crossings <= limit && mean >= probability) || e.peak >= 0.98
        })
    }
}
