mod audio_toolkit;
mod commands;
mod engine;
mod error;
mod managers;

use tauri::Manager;
use tauri_specta::{Builder, collect_commands, collect_events};

/// Percorso di `bindings.ts`, relativo a `src-tauri` (la cwd di `tauri dev` e di `cargo test`).
#[cfg(any(debug_assertions, test))]
const BINDINGS_PATH: &str = "../src/bindings.ts";

/// Comandi ed eventi esposti al frontend, tipizzati in `src/bindings.ts`.
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::app_version,
            commands::pick_source,
            commands::open_source,
            commands::transcribe,
            commands::cancel_transcription,
            commands::list_models,
            commands::download_model,
            commands::cancel_model_download,
            commands::delete_model,
            commands::get_settings,
            commands::set_settings,
        ])
        .events(collect_events![
            managers::transcription::TranscriptPartial,
            managers::transcription::TranscriptPhrase,
            managers::transcription::TranscriptionProgress,
            managers::models::ModelDownloadProgress,
            managers::models::ModelStateChanged,
        ])
}

pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), BINDINGS_PATH)
        .expect("export di src/bindings.ts fallito");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(managers::activity::Activity::default())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let data = app.path().app_data_dir()?;
            app.manage(managers::settings::SettingsStore::load(
                data.join("settings.json"),
            ));
            app.manage(managers::models::Models::new(data.join("models"))?);
            managers::transcription::preload(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("errore all'avvio dell'applicazione");
}

#[cfg(test)]
mod tests {
    use super::{BINDINGS_PATH, specta_builder};

    #[test]
    fn i_bindings_committati_sono_aggiornati() {
        let path = std::env::temp_dir().join("sbobino-bindings-test.ts");
        specta_builder()
            .export(specta_typescript::Typescript::default(), &path)
            .expect("i comandi e gli eventi devono essere esportabili (niente u64/i64)");
        let fresh = std::fs::read_to_string(&path).unwrap();
        let committed = std::fs::read_to_string(BINDINGS_PATH).unwrap_or_default();
        assert!(
            committed == fresh,
            "src/bindings.ts non è aggiornato: rigeneralo con `bun tauri dev`"
        );
    }
}
