//! Decodifica dei file con Symphonia (+ libopus per Opus) in blocchi di campioni interleaved.

use std::path::Path;
use std::sync::LazyLock;

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, TrackType};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;

use crate::audio_toolkit::forma_onda::Picchi;
use crate::bino;
use crate::error::AppError;

static CODECS: LazyLock<CodecRegistry> = LazyLock::new(|| {
    let mut registry = CodecRegistry::new();
    symphonia::default::register_enabled_codecs(&mut registry);
    registry.register_audio_decoder::<symphonia_adapter_libopus::OpusDecoder>();
    registry
});

pub struct Decoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    /// Frame dichiarati dal container (`n_frames`), se li conosce.
    total_frames: Option<u64>,
    decoded_frames: u64,
}

impl Decoder {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let unreadable = |e: &dyn std::fmt::Display| {
            AppError::UnreadableFile(format!("{}: {e}", path.display()))
        };
        let mut hint = Hint::new();
        // Di un Bino si decodifica il mix.
        let source: Box<dyn MediaSource> = if bino::is_bino(path) {
            hint.with_extension("ogg");
            Box::new(bino::Mix::open(path)?)
        } else {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                hint.with_extension(ext);
            }
            Box::new(std::fs::File::open(path).map_err(|e| unreadable(&e))?)
        };
        let format = symphonia::default::get_probe()
            .probe(
                &hint,
                MediaSourceStream::new(source, Default::default()),
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .map_err(|e| match e {
                SymphoniaError::IoError(e) => unreadable(&e),
                e => AppError::UnsupportedCodec(e.to_string()),
            })?;
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| AppError::UnsupportedCodec("nessuna traccia audio".into()))?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or_else(|| AppError::UnsupportedCodec("parametri del codec assenti".into()))?;
        let decoder = CODECS
            .make_audio_decoder(params, &AudioDecoderOptions::default())
            .map_err(|e| AppError::UnsupportedCodec(e.to_string()))?;
        let track_id = track.id;
        let total_frames = track.num_frames.filter(|&n| n > 0);
        Ok(Self {
            format,
            decoder,
            track_id,
            total_frames,
            decoded_frames: 0,
        })
    }

    /// Percentuale decodificata, da frame decodificati / `n_frames`. `None` se la durata non è nota.
    pub fn progress(&self) -> Option<u8> {
        self.total_frames
            .map(|total| (self.decoded_frames.saturating_mul(100) / total).min(100) as u8)
    }

    /// Il prossimo blocco decodificato, con i suoi canali e la sua frequenza. `None` a fine file.
    pub fn next_block(&mut self) -> Result<Option<Block>, AppError> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                // Le catene Ogg con flussi diversi non sono previste: finiscono qui.
                Ok(None) | Err(SymphoniaError::ResetRequired) => return Ok(None),
                // Senza durata nel container (es. WAV in streaming) la fine arriva come EOF.
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof
                        && self.total_frames.is_none() =>
                {
                    return Ok(None);
                }
                Err(SymphoniaError::IoError(e)) => {
                    return Err(AppError::UnreadableFile(e.to_string()));
                }
                Err(e) => return Err(AppError::UnsupportedCodec(e.to_string())),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                // Un pacchetto rovinato si salta, come nell'esempio ufficiale di Symphonia.
                Err(SymphoniaError::DecodeError(_) | SymphoniaError::IoError(_)) => continue,
                Err(e) => return Err(AppError::UnsupportedCodec(e.to_string())),
            };
            let channels = decoded.spec().channels().count().max(1);
            let rate = decoded.spec().rate();
            let mut samples = Vec::new();
            decoded.copy_to_vec_interleaved(&mut samples);
            self.decoded_frames += (samples.len() / channels) as u64;
            return Ok(Some(Block {
                samples,
                channels,
                rate,
            }));
        }
    }
}

/// Un blocco decodificato: campioni interleaved.
pub struct Block {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub rate: u32,
}

impl Block {
    /// I campioni mixati in mono.
    pub fn mono(&self) -> Vec<f32> {
        self.samples
            .chunks_exact(self.channels)
            .map(|frame| frame.iter().sum::<f32>() / self.channels as f32)
            .collect()
    }
}

/// La Forma d'onda di `path` (di un Bino, il mix): `count` picchi (0–1) dall'inizio alla fine,
/// ciascuno il massimo assoluto della sua parte di audio. Meno di `count` se l'audio dura meno di
/// `count` × 20 ms.
pub fn peaks(path: &Path, count: usize) -> Result<Vec<f32>, AppError> {
    let mut decoder = Decoder::open(path)?;
    let mut picchi = None;
    while let Some(block) = decoder.next_block()? {
        // ponytail: frequenza e canali fissati dal primo blocco, come per la Trascrizione; un
        // `Picchi::new` a ogni cambio se arrivano file con flussi che cambiano formato.
        picchi
            .get_or_insert_with(|| Picchi::new(block.rate, block.channels))
            .push(&block.samples);
    }
    Ok(picchi.map_or_else(Vec::new, |p| p.values(count)))
}

impl MediaSource for bino::Mix {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_forma_d_onda_ha_i_picchi_chiesti_e_segue_il_parlato() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parlato-it.wav");
        let peaks = peaks(&path, 120).unwrap();
        assert_eq!(peaks.len(), 120);
        assert!(peaks.iter().all(|p| (0.0..=1.0).contains(p)));
        // Il parlato c'è, e non dappertutto alla stessa altezza.
        let max = peaks.iter().copied().fold(0.0, f32::max);
        let min = peaks.iter().copied().fold(1.0, f32::min);
        assert!(max > 0.1 && min < max / 2.0, "{min}..{max}");
        // Un audio più corto dei picchi chiesti ne dà uno ogni 20 ms.
        let fine = super::peaks(&path, 1_000_000).unwrap();
        assert!(fine.len() > 120 && fine.len() < 1_000_000);
    }
}
