//! Trascrizione di una Sorgente, o dal vivo di una Registrazione: prende il motore del modello
//! scelto (caricato una volta e tenuto tra una Trascrizione e l'altra), esegue la pipeline, la
//! traduce in eventi e salva il Markdown accanto alla Sorgente. Tiene l'ultima Trascrizione per
//! Copia testo.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use transcribe_cpp::CancelToken;

use crate::audio_toolkit::vad::{Silero, VoiceDetector};
use crate::engine::TranscriptionEngine;
use crate::engine::live::LiveFrames;
use crate::engine::pipeline::{self, PipelineEvent, transcribe_file};
use crate::engine::transcribe_cpp::TranscribeCpp;
use crate::error::AppError;
use crate::managers::activity::Activity;
use crate::managers::loaded_model::Lease;
use crate::managers::models::Models;
use crate::managers::settings::{CopiaCome, Language, Settings, SettingsStore};
use crate::transcript::{self, Labels, Phrase, Transcript};

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
}

/// La Trascrizione dal vivo si è fermata (modello assente, guasto): la Registrazione continua senza
/// testo.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct LiveTranscriptionFailed {
    pub error: AppError,
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

/// L'ultima Trascrizione, di un file o dal vivo, anche annullata: quella che Copia testo rende. Si
/// riempie man mano che arrivano le Frasi. In `tauri::State`.
#[derive(Default)]
pub struct LastTranscript(Mutex<Option<Transcript>>);

impl LastTranscript {
    /// Modifica la Trascrizione tenuta, se c'è.
    fn update(&self, change: impl FnOnce(&mut Transcript)) {
        if let Some(transcript) = self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_mut()
        {
            change(transcript);
        }
    }

    fn set(&self, transcript: Transcript) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(transcript);
    }

    fn get(&self) -> Option<Transcript> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Il testo di Copia testo: l'ultima Trascrizione in testo semplice o in Markdown, come dicono le
/// impostazioni. `None` se non ce n'è ancora una.
pub fn transcript_text(last: &LastTranscript, settings: &Settings) -> Option<String> {
    last.get()
        .map(|t| transcript::render(&t, &labels(settings), settings.copia_come))
}

/// I testi del documento nella Lingua dell'interfaccia.
fn labels(settings: &Settings) -> Labels {
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
/// `transcript-phrase`, poi salva il Markdown.
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn transcribe(
    app: AppHandle,
    activity: &Activity,
    source: PathBuf,
) -> Result<TranscriptionOutcome, AppError> {
    let cancel = CancelToken::new();
    let _activity = activity.begin({
        let cancel = cancel.clone();
        move || cancel.cancel()
    })?;
    let settings = app.state::<SettingsStore>().get();
    let model = SettingsStore::model_of(&settings);
    let silero = silero_path(&app)?;
    let transcript = begin_transcript(&app, title_of(&source), &settings);
    tauri::async_runtime::spawn_blocking(move || {
        let models = app.state::<Models>();
        let engine = models.take(&app, || model.id.as_str())?;
        let transcript = run_pipeline(
            &app,
            engine,
            &silero,
            &cancel,
            transcript,
            |engine, detector, on_event| {
                transcribe_file(
                    &source,
                    engine,
                    detector,
                    settings.speech_language.code(),
                    &cancel,
                    on_event,
                )
            },
        )?;
        // Annulla premuto dopo l'ultima Frase: il Markdown non si salva lo stesso.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        save_transcript(&source, &transcript, &labels(&settings))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

/// La Trascrizione dal vivo di una Registrazione: carica il modello scelto e trascrive `frames`
/// man mano che arrivano, fino alla fine della Registrazione e della coda, con gli eventi di
/// `transcribe`. Restituisce il documento, intitolato `title` finché non si sa il nome del file.
/// Se il modello non si carica (`liveTranscriptionUnavailable`) o la pipeline si guasta emette
/// `live-transcription-failed`: la Registrazione continua.
pub fn transcribe_live(
    app: &AppHandle,
    mut frames: LiveFrames,
    settings: &Settings,
    title: &str,
    cancel: &CancelToken,
) -> Result<Transcript, AppError> {
    let model = SettingsStore::model_of(settings);
    let transcript = begin_transcript(app, title.to_string(), settings);
    let transcribed = silero_path(app).and_then(|silero| {
        let models = app.state::<Models>();
        let engine = models.take(app, || model.id.as_str()).map_err(|e| {
            log::warn!("Trascrizione dal vivo senza modello: {e}");
            AppError::LiveTranscriptionUnavailable(model.name.clone())
        })?;
        run_pipeline(
            app,
            engine,
            &silero,
            cancel,
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
    });
    // Annullata (anche perché la Registrazione non è partita): nessun avviso.
    if let Err(error) = &transcribed
        && !cancel.is_cancelled()
    {
        let failed = LiveTranscriptionFailed {
            error: error.clone(),
        };
        if let Err(e) = failed.emit(app) {
            log::warn!("live-transcription-failed non emesso: {e}");
        }
    }
    transcribed
}

/// L'esito della Trascrizione dal vivo della Registrazione `recording`, finita la coda: il documento
/// prende il nome del file e si salva nel Markdown accanto, a meno che la Trascrizione non sia
/// stata annullata o guasta.
pub fn finish_live(
    app: &AppHandle,
    recording: &Path,
    transcript: Result<Transcript, AppError>,
    cancel: &CancelToken,
) -> LiveTranscription {
    let title = title_of(recording);
    app.state::<LastTranscript>()
        .update(|last| last.title.clone_from(&title));
    let settings = app.state::<SettingsStore>().get();
    save_live(
        recording,
        transcript.map(|transcript| Transcript {
            title,
            ..transcript
        }),
        cancel,
        &labels(&settings),
    )
}

/// Salva il documento della Trascrizione dal vivo, se non è stata annullata o guasta.
fn save_live(
    recording: &Path,
    transcript: Result<Transcript, AppError>,
    cancel: &CancelToken,
    labels: &Labels,
) -> LiveTranscription {
    let saved = transcript.and_then(|transcript| {
        // Annulla premuto dopo l'ultima Frase: il Markdown non si salva lo stesso.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        save_transcript(recording, &transcript, labels)
    });
    match saved {
        Ok(TranscriptionOutcome::Saved(finished)) => LiveTranscription::Saved(finished),
        Ok(TranscriptionOutcome::NoSpeech) => LiveTranscription::NoSpeech,
        Err(error) => LiveTranscription::Failed { error },
    }
}

fn silero_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .resolve(SILERO_RESOURCE, BaseDirectory::Resource)
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Esegue `run` con `engine` e Silero, traduce gli eventi della pipeline in
/// `transcription-progress`, `transcript-partial` e `transcript-phrase` e aggiunge le Frasi a
/// `transcript`, che restituisce con la durata. Aggiorna anche `LastTranscript` a
/// ogni Frase. Poi rende il motore: dopo un guasto interno si scarta e alla volta successiva si
/// ricarica.
fn run_pipeline(
    app: &AppHandle,
    mut engine: Lease<'_, TranscribeCpp>,
    silero: &Path,
    cancel: &CancelToken,
    mut transcript: Transcript,
    run: impl FnOnce(
        &mut dyn TranscriptionEngine,
        &mut dyn VoiceDetector,
        &mut dyn FnMut(PipelineEvent),
    ) -> Result<u32, AppError>,
) -> Result<Transcript, AppError> {
    engine.set_cancel_token(cancel);
    let last = app.state::<LastTranscript>();
    let transcribed = Silero::new(silero).and_then(|mut detector| {
        run(&mut *engine, &mut detector, &mut |event| {
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
                    }
                    .emit(app);
                    let phrase = Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        ingresso: None,
                        parlante: None,
                    };
                    last.update(|last| last.phrases.push(phrase.clone()));
                    transcript.phrases.push(phrase);
                    emitted
                }
            };
            if let Err(e) = emitted {
                log::warn!("evento della Trascrizione non emesso: {e}");
            }
        })
        .map(|durata_ms| {
            last.update(|last| last.durata_ms = Some(durata_ms));
            Transcript {
                durata_ms: Some(durata_ms),
                ..transcript
            }
        })
    });
    app.state::<Models>().release(
        app,
        engine,
        !matches!(transcribed, Err(AppError::Internal(_))),
    );
    transcribed
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
                    ingresso: None,
                    parlante: None,
                })
                .collect(),
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
        let recording = dir.join("Registrazione.ogg");
        let document = || Ok(transcript(&["Uno."]));
        let cancel = CancelToken::new();
        let guasta = save_live(
            &recording,
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
        let saved = save_live(&recording, document(), &cancel, &labels());
        assert!(matches!(saved, LiveTranscription::Saved(_)), "{saved:?}");
        // Annulla dopo l'ultima Frase: niente secondo Markdown.
        cancel.cancel();
        assert_eq!(
            save_live(&recording, document(), &cancel, &labels()),
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
    fn una_cartella_non_scrivibile_da_errore_dedicato() {
        let source = std::env::temp_dir().join("sbobino-test-non-esiste/Audio.wav");
        let error = save_md(&source, "testo").unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
    }
}
