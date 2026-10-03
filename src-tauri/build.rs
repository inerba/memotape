fn main() {
    // Il manifest con Common Controls v6 (richiesto da tauri-plugin-dialog) va incorporato
    // in tutti gli eseguibili, test compresi: senza, `cargo test` esce con
    // STATUS_ENTRYPOINT_NOT_FOUND. Per questo non lo aggiunge tauri-build.
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attributes).expect("tauri-build fallito");

    let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}
