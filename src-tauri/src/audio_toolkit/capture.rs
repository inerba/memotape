//! Cattura con cpal (WASAPI). La callback non alloca: copia i campioni in un buffer preso da un
//! pool e lo passa al worker con il timestamp di cattura (QPC). Non testata: richiede hardware.

use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use crate::error::AppError;

/// Buffer nel pool: a 10 ms per callback bastano per oltre 1 s di ritardo del worker.
const POOL: usize = 128;
/// Capacità di un buffer, in secondi di audio: ben oltre un periodo WASAPI (10 ms).
const BUFFER_SECONDS: f32 = 0.25;

/// Un dispositivo di ingresso, per la scelta in Impostazioni.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    /// L'id stabile di cpal (`wasapi:{…}`), salvato nelle impostazioni.
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Un blocco catturato.
pub struct Block {
    /// Timestamp di cattura in ns (QPC su WASAPI).
    pub capture_ns: u64,
    /// Campioni interleaved del dispositivo, convertiti in f32.
    pub samples: Vec<f32>,
}

/// I microfoni rilevati.
pub fn microphones() -> Result<Vec<AudioDevice>, AppError> {
    let host = cpal::default_host();
    let default = host.default_input_device().and_then(|d| d.id().ok());
    let devices = host.input_devices().map_err(cpal_error)?;
    Ok(devices
        .filter_map(|device| {
            let id = device.id().ok()?;
            Some(AudioDevice {
                is_default: Some(&id) == default.as_ref(),
                id: id.to_string(),
                name: device.to_string(),
            })
        })
        .collect())
}

/// Una cattura in corso: si ferma al drop.
pub struct Capture {
    _stream: cpal::Stream,
    /// I blocchi catturati, in ordine.
    pub blocks: Receiver<Block>,
    /// Gli errori del dispositivo (scollegato, discontinuità…).
    pub errors: Receiver<cpal::Error>,
    recycle: SyncSender<Vec<f32>>,
    pub rate: u32,
    pub channels: usize,
    /// Il nome del dispositivo, per l'errore "dispositivo scollegato".
    pub name: String,
}

impl Capture {
    /// Apre il microfono con l'id dato, o quello predefinito, alla sua frequenza nativa.
    /// Senza microfoni, o se quello scelto non è collegato, dà `MicrophoneMissing`.
    pub fn microphone(id: Option<&str>) -> Result<Self, AppError> {
        let host = cpal::default_host();
        let device = match id {
            Some(id) => cpal::DeviceId::from_str(id)
                .ok()
                .and_then(|id| host.device_by_id(&id))
                .ok_or(AppError::MicrophoneMissing)?,
            None => host
                .default_input_device()
                .ok_or(AppError::MicrophoneMissing)?,
        };
        let name = device.to_string();
        let config = device.default_input_config().map_err(cpal_error)?;
        let rate = config.sample_rate();
        let channels = usize::from(config.channels());
        let capacity = (rate as f32 * BUFFER_SECONDS) as usize * channels;
        let (recycle, pool) = sync_channel(POOL);
        for _ in 0..POOL {
            recycle
                .send(Vec::with_capacity(capacity))
                .expect("il pool ha spazio per tutti i buffer");
        }
        let (blocks_tx, blocks) = sync_channel(POOL);
        let (errors_tx, errors) = std::sync::mpsc::channel();
        let format = config.sample_format();
        let config = config.config();
        let stream = match format {
            SampleFormat::F32 => build::<f32>(&device, config, pool, blocks_tx, errors_tx),
            SampleFormat::I16 => build::<i16>(&device, config, pool, blocks_tx, errors_tx),
            SampleFormat::I32 => build::<i32>(&device, config, pool, blocks_tx, errors_tx),
            other => {
                return Err(AppError::Internal(format!(
                    "formato del microfono non supportato: {other}"
                )));
            }
        }
        .map_err(cpal_error)?;
        stream.play().map_err(cpal_error)?;
        Ok(Self {
            _stream: stream,
            blocks,
            errors,
            recycle,
            rate,
            channels,
            name,
        })
    }

    /// Restituisce al pool il buffer di un blocco già elaborato.
    pub fn recycle(&self, samples: Vec<f32>) {
        // Il pool contiene al massimo i suoi buffer: c'è sempre posto.
        let _ = self.recycle.try_send(samples);
    }
}

fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    pool: Receiver<Vec<f32>>,
    blocks: SyncSender<Block>,
    errors: std::sync::mpsc::Sender<cpal::Error>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream::<T, _, _>(
        config,
        move |data: &[T], info: &cpal::InputCallbackInfo| {
            // Niente allocazioni, lock o log: senza buffer libero il blocco si perde e il mixer
            // lo riempie di silenzio dai timestamp.
            let Ok(mut buffer) = pool.try_recv() else {
                return;
            };
            buffer.clear();
            let n = data.len().min(buffer.capacity());
            buffer.extend(data[..n].iter().map(|s| s.to_sample::<f32>()));
            let capture_ns = u64::try_from(info.timestamp().capture.as_nanos()).unwrap_or(0);
            // Il canale ha posto per tutti i buffer del pool: l'invio non fallisce.
            let _ = blocks.try_send(Block {
                capture_ns,
                samples: buffer,
            });
        },
        move |error| {
            let _ = errors.send(error);
        },
        None,
    )
}

fn cpal_error(e: cpal::Error) -> AppError {
    AppError::Internal(format!("audio: {e}"))
}
