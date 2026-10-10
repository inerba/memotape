use std::path::{Path, PathBuf};

fn main() {
    link_vulkan_sdk();
    stage_runtime_libs();

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
    let sdk = std::env::var("VULKAN_SDK").expect("VULKAN_SDK non impostata: vedi docs/sviluppo/prerequisiti.md");
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&sdk).join("Lib").display()
    );
}

/// Le DLL da mettere accanto all'exe: l'ONNX Runtime ufficiale (`ORT_LIB_LOCATION`, `ort` è linkato
/// in dinamico) e, con `dynamic-backends`, `transcribe.dll`, `ggml*.dll` e i moduli dei backend
/// (`DEP_TRANSCRIBE_CPP_RUNTIME_DIR` e `MODULE_DIR`). transcribe-cpp-sys copia già le sue in
/// `target/<profilo>` e `deps`; qui si copia `onnxruntime.dll` negli stessi posti, altrimenti
/// Windows caricherebbe quello di System32, più vecchio. Tutte finiscono anche in `runtime-libs/`,
/// che `tauri.windows.conf.json` mette accanto all'exe nel bundle (e in dev in `target/debug`).
fn stage_runtime_libs() {
    println!("cargo:rerun-if-env-changed=ORT_LIB_LOCATION");
    let lib_dir =
        std::env::var("ORT_LIB_LOCATION").expect("ORT_LIB_LOCATION non impostata: vedi docs/sviluppo/prerequisiti.md");
    let ort = Path::new(&lib_dir).join("onnxruntime.dll");
    println!("cargo:rerun-if-changed={}", ort.display());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // OUT_DIR = target/<profilo>/build/<crate>-<hash>/out
    let profile_dir = out_dir.ancestors().nth(3).unwrap();
    for dir in [profile_dir.to_path_buf(), profile_dir.join("deps")] {
        copy_if_changed(&ort, &dir);
    }

    let mut dlls = vec![ort];
    for var in [
        "DEP_TRANSCRIBE_CPP_RUNTIME_DIR",
        "DEP_TRANSCRIBE_CPP_MODULE_DIR",
    ] {
        let dir = std::env::var(var)
            .unwrap_or_else(|_| panic!("{var} assente: serve la feature `dynamic-backends`"));
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("lettura di {dir}: {e}")) {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "dll") && !dlls.contains(&path) {
                dlls.push(path);
            }
        }
    }
    // `msvcp140*`/`vcruntime140*`: le chiedono le DLL C++ (transcribe, ggml, ONNX Runtime), e un PC
    // pulito può non avere il redistribuibile VC++. L'exe Rust no: tauri-build linka statico il
    // vcruntime.
    dlls.extend(vc_runtime_dlls());
    // Senza moduli dei backend la build `dynamic-backends` non registra nessun dispositivo.
    assert!(
        dlls.iter().any(|d| d
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("ggml-cpu")),
        "nessun modulo ggml-cpu tra le DLL di transcribe-cpp"
    );

    let staging = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("runtime-libs");
    std::fs::create_dir_all(&staging).unwrap();
    // Via le DLL di una build precedente che non servono più.
    for entry in std::fs::read_dir(&staging).unwrap() {
        let path = entry.unwrap().path();
        let stale = !dlls.iter().any(|d| d.file_name() == path.file_name());
        if stale && path.extension().is_some_and(|e| e == "dll") {
            std::fs::remove_file(&path)
                .unwrap_or_else(|e| panic!("rimozione di {}: {e}", path.display()));
        }
    }
    for dll in &dlls {
        // tauri-build emette già dei `rerun-if-changed`, che spengono il controllo di Cargo su
        // tutto il pacchetto: senza questi una DLL nuova (aggiornamento di VS) non arriverebbe.
        println!("cargo:rerun-if-changed={}", dll.display());
        copy_if_changed(dll, &staging);
    }
}

/// Le DLL del runtime VC++ per il deployment app-local, dal redistribuibile di Visual Studio:
/// `VCToolsRedistDir` (prompt dei comandi di VS) o l'installazione con il workload C++ trovata da
/// `vswhere`. Devono essere almeno della versione del toolset che ha linkato le DLL (ONNX Runtime
/// 1.24.2 e transcribe.cpp: 14.44), altrimenti su un PC pulito possono andare in crash.
fn vc_runtime_dlls() -> Vec<PathBuf> {
    println!("cargo:rerun-if-env-changed=VCToolsRedistDir");
    let redist = std::env::var("VCToolsRedistDir")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let vswhere = Path::new(&std::env::var("ProgramFiles(x86)").unwrap())
                .join("Microsoft Visual Studio/Installer/vswhere.exe");
            let out = std::process::Command::new(&vswhere)
                .args(["-latest", "-products", "*", "-requires"])
                .arg("Microsoft.VisualStudio.Component.VC.Tools.x86.x64")
                .args(["-property", "installationPath", "-utf8"])
                .output()
                .unwrap_or_else(|e| panic!("{}: {e}", vswhere.display()));
            let install = String::from_utf8(out.stdout).unwrap();
            assert!(
                !install.trim().is_empty(),
                "Visual Studio con il workload C++ non trovato: imposta VCToolsRedistDir"
            );
            let vc = Path::new(install.trim()).join("VC");
            let version = std::fs::read_to_string(
                vc.join("Auxiliary/Build/Microsoft.VCRedistVersion.default.txt"),
            )
            .expect("redistribuibile VC++ non trovato: imposta VCToolsRedistDir");
            vc.join("Redist/MSVC").join(version.trim())
        });
    // La cartella del redistribuibile ha il nome della versione, es. `14.44.35112`.
    let version: Vec<u32> = redist
        .file_name()
        .unwrap()
        .to_string_lossy()
        .split('.')
        .map_while(|n| n.parse().ok())
        .collect();
    assert!(
        version.len() >= 2 && (version[0], version[1]) >= (14, 44),
        "redistribuibile VC++ {} più vecchio di 14.44",
        redist.display()
    );
    let crt = std::fs::read_dir(redist.join("x64"))
        .unwrap_or_else(|e| panic!("{}: {e}", redist.display()))
        .map(|e| e.unwrap().path())
        .find(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name.starts_with("Microsoft.VC14") && name.ends_with(".CRT")
        })
        .expect("cartella Microsoft.VC14*.CRT assente nel redistribuibile");
    [
        "msvcp140.dll",
        "msvcp140_1.dll",
        "vcruntime140.dll",
        "vcruntime140_1.dll",
    ]
    .iter()
    .map(|dll| crt.join(dll))
    .collect()
}

/// Copia `file` in `dir` solo se manca o è diverso (dimensione o data). `fs::copy` su Windows
/// conserva la data di modifica, quindi una copia identica non fa ripartire il build script per il
/// `rerun-if-changed` che tauri-build mette sulle risorse.
fn copy_if_changed(file: &Path, dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    let dest = dir.join(file.file_name().unwrap());
    let src = std::fs::metadata(file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    if let Ok(old) = std::fs::metadata(&dest)
        && old.len() == src.len()
        && old.modified().ok() == src.modified().ok()
    {
        return;
    }
    std::fs::copy(file, &dest).unwrap_or_else(|e| panic!("copia di {}: {e}", file.display()));
}
