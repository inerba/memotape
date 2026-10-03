//! Impostazioni persistenti: `settings.json` in `app_data_dir`, letto e validato all'avvio.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::error::AppError;
use crate::managers::models;

/// I bitrate di una Registrazione, in kbps.
pub const BITRATES_KBPS: [u32; 9] = [16, 24, 32, 48, 64, 96, 128, 192, 320];
/// Le frequenze di una Registrazione, in Hz.
pub const SAMPLE_RATES: [u32; 4] = [8_000, 16_000, 24_000, 48_000];

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub recording_source: RecordingSource,
    /// `null`: il microfono predefinito di sistema.
    pub microphone: Option<String>,
    /// `null`: il dispositivo di uscita predefinito, per l'audio di sistema.
    pub output_device: Option<String>,
    /// L'id del modello nel catalogo.
    pub model: String,
    /// `null`: Automatica.
    pub speech_language: Option<Language>,
    pub bitrate_kbps: u32,
    pub channels: Channels,
    /// Hz.
    pub sample_rate: u32,
    /// `null`: `Documenti\Sbobino`.
    pub recordings_folder: Option<String>,
    /// `null`: la lingua di sistema se è tra le sei, altrimenti l'inglese.
    pub interface_language: Option<Language>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RecordingSource {
    Mic,
    System,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Channels {
    Mono,
    Stereo,
}

/// Le sei lingue dell'app, per il parlato e per l'interfaccia.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    It,
    En,
    Fr,
    Es,
    De,
    Pl,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::It => "it",
            Self::En => "en",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::De => "de",
            Self::Pl => "pl",
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recording_source: RecordingSource::Mic,
            microphone: None,
            output_device: None,
            model: models::default_model().id.clone(),
            speech_language: None,
            bitrate_kbps: 32,
            channels: Channels::Mono,
            sample_rate: 48_000,
            recordings_folder: None,
            interface_language: None,
        }
    }
}

impl Settings {
    /// Legge `path`. Se manca, è corrotto o non è valido restituisce i predefiniti: l'avvio non si
    /// blocca.
    pub fn load(path: &Path) -> Self {
        let read = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                log::warn!("impostazioni illeggibili ({}): {e}", path.display());
                return Self::default();
            }
        };
        match serde_json::from_str::<Self>(&read) {
            Ok(settings) if settings.is_valid() => settings,
            Ok(_) => {
                log::warn!("impostazioni non valide in {}", path.display());
                Self::default()
            }
            Err(e) => {
                log::warn!("impostazioni corrotte in {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Scrive `path` passando da un file temporaneo, così un'interruzione non lo lascia a metà.
    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        if !self.is_valid() {
            return Err(AppError::Internal(format!(
                "impostazioni non valide: {self:?}"
            )));
        }
        let unwritable =
            |e: std::io::Error| AppError::UnwritableFolder(format!("{}: {e}", path.display()));
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(unwritable)?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| AppError::Internal(e.to_string()))?;
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, json).map_err(unwritable)?;
        std::fs::rename(&temp, path).map_err(unwritable)
    }

    fn is_valid(&self) -> bool {
        BITRATES_KBPS.contains(&self.bitrate_kbps)
            && SAMPLE_RATES.contains(&self.sample_rate)
            && models::find(&self.model).is_some()
    }
}

/// Le impostazioni correnti e il file in cui si salvano. In `tauri::State`.
pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    /// Legge `path` all'avvio.
    pub fn load(path: PathBuf) -> Self {
        let current = Mutex::new(Settings::load(&path));
        Self { path, current }
    }

    /// Il modello scelto.
    pub fn model(&self) -> &'static models::Model {
        Self::model_of(&self.get())
    }

    /// Il modello di `settings`, che sono validate: è nel catalogo.
    pub fn model_of(settings: &Settings) -> &'static models::Model {
        models::find(&settings.model).unwrap_or_else(models::default_model)
    }

    pub fn get(&self) -> Settings {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Salva `settings` e le rende correnti; restituisce le precedenti.
    pub fn set(&self, settings: Settings) -> Result<Settings, AppError> {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        settings.save(&self.path)?;
        Ok(std::mem::replace(&mut current, settings))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sbobino-test-impostazioni-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("settings.json")
    }

    #[test]
    fn senza_file_valgono_i_predefiniti() {
        let settings = Settings::load(&temp_file("assente"));
        assert_eq!(settings, Settings::default());
        assert_eq!(settings.model, "nemotron-3.5-streaming-0.6b-q5km");
        assert_eq!(settings.speech_language, None);
        assert_eq!(settings.recording_source, RecordingSource::Mic);
        assert_eq!(
            (
                settings.bitrate_kbps,
                settings.channels,
                settings.sample_rate
            ),
            (32, Channels::Mono, 48_000)
        );
        assert_eq!(settings.recordings_folder, None);
        assert_eq!(settings.interface_language, None);
    }

    #[test]
    fn un_file_corrotto_o_non_valido_riporta_ai_predefiniti() {
        let path = temp_file("corrotto");
        let valid = serde_json::to_value(Settings::default()).unwrap();
        let invalid = [
            "{ non è json".to_string(),
            "[]".to_string(),
            // Un campo mancante, un valore fuori dall'elenco, un modello sconosciuto.
            r#"{"model": "nemotron-3.5-streaming-0.6b-q5km"}"#.to_string(),
            with(&valid, "bitrateKbps", 33.into()),
            with(&valid, "sampleRate", 44_100.into()),
            with(&valid, "speechLanguage", "ja".into()),
            with(&valid, "model", "inesistente".into()),
        ];
        for content in invalid {
            std::fs::write(&path, &content).unwrap();
            assert_eq!(Settings::load(&path), Settings::default(), "{content}");
        }
    }

    fn with(value: &serde_json::Value, key: &str, field: serde_json::Value) -> String {
        let mut value = value.clone();
        value[key] = field;
        value.to_string()
    }

    #[test]
    fn le_impostazioni_salvate_si_rileggono() {
        let path = temp_file("salvate");
        let settings = Settings {
            recording_source: RecordingSource::Both,
            microphone: Some("Microfono USB".into()),
            output_device: Some("Cuffie".into()),
            model: "whisper-large-v3-turbo-q5km".into(),
            speech_language: Some(Language::It),
            bitrate_kbps: 128,
            channels: Channels::Stereo,
            sample_rate: 24_000,
            recordings_folder: Some(r"D:\Registrazioni".into()),
            interface_language: Some(Language::Pl),
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
        // Un secondo salvataggio sostituisce il primo.
        let again = Settings {
            speech_language: None,
            ..settings
        };
        again.save(&path).unwrap();
        assert_eq!(Settings::load(&path), again);
    }

    #[test]
    fn non_si_salvano_impostazioni_non_valide() {
        let path = temp_file("rifiutate");
        let invalid = Settings {
            bitrate_kbps: 33,
            ..Settings::default()
        };
        assert!(invalid.save(&path).is_err());
        assert!(!path.exists());
    }
}
