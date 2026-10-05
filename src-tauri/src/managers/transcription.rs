//! Trascrizione di una Sorgente, o dal vivo di una Registrazione (il mix, o con gli Ingressi separati
//! ogni Ingresso con una sua pipeline): prende il motore del modello scelto (caricato una volta e
//! tenuto tra una Trascrizione e l'altra), esegue la pipeline e la traduce in eventi. Un file audio o
//! video diventa un Bino nella Raccolta, di un Bino si riscrive il testo. Tiene l'ultima Trascrizione
//! per Copia testo finché non c'è un Bino da cui copiare.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::ogg_opus::OggCopy;
use crate::audio_toolkit::vad::{Silero, VoiceDetector};
use crate::bino;
use crate::engine::live::LiveFrames;
use crate::engine::pipeline::{self, Feed, PipelineEvent, transcribe_file};
use crate::engine::transcribe_cpp::TranscribeCpp;
use crate::engine::{TranscriptionEngine, diarize};
use crate::error::AppError;
use crate::library::Library;
use crate::managers::activity::Activity;
use crate::managers::models::{self, DiarizerLease, Models};
use crate::managers::recording::{
    TEMP_FOLDER, channel_count, create_numbered, numbered, recordings_folder, temp_folder,
};
use crate::managers::settings::{CopiaCome, Language, Settings, SettingsStore, SpeechLanguage};
use crate::transcript::{self, Ingresso, Labels, Phrase, Transcript};

const SILERO_RESOURCE: &str = "resources/silero_vad.onnx";

/// Una Frase conclusa, una per riga nell'area di testo. Sostituisce il Parziale con lo stesso id.
/// `inizio_ms` e `fine_ms` sono sulla linea del tempo della Sorgente.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptPhrase {
    pub phrase_id: u32,
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub text: String,
    /// Con gli Ingressi separati ogni Ingresso ha le sue Frasi, con id propri.
    pub ingresso: Ingresso,
    /// Il Parlante, da 1: c'è nelle Frasi di un Bino diarizzato. Durante una Trascrizione arriva
    /// dopo, con `speakers-assigned`.
    pub parlante: Option<u32>,
}

/// Il Parziale della Frase in corso (solo con i modelli in streaming): sostituisce il precedente e
/// ha l'id che avrà la Frase. `fine_ms` è la fine dell'audio letto finora.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptPartial {
    pub phrase_id: u32,
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub text: String,
    pub ingresso: Ingresso,
}

/// La Trascrizione dal vivo si è fermata (modello assente, guasto): la Registrazione continua senza
/// testo.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct LiveTranscriptionFailed {
    pub error: AppError,
}

/// Finita la Trascrizione, comincia la Diarizzazione (Riconosci i parlanti).
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct DiarizationStarted;

/// I Parlanti delle Frasi dopo la Diarizzazione: `parlante` da 1 per ordine di comparsa, `null` se
/// nessuno parlava durante la Frase.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct SpeakersAssigned {
    pub speakers: Vec<SpeakerAssignment>,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerAssignment {
    pub ingresso: Ingresso,
    pub phrase_id: u32,
    pub parlante: Option<u32>,
}

/// Avanzamento della Trascrizione, o dello smaltimento della coda dal vivo dopo Stop: `percent` è
/// `null` se la durata della Sorgente non è nota.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct TranscriptionProgress {
    pub percent: Option<u8>,
}

/// Esito di una Trascrizione arrivata alla fine della Sorgente. Annulla e i guasti sono `AppError`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum TranscriptionOutcome {
    /// Il testo è nel Bino `path`: quello nuovo di un file, o il Bino trascritto. Diventa la
    /// Sorgente.
    Saved { path: String },
    /// Nessuna Frase: un file non diventa un Bino; un Bino resta senza Frasi.
    NoSpeech,
}

/// Com'è finita la Trascrizione dal vivo di una Registrazione salvata.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum LiveTranscription {
    /// Il testo è nel Bino o, se il Bino non si è scritto, nel Markdown accanto all'Ogg.
    Saved,
    NoSpeech,
    /// Modello assente (`liveTranscriptionUnavailable`), guasto o Annulla (`cancelled`): il Bino ha
    /// le Frasi arrivate, con il testo incompleto.
    Failed {
        error: AppError,
    },
}

/// Un Bino aperto come Sorgente: le Frasi, i nomi dei Parlanti e le informazioni.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct OpenedBino {
    pub phrases: Vec<TranscriptPhrase>,
    /// Per chiave `<ingresso>:<n>`, come nel Bino.
    pub parlanti: BTreeMap<String, String>,
    pub info: BinoInfo,
}

/// La riga di informazioni della vista di un Bino.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BinoInfo {
    /// Data e ora della Registrazione o della Trascrizione, ISO 8601 con il fuso.
    pub creato: String,
    pub durata_ms: u32,
    /// Il nome del modello; `null` se il testo non è stato trascritto.
    pub modello: Option<String>,
    pub lingua_parlato: SpeechLanguage,
    pub ingressi_separati: bool,
    pub completa: bool,
    /// Il nome del file audio o video da cui viene.
    pub origine: Option<String>,
}

/// L'ultima Trascrizione, di un file o dal vivo, anche annullata: quella che Copia testo rende
/// finché non c'è un Bino aperto. Si riempie man mano che arrivano le Frasi. In `tauri::State`.
#[derive(Default)]
pub struct LastTranscript(Mutex<Option<Transcript>>);

impl LastTranscript {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Transcript>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Modifica la Trascrizione tenuta, se c'è.
    fn update(&self, change: impl FnOnce(&mut Transcript)) {
        if let Some(transcript) = self.lock().as_mut() {
            change(transcript);
        }
    }

    fn set(&self, transcript: Transcript) {
        *self.lock() = Some(transcript);
    }

    fn get(&self) -> Option<Transcript> {
        self.lock().clone()
    }
}

/// Il testo di Copia testo: l'ultima Trascrizione in testo semplice o in Markdown, come dicono le
/// impostazioni. `None` se non ce n'è ancora una.
pub fn transcript_text(last: &LastTranscript, settings: &Settings) -> Option<String> {
    last.get()
        .map(|t| transcript::render(&t, &labels(settings), settings.copia_come))
}

/// Il Bino `path` reso come documento, con le correzioni e i nomi dei Parlanti, in `format`: per
/// Copia testo e per Esporta Markdown….
pub fn bino_text(path: &Path, settings: &Settings, format: CopiaCome) -> Result<String, AppError> {
    let transcript = bino_transcript(title_of(path), bino::read(path)?);
    Ok(transcript::render(&transcript, &labels(settings), format))
}

/// I testi del documento nella Lingua dell'interfaccia.
pub fn labels(settings: &Settings) -> Labels {
    Labels::of(settings.interface_language.unwrap_or_else(Language::system))
}

/// Una Trascrizione che parte adesso, ancora senza Frasi, con il modello e la Lingua del parlato
/// delle impostazioni. Diventa subito `LastTranscript`, prima di caricare il modello: se il
/// caricamento fallisce, Copia testo non rende la Trascrizione precedente.
fn begin_transcript(app: &AppHandle, title: String, settings: &Settings) -> Transcript {
    let transcript = Transcript {
        title,
        date: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        durata_ms: None,
        model: SettingsStore::model_of(settings).name.clone(),
        speech_language: settings.speech_language.clone(),
        phrases: Vec::new(),
        parlanti: BTreeMap::new(),
    };
    app.state::<LastTranscript>().set(transcript.clone());
    transcript
}

/// Il nome della Sorgente senza l'ultima estensione: il titolo del documento.
pub(crate) fn title_of(source: &Path) -> String {
    source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Trascrive `source` emettendo `transcription-progress`, `transcript-partial` e
/// `transcript-phrase`; con Riconosci i parlanti poi diarizza (`diarization-started`,
/// `speakers-assigned`). Un file audio o video diventa un Bino nella Raccolta `raccolta` (la radice
/// della Libreria per `None` o `""`); di un Bino si riscrive il testo.
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn transcribe(
    app: AppHandle,
    activity: &Activity,
    source: PathBuf,
    raccolta: Option<String>,
) -> Result<TranscriptionOutcome, AppError> {
    let library = recordings_folder(&app)?;
    let destination = Library::raccolta_dir(&library, raccolta.as_deref())?;
    let cancel = CancelToken::new();
    // Le scritture di altri comandi su quel Bino, o sulla Raccolta del Bino che nascerà, si
    // rifiutano finché la Trascrizione non è finita.
    let target = if bino::is_bino(&source) {
        source.clone()
    } else {
        destination.clone()
    };
    let _activity = activity.begin(Some(target), {
        let cancel = cancel.clone();
        move || cancel.cancel()
    })?;
    let settings = app.state::<SettingsStore>().get();
    let model = SettingsStore::model_of(&settings);
    let silero = silero_path(&app)?;
    // Un Bino illeggibile o di una versione più nuova si rifiuta prima di caricare il modello.
    let bino = bino::is_bino(&source)
        .then(|| bino::read(&source))
        .transpose()?;
    // Senza Sortformer lo si dice subito, non dopo aver trascritto.
    let diarizer = settings
        .parlanti_file
        .then(|| app.state::<Models>().reserve_diarizer(&app))
        .transpose()?;
    let diarize = diarizer.is_some();
    let started = chrono::Local::now();
    let transcript = Mutex::new(begin_transcript(&app, title_of(&source), &settings));
    tauri::async_runtime::spawn_blocking(move || {
        // La Trascrizione, con la Diarizzazione; `copy` riceve l'audio di un file.
        let run = |copy: Option<OggCopy>| {
            let models = app.state::<Models>();
            let mut engine = models.take(&app, || model.id.as_str())?;
            let mut audio = Vec::new();
            let transcribed = run_pipeline(
                &app,
                &mut engine,
                &silero,
                &cancel,
                Ingresso::Mix,
                &transcript,
                |engine, detector, on_event| {
                    transcribe_file(
                        &source,
                        engine,
                        detector,
                        settings.speech_language.code(),
                        diarize.then_some(&mut audio),
                        copy,
                        &cancel,
                        on_event,
                    )
                },
            );
            models.release(&app, engine, keep_engine(&transcribed));
            transcribed?;
            let mut transcript = transcript
                .into_inner()
                .unwrap_or_else(PoisonError::into_inner);
            if let Some(diarizer) = diarizer
                && !cancel.is_cancelled()
            {
                diarize_phrases(
                    &app,
                    &diarizer,
                    &[(Ingresso::Mix, audio)],
                    &mut transcript,
                    &cancel,
                )?;
            }
            // Annulla premuto dopo l'ultima Frase: il Bino non cambia e non nasce.
            if cancel.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            Ok(transcript)
        };
        let document = |creato, durata_ms, transcript: &Transcript| {
            bino::Document::new(
                creato,
                durata_ms,
                bino::Modalita::Mix,
                Some(model.id.clone()),
                settings.speech_language.clone(),
                true,
                &transcript.phrases,
            )
        };
        // Il Bino con le Frasi; anche senza parlato un Bino si riscrive.
        let saved = match bino {
            Some(old) => {
                let transcript = run(None)?;
                let rewritten = bino::Document {
                    origine: old.origine,
                    ..document(old.creato, old.durata_ms, &transcript)
                };
                bino::rewrite(&source, &rewritten)?;
                (!transcript.phrases.is_empty()).then_some(source)
            }
            None => file_to_bino(&library, &destination, &source, &settings, |copy| {
                let transcript = run(Some(copy))?;
                Ok(bino::Document {
                    origine: source.file_name().map(|n| n.to_string_lossy().into_owned()),
                    // L'ora del file, se la dice; altrimenti quella della Trascrizione.
                    ..document(
                        bino::creato(modified_at(&source).unwrap_or(started)),
                        transcript.durata_ms.unwrap_or_default(),
                        &transcript,
                    )
                })
            })?,
        };
        Ok(saved.map_or(TranscriptionOutcome::NoSpeech, |path| {
            TranscriptionOutcome::Saved {
                path: path.display().to_string(),
            }
        }))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Il Bino `<nome del file>.bino` del file audio o video `source`, nella cartella `destination` (la
/// radice della Libreria `library` se nel frattempo è sparita), con " 2", " 3"… se esiste già.
/// `transcribe` riceve la copia dell'audio, scritta in un Ogg temporaneo nella cartella nascosta con
/// il formato della Registrazione, e restituisce il documento. Annullata, guasta o senza Frasi
/// (`None`): non restano né Bino né Ogg temporaneo.
fn file_to_bino(
    library: &Path,
    destination: &Path,
    source: &Path,
    settings: &Settings,
    transcribe: impl FnOnce(OggCopy) -> Result<bino::Document, AppError>,
) -> Result<Option<PathBuf>, AppError> {
    let temp = temp_folder(library)?;
    let (ogg, file) = create_numbered(&temp, &title_of(source), "ogg")?;
    let _temporary = Temporary(ogg.clone());
    let copy = OggCopy::new(
        file,
        settings.sample_rate,
        channel_count(settings.channels),
        settings.bitrate_kbps,
    )?;
    let forma_onda = copy.forma_onda();
    let document = transcribe(copy)?;
    if document.frasi.is_empty() {
        return Ok(None);
    }
    let folder = if destination.is_dir() {
        destination
    } else {
        library
    };
    let path = numbered(folder, &title_of(source), "bino", Path::exists);
    bino::write(
        &path,
        &[(Ingresso::Mix, &ogg)],
        &document,
        forma_onda.get().map(Vec::as_slice),
    )?;
    Ok(Some(path))
}

/// La data di modifica del file `source`: per un audio o un video importato è l'ora più vicina a
/// quella in cui è stato registrato.
fn modified_at(source: &Path) -> Option<chrono::DateTime<chrono::Local>> {
    std::fs::metadata(source)
        .and_then(|m| m.modified())
        .ok()
        .map(Into::into)
}

/// L'Ogg temporaneo del Bino di un file: al drop si cancella, e con lui la cartella nascosta se resta
/// vuota.
struct Temporary(PathBuf);

impl Drop for Temporary {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.0) {
            log::warn!("{} non cancellato: {e}", self.0.display());
        }
        if let Some(temp) = self.0.parent().filter(|p| p.ends_with(TEMP_FOLDER)) {
            let _ = std::fs::remove_dir(temp);
        }
    }
}

/// Attribuisce ai Parlanti le Frasi di `transcript` con Sortformer sull'audio intero (a 16 kHz) di
/// ogni Ingresso di `audio`, uno alla volta, e lo dice con `speakers-assigned`. Un Ingresso senza
/// Frasi non si diarizza; senza nessuna non emette nulla.
fn diarize_phrases(
    app: &AppHandle,
    diarizer: &DiarizerLease,
    audio: &[(Ingresso, Vec<f32>)],
    transcript: &mut Transcript,
    cancel: &CancelToken,
) -> Result<(), AppError> {
    let with_phrases: Vec<_> = audio
        .iter()
        .filter(|(ingresso, _)| transcript.phrases.iter().any(|p| p.ingresso == *ingresso))
        .collect();
    if with_phrases.is_empty() {
        return Ok(());
    }
    if let Err(e) = DiarizationStarted.emit(app) {
        log::warn!("diarization-started non emesso: {e}");
    }
    // Prima tutti i turni, poi i Parlanti: con un errore o Annulla a metà nessuna Frase li ha.
    let mut turns = Vec::new();
    for (ingresso, pcm) in with_phrases {
        let started = std::time::Instant::now();
        let found = diarizer.diarize(pcm, cancel)?;
        log::info!(
            "Diarizzazione di {ingresso:?}, {} s in {:?}: {} turni",
            pcm.len() / 16_000,
            started.elapsed(),
            found.len()
        );
        turns.push((*ingresso, found));
    }
    for (ingresso, found) in &turns {
        diarize::assign_ingresso(&mut transcript.phrases, *ingresso, found);
    }
    app.state::<LastTranscript>()
        .update(|last| last.phrases.clone_from(&transcript.phrases));
    let assigned = SpeakersAssigned {
        speakers: assignments(&transcript.phrases),
    };
    if let Err(e) = assigned.emit(app) {
        log::warn!("speakers-assigned non emesso: {e}");
    }
    Ok(())
}

/// Il Parlante di ogni Frase per `speakers-assigned`, con l'id che le ha dato la pipeline del suo
/// Ingresso: la posizione tra le Frasi di quell'Ingresso, che la pipeline numera in ordine di inizio.
fn assignments(phrases: &[Phrase]) -> Vec<SpeakerAssignment> {
    let mut next_id: Vec<(Ingresso, u32)> = Vec::new();
    phrases
        .iter()
        .map(|phrase| {
            let phrase_id = match next_id.iter_mut().find(|(i, _)| *i == phrase.ingresso) {
                Some((_, id)) => {
                    *id += 1;
                    *id
                }
                None => {
                    next_id.push((phrase.ingresso, 0));
                    0
                }
            };
            SpeakerAssignment {
                ingresso: phrase.ingresso,
                phrase_id,
                parlante: phrase.parlante,
            }
        })
        .collect()
}

/// La Trascrizione dal vivo di una Registrazione: carica il modello scelto e trascrive ogni fonte di
/// `sources` (il mix, o con gli Ingressi separati ogni Ingresso, ciascuno con una sua istanza del
/// modello e una sua pipeline, in parallelo) man mano che arrivano i frame, fino alla fine della
/// Registrazione e delle code, con gli eventi di `transcribe`. Restituisce il documento, intitolato
/// `title` finché non si sa il nome del file, con le Frasi arrivate in ordine di inizio anche se la
/// Trascrizione è stata annullata o si è guastata, e com'è finita (il primo errore). Se il modello
/// non si carica (`liveTranscriptionUnavailable`) o una pipeline si guasta emette subito
/// `live-transcription-failed`: la Registrazione continua, e l'altro Ingresso anche. Con Riconosci
/// i parlanti (`Settings::parlanti_registrazione`) tiene l'audio degli Ingressi scelti e, finite le
/// code, li diarizza; senza Sortformer lo avvisa subito con `live-transcription-failed`
/// (`diarizerMissing`) e trascrive senza Parlanti.
pub fn transcribe_live(
    app: &AppHandle,
    sources: Vec<(Ingresso, LiveFrames)>,
    settings: &Settings,
    title: &str,
    cancel: &CancelToken,
) -> (Transcript, Result<(), AppError>) {
    let model = SettingsStore::model_of(settings);
    let transcript = Mutex::new(begin_transcript(app, title.to_string(), settings));
    let models = app.state::<Models>();
    let unavailable = |e: AppError| {
        log::warn!("Trascrizione dal vivo senza modello: {e}");
        AppError::LiveTranscriptionUnavailable(model.name.clone())
    };
    let prepared = silero_path(app).and_then(|silero| {
        let lease = models
            .take(app, || model.id.as_str())
            .map_err(unavailable)?;
        let extra: Result<Vec<_>, _> = (1..sources.len())
            .map(|_| models.load_instance(app, &lease))
            .collect();
        match extra {
            Ok(extra) => Ok((silero, lease, extra)),
            Err(e) => {
                models.release(app, lease, true);
                Err(unavailable(e))
            }
        }
    });
    let transcribed = match prepared {
        Err(error) => {
            live_failed(app, &error, cancel);
            Err(error)
        }
        Ok((silero, mut lease, mut extra)) => {
            let diarized = settings.parlanti_registrazione();
            // Una Registrazione che non è partita non prenota Sortformer.
            let diarizer = if diarized.is_empty() || cancel.is_cancelled() {
                None
            } else {
                models
                    .reserve_diarizer(app)
                    .inspect_err(|e| live_failed(app, e, cancel))
                    .ok()
            };
            let engines = std::iter::once(&mut *lease).chain(&mut extra);
            let results: Vec<_> = std::thread::scope(|scope| {
                let pipelines: Vec<_> = sources
                    .into_iter()
                    .zip(engines)
                    .map(|((ingresso, mut frames), engine)| {
                        let (silero, transcript) = (&silero, &transcript);
                        // La Diarizzazione vuole l'audio intero dell'Ingresso, come la pipeline lo
                        // ha ricevuto. ponytail: in memoria, 230 MB l'ora per Ingresso.
                        let keep = diarizer.is_some() && diarized.contains(&ingresso);
                        let pipeline = scope.spawn(move || {
                            let mut audio = Vec::new();
                            let mut frames = frames.by_ref().inspect(|feed| {
                                if let (true, Ok(Feed::Frame(frame))) = (keep, feed) {
                                    audio.extend_from_slice(frame);
                                }
                            });
                            let transcribed = run_pipeline(
                                app,
                                engine,
                                silero,
                                cancel,
                                ingresso,
                                transcript,
                                |engine, detector, on_event| {
                                    pipeline::transcribe(
                                        &mut frames,
                                        engine,
                                        detector,
                                        settings.speech_language.code(),
                                        cancel,
                                        on_event,
                                    )
                                },
                            );
                            if let Err(error) = &transcribed {
                                live_failed(app, error, cancel);
                            }
                            drop(frames);
                            (transcribed, audio)
                        });
                        (ingresso, pipeline)
                    })
                    .collect();
                pipelines
                    .into_iter()
                    .map(|(ingresso, pipeline)| {
                        let (transcribed, audio) = pipeline.join().unwrap_or_else(|_| {
                            let error = AppError::Internal("pipeline dal vivo interrotta".into());
                            (Err(error), Vec::new())
                        });
                        (transcribed, (ingresso, audio))
                    })
                    .collect()
            });
            // La seconda istanza si libera con la Registrazione.
            drop(extra);
            let (outcomes, audio): (Vec<_>, Vec<_>) = results.into_iter().unzip();
            models.release(app, lease, outcomes.iter().all(keep_engine));
            outcomes
                .into_iter()
                .collect::<Result<(), AppError>>()
                .and_then(|()| match diarizer {
                    // Dopo Stop, finite le code: i Parlanti arrivano prima che si componga il Bino.
                    Some(diarizer) if !cancel.is_cancelled() => {
                        let audio: Vec<_> =
                            audio.into_iter().filter(|(_, a)| !a.is_empty()).collect();
                        diarize_phrases(
                            app,
                            &diarizer,
                            &audio,
                            &mut transcript.lock().unwrap_or_else(PoisonError::into_inner),
                            cancel,
                        )
                    }
                    _ => Ok(()),
                })
        }
    };
    let transcript = transcript
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner);
    (transcript, transcribed)
}

/// Avvisa che la Trascrizione dal vivo si è fermata, a meno che non sia stata annullata (anche
/// perché la Registrazione non è partita).
fn live_failed(app: &AppHandle, error: &AppError, cancel: &CancelToken) {
    if cancel.is_cancelled() {
        return;
    }
    let failed = LiveTranscriptionFailed {
        error: error.clone(),
    };
    if let Err(e) = failed.emit(app) {
        log::warn!("live-transcription-failed non emesso: {e}");
    }
}

/// L'esito della Trascrizione dal vivo della Registrazione `recording`, finita la coda: il documento
/// prende il nome del file. Il testo è già nel Bino; se il Bino non si è scritto (`recording` è
/// l'Ogg) si salva nel Markdown accanto, a meno che la Trascrizione (`transcribed`) non sia stata
/// annullata o guasta.
pub fn finish_live(
    app: &AppHandle,
    recording: &Path,
    transcript: Transcript,
    transcribed: Result<(), AppError>,
    cancel: &CancelToken,
) -> LiveTranscription {
    let title = title_of(recording);
    app.state::<LastTranscript>()
        .update(|last| last.title.clone_from(&title));
    let settings = app.state::<SettingsStore>().get();
    save_live(
        recording,
        &Transcript {
            title,
            ..transcript
        },
        transcribed,
        cancel,
        &labels(&settings),
    )
}

/// Com'è finita la Trascrizione dal vivo; senza Bino ne salva il documento nel Markdown accanto a
/// `recording`, se non è stata annullata o guasta.
fn save_live(
    recording: &Path,
    transcript: &Transcript,
    transcribed: Result<(), AppError>,
    cancel: &CancelToken,
    labels: &Labels,
) -> LiveTranscription {
    let saved = transcribed.and_then(|()| {
        // Annulla premuto dopo l'ultima Frase.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        if transcript.phrases.is_empty() {
            return Ok(LiveTranscription::NoSpeech);
        }
        if !bino::is_bino(recording) {
            let markdown = transcript::render(transcript, labels, CopiaCome::Markdown);
            save_md(recording, &markdown)?;
        }
        Ok(LiveTranscription::Saved)
    });
    saved.unwrap_or_else(|error| LiveTranscription::Failed { error })
}

/// Apre il Bino `source` come Sorgente, senza ritrascrivere: le Frasi, come se arrivassero da una
/// Trascrizione, i nomi dei Parlanti e le informazioni.
pub fn open_bino(source: &Path) -> Result<OpenedBino, AppError> {
    let document = bino::read(source)?;
    let phrases = document
        .frasi
        .iter()
        .map(|frase| TranscriptPhrase {
            phrase_id: frase.id,
            inizio_ms: frase.inizio_ms,
            fine_ms: frase.fine_ms,
            text: frase.testo.clone(),
            ingresso: frase.ingresso,
            parlante: frase.parlante,
        })
        .collect();
    Ok(OpenedBino {
        phrases,
        info: BinoInfo {
            creato: document.creato.clone(),
            durata_ms: document.durata_ms,
            modello: document.modello.as_deref().map(model_name),
            lingua_parlato: document.lingua_parlato,
            ingressi_separati: document.modalita == bino::Modalita::IngressiSeparati,
            completa: document.completa,
            origine: document.origine,
        },
        parlanti: document.parlanti,
    })
}

/// Il nome del modello con id `id` nel catalogo; un id sconosciuto resta com'è.
fn model_name(id: &str) -> String {
    models::find(id).map_or_else(|| id.to_string(), |m| m.name.clone())
}

/// Il documento di un Bino come Trascrizione, con il nome del modello dal catalogo.
fn bino_transcript(title: String, document: bino::Document) -> Transcript {
    Transcript {
        title,
        date: document.date(),
        durata_ms: Some(document.durata_ms),
        model: document
            .modello
            .as_deref()
            .map(model_name)
            .unwrap_or_default(),
        speech_language: document.lingua_parlato,
        phrases: document
            .frasi
            .into_iter()
            .map(|frase| Phrase {
                inizio_ms: frase.inizio_ms,
                fine_ms: frase.fine_ms,
                text: frase.testo,
                ingresso: frase.ingresso,
                parlante: frase.parlante,
            })
            .collect(),
        parlanti: document.parlanti,
    }
}

fn silero_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .resolve(SILERO_RESOURCE, BaseDirectory::Resource)
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Esegue `run` con `engine` e Silero, traduce gli eventi della pipeline in
/// `transcription-progress`, `transcript-partial` e `transcript-phrase` con `ingresso`, e
/// inserisce le Frasi in `transcript` in ordine di inizio, e alla fine la durata. Aggiorna anche
/// `LastTranscript` a ogni Frase. Con gli Ingressi separati girano due pipeline sullo stesso
/// `transcript`.
fn run_pipeline(
    app: &AppHandle,
    engine: &mut TranscribeCpp,
    silero: &Path,
    cancel: &CancelToken,
    ingresso: Ingresso,
    transcript: &Mutex<Transcript>,
    run: impl FnOnce(
        &mut dyn TranscriptionEngine,
        &mut dyn VoiceDetector,
        &mut dyn FnMut(PipelineEvent),
    ) -> Result<u32, AppError>,
) -> Result<(), AppError> {
    engine.set_cancel_token(cancel);
    let last = app.state::<LastTranscript>();
    let update = |change: &dyn Fn(&mut Transcript)| {
        last.update(change);
        change(&mut transcript.lock().unwrap_or_else(PoisonError::into_inner));
    };
    Silero::new(silero).and_then(|mut detector| {
        run(engine, &mut detector, &mut |event| {
            let emitted = match event {
                PipelineEvent::Progress(percent) => TranscriptionProgress { percent }.emit(app),
                PipelineEvent::Partial {
                    id,
                    inizio_ms,
                    fine_ms,
                    text,
                } => TranscriptPartial {
                    phrase_id: id,
                    inizio_ms,
                    fine_ms,
                    text,
                    ingresso,
                }
                .emit(app),
                PipelineEvent::Phrase {
                    id,
                    inizio_ms,
                    fine_ms,
                    text,
                } => {
                    let emitted = TranscriptPhrase {
                        phrase_id: id,
                        inizio_ms,
                        fine_ms,
                        text: text.clone(),
                        ingresso,
                        parlante: None,
                    }
                    .emit(app);
                    let phrase = Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        ingresso,
                        parlante: None,
                    };
                    update(&|t| t.insert(phrase.clone()));
                    emitted
                }
            };
            if let Err(e) = emitted {
                log::warn!("evento della Trascrizione non emesso: {e}");
            }
        })
        // Gli Ingressi separati ricevono lo stesso audio: vale la durata più lunga.
        .map(|durata_ms| update(&|t| t.durata_ms = t.durata_ms.max(Some(durata_ms))))
    })
}

/// Se il motore torna a `Models` dopo `transcribed`: dopo un guasto interno si scarta e alla volta
/// successiva si ricarica.
fn keep_engine(transcribed: &Result<(), AppError>) -> bool {
    !matches!(transcribed, Err(AppError::Internal(_)))
}

/// Carica in background il modello scelto, se non è già quello tenuto, così la prossima
/// Trascrizione parte subito e le sue lingue sono note. Chiamata all'avvio, quando cambia il
/// modello scelto e quando finisce un download.
pub fn preload(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let models = app.state::<Models>();
        // Il modello si legge quando il motore è libero: vince l'ultima scelta.
        let settings = app.state::<SettingsStore>();
        match models.take(&app, || settings.model().id.as_str()) {
            Ok(engine) => models.release(&app, engine, true),
            // Un modello non scaricato non è un errore: lo dirà Trascrivi.
            Err(AppError::ModelMissing(_)) => {}
            Err(e) => log::warn!("caricamento del modello scelto: {e}"),
        }
    });
}

/// Scrive `text` nel primo `<stem> trascrizione <N>.md` libero accanto alla Sorgente.
fn save_md(source: &Path, text: &str) -> Result<PathBuf, AppError> {
    let (path, mut file) = loop {
        let path = md_path(source, |p| p.exists());
        // `create_new`: un file comparso dopo il controllo non si sovrascrive, si passa al prossimo N.
        match std::fs::File::create_new(&path) {
            Ok(file) => break (path, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(unwritable(&path, &e)),
        }
    };
    file.write_all(text.as_bytes())
        .map_err(|e| unwritable(&path, &e))?;
    Ok(path)
}

fn unwritable(path: &Path, e: &std::io::Error) -> AppError {
    AppError::UnwritableFolder(format!("{}: {e}", path.display()))
}

/// `<stem della Sorgente> trascrizione <N>.md` accanto alla Sorgente, con N il primo libero da 1.
fn md_path(source: &Path, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let stem = title_of(source);
    (1..)
        .map(|n| source.with_file_name(format!("{stem} trascrizione {n}.md")))
        .find(|path| !exists(path))
        .expect("i numeri non finiscono")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::ogg_opus::tests::decoded_seconds;
    use crate::engine::pipeline::tests::{EnergyDetector, FakeEngine, fixture, wav};
    use crate::managers::settings::Channels;

    fn transcript(phrases: &[&str]) -> Transcript {
        Transcript {
            title: "Riunione".into(),
            date: "2026-10-03 17:05".into(),
            durata_ms: Some(4000),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::auto(),
            phrases: phrases
                .iter()
                .map(|text| Phrase {
                    inizio_ms: 0,
                    fine_ms: 1000,
                    text: (*text).into(),
                    ingresso: Ingresso::Mix,
                    parlante: None,
                })
                .collect(),
            parlanti: BTreeMap::new(),
        }
    }

    fn labels() -> Labels {
        Labels::of(Language::It)
    }

    #[test]
    fn la_data_di_un_file_e_la_sua_data_di_modifica() {
        let dir = crate::audio_toolkit::ogg_opus::tests::temp_dir("data-del-file");
        let path = dir.join("Lezione.wav");
        use chrono::TimeZone;
        let file = std::fs::File::create(&path).unwrap();
        let when = chrono::Local
            .with_ymd_and_hms(2024, 3, 9, 8, 15, 0)
            .unwrap();
        file.set_modified(when.into()).unwrap();
        drop(file);
        assert_eq!(modified_at(&path), Some(when));
        assert_eq!(modified_at(&dir.join("assente.wav")), None);
    }

    #[test]
    fn il_markdown_prende_il_primo_numero_libero_da_1() {
        let source = Path::new(r"C:\Lezioni\Lezione 1.mp4");
        let taken = |names: &'static [&str]| {
            move |p: &Path| names.iter().any(|n| p == Path::new(r"C:\Lezioni").join(n))
        };
        assert_eq!(
            md_path(source, taken(&[])),
            Path::new(r"C:\Lezioni\Lezione 1 trascrizione 1.md")
        );
        assert_eq!(
            md_path(
                source,
                taken(&["Lezione 1 trascrizione 1.md", "Lezione 1 trascrizione 2.md"])
            ),
            Path::new(r"C:\Lezioni\Lezione 1 trascrizione 3.md")
        );
        // Un buco nella numerazione si riempie, e i vecchi TXT non contano.
        assert_eq!(
            md_path(
                source,
                taken(&[
                    "Lezione 1 trascrizione 2.md",
                    "Lezione 1 trascrizione 1.txt"
                ])
            ),
            Path::new(r"C:\Lezioni\Lezione 1 trascrizione 1.md")
        );
    }

    #[test]
    fn il_nome_del_markdown_toglie_solo_l_ultima_estensione() {
        assert_eq!(
            md_path(Path::new(r"D:\a\intervista.v2.mkv"), |_| false),
            Path::new(r"D:\a\intervista.v2 trascrizione 1.md")
        );
    }

    #[test]
    fn salvare_due_volte_non_sovrascrive() {
        let dir = temp_dir("sbobino-test-md");
        let source = dir.join("Riunione.ogg");
        let first = save_md(&source, "Perché sì.\nVa bene.").unwrap();
        let second = save_md(&source, "altro").unwrap();
        assert_eq!(first, dir.join("Riunione trascrizione 1.md"));
        assert_eq!(second, dir.join("Riunione trascrizione 2.md"));
        assert_eq!(
            std::fs::read_to_string(&first).unwrap(),
            "Perché sì.\nVa bene."
        );
    }

    #[test]
    fn la_trascrizione_dal_vivo_salva_il_markdown_solo_senza_bino() {
        let dir = temp_dir("sbobino-test-dal-vivo");
        let document = transcript(&["Uno."]);
        let cancel = CancelToken::new();
        let live = |recording: &str, transcribed, document: &Transcript| {
            save_live(
                &dir.join(recording),
                document,
                transcribed,
                &cancel,
                &labels(),
            )
        };
        // Il testo è nel Bino.
        assert_eq!(
            live("Registrazione.bino", Ok(()), &document),
            LiveTranscription::Saved
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        assert_eq!(
            live("Registrazione.ogg", Ok(()), &transcript(&[])),
            LiveTranscription::NoSpeech
        );
        let guasta = live(
            "Registrazione.ogg",
            Err(AppError::Internal("x".into())),
            &document,
        );
        assert!(matches!(
            guasta,
            LiveTranscription::Failed {
                error: AppError::Internal(_)
            }
        ));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        // Senza Bino, accanto all'Ogg.
        assert_eq!(
            live("Registrazione.ogg", Ok(()), &document),
            LiveTranscription::Saved
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("Registrazione trascrizione 1.md")).unwrap(),
            transcript::render(&document, &labels(), CopiaCome::Markdown)
        );
        // Annulla dopo l'ultima Frase: niente secondo Markdown.
        cancel.cancel();
        assert_eq!(
            live("Registrazione.ogg", Ok(()), &document),
            LiveTranscription::Failed {
                error: AppError::Cancelled
            }
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    }

    #[test]
    fn copia_testo_rende_l_ultima_trascrizione_nel_formato_delle_impostazioni() {
        let last = LastTranscript::default();
        let mut settings = Settings {
            interface_language: Some(Language::It),
            ..Settings::default()
        };
        assert_eq!(transcript_text(&last, &settings), None);
        last.set(transcript(&["Uno."]));
        last.update(|t| t.title = "Lezione".into());
        let text = transcript_text(&last, &settings).unwrap();
        assert!(text.starts_with("Lezione\n\nData: "), "{text}");
        settings.copia_come = CopiaCome::Markdown;
        let text = transcript_text(&last, &settings).unwrap();
        assert!(text.starts_with("# Lezione\n\n- **Data:** "), "{text}");
    }

    #[test]
    fn il_testo_di_un_bino_diventa_la_trascrizione_con_il_nome_del_modello() {
        let phrases = transcript(&["Uno.", "Due."]).phrases;
        let mut document = bino::Document::new(
            "2026-10-03T17:05:42+02:00".into(),
            4000,
            bino::Modalita::Mix,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            false,
            &phrases,
        );
        document.parlanti.insert("mix:1".into(), "Mario".into());
        assert_eq!(
            bino_transcript("Registrazione".into(), document.clone()),
            Transcript {
                title: "Registrazione".into(),
                date: "2026-10-03 17:05".into(),
                durata_ms: Some(4000),
                model: models::default_model().name.clone(),
                speech_language: SpeechLanguage::from("it"),
                phrases,
                parlanti: document.parlanti.clone(),
            }
        );
        // Senza Trascrizione dal vivo il modello non c'è; un id sconosciuto resta com'è.
        for (modello, model) in [(None, ""), (Some("futuro"), "futuro")] {
            let document = bino::Document {
                modello: modello.map(Into::into),
                ..document.clone()
            };
            assert_eq!(bino_transcript(String::new(), document).model, model);
        }
    }

    #[test]
    fn un_bino_aperto_porta_frasi_nomi_e_informazioni() {
        let dir = temp_dir("sbobino-test-apri");
        let ogg = dir.join("mix.ogg");
        std::fs::write(&ogg, b"audio").unwrap();
        let mut phrases = transcript(&["Ciao.", "Salve."]).phrases;
        phrases[1].ingresso = Ingresso::Sistema;
        phrases[1].parlante = Some(2);
        let mut document = bino::Document::new(
            "2026-10-03T17:05:42+02:00".into(),
            4000,
            bino::Modalita::IngressiSeparati,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            false,
            &phrases,
        );
        document.origine = Some("Call.mp4".into());
        document.parlanti.insert("sistema:2".into(), "Lucia".into());
        let path = dir.join("Call.bino");
        bino::write(&path, &[(Ingresso::Mix, &ogg)], &document, None).unwrap();
        let opened = open_bino(&path).unwrap();
        assert_eq!(
            opened.info,
            BinoInfo {
                creato: "2026-10-03T17:05:42+02:00".into(),
                durata_ms: 4000,
                modello: Some(models::default_model().name.clone()),
                lingua_parlato: SpeechLanguage::from("it"),
                ingressi_separati: true,
                completa: false,
                origine: Some("Call.mp4".into()),
            }
        );
        assert_eq!(opened.parlanti, document.parlanti);
        let ids: Vec<_> = opened
            .phrases
            .iter()
            .map(|p| (p.phrase_id, p.ingresso, p.parlante, p.text.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                (0, Ingresso::Mix, None, "Ciao."),
                (1, Ingresso::Sistema, Some(2), "Salve.")
            ]
        );
        // Copia testo ed Esporta Markdown… rendono il Bino con i nomi dei Parlanti.
        let settings = Settings {
            interface_language: Some(Language::It),
            ..Settings::default()
        };
        let markdown = bino_text(&path, &settings, CopiaCome::Markdown).unwrap();
        assert!(markdown.starts_with("# Call\n"), "{markdown}");
        assert!(
            markdown.ends_with("**Audio di sistema · Lucia:** Salve.\n"),
            "{markdown}"
        );
    }

    #[test]
    fn una_cartella_non_scrivibile_da_errore_dedicato() {
        let source = std::env::temp_dir().join("sbobino-test-non-esiste/Audio.wav");
        let error = save_md(&source, "testo").unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
    }

    /// Una cartella vuota per un test.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Il Bino del file `source` nella Raccolta `destination` della Libreria `library`, trascritto
    /// con il motore finto.
    fn file_bino(
        library: &Path,
        destination: &Path,
        source: &Path,
        settings: &Settings,
        engine: &mut FakeEngine,
        cancel: &CancelToken,
    ) -> Result<Option<PathBuf>, AppError> {
        file_to_bino(library, destination, source, settings, |copy| {
            let mut phrases = Vec::new();
            let durata_ms = transcribe_file(
                source,
                engine,
                &mut EnergyDetector,
                None,
                None,
                Some(copy),
                cancel,
                &mut |event| {
                    if let PipelineEvent::Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        ..
                    } = event
                    {
                        phrases.push(Phrase {
                            inizio_ms,
                            fine_ms,
                            text,
                            ingresso: Ingresso::Mix,
                            parlante: None,
                        });
                    }
                },
            )?;
            Ok(bino::Document::new(
                "2026-10-04T10:15:00+02:00".into(),
                durata_ms,
                bino::Modalita::Mix,
                None,
                SpeechLanguage::auto(),
                true,
                &phrases,
            ))
        })
    }

    /// Canali e frequenza dichiarati nell'`OpusHead` del mix del Bino.
    fn opus_head(path: &Path) -> (u8, u32) {
        use std::io::Read;
        let mut mix = Vec::new();
        bino::Mix::open(path)
            .unwrap()
            .read_to_end(&mut mix)
            .unwrap();
        let at = mix.windows(8).position(|w| w == b"OpusHead").unwrap();
        let rate = u32::from_le_bytes(mix[at + 12..at + 16].try_into().unwrap());
        (mix[at + 9], rate)
    }

    /// I file e le cartelle sotto `dir`, relativi.
    fn tree(dir: &Path) -> Vec<String> {
        let mut found = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                found.extend(tree(&path).into_iter().map(|p| format!("{name}/{p}")));
            }
            found.push(name);
        }
        found.sort();
        found
    }

    #[test]
    fn un_file_diventa_un_bino_nella_raccolta_con_l_audio_nel_formato_della_registrazione() {
        for (name, rate, channels) in [
            ("parlato-it.mp4", 16_000, Channels::Mono),
            ("parlato-it.wav", 48_000, Channels::Stereo),
        ] {
            let library = temp_dir("sbobino-test-file-bino");
            let raccolta = library.join("Acme");
            std::fs::create_dir(&raccolta).unwrap();
            let settings = Settings {
                sample_rate: rate,
                channels,
                ..Settings::default()
            };
            let source = fixture(name);
            let bino = |engine: &mut FakeEngine| {
                file_bino(
                    &library,
                    &raccolta,
                    &source,
                    &settings,
                    engine,
                    &CancelToken::new(),
                )
                .unwrap()
                .unwrap()
            };
            let path = bino(&mut FakeEngine::default());
            assert_eq!(path, raccolta.join("parlato-it.bino"));
            // Il mix ha la durata dell'originale, nel formato chiesto.
            let (original, mix) = (decoded_seconds(&source), decoded_seconds(&path));
            assert!(
                (original - mix).abs() < 0.001,
                "{name}: {original} s, {mix} s"
            );
            assert_eq!(
                opus_head(&path),
                (channel_count(channels) as u8, rate),
                "{name}"
            );
            // La Forma d'onda c'è già, uguale a quella ricalcolata dal mix.
            let forma_onda = bino::forma_onda(&path).unwrap();
            let decoded = crate::audio_toolkit::decode::peaks(&path, 1000).unwrap();
            assert_eq!(forma_onda.len(), decoded.len(), "{name}");
            assert!(
                forma_onda
                    .iter()
                    .zip(&decoded)
                    .all(|(f, d)| (f - d).abs() < 0.1),
                "{name}: {forma_onda:?} contro {decoded:?}"
            );
            let document = bino::read(&path).unwrap();
            assert!(!document.frasi.is_empty(), "{name}");
            assert!(
                document
                    .frasi
                    .iter()
                    .all(|f| f.inizio_ms < f.fine_ms && f.fine_ms <= document.durata_ms),
                "{name}: {:?}",
                document.frasi
            );
            // Lo stesso file un'altra volta: un nome nuovo, e niente temporanei.
            let again = bino(&mut FakeEngine::default());
            assert_eq!(again, raccolta.join("parlato-it 2.bino"));
            assert_eq!(
                tree(&library),
                ["Acme", "Acme/parlato-it 2.bino", "Acme/parlato-it.bino"]
            );
        }
    }

    #[test]
    fn annullata_o_senza_parlato_non_resta_nessun_file() {
        let library = temp_dir("sbobino-test-file-annullato");
        let settings = Settings::default();
        let cancel = CancelToken::new();
        let mut engine = FakeEngine {
            cancel_at: Some((1, cancel.clone())),
            ..FakeEngine::default()
        };
        let source = fixture("parlato-it.wav");
        let error =
            file_bino(&library, &library, &source, &settings, &mut engine, &cancel).unwrap_err();
        assert_eq!(error, AppError::Cancelled);
        assert!(tree(&library).is_empty(), "{:?}", tree(&library));
        let silence = wav("silenzio-bino", 16_000, 1, &[(2.0, false)]);
        let none = file_bino(
            &library,
            &library,
            &silence,
            &settings,
            &mut FakeEngine::default(),
            &CancelToken::new(),
        )
        .unwrap();
        assert_eq!(none, None);
        assert!(tree(&library).is_empty(), "{:?}", tree(&library));
    }

    #[test]
    fn senza_la_raccolta_il_bino_va_nella_radice() {
        let library = temp_dir("sbobino-test-file-radice");
        let path = file_bino(
            &library,
            &library.join("Sparita"),
            &fixture("parlato-it.wav"),
            &Settings::default(),
            &mut FakeEngine::default(),
            &CancelToken::new(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(path, library.join("parlato-it.bino"));
    }

    #[test]
    fn i_parlanti_arrivano_con_l_id_della_frase_nel_suo_ingresso() {
        let phrase = |ingresso, parlante| Phrase {
            inizio_ms: 0,
            fine_ms: 0,
            text: String::new(),
            ingresso,
            parlante,
        };
        let phrases = [
            phrase(Ingresso::Microfono, None),
            phrase(Ingresso::Sistema, Some(1)),
            phrase(Ingresso::Sistema, Some(2)),
            phrase(Ingresso::Microfono, None),
            phrase(Ingresso::Sistema, Some(1)),
        ];
        let ids: Vec<_> = assignments(&phrases)
            .iter()
            .map(|s| (s.ingresso, s.phrase_id, s.parlante))
            .collect();
        assert_eq!(
            ids,
            [
                (Ingresso::Microfono, 0, None),
                (Ingresso::Sistema, 0, Some(1)),
                (Ingresso::Sistema, 1, Some(2)),
                (Ingresso::Microfono, 1, None),
                (Ingresso::Sistema, 2, Some(1)),
            ]
        );
    }
}
