//! VAD: il trait `VoiceDetector` e l'implementazione Silero v6.2 (ruolo VAD di transcribe-cpp).

use std::path::Path;

use sha2::{Digest, Sha256};
use transcribe_cpp::{Model, VadSession};

use crate::engine::transcribe_cpp::{catch_native, init_backends};
use crate::error::AppError;

/// Il GGUF di Silero v6.2, risorsa del bundle: `handy-computer/silero-vad-v6.2-gguf` alla revisione
/// `a9b2d61c20ae4ddf3644edb34969e8185a485448`.
pub const SILERO_FILE: &str = "resources/silero-vad-v6.2-F32.gguf";
const SILERO_SIZE: usize = 1_241_056;
const SILERO_SHA256: &str = "0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6";

/// Probabilità di parlato di un frame da `FRAME_SAMPLES` campioni a 16 kHz. `Send`: per un file
/// gira in un thread a parte, in parallelo al motore.
pub trait VoiceDetector: Send {
    fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError>;
    fn reset(&mut self);
}

/// Una sessione VAD con il suo modello: un `Silero` per pipeline, perché transcribe-cpp fa un
/// calcolo alla volta per modello e due Ingressi in parallelo non devono aspettarsi. Il backend è
/// quello che `AUTO` sceglie per il VAD, la CPU.
pub struct Silero(VadSession);

impl Silero {
    pub fn new(model: &Path) -> Result<Self, AppError> {
        validate(model)?;
        init_backends()?;
        catch_native(|| Model::load(model)?.vad_session())
            .map(Self)
            .map_err(|e| AppError::VadModelIncompatible(e.to_string()))
    }
}

/// Il modello deve essere proprio il GGUF di Silero v6.2 distribuito con l'app.
fn validate(path: &Path) -> Result<(), AppError> {
    let bytes = std::fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::VadModelMissing
        } else {
            AppError::VadModelIncompatible(error.to_string())
        }
    })?;
    if bytes.len() != SILERO_SIZE || format!("{:x}", Sha256::digest(&bytes)) != SILERO_SHA256 {
        return Err(AppError::VadModelIncompatible(
            "dimensione o SHA-256 non corrispondono a Silero v6.2".into(),
        ));
    }
    Ok(())
}

impl VoiceDetector for Silero {
    fn probability(&mut self, frame: &[f32]) -> Result<f32, AppError> {
        let session = &mut self.0;
        let result = catch_native(|| session.stream_feed(frame))?;
        result
            .probs
            .last()
            .copied()
            .ok_or_else(|| AppError::Internal(format!("Silero: frame di {} campioni", frame.len())))
    }

    fn reset(&mut self) {
        self.0.stream_reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::decode::Decoder;
    use crate::audio_toolkit::resample::{FRAME_SAMPLES, FrameResampler};
    use crate::audio_toolkit::segmenter::Params;

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    /// I frame del parlato della fixture.
    fn voice() -> Vec<Vec<f32>> {
        let mut decoder = Decoder::open(&root().join("tests/fixtures/parlato-it.wav")).unwrap();
        let mut resampler = FrameResampler::new(16_000).unwrap();
        let mut frames = Vec::new();
        while let Some(block) = decoder.next_block().unwrap() {
            frames.extend(resampler.push(&block.mono()));
        }
        frames.extend(resampler.finish());
        frames
    }

    fn probabilities(silero: &mut Silero, frames: &[Vec<f32>]) -> Vec<f32> {
        frames
            .iter()
            .map(|frame| silero.probability(frame).unwrap())
            .collect()
    }

    #[test]
    fn il_parlato_supera_la_soglia_e_il_silenzio_no() {
        let mut silero = Silero::new(&root().join(SILERO_FILE)).unwrap();
        let threshold = Params::default().threshold;
        let speech = probabilities(&mut silero, &voice());
        assert!(
            speech.windows(2).any(|p| p.iter().all(|&p| p >= threshold)),
            "{speech:?}"
        );
        silero.reset();
        let silence = probabilities(&mut silero, &vec![vec![0.0; FRAME_SAMPLES]; 50]);
        assert!(silence.iter().all(|&p| p < threshold), "{silence:?}");
    }

    #[test]
    fn reset_riporta_lo_stato_iniziale() {
        let mut silero = Silero::new(&root().join(SILERO_FILE)).unwrap();
        let frames = voice();
        let first = probabilities(&mut silero, &frames);
        silero.reset();
        assert_eq!(probabilities(&mut silero, &frames), first);
    }

    #[test]
    fn modello_assente_o_diverso_ha_un_errore_distinto() {
        let missing = std::env::temp_dir().join("memotape-silero-mancante.gguf");
        assert!(matches!(
            Silero::new(&missing),
            Err(AppError::VadModelMissing)
        ));
        let path = std::env::temp_dir().join("memotape-silero-errato.gguf");
        std::fs::write(&path, b"altro modello").unwrap();
        assert!(matches!(
            Silero::new(&path),
            Err(AppError::VadModelIncompatible(_))
        ));
        std::fs::remove_file(path).unwrap();
    }
}
