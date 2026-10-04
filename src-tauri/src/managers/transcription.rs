//! Trascrizione di una Sorgente, o dal vivo di una Registrazione (il mix, o con gli Ingressi separati
//! ogni Ingresso con una sua pipeline): prende il motore del modello scelto (caricato una volta e
//! tenuto tra una Trascrizione e l'altra), esegue la pipeline, la traduce in eventi e salva il
//! Markdown accanto alla Sorgente; di un Bino riscrive anche il testo dentro il Bino. Tiene l'ultima
//! Trascrizione per Copia testo e per la rinomina dei Parlanti.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::vad::{Silero, VoiceDetector};
use crate::bino;
use crate::engine::live::LiveFrames;
use crate::engine::pipeline::{self, Feed, PipelineEvent, transcribe_file};
use crate::engine::transcribe_cpp::TranscribeCpp;
use crate::engine::{TranscriptionEngine, diarize};
use crate::error::AppError;
use crate::library;
use crate::managers::activity::Activity;
use crate::managers::models::{self, DiarizerLease, Models};
use crate::managers::settings::{CopiaCome, Language, Settings, SettingsStore};
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

/// Dove è il Markdown salvato e quanti caratteri contiene.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionFinished {
    pub md_path: String,
    pub chars: u32,
}

/// Esito di una Trascrizione arrivata alla fine della Sorgente. Annulla e i guasti sono `AppError`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum TranscriptionOutcome {
    /// Il testo è salvato nel Markdown.
    Saved(TranscriptionFinished),
    /// Nessuna Frase: il Markdown non si crea.
    NoSpeech,
}

/// Com'è finita la Trascrizione dal vivo di una Registrazione salvata.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum LiveTranscription {
    /// Il testo è salvato nel Markdown accanto alla Registrazione.
    Saved(TranscriptionFinished),
    /// Nessuna Frase: il Markdown non si crea.
    NoSpeech,
    /// Senza Markdown: modello assente (`liveTranscriptionUnavailable`), guasto o Annulla
    /// (`cancelled`).
    Failed { error: AppError },
}

/// Un Bino aperto come Sorgente: le Frasi per l'area e i nomi dei Parlanti.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct OpenedBino {
    pub phrases: Vec<TranscriptPhrase>,
    /// Per chiave `<ingresso>:<n>`, come nel Bino.
    pub parlanti: BTreeMap<String, String>,
}

/// L'ultima Trascrizione, di un file o dal vivo, anche annullata: quella che Copia testo rende e i
/// cui Parlanti si rinominano. Si riempie man mano che arrivano le Frasi. In `tauri::State`.
#[derive(Default)]
pub struct LastTranscript(Mutex<Last>);

#[derive(Default)]
struct Last {
    transcript: Option<Transcript>,
    /// Il Markdown prodotto con queste Frasi, che la rinomina riscrive.
    md: Option<PathBuf>,
    /// Il Bino che contiene queste Frasi, dove la rinomina salva i nomi.
    bino: Option<PathBuf>,
}

impl LastTranscript {
    fn lock(&self) -> std::sync::MutexGuard<'_, Last> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Modifica la Trascrizione tenuta, se c'è.
    fn update(&self, change: impl FnOnce(&mut Transcript)) {
        if let Some(transcript) = self.lock().transcript.as_mut() {
            change(transcript);
        }
    }

    /// Una Trascrizione nuova, ancora senza Markdown né Bino.
    fn set(&self, transcript: Transcript) {
        *self.lock() = Last {
            transcript: Some(transcript),
            ..Last::default()
        };
    }

    /// Dove sono finite le Frasi della Trascrizione tenuta: il Markdown e il Bino, se ci sono.
    fn saved_in(&self, md: Option<PathBuf>, bino: Option<PathBuf>) {
        let mut last = self.lock();
        last.md = md;
        last.bino = bino;
    }

    fn get(&self) -> Option<Transcript> {
        self.lock().transcript.clone()
    }

    /// Il Bino, o la Raccolta, `from` è stato spostato o rinominato in `to`: la rinomina dei
    /// Parlanti cerca lì il Bino che contiene queste Frasi.
    pub fn moved(&self, from: &Path, to: &Path) {
        let mut last = self.lock();
        if let Some(bino) = &last.bino
            && library::inside(bino, from)
        {
            let rest = bino
                .components()
                .skip(from.components().count())
                .collect::<PathBuf>();
            last.bino = Some(to.join(rest));
        }
    }
}

/// Rinomina il Parlante `parlante` di `ingresso` nell'ultima Trascrizione: il nome (senza spazi in
/// testa e in coda, non vuoto) vale per tutte le sue Frasi, si salva nel Bino che le contiene e
/// riscrive il Markdown che hanno prodotto, se c'è ancora.
pub fn rename_parlante(
    last: &LastTranscript,
    ingresso: Ingresso,
    parlante: u32,
    nome: &str,
    labels: &Labels,
) -> Result<(), AppError> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Err(AppError::Internal("nome del Parlante vuoto".into()));
    }
    let mut last = last.lock();
    let Last {
        transcript: Some(transcript),
        md,
        bino,
    } = &mut *last
    else {
        return Err(AppError::Internal(
            "nessuna Trascrizione da rinominare".into(),
        ));
    };
    let key = ingresso.parlante_key(parlante);
    let mut renamed = transcript.clone();
    renamed.parlanti.insert(key.clone(), nome.to_string());
    if let Some(bino) = bino {
        let mut document = bino::read(bino)?;
        document.parlanti.insert(key, nome.to_string());
        bino::rewrite(bino, &document)?;
    }
    // Il nome è salvato dove si rilegge: anche se il Markdown non si scrive, vale.
    *transcript = renamed;
    // Un Markdown cancellato nel frattempo non si ricrea.
    match md {
        Some(md) if md.exists() => replace_file(
            md,
            &transcript::render(transcript, labels, CopiaCome::Markdown),
        ),
        _ => Ok(()),
    }
}

/// Sostituisce `path` con `text` scrivendo `<nome>.tmp` e rinominandolo sopra: a metà, il file
/// resta com'era.
fn replace_file(path: &Path, text: &str) -> Result<(), AppError> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    let written = std::fs::write(&temp, text).and_then(|()| std::fs::rename(&temp, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written.map_err(|e| unwritable(path, &e))
}

/// Il testo di Copia testo: l'ultima Trascrizione in testo semplice o in Markdown, come dicono le
/// impostazioni. `None` se non ce n'è ancora una.
pub fn transcript_text(last: &LastTranscript, settings: &Settings) -> Option<String> {
    last.get()
        .map(|t| transcript::render(&t, &labels(settings), settings.copia_come))
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
        speech_language: settings.speech_language,
        phrases: Vec::new(),
        parlanti: BTreeMap::new(),
    };
    app.state::<LastTranscript>().set(transcript.clone());
    transcript
}

/// Il nome della Sorgente senza l'ultima estensione: il titolo del documento.
fn title_of(source: &Path) -> String {
    source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Trascrive `source` emettendo `transcription-progress`, `transcript-partial` e
/// `transcript-phrase`; con Riconosci i parlanti poi diarizza (`diarization-started`,
/// `speakers-assigned`). Infine salva il Markdown.
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn transcribe(
    app: AppHandle,
    activity: &Activity,
    source: PathBuf,
) -> Result<TranscriptionOutcome, AppError> {
    let cancel = CancelToken::new();
    // Su un Bino le scritture di altri comandi si rifiutano finché non è riscritto.
    let _activity = activity.begin(bino::is_bino(&source).then(|| source.clone()), {
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
    let transcript = Mutex::new(begin_transcript(&app, title_of(&source), &settings));
    tauri::async_runtime::spawn_blocking(move || {
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
        // Annulla premuto dopo l'ultima Frase: né il Bino né il Markdown cambiano.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        let rewritten = bino.is_some();
        if let Some(old) = bino {
            let document = bino::Document::new(
                old.creato,
                old.durata_ms,
                bino::Modalita::Mix,
                Some(model.id.clone()),
                settings.speech_language,
                true,
                &transcript.phrases,
            );
            bino::rewrite(&source, &document)?;
        }
        let outcome = save_transcript(&source, &transcript, &labels(&settings))?;
        app.state::<LastTranscript>()
            .saved_in(md_of(&outcome), rewritten.then_some(source));
        Ok(outcome)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
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
/// prende il nome del file e si salva nel Markdown accanto, a meno che la Trascrizione
/// (`transcribed`) non sia stata annullata o guasta.
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
    let live = save_live(
        recording,
        &Transcript {
            title,
            ..transcript
        },
        transcribed,
        cancel,
        &labels(&settings),
    );
    let md = match &live {
        LiveTranscription::Saved(finished) => Some(PathBuf::from(&finished.md_path)),
        _ => None,
    };
    // Il Bino ha le Frasi anche se la Trascrizione è annullata o guasta.
    app.state::<LastTranscript>().saved_in(
        md,
        bino::is_bino(recording).then(|| recording.to_path_buf()),
    );
    live
}

/// Salva il documento della Trascrizione dal vivo, se non è stata annullata o guasta.
fn save_live(
    recording: &Path,
    transcript: &Transcript,
    transcribed: Result<(), AppError>,
    cancel: &CancelToken,
    labels: &Labels,
) -> LiveTranscription {
    let saved = transcribed.and_then(|()| {
        // Annulla premuto dopo l'ultima Frase: il Markdown non si salva lo stesso.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        save_transcript(recording, transcript, labels)
    });
    match saved {
        Ok(TranscriptionOutcome::Saved(finished)) => LiveTranscription::Saved(finished),
        Ok(TranscriptionOutcome::NoSpeech) => LiveTranscription::NoSpeech,
        Err(error) => LiveTranscription::Failed { error },
    }
}

/// Apre il Bino `source` come Sorgente: il suo testo diventa l'ultima Trascrizione (se non c'è
/// un'Attività in corso), senza ritrascrivere, e la rinomina dei Parlanti riscrive il Bino e il suo
/// Markdown più recente.
/// Restituisce le Frasi per l'area, come se arrivassero da una Trascrizione, e i nomi dei Parlanti.
pub fn open_bino(app: &AppHandle, source: &Path) -> Result<OpenedBino, AppError> {
    let document = bino::read(source)?;
    let parlanti = document.parlanti.clone();
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
    // Durante un'Attività il Bino si consulta soltanto: l'ultima Trascrizione resta la sua.
    if !app.state::<Activity>().is_running() {
        let last = app.state::<LastTranscript>();
        last.set(bino_transcript(title_of(source), document));
        last.saved_in(latest_md(source), Some(source.to_path_buf()));
    }
    Ok(OpenedBino { phrases, parlanti })
}

/// Il `<stem> trascrizione <N>.md` accanto alla Sorgente modificato per ultimo, se ce n'è uno.
fn latest_md(source: &Path) -> Option<PathBuf> {
    let prefix = format!("{} trascrizione ", title_of(source));
    std::fs::read_dir(source.parent()?)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix(&prefix)?.strip_suffix(".md"))
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

/// Il documento di un Bino come Trascrizione, con il nome del modello dal catalogo.
fn bino_transcript(title: String, document: bino::Document) -> Transcript {
    Transcript {
        title,
        date: document.date(),
        durata_ms: Some(document.durata_ms),
        model: document
            .modello
            .map(|id| models::find(&id).map_or(id, |m| m.name.clone()))
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

/// Il Markdown salvato, se c'è.
fn md_of(outcome: &TranscriptionOutcome) -> Option<PathBuf> {
    match outcome {
        TranscriptionOutcome::Saved(finished) => Some(PathBuf::from(&finished.md_path)),
        TranscriptionOutcome::NoSpeech => None,
    }
}

/// Salva il documento in Markdown accanto alla Sorgente; senza Frasi non crea il file.
fn save_transcript(
    source: &Path,
    transcript: &Transcript,
    labels: &Labels,
) -> Result<TranscriptionOutcome, AppError> {
    if transcript.phrases.is_empty() {
        return Ok(TranscriptionOutcome::NoSpeech);
    }
    let markdown = transcript::render(transcript, labels, CopiaCome::Markdown);
    save_md(source, &markdown).map(TranscriptionOutcome::Saved)
}

/// Scrive `text` nel primo `<stem> trascrizione <N>.md` libero accanto alla Sorgente.
fn save_md(source: &Path, text: &str) -> Result<TranscriptionFinished, AppError> {
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
    Ok(TranscriptionFinished {
        md_path: path.display().to_string(),
        chars: u32::try_from(text.chars().count()).unwrap_or(u32::MAX),
    })
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
    use crate::managers::settings::SpeechLanguage;

    fn transcript(phrases: &[&str]) -> Transcript {
        Transcript {
            title: "Riunione".into(),
            date: "2026-10-03 17:05".into(),
            durata_ms: Some(4000),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::Auto,
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
    fn salvare_due_volte_non_sovrascrive_e_conta_i_caratteri() {
        let dir = std::env::temp_dir().join("sbobino-test-md");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("Riunione.mp3");
        let first = save_md(&source, "Perché sì.\nVa bene.").unwrap();
        let second = save_md(&source, "altro").unwrap();
        assert_eq!(
            first,
            TranscriptionFinished {
                md_path: dir.join("Riunione trascrizione 1.md").display().to_string(),
                chars: 19,
            }
        );
        assert_eq!(
            second.md_path,
            dir.join("Riunione trascrizione 2.md").display().to_string()
        );
        assert_eq!(
            std::fs::read_to_string(&first.md_path).unwrap(),
            "Perché sì.\nVa bene."
        );
    }

    #[test]
    fn senza_frasi_non_si_salva_nessun_markdown() {
        let dir = std::env::temp_dir().join("sbobino-test-nessun-parlato");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("Silenzio.wav");
        assert_eq!(
            save_transcript(&source, &transcript(&[]), &labels()).unwrap(),
            TranscriptionOutcome::NoSpeech
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let document = transcript(&["Uno.", "Due."]);
        let saved = save_transcript(&source, &document, &labels()).unwrap();
        let TranscriptionOutcome::Saved(finished) = saved else {
            panic!("{saved:?}");
        };
        assert_eq!(
            std::fs::read_to_string(finished.md_path).unwrap(),
            transcript::render(&document, &labels(), CopiaCome::Markdown)
        );
    }

    #[test]
    fn la_trascrizione_dal_vivo_annullata_o_guasta_non_salva_il_markdown() {
        let dir = std::env::temp_dir().join("sbobino-test-dal-vivo");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let recording = dir.join("Registrazione.bino");
        let document = transcript(&["Uno."]);
        let cancel = CancelToken::new();
        let guasta = save_live(
            &recording,
            &document,
            Err(AppError::Internal("x".into())),
            &cancel,
            &labels(),
        );
        assert!(matches!(
            guasta,
            LiveTranscription::Failed {
                error: AppError::Internal(_)
            }
        ));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let saved = save_live(&recording, &document, Ok(()), &cancel, &labels());
        assert!(matches!(saved, LiveTranscription::Saved(_)), "{saved:?}");
        // Annulla dopo l'ultima Frase: niente secondo Markdown.
        cancel.cancel();
        assert_eq!(
            save_live(&recording, &document, Ok(()), &cancel, &labels()),
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
            SpeechLanguage::It,
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
                speech_language: SpeechLanguage::It,
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

    #[test]
    fn il_bino_spostato_o_nella_raccolta_rinominata_si_ritrova() {
        let last = LastTranscript::default();
        last.set(transcript(&["Uno."]));
        let bino = Path::new(r"C:\Sbobino\Acme\Call.bino");
        last.saved_in(None, Some(bino.to_path_buf()));
        last.moved(Path::new(r"C:\Sbobino\Altro.bino"), Path::new(r"C:\X.bino"));
        assert_eq!(last.lock().bino.as_deref(), Some(bino));
        last.moved(
            Path::new(r"C:\sbobino\acme"),
            Path::new(r"C:\Sbobino\Acme Srl"),
        );
        assert_eq!(
            last.lock().bino.as_deref(),
            Some(Path::new(r"C:\Sbobino\Acme Srl\Call.bino"))
        );
        last.moved(
            Path::new(r"C:\Sbobino\Acme Srl\Call.bino"),
            Path::new(r"C:\Sbobino\Call.bino"),
        );
        assert_eq!(
            last.lock().bino.as_deref(),
            Some(Path::new(r"C:\Sbobino\Call.bino"))
        );
    }

    #[test]
    fn la_rinomina_vale_per_copia_testo_il_bino_e_il_markdown() {
        let dir = temp_dir("sbobino-test-rinomina");
        let ogg = dir.join("mix.ogg");
        std::fs::write(&ogg, b"audio").unwrap();
        let mut document = transcript(&["Ciao.", "Salve."]);
        document.phrases[0].parlante = Some(1);
        document.phrases[1].parlante = Some(2);
        let path = dir.join("Riunione.bino");
        let bino_document = bino::Document::new(
            "2026-10-03T17:05:42+02:00".into(),
            4000,
            bino::Modalita::Mix,
            None,
            SpeechLanguage::Auto,
            true,
            &document.phrases,
        );
        bino::write(&path, &[(Ingresso::Mix, &ogg)], &bino_document).unwrap();
        let TranscriptionOutcome::Saved(saved) =
            save_transcript(&path, &document, &labels()).unwrap()
        else {
            panic!("senza Markdown");
        };
        let last = LastTranscript::default();
        // Senza Trascrizione non c'è niente da rinominare.
        assert!(rename_parlante(&last, Ingresso::Mix, 1, "Mario", &labels()).is_err());
        last.set(document);
        last.saved_in(Some(PathBuf::from(&saved.md_path)), Some(path.clone()));
        rename_parlante(&last, Ingresso::Mix, 1, "  Mario ", &labels()).unwrap();
        // Un nome vuoto si rifiuta e non cambia nulla.
        assert!(rename_parlante(&last, Ingresso::Mix, 2, "  ", &labels()).is_err());
        let settings = Settings {
            interface_language: Some(Language::It),
            ..Settings::default()
        };
        let text = transcript_text(&last, &settings).unwrap();
        assert!(
            text.ends_with("\nMario: Ciao.\n\nParlante 2: Salve.\n"),
            "{text}"
        );
        let md = std::fs::read_to_string(&saved.md_path).unwrap();
        assert!(
            md.ends_with("\n**Mario:** Ciao.\n\n**Parlante 2:** Salve.\n"),
            "{md}"
        );
        let reread = bino::read(&path).unwrap();
        assert_eq!(
            reread.parlanti,
            BTreeMap::from([("mix:1".to_string(), "Mario".to_string())])
        );
        assert_eq!(reread.frasi, bino_document.frasi);
        // Un Markdown cancellato non si ricrea.
        std::fs::remove_file(&saved.md_path).unwrap();
        rename_parlante(&last, Ingresso::Mix, 2, "Lucia", &labels()).unwrap();
        assert!(!Path::new(&saved.md_path).exists());
        assert_eq!(bino::read(&path).unwrap().parlanti.len(), 2);
    }

    #[test]
    fn il_markdown_di_un_bino_e_quello_modificato_per_ultimo() {
        let dir = temp_dir("sbobino-test-ultimo-md");
        let source = dir.join("Riunione.bino");
        assert_eq!(latest_md(&source), None);
        let older = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
        for name in [
            "Riunione trascrizione 2.md",
            "Riunione trascrizione 1.md",
            "Riunione trascrizione 3.txt",
            "Riunione trascrizione x.md",
            "Altro trascrizione 4.md",
        ] {
            std::fs::write(dir.join(name), "").unwrap();
        }
        std::fs::File::options()
            .write(true)
            .open(dir.join("Riunione trascrizione 2.md"))
            .unwrap()
            .set_modified(older)
            .unwrap();
        assert_eq!(
            latest_md(&source),
            Some(dir.join("Riunione trascrizione 1.md"))
        );
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
