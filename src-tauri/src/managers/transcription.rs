//! Trascrizione di una Sorgente, o dal vivo di una Registrazione: prende il motore del modello
//! scelto (caricato una volta e tenuto tra una Trascrizione e l'altra), esegue la pipeline, la
//! traduce in eventi e salva il TXT accanto alla Sorgente.

use std::io::Write;
use std::path::{Path, PathBuf};

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
use crate::managers::settings::{Settings, SettingsStore};

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

/// Dove è il TXT salvato e quanti caratteri contiene.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionFinished {
    pub txt_path: String,
    pub chars: u32,
}

/// Esito di una Trascrizione arrivata alla fine della Sorgente. Annulla e i guasti sono `AppError`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum TranscriptionOutcome {
    /// Il testo è salvato nel TXT.
    Saved(TranscriptionFinished),
    /// Nessuna Frase: il TXT non si crea.
    NoSpeech,
}

/// Com'è finita la Trascrizione dal vivo di una Registrazione salvata.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum LiveTranscription {
    /// Il testo è salvato nel TXT accanto alla Registrazione.
    Saved(TranscriptionFinished),
    /// Nessuna Frase: il TXT non si crea.
    NoSpeech,
    /// Senza TXT: modello assente (`liveTranscriptionUnavailable`), guasto o Annulla (`cancelled`).
    Failed { error: AppError },
}

/// Trascrive `source` emettendo `transcription-progress`, `transcript-partial` e
/// `transcript-phrase`, poi salva il TXT.
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
    tauri::async_runtime::spawn_blocking(move || {
        let models = app.state::<Models>();
        let engine = models.take(&app, || model.id.as_str())?;
        let phrases = run_pipeline(
            &app,
            engine,
            &silero,
            &cancel,
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
        // Annulla premuto dopo l'ultima Frase: il TXT non si salva lo stesso.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        save_transcript(&source, &phrases)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

/// La Trascrizione dal vivo di una Registrazione: carica il modello scelto e trascrive `frames`
/// man mano che arrivano, fino alla fine della Registrazione e della coda, con gli eventi di
/// `transcribe`. Restituisce le Frasi. Se il modello non si carica (`liveTranscriptionUnavailable`)
/// o la pipeline si guasta emette `live-transcription-failed`: la Registrazione continua.
pub fn transcribe_live(
    app: &AppHandle,
    mut frames: LiveFrames,
    settings: &Settings,
    cancel: &CancelToken,
) -> Result<Vec<String>, AppError> {
    let model = SettingsStore::model_of(settings);
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

/// L'esito della Trascrizione dal vivo della Registrazione `recording`, finita la coda: salva le
/// Frasi nel TXT accanto, a meno che non sia stata annullata o guasta.
pub fn finish_live(
    recording: &Path,
    phrases: Result<Vec<String>, AppError>,
    cancel: &CancelToken,
) -> LiveTranscription {
    let saved = phrases.and_then(|phrases| {
        // Annulla premuto dopo l'ultima Frase: il TXT non si salva lo stesso.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        save_transcript(recording, &phrases)
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
/// `transcription-progress`, `transcript-partial` e `transcript-phrase` e restituisce le Frasi.
/// Poi rende il motore: dopo un guasto interno si scarta e alla volta successiva si ricarica.
fn run_pipeline(
    app: &AppHandle,
    mut engine: Lease<'_, TranscribeCpp>,
    silero: &Path,
    cancel: &CancelToken,
    run: impl FnOnce(
        &mut dyn TranscriptionEngine,
        &mut dyn VoiceDetector,
        &mut dyn FnMut(PipelineEvent),
    ) -> Result<(), AppError>,
) -> Result<Vec<String>, AppError> {
    engine.set_cancel_token(cancel);
    let transcribed = Silero::new(silero).and_then(|mut detector| {
        let mut phrases = Vec::new();
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
                    phrases.push(text);
                    emitted
                }
            };
            if let Err(e) = emitted {
                log::warn!("evento della Trascrizione non emesso: {e}");
            }
        })
        .map(|()| phrases)
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

/// Salva le Frasi, una per riga, nel TXT accanto alla Sorgente; senza Frasi non crea il file.
pub fn save_transcript(
    source: &Path,
    phrases: &[String],
) -> Result<TranscriptionOutcome, AppError> {
    if phrases.is_empty() {
        return Ok(TranscriptionOutcome::NoSpeech);
    }
    save_txt(source, &phrases.join("\n")).map(TranscriptionOutcome::Saved)
}

/// Scrive `text` nel primo `<stem> trascrizione <N>.txt` libero accanto alla Sorgente.
pub fn save_txt(source: &Path, text: &str) -> Result<TranscriptionFinished, AppError> {
    let (path, mut file) = loop {
        let path = txt_path(source, |p| p.exists());
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
        txt_path: path.display().to_string(),
        chars: u32::try_from(text.chars().count()).unwrap_or(u32::MAX),
    })
}

fn unwritable(path: &Path, e: &std::io::Error) -> AppError {
    AppError::UnwritableFolder(format!("{}: {e}", path.display()))
}

/// `<stem della Sorgente> trascrizione <N>.txt` accanto alla Sorgente, con N il primo libero da 1.
pub fn txt_path(source: &Path, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    (1..)
        .map(|n| source.with_file_name(format!("{stem} trascrizione {n}.txt")))
        .find(|path| !exists(path))
        .expect("i numeri non finiscono")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn il_txt_prende_il_primo_numero_libero_da_1() {
        let source = Path::new(r"C:\Lezioni\Lezione 1.mp4");
        let taken = |names: &'static [&str]| {
            move |p: &Path| names.iter().any(|n| p == Path::new(r"C:\Lezioni").join(n))
        };
        assert_eq!(
            txt_path(source, taken(&[])),
            Path::new(r"C:\Lezioni\Lezione 1 trascrizione 1.txt")
        );
        assert_eq!(
            txt_path(
                source,
                taken(&[
                    "Lezione 1 trascrizione 1.txt",
                    "Lezione 1 trascrizione 2.txt"
                ])
            ),
            Path::new(r"C:\Lezioni\Lezione 1 trascrizione 3.txt")
        );
        // Un buco nella numerazione si riempie.
        assert_eq!(
            txt_path(source, taken(&["Lezione 1 trascrizione 2.txt"])),
            Path::new(r"C:\Lezioni\Lezione 1 trascrizione 1.txt")
        );
    }

    #[test]
    fn il_nome_del_txt_toglie_solo_l_ultima_estensione() {
        assert_eq!(
            txt_path(Path::new(r"D:\a\intervista.v2.mkv"), |_| false),
            Path::new(r"D:\a\intervista.v2 trascrizione 1.txt")
        );
    }

    #[test]
    fn salvare_due_volte_non_sovrascrive_e_conta_i_caratteri() {
        let dir = std::env::temp_dir().join("sbobino-test-txt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("Riunione.mp3");
        let first = save_txt(&source, "Perché sì.\nVa bene.").unwrap();
        let second = save_txt(&source, "altro").unwrap();
        assert_eq!(
            first,
            TranscriptionFinished {
                txt_path: dir
                    .join("Riunione trascrizione 1.txt")
                    .display()
                    .to_string(),
                chars: 19,
            }
        );
        assert_eq!(
            second.txt_path,
            dir.join("Riunione trascrizione 2.txt")
                .display()
                .to_string()
        );
        assert_eq!(
            std::fs::read_to_string(&first.txt_path).unwrap(),
            "Perché sì.\nVa bene."
        );
    }

    #[test]
    fn senza_frasi_non_si_salva_nessun_txt() {
        let dir = std::env::temp_dir().join("sbobino-test-nessun-parlato");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("Silenzio.wav");
        assert_eq!(
            save_transcript(&source, &[]).unwrap(),
            TranscriptionOutcome::NoSpeech
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let saved = save_transcript(&source, &["Uno.".into(), "Due.".into()]).unwrap();
        let TranscriptionOutcome::Saved(finished) = saved else {
            panic!("{saved:?}");
        };
        assert_eq!(
            std::fs::read_to_string(finished.txt_path).unwrap(),
            "Uno.
Due."
        );
    }

    #[test]
    fn la_trascrizione_dal_vivo_annullata_o_guasta_non_salva_il_txt() {
        let dir = std::env::temp_dir().join("sbobino-test-dal-vivo");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let recording = dir.join("Registrazione.ogg");
        let phrases = || Ok(vec!["Uno.".to_string()]);
        let cancel = CancelToken::new();
        let guasta = finish_live(&recording, Err(AppError::Internal("x".into())), &cancel);
        assert!(matches!(
            guasta,
            LiveTranscription::Failed {
                error: AppError::Internal(_)
            }
        ));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let saved = finish_live(&recording, phrases(), &cancel);
        assert!(matches!(
            saved,
            LiveTranscription::Saved(TranscriptionFinished { chars: 4, .. })
        ));
        // Annulla dopo l'ultima Frase: niente secondo TXT.
        cancel.cancel();
        assert_eq!(
            finish_live(&recording, phrases(), &cancel),
            LiveTranscription::Failed {
                error: AppError::Cancelled
            }
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    }

    #[test]
    fn una_cartella_non_scrivibile_da_errore_dedicato() {
        let source = std::env::temp_dir().join("sbobino-test-non-esiste/Audio.wav");
        let error = save_txt(&source, "testo").unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
    }
}
