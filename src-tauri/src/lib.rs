mod audio_toolkit;
mod commands;
mod engine;
mod managers;

use tauri_specta::{Builder, collect_commands};

/// Comandi ed eventi esposti al frontend, tipizzati in `src/bindings.ts`.
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![commands::app_version])
}

pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("export di src/bindings.ts fallito");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("errore all'avvio dell'applicazione");
}

#[cfg(test)]
mod tests {
    use super::specta_builder;

    #[test]
    fn i_bindings_committati_sono_aggiornati() {
        let path = std::env::temp_dir().join("sbobino-bindings-test.ts");
        specta_builder()
            .export(specta_typescript::Typescript::default(), &path)
            .expect("i comandi e gli eventi devono essere esportabili (niente u64/i64)");
        let fresh = std::fs::read_to_string(&path).unwrap();
        let committed = std::fs::read_to_string("../src/bindings.ts").unwrap_or_default();
        assert!(
            committed == fresh,
            "src/bindings.ts non è aggiornato: rigeneralo con `bun tauri dev`"
        );
    }
}
