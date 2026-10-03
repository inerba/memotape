//! Trascrizione di una Sorgente: carica motore e VAD, esegue la pipeline, la traduce in eventi
//! e salva il TXT accanto alla Sorgente.

use std::io::Write;
use std::path::{Path, PathBuf};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::audio_toolkit::vad::Silero;
use crate::engine::pipeline::{PipelineEvent, transcribe_file};
use crate::engine::transcribe_cpp::{NEMOTRON_FILE, TranscribeCpp};
use crate::error::AppError;

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

/// Esito di una Trascrizione completa: dove è il TXT e quanti caratteri contiene.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionFinished {
    pub txt_path: String,
    pub chars: u32,
}

/// Trascrive `source` emettendo `transcription-progress` e `transcript-phrase`, poi salva il TXT.
pub async fn transcribe(
    app: AppHandle,
    source: PathBuf,
) -> Result<TranscriptionFinished, AppError> {
    let internal = |e: tauri::Error| AppError::Internal(e.to_string());
    let model = app
        .path()
        .app_data_dir()
        .map_err(internal)?
        .join("models")
        .join(NEMOTRON_FILE);
    let silero = app
        .path()
        .resolve(SILERO_RESOURCE, BaseDirectory::Resource)
        .map_err(internal)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut engine = TranscribeCpp::load(&model)?;
        let mut detector = Silero::new(&silero)?;
        let mut phrases = Vec::new();
        transcribe_file(&source, &mut engine, &mut detector, &mut |event| {
            let emitted = match event {
                PipelineEvent::Progress(percent) => TranscriptionProgress { percent }.emit(&app),
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
        })?;
        save_txt(&source, &phrases.join("\n"))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
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
    fn una_cartella_non_scrivibile_da_errore_dedicato() {
        let source = std::env::temp_dir().join("sbobino-test-non-esiste/Audio.wav");
        let error = save_txt(&source, "testo").unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
    }
}
