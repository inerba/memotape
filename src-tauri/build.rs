use std::path::{Path, PathBuf};

fn main() {
    link_vulkan_sdk();
    stage_onnxruntime();

    // Il manifest con Common Controls v6 (richiesto da tauri-plugin-dialog) va incorporato
    // in tutti gli eseguibili, test compresi: senza, `cargo test` esce con
    // STATUS_ENTRYPOINT_NOT_FOUND. Per questo non lo aggiunge tauri-build.
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attributes).expect("tauri-build fallito");

    let manifest =
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}

/// La feature `vulkan` di transcribe-cpp linka `vulkan-1.lib`, che sta in `%VULKAN_SDK%\Lib`.
fn link_vulkan_sdk() {
    println!("cargo:rerun-if-env-changed=VULKAN_SDK");
    let sdk = std::env::var("VULKAN_SDK").expect("VULKAN_SDK non impostata: vedi AGENTS.md");
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&sdk).join("Lib").display()
    );
}

/// `ort` è linkato in dinamico all'ONNX Runtime ufficiale (`ORT_LIB_LOCATION`): la DLL va accanto
/// agli eseguibili. Si copia in `target/<profilo>` (app), in `target/<profilo>/deps` (test) e in
/// `runtime-libs/`, che `tauri.windows.conf.json` mette accanto all'exe nel bundle.
/// Senza la copia Windows caricherebbe l'`onnxruntime.dll` di System32, più vecchio.
fn stage_onnxruntime() {
    println!("cargo:rerun-if-env-changed=ORT_LIB_LOCATION");
    let lib_dir =
        std::env::var("ORT_LIB_LOCATION").expect("ORT_LIB_LOCATION non impostata: vedi AGENTS.md");
    let dll = Path::new(&lib_dir).join("onnxruntime.dll");
    println!("cargo:rerun-if-changed={}", dll.display());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // OUT_DIR = target/<profilo>/build/<crate>-<hash>/out
    let profile_dir = out_dir.ancestors().nth(3).unwrap();
    let staging = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("runtime-libs");
    for dir in [profile_dir.to_path_buf(), profile_dir.join("deps"), staging] {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(&dll, dir.join("onnxruntime.dll"))
            .unwrap_or_else(|e| panic!("copia di {}: {e}", dll.display()));
    }
}
