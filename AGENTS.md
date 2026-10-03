# Sbobino

App desktop Tauri 2 + React, solo Windows x64, che trascrive in locale audio, video e Registrazioni.

- Requisiti di prodotto: `PRODUCT.md`, la fonte di verità. Aggiornalo quando cambia un requisito.
- Glossario: `CONTEXT.md`. Usa i suoi termini nel codice, nei test e nei commit (Sorgente, Attività, Frase, Parziale…).
- Decisioni: `docs/adr/`. Ricerche con fonti e versioni verificate: `docs/research/`.
- Spec e ticket: `.scratch/sbobino/`.

## Comandi

| Comando | Cosa fa |
|---|---|
| `bun install` | dipendenze e hook git (Husky) |
| `bun tauri dev` | avvia l'app; in debug rigenera `src/bindings.ts` |
| `bun run typecheck` | `tsc --noEmit` |
| `bun run test` | test del frontend (`bun test src`, file `*.test.ts` accanto al codice) |
| `bun run check` / `bun run fix` | Biome via ultracite, controllo / correzione |
| `bun run format:backend` | `cargo fmt --check` (per correggere: `cargo fmt` in `src-tauri`) |
| `bun run lint:backend` | `cargo clippy --all-targets -- -D warnings` |
| `cargo test` (in `src-tauri`) | test Rust |

Ogni ticket si chiude con tutti e sei i controlli verdi. Lancia una sola build Rust alla volta: condividono `src-tauri/target`.
Il pre-commit esegue `bunx ultracite fix` sui file staged (lint-staged).
Commit con prefissi convenzionali (`feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`).

## Prerequisiti di build

- Rust stable MSVC (`x86_64-pc-windows-msvc`, almeno 1.90 per `tauri` 2.12) e Visual Studio 2022 con il workload C++.
- Bun 1.4 e Node 22.22 o successivo (lo richiede `lint-staged` 17).
- Da M1 in poi, per la pipeline audio:
  - CMake nel PATH: lo usano `opus` e `transcribe-cpp`;
  - Vulkan SDK LunarG (`VULKAN_SDK` impostata), per la feature `vulkan` di `transcribe-cpp`;
  - lo zip ufficiale `onnxruntime-win-x64-1.24.2.zip` dalle release GitHub di Microsoft, estratto in locale;
  - un `src-tauri/.cargo/config.toml` locale (ignorato da git) che fissa le variabili anche nei terminali che non le hanno:

    ```toml
    [env]
    VULKAN_SDK = "C:\\VulkanSDK\\<versione>"
    ORT_LIB_LOCATION = "C:\\percorso\\onnxruntime-win-x64-1.24.2\\lib"
    ORT_PREFER_DYNAMIC_LINK = "1"
    LOCALAPPDATA = "C:\\Users\\<utente>\\AppData\\Local"
    ```

    `LOCALAPPDATA` serve a `transcribe-cpp-sys`, che compila passando da una junction corta in `%LOCALAPPDATA%\tcs` (vedi Insidie).

## Architettura

```
src/                     frontend React, struttura bulletproof-react, alias @/ → src/
  app/                   router, provider, routes/, global.css con i token shadcn
  components/ui/         componenti shadcn (solo `shadcn add`, mai modificati a mano)
  features/<feature>/    source, transcription, recording, models, settings, status, updates, about
  lib/                   utilità condivise, i18n
  locales/               traduzioni (oggi solo it.json)
  bindings.ts            generato da tauri-specta, committato, non si modifica a mano
src-tauri/src/
  lib.rs                 builder tauri-specta (comandi, eventi) e avvio di Tauri
  commands/              comandi sottili: validano gli argomenti e delegano ai manager
  managers/              stato in `tauri::State`; traducono i callback della pipeline in eventi
  audio_toolkit/         cattura, ricampionamento, VAD, segmentatore, decodifica, mixer, writer Ogg/Opus
  engine/                trait `TranscriptionEngine`, motore transcribe-cpp, pipeline di un file
```

- `audio_toolkit` ed `engine` non dipendono da Tauri: si testano senza `AppHandle`. Le uniche seam finte nei test sono `TranscriptionEngine` e `VoiceDetector`.
- Il frontend chiama il backend solo tramite `commands` ed `events` di `@/bindings`.
- La logica pura del frontend vive nelle feature, con un `*.test.ts` accanto. Niente test sui componenti.
- Errori applicativi: un unico enum serializzato con un codice; il frontend mappa ogni codice a un messaggio tradotto.

## Insidie

### Toolchain e tooling

- **Tauri**: crate `tauri` e pacchetti `@tauri-apps/*` devono condividere major e minor (oggi 2.12), altrimenti la CLI si rifiuta di partire. I plugin seguono la stessa regola (`tauri-plugin-log` 2.10 con `@tauri-apps/plugin-log` 2.10). Tauri 3 è in alpha: le versioni npm sono pinnate esatte per non agganciare prerelease.
- **specta**: `specta`, `tauri-specta` (`=2.0.0-rc.25`) e `specta-typescript` (`=0.0.12`) restano pinnati con `=`; un aggiornamento può cambiare il formato di `bindings.ts`.
- **Tipi esportati**: niente `u64`/`i64`, bloccano l'export dei bindings. Usa `u32` (tutti i modelli stanno sotto i 4 GiB) o `f64`.
- **`bindings.ts`**: si rigenera solo in debug all'avvio dell'app (`bun tauri dev`). Il test `i_bindings_committati_sono_aggiornati` fallisce se il file committato non corrisponde ai comandi Rust: rigeneralo e committalo.
- **`cargo test` e il manifest Windows**: `tauri-plugin-dialog` richiede Common Controls v6. `build.rs` incorpora `windows-app-manifest.xml` in tutti gli eseguibili, test compresi; senza, i test escono con `STATUS_ENTRYPOINT_NOT_FOUND`.
- **TypeScript**: resta su 6.0.3, non la 7 (binari nativi, API JS sperimentale). In TS 6 `types` vale `[]` per default: i tipi globali (`bun`, `node`) vanno elencati nel tsconfig.
- **react-router**: resta su 7.18.x, non la 8.
- **Biome**: in `biome.jsonc` gli `includes` iniziano con `"**"`. Con le sole negazioni Biome non controlla nessun file.
- **shadcn**: `components.json` è scritto a mano (stile `new-york`, base `neutral`); si usa solo `shadcn add`, perché `init` ora parte dai preset. I componenti importano `cn` dal pacchetto `cn`: è voluto (changelog shadcn del 2026-09-03), e `@/lib/utils` lo ri-esporta. Non riscrivere l'import.
- **Animazioni**: `tw-animate-css` è importato in `global.css`. `tailwindcss-animate` è il plugin di Tailwind 3 e non serve.
- **Line ending**: `.gitattributes` forza LF, come si aspetta Biome.
- **reqwest**: usa `native-tls` (Schannel) senza feature di default. Il default rustls + aws-lc richiede NASM sulla build MSVC.

### Audio e motore

- **rubato 5** non ha `FftFixedIn`: si usa `Fft` con `FixedSync::Input` e buffer `audioadapter`.
- **Symphonia** non ha una feature `opus`: Opus passa da `symphonia-adapter-libopus`.
- **ONNX Runtime**: niente `DirectML.dll`. Si caricano dinamicamente le DLL dell'ONNX Runtime ufficiale 1.24.2 (`ORT_LIB_LOCATION` + `ORT_PREFER_DYNAMIC_LINK=1`), perché la build prebuilt di `ort` è compilata AVX2 e va in crash all'avvio sulle CPU pre-Haswell.
- **Silero**: il file è `silero_vad.onnx` dal tag v4.0 di snakers4/silero-vad (1 807 522 byte, SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`). vad-rs accetta solo 8 o 16 kHz.
- **transcribe-cpp**: la feature di default è `metal`, quindi `default-features = false` con `vulkan`. Compila passando da una junction NTFS corta in `%LOCALAPPDATA%\tcs` perché MSBuild ignora `LongPathsEnabled`; se la junction fallisce, usa un `CARGO_TARGET_DIR` corto (es. `C:\tc-target`).
- **Loopback WASAPI**: la config del dispositivo di uscita si prende da `default_output_config()`; `default_input_config()` lì dà errore.
- **Cartelle dell'app**: `app_data_dir` è `%APPDATA%\sbobino` (roaming); `app_local_data_dir` è `%LOCALAPPDATA%\sbobino` ed è anche la cartella dati della webview.

### Da verificare sull'hardware reale

- Il loopback non consegna callback a riproduzione ferma su Windows 11 (fonte del 2008).
- Il drift tra il clock del microfono e quello del dispositivo di uscita nelle sessioni lunghe.
- Il pre-skip dell'OpusHead quando l'encoder lavora sotto i 48 kHz (`opusinfo`).
- Se serve `vcomp140.dll` (OpenMP è spento per default in `transcribe-cpp` 0.2.4).

## Agent skills

### Issue tracker

Le issue sono file markdown locali in `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Le cinque etichette canoniche predefinite (needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: un `CONTEXT.md` e `docs/adr/` alla root. See `docs/agents/domain.md`.
