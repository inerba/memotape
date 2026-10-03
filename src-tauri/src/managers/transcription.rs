//! Trascrizione di una Sorgente: prende il motore del modello scelto (caricato una volta e tenuto
//! tra una Trascrizione e l'altra), esegue la pipeline, la traduce in eventi e salva il TXT
//! accanto alla Sorgente.

use std::io::Write;
use std::path::{Path, PathBuf};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::audio_toolkit::vad::Silero;
use crate::engine::pipeline::{PipelineEvent, transcribe_file};
use crate::error::AppError;
use crate::managers::activity::Activity;
use crate::managers::models::Models;
use crate::managers::settings::{Language, SettingsStore};

const SILERO_RESOURCE: &str = "resources/silero_vad.onnx";

/// Una Frase conclusa, una per riga nell'area di testo.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptPhrase {
    pub phrase_id: u32,
    pub text: String,
}

/// Avanzamento della Trascrizione: `percent` è `null` se la durata della Sorgente non è nota.
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

/// Trascrive `source` emettendo `transcription-progress` e `transcript-phrase`, poi salva il TXT.
/// È un'Attività: se ce n'è già una restituisce `AppError::ActivityInProgress`.
pub async fn transcribe(
    app: AppHandle,
    activity: &Activity,
    source: PathBuf,
) -> Result<TranscriptionOutcome, AppError> {
    let guard = activity.begin()?;
    let cancel = guard.cancel.clone();
    let internal = |e: tauri::Error| AppError::Internal(e.to_string());
    let settings = app.state::<SettingsStore>().get();
    let model = SettingsStore::model_of(&settings);
    let silero = app
        .path()
        .resolve(SILERO_RESOURCE, BaseDirectory::Resource)
        .map_err(internal)?;
    tauri::async_runtime::spawn_blocking(move || {
        let models = app.state::<Models>();
        let mut engine = models.take(&app, || model.id.as_str())?;
        engine.prepare(&cancel, settings.speech_language.map(Language::code));
        let transcribed = Silero::new(&silero).and_then(|mut detector| {
            let mut phrases = Vec::new();
            transcribe_file(
                &source,
                &mut *engine,
                &mut detector,
                &cancel,
                &mut |event| {
                    let emitted = match event {
                        PipelineEvent::Progress(percent) => {
                            TranscriptionProgress { percent }.emit(&app)
                        }
                        PipelineEvent::Phrase { id, text } => {
                            let emitted = TranscriptPhrase {
                                phrase_id: id,
                                text: text.clone(),
                            }
                            .emit(&app);
                            phrases.push(text);
                            emitted
                        }
                    };
                    if let Err(e) = emitted {
                        log::warn!("evento della Trascrizione non emesso: {e}");
                    }
                },
            )
            .map(|()| phrases)
        });
        // Dopo un guasto interno il motore si scarta e alla prossima Trascrizione si ricarica.
        models.release(
            &app,
            engine,
            !matches!(transcribed, Err(AppError::Internal(_))),
        );
        let phrases = transcribed?;
        // Annulla premuto dopo l'ultima Frase: il TXT non si salva lo stesso.
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        save_transcript(&source, &phrases)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
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
    fn una_cartella_non_scrivibile_da_errore_dedicato() {
        let source = std::env::temp_dir().join("sbobino-test-non-esiste/Audio.wav");
        let error = save_txt(&source, "testo").unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
    }
}
