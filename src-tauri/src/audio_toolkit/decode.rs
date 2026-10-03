//! Decodifica dei file con Symphonia (+ libopus per Opus) in blocchi di campioni mono.

use std::path::Path;
use std::sync::LazyLock;

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, TrackType};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;

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
    interleaved: Vec<f32>,
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
            interleaved: Vec::new(),
            total_frames,
            decoded_frames: 0,
        })
    }

    /// Percentuale decodificata, da frame decodificati / `n_frames`. `None` se la durata non è nota.
    pub fn progress(&self) -> Option<u8> {
        self.total_frames
            .map(|total| (self.decoded_frames.saturating_mul(100) / total).min(100) as u8)
    }

    /// Il prossimo blocco decodificato, mixato in mono, con la sua frequenza. `None` a fine file.
    pub fn next_mono(&mut self) -> Result<Option<(Vec<f32>, u32)>, AppError> {
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
            decoded.copy_to_vec_interleaved(&mut self.interleaved);
            let mono: Vec<f32> = self
                .interleaved
                .chunks_exact(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                .collect();
            self.decoded_frames += mono.len() as u64;
            return Ok(Some((mono, rate)));
        }
    }
}

impl MediaSource for bino::Mix {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.len())
    }
}
