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
  - lo zip ufficiale `onnxruntime-win-x64-1.24.2.zip` dalle [release GitHub di Microsoft](https://github.com/microsoft/onnxruntime/releases/download/v1.24.2/onnxruntime-win-x64-1.24.2.zip) (74 075 355 byte, SHA-256 `8e3e9c826375352e29cb2614fe44f3d7a4b0ff7b8028ad7a456af9d949a7e8b0`), estratto **fuori dal repo**. Sulla macchina di sviluppo sta in `..\sbobino-deps\onnxruntime-win-x64-1.24.2`, accanto alla cartella del repo;
  - un `.cargo/config.toml` locale **alla root del repo** (ignorato da git). Va alla root e non in `src-tauri`, perché Cargo cerca la config partendo dalla cwd e gli script `bun run *:backend` girano dalla root con `--manifest-path`:

    ```toml
    [env]
    LOCALAPPDATA = { value = "src-tauri/target", relative = true, force = true }
    VULKAN_SDK = { value = 'C:\VulkanSDK\<versione>', force = false }
    ORT_LIB_LOCATION = 'D:\percorso\onnxruntime-win-x64-1.24.2\lib'
    ORT_PREFER_DYNAMIC_LINK = "1"
    ```

    `LOCALAPPDATA` serve a `transcribe-cpp-sys`, che compila passando da una junction corta in `%LOCALAPPDATA%\tcs` (vedi Insidie). `relative = true` risolve il percorso rispetto alla cartella che contiene `.cargo`.
- Per trascrivere, finché non arriva il download dei modelli (ticket 05), il modello Nemotron si mette a mano in `%APPDATA%\it.sbobino.desktop\models` (`app_data_dir/models`). Non va committato:
  - URL: `https://huggingface.co/handy-computer/nemotron-3.5-asr-streaming-0.6b-gguf/resolve/6d44e540bc31b0de1dbe174a3cea87f53a7f22fb/nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf`;
  - 559 647 200 byte, SHA-256 `86429e8c4f7fdcf9b3312269ad1ca6669478ba7805331c4aea7a2e33e9910d65`;
  - il nome del file resta `nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf`. Senza il file Trascrivi mostra l'errore "modello assente" con il percorso atteso.
  - Lo smoke test con il modello vero si lancia a mano: `cargo test -- --ignored` in `src-tauri`.

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
  error.rs               `AppError`, l'enum degli errori applicativi
  commands/              comandi sottili: validano gli argomenti e delegano ai manager
  managers/              stato in `tauri::State`; traducono i callback della pipeline in eventi
  audio_toolkit/         cattura, ricampionamento, VAD, segmentatore, decodifica, mixer, writer Ogg/Opus
  engine/                trait `TranscriptionEngine`, motore transcribe-cpp, pipeline di un file
src-tauri/resources/     risorse del bundle (`silero_vad.onnx`)
src-tauri/runtime-libs/  DLL copiate da `build.rs` e messe accanto all'exe dal bundle (ignorata da git)
src-tauri/tests/fixtures/ audio per i test (`parlato-it.wav`, sintesi vocale di Windows)
```

- La pipeline di un file (`engine::pipeline`) è a trazione: il motore legge i frame della Frase da un iteratore, e ogni `next()` decodifica, ricampiona e passa per VAD e segmentatore solo quanto serve. Così con lo streaming (ticket 07) il motore riceverà l'audio mentre la Frase è ancora in corso.

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
- **Biome**: senza un `"**"` iniziale negli `includes` Biome non controlla nessun file (era il bug del template). Il core di ultracite 7.12 lo mette già, quindi in `biome.jsonc` restano solo le negazioni (`components/ui`, `bindings.ts`): ripetere `"**"` fa scattare `noBiomeFirstException`, e `ultracite fix` lo toglie. Se aggiorni ultracite, verifica che un `debugger` in `src/` venga ancora segnalato.
- **shadcn**: `components.json` è scritto a mano (stile `new-york`, base `neutral`); si usa solo `shadcn add`, perché `init` ora parte dai preset. I componenti importano `cn` dal pacchetto `cn`: è voluto (changelog shadcn del 2026-09-03), e `@/lib/utils` lo ri-esporta. Non riscrivere l'import.
- **Animazioni**: `tw-animate-css` è importato in `global.css`. `tailwindcss-animate` è il plugin di Tailwind 3 e non serve.
- **Line ending**: `.gitattributes` forza LF, come si aspetta Biome.
- **reqwest**: usa `native-tls` (Schannel) senza feature di default. Il default rustls + aws-lc richiede NASM sulla build MSVC.

### Audio e motore

- **rubato 5** non ha `FftFixedIn`: si usa `Fft` con `FixedSync::Input` e buffer `audioadapter`.
- **Symphonia** non ha una feature `opus`: Opus passa da `symphonia-adapter-libopus`.
- **ONNX Runtime**: niente build prebuilt di `ort` (compilata AVX2, va in crash all'avvio sulle CPU pre-Haswell) e niente `DirectML.dll`. Si usa l'ONNX Runtime ufficiale 1.24.2 in link dinamico (`ORT_LIB_LOCATION` + `ORT_PREFER_DYNAMIC_LINK=1`): `build.rs` copia `onnxruntime.dll` in `target/<profilo>`, in `target/<profilo>/deps` (per i test) e in `runtime-libs/` per il bundle. Senza quella copia Windows carica l'`onnxruntime.dll` di System32 (Windows ML), più vecchio, e l'avvio fallisce. L'installer NSIS (ticket 12) deve includere la DLL insieme a Silero.
- **Silero**: il file è `silero_vad.onnx` dal tag v4.0 di snakers4/silero-vad (1 807 522 byte, SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`). vad-rs accetta solo 8 o 16 kHz.
- **transcribe-cpp**: la feature di default è `metal`, quindi `default-features = false` con `vulkan`.
  - Con `vulkan` serve il Vulkan SDK: `build.rs` aggiunge `%VULKAN_SDK%\Lib` al percorso del linker per `vulkan-1.lib`. La prima build nativa dura diversi minuti.
  - `transcribe-cpp-sys` compila passando da una junction NTFS corta in `%LOCALAPPDATA%\tcs`, perché MSBuild ignora `LongPathsEnabled`. **Dentro l'app desktop di Claude `%LOCALAPPDATA%` è virtualizzato e CMake fallisce con "os error 267"**: per questo il `.cargo/config.toml` locale sposta `LOCALAPPDATA` in `src-tauri/target`. Se la junction fallisce comunque, usa un `CARGO_TARGET_DIR` corto (es. `C:\tc-target`).
  - Una build Rust alla volta: due build contemporanee condividono la cartella CMake. Se si corrompe, cancella `src-tauri/target/debug/build/transcribe-cpp-sys-*`.
  - La build statica è ottimizzata per la CPU della macchina di build: per distribuire serve `dynamic-backends` (ticket 12), con le DLL dei backend accanto all'exe.
- **Loopback WASAPI**: la config del dispositivo di uscita si prende da `default_output_config()`; `default_input_config()` lì dà errore. A riproduzione ferma il loopback non consegna pacchetti: timer e mix di "Entrambi" usano il timestamp di cattura dei buffer (QPC, lo stesso orologio per le due sorgenti), non il conteggio dei campioni.
- **Cartelle dell'app**: l'identifier Tauri è `it.sbobino.desktop`, non `sbobino`: `%APPDATA%\sbobino` appartiene a una vecchia app con lo stesso nome e va lasciata intatta. Quindi `app_data_dir` è `%APPDATA%\it.sbobino.desktop` (roaming), mentre `app_local_data_dir` è `%LOCALAPPDATA%\it.sbobino.desktop` ed è anche la cartella dati della webview. Tauri sconsiglia un identifier che finisce in `.app`.

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
