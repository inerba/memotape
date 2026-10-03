// Niente finestra console in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sbobino_lib::run();
}
