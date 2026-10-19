//! L'Aggiornamento (ADR-0036): `tauri-plugin-updater` legge `latest.json` dall'ultima release di
//! GitHub, scarica l'installer, ne verifica la firma e lo avvia. Il controllo non dà mai errori:
//! offline, `latest.json` assente o illeggibile valgono "nessun Aggiornamento".

use std::time::{Duration, Instant};

use serde::Serialize;
use specta::Type;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use tauri_specta::Event;

use crate::error::AppError;

/// Le pagine che `open_update` accetta di aprire.
pub const RELEASES_PREFIX: &str = "https://github.com/inerba/memotape/";
const CHECK_TIMEOUT: Duration = Duration::from_secs(10);
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// Una versione più recente di quella in esecuzione.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct UpdateInfo {
    pub version: String,
    /// La pagina della release, per le Novità e per scaricare a mano.
    pub url: String,
}

/// Avanzamento del download dell'Aggiornamento, al massimo 10 volte al secondo.
#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
pub struct UpdateProgress {
    pub percent: u8,
}

fn release_page(version: &str) -> String {
    format!("{RELEASES_PREFIX}releases/tag/v{version}")
}

/// La percentuale di `received` su `total`; 0 se la dimensione non è nota.
fn percent(received: u64, total: Option<u64>) -> u8 {
    match total {
        Some(total) if total > 0 => (received.min(total) * 100 / total) as u8,
        _ => 0,
    }
}

/// L'Aggiornamento disponibile, o `None` se non c'è o non si può sapere.
pub async fn check(app: &AppHandle) -> Option<UpdateInfo> {
    let updater = app.updater_builder().timeout(CHECK_TIMEOUT).build().ok()?;
    let update = match updater.check().await {
        Ok(update) => update?,
        Err(e) => {
            log::info!("controllo aggiornamenti: {e}");
            return None;
        }
    };
    Some(UpdateInfo {
        url: release_page(&update.version),
        version: update.version,
    })
}

/// Scarica l'Aggiornamento con l'avanzamento in `UpdateProgress`, ne verifica la firma e avvia
/// l'installer: su Windows il plugin chiude l'app, e l'installer la riapre aggiornata.
pub async fn install(app: &AppHandle) -> Result<(), AppError> {
    let failed = |e: tauri_plugin_updater::Error| AppError::UpdateFailed(e.to_string());
    // Senza il timeout del controllo: su una connessione lenta l'installer ci mette di più.
    let update = app
        .updater()
        .map_err(failed)?
        .check()
        .await
        .map_err(failed)?
        .ok_or_else(|| AppError::UpdateFailed("nessun aggiornamento disponibile".into()))?;
    let mut received = 0;
    let mut reported: Option<(u8, Instant)> = None;
    let bytes = update
        .download(
            |chunk, total| {
                received += chunk as u64;
                let now = percent(received, total);
                if reported.is_none_or(|(p, at)| p != now && at.elapsed() >= PROGRESS_INTERVAL) {
                    let _ = UpdateProgress { percent: now }.emit(app);
                    reported = Some((now, Instant::now()));
                }
            },
            || {},
        )
        .await
        .map_err(failed)?;
    update.install(bytes).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_pagina_della_release_ha_il_tag_con_la_v() {
        assert_eq!(
            release_page("0.6.1"),
            "https://github.com/inerba/memotape/releases/tag/v0.6.1"
        );
        assert!(release_page("0.6.1").starts_with(RELEASES_PREFIX));
    }

    #[test]
    fn la_percentuale_non_supera_cento_e_senza_dimensione_e_zero() {
        assert_eq!(percent(50, Some(200)), 25);
        assert_eq!(percent(300, Some(200)), 100);
        assert_eq!(percent(50, None), 0);
        assert_eq!(percent(50, Some(0)), 0);
    }
}
