// Niente finestra console in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> std::process::ExitCode {
    // Il server MCP degli Assistenti (ADR-0012), prima di Tauri: il plugin single-instance
    // passerebbe gli argomenti all'app aperta.
    if std::env::args().nth(1).as_deref() == Some("--mcp") {
        return memotape_lib::serve_mcp();
    }
    memotape_lib::run();
    std::process::ExitCode::SUCCESS
}
