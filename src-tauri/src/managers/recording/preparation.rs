use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager};

use super::{channel_count, input_kinds};
use crate::audio_toolkit::deepfilter::{MODEL_FILE, PreparedFilter};
use crate::audio_toolkit::processing::Format;
use crate::error::AppError;
use crate::managers::settings::{Settings, SettingsStore};

struct PreparedInputs {
    path: PathBuf,
    format: Format,
    filters: Vec<PreparedFilter>,
}

/// Una sola configurazione residente; i prestiti delle sessioni precedenti restano indipendenti.
#[derive(Default)]
pub struct RecordingFilters(Mutex<Option<PreparedInputs>>);

impl RecordingFilters {
    pub(crate) fn configure(&self, path: PathBuf, settings: &Settings) -> Vec<PreparedFilter> {
        let format = Format {
            rate: settings.sample_rate,
            channels: channel_count(settings.channels),
        };
        let count = input_kinds(settings).len();
        let mut prepared = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(current) = &mut *prepared
            && current.path == path
            && current.format == format
            && current.filters.len() == count
        {
            for filter in &mut current.filters {
                if filter.failed() {
                    *filter = PreparedFilter::new(path.clone(), format);
                }
            }
            return current.filters.clone();
        }
        let filters: Vec<_> = (0..count)
            .map(|_| PreparedFilter::new(path.clone(), format))
            .collect();
        *prepared = Some(PreparedInputs {
            path,
            format,
            filters: filters.clone(),
        });
        filters
    }
}

pub(crate) fn preload(app: &AppHandle) {
    match resource_path(app) {
        Ok(path) => {
            app.state::<RecordingFilters>()
                .configure(path, &app.state::<SettingsStore>().get());
        }
        Err(error) => log::warn!("preparazione filtri: {error}"),
    }
}

pub(super) fn resource_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .resolve(MODEL_FILE, tauri::path::BaseDirectory::Resource)
        .map_err(|error| AppError::Internal(error.to_string()))
}
