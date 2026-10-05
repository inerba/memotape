//! Registrazione dal microfono, dall'audio di sistema o da entrambi: catture, mixer e writer
//! Ogg/Opus in un thread, con Pausa e Stop comandati da flag atomici. L'Ogg si scrive mentre si
//! registra, in una cartella nascosta dentro la Cartella della Libreria; a Stop, finita la
//! Trascrizione dal vivo, diventa un Bino con il testo, che è la Sorgente.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, NaiveDateTime};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::capture::{self, Capture, Kind};
use crate::audio_toolkit::mixer::Mixer;
use crate::audio_toolkit::ogg_opus::OggOpusWriter;
use crate::bino;
use crate::engine::live::{self, LiveFeed};
use crate::error::AppError;
use crate::library::Library;
use crate::managers::activity::Activity;
use crate::managers::settings::{Channels, RecordingSource, Settings, SettingsStore};
use crate::managers::transcription::{self, LiveTranscription, TranscriptionProgress};
use crate::transcript::Ingresso;

/// Ogni quanto arriva `recording-tick`.
const TICK: Duration = Duration::from_millis(100);
/// La cartella nascosta degli Ogg delle Registrazioni in corso, dentro la Cartella della Libreria.
/// Dopo un crash l'audio è lì.
pub(crate) const TEMP_FOLDER: &str = ".sbobino";

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
    /// Il Bino; se non si è potuto scrivere, l'Ogg nella cartella nascosta.
    pub path: String,
    /// Perché la Registrazione si è fermata da sola (`deviceDisconnected`, `unwritableFolder`) o
    /// perché il Bino non si è scritto; `null` dopo Stop.
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
/// emettendo `recording-tick`. `prefix` è il prefisso tradotto del nome del file; il Bino va nella
/// Raccolta `raccolta` (la radice della Libreria per `None` o `""`).
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn record(
    app: AppHandle,
    activity: &Activity,
    recorder: &Recorder,
    prefix: String,
    raccolta: Option<String>,
) -> Result<RecordingSaved, AppError> {
    let folder = recordings_folder(&app)?;
    let destination = Library::raccolta_dir(&folder, raccolta.as_deref())?;
    let controls = Arc::new(Controls::default());
    let cancel = CancelToken::new();
    // La Raccolta non si rinomina né si elimina finché il Bino non è scritto.
    let _activity = activity.begin(Some(destination.clone()), {
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
    let separate = settings.ingressi_separati();
    // Con la Trascrizione dal vivo le pipeline girano in un loro thread e leggono l'uscita del
    // mixer: il mix, o con gli Ingressi separati ogni Ingresso, nell'ordine degli ingressi di `run`.
    let transcribed: &[Ingresso] = match (settings.trascrizione_dal_vivo, separate) {
        (false, _) => &[],
        (true, false) => &[Ingresso::Mix],
        (true, true) => &[Ingresso::Microfono, Ingresso::Sistema],
    };
    let (feeds, live) = if transcribed.is_empty() {
        (Vec::new(), None)
    } else {
        let (feeds, frames): (Vec<_>, Vec<_>) = live::channels(
            settings.sample_rate,
            channel_count(settings.channels),
            transcribed.len(),
        )?
        .into_iter()
        .unzip();
        let sources = transcribed.iter().copied().zip(frames).collect();
        let live = tauri::async_runtime::spawn_blocking({
            let (app, settings, cancel) = (app.clone(), settings.clone(), cancel.clone());
            let title = prefix.clone();
            move || transcription::transcribe_live(&app, sources, &settings, &title, &cancel)
        });
        (feeds, Some(live))
    };
    *recorder.current() = Some(Arc::clone(&controls));
    let recorded = tauri::async_runtime::spawn_blocking({
        let (app, controls, settings) = (app.clone(), Arc::clone(&controls), settings.clone());
        let (folder, prefix) = (folder.clone(), prefix.clone());
        move || run(&app, &folder, &prefix, &controls, &settings, feeds)
    })
    .await
    .map_err(internal)
    .and_then(|recorded| recorded);
    *recorder.current() = None;
    let recorded = match recorded {
        Ok(recorded) => recorded,
        Err(e) => {
            // La Registrazione non è partita: la pipeline si ferma da sola, senza aspettarla.
            cancel.cancel();
            return Err(e);
        }
    };
    let live = match live {
        Some(live) => {
            // Dopo Stop la status bar passa subito al completamento, anche col motore a metà Frase.
            if let Err(e) = (TranscriptionProgress { percent: None }).emit(&app) {
                log::warn!("transcription-progress non emesso: {e}");
            }
            // L'Attività finisce quando la coda è smaltita.
            Some(match live.await {
                Ok((transcript, transcribed)) => (Some(transcript), transcribed),
                Err(e) => (None, Err(internal(e))),
            })
        }
        None => None,
    };
    // Il Bino ha le Frasi arrivate anche se la Trascrizione è stata annullata o si è guastata.
    let phrases = live
        .as_ref()
        .and_then(|(transcript, _)| transcript.as_ref())
        .map_or(&[][..], |transcript| &transcript.phrases);
    let document = bino::Document::new(
        bino::creato(recorded.start),
        recorded.durata_ms,
        if separate {
            bino::Modalita::IngressiSeparati
        } else {
            bino::Modalita::Mix
        },
        live.is_some()
            .then(|| SettingsStore::model_of(&settings).id.clone()),
        settings.speech_language,
        completa(
            live.as_ref().map(|(_, transcribed)| transcribed),
            cancel.is_cancelled(),
        ),
        phrases,
    );
    let mut error = recorded.error.clone();
    let path =
        save_bino(&folder, &destination, &prefix, &recorded, &document).unwrap_or_else(|e| {
            error.get_or_insert(e);
            keep_ogg(&folder, &prefix, &recorded)
        });
    let transcription = live.map(|(transcript, transcribed)| match transcript {
        Some(transcript) => {
            transcription::finish_live(&app, &path, transcript, transcribed, &cancel)
        }
        None => LiveTranscription::Failed {
            error: transcribed.err().unwrap_or(AppError::Cancelled),
        },
    });
    Ok(RecordingSaved {
        path: path.display().to_string(),
        error,
        transcription,
    })
}

/// Se il testo del Bino è completo: la Trascrizione dal vivo c'era (`transcribed`), è arrivata alla
/// fine e non è stata annullata, nemmeno dopo l'ultima Frase.
fn completa(transcribed: Option<&Result<(), AppError>>, cancelled: bool) -> bool {
    matches!(transcribed, Some(Ok(()))) && !cancelled
}

/// Senza Bino, l'Ogg temporaneo del mix esce dalla cartella nascosta e va nella radice della
/// Libreria come `<prefisso> <data ora>.ogg`: è la Sorgente, con accanto il Markdown della
/// Trascrizione dal vivo, l'unico posto in cui resta il testo. Se nemmeno
/// questo riesce resta dov'è. Gli Ogg degli Ingressi restano nella cartella nascosta.
fn keep_ogg(folder: &Path, prefix: &str, recorded: &Recorded) -> PathBuf {
    let path = recording_path(
        folder,
        prefix,
        recorded.start.naive_local(),
        "ogg",
        Path::exists,
    );
    let mix = recorded.mix();
    match std::fs::rename(mix, &path) {
        Ok(()) => {
            log::warn!("Bino non scritto, la Registrazione è in {}", path.display());
            let _ = std::fs::remove_dir(folder.join(TEMP_FOLDER));
            path
        }
        Err(e) => {
            log::warn!(
                "Bino non scritto, la Registrazione resta in {}: {e}",
                mix.display()
            );
            mix.to_path_buf()
        }
    }
}

/// La Registrazione finita, ancora negli Ogg temporanei.
struct Recorded {
    /// Il mix e, con gli Ingressi separati, ogni Ingresso.
    oggs: Vec<(Ingresso, PathBuf)>,
    start: DateTime<Local>,
    durata_ms: u32,
    /// Perché si è fermata da sola, se non è stato Stop.
    error: Option<AppError>,
    /// La Forma d'onda del mix.
    forma_onda: Vec<f32>,
}

impl Recorded {
    fn mix(&self) -> &Path {
        &self.oggs[0].1
    }
}

/// Scrive il Bino `<prefisso> <data ora>.bino` nella cartella `destination` (una Raccolta della
/// Libreria `folder`, o la radice se nel frattempo è sparita) e cancella gli Ogg temporanei, e con
/// loro la cartella nascosta se resta vuota.
fn save_bino(
    folder: &Path,
    destination: &Path,
    prefix: &str,
    recorded: &Recorded,
    document: &bino::Document,
) -> Result<PathBuf, AppError> {
    let path = recording_path(
        if destination.is_dir() {
            destination
        } else {
            folder
        },
        prefix,
        recorded.start.naive_local(),
        "bino",
        Path::exists,
    );
    let audio: Vec<_> = recorded
        .oggs
        .iter()
        .map(|(ingresso, ogg)| (*ingresso, ogg.as_path()))
        .collect();
    bino::write(&path, &audio, document, Some(&recorded.forma_onda))?;
    for (_, ogg) in &recorded.oggs {
        if let Err(e) = std::fs::remove_file(ogg) {
            log::warn!("{} non cancellato: {e}", ogg.display());
        }
    }
    // Fallisce se ci sono altri Ogg (una Registrazione interrotta da un crash): restano lì.
    let _ = std::fs::remove_dir(folder.join(TEMP_FOLDER));
    Ok(path)
}

/// La cartella nascosta degli Ogg temporanei, creata se manca.
pub(crate) fn temp_folder(folder: &Path) -> Result<PathBuf, AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, SetFileAttributesW};
    let temp = folder.join(TEMP_FOLDER);
    std::fs::create_dir_all(&temp).map_err(|e| unwritable(&temp, &e))?;
    let wide: Vec<u16> = temp.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: `wide` è un percorso terminato da zero che vive per tutta la chiamata.
    if unsafe { SetFileAttributesW(wide.as_ptr(), FILE_ATTRIBUTE_HIDDEN) } == 0 {
        log::warn!(
            "{} non nascosta: {}",
            temp.display(),
            std::io::Error::last_os_error()
        );
    }
    Ok(temp)
}

pub(crate) fn channel_count(channels: Channels) -> usize {
    match channels {
        Channels::Mono => 1,
        Channels::Stereo => 2,
    }
}

/// La Cartella della Libreria: quella delle impostazioni, o `Documenti\Sbobino`.
pub fn recordings_folder(app: &AppHandle) -> Result<PathBuf, AppError> {
    match app.state::<SettingsStore>().get().recordings_folder {
        Some(folder) => Ok(PathBuf::from(folder)),
        None => default_recordings_folder(app),
    }
}

/// `Documenti\Sbobino`, la Cartella della Libreria se le impostazioni non ne indicano un'altra.
pub fn default_recordings_folder(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .document_dir()
        .map(|documents| documents.join(crate::library::DEFAULT_FOLDER))
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Un Ogg della Registrazione in corso: il mix o, con gli Ingressi separati, un Ingresso, con la sua
/// Trascrizione dal vivo.
struct Output {
    ingresso: Ingresso,
    path: PathBuf,
    writer: OggOpusWriter<File>,
    feed: Option<LiveFeed>,
}

/// Scrive in ogni Ogg il suo audio pronto (il mix da `mix`, gli Ingressi da `tracks`), lo manda alla
/// sua Trascrizione dal vivo e lo svuota.
fn write_ready(
    outputs: &mut [Output],
    mix: &mut Vec<f32>,
    tracks: &mut [Vec<f32>],
    paused: bool,
) -> Result<(), AppError> {
    for (output, audio) in outputs.iter_mut().zip(std::iter::once(mix).chain(tracks)) {
        output.writer.write(audio)?;
        if let Some(feed) = &mut output.feed {
            feed.push(audio, paused);
        }
        audio.clear();
    }
    Ok(())
}

/// Registra fino a Stop negli Ogg temporanei dentro `folder`: il mix e, con gli Ingressi separati,
/// ogni Ingresso. `feeds` sono le Trascrizioni dal vivo dell'audio salvato: una per il mix, una per
/// Ingresso con gli Ingressi separati, o nessuna.
fn run(
    app: &AppHandle,
    folder: &Path,
    prefix: &str,
    controls: &Controls,
    settings: &Settings,
    feeds: Vec<LiveFeed>,
) -> Result<Recorded, AppError> {
    let separate = settings.ingressi_separati();
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
    if separate {
        // Gli ingressi sono microfono e audio di sistema, in quest'ordine.
        mixer = mixer.with_tracks();
    }
    let temp = temp_folder(folder)?;
    let start = Local::now();
    let (path, file) = create_numbered(&temp, &base_name(prefix, start.naive_local()), "ogg")?;
    for capture in &captures {
        log::info!(
            "Registrazione da {} ({} Hz, {} canali) in {}",
            capture.name,
            capture.rate,
            capture.channels,
            path.display()
        );
    }
    let mut created = vec![path.clone()];
    let outputs = (|| {
        let mut files = vec![(Ingresso::Mix, path.clone(), file)];
        if separate {
            // Accanto al mix, con il suo nome: `<nome>.microfono.ogg`, `<nome>.sistema.ogg`.
            for ingresso in [Ingresso::Microfono, Ingresso::Sistema] {
                let track = path.with_extension(bino::audio_entry(ingresso));
                // Come il mix, non sovrascrive gli Ogg di una Registrazione interrotta da un crash.
                let file = File::create_new(&track).map_err(|e| unwritable(&track, &e))?;
                created.push(track.clone());
                files.push((ingresso, track, file));
            }
        }
        let mut feeds = feeds.into_iter();
        files
            .into_iter()
            .map(|(ingresso, path, file)| {
                let writer = OggOpusWriter::new(
                    file,
                    settings.sample_rate,
                    channels,
                    settings.bitrate_kbps,
                )?;
                // Si trascrive il mix, o con gli Ingressi separati ogni Ingresso.
                let transcribed = separate == (ingresso != Ingresso::Mix);
                Ok(Output {
                    ingresso,
                    path,
                    writer,
                    feed: if transcribed { feeds.next() } else { None },
                })
            })
            .collect::<Result<Vec<_>, AppError>>()
    })();
    let mut outputs = outputs.inspect_err(|_| {
        // Senza intestazioni i file sono vuoti: non restano nella cartella.
        for path in &created {
            let _ = std::fs::remove_file(path);
        }
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
        if let Err(e) = write_ready(&mut outputs, &mut out, mixer.tracks(), paused) {
            error = Some(e);
            break;
        }
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
    let durata_ms = mixer.elapsed_ms();
    let mut oggs = Vec::new();
    let mut forma_onda = Vec::new();
    for (output, audio) in outputs
        .into_iter()
        .zip(std::iter::once(&mut out).chain(mixer.tracks()))
    {
        let Output {
            ingresso,
            path,
            mut writer,
            feed,
        } = output;
        let written = writer.write(audio);
        // Anche se la scrittura non è riuscita: è quella dell'audio arrivato al writer.
        if ingresso == Ingresso::Mix {
            forma_onda = writer.forma_onda();
        }
        if let Err(e) = written.and_then(|()| writer.finish().map(drop)) {
            log::warn!("chiusura della Registrazione: {e}");
            error.get_or_insert(e);
        }
        if let Some(mut feed) = feed {
            feed.push(audio, false);
            feed.finish();
        }
        oggs.push((ingresso, path));
    }
    log::info!("Registrazione salvata: {durata_ms} ms");
    Ok(Recorded {
        oggs,
        start,
        durata_ms,
        error,
        forma_onda,
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

/// `<prefisso> AAAA-MM-GG HH-MM-SS.<extension>` in `folder`, con " 2", " 3"… se il nome esiste già.
pub fn recording_path(
    folder: &Path,
    prefix: &str,
    start: NaiveDateTime,
    extension: &str,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    numbered(folder, &base_name(prefix, start), extension, exists)
}

/// `<prefisso> AAAA-MM-GG HH-MM-SS`, il nome di una Registrazione senza estensione.
fn base_name(prefix: &str, start: NaiveDateTime) -> String {
    format!("{prefix} {}", start.format("%Y-%m-%d %H-%M-%S"))
}

/// `<base>.<extension>` in `folder`, con " 2", " 3"… se il nome esiste già.
pub fn numbered(
    folder: &Path,
    base: &str,
    extension: &str,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    std::iter::once(folder.join(format!("{base}.{extension}")))
        .chain((2..).map(|n| folder.join(format!("{base} {n}.{extension}"))))
        .find(|path| !exists(path))
        .expect("i numeri non finiscono")
}

/// Crea il file `numbered` in `folder`. `create_new`: un file comparso dopo il controllo non si
/// sovrascrive, si passa al numero successivo.
pub(crate) fn create_numbered(
    folder: &Path,
    base: &str,
    extension: &str,
) -> Result<(PathBuf, File), AppError> {
    loop {
        let path = numbered(folder, base, extension, Path::exists);
        match File::create_new(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(unwritable(&path, &e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn il_testo_del_bino_e_completo_solo_se_la_trascrizione_dal_vivo_e_finita() {
        let guasta = Err(AppError::Internal("x".into()));
        assert!(completa(Some(&Ok(())), false));
        assert!(!completa(Some(&Ok(())), true));
        assert!(!completa(Some(&guasta), false));
        assert!(!completa(Some(&Err(AppError::Cancelled)), true));
        // Senza Trascrizione dal vivo non c'è testo.
        assert!(!completa(None, false));
    }

    #[test]
    fn il_bino_va_nella_raccolta_o_nella_radice_se_la_raccolta_non_c_e_piu() {
        use crate::audio_toolkit::ogg_opus::tests::{sine, temp_dir};
        let folder = temp_dir("registrazione-raccolta");
        let raccolta = folder.join("Acme");
        std::fs::create_dir(&raccolta).unwrap();
        let start = DateTime::parse_from_rfc3339("2026-10-04T10:15:00+02:00")
            .unwrap()
            .with_timezone(&Local);
        let recorded = || {
            let ogg = temp_folder(&folder).unwrap().join("mix.ogg");
            let mut writer =
                OggOpusWriter::new(File::create(&ogg).unwrap(), 16_000, 1, 16).unwrap();
            writer.write(&sine(16_000, 1, 0.1)).unwrap();
            writer.finish().unwrap();
            Recorded {
                oggs: vec![(Ingresso::Mix, ogg)],
                start,
                durata_ms: 100,
                error: None,
                forma_onda: vec![0.5, 0.25],
            }
        };
        let document = bino::Document::new(
            bino::creato(start),
            100,
            bino::Modalita::Mix,
            None,
            crate::managers::settings::SpeechLanguage::Auto,
            false,
            &[],
        );
        let path = save_bino(&folder, &raccolta, "Registrazione", &recorded(), &document).unwrap();
        assert_eq!(
            path,
            raccolta.join(format!(
                "Registrazione {}.bino",
                start.format("%Y-%m-%d %H-%M-%S")
            ))
        );
        assert_eq!(bino::read(&path).unwrap(), document);
        assert_eq!(bino::forma_onda(&path), Some(vec![0.5, 0.25]));
        // L'Ogg temporaneo stava nella radice, e la cartella nascosta se ne va con lui.
        assert!(!folder.join(TEMP_FOLDER).exists());
        let sparita = folder.join("Sparita");
        let path = save_bino(&folder, &sparita, "Registrazione", &recorded(), &document).unwrap();
        assert_eq!(path.parent(), Some(folder.as_path()));
    }

    #[test]
    fn il_nome_ha_prefisso_data_e_ora_e_non_sovrascrive_mai() {
        let folder = Path::new(r"C:\Users\me\Documents\Sbobino");
        let start = at("2026-03-07 09:05:01");
        let taken =
            |names: &'static [&str]| move |p: &Path| names.iter().any(|n| p == folder.join(n));
        assert_eq!(
            recording_path(folder, "Registrazione", start, "bino", taken(&[])),
            folder.join("Registrazione 2026-03-07 09-05-01.bino")
        );
        assert_eq!(
            recording_path(
                folder,
                "Recording",
                start,
                "ogg",
                taken(&[
                    "Recording 2026-03-07 09-05-01.ogg",
                    "Recording 2026-03-07 09-05-01 2.ogg"
                ])
            ),
            folder.join("Recording 2026-03-07 09-05-01 3.ogg")
        );
    }
}
