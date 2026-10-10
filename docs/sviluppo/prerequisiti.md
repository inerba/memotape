# Prerequisiti di build

Toolchain, dipendenze native, `.cargo/config.toml` e modelli per gli smoke test. Riferimento per gli agenti, spostato da `AGENTS.md` il 10 ottobre 2026: aggiornalo quando cambia il comportamento che descrive.

- Rust stable MSVC (`x86_64-pc-windows-msvc`, almeno 1.90 per `tauri` 2.12) e Visual Studio 2022 con il workload C++.
- Bun 1.4 e Node 22.22 o successivo (lo richiede `lint-staged` 17).
- Da M1 in poi, per la pipeline audio:
  - CMake nel PATH: lo usano `opus` e `transcribe-cpp`;
  - Vulkan SDK LunarG (`VULKAN_SDK` impostata), per la feature `vulkan` di `transcribe-cpp`;
  - lo zip ufficiale `onnxruntime-win-x64-1.24.2.zip` dalle [release GitHub di Microsoft](https://github.com/microsoft/onnxruntime/releases/download/v1.24.2/onnxruntime-win-x64-1.24.2.zip) (74 075 355 byte, SHA-256 `8e3e9c826375352e29cb2614fe44f3d7a4b0ff7b8028ad7a456af9d949a7e8b0`), estratto **fuori dal repo**. Sulla macchina di sviluppo sta in `..\sbobino-deps\onnxruntime-win-x64-1.24.2`, accanto alla cartella del repo;
  - il redistribuibile VC++ di Visual Studio (`VC\Redist\MSVC`, arriva con il workload C++): `build.rs` lo trova con `vswhere`, o da `VCToolsRedistDir` se impostata (prompt dei comandi di VS), e ne mette le DLL accanto all'exe;
  - un `.cargo/config.toml` locale **alla root del repo** (ignorato da git). Va alla root e non in `src-tauri`, perché Cargo cerca la config partendo dalla cwd e gli script `bun run *:backend` girano dalla root con `--manifest-path`:

    ```toml
    [env]
    LOCALAPPDATA = { value = 'src-tauri\target', relative = true, force = true }
    VULKAN_SDK = { value = 'C:\VulkanSDK\<versione>', force = false }
    ORT_LIB_LOCATION = 'D:\percorso\onnxruntime-win-x64-1.24.2\lib'
    ORT_PREFER_DYNAMIC_LINK = "1"
    ```

    `LOCALAPPDATA` serve a `transcribe-cpp-sys`, che compila passando da una junction corta in `%LOCALAPPDATA%\tcs` (vedi «transcribe-cpp» in `insidie-audio.md`). `relative = true` risolve il percorso rispetto alla cartella che contiene `.cargo`. Usa separatori Windows: il `mklink` del runtime fissato rifiuta percorsi con `/`. Nei worktree lunghi imposta una base assoluta breve fuori da AppData (per questa prova: `D:\local\tauri\sbobino-deps`).
- Per trascrivere serve il modello scelto (Nemotron di default) scaricato da Impostazioni → Trascrizione in `%APPDATA%\it.memotape.desktop\models` (`app_data_dir/models`). Senza il file Trascrivi mostra l'errore "modello assente" con il nome del modello e il link a Impostazioni.
  - Lo smoke test con i modelli veri si lancia a mano: `cargo test -- --ignored --test-threads=1` in `src-tauri`. Va lanciato in sequenza, perché due modelli caricati insieme su Vulkan da thread diversi fanno cadere il processo (`vkCreateFence: Invalid device`, poi `STATUS_STACK_BUFFER_OVERRUN`). Era già così prima del ticket v2/10. Richiede tutti e tre i modelli e con ciascuno trascrive `parlato-it.wav` (Lingua del parlato `it`) e `parlato-it.mp4` (Automatica). Lo smoke test della Diarizzazione richiede anche Sortformer: trascrive `parlato-due-voci.wav` con Nemotron e deve trovare 2 Parlanti alternati.
