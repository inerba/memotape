//! Trascrizione di una Sorgente, o dal vivo di una Registrazione (il mix, o con gli Ingressi separati
//! ogni Ingresso con una sua pipeline): prende il motore del modello scelto (caricato una volta e
//! tenuto tra una Trascrizione e l'altra), esegue la pipeline e la traduce in eventi. Un file audio o
//! video diventa un Tape nella Raccolta, di un Tape si riscrive il testo. Tiene l'ultima Trascrizione
//! per Copia testo finché non c'è un Tape da cui copiare.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::decode::Decoder;
use crate::audio_toolkit::ogg_opus::OggCopy;
use crate::audio_toolkit::vad::{Silero, VoiceDetector};
use crate::engine::live::LiveFrames;
use crate::engine::live_diarization::{LiveDiarizer, LiveTranscript};
use crate::engine::pipeline::{self, PipelineEvent, transcribe_decoded};
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
use crate::tape;
use crate::transcript::{self, Ingresso, Labels, Phrase, Transcript};

const SILERO_RESOURCE: &str = "resources/silero_vad.onnx";

/// Una Frase conclusa, una per riga nell'area di testo. Sostituisce il Parziale con lo stesso id.
/// `inizio_ms` e `fine_ms` sono sulla linea del tempo della Sorgente.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptPhrase {
    #[serde(default)]
    pub session_id: Option<String>,
    pub phrase_id: u32,
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub text: String,
    /// Con gli Ingressi separati ogni Ingresso ha le sue Frasi, con id propri.
    pub ingresso: Ingresso,
    /// Il Parlante, da 1: c'è nelle Frasi di un Tape diarizzato. Durante una Trascrizione arriva
    /// dopo, con `speakers-assigned`.
    pub parlante: Option<u32>,
    #[serde(default)]
    pub parlante_non_determinato: bool,
    #[serde(default)]
    pub parlante_provvisorio: bool,
}

/// Il Parziale della Frase in corso (solo con i modelli in streaming): sostituisce il precedente e
/// ha l'id che avrà la Frase. `fine_ms` è la fine dell'audio letto finora.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptPartial {
    #[serde(default)]
    pub session_id: Option<String>,
    pub phrase_id: u32,
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub text: String,
    pub ingresso: Ingresso,
}

/// La Trascrizione dal vivo si è fermata (modello assente, guasto): la Registrazione continua senza
/// testo.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LiveTranscriptionFailed {
    pub session_id: String,
    pub error: AppError,
}

/// Sostituisce il testo dal vivo di un Ingresso. Revisioni includono sia ASR sia rettifiche;
/// un risultato precedente non può far ricomparire un Parziale ormai concluso.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LiveTranscriptUpdated {
    pub session_id: String,
    pub ingresso: Ingresso,
    pub revision: u32,
    pub finished: bool,
    pub phrases: Vec<TranscriptPhrase>,
    pub partials: Vec<TranscriptPhrase>,
}

pub struct LiveSource {
    pub ingresso: Ingresso,
    pub frames: LiveFrames,
    pub diarizer: Option<LiveDiarizer>,
}

fn transcript_phrase(id: u32, phrase: &Phrase, session_id: Option<&str>) -> TranscriptPhrase {
    TranscriptPhrase {
        session_id: session_id.map(str::to_owned),
        phrase_id: id,
        inizio_ms: phrase.inizio_ms,
        fine_ms: phrase.fine_ms,
        text: phrase.text.clone(),
        ingresso: phrase.ingresso,
        parlante: phrase.parlante,
        parlante_non_determinato: phrase.parlante_non_determinato,
        parlante_provvisorio: phrase.parlante_provvisorio,
    }
}

struct LiveSession<'a> {
    session_id: &'a str,
    app: &'a AppHandle,
    ingresso: Ingresso,
    transcript: &'a Mutex<Transcript>,
    state: Mutex<LiveTranscript>,
}

impl LiveSession<'_> {
    fn run(
        &self,
        engine: &mut TranscribeCpp,
        silero: &Path,
        cancel: &CancelToken,
        speech_language: Option<&str>,
        frames: &mut LiveFrames,
        mut diarizer: LiveDiarizer,
    ) -> Result<(), AppError> {
        let app = self.app;
        let transcript = self.transcript;
        std::thread::scope(|scope| {
            let worker_cancel = diarizer.cancel_token();
            let worker_cancelled = worker_cancel.clone();
            let session_ref = self;
            let worker = scope.spawn(move || {
                let analyzed = diarizer.run(cancel, &mut |turns| session_ref.turns(turns));
                if let Err(error) = analyzed
                    && ((!cancel.is_cancelled() && !worker_cancelled.is_cancelled())
                        || matches!(error, AppError::LiveDiarizationLagging))
                {
                    let error = if matches!(error, AppError::LiveDiarizationLagging) {
                        error
                    } else {
                        AppError::LiveDiarizationUnavailable(error.to_string())
                    };
                    if let Err(e) = (LiveDiarizationFailed {
                        session_id: self.session_id.to_owned(),
                        ingresso: self.ingresso,
                        error,
                    })
                    .emit(app)
                    {
                        log::warn!("guasto dei Parlanti non emesso: {e}");
                    }
                }
            });
            engine.set_cancel_token(cancel);
            let backlog = frames.backlog();
            let result = Silero::new(silero).and_then(|mut detector| {
                pipeline::transcribe(
                    frames,
                    engine,
                    &mut detector,
                    speech_language,
                    cancel,
                    &mut |event| {
                        log::info!("ASR dal vivo: coda={} ms", backlog());
                        self.asr(event);
                    },
                )
            });
            if result.is_err() {
                worker_cancel.cancel();
                let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
                state.clear_partial();
                self.publish(&state);
            }
            if worker.join().is_err() {
                let _ = (LiveDiarizationFailed {
                    session_id: self.session_id.to_owned(),
                    ingresso: self.ingresso,
                    error: AppError::LiveDiarizationUnavailable("stream interrotto".into()),
                })
                .emit(app);
            }
            result.map(|duration| {
                let mut document = transcript.lock().unwrap_or_else(PoisonError::into_inner);
                document.durata_ms = document.durata_ms.max(Some(duration));
            })
        })
    }

    /// Chiamato sotto il lock della sessione, prima di pubblicare la revisione.
    fn publish(&self, state: &LiveTranscript) {
        let phrases: Vec<_> = state
            .phrases
            .iter()
            .map(|(id, phrase)| transcript_phrase(*id, phrase, Some(self.session_id)))
            .collect();
        let partials: Vec<_> = state
            .partials
            .iter()
            .map(|(id, phrase)| transcript_phrase(*id, phrase, Some(self.session_id)))
            .collect();
        let mut transcript = self
            .transcript
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        transcript.phrases.retain(|p| p.ingresso != self.ingresso);
        for (_, phrase) in &state.phrases {
            transcript.insert(phrase.clone());
        }
        transcript.live_asr.retain(|p| p.ingresso != self.ingresso);
        transcript.live_asr.extend(state.originals().cloned());
        let mut visible = transcript.clone();
        for (_, phrase) in &state.partials {
            visible.insert(phrase.clone());
        }
        self.app.state::<LastTranscript>().set(visible);
        if let Err(e) = (LiveTranscriptUpdated {
            session_id: self.session_id.to_owned(),
            ingresso: self.ingresso,
            revision: state.revision,
            finished: false,
            phrases,
            partials,
        })
        .emit(self.app)
        {
            log::warn!("rettifica dal vivo non emessa: {e}");
        }
    }

    fn asr(&self, event: PipelineEvent) {
        if let PipelineEvent::Progress(percent) = event {
            if let Err(e) = (TranscriptionProgress {
                session_id: Some(self.session_id.to_owned()),
                percent,
            })
            .emit(self.app)
            {
                log::warn!("progresso dal vivo non emesso: {e}");
            }
            return;
        }
        if let PipelineEvent::Partial { fine_ms, .. } | PipelineEvent::Phrase { fine_ms, .. } =
            &event
        {
            log::info!("ASR dal vivo: testo disponibile fino a {fine_ms} ms");
        }
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.on_asr(self.ingresso, event);
        self.publish(&state);
    }

    fn turns(&self, turns: Vec<diarize::Turn>) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.on_turns(turns);
        self.publish(&state);
    }
}

/// Il riconoscimento dei Parlanti non è disponibile; audio e ASR continuano.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LiveDiarizationFailed {
    pub session_id: String,
    pub ingresso: Ingresso,
    pub error: AppError,
}

/// Finita la Trascrizione, comincia la Diarizzazione (Riconosci i parlanti).
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DiarizationStarted {
    pub session_id: Option<String>,
}

/// I Parlanti delle Frasi dopo la Diarizzazione: `parlante` da 1 per ordine di comparsa, `null` se
/// nessuno parlava durante la Frase.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct SpeakersAssigned {
    #[serde(default)]
    pub session_id: Option<String>,
    pub speakers: Vec<SpeakerAssignment>,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerAssignment {
    pub ingresso: Ingresso,
    pub phrase_id: u32,
    pub parlante: Option<u32>,
    #[serde(default)]
    pub parlante_non_determinato: bool,
    #[serde(default)]
    pub parlante_provvisorio: bool,
}

/// Avanzamento della Trascrizione, o dello smaltimento della coda dal vivo dopo Stop: `percent` è
/// `null` se la durata della Sorgente non è nota.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionProgress {
    pub session_id: Option<String>,
    pub percent: Option<u8>,
}

/// Esito di una Trascrizione arrivata alla fine della Sorgente. Annulla e i guasti sono `AppError`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum TranscriptionOutcome {
    /// Il testo è nel Tape `path`: quello nuovo di un file, o il Tape trascritto. Diventa la
    /// Sorgente.
    Saved { path: String },
    /// Nessuna Frase: un file non diventa un Tape; un Tape resta senza Frasi.
    NoSpeech,
}

/// Com'è finita la Trascrizione dal vivo di una Registrazione salvata.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum LiveTranscription {
    /// Il testo è nel Tape o, se il Tape non si è scritto, nel Markdown accanto all'Ogg.
    Saved,
    NoSpeech,
    /// Modello assente (`liveTranscriptionUnavailable`), guasto o Annulla (`cancelled`): il Tape ha
    /// le Frasi arrivate, con il testo incompleto.
    Failed {
        error: AppError,
    },
}

/// Un Tape aperto come Sorgente: le Frasi, i nomi dei Parlanti e le informazioni.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct OpenedTape {
    pub phrases: Vec<TranscriptPhrase>,
    /// Per chiave `<ingresso>:<n>`, come nel Tape.
    pub parlanti: BTreeMap<String, String>,
    pub info: TapeInfo,
}

/// La riga di informazioni della vista di un Tape.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TapeInfo {
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
    #[serde(default)]
    pub diarizzazione: Option<crate::transcript::Diarizzazione>,
}

/// L'ultima Trascrizione, di un file o dal vivo, anche annullata: quella che Copia testo rende
/// finché non c'è un Tape aperto. Si riempie man mano che arrivano le Frasi. In `tauri::State`.
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
pub fn transcript_text(
    last: &LastTranscript,
    settings: &Settings,
    visible: Option<&[TranscriptPhrase]>,
) -> Option<String> {
    last.get().map(|mut transcript| {
        // Il pulsante copia lo snapshot visibile al click, anche se il backend è già più avanti.
        if let Some(visible) = visible {
            transcript.phrases = visible
                .iter()
                .map(|p| Phrase {
                    inizio_ms: p.inizio_ms,
                    fine_ms: p.fine_ms,
                    text: p.text.clone(),
                    ingresso: p.ingresso,
                    parlante: p.parlante,
                    parlante_non_determinato: p.parlante_non_determinato,
                    parlante_provvisorio: p.parlante_provvisorio,
                    tempi: Vec::new(),
                })
                .collect();
        }
        transcript::render(&transcript, &labels(settings), settings.copia_come)
    })
}

/// Il Tape `path` reso come documento, con le correzioni e i nomi dei Parlanti, in `format`: per
/// Copia testo e per Esporta Markdown….
pub fn tape_text(path: &Path, settings: &Settings, format: CopiaCome) -> Result<String, AppError> {
    let transcript = tape_transcript(title_of(path), tape::read(path)?);
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
        live_asr: Vec::new(),
        parlanti: BTreeMap::new(),
        diarizzazione: None,
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
/// `speakers-assigned`). Un file audio o video diventa un Tape nella Raccolta `raccolta` (la radice
/// della Libreria per `None` o `""`); di un Tape si riscrive il testo.
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
    // Le scritture di altri comandi su quel Tape, o sulla Raccolta del Tape che nascerà, si
    // rifiutano finché la Trascrizione non è finita.
    let target = if tape::is_tape(&source) {
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
    // Un Tape illeggibile o di una versione più nuova si rifiuta prima di caricare il modello.
    let tape = tape::is_tape(&source)
        .then(|| tape::read(&source))
        .transpose()?;
    let ingressi = ingressi_of(&source)?;
    let separate = ingressi != [Ingresso::Mix];
    // Gli Ingressi da diarizzare: con gli Ingressi separati il Microfono solo a richiesta.
    let diarized = settings.parlanti_trascrivi(separate);
    // Si verifica il modello selezionato prima dell'ASR, fuori dall'esecutore async (hash del GGUF).
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
            .map_err(|e| AppError::Internal(e.to_string()))??,
        )
    };
    let started = chrono::Local::now();
    let transcript = Mutex::new(begin_transcript(&app, title_of(&source), &settings));
    tauri::async_runtime::spawn_blocking(move || {
        // La Trascrizione di ogni Ingresso, uno dopo l'altro con lo stesso motore, con la
        // Diarizzazione; `copy` riceve l'audio di un file.
        let run = |mut copy: Option<OggCopy>| {
            let models = app.state::<Models>();
            let mut engine = models.take(&app, || model.id.as_str())?;
            let mut audio = Vec::new();
            let mut transcribed = Ok(());
            for (index, &ingresso) in ingressi.iter().enumerate() {
                let mut pcm = Vec::new();
                let keep = diarized.contains(&ingresso);
                transcribed = run_pipeline(
                    &app,
                    &mut engine,
                    &silero,
                    &cancel,
                    EventSource {
                        ingresso,
                        session_id: None,
                    },
                    &transcript,
                    |engine, detector, on_event| {
                        transcribe_decoded(
                            Decoder::open_ingresso(&source, ingresso)?,
                            engine,
                            detector,
                            settings.speech_language.code(),
                            keep.then_some(&mut pcm),
                            copy.take(),
                            &cancel,
                            &mut |event| match event {
                                PipelineEvent::Progress(percent) => {
                                    on_event(PipelineEvent::Progress(
                                        percent.map(|p| overall_percent(index, ingressi.len(), p)),
                                    ));
                                }
                                event => on_event(event),
                            },
                        )
                    },
                );
                if keep {
                    audio.push((ingresso, pcm));
                }
                if transcribed.is_err() {
                    break;
                }
            }
            models.release(&app, engine, keep_engine(&transcribed));
            transcribed?;
            let mut transcript = transcript
                .into_inner()
                .unwrap_or_else(PoisonError::into_inner);
            if let Some(diarizer) = diarizer
                && !cancel.is_cancelled()
            {
                diarize_phrases(&app, &diarizer, &audio, &mut transcript, &cancel)?;
            }
            // Annulla premuto dopo l'ultima Frase: il Tape non cambia e non nasce.
            if cancel.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            Ok(transcript)
        };
        let document = |creato, durata_ms, transcript: &Transcript| {
            tape::Document::new(
                creato,
                durata_ms,
                if separate {
                    tape::Modalita::IngressiSeparati
                } else {
                    tape::Modalita::Mix
                },
                Some(model.id.clone()),
                settings.speech_language.clone(),
                true,
                &transcript.phrases,
            )
        };
        // Il Tape con le Frasi; anche senza parlato un Tape si riscrive.
        let saved = match tape {
            Some(old) => {
                let transcript = run(None)?;
                let rewritten = tape::Document {
                    origine: old.origine,
                    ..document(old.creato, old.durata_ms, &transcript)
                };
                tape::rewrite(&source, &rewritten)?;
                (!transcript.phrases.is_empty()).then_some(source)
            }
            None => file_to_tape(&library, &destination, &source, &settings, |copy| {
                let transcript = run(Some(copy))?;
                Ok(tape::Document {
                    origine: source.file_name().map(|n| n.to_string_lossy().into_owned()),
                    // L'ora del file, se la dice; altrimenti quella della Trascrizione.
                    ..document(
                        tape::creato(modified_at(&source).unwrap_or(started)),
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

/// Gli Ingressi da trascrivere di `source`, uno dopo l'altro: di un Tape con l'audio di ogni
/// Ingresso il Microfono e l'Audio di sistema, mai il mix, così i loro Parlanti non si mescolano
/// (ADR-0015); altrimenti il mix.
fn ingressi_of(source: &Path) -> Result<Vec<Ingresso>, AppError> {
    Ok(if tape::is_tape(source) && tape::has_ingressi(source)? {
        vec![Ingresso::Microfono, Ingresso::Sistema]
    } else {
        vec![Ingresso::Mix]
    })
}

/// Il progresso di tutta la Trascrizione con `percent` dell'Ingresso `index` di `count`, trascritti
/// uno dopo l'altro.
fn overall_percent(index: usize, count: usize, percent: u8) -> u8 {
    u8::try_from((index * 100 + usize::from(percent)) / count.max(1)).unwrap_or(100)
}

/// Il Tape `<nome del file>.tape` del file audio o video `source`, nella cartella `destination` (la
/// radice della Libreria `library` se nel frattempo è sparita), con " 2", " 3"… se esiste già.
/// `transcribe` riceve la copia dell'audio, scritta in un Ogg temporaneo nella cartella nascosta con
/// il formato della Registrazione, e restituisce il documento. Annullata, guasta o senza Frasi
/// (`None`): non restano né Tape né Ogg temporaneo.
fn file_to_tape(
    library: &Path,
    destination: &Path,
    source: &Path,
    settings: &Settings,
    transcribe: impl FnOnce(OggCopy) -> Result<tape::Document, AppError>,
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
    let path = numbered(folder, &title_of(source), "tape", Path::exists);
    tape::write(
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

/// L'Ogg temporaneo del Tape di un file: al drop si cancella, e con lui la cartella nascosta se resta
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
    if let Err(e) = (DiarizationStarted { session_id: None }).emit(app) {
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
        diarize::assign_configured(
            &mut transcript.phrases,
            *ingresso,
            found,
            if diarizer.is_nemotron3() {
                crate::managers::settings::Diarizer::Nemotron3
            } else {
                crate::managers::settings::Diarizer::Sortformer
            },
        );
    }
    final_speakers(app, transcript, None);
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
                parlante_non_determinato: phrase.parlante_non_determinato,
                parlante_provvisorio: phrase.parlante_provvisorio,
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
/// i parlanti, `record` rilegge gli Ogg salvati dopo questa funzione: qui Nemotron affianca ASR con un modello e uno stream distinti per ogni Ingresso richiesto.
pub fn transcribe_live(
    app: &AppHandle,
    sources: Vec<LiveSource>,
    session_id: &str,
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
            live_failed(app, session_id, &error, cancel);
            Err(error)
        }
        Ok((silero, mut lease, mut extra)) => {
            let engines = std::iter::once(&mut *lease).chain(&mut extra);
            let results: Vec<_> = std::thread::scope(|scope| {
                let pipelines: Vec<_> = sources
                    .into_iter()
                    .zip(engines)
                    .map(|(source, engine)| {
                        let LiveSource {
                            ingresso,
                            mut frames,
                            diarizer,
                        } = source;
                        let (silero, transcript) = (&silero, &transcript);
                        let pipeline = scope.spawn(move || {
                            let transcribed = if let Some(diarizer) = diarizer {
                                let session = LiveSession {
                                    session_id,
                                    app,
                                    ingresso,
                                    transcript,
                                    state: Mutex::default(),
                                };
                                session.run(
                                    engine,
                                    silero,
                                    cancel,
                                    settings.speech_language.code(),
                                    &mut frames,
                                    diarizer,
                                )
                            } else {
                                run_pipeline(
                                    app,
                                    engine,
                                    silero,
                                    cancel,
                                    EventSource {
                                        ingresso,
                                        session_id: Some(session_id),
                                    },
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
                                )
                            };
                            if let Err(error) = &transcribed {
                                live_failed(app, session_id, error, cancel);
                            }
                            drop(frames);
                            transcribed
                        });
                        (ingresso, pipeline)
                    })
                    .collect();
                pipelines
                    .into_iter()
                    .map(|(_, pipeline)| {
                        pipeline.join().unwrap_or_else(|_| {
                            Err(AppError::Internal("pipeline dal vivo interrotta".into()))
                        })
                    })
                    .collect()
            });
            // La seconda istanza si libera con la Registrazione.
            drop(extra);
            models.release(app, lease, results.iter().all(keep_engine));
            results.into_iter().collect::<Result<(), AppError>>()
        }
    };
    let transcript = transcript
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner);
    (transcript, transcribed)
}

/// Aggiorna Copia testo e le etichette prima di comporre il Tape.
pub fn final_speakers(app: &AppHandle, transcript: &Transcript, session_id: Option<&str>) {
    let last = app.state::<LastTranscript>();
    let same_layout = last.get().is_some_and(|previous| {
        let identity = |p: &Phrase| (p.ingresso, p.inizio_ms, p.fine_ms, p.text.clone());
        previous
            .phrases
            .iter()
            .map(identity)
            .eq(transcript.phrases.iter().map(identity))
    });
    last.set(transcript.clone());
    // La revisione terminale porta anche il testo: resta completa se una rettifica dal vivo
    // arriva fuori ordine, e nel fallback Ogg non c'è una riapertura del Tape a riparare la vista.
    for ingresso in [Ingresso::Mix, Ingresso::Microfono, Ingresso::Sistema] {
        let phrases: Vec<_> = transcript
            .phrases
            .iter()
            .zip(assignments(&transcript.phrases))
            .filter(|(p, _)| p.ingresso == ingresso)
            .map(|(p, a)| transcript_phrase(a.phrase_id, p, session_id))
            .collect();
        if !phrases.is_empty()
            && let Some(session_id) = session_id
            && let Err(error) = (LiveTranscriptUpdated {
                session_id: session_id.to_owned(),
                ingresso,
                revision: u32::MAX,
                finished: true,
                phrases,
                partials: Vec::new(),
            })
            .emit(app)
        {
            log::warn!("testo finale non emesso: {error}");
        }
    }
    if session_id.is_some() {
        return;
    }
    // L'ASR è terminata: le divisioni aumentano le righe, quindi la sequenza completa con gli id
    // per Ingresso sostituisce quelle precedenti e aggiunge le parti. Vale anche se il salvataggio
    // del Tape fallisce e resta solo l'Ogg: la vista e Copia testo ricevono lo stesso risultato.
    if !same_layout {
        for (phrase, assignment) in transcript
            .phrases
            .iter()
            .zip(assignments(&transcript.phrases))
        {
            if let Err(error) =
                transcript_phrase(assignment.phrase_id, phrase, session_id).emit(app)
            {
                log::warn!("Frase finale non emessa: {error}");
            }
        }
        return;
    }
    if let Err(error) = (SpeakersAssigned {
        session_id: session_id.map(str::to_owned),
        speakers: assignments(&transcript.phrases),
    })
    .emit(app)
    {
        log::warn!("speakers-assigned non emesso: {error}");
    }
}

/// Avvisa che la Trascrizione dal vivo si è fermata, a meno che non sia stata annullata (anche
/// perché la Registrazione non è partita).
fn live_failed(app: &AppHandle, session_id: &str, error: &AppError, cancel: &CancelToken) {
    if cancel.is_cancelled() {
        return;
    }
    let failed = LiveTranscriptionFailed {
        session_id: session_id.to_owned(),
        error: error.clone(),
    };
    if let Err(e) = failed.emit(app) {
        log::warn!("live-transcription-failed non emesso: {e}");
    }
}

/// L'esito della Trascrizione dal vivo della Registrazione `recording`, finita la coda: il documento
/// prende il nome del file. Il testo è già nel Tape; se il Tape non si è scritto (`recording` è
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

/// Com'è finita la Trascrizione dal vivo; senza Tape ne salva il documento nel Markdown accanto a
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
        if !tape::is_tape(recording) {
            let markdown = transcript::render(transcript, labels, CopiaCome::Markdown);
            save_md(recording, &markdown)?;
        }
        Ok(LiveTranscription::Saved)
    });
    saved.unwrap_or_else(|error| LiveTranscription::Failed { error })
}

/// Apre il Tape `source` come Sorgente, senza ritrascrivere: le Frasi, come se arrivassero da una
/// Trascrizione, i nomi dei Parlanti e le informazioni.
pub fn open_tape(source: &Path) -> Result<OpenedTape, AppError> {
    let document = tape::read(source)?;
    let phrases = document
        .frasi
        .iter()
        .map(|frase| TranscriptPhrase {
            session_id: None,
            phrase_id: frase.id,
            inizio_ms: frase.inizio_ms,
            fine_ms: frase.fine_ms,
            text: frase.testo.clone(),
            ingresso: frase.ingresso,
            parlante: frase.parlante,
            parlante_non_determinato: frase.parlante_non_determinato,
            parlante_provvisorio: frase.parlante_provvisorio,
        })
        .collect();
    Ok(OpenedTape {
        phrases,
        info: TapeInfo {
            creato: document.creato.clone(),
            durata_ms: document.durata_ms,
            modello: document.modello.as_deref().map(model_name),
            lingua_parlato: document.lingua_parlato,
            ingressi_separati: document.modalita == tape::Modalita::IngressiSeparati,
            completa: document.completa,
            origine: document.origine,
            diarizzazione: document.diarizzazione.clone(),
        },
        parlanti: document.parlanti,
    })
}

/// Il nome del modello con id `id` nel catalogo; un id sconosciuto resta com'è.
fn model_name(id: &str) -> String {
    models::find(id).map_or_else(|| id.to_string(), |m| m.name.clone())
}

/// Il documento di un Tape come Trascrizione, con il nome del modello dal catalogo.
fn tape_transcript(title: String, document: tape::Document) -> Transcript {
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
                parlante_non_determinato: frase.parlante_non_determinato,
                parlante_provvisorio: frase.parlante_provvisorio,
                tempi: frase.tempi,
            })
            .collect(),
        live_asr: Vec::new(),
        parlanti: document.parlanti,
        diarizzazione: document.diarizzazione,
    }
}

fn silero_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .resolve(SILERO_RESOURCE, BaseDirectory::Resource)
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Identità degli eventi: nessuna sessione per i file, UUID proprio per ogni Registrazione.
struct EventSource<'a> {
    ingresso: Ingresso,
    session_id: Option<&'a str>,
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
    source: EventSource<'_>,
    transcript: &Mutex<Transcript>,
    run: impl FnOnce(
        &mut dyn TranscriptionEngine,
        &mut dyn VoiceDetector,
        &mut dyn FnMut(PipelineEvent),
    ) -> Result<u32, AppError>,
) -> Result<(), AppError> {
    let EventSource {
        ingresso,
        session_id,
    } = source;
    engine.set_cancel_token(cancel);
    let last = app.state::<LastTranscript>();
    let update = |change: &dyn Fn(&mut Transcript)| {
        last.update(change);
        change(&mut transcript.lock().unwrap_or_else(PoisonError::into_inner));
    };
    Silero::new(silero).and_then(|mut detector| {
        run(engine, &mut detector, &mut |event| {
            let emitted = match event {
                PipelineEvent::Progress(percent) => TranscriptionProgress {
                    session_id: session_id.map(str::to_owned),
                    percent,
                }
                .emit(app),
                PipelineEvent::Partial {
                    id,
                    inizio_ms,
                    fine_ms,
                    text,
                    ..
                } => TranscriptPartial {
                    session_id: session_id.map(str::to_owned),
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
                    tempi,
                } => {
                    let emitted = TranscriptPhrase {
                        session_id: session_id.map(str::to_owned),
                        phrase_id: id,
                        inizio_ms,
                        fine_ms,
                        text: text.clone(),
                        ingresso,
                        parlante: None,
                        parlante_non_determinato: false,
                        parlante_provvisorio: false,
                    }
                    .emit(app);
                    let phrase = Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        ingresso,
                        parlante: None,
                        parlante_non_determinato: false,
                        parlante_provvisorio: false,
                        tempi,
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
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::decoded_seconds;
    use crate::engine::pipeline::tests::{EnergyDetector, FakeEngine, fixture, wav};
    use crate::engine::pipeline::transcribe_file;
    use crate::engine::transcribe_cpp::OfflineDiarizer;
    use crate::managers::settings::Channels;

    #[test]
    fn divisioni_e_identita_per_ingresso_restano_coerenti_in_tape_copia_correzioni_e_indice() {
        use crate::engine::asr::AsrResult;
        use crate::engine::diarize::Turn;
        use crate::managers::settings::Diarizer;
        let dir = temp_dir("memotape-test-divisioni");
        let root = dir.join("Libreria");
        std::fs::create_dir(&root).unwrap();
        let ogg = dir.join("audio.ogg");
        tone_ogg(&ogg, &[(3.0, true)]);
        let make = |ingresso, text: &str, rows: Vec<(i64, i64, String)>| {
            let result = AsrResult::timed(text.into(), rows);
            Phrase {
                inizio_ms: 0,
                fine_ms: 3000,
                text: result.text,
                tempi: result.tempi,
                ingresso,
                parlante: None,
                parlante_non_determinato: false,
                parlante_provvisorio: false,
            }
        };
        let mut phrases = vec![
            make(
                Ingresso::Microfono,
                "Buongiorno! Arrivederci.",
                vec![
                    (100, 700, "Buongiorno!".into()),
                    (1500, 2200, "Arrivederci.".into()),
                ],
            ),
            make(
                Ingresso::Sistema,
                "Sì. No.",
                vec![(500, 900, "Sì.".into()), (2300, 2700, "No.".into())],
            ),
        ];
        for ingresso in [Ingresso::Microfono, Ingresso::Sistema] {
            diarize::assign_configured(
                &mut phrases,
                ingresso,
                &[
                    Turn {
                        inizio_ms: 0,
                        fine_ms: 1000,
                        parlante: 7,
                    },
                    Turn {
                        inizio_ms: 1000,
                        fine_ms: 3000,
                        parlante: 3,
                    },
                ],
                Diarizer::Nemotron3,
            );
        }
        let document = tape::Document::new(
            "2026-10-05T12:00:00+02:00".into(),
            3000,
            tape::Modalita::IngressiSeparati,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            true,
            &phrases,
        );
        let path = root.join("Conversazione.tape");
        tape::write(
            &path,
            &[
                (Ingresso::Mix, &ogg),
                (Ingresso::Microfono, &ogg),
                (Ingresso::Sistema, &ogg),
            ],
            &document,
            Some(&[0.2, 0.5]),
        )
        .unwrap();
        assert_eq!(tape::read(&path).unwrap(), document);
        let opened = open_tape(&path).unwrap();
        assert_eq!(opened.phrases.len(), 4);
        for ingresso in [Ingresso::Microfono, Ingresso::Sistema] {
            let own: Vec<_> = opened
                .phrases
                .iter()
                .filter(|p| p.ingresso == ingresso)
                .collect();
            assert_ne!(own[0].phrase_id, own[1].phrase_id);
            assert_eq!((own[0].parlante, own[1].parlante), (Some(1), Some(2)));
            assert!(own.iter().all(|p| p.fine_ms > p.inizio_ms));
        }
        let target = opened
            .phrases
            .iter()
            .find(|p| p.text == "Arrivederci.")
            .unwrap();
        assert_eq!((target.inizio_ms, target.fine_ms), (1500, 2200));
        let settings = Settings {
            interface_language: Some(Language::It),
            ..Default::default()
        };
        for format in [CopiaCome::Testo, CopiaCome::Markdown] {
            let text = tape_text(&path, &settings, format).unwrap();
            assert!(text.contains("Buongiorno!"));
            assert!(text.contains("Arrivederci."));
            assert!(text.contains("Microfono · Parlante 2:"));
            assert!(text.contains("Audio di sistema · Parlante 2:"));
        }
        let mut library = Library::open(&root, &dir.join("indice.sqlite")).unwrap();
        library.sync().unwrap();
        let hits = library.search("Arrivederci", None).unwrap();
        assert_eq!(
            (
                hits[0].frasi[0].phrase_id,
                hits[0].frasi[0].ingresso,
                hits[0].frasi[0].inizio_ms
            ),
            (target.phrase_id, Ingresso::Microfono, 1500)
        );
        tape::edit_frase(&path, Ingresso::Microfono, target.phrase_id, "A presto!").unwrap();
        let corrected = tape::read(&path).unwrap();
        let corrected = corrected
            .frasi
            .iter()
            .find(|p| p.id == target.phrase_id)
            .unwrap();
        assert!(corrected.tempi.is_empty());
        assert_eq!(corrected.testo, "A presto!");
        assert_eq!(
            (corrected.inizio_ms, corrected.fine_ms, corrected.parlante),
            (1500, 2200, Some(2))
        );
        assert_eq!(tape::forma_onda(&path), Some(vec![0.2, 0.5]));
        assert!((decoded_seconds(&path) - 3.0).abs() < 0.001);
        assert!(
            tape_text(&path, &settings, CopiaCome::Markdown)
                .unwrap()
                .contains("A presto!")
        );
        let mut old = serde_json::to_value(&document).unwrap();
        for phrase in old["frasi"].as_array_mut().unwrap() {
            phrase.as_object_mut().unwrap().remove("tempi");
        }
        let old: tape::Document = serde_json::from_value(old).unwrap();
        assert!(old.frasi.iter().all(|p| p.tempi.is_empty()));
    }

    /// Un Ogg a 16 kHz mono con un tono nei tratti `true` di `parts` (secondi, parlato).
    fn tone_ogg(path: &Path, parts: &[(f32, bool)]) {
        let samples: Vec<f32> = parts
            .iter()
            .flat_map(|&(seconds, voiced)| {
                (0..(seconds * 16_000.0) as usize).map(move |i| {
                    if voiced {
                        0.5 * (i as f32 * 440.0 * std::f32::consts::TAU / 16_000.0).sin()
                    } else {
                        0.0
                    }
                })
            })
            .collect();
        let mut writer =
            OggOpusWriter::new(std::fs::File::create(path).unwrap(), 16_000, 1, 32).unwrap();
        writer.write(&samples).unwrap();
        writer.finish().unwrap();
    }

    /// Un Tape di una Registrazione da Entrambi senza Frasi: il Microfono parla in 0–1 s, l'Audio
    /// di sistema in 2–3 s, e il mix li somma.
    fn tape_da_entrambi(dir: &Path) -> PathBuf {
        let mic = dir.join("mic.ogg");
        let sys = dir.join("sys.ogg");
        let mix = dir.join("mix.ogg");
        tone_ogg(&mic, &[(1.0, true), (3.0, false)]);
        tone_ogg(&sys, &[(2.0, false), (1.0, true), (1.0, false)]);
        tone_ogg(
            &mix,
            &[(1.0, true), (1.0, false), (1.0, true), (1.0, false)],
        );
        let path = dir.join("Call.tape");
        tape::write(
            &path,
            &[
                (Ingresso::Mix, &mix),
                (Ingresso::Microfono, &mic),
                (Ingresso::Sistema, &sys),
            ],
            &tape::Document::new(
                "2026-10-05T14:24:24+02:00".into(),
                4000,
                tape::Modalita::IngressiSeparati,
                None,
                SpeechLanguage::auto(),
                false,
                &[],
            ),
            None,
        )
        .unwrap();
        path
    }

    #[test]
    fn un_tape_con_l_audio_degli_ingressi_si_trascrive_per_ingresso_e_mai_sul_mix() {
        let dir = temp_dir("memotape-test-tape-ingressi");
        let path = tape_da_entrambi(&dir);
        let ingressi = ingressi_of(&path).unwrap();
        assert_eq!(ingressi, [Ingresso::Microfono, Ingresso::Sistema]);
        // Ogni Ingresso dà le Frasi del suo audio, non di quello dell'altro.
        let starts: Vec<_> = ingressi
            .iter()
            .map(|&ingresso| {
                let mut starts = Vec::new();
                transcribe_decoded(
                    Decoder::open_ingresso(&path, ingresso).unwrap(),
                    &mut FakeEngine::default(),
                    &mut EnergyDetector,
                    None,
                    None,
                    None,
                    &CancelToken::new(),
                    &mut |event| {
                        if let PipelineEvent::Phrase { inizio_ms, .. } = event {
                            starts.push(inizio_ms);
                        }
                    },
                )
                .unwrap();
                starts
            })
            .collect();
        assert_eq!(starts.len(), 2);
        assert!(matches!(starts[0][..], [s] if s < 500), "{starts:?}");
        assert!(
            matches!(starts[1][..], [s] if (1500..2100).contains(&s)),
            "{starts:?}"
        );
    }

    #[test]
    fn un_file_o_un_tape_senza_l_audio_degli_ingressi_si_trascrive_sul_mix() {
        let dir = temp_dir("memotape-test-tape-mix");
        let mix = dir.join("mix.ogg");
        tone_ogg(&mix, &[(1.0, true)]);
        let path = dir.join("Solo mix.tape");
        tape::write(
            &path,
            &[(Ingresso::Mix, &mix)],
            &tape::Document::new(
                "2026-10-05T14:24:24+02:00".into(),
                1000,
                tape::Modalita::Mix,
                None,
                SpeechLanguage::auto(),
                false,
                &[],
            ),
            None,
        )
        .unwrap();
        assert_eq!(ingressi_of(&path).unwrap(), [Ingresso::Mix]);
        assert_eq!(ingressi_of(&mix).unwrap(), [Ingresso::Mix]);
    }

    #[test]
    fn il_progresso_degli_ingressi_trascritti_uno_dopo_l_altro_va_da_0_a_100() {
        assert_eq!(overall_percent(0, 1, 40), 40);
        assert_eq!([0, 100].map(|p| overall_percent(0, 2, p)), [0, 50]);
        assert_eq!(
            [0, 50, 100].map(|p| overall_percent(1, 2, p)),
            [50, 75, 100]
        );
    }

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
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                    tempi: Vec::new(),
                })
                .collect(),
            live_asr: Vec::new(),
            parlanti: BTreeMap::new(),
            diarizzazione: None,
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
        let dir = temp_dir("memotape-test-md");
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
    fn la_trascrizione_dal_vivo_salva_il_markdown_solo_senza_tape() {
        let dir = temp_dir("memotape-test-dal-vivo");
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
        // Il testo è nel Tape.
        assert_eq!(
            live("Registrazione.tape", Ok(()), &document),
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
        // Senza Tape, accanto all'Ogg.
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
        assert_eq!(transcript_text(&last, &settings, None), None);
        last.set(transcript(&["Uno."]));
        last.update(|t| t.title = "Lezione".into());
        let text = transcript_text(&last, &settings, None).unwrap();
        assert!(text.starts_with("Lezione\n\nData: "), "{text}");
        settings.copia_come = CopiaCome::Markdown;
        let text = transcript_text(&last, &settings, None).unwrap();
        assert!(text.starts_with("# Lezione\n\n- **Data:** "), "{text}");
    }

    #[test]
    fn copia_testo_congela_la_revisione_visibile_compresi_i_parziali_divisi() {
        let last = LastTranscript::default();
        last.set(transcript(&["Testo più recente nel backend"]));
        let settings = Settings {
            interface_language: Some(Language::It),
            ..Settings::default()
        };
        let part = |id, speaker, text: &str| TranscriptPhrase {
            session_id: Some("live".into()),
            phrase_id: id,
            inizio_ms: id * 1000,
            fine_ms: (id + 1) * 1000,
            text: text.into(),
            ingresso: Ingresso::Mix,
            parlante: Some(speaker),
            parlante_provvisorio: true,
            parlante_non_determinato: false,
        };
        let visible = [part(0, 1, "Perché sì! "), part(1, 2, "D'accordo?")];
        for format in [CopiaCome::Testo, CopiaCome::Markdown] {
            let settings = Settings {
                copia_come: format,
                ..settings.clone()
            };
            let copied = transcript_text(&last, &settings, Some(&visible)).unwrap();
            assert!(copied.contains("Parlante 1 (provvisorio)"), "{copied}");
            assert!(copied.contains("Parlante 2 (provvisorio)"), "{copied}");
            assert!(copied.contains("Perché sì! "));
            assert!(copied.contains("D'accordo?"));
            assert!(!copied.contains("più recente"));
        }
    }

    #[test]
    fn il_testo_di_un_tape_diventa_la_trascrizione_con_il_nome_del_modello() {
        let phrases = transcript(&["Uno.", "Due."]).phrases;
        let mut document = tape::Document::new(
            "2026-10-03T17:05:42+02:00".into(),
            4000,
            tape::Modalita::Mix,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            false,
            &phrases,
        );
        document.parlanti.insert("mix:1".into(), "Mario".into());
        assert_eq!(
            tape_transcript("Registrazione".into(), document.clone()),
            Transcript {
                title: "Registrazione".into(),
                date: "2026-10-03 17:05".into(),
                durata_ms: Some(4000),
                model: models::default_model().name.clone(),
                speech_language: SpeechLanguage::from("it"),
                phrases,
                live_asr: Vec::new(),
                parlanti: document.parlanti.clone(),
                diarizzazione: document.diarizzazione.clone(),
            }
        );
        // Senza Trascrizione dal vivo il modello non c'è; un id sconosciuto resta com'è.
        for (modello, model) in [(None, ""), (Some("futuro"), "futuro")] {
            let document = tape::Document {
                modello: modello.map(Into::into),
                ..document.clone()
            };
            assert_eq!(tape_transcript(String::new(), document).model, model);
        }
    }

    #[test]
    fn un_tape_aperto_porta_frasi_nomi_e_informazioni() {
        let dir = temp_dir("memotape-test-apri");
        let ogg = dir.join("mix.ogg");
        std::fs::write(&ogg, b"audio").unwrap();
        let mut phrases = transcript(&["Ciao.", "Salve."]).phrases;
        phrases[1].ingresso = Ingresso::Sistema;
        phrases[1].parlante = Some(2);
        let mut document = tape::Document::new(
            "2026-10-03T17:05:42+02:00".into(),
            4000,
            tape::Modalita::IngressiSeparati,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            false,
            &phrases,
        );
        document.origine = Some("Call.mp4".into());
        document.parlanti.insert("sistema:2".into(), "Lucia".into());
        let path = dir.join("Call.tape");
        tape::write(&path, &[(Ingresso::Mix, &ogg)], &document, None).unwrap();
        let opened = open_tape(&path).unwrap();
        assert_eq!(
            opened.info,
            TapeInfo {
                creato: "2026-10-03T17:05:42+02:00".into(),
                durata_ms: 4000,
                modello: Some(models::default_model().name.clone()),
                lingua_parlato: SpeechLanguage::from("it"),
                ingressi_separati: true,
                completa: false,
                origine: Some("Call.mp4".into()),
                diarizzazione: None,
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
        // Copia testo ed Esporta Markdown… rendono il Tape con i nomi dei Parlanti.
        let settings = Settings {
            interface_language: Some(Language::It),
            ..Settings::default()
        };
        let markdown = tape_text(&path, &settings, CopiaCome::Markdown).unwrap();
        assert!(markdown.starts_with("# Call\n"), "{markdown}");
        assert!(
            markdown.ends_with("**Audio di sistema · Lucia:** Salve.\n"),
            "{markdown}"
        );
    }

    #[test]
    fn un_tape_riaperto_conserva_la_voce_non_determinata_audio_nomi_e_forma_onda() {
        let dir = temp_dir("memotape-test-tape-voce-ambigua");
        let audio = dir.join("mix.ogg");
        std::fs::write(&audio, b"audio originale").unwrap();
        let mut phrases = transcript(&["Una voce.", "Due voci insieme."]).phrases;
        phrases[0].parlante = Some(1);
        phrases[1].parlante_non_determinato = true;
        let mut document = tape::Document::new(
            "2026-10-05T17:00:00+02:00".into(),
            4000,
            tape::Modalita::Mix,
            Some(models::default_model().id.clone()),
            SpeechLanguage::from("it"),
            true,
            &phrases,
        );
        document.parlanti.insert("mix:1".into(), "Mario".into());
        let path = dir.join("Intervista.tape");
        tape::write(
            &path,
            &[(Ingresso::Mix, &audio)],
            &document,
            Some(&[0.1, 0.5]),
        )
        .unwrap();
        tape::edit_frase(&path, Ingresso::Mix, 1, "Due voci, testo corretto.").unwrap();
        let opened = open_tape(&path).unwrap();
        assert_eq!(opened.parlanti, document.parlanti);
        assert!(opened.phrases[1].parlante_non_determinato);
        assert_eq!(opened.phrases[1].parlante, None);
        assert_eq!(
            (opened.phrases[1].inizio_ms, opened.phrases[1].fine_ms),
            (0, 1000)
        );
        assert_eq!(tape::forma_onda(&path), Some(vec![0.1, 0.5]));
        use std::io::Read;
        let mut read_audio = Vec::new();
        tape::Mix::open(&path)
            .unwrap()
            .read_to_end(&mut read_audio)
            .unwrap();
        assert_eq!(read_audio, b"audio originale");
        let settings = Settings {
            interface_language: Some(Language::It),
            ..Settings::default()
        };
        for format in [CopiaCome::Testo, CopiaCome::Markdown] {
            let text = tape_text(&path, &settings, format).unwrap();
            assert!(text.contains("Mario:"));
            assert!(text.contains("Parlante non determinato:"));
            assert!(text.contains("Due voci, testo corretto."));
        }
        // Nei documenti v1 precedenti il campo manca e l'attribuzione resta quella di prima.
        let mut old = serde_json::to_value(&document).unwrap();
        for phrase in old["frasi"].as_array_mut().unwrap() {
            phrase
                .as_object_mut()
                .unwrap()
                .remove("parlante_non_determinato");
        }
        let read: tape::Document = serde_json::from_value(old).unwrap();
        assert!(read.frasi.iter().all(|f| !f.parlante_non_determinato));
        assert_eq!(read.frasi[0].parlante, Some(1));
    }

    /// Prova nativa separata dalla suite del core, con l'artefatto locale del ticket 01.
    #[test]
    #[ignore = "richiede MEMOTAPE_NEMOTRON3_MODEL e Nemotron ASR scaricato"]
    fn nemotron3_trascrive_un_file_e_riapre_il_tape_con_i_parlanti() {
        use crate::engine::transcribe_cpp::TranscribeCpp;
        let local = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
        let models_dir =
            PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
        let settings = Settings {
            diarizer: crate::managers::settings::Diarizer::Nemotron3,
            nemotron3_path: Some(local.display().to_string()),
            parlanti_file: true,
            ..Settings::default()
        };
        let mut engine = TranscribeCpp::load(&models::default_model().path(&models_dir)).unwrap();
        let mut detector = crate::audio_toolkit::vad::Silero::new(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/silero_vad.onnx"),
        )
        .unwrap();
        let cancel = CancelToken::new();
        let source = fixture("parlato-due-voci.wav");
        let library = temp_dir("memotape-test-nemotron3-nativo");
        let path = file_to_tape(&library, &library, &source, &settings, |copy| {
            let mut audio = Vec::new();
            let mut phrases = Vec::new();
            let duration = transcribe_file(
                &source,
                &mut engine,
                &mut detector,
                Some("it"),
                Some(&mut audio),
                Some(copy),
                &cancel,
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
                            ingresso: Ingresso::Mix,
                            parlante: None,
                            parlante_non_determinato: false,
                            parlante_provvisorio: false,
                            tempi,
                        });
                    }
                },
            )?;
            let turns = OfflineDiarizer::load_nemotron3(&local)?.diarize(&audio, &cancel)?;
            diarize::assign_configured(
                &mut phrases,
                Ingresso::Mix,
                &turns,
                crate::managers::settings::Diarizer::Nemotron3,
            );
            println!("Frasi native: {phrases:?}");
            let mut found: Vec<_> = phrases.iter().filter_map(|p| p.parlante).collect();
            found.sort_unstable();
            found.dedup();
            assert_eq!(found, [1, 2]);
            assert!(phrases.iter().any(|p| !p.text.is_empty()));
            Ok(tape::Document::new(
                tape::creato(chrono::Local::now()),
                duration,
                tape::Modalita::Mix,
                Some(models::default_model().id.clone()),
                SpeechLanguage::from("it"),
                true,
                &phrases,
            ))
        })
        .unwrap()
        .unwrap();
        let opened = open_tape(&path).unwrap();
        assert!(!opened.phrases.is_empty());
        assert!(tape::forma_onda(&path).is_some());
        assert!((decoded_seconds(&path) - decoded_seconds(&source)).abs() < 0.001);
        let text = tape_text(
            &path,
            &Settings {
                interface_language: Some(Language::It),
                ..settings
            },
            CopiaCome::Markdown,
        )
        .unwrap();
        assert!(text.contains("Parlante 1:"));
        assert!(text.contains("Parlante 2:"));
    }

    #[test]
    fn una_cartella_non_scrivibile_da_errore_dedicato() {
        let source = std::env::temp_dir().join("memotape-test-non-esiste/Audio.wav");
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

    /// Il Tape del file `source` nella Raccolta `destination` della Libreria `library`, trascritto
    /// con il motore finto.
    fn file_tape(
        library: &Path,
        destination: &Path,
        source: &Path,
        settings: &Settings,
        engine: &mut FakeEngine,
        cancel: &CancelToken,
    ) -> Result<Option<PathBuf>, AppError> {
        file_to_tape(library, destination, source, settings, |copy| {
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
                        tempi,
                        ..
                    } = event
                    {
                        phrases.push(Phrase {
                            inizio_ms,
                            fine_ms,
                            text,
                            ingresso: Ingresso::Mix,
                            parlante: None,
                            parlante_non_determinato: false,
                            parlante_provvisorio: false,
                            tempi,
                        });
                    }
                },
            )?;
            Ok(tape::Document::new(
                "2026-10-04T10:15:00+02:00".into(),
                durata_ms,
                tape::Modalita::Mix,
                None,
                SpeechLanguage::auto(),
                true,
                &phrases,
            ))
        })
    }

    /// Canali e frequenza dichiarati nell'`OpusHead` del mix del Tape.
    fn opus_head(path: &Path) -> (u8, u32) {
        use std::io::Read;
        let mut mix = Vec::new();
        tape::Mix::open(path)
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
    fn un_file_diventa_un_tape_nella_raccolta_con_l_audio_nel_formato_della_registrazione() {
        for (name, rate, channels) in [
            ("parlato-it.mp4", 16_000, Channels::Mono),
            ("parlato-it.wav", 48_000, Channels::Stereo),
        ] {
            let library = temp_dir("memotape-test-file-tape");
            let raccolta = library.join("Acme");
            std::fs::create_dir(&raccolta).unwrap();
            let settings = Settings {
                sample_rate: rate,
                channels,
                ..Settings::default()
            };
            let source = fixture(name);
            let tape = |engine: &mut FakeEngine| {
                file_tape(
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
            let path = tape(&mut FakeEngine::default());
            assert_eq!(path, raccolta.join("parlato-it.tape"));
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
            let forma_onda = tape::forma_onda(&path).unwrap();
            let decoded = crate::audio_toolkit::decode::peaks(&path, 1000).unwrap();
            assert_eq!(forma_onda.len(), decoded.len(), "{name}");
            assert!(
                forma_onda
                    .iter()
                    .zip(&decoded)
                    .all(|(f, d)| (f - d).abs() < 0.1),
                "{name}: {forma_onda:?} contro {decoded:?}"
            );
            let document = tape::read(&path).unwrap();
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
            let again = tape(&mut FakeEngine::default());
            assert_eq!(again, raccolta.join("parlato-it 2.tape"));
            assert_eq!(
                tree(&library),
                ["Acme", "Acme/parlato-it 2.tape", "Acme/parlato-it.tape"]
            );
        }
    }

    #[test]
    fn annullata_o_senza_parlato_non_resta_nessun_file() {
        let library = temp_dir("memotape-test-file-annullato");
        let settings = Settings::default();
        let cancel = CancelToken::new();
        let mut engine = FakeEngine {
            cancel_at: Some((1, cancel.clone())),
            ..FakeEngine::default()
        };
        let source = fixture("parlato-it.wav");
        let error =
            file_tape(&library, &library, &source, &settings, &mut engine, &cancel).unwrap_err();
        assert_eq!(error, AppError::Cancelled);
        assert!(tree(&library).is_empty(), "{:?}", tree(&library));
        let silence = wav("silenzio-tape", 16_000, 1, &[(2.0, false)]);
        let none = file_tape(
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
    fn senza_la_raccolta_il_tape_va_nella_radice() {
        let library = temp_dir("memotape-test-file-radice");
        let path = file_tape(
            &library,
            &library.join("Sparita"),
            &fixture("parlato-it.wav"),
            &Settings::default(),
            &mut FakeEngine::default(),
            &CancelToken::new(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(path, library.join("parlato-it.tape"));
    }

    #[test]
    fn i_parlanti_arrivano_con_l_id_della_frase_nel_suo_ingresso() {
        let phrase = |ingresso, parlante| Phrase {
            inizio_ms: 0,
            fine_ms: 0,
            text: String::new(),
            ingresso,
            parlante,
            parlante_non_determinato: false,
            parlante_provvisorio: false,
            tempi: Vec::new(),
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
