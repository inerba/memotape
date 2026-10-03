//! Writer Ogg/Opus (RFC 7845) per le Registrazioni: libopus via `opus`, pagine con `ogg`.

use std::io::Write;

use ogg::writing::{PacketWriteEndInfo, PacketWriter};
use opus::{Application, Bitrate, Channels, Encoder};

use crate::error::AppError;

/// Pacchetti da 20 ms: 960 campioni a 48 kHz, l'unità della granule position.
const PACKET_48K: u64 = 960;
/// Una pagina chiusa ogni 50 pacchetti, cioè ogni secondo.
const PACKETS_PER_PAGE: u64 = 50;
/// Il massimo consigliato da libopus per un pacchetto.
const MAX_PACKET: usize = 4000;

/// Scrive audio interleaved (mono o stereo, a 8/16/24/48 kHz) in un file Ogg/Opus, chiudendo una
/// pagina circa ogni secondo: anche durante la Registrazione il file è leggibile fin lì.
pub struct OggOpusWriter<W: Write> {
    pages: PacketWriter<'static, W>,
    encoder: Encoder,
    serial: u32,
    rate: u64,
    channels: usize,
    /// Campioni interleaved di un frame da 20 ms.
    frame_len: usize,
    /// Campioni in attesa di completare un frame.
    pending: Vec<f32>,
    /// Il ritardo dell'encoder, a 48 kHz.
    pre_skip: u64,
    /// Frame (per canale) ricevuti, alla frequenza dell'encoder.
    frames_in: u64,
    packets: u64,
    packet: Vec<u8>,
}

impl<W: Write> OggOpusWriter<W> {
    /// Scrive le intestazioni `OpusHead` e `OpusTags` in `out`.
    pub fn new(out: W, rate: u32, channels: usize, bitrate_kbps: u32) -> Result<Self, AppError> {
        let opus_channels = match channels {
            1 => Channels::Mono,
            2 => Channels::Stereo,
            n => return Err(AppError::Internal(format!("{n} canali non supportati"))),
        };
        let mut encoder =
            Encoder::new(rate, opus_channels, Application::Audio).map_err(internal)?;
        let bits = i32::try_from(bitrate_kbps * 1000).map_err(internal)?;
        encoder.set_bitrate(Bitrate::Bits(bits)).map_err(internal)?;
        // Il lookahead è alla frequenza dell'encoder; il pre-skip è sempre a 48 kHz.
        let lookahead =
            u64::try_from(encoder.get_lookahead().map_err(internal)?).map_err(internal)?;
        let pre_skip = lookahead * 48_000 / u64::from(rate);
        let mut writer = Self {
            pages: PacketWriter::new(out),
            encoder,
            serial: random_serial(),
            rate: u64::from(rate),
            channels,
            frame_len: rate as usize / 50 * channels,
            pending: Vec::new(),
            pre_skip,
            frames_in: 0,
            packets: 0,
            packet: vec![0; MAX_PACKET],
        };
        let mut head = b"OpusHead".to_vec();
        head.push(1); // versione
        head.push(channels as u8);
        head.extend_from_slice(&u16::try_from(pre_skip).map_err(internal)?.to_le_bytes());
        head.extend_from_slice(&rate.to_le_bytes());
        head.extend_from_slice(&0i16.to_le_bytes()); // output gain
        head.push(0); // mapping family 0: mono o stereo, senza tabella dei canali
        writer.page(head, PacketWriteEndInfo::EndPage, 0)?;
        let vendor = opus::version().as_bytes();
        let mut tags = b"OpusTags".to_vec();
        tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        tags.extend_from_slice(vendor);
        tags.extend_from_slice(&0u32.to_le_bytes()); // nessun commento
        writer.page(tags, PacketWriteEndInfo::EndPage, 0)?;
        Ok(writer)
    }

    /// Accoda campioni interleaved e codifica i frame da 20 ms completi.
    pub fn write(&mut self, samples: &[f32]) -> Result<(), AppError> {
        self.frames_in += (samples.len() / self.channels) as u64;
        self.pending.extend_from_slice(samples);
        let mut start = 0;
        while self.pending.len() - start >= self.frame_len {
            let frame = self.pending[start..start + self.frame_len].to_vec();
            self.encode(&frame, false)?;
            start += self.frame_len;
        }
        self.pending.drain(..start);
        Ok(())
    }

    /// Codifica il resto completato con silenzio, chiude il flusso (`EndStream`) con la granule
    /// della fine esatta dell'audio e restituisce il writer.
    pub fn finish(mut self) -> Result<W, AppError> {
        let end = self.pre_skip + (self.frames_in * 48_000).div_ceil(self.rate);
        // Si codifica finché i pacchetti coprono il ritardo dell'encoder più tutto l'audio.
        loop {
            let mut frame = std::mem::take(&mut self.pending);
            frame.resize(self.frame_len, 0.0);
            let last = (self.packets + 1) * PACKET_48K >= end;
            self.encode(&frame, last)?;
            if last {
                break;
            }
        }
        let mut out = self.pages.into_inner();
        out.flush().map_err(io)?;
        Ok(out)
    }

    fn encode(&mut self, frame: &[f32], last: bool) -> Result<(), AppError> {
        let len = self
            .encoder
            .encode_float(frame, &mut self.packet)
            .map_err(internal)?;
        self.packets += 1;
        let packet = self.packet[..len].to_vec();
        if last {
            let end = self.pre_skip + (self.frames_in * 48_000).div_ceil(self.rate);
            return self.page(packet, PacketWriteEndInfo::EndStream, end);
        }
        let info = if self.packets.is_multiple_of(PACKETS_PER_PAGE) {
            PacketWriteEndInfo::EndPage
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        self.page(packet, info, self.packets * PACKET_48K)
    }

    fn page(
        &mut self,
        packet: Vec<u8>,
        info: PacketWriteEndInfo,
        granule: u64,
    ) -> Result<(), AppError> {
        self.pages
            .write_packet(packet, self.serial, info, granule)
            .map_err(io)
    }
}

fn random_serial() -> u32 {
    use std::hash::{BuildHasher, RandomState};
    RandomState::new().hash_one(std::time::SystemTime::now()) as u32
}

fn internal(e: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("Opus: {e}"))
}

fn io(e: std::io::Error) -> AppError {
    AppError::UnwritableFolder(e.to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::audio_toolkit::decode::Decoder;

    /// Una sinusoide da 440 Hz interleaved, uguale su tutti i canali.
    pub fn sine(rate: u32, channels: usize, seconds: f64) -> Vec<f32> {
        let frames = (f64::from(rate) * seconds).round() as usize;
        (0..frames)
            .flat_map(|i| {
                let s = (i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 0.5;
                std::iter::repeat_n(s, channels)
            })
            .collect()
    }

    /// Durata in secondi del file riletto con la decodifica dell'app (Symphonia + libopus).
    pub fn decoded_seconds(path: &Path) -> f64 {
        let mut decoder = Decoder::open(path).unwrap();
        let mut seconds = 0.0;
        while let Some((mono, rate)) = decoder.next_mono().unwrap() {
            seconds += mono.len() as f64 / f64::from(rate);
        }
        seconds
    }

    /// I canali dichiarati nell'`OpusHead`, come li legge Symphonia.
    fn declared_channels(path: &Path) -> usize {
        use symphonia::core::formats::probe::Hint;
        use symphonia::core::formats::{FormatOptions, TrackType};
        use symphonia::core::io::MediaSourceStream;
        let file = std::fs::File::open(path).unwrap();
        let format = symphonia::default::get_probe()
            .probe(
                &Hint::new(),
                MediaSourceStream::new(Box::new(file), Default::default()),
                FormatOptions::default(),
                Default::default(),
            )
            .unwrap();
        let track = format.default_track(TrackType::Audio).unwrap();
        let params = track.codec_params.as_ref().unwrap().audio().unwrap();
        params.channels.as_ref().unwrap().count()
    }

    pub fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sbobino-test-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn il_file_rilegge_con_la_durata_scritta_per_ogni_frequenza_e_canale() {
        let dir = temp_dir("ogg-opus-durata");
        for rate in [8_000, 16_000, 24_000, 48_000] {
            for channels in [1, 2] {
                let path = dir.join(format!("{rate}-{channels}.ogg"));
                let file = std::fs::File::create(&path).unwrap();
                let mut writer = OggOpusWriter::new(file, rate, channels, 32).unwrap();
                // Blocchi di lunghezza qualsiasi, non multipli di 20 ms.
                for block in sine(rate, channels, 1.234).chunks(333 * channels) {
                    writer.write(block).unwrap();
                }
                writer.finish().unwrap();
                let seconds = decoded_seconds(&path);
                assert_eq!(declared_channels(&path), channels);
                assert!(
                    (seconds - 1.234).abs() < 0.001,
                    "{rate} Hz, {channels} canali: {seconds} s"
                );
            }
        }
    }

    #[test]
    fn durante_la_scrittura_il_file_e_gia_leggibile_fino_a_circa_un_secondo_prima() {
        let dir = temp_dir("ogg-opus-parziale");
        let path = dir.join("in-corso.ogg");
        let file = std::fs::File::create(&path).unwrap();
        let mut writer = OggOpusWriter::new(file, 48_000, 1, 32).unwrap();
        writer.write(&sine(48_000, 1, 3.0)).unwrap();
        // Senza `finish`, come dopo un crash: le pagine chiuse sono già sul disco.
        let seconds = decoded_seconds(&path);
        assert!((1.9..=3.0).contains(&seconds), "{seconds} s");
        drop(writer);
    }
}
