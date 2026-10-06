//! L'artefatto sperimentale del ticket 01: nessun download né sostituzione implicita.
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::AppError;

pub const NAME: &str = "Nemotron 3 Diarization BF16";
pub const SIZE: u64 = 198_937_280;
pub const SHA256: &str = "4b11ce10e009fedf496cc9f879dc634e67605ecbf240463a155e0128657019e3";

/// Verifica il file scelto prima di prenotarlo e quando viene caricato. Il nome non basta.
pub fn validate(path: &Path) -> Result<(), AppError> {
    let invalid = || AppError::LocalDiarizerIncompatible(path.display().to_string());
    let mut file = std::fs::File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::LocalDiarizerMissing
        } else {
            invalid()
        }
    })?;
    if file.metadata().map_err(|_| invalid())?.len() != SIZE {
        return Err(invalid());
    }
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let count = file.read(&mut buffer).map_err(|_| invalid())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if format!("{:x}", digest.finalize()) != SHA256 {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn un_artefatto_assente_o_estraneo_non_diventa_sortformer() {
        let dir = crate::audio_toolkit::ogg_opus::tests::temp_dir("nemotron3-artefatto");
        let path = dir.join("Nemotron-3-Diarization-BF16.gguf");
        assert_eq!(validate(&path), Err(AppError::LocalDiarizerMissing));
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(SIZE).unwrap();
        assert!(matches!(
            validate(&path),
            Err(AppError::LocalDiarizerIncompatible(_))
        ));
    }
}
