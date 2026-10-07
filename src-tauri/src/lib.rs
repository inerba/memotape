mod audio_toolkit;
mod commands;
#[cfg(any(debug_assertions, test))]
mod dev_bindings;
mod engine;
mod error;
mod library;
mod managers;
mod mcp;
mod player;
mod startup;
mod tape;
mod transcript;
mod updates;

pub use mcp::serve as serve_mcp;
use tauri::Manager;
use tauri_specta::{Builder, collect_commands, collect_events};

// Il default TRACE stampa migliaia di passaggi Tract prima della cattura.
const APP_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

/// Percorso di `bindings.ts`, relativo a `src-tauri` (la cwd di `tauri dev` e di `cargo test`).
#[cfg(any(debug_assertions, test))]
const BINDINGS_PATH: &str = "../src/bindings.ts";

/// Comandi ed eventi esposti al frontend, tipizzati in `src/bindings.ts`.
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::app_version,
            commands::app_exe,
            commands::pick_source,
            commands::pick_diarizer_model,
            commands::open_source,
            commands::check_update,
            commands::open_update,
            commands::transcribe,
            commands::diarize,
            commands::cancel_transcription,
            commands::transcript_text,
            commands::open_tape,
            commands::rename_parlante,
            commands::edit_frase,
            commands::edit_turno,
            commands::unisci_turno,
            commands::set_creato,
            commands::tape_text,
            commands::tape_peaks,
            commands::export_markdown,
            commands::take_pending_tape,
            commands::list_models,
            commands::download_model,
            commands::cancel_model_download,
            commands::delete_model,
            commands::get_settings,
            commands::set_settings,
            commands::system_language,
            commands::list_microphones,
            commands::list_output_devices,
            commands::record,
            commands::pause_recording,
            commands::set_muto,
            commands::stop_recording,
            commands::cancel_recording_start,
            commands::recordings_folder,
            commands::pick_folder,
            commands::library_list,
            commands::library_search,
            commands::create_raccolta,
            commands::rename_raccolta,
            commands::delete_raccolta,
            commands::rename_tape,
            commands::move_tape,
            commands::trash_tape,
        ])
        .events(collect_events![
            managers::transcription::TranscriptPartial,
            managers::transcription::TranscriptPhrase,
            managers::transcription::TranscriptionProgress,
            managers::transcription::LiveTranscriptionFailed,
            managers::transcription::LiveDiarizationFailed,
            managers::transcription::LiveTranscriptUpdated,
            managers::transcription::DiarizationStarted,
            managers::transcription::SpeakersAssigned,
            managers::pending_tape::TapeRequested,
            managers::models::ModelDownloadProgress,
            managers::models::ModelStateChanged,
            managers::recording::RecordingTick,
            managers::recording::RecordingCleaningFailed,
            managers::recording::RecordingPhaseChanged,
            managers::recording::RecordingCleaningPreparing,
            managers::library::LibraryChanged,
        ])
}

pub fn run() {
    let builder = specta_builder();

    tauri::Builder::default()
        // Per primo, come chiede il plugin: un secondo avvio esce prima di inizializzare il resto.
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            focus_main(app);
            managers::pending_tape::request(app, args, std::path::Path::new(&cwd));
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(APP_LOG_LEVEL)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(managers::activity::Activity::default())
        .manage(managers::recording::Recorder::default())
        .manage(managers::recording::RecordingFilters::default())
        .manage(managers::transcription::LastTranscript::default())
        .manage(managers::pending_tape::PendingTape::default())
        .manage(managers::library::LibraryState::default())
        .invoke_handler(builder.invoke_handler())
        // Il mix di un Tape per il player, un tratto alla volta, letto fuori dal thread della finestra.
        .register_asynchronous_uri_scheme_protocol("tape", |_, request, responder| {
            tauri::async_runtime::spawn_blocking(move || {
                responder.respond(player::respond(&request));
            });
        })
        // Quello che l'utente ha fatto in Esplora file si vede appena torna all'app.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(true) = event {
                managers::library::sync_in_background(window.app_handle());
            }
        })
        .setup(move |app| {
            // Qui e non prima: un secondo avvio esce prima del setup, e riscrivere il file farebbe
            // ricaricare la pagina da Vite all'istanza aperta.
            #[cfg(debug_assertions)]
            dev_bindings::export(std::path::Path::new(BINDINGS_PATH), |path| {
                builder.export(specta_typescript::Typescript::default(), path)
            })
            .expect("export di src/bindings.ts fallito");
            builder.mount_events(app);
            let data = app.path().app_data_dir()?;
            let settings = managers::settings::SettingsStore::load(data.join("settings.json"));
            app.manage(settings);
            app.manage(managers::models::Models::new(data.join("models"))?);
            // Tema e lingua sono già disponibili quando la WebView legge il documento iniziale.
            startup::create_main(app)?;
            managers::transcription::preload(app.handle());
            managers::recording::preload(app.handle());
            managers::library::sync_in_background(app.handle());
            managers::pending_tape::request(
                app.handle(),
                std::env::args(),
                &std::env::current_dir().unwrap_or_default(),
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("errore all'avvio dell'applicazione");
}

/// Porta in primo piano la finestra principale, anche se ridotta a icona.
fn focus_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::{BINDINGS_PATH, specta_builder};

    #[test]
    #[ignore = "DFN3 reale e logger globale: eseguire isolatamente"]
    fn avvio_dfn3_non_inonda_il_log_della_registrazione() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Counter(AtomicUsize);
        impl log::Log for Counter {
            fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
                metadata.level() <= super::APP_LOG_LEVEL
            }
            fn log(&self, record: &log::Record<'_>) {
                if self.enabled(record.metadata()) {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            }
            fn flush(&self) {}
        }
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        log::set_logger(&COUNTER).unwrap();
        log::set_max_level(super::APP_LOG_LEVEL);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(crate::audio_toolkit::deepfilter::MODEL_FILE);
        let started = std::time::Instant::now();
        let filter = crate::audio_toolkit::deepfilter::DeepFilter::new(
            &path,
            crate::audio_toolkit::processing::Format {
                rate: 48_000,
                channels: 2,
            },
        )
        .unwrap();
        drop(filter);
        let messages = COUNTER.0.load(Ordering::Relaxed);
        eprintln!("Avvio DFN3: {messages} messaggi in {:?}", started.elapsed());
        assert!(
            messages < 1000,
            "l'avvio intasa terminale e log: {messages} messaggi"
        );
    }

    #[test]
    #[ignore = "generatore dei contratti, invocare esplicitamente dopo modifiche agli eventi"]
    fn rigenera_bindings_di_sviluppo() {
        crate::dev_bindings::export(std::path::Path::new(BINDINGS_PATH), |path| {
            specta_builder().export(specta_typescript::Typescript::default(), path)
        })
        .unwrap();
    }

    #[test]
    fn i_bindings_committati_sono_aggiornati() {
        let path = std::env::temp_dir().join("memotape-bindings-test.ts");
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
