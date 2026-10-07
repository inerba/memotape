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
    #[cfg(test)]
    pub metrics: std::sync::Arc<CaptureMetrics>,
}

/// Misura del pool nello smoke nativo; nessuna inferenza o attesa nella callback.
#[cfg(test)]
#[derive(Default)]
pub struct CaptureMetrics {
    pub lost_frames: std::sync::atomic::AtomicU64,
    pub in_flight: std::sync::atomic::AtomicUsize,
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
        #[cfg(test)]
        let metrics = std::sync::Arc::new(CaptureMetrics::default());
        let stream = match format {
            SampleFormat::F32 => build::<f32>(
                &device,
                config,
                input,
                pool,
                blocks,
                errors_tx,
                #[cfg(test)]
                metrics.clone(),
            ),
            SampleFormat::I16 => build::<i16>(
                &device,
                config,
                input,
                pool,
                blocks,
                errors_tx,
                #[cfg(test)]
                metrics.clone(),
            ),
            SampleFormat::I32 => build::<i32>(
                &device,
                config,
                input,
                pool,
                blocks,
                errors_tx,
                #[cfg(test)]
                metrics.clone(),
            ),
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
            #[cfg(test)]
            metrics,
        })
    }

    /// L'istante corrente sullo stesso orologio dei timestamp di cattura (QPC su WASAPI), in ns.
    pub fn now(&self) -> u64 {
        u64::try_from(self.stream.now().as_nanos()).unwrap_or(0)
    }

    /// Restituisce al pool il buffer di un blocco già elaborato.
    pub fn recycle(&self, samples: Vec<f32>) {
        #[cfg(test)]
        self.metrics
            .in_flight
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
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
    #[cfg(test)] metrics: std::sync::Arc<CaptureMetrics>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    #[cfg(test)]
    let channels = usize::from(config.channels);
    device.build_input_stream::<T, _, _>(
        config,
        move |data: &[T], info: &cpal::InputCallbackInfo| {
            // Niente allocazioni, lock o log: senza buffer libero il blocco si perde e il mixer
            // lo riempie di silenzio dai timestamp.
            let Ok(mut buffer) = pool.try_recv() else {
                #[cfg(test)]
                metrics.lost_frames.fetch_add(
                    (data.len() / channels) as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                return;
            };
            buffer.clear();
            let n = data.len().min(buffer.capacity());
            #[cfg(test)]
            {
                metrics.lost_frames.fetch_add(
                    ((data.len() - n) / channels) as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                metrics
                    .in_flight
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
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

#[cfg(all(test, windows))]
#[test]
#[ignore = "WASAPI reale: riproduce la fixture e verifica il segnale nel mixer"]
fn cattura_nativa_non_diventa_silenzio_nel_mixer() {
    use crate::audio_toolkit::mixer::Mixer;
    use std::os::windows::process::CommandExt;
    use std::time::{Duration, Instant};
    let cleaning = std::env::var_os("MEMOTAPE_CAPTURE_CLEANING").is_some();
    let mut processors: Vec<Box<dyn crate::audio_toolkit::processing::AudioProcessor>> = Vec::new();
    if cleaning {
        let model = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(crate::audio_toolkit::deepfilter::MODEL_FILE);
        for ingresso in [
            crate::transcript::Ingresso::Microfono,
            crate::transcript::Ingresso::Sistema,
        ] {
            let mut processor = crate::audio_toolkit::cleaning::ConfiguredCleaning::new(
                model.clone(),
                || true,
                ingresso,
                crate::audio_toolkit::cleaning::CleaningLog::default(),
            )
            .recovering(|error| panic!("pulizia fallita: {error}"));
            processor
                .prepare(crate::audio_toolkit::processing::Format {
                    rate: 16_000,
                    channels: 1,
                })
                .unwrap();
            processors.push(Box::new(processor));
        }
    }
    let (sender, blocks) = channel(2);
    let captures = [
        Capture::open(Kind::Microphone, None, 0, sender.clone()).unwrap(),
        Capture::open(Kind::System, None, 1, sender).unwrap(),
    ];
    let origin = captures[0].now();
    let formats: Vec<_> = captures.iter().map(|c| (c.rate, c.channels)).collect();
    let mut mixer = if cleaning {
        Mixer::with_processors(origin, &formats, (16_000, 1), processors).unwrap()
    } else {
        Mixer::new(origin, &formats, (16_000, 1)).unwrap()
    };
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parlato-it.wav");
    let mut playback = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command"])
        .arg(format!(
            "(New-Object System.Media.SoundPlayer '{}').PlaySync()",
            fixture.display()
        ))
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();
    let started = Instant::now();
    let mut out = Vec::new();
    let mut raw_peak = [0.0_f32; 2];
    let mut counts = [0_usize; 2];
    let mut first = [None; 2];
    while started.elapsed() < Duration::from_secs(3) {
        let received = blocks.recv_timeout(Duration::from_millis(100));
        mixer.advance(captures[0].now(), false, &mut out).unwrap();
        if let Ok(block) = received {
            counts[block.input] += 1;
            first[block.input].get_or_insert(block.capture_ns);
            for &sample in &block.samples {
                raw_peak[block.input] = raw_peak[block.input].max(sample.abs());
            }
            mixer
                .push(block.input, block.capture_ns, &block.samples, &mut out)
                .unwrap();
            captures[block.input].recycle(block.samples);
        }
    }
    mixer.finish(captures[0].now(), &mut out).unwrap();
    let _ = playback.kill();
    let _ = playback.wait();
    let peak = out.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
    eprintln!(
        "origin={origin}, first={first:?}, blocchi={counts:?}, picchi grezzi={raw_peak:?}, mix={peak}, durata={}ms",
        mixer.elapsed_ms()
    );
    assert!(
        raw_peak[1] > 0.001,
        "la fixture deve raggiungere il loopback"
    );
    assert!(
        peak > 0.001,
        "la cattura non deve diventare silenzio nel mixer"
    );
}
