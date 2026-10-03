//! Registrazione dal microfono, dall'audio di sistema o da entrambi: catture, mixer e writer
//! Ogg/Opus in un thread, con Pausa e Stop comandati da flag atomici. Il file si scrive mentre si
//! registra e a Stop diventa la Sorgente.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use chrono::NaiveDateTime;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::capture::{self, Capture, Kind};
use crate::audio_toolkit::mixer::Mixer;
use crate::audio_toolkit::ogg_opus::OggOpusWriter;
use crate::engine::live::{self, LiveFeed};
use crate::error::AppError;
use crate::managers::activity::Activity;
use crate::managers::settings::{Channels, RecordingSource, Settings, SettingsStore};
use crate::managers::transcription::{self, LiveTranscription, TranscriptionProgress};

/// Ogni quanto arriva `recording-tick`.
const TICK: Duration = Duration::from_millis(100);

/// Durata registrata (pause escluse) e livelli dall'evento precedente.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTick {
    pub elapsed_ms: u32,
    pub levels: Levels,
}

/// Il picco, tra 0 e 1, di ogni ingresso; `null` per quello che non si registra.
#[derive(Debug, Clone, Default, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Levels {
    pub microphone: Option<f32>,
    pub system: Option<f32>,
}

/// La Registrazione salvata, che diventa la Sorgente.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSaved {
    pub path: String,
    /// Perché la Registrazione si è fermata da sola (`deviceDisconnected`, `unwritableFolder`);
    /// `null` dopo Stop.
    pub error: Option<AppError>,
    /// L'esito della Trascrizione dal vivo; `null` se era spenta.
    pub transcription: Option<LiveTranscription>,
}

#[derive(Default)]
struct Controls {
    paused: AtomicBool,
    stop: AtomicBool,
}

/// Pausa e Stop della Registrazione in corso. In `tauri::State`.
#[derive(Default)]
pub struct Recorder {
    current: Mutex<Option<Arc<Controls>>>,
}

impl Recorder {
    /// Mette in pausa o riprende. Restituisce `false` se non c'è una Registrazione.
    pub fn set_paused(&self, paused: bool) -> bool {
        self.with(|c| c.paused.store(paused, Ordering::Relaxed))
    }

    /// Ferma e salva. Restituisce `false` se non c'è una Registrazione.
    pub fn stop(&self) -> bool {
        self.with(|c| c.stop.store(true, Ordering::Relaxed))
    }

    fn with(&self, f: impl FnOnce(&Controls)) -> bool {
        self.current().as_deref().map(f).is_some()
    }

    fn current(&self) -> MutexGuard<'_, Option<Arc<Controls>>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Registra dagli ingressi delle impostazioni finché arriva Stop o un dispositivo si scollega,
/// emettendo `recording-tick`. `prefix` è il prefisso tradotto del nome del file.
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn record(
    app: AppHandle,
    activity: &Activity,
    recorder: &Recorder,
    prefix: String,
) -> Result<RecordingSaved, AppError> {
    let controls = Arc::new(Controls::default());
    let cancel = CancelToken::new();
    let _activity = activity.begin({
        let controls = Arc::clone(&controls);
        let cancel = cancel.clone();
        // Stop durante la Registrazione, Annulla durante lo smaltimento della coda dal vivo.
        move || {
            controls.stop.store(true, Ordering::Relaxed);
            cancel.cancel();
        }
    })?;
    let internal = |e: tauri::Error| AppError::Internal(e.to_string());
    let settings = app.state::<SettingsStore>().get();
    // Con la Trascrizione dal vivo, la pipeline gira in un suo thread e legge l'uscita del mixer.
    let (feed, live) = if settings.trascrizione_dal_vivo {
        let (feed, frames) = live::channel(settings.sample_rate, channel_count(settings.channels))?;
        let live = tauri::async_runtime::spawn_blocking({
            let (app, settings, cancel) = (app.clone(), settings.clone(), cancel.clone());
            let title = prefix.clone();
            move || transcription::transcribe_live(&app, frames, &settings, &title, &cancel)
        });
        (Some(feed), Some(live))
    } else {
        (None, None)
    };
    *recorder.current() = Some(Arc::clone(&controls));
    let recorded = tauri::async_runtime::spawn_blocking({
        let (app, controls) = (app.clone(), Arc::clone(&controls));
        move || run(&app, &prefix, &controls, &settings, feed)
    })
    .await
    .map_err(internal)
    .and_then(|recorded| recorded);
    *recorder.current() = None;
    let Some(live) = live else {
        return recorded;
    };
    let mut saved = match recorded {
        Ok(saved) => saved,
        Err(e) => {
            // La Registrazione non è partita: la pipeline si ferma da sola, senza aspettarla.
            cancel.cancel();
            return Err(e);
        }
    };
    // Dopo Stop la status bar passa subito al completamento, anche col motore a metà Frase.
    if let Err(e) = (TranscriptionProgress { percent: None }).emit(&app) {
        log::warn!("transcription-progress non emesso: {e}");
    }
    // L'Attività finisce quando la coda è smaltita.
    let transcript = live
        .await
        .map_err(internal)
        .and_then(|transcript| transcript);
    saved.transcription = Some(transcription::finish_live(
        &app,
        Path::new(&saved.path),
        transcript,
        &cancel,
    ));
    Ok(saved)
}

fn channel_count(channels: Channels) -> usize {
    match channels {
        Channels::Mono => 1,
        Channels::Stereo => 2,
    }
}

/// La Cartella predefinita: quella delle impostazioni, o `Documenti\Sbobino`.
pub fn recordings_folder(app: &AppHandle) -> Result<PathBuf, AppError> {
    match app.state::<SettingsStore>().get().recordings_folder {
        Some(folder) => Ok(PathBuf::from(folder)),
        None => default_recordings_folder(app),
    }
}

/// `Documenti\Sbobino`, la Cartella predefinita se le impostazioni non ne indicano un'altra.
pub fn default_recordings_folder(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .document_dir()
        .map(|documents| documents.join("Sbobino"))
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Registra fino a Stop. Con `feed` l'audio salvato va anche alla Trascrizione dal vivo.
fn run(
    app: &AppHandle,
    prefix: &str,
    controls: &Controls,
    settings: &Settings,
    mut feed: Option<LiveFeed>,
) -> Result<RecordingSaved, AppError> {
    let channels = channel_count(settings.channels);
    let kinds: &[Kind] = match settings.recording_source {
        RecordingSource::Mic => &[Kind::Microphone],
        RecordingSource::System => &[Kind::System],
        RecordingSource::Both => &[Kind::Microphone, Kind::System],
    };
    let (blocks_tx, blocks) = capture::channel(kinds.len());
    let captures = kinds
        .iter()
        .enumerate()
        .map(|(input, &kind)| {
            let id = match kind {
                Kind::Microphone => settings.microphone.as_deref(),
                Kind::System => settings.output_device.as_deref(),
            };
            Capture::open(kind, id, input, blocks_tx.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Restano solo i mittenti delle callback.
    drop(blocks_tx);
    // Tutte le catture hanno lo stesso orologio (QPC).
    let now = || captures[0].now();
    let formats: Vec<_> = captures.iter().map(|c| (c.rate, c.channels)).collect();
    let mut mixer = Mixer::new(now(), &formats, (settings.sample_rate, channels))?;
    let folder = recordings_folder(app)?;
    std::fs::create_dir_all(&folder).map_err(|e| unwritable(&folder, &e))?;
    let start = chrono::Local::now().naive_local();
    let (path, file) = loop {
        let path = recording_path(&folder, prefix, start, Path::exists);
        // `create_new`: un file comparso dopo il controllo non si sovrascrive.
        match std::fs::File::create_new(&path) {
            Ok(file) => break (path, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(unwritable(&path, &e)),
        }
    };
    for capture in &captures {
        log::info!(
            "Registrazione da {} ({} Hz, {} canali) in {}",
            capture.name,
            capture.rate,
            capture.channels,
            path.display()
        );
    }
    let mut writer =
        OggOpusWriter::new(file, settings.sample_rate, channels, settings.bitrate_kbps)
            .inspect_err(|_| {
                // Senza intestazioni il file è vuoto: non resta nella cartella.
                let _ = std::fs::remove_file(&path);
            })?;
    let mut out = Vec::new();
    let mut last_tick = Instant::now();
    let mut error = None;
    while !controls.stop.load(Ordering::Relaxed) {
        if let Some((capture, e)) = captures
            .iter()
            .find_map(|c| device_error(c).map(|e| (c, e)))
        {
            log::warn!("Registrazione fermata, {}: {e}", capture.name);
            error = Some(AppError::DeviceDisconnected(capture.name.clone()));
            break;
        }
        // Anche senza blocchi (il loopback a riproduzione ferma) l'orologio avanza ogni `TICK`.
        let received = blocks.recv_timeout(TICK);
        let paused = controls.paused.load(Ordering::Relaxed);
        mixer.advance(now(), paused, &mut out);
        match received {
            Ok(block) => {
                mixer.push(block.input, block.capture_ns, &block.samples, &mut out);
                captures[block.input].recycle(block.samples);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                error = Some(AppError::DeviceDisconnected(captures[0].name.clone()));
                break;
            }
        }
        // Un errore di scrittura (disco pieno) ferma come Stop: quanto è già sul disco diventa
        // comunque la Sorgente.
        if let Err(e) = writer.write(&out) {
            error = Some(e);
            break;
        }
        if let Some(feed) = &mut feed {
            feed.push(&out, paused);
        }
        out.clear();
        if last_tick.elapsed() >= TICK {
            last_tick = Instant::now();
            let mut levels = Levels::default();
            for (&kind, peak) in kinds.iter().zip(mixer.take_peaks()) {
                match kind {
                    Kind::Microphone => levels.microphone = Some(peak),
                    Kind::System => levels.system = Some(peak),
                }
            }
            let tick = RecordingTick {
                elapsed_ms: mixer.elapsed_ms(),
                levels,
            };
            if let Err(e) = tick.emit(app) {
                log::warn!("recording-tick non emesso: {e}");
            }
        }
    }
    // Quanto è stato catturato prima di Stop fa parte della Registrazione.
    let stop = now();
    for block in blocks.try_iter().filter(|b| b.capture_ns < stop) {
        mixer.push(block.input, block.capture_ns, &block.samples, &mut out);
    }
    drop(captures);
    mixer.finish(stop, &mut out);
    if let Err(e) = writer.write(&out).and_then(|()| writer.finish().map(drop)) {
        log::warn!("chiusura della Registrazione: {e}");
        error.get_or_insert(e);
    }
    log::info!("Registrazione salvata: {} ms", mixer.elapsed_ms());
    if let Some(mut feed) = feed {
        feed.push(&out, false);
        feed.finish();
    }
    Ok(RecordingSaved {
        path: path.display().to_string(),
        error,
        transcription: None,
    })
}

/// Il primo errore del dispositivo che ferma la cattura. Le discontinuità (`Xrun`) no: il mixer
/// riempie il buco di silenzio dai timestamp.
fn device_error(capture: &Capture) -> Option<cpal::Error> {
    capture.errors.try_iter().find(|e| {
        let ignore = matches!(
            e.kind(),
            cpal::ErrorKind::Xrun
                | cpal::ErrorKind::RealtimeDenied
                | cpal::ErrorKind::DeviceChanged
        );
        if ignore {
            log::warn!("{}: {e}", capture.name);
        }
        !ignore
    })
}

fn unwritable(path: &Path, e: &std::io::Error) -> AppError {
    AppError::UnwritableFolder(format!("{}: {e}", path.display()))
}

/// `<prefisso> AAAA-MM-GG HH-MM-SS.ogg` nella Cartella predefinita, con " 2", " 3"… se il nome
/// esiste già.
pub fn recording_path(
    folder: &Path,
    prefix: &str,
    start: NaiveDateTime,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    let base = format!("{prefix} {}", start.format("%Y-%m-%d %H-%M-%S"));
    std::iter::once(folder.join(format!("{base}.ogg")))
        .chain((2..).map(|n| folder.join(format!("{base} {n}.ogg"))))
        .find(|path| !exists(path))
        .expect("i numeri non finiscono")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn il_nome_ha_prefisso_data_e_ora_e_non_sovrascrive_mai() {
        let folder = Path::new(r"C:\Users\me\Documents\Sbobino");
        let start = at("2026-03-07 09:05:01");
        let taken =
            |names: &'static [&str]| move |p: &Path| names.iter().any(|n| p == folder.join(n));
        assert_eq!(
            recording_path(folder, "Registrazione", start, taken(&[])),
            folder.join("Registrazione 2026-03-07 09-05-01.ogg")
        );
        assert_eq!(
            recording_path(
                folder,
                "Recording",
                start,
                taken(&[
                    "Recording 2026-03-07 09-05-01.ogg",
                    "Recording 2026-03-07 09-05-01 2.ogg"
                ])
            ),
            folder.join("Recording 2026-03-07 09-05-01 3.ogg")
        );
    }
}
