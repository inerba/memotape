//! Cattura con cpal (WASAPI) dal microfono o dall'audio di sistema (loopback: uno stream di input
//! sul dispositivo di uscita). La callback non alloca: copia i campioni in un buffer preso da un
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

/// Un microfono o un dispositivo di uscita, per la scelta in Impostazioni.
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
    /// L'ingresso che l'ha catturato, come indicato a `Capture::open`.
    pub input: usize,
    /// Timestamp di cattura in ns (QPC su WASAPI).
    pub capture_ns: u64,
    /// Campioni interleaved del dispositivo, convertiti in f32.
    pub samples: Vec<f32>,
}

/// I microfoni rilevati.
pub fn microphones() -> Result<Vec<AudioDevice>, AppError> {
    let host = cpal::default_host();
    let default = host.default_input_device();
    devices(host.input_devices().map_err(cpal_error)?, default)
}

/// I dispositivi di uscita rilevati, per l'audio di sistema.
pub fn output_devices() -> Result<Vec<AudioDevice>, AppError> {
    let host = cpal::default_host();
    let default = host.default_output_device();
    devices(host.output_devices().map_err(cpal_error)?, default)
}

fn devices(
    devices: impl Iterator<Item = cpal::Device>,
    default: Option<cpal::Device>,
) -> Result<Vec<AudioDevice>, AppError> {
    let default = default.and_then(|d| d.id().ok());
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

/// Il canale dei blocchi di `inputs` catture: ha posto per tutti i loro buffer, quindi la
/// callback non trova mai il canale pieno.
pub fn channel(inputs: usize) -> (SyncSender<Block>, Receiver<Block>) {
    sync_channel(POOL * inputs)
}

/// Quale dispositivo catturare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Microphone,
    /// Il loopback del dispositivo di uscita.
    System,
}

/// Una cattura in corso: si ferma al drop.
pub struct Capture {
    stream: cpal::Stream,
    /// Gli errori del dispositivo (scollegato, discontinuità…).
    pub errors: Receiver<cpal::Error>,
    recycle: SyncSender<Vec<f32>>,
    pub rate: u32,
    pub channels: usize,
    /// Il nome del dispositivo, per l'errore "dispositivo scollegato".
    pub name: String,
}

impl Capture {
    /// Apre il dispositivo con l'id dato, o quello predefinito, alla sua frequenza nativa, e manda
    /// i blocchi in `blocks` con `input`. Senza dispositivi, o se quello scelto non è collegato,
    /// dà `MicrophoneMissing` o `OutputDeviceMissing`.
    pub fn open(
        kind: Kind,
        id: Option<&str>,
        input: usize,
        blocks: SyncSender<Block>,
    ) -> Result<Self, AppError> {
        let host = cpal::default_host();
        let missing = match kind {
            Kind::Microphone => AppError::MicrophoneMissing,
            Kind::System => AppError::OutputDeviceMissing,
        };
        let device = match (id, kind) {
            (Some(id), _) => cpal::DeviceId::from_str(id)
                .ok()
                .and_then(|id| host.device_by_id(&id)),
            (None, Kind::Microphone) => host.default_input_device(),
            (None, Kind::System) => host.default_output_device(),
        }
        .ok_or(missing)?;
        let name = device.to_string();
        // Il loopback è uno stream di input sul dispositivo di uscita, nel formato del suo mix:
        // lì `default_input_config` dà errore.
        let config = match kind {
            Kind::Microphone => device.default_input_config(),
            Kind::System => device.default_output_config(),
        }
        .map_err(cpal_error)?;
        let rate = config.sample_rate();
        let channels = usize::from(config.channels());
        let capacity = (rate as f32 * BUFFER_SECONDS) as usize * channels;
        let (recycle, pool) = sync_channel(POOL);
        for _ in 0..POOL {
            recycle
                .send(Vec::with_capacity(capacity))
                .expect("il pool ha spazio per tutti i buffer");
        }
        let (errors_tx, errors) = std::sync::mpsc::channel();
        let format = config.sample_format();
        let config = config.config();
        let stream = match format {
            SampleFormat::F32 => build::<f32>(&device, config, input, pool, blocks, errors_tx),
            SampleFormat::I16 => build::<i16>(&device, config, input, pool, blocks, errors_tx),
            SampleFormat::I32 => build::<i32>(&device, config, input, pool, blocks, errors_tx),
            other => {
                return Err(AppError::Internal(format!(
                    "formato audio non supportato da {name}: {other}"
                )));
            }
        }
        .map_err(cpal_error)?;
        stream.play().map_err(cpal_error)?;
        Ok(Self {
            stream,
            errors,
            recycle,
            rate,
            channels,
            name,
        })
    }

    /// L'istante corrente sullo stesso orologio dei timestamp di cattura (QPC su WASAPI), in ns.
    pub fn now(&self) -> u64 {
        u64::try_from(self.stream.now().as_nanos()).unwrap_or(0)
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
    input: usize,
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
                input,
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
