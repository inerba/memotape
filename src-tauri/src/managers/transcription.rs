//! Trascrizione di una Sorgente: carica motore e VAD, esegue la pipeline e la traduce in eventi.

use std::path::PathBuf;

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::audio_toolkit::vad::Silero;
use crate::engine::pipeline::transcribe_file;
use crate::engine::transcribe_cpp::TranscribeCpp;
use crate::error::AppError;

// ponytail: modello fisso messo a mano in `app_data_dir/models`; il catalogo arriva con il ticket 05.
pub const NEMOTRON_FILE: &str = "nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf";
const SILERO_RESOURCE: &str = "resources/silero_vad.onnx";

/// Una Frase conclusa, una per riga nell'area di testo.
#[derive(Debug, Clone, serde::Serialize, specta::Type, Event)]
pub struct TranscriptPhrase {
    pub phrase_id: u32,
    pub text: String,
}

/// Trascrive `source` emettendo `transcript-phrase`; termina a fine Trascrizione.
pub async fn transcribe(app: AppHandle, source: PathBuf) -> Result<(), AppError> {
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
        transcribe_file(
            &source,
            &mut engine,
            &mut detector,
            &mut |phrase_id, text| {
                if let Err(e) = (TranscriptPhrase { phrase_id, text }).emit(&app) {
                    log::warn!("transcript-phrase non emesso: {e}");
                }
            },
        )
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}
