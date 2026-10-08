//! Registrazione dal microfono, dall'audio di sistema o da entrambi: catture, mixer e writer
//! Ogg/Opus in un thread, con Pausa e Stop comandati da flag atomici. L'Ogg si scrive mentre si
//! registra, in una cartella nascosta dentro la Cartella della Libreria; a Stop, finita la
//! Trascrizione dal vivo, diventa un Tape con il testo, che è la Sorgente.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, NaiveDateTime};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::capture::{self, Capture, Kind};
use crate::audio_toolkit::cleaning::{CleaningLog, ConfiguredCleaning, TrattoPulizia};
#[cfg(test)]
use crate::audio_toolkit::deepfilter;
use crate::audio_toolkit::mixer::Mixer;
use crate::audio_toolkit::ogg_opus::OggOpusWriter;
use crate::audio_toolkit::processing::AudioProcessor;
#[cfg(test)]
use crate::audio_toolkit::processing::Format;
use crate::audio_toolkit::protection::{ProtectionTimeline, Sensibilita};
use crate::engine::live::{self, LiveFeed};
use crate::error::AppError;
use crate::library::Library;
use crate::managers::activity::Activity;
use crate::managers::models::Models;
use crate::managers::settings::{
    Channels, RecordingSource, Settings, SettingsStore, guadagno_factor,
};
use crate::managers::transcription::{
    self, DiarizationStarted, LiveTranscription, TranscriptionProgress,
};
use crate::tape;
use crate::transcript::Ingresso;

mod preparation;
pub(crate) use preparation::{RecordingFilters, preload};

#[derive(Debug, Clone, Copy, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RecordingPhase {
    Preparing,
    Cleaning,
    Devices,
    Recording,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct RecordingPhaseChanged {
    pub session_id: String,
    pub phase: RecordingPhase,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct RecordingCleaningPreparing {
    pub session_id: String,
    pub ingresso: Ingresso,
    pub preparing: bool,
}

fn emit_phase(app: &AppHandle, session_id: &str, phase: RecordingPhase) {
    if let Err(error) = (RecordingPhaseChanged {
        session_id: session_id.into(),
        phase,
    })
    .emit(app)
    {
        log::warn!("fase Registrazione non emessa: {error}");
    }
}

fn emit_cleaning_preparing(app: &AppHandle, session_id: &str, ingresso: Ingresso, preparing: bool) {
    if let Err(error) = (RecordingCleaningPreparing {
        session_id: session_id.into(),
        ingresso,
        preparing,
    })
    .emit(app)
    {
        log::warn!("preparazione pulizia non emessa: {error}");
    }
}

fn input_kinds(settings: &Settings) -> &'static [Kind] {
    match settings.recording_source {
        RecordingSource::Mic => &[Kind::Microphone],
        RecordingSource::System => &[Kind::System],
        RecordingSource::Both => &[Kind::Microphone, Kind::System],
    }
}

/// Il passo massimo dell'orologio del worker, anche senza blocchi (il loopback fermo).
const TICK: Duration = Duration::from_millis(100);
/// Ogni quanto arriva `recording-tick` con timer e livelli: abbastanza spesso da seguire il suono.
const LEVELS_TICK: Duration = Duration::from_millis(40);

/// Guasto recuperato: la cattura continua in bypass soltanto su questo Ingresso.
#[derive(Debug, Clone, serde::Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct RecordingCleaningFailed {
    pub session_id: String,
    pub ingresso: Ingresso,
    pub error: AppError,
}
/// La cartella nascosta degli Ogg delle Registrazioni in corso, dentro la Cartella della Libreria.
/// Dopo un crash l'audio è lì.
pub(crate) const TEMP_FOLDER: &str = ".memotape";

/// Durata registrata (pause escluse) e livelli dall'evento precedente.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTick {
    pub session_id: String,
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
    /// Il Tape; se non si è potuto scrivere, l'Ogg nella cartella nascosta.
    pub path: String,
    /// Perché la Registrazione si è fermata da sola (`deviceDisconnected`, `unwritableFolder`) o
    /// perché il Tape non si è scritto; `null` dopo Stop.
    pub error: Option<AppError>,
    /// L'esito della Trascrizione dal vivo; `null` se era spenta.
    pub transcription: Option<LiveTranscription>,
    #[serde(default)]
    pub diarizzazione: Option<crate::transcript::Diarizzazione>,
}

#[derive(Default)]
struct Controls {
    session_id: String,
    started: Mutex<bool>,
    paused: AtomicBool,
    stop: AtomicBool,
    /// Il Guadagno in dB del microfono e dell'audio di sistema.
    guadagno_microfono: AtomicI8,
    guadagno_sistema: AtomicI8,
    pulizia_microfono: AtomicBool,
    pulizia_sistema: AtomicBool,
    cleaning_pending: [AtomicBool; 2],
    sensibilita: Mutex<[Sensibilita; 2]>,
    /// Il Muto del microfono e dell'audio di sistema: ogni Registrazione parte senza.
    muto: [AtomicBool; 2],
}

impl Controls {
    fn stop(&self) {
        let _started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
        self.stop.store(true, Ordering::Release);
    }

    fn checkpoint(&self) -> Result<(), AppError> {
        if self.stop.load(Ordering::Acquire) {
            Err(AppError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn begin(&self) -> Result<(), AppError> {
        let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
        self.checkpoint()?;
        *started = true;
        Ok(())
    }
    fn set_audio(&self, settings: &Settings) {
        let _started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
        if self.stop.load(Ordering::Relaxed) {
            return;
        }
        self.guadagno_microfono
            .store(settings.guadagno_microfono, Ordering::Relaxed);
        self.guadagno_sistema
            .store(settings.guadagno_sistema, Ordering::Relaxed);
        self.pulizia_microfono
            .store(settings.audio_microfono.pulizia, Ordering::Relaxed);
        self.pulizia_sistema
            .store(settings.audio_sistema.pulizia, Ordering::Relaxed);
        for kind in [Kind::Microphone, Kind::System] {
            if !self.pulizia(kind) {
                self.cleaning_preparing(kind, false);
            }
        }
        *self
            .sensibilita
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = [
            settings.audio_microfono.sensibilita,
            settings.audio_sistema.sensibilita,
        ];
    }

    fn sensibilita(&self, kind: Kind) -> Sensibilita {
        self.sensibilita
            .lock()
            .unwrap_or_else(PoisonError::into_inner)[kind_index(kind)]
    }

    fn pulizia(&self, kind: Kind) -> bool {
        match kind {
            Kind::Microphone => self.pulizia_microfono.load(Ordering::Relaxed),
            Kind::System => self.pulizia_sistema.load(Ordering::Relaxed),
        }
    }

    /// Deduplica l'attesa per Ingresso; OFF la azzera anche fra due blocchi PCM.
    fn cleaning_preparing(&self, kind: Kind, pending: bool) -> bool {
        self.cleaning_pending[kind_index(kind)].swap(pending, Ordering::Relaxed) != pending
    }

    fn muto(&self, kind: Kind) -> bool {
        self.muto[kind_index(kind)].load(Ordering::Relaxed)
    }

    /// Il Guadagno in dB dell'ingresso `kind`.
    fn guadagno(&self, kind: Kind) -> i8 {
        match kind {
            Kind::Microphone => self.guadagno_microfono.load(Ordering::Relaxed),
            Kind::System => self.guadagno_sistema.load(Ordering::Relaxed),
        }
    }
}

/// La posizione dell'ingresso `kind` nei controlli per ingresso.
fn kind_index(kind: Kind) -> usize {
    match kind {
        Kind::Microphone => 0,
        Kind::System => 1,
    }
}

/// Pausa e Stop della Registrazione in corso. In `tauri::State`.
#[derive(Default)]
pub struct Recorder {
    current: Mutex<Option<Arc<Controls>>>,
    cancelled_start: Mutex<Option<String>>,
}

impl Recorder {
    /// Mette in pausa o riprende. Restituisce `false` se non c'è una Registrazione.
    pub fn set_paused(&self, paused: bool) -> bool {
        self.with(|c| {
            let _started = c.started.lock().unwrap_or_else(PoisonError::into_inner);
            c.paused.store(paused, Ordering::Relaxed);
        })
    }

    /// Mette in Muto (`true`) o toglie il Muto dell'ingresso `kind`, anche in Pausa. Restituisce
    /// `false` se non c'è una Registrazione.
    pub fn set_muto(&self, kind: Kind, muto: bool) -> bool {
        self.with(|c| c.muto[kind_index(kind)].store(muto, Ordering::Relaxed))
    }

    /// Ferma e salva. Restituisce `false` se non c'è una Registrazione.
    pub fn stop(&self) -> bool {
        self.with(Controls::stop)
    }

    /// Memorizza anche Annulla arrivato prima dell'invocazione di record, per questa sola sessione.
    pub fn cancel_start(&self, session_id: String) {
        let mut cancelled = self
            .cancelled_start
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(controls) = self.current().as_ref() {
            if controls.session_id == session_id {
                controls.stop();
            }
        } else {
            *cancelled = Some(session_id);
        }
    }

    fn register<'a>(&'a self, controls: &Arc<Controls>, settings: &Settings) -> Registration<'a> {
        let mut cancelled = self
            .cancelled_start
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut current = self.current();
        controls.set_audio(settings);
        if cancelled.as_deref() == Some(&controls.session_id) {
            controls.stop();
        }
        *cancelled = None;
        *current = Some(Arc::clone(controls));
        Registration {
            recorder: self,
            session_id: controls.session_id.clone(),
        }
    }

    /// Guadagno e pulizia salvati in `store`, per la Registrazione in corso. Legge sotto il
    /// lock dei controlli, come `record` quando li pubblica: vince sempre l'ultimo salvato.
    pub fn set_audio(&self, store: &SettingsStore) {
        self.with(|c| c.set_audio(&store.get()));
    }

    fn with(&self, f: impl FnOnce(&Controls)) -> bool {
        self.current().as_deref().map(f).is_some()
    }

    fn current(&self) -> MutexGuard<'_, Option<Arc<Controls>>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

struct Registration<'a> {
    recorder: &'a Recorder,
    session_id: String,
}
impl Drop for Registration<'_> {
    fn drop(&mut self) {
        let mut current = self.recorder.current();
        if current
            .as_ref()
            .is_some_and(|controls| controls.session_id == self.session_id)
        {
            *current = None;
        }
    }
}

#[cfg(test)]
#[path = "recording/start_tests.rs"]
mod start_tests;

#[cfg(test)]
#[path = "recording/cleaning_tests.rs"]
mod cleaning_tests;

/// Registra dagli ingressi delle impostazioni finché arriva Stop o un dispositivo si scollega,
/// emettendo `recording-tick`. `prefix` è il prefisso tradotto del nome del file; il Tape va nella
/// Raccolta `raccolta` (la radice della Libreria per `None` o `""`).
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn record(
    app: AppHandle,
    activity: &Activity,
    recorder: &Recorder,
    session_id: String,
    prefix: String,
    raccolta: Option<String>,
) -> Result<RecordingSaved, AppError> {
    let folder = recordings_folder(&app)?;
    let destination = Library::raccolta_dir(&folder, raccolta.as_deref())?;
    let started_at = Instant::now();
    let controls = Arc::new(Controls {
        session_id: session_id.clone(),
        ..Controls::default()
    });
    let cancel = CancelToken::new();
    let final_cancel = CancelToken::new();
    let final_phase = Arc::new(Mutex::new(false));
    // La Raccolta non si rinomina né si elimina finché il Tape non è scritto.
    let _activity = activity.begin(Some(destination.clone()), {
        let controls = Arc::clone(&controls);
        let cancel = cancel.clone();
        let final_cancel = final_cancel.clone();
        let final_phase = Arc::clone(&final_phase);
        // Stop durante la Registrazione, Annulla durante lo smaltimento della coda dal vivo.
        move || {
            if *final_phase.lock().unwrap_or_else(PoisonError::into_inner) {
                final_cancel.cancel();
            } else {
                controls.stop();
                cancel.cancel();
            }
        }
    })?;
    let internal = |e: tauri::Error| AppError::Internal(e.to_string());
    let settings = app.state::<SettingsStore>().get();
    let registration = recorder.register(&controls, &settings);
    controls.checkpoint()?;
    emit_phase(&app, &session_id, RecordingPhase::Preparing);
    let diarized = settings.parlanti_registrazione();
    // La scelta si fissa all'avvio, inclusi errori di disponibilità: niente fallback implicito.
    let diarizer = if diarized.is_empty() {
        None
    } else {
        let (app, settings) = (app.clone(), settings.clone());
        Some(
            tauri::async_runtime::spawn_blocking(move || {
                app.state::<Models>()
                    .reserve_configured_diarizer(&app, &settings)
            })
            .await
            .map_err(internal)
            .and_then(|result| result),
        )
    };
    controls.checkpoint()?;
    // Durante la cattura si avvia soltanto ASR. La Diarizzazione usa gli Ogg dopo Stop.
    let (mut feeds, sources) = live_sources(&settings)?;
    for feed in &mut feeds {
        let (notify_app, notify_session) = (app.clone(), session_id.clone());
        feed.on_failure(move |error| {
            if let Err(error) = (transcription::LiveTranscriptionFailed {
                session_id: notify_session,
                error,
            })
            .emit(&notify_app)
            {
                log::warn!("guasto coda ASR non emesso: {error}");
            }
        });
    }
    let live = if sources.is_empty() {
        None
    } else {
        Some(tauri::async_runtime::spawn_blocking({
            let (app, settings, cancel) = (app.clone(), settings.clone(), cancel.clone());
            let title = prefix.clone();
            let live_session_id = session_id.clone();
            move || {
                transcription::transcribe_live(
                    &app,
                    sources,
                    &live_session_id,
                    &settings,
                    &title,
                    &cancel,
                )
            }
        }))
    };
    let separate = settings.ingressi_separati();
    let recorded = tauri::async_runtime::spawn_blocking({
        let (app, controls, settings) = (app.clone(), Arc::clone(&controls), settings.clone());
        let (folder, prefix) = (folder.clone(), prefix.clone());
        let capture_session_id = session_id.clone();
        move || {
            run(
                &app,
                &capture_session_id,
                &folder,
                &prefix,
                &controls,
                &settings,
                feeds,
            )
        }
    })
    .await
    .map_err(internal)
    .and_then(|recorded| recorded);
    drop(registration);
    log::info!(
        "sessione {session_id}: cattura terminata dopo {:?} dalla richiesta backend",
        started_at.elapsed()
    );
    let recorded = match recorded {
        Ok(recorded) => recorded,
        Err(e) => {
            // La Registrazione non è partita: la pipeline si ferma da sola, senza aspettarla.
            cancel.cancel();
            return Err(e);
        }
    };
    let mut live = match live {
        Some(live) => {
            // Dopo Stop la status bar passa subito al completamento, anche col motore a metà Frase.
            if let Err(e) = (TranscriptionProgress {
                session_id: Some(session_id.clone()),
                percent: None,
            })
            .emit(&app)
            {
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
    let text_complete = {
        let mut phase = final_phase.lock().unwrap_or_else(PoisonError::into_inner);
        let complete = completa(
            live.as_ref().map(|(_, result)| result),
            cancel.is_cancelled(),
        );
        if cancel.is_cancelled() {
            final_cancel.cancel();
        }
        *phase = true;
        complete
    };
    if let (Some(diarizer), Some((Some(transcript), _))) = (diarizer, live.as_mut()) {
        if transcript.phrases.is_empty() {
            if let Err(error) = diarizer {
                crate::engine::diarize::finalize_live_ingressi(
                    transcript,
                    settings.diarizer,
                    diarized
                        .iter()
                        .map(|&ingresso| (ingresso, Err(error.clone())))
                        .collect(),
                );
            }
        } else {
            if let Err(error) = (DiarizationStarted {
                session_id: Some(session_id.clone()),
            })
            .emit(&app)
            {
                log::warn!("diarization-started non emesso: {error}");
            }
            let analyzed = tauri::async_runtime::spawn_blocking({
                let (mut transcript, cancel) = (transcript.clone(), final_cancel.clone());
                let (audio, modello) = (recorded.oggs.clone(), settings.diarizer);
                let diarized = diarized.clone();
                move || {
                    let analyzed = diarized
                        .into_iter()
                        .map(|ingresso| {
                            let result = (|| {
                                let lease = diarizer.as_ref().map_err(Clone::clone)?;
                                if cancel.is_cancelled() {
                                    return Err(AppError::Cancelled);
                                }
                                let path = audio
                                    .iter()
                                    .find(|(i, _)| *i == ingresso)
                                    .map(|(_, path)| path.as_path())
                                    .ok_or_else(|| {
                                        AppError::Internal("audio dell'Ingresso assente".into())
                                    })?;
                                lease.diarize_saved(path, &cancel)
                            })();
                            // Fissa l'esito quando termina questo Ingresso: Annulla nell'altro
                            // non invalida un successo già ottenuto.
                            let result = if cancel.is_cancelled() {
                                Err(AppError::Cancelled)
                            } else {
                                result
                            };
                            (ingresso, result)
                        })
                        .collect();
                    crate::engine::diarize::finalize_live_ingressi(
                        &mut transcript,
                        modello,
                        analyzed,
                    );
                    transcript
                }
            })
            .await;
            match analyzed {
                Ok(finalized) => *transcript = finalized,
                Err(error) => {
                    log::warn!("analisi finale interrotta: {error}");
                    crate::engine::diarize::finalize_live_ingressi(
                        transcript,
                        settings.diarizer,
                        diarized
                            .iter()
                            .map(|&ingresso| (ingresso, Err(AppError::Internal(error.to_string()))))
                            .collect(),
                    );
                }
            }
            transcription::final_speakers(&app, transcript, Some(&session_id));
        }
    }
    // Il Tape ha le Frasi arrivate anche se la Trascrizione è stata annullata o si è guastata.
    let phrases = live
        .as_ref()
        .and_then(|(transcript, _)| transcript.as_ref())
        .map_or(&[][..], |transcript| &transcript.phrases);
    let mut document = tape::Document::new(
        tape::creato(recorded.start),
        recorded.durata_ms,
        if separate {
            tape::Modalita::IngressiSeparati
        } else {
            tape::Modalita::Mix
        },
        live.is_some()
            .then(|| SettingsStore::model_of(&settings).id.clone()),
        settings.speech_language.clone(),
        text_complete,
        phrases,
    );
    document.diarizzazione = live
        .as_ref()
        .and_then(|(t, _)| t.as_ref())
        .and_then(|t| t.diarizzazione.clone());
    document.pulizia_audio = recorded.pulizia_audio.clone();
    document.set_nome_microfono(
        settings.nome_microfono_per(separate, &settings.parlanti_registrazione()),
    );
    let diarizzazione = document.diarizzazione.clone();
    let mut error = recorded.error.clone();
    let path =
        save_tape(&folder, &destination, &prefix, &recorded, &document).unwrap_or_else(|e| {
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
        diarizzazione,
    })
}

/// Se il testo del Tape è completo: la Trascrizione dal vivo c'era (`transcribed`), è arrivata alla
/// fine e non è stata annullata, nemmeno dopo l'ultima Frase.
fn completa(transcribed: Option<&Result<(), AppError>>, cancelled: bool) -> bool {
    matches!(transcribed, Some(Ok(()))) && !cancelled
}

/// Senza Tape, l'Ogg temporaneo del mix esce dalla cartella nascosta e va nella radice della
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
            log::warn!("Tape non scritto, la Registrazione è in {}", path.display());
            let _ = std::fs::remove_dir(folder.join(TEMP_FOLDER));
            path
        }
        Err(e) => {
            log::warn!(
                "Tape non scritto, la Registrazione resta in {}: {e}",
                mix.display()
            );
            mix.to_path_buf()
        }
    }
}

/// Solo i canali ASR: nessuna coda o istanza del diarizer durante la Registrazione.
fn live_sources(
    settings: &Settings,
) -> Result<(Vec<LiveFeed>, Vec<transcription::LiveSource>), AppError> {
    let ingressi: &[Ingresso] = match (settings.trascrizione_dal_vivo, settings.ingressi_separati())
    {
        (false, _) => &[],
        (true, false) => &[Ingresso::Mix],
        (true, true) => &[Ingresso::Microfono, Ingresso::Sistema],
    };
    if ingressi.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let (feeds, frames): (Vec<_>, Vec<_>) = live::channels(
        settings.sample_rate,
        channel_count(settings.channels),
        ingressi.len(),
    )?
    .into_iter()
    .unzip();
    let sources = ingressi
        .iter()
        .copied()
        .zip(frames)
        .map(|(ingresso, frames)| transcription::LiveSource { ingresso, frames })
        .collect();
    Ok((feeds, sources))
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
    pulizia_audio: Vec<TrattoPulizia>,
}

impl Recorded {
    fn mix(&self) -> &Path {
        &self.oggs[0].1
    }
}

/// Scrive il Tape `<prefisso> <data ora>.tape` nella cartella `destination` (una Raccolta della
/// Libreria `folder`, o la radice se nel frattempo è sparita) e cancella gli Ogg temporanei, e con
/// loro la cartella nascosta se resta vuota.
fn save_tape(
    folder: &Path,
    destination: &Path,
    prefix: &str,
    recorded: &Recorded,
    document: &tape::Document,
) -> Result<PathBuf, AppError> {
    let path = recording_path(
        if destination.is_dir() {
            destination
        } else {
            folder
        },
        prefix,
        recorded.start.naive_local(),
        "tape",
        Path::exists,
    );
    let audio: Vec<_> = recorded
        .oggs
        .iter()
        .map(|(ingresso, ogg)| (*ingresso, ogg.as_path()))
        .collect();
    tape::write(&path, &audio, document, Some(&recorded.forma_onda))?;
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

/// La Cartella della Libreria: quella delle impostazioni, o `Documenti\Memotape`.
pub fn recordings_folder(app: &AppHandle) -> Result<PathBuf, AppError> {
    match app.state::<SettingsStore>().get().recordings_folder {
        Some(folder) => Ok(PathBuf::from(folder)),
        None => default_recordings_folder(app),
    }
}

/// `Documenti\Memotape`, la Cartella della Libreria se le impostazioni non ne indicano un'altra.
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
    session_id: &str,
    folder: &Path,
    prefix: &str,
    controls: &Arc<Controls>,
    settings: &Settings,
    feeds: Vec<LiveFeed>,
) -> Result<Recorded, AppError> {
    let preparing_at = Instant::now();
    controls.checkpoint()?;
    let separate = settings.ingressi_separati();
    let channels = channel_count(settings.channels);
    let kinds = input_kinds(settings);
    let log = CleaningLog::default();
    let enabled: Vec<_> = kinds
        .iter()
        .map(|&kind| Arc::new(AtomicBool::new(controls.pulizia(kind))))
        .collect();
    // Un solo dispositivo trascrive il mix, ma usa il profilo di quel dispositivo.
    let timelines: Vec<_> = kinds
        .iter()
        .enumerate()
        .map(|(i, _)| {
            feeds
                .get(i)
                .map_or_else(ProtectionTimeline::default, LiveFeed::protection)
        })
        .collect();
    let model_path = preparation::resource_path(app)?;
    let filters = app
        .state::<RecordingFilters>()
        .configure(model_path.clone(), settings);
    let processors = kinds
        .iter()
        .zip(&enabled)
        .zip(filters)
        .map(|((&kind, enabled), filter)| {
            controls.checkpoint()?;
            let ingresso = match kind {
                Kind::Microphone => Ingresso::Microfono,
                Kind::System => Ingresso::Sistema,
            };
            let enabled = Arc::clone(enabled);
            let (notify_app, notify_session) = (app.clone(), session_id.to_owned());
            let deferred_filter = filter.clone();
            let (ready_app, ready_session) = (app.clone(), session_id.to_owned());
            let readiness_controls = Arc::clone(controls);
            let mut processor = ConfiguredCleaning::new(
                model_path.clone(),
                move || enabled.load(Ordering::Relaxed),
                ingresso,
                log.clone(),
            )
            .deferred(Box::new(move || {
                // Lo stesso lock di Stop decide se l'attivazione è ancora valida.
                let _started = readiness_controls
                    .started
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                if readiness_controls.stop.load(Ordering::Acquire)
                    || readiness_controls.paused.load(Ordering::Relaxed)
                    || !readiness_controls.pulizia(kind)
                {
                    return Ok(None);
                }
                let ready = deferred_filter.take_ready()?;
                let pending = ready.is_none();
                if readiness_controls.cleaning_preparing(kind, pending) {
                    emit_cleaning_preparing(&ready_app, &ready_session, ingresso, pending);
                }
                Ok(ready.map(|filter| Box::new(filter) as Box<dyn AudioProcessor>))
            }))
            .recovering(move |error| {
                log::warn!("pulizia {ingresso:?} interrotta: {error}");
                if let Err(error) = (RecordingCleaningFailed {
                    session_id: notify_session.clone(),
                    ingresso,
                    error,
                })
                .emit(&notify_app)
                {
                    log::warn!("recording-cleaning-failed non emesso: {error}");
                }
            });
            if controls.pulizia(kind) {
                emit_phase(app, session_id, RecordingPhase::Cleaning);
                let waiting_at = Instant::now();
                match filter
                    .wait(|| controls.stop.load(Ordering::Acquire) || !controls.pulizia(kind))
                {
                    Ok(filter) => processor = processor.prepared(Box::new(filter)),
                    Err(AppError::Cancelled) => controls.checkpoint()?,
                    Err(error) => processor.preparation_failed(error)?,
                }
                log::info!(
                    "sessione {session_id}: attesa pulizia {ingresso:?} {:?}",
                    waiting_at.elapsed()
                );
            }
            Ok(Box::new(processor) as Box<dyn AudioProcessor>)
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    controls.checkpoint()?;
    emit_phase(app, session_id, RecordingPhase::Devices);
    let devices_at = Instant::now();
    let (blocks_tx, blocks) = capture::channel(kinds.len());
    let captures = kinds
        .iter()
        .enumerate()
        .map(|(input, &kind)| {
            controls.checkpoint()?;
            let id = match kind {
                Kind::Microphone => settings.microphone.as_deref(),
                Kind::System => settings.output_device.as_deref(),
            };
            let capture = Capture::open(kind, id, input, blocks_tx.clone())?;
            controls.checkpoint()?;
            Ok::<_, AppError>(capture)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Restano solo i mittenti delle callback.
    drop(blocks_tx);
    // Tutte le catture hanno lo stesso orologio (QPC).
    let now = || captures[0].now();
    let formats: Vec<_> = captures.iter().map(|c| (c.rate, c.channels)).collect();
    let mut mixer = Mixer::with_processors(
        now(),
        &formats,
        (settings.sample_rate, channels),
        processors,
    )?;
    for (input, &kind) in kinds.iter().enumerate() {
        mixer.protect(input, &timelines[input], controls.sensibilita(kind));
    }
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
                let track = path.with_extension(tape::audio_entry(ingresso));
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
        let _ = std::fs::remove_dir(&temp);
    })?;
    if let Err(error) = controls.begin() {
        drop(outputs);
        drop(captures);
        for path in &created {
            let _ = std::fs::remove_file(path);
        }
        let _ = std::fs::remove_dir(&temp);
        return Err(error);
    }
    log::info!(
        "sessione {session_id}: dispositivi e writer {:?}, avvio audio {:?}",
        devices_at.elapsed(),
        preparing_at.elapsed()
    );
    emit_phase(app, session_id, RecordingPhase::Recording);
    let mut out = Vec::new();
    let mut last_tick = Instant::now();
    let mut error = None;
    // Guadagno e Muto applicati a ogni ingresso; ogni Registrazione parte senza Muto.
    let mut applied = vec![(None, false); kinds.len()];
    while !controls.stop.load(Ordering::Relaxed) {
        for ((input, &kind), (guadagno, muto)) in kinds.iter().enumerate().zip(&mut applied) {
            let db = controls.guadagno(kind);
            if *guadagno != Some(db) {
                *guadagno = Some(db);
                mixer.set_guadagno(input, guadagno_factor(db));
            }
            // Il silenzio del Muto chiude la Frase in corso con il VAD, come una pausa nel parlato.
            if *muto != controls.muto(kind) {
                *muto = !*muto;
                mixer.set_muto(input, *muto);
            }
        }
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
        if !paused && !controls.stop.load(Ordering::Relaxed) {
            for (input, (&kind, applied)) in kinds.iter().zip(&enabled).enumerate() {
                let next = controls.pulizia(kind);
                if next != applied.load(Ordering::Relaxed) {
                    // Scarica la coda col vecchio valore prima di osservare il nuovo PCM.
                    mixer.configuration(input, &mut out)?;
                    applied.store(next, Ordering::Relaxed);
                }
                mixer.protect(input, &timelines[input], controls.sensibilita(kind));
            }
        }
        if let Err(e) = mixer.advance_pending(
            now(),
            paused,
            received.as_ref().ok().map(|block| block.capture_ns),
            &mut out,
        ) {
            error = Some(e);
            break;
        }
        match received {
            Ok(block) => {
                let processed = mixer.push(block.input, block.capture_ns, &block.samples, &mut out);
                captures[block.input].recycle(block.samples);
                if let Err(e) = processed {
                    error = Some(e);
                    break;
                }
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
        if last_tick.elapsed() >= LEVELS_TICK {
            last_tick = Instant::now();
            let mut levels = Levels::default();
            for (&kind, peak) in kinds.iter().zip(mixer.take_peaks()) {
                match kind {
                    Kind::Microphone => levels.microphone = Some(peak),
                    Kind::System => levels.system = Some(peak),
                }
            }
            let tick = RecordingTick {
                session_id: session_id.to_owned(),
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
    // Chiude prima i produttori: la coda a Stop è finita anche se il DSP è più lento della cattura.
    drop(captures);
    for block in blocks.try_iter().filter(|b| b.capture_ns < stop) {
        if let Err(e) = mixer.push(block.input, block.capture_ns, &block.samples, &mut out) {
            error.get_or_insert(e);
            break;
        }
    }
    if let Err(e) = mixer.finish(stop, &mut out) {
        error.get_or_insert(e);
    }
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
        pulizia_audio: log.intervals(),
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

    #[test]
    fn pcm_elaborato_condivide_ogg_asr_forma_onda_e_tape_separato() {
        use crate::audio_toolkit::decode::Decoder;
        use crate::audio_toolkit::forma_onda::Picchi;
        use crate::audio_toolkit::ogg_opus::tests::{sine, temp_dir};
        use crate::audio_toolkit::processing::{Bypass, tests::DelayedScale};
        use crate::audio_toolkit::resample::FrameResampler;
        use crate::engine::pipeline::{
            self, PipelineEvent,
            tests::{EnergyDetector, FakeEngine},
        };
        use crate::managers::settings::SpeechLanguage;
        use crate::transcript::Phrase;

        let folder = temp_dir("pcm-comune-tape");
        let cleaning_log = CleaningLog::default();
        let processor = ConfiguredCleaning::with_factory(
            Box::new(|| true),
            Ingresso::Microfono,
            cleaning_log.clone(),
            Box::new(|_| Ok(Box::new(DelayedScale::new(2400)))),
        )
        .recovering(|error| panic!("{error}"));
        let ingressi = [Ingresso::Mix, Ingresso::Microfono, Ingresso::Sistema];
        let (feeds, sources): (Vec<_>, Vec<_>) =
            live::channels(48_000, 2, 2).unwrap().into_iter().unzip();
        let mut feeds = feeds.into_iter();
        let mut outputs: Vec<_> = ingressi
            .iter()
            .map(|&ingresso| {
                let path = folder.join(tape::audio_entry(ingresso));
                Output {
                    writer: OggOpusWriter::new(File::create(&path).unwrap(), 48_000, 2, 64)
                        .unwrap(),
                    ingresso,
                    path,
                    feed: (ingresso != Ingresso::Mix).then(|| feeds.next().unwrap()),
                }
            })
            .collect();
        let mut mixer = Mixer::with_processors(
            0,
            &[(48_000, 2), (48_000, 2)],
            (48_000, 2),
            vec![Box::new(processor), Box::new(Bypass)],
        )
        .unwrap()
        .with_tracks();
        let mic = sine(48_000, 2, 0.9);
        let system: Vec<f32> = mic.iter().map(|s| s * 0.25).collect();
        let expected = [
            mic.iter()
                .zip(&system)
                .map(|(m, s)| m * 0.5 + s)
                .collect::<Vec<_>>(),
            mic.iter().map(|s| s * 0.5).collect(),
            system.clone(),
        ];
        let mut out = Vec::new();
        let mut delivered = [Vec::new(), Vec::new(), Vec::new()];
        for (k, (m, s)) in mic.chunks(960).zip(system.chunks(960)).enumerate() {
            mixer
                .advance(k as u64 * 10_000_000, false, &mut out)
                .unwrap();
            mixer.push(0, k as u64 * 10_000_000, m, &mut out).unwrap();
            mixer.push(1, k as u64 * 10_000_000, s, &mut out).unwrap();
            for (all, audio) in delivered
                .iter_mut()
                .zip(std::iter::once(&out).chain(mixer.tracks().iter()))
            {
                all.extend_from_slice(audio);
            }
            write_ready(&mut outputs, &mut out, mixer.tracks(), false).unwrap();
        }
        mixer.finish(900_000_000, &mut out).unwrap();
        for (all, audio) in delivered
            .iter_mut()
            .zip(std::iter::once(&out).chain(mixer.tracks().iter()))
        {
            all.extend_from_slice(audio);
        }
        write_ready(&mut outputs, &mut out, mixer.tracks(), false).unwrap();
        assert_eq!(delivered, expected);
        let mut picchi = Picchi::new(48_000, 2);
        picchi.push(&expected[0]);
        let forma_onda = picchi.values(1000);
        let mut oggs = Vec::new();
        for output in outputs {
            if output.ingresso == Ingresso::Mix {
                assert_eq!(output.writer.forma_onda(), forma_onda);
            }
            output.writer.finish().unwrap();
            if let Some(feed) = output.feed {
                feed.finish();
            }
            oggs.push((output.ingresso, output.path));
        }
        let mut phrases = Vec::new();
        for ((ingresso, mut frames), pcm) in [Ingresso::Microfono, Ingresso::Sistema]
            .into_iter()
            .zip(sources)
            .zip(&expected[1..])
        {
            let mut engine = FakeEngine::default();
            pipeline::transcribe(
                &mut frames,
                &mut engine,
                &mut EnergyDetector,
                None,
                &CancelToken::new(),
                &mut |event| {
                    if let PipelineEvent::Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        tempi,
                        ..
                    } = event
                    {
                        phrases.push(Phrase {
                            inizio_ms,
                            fine_ms,
                            text,
                            tempi,
                            ingresso,
                            parlante: None,
                            parlante_provvisorio: false,
                            parlante_non_determinato: false,
                        });
                    }
                },
            )
            .unwrap();
            let mono: Vec<f32> = pcm.chunks_exact(2).map(|s| (s[0] + s[1]) * 0.5).collect();
            let mut resampler = FrameResampler::new(48_000).unwrap();
            let mut expected_asr = resampler.push(&mono);
            expected_asr.extend(resampler.finish());
            assert_eq!(engine.pcm, expected_asr.concat());
        }
        assert_eq!(phrases.len(), 2);
        assert!(phrases.iter().all(|p| p.inizio_ms == 0 && p.fine_ms == 900));
        let recorded = Recorded {
            oggs,
            start: Local::now(),
            durata_ms: 900,
            error: None,
            forma_onda: forma_onda.clone(),
            pulizia_audio: cleaning_log.intervals(),
        };
        let mut document = tape::Document::new(
            tape::creato(recorded.start),
            900,
            tape::Modalita::IngressiSeparati,
            Some("prova".into()),
            SpeechLanguage::from("it"),
            true,
            &phrases,
        );
        document.pulizia_audio = recorded.pulizia_audio.clone();
        assert_eq!(document.pulizia_audio.len(), 1);
        assert_eq!(document.pulizia_audio[0].ingresso, Ingresso::Microfono);
        assert_eq!(
            (
                document.pulizia_audio[0].inizio_frame,
                document.pulizia_audio[0].fine_frame
            ),
            (0, 43200)
        );
        let path = save_tape(&folder, &folder, "Registrazione", &recorded, &document).unwrap();
        assert_eq!(tape::read(&path).unwrap(), document);
        assert_eq!(tape::forma_onda(&path), Some(forma_onda));
        for (ingresso, pcm) in ingressi.into_iter().zip(&expected) {
            let mut decoder = Decoder::open_ingresso(&path, ingresso).unwrap();
            let mut reread = Vec::new();
            while let Some(block) = decoder.next_block().unwrap() {
                reread.extend(block.samples);
            }
            assert_eq!(reread.len(), pcm.len());
            let rmse = (reread
                .iter()
                .zip(pcm)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                / pcm.len() as f32)
                .sqrt();
            assert!(rmse < 0.015, "{ingresso:?}: RMSE {rmse}");
            let frames =
                pipeline::FileFrames::new(Decoder::open_ingresso(&path, ingresso).unwrap(), None);
            assert!(frames.collect::<Result<Vec<_>, _>>().is_ok());
        }
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn la_registrazione_avvia_solo_asr_anche_con_nemotron_e_tutti_i_parlanti_attivi() {
        use crate::engine::pipeline::Feed;
        use crate::managers::settings::Diarizer;
        for diarizer in [Diarizer::Sortformer, Diarizer::Nemotron3] {
            for recording_source in [
                RecordingSource::Mic,
                RecordingSource::System,
                RecordingSource::Both,
            ] {
                for enabled in [false, true] {
                    let settings = Settings {
                        diarizer,
                        recording_source,
                        trascrizione_dal_vivo: enabled,
                        parlanti_mix: true,
                        parlanti_microfono: true,
                        parlanti_sistema: true,
                        sample_rate: 16_000,
                        channels: Channels::Mono,
                        ..Settings::default()
                    };
                    let (feeds, sources) = live_sources(&settings).unwrap();
                    let expected = match (enabled, recording_source) {
                        (false, _) => vec![],
                        (true, RecordingSource::Both) => {
                            vec![Ingresso::Microfono, Ingresso::Sistema]
                        }
                        (true, _) => vec![Ingresso::Mix],
                    };
                    assert_eq!(
                        sources
                            .iter()
                            .map(|source| source.ingresso)
                            .collect::<Vec<_>>(),
                        expected
                    );
                    for (mut feed, source) in feeds.into_iter().zip(sources) {
                        feed.push(&[0.25; 480], false);
                        feed.push(&[], true);
                        feed.finish();
                        let received = source.frames.collect::<Vec<_>>();
                        assert!(
                            received
                                .iter()
                                .any(|item| matches!(item, Ok(Feed::Frame(_))))
                        );
                        assert!(
                            received
                                .iter()
                                .any(|item| matches!(item, Ok(Feed::ClosePhrase)))
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn dopo_l_analisi_finale_il_tape_si_salva_anche_con_annulla_o_guasto() {
        use crate::audio_toolkit::ogg_opus::tests::{sine, temp_dir};
        use crate::engine::diarize::{Turn, finalize};
        use crate::managers::settings::{Diarizer, SpeechLanguage};
        use crate::transcript::{EsitoDiarizzazione, Phrase};
        for esito in [
            EsitoDiarizzazione::Completata,
            EsitoDiarizzazione::Annullata,
            EsitoDiarizzazione::Fallita,
        ] {
            let folder = temp_dir("analisi-finale-tape");
            let ogg = temp_folder(&folder).unwrap().join("mix.ogg");
            let mut writer =
                OggOpusWriter::new(File::create(&ogg).unwrap(), 16_000, 1, 32).unwrap();
            writer.write(&sine(16_000, 1, 1.0)).unwrap();
            writer.finish().unwrap();
            let audio = std::fs::read(&ogg).unwrap();
            let recorded = Recorded {
                oggs: vec![(Ingresso::Mix, ogg)],
                start: Local::now(),
                durata_ms: 1000,
                error: None,
                forma_onda: vec![0.5],
                pulizia_audio: Vec::new(),
            };
            let mut phrases = vec![Phrase {
                inizio_ms: 0,
                fine_ms: 1000,
                text: "Testo concluso.".into(),
                tempi: Vec::new(),
                ingresso: Ingresso::Mix,
                parlante: Some(2),
                parlante_provvisorio: true,
                parlante_non_determinato: false,
            }];
            let state = finalize(
                &mut phrases,
                Diarizer::Nemotron3,
                &CancelToken::new(),
                match esito {
                    EsitoDiarizzazione::Completata => Ok(vec![(
                        Ingresso::Mix,
                        vec![Turn {
                            inizio_ms: 0,
                            fine_ms: 1000,
                            parlante: 7,
                        }],
                    )]),
                    EsitoDiarizzazione::Annullata => Err(AppError::Cancelled),
                    EsitoDiarizzazione::Fallita => Err(AppError::Internal("guasto".into())),
                },
            )
            .unwrap();
            assert_eq!(state.esito, esito);
            let mut document = tape::Document::new(
                tape::creato(recorded.start),
                1000,
                tape::Modalita::Mix,
                Some("nemotron".into()),
                SpeechLanguage::from("it"),
                true,
                &phrases,
            );
            document.diarizzazione = Some(state);
            let path = save_tape(&folder, &folder, "Registrazione", &recorded, &document).unwrap();
            assert_eq!(tape::read(&path).unwrap(), document);
            let opened = transcription::open_tape(&path).unwrap();
            assert!(opened.info.completa);
            assert_eq!(opened.phrases[0].text, "Testo concluso.");
            assert_eq!(
                opened.phrases[0].parlante_provvisorio,
                esito != EsitoDiarizzazione::Completata
            );
            let mut mix = tape::Mix::open(&path).unwrap();
            let mut saved_audio = Vec::new();
            std::io::Read::read_to_end(&mut mix, &mut saved_audio).unwrap();
            assert_eq!(saved_audio, audio);
            assert_eq!(tape::forma_onda(&path), Some(vec![0.5]));
            assert!(!folder.join(TEMP_FOLDER).exists());
        }
    }

    #[test]
    #[ignore = "richiede Nemotron ASR e MEMOTAPE_NEMOTRON3_MODEL; eseguire in sequenza"]
    fn nemotron3_analisi_finale_salva_e_riapre_la_registrazione() {
        use crate::audio_toolkit::decode::Decoder;
        use crate::audio_toolkit::ogg_opus::tests::temp_dir;
        use crate::audio_toolkit::vad::Silero;
        use crate::engine::diarize::finalize;
        use crate::engine::pipeline::{PipelineEvent, transcribe_file};
        use crate::engine::transcribe_cpp::{OfflineDiarizer, TranscribeCpp};
        use crate::managers::models;
        use crate::managers::settings::{Diarizer, SpeechLanguage};
        use crate::transcript::{EsitoDiarizzazione, Phrase};
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture = root.join("tests/fixtures/parlato-due-voci.wav");
        let folder = temp_dir("nemotron3-registrazione-finale");
        let ogg = temp_folder(&folder).unwrap().join("mix.ogg");
        let mut writer = OggOpusWriter::new(File::create(&ogg).unwrap(), 16_000, 1, 32).unwrap();
        let mut decoder = Decoder::open(&fixture).unwrap();
        while let Some(block) = decoder.next_block().unwrap() {
            assert_eq!(block.rate, 16_000);
            writer.write(&block.mono()).unwrap();
        }
        let forma_onda = writer.forma_onda();
        writer.finish().unwrap();
        let audio = std::fs::read(&ogg).unwrap();
        let models_dir =
            PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
        let mut engine = TranscribeCpp::load(&models::default_model().path(&models_dir)).unwrap();
        let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
        let mut phrases = Vec::new();
        let durata_ms = transcribe_file(
            &ogg,
            &mut engine,
            &mut detector,
            Some("it"),
            None,
            None,
            &CancelToken::new(),
            &mut |event| {
                if let PipelineEvent::Phrase {
                    inizio_ms,
                    fine_ms,
                    text,
                    tempi,
                    ..
                } = event
                {
                    phrases.push(Phrase {
                        tempi,
                        inizio_ms,
                        fine_ms,
                        text,
                        ingresso: Ingresso::Mix,
                        parlante: None,
                        parlante_provvisorio: false,
                        parlante_non_determinato: false,
                    });
                }
            },
        )
        .unwrap();
        drop(engine);
        let texts: Vec<_> = phrases.iter().map(|p| p.text.clone()).collect();
        assert!(!texts.is_empty());
        let cancel = CancelToken::new();
        let model = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
        let mut diarizer = OfflineDiarizer::load_nemotron3(&model).unwrap();
        let state = finalize(
            &mut phrases,
            Diarizer::Nemotron3,
            &cancel,
            diarizer
                .diarize_saved(Decoder::open(&ogg).unwrap(), &cancel)
                .map(|turns| vec![(Ingresso::Mix, turns)]),
        )
        .unwrap();
        assert_eq!(state.esito, EsitoDiarizzazione::Completata);
        assert!(phrases.iter().any(|p| p.parlante == Some(1)));
        assert!(phrases.iter().any(|p| p.parlante == Some(2)));
        assert_eq!(
            phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
            texts.concat()
        );
        let recorded = Recorded {
            oggs: vec![(Ingresso::Mix, ogg)],
            start: Local::now(),
            durata_ms,
            error: None,
            forma_onda,
            pulizia_audio: Vec::new(),
        };
        let mut document = tape::Document::new(
            tape::creato(recorded.start),
            durata_ms,
            tape::Modalita::Mix,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            true,
            &phrases,
        );
        document.diarizzazione = Some(state);
        let path = save_tape(&folder, &folder, "Registrazione", &recorded, &document).unwrap();
        assert_eq!(tape::read(&path).unwrap(), document);
        assert_eq!(
            transcription::open_tape(&path).unwrap().info.diarizzazione,
            document.diarizzazione
        );
        let mut mix = tape::Mix::open(&path).unwrap();
        let mut saved_audio = Vec::new();
        std::io::Read::read_to_end(&mut mix, &mut saved_audio).unwrap();
        assert_eq!(saved_audio, audio);
        assert_eq!(tape::forma_onda(&path), Some(recorded.forma_onda));
        assert!(!folder.join(TEMP_FOLDER).exists());
    }

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn il_testo_del_tape_e_completo_solo_se_la_trascrizione_dal_vivo_e_finita() {
        let guasta = Err(AppError::Internal("x".into()));
        assert!(completa(Some(&Ok(())), false));
        assert!(!completa(Some(&Ok(())), true));
        assert!(!completa(Some(&guasta), false));
        assert!(!completa(Some(&Err(AppError::Cancelled)), true));
        // Senza Trascrizione dal vivo non c'è testo.
        assert!(!completa(None, false));
    }

    #[test]
    fn il_tape_va_nella_raccolta_o_nella_radice_se_la_raccolta_non_c_e_piu() {
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
                pulizia_audio: Vec::new(),
            }
        };
        let document = tape::Document::new(
            tape::creato(start),
            100,
            tape::Modalita::Mix,
            None,
            crate::managers::settings::SpeechLanguage::auto(),
            false,
            &[],
        );
        let path = save_tape(&folder, &raccolta, "Registrazione", &recorded(), &document).unwrap();
        assert_eq!(
            path,
            raccolta.join(format!(
                "Registrazione {}.tape",
                start.format("%Y-%m-%d %H-%M-%S")
            ))
        );
        assert_eq!(tape::read(&path).unwrap(), document);
        assert_eq!(tape::forma_onda(&path), Some(vec![0.5, 0.25]));
        // L'Ogg temporaneo stava nella radice, e la cartella nascosta se ne va con lui.
        assert!(!folder.join(TEMP_FOLDER).exists());
        let sparita = folder.join("Sparita");
        let path = save_tape(&folder, &sparita, "Registrazione", &recorded(), &document).unwrap();
        assert_eq!(path.parent(), Some(folder.as_path()));
    }

    #[test]
    fn il_nome_ha_prefisso_data_e_ora_e_non_sovrascrive_mai() {
        let folder = Path::new(r"C:\Users\me\Documents\Memotape");
        let start = at("2026-03-07 09:05:01");
        let taken =
            |names: &'static [&str]| move |p: &Path| names.iter().any(|n| p == folder.join(n));
        assert_eq!(
            recording_path(folder, "Registrazione", start, "tape", taken(&[])),
            folder.join("Registrazione 2026-03-07 09-05-01.tape")
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
