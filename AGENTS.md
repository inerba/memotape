# Memotape

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
| `bun tauri build` | installer NSIS in `src-tauri/target/release/bundle/nsis/` (vedi `docs/sviluppo/installer.md`) |

Ogni ticket si chiude con i sei controlli verdi (da `typecheck` a `cargo test`; `bun tauri build` non è un controllo). Lancia una sola build Rust alla volta: condividono `src-tauri/target`.
Il pre-commit esegue `bunx ultracite fix` sui file staged (lint-staged).
Commit con prefissi convenzionali (`feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`).

## Prerequisiti

Rust MSVC, Visual Studio 2022 con il workload C++, Bun 1.4, Node 22.22, CMake, Vulkan SDK, ONNX Runtime 1.24.2 fuori dal repo e un `.cargo/config.toml` locale alla root (ignorato da git, quindi assente nei worktree): leggi `docs/sviluppo/prerequisiti.md` prima di preparare una build, un worktree o gli smoke test con i modelli veri.

## Struttura

```
src/                     frontend React, struttura bulletproof-react, alias @/ → src/
  app/                   router, provider, routes/, global.css con i token shadcn
  components/ui/         componenti shadcn (solo `shadcn add`, mai modificati a mano)
  components/            componenti condivisi scritti qui (`popover-menu.tsx`, `brand-mark.tsx`)
  features/<feature>/    source, transcription, recording, library, player, models, settings, status, about
  lib/                   utilità condivise, i18n
  locales/               traduzioni delle sei lingue; it.json è il riferimento
  bindings.ts            generato da tauri-specta, committato, non si modifica a mano
src-tauri/src/
  lib.rs                 builder tauri-specta (comandi, eventi) e avvio di Tauri
  error.rs               `AppError`, l'enum degli errori applicativi
  commands/              comandi sottili: validano gli argomenti e delegano ai manager
  managers/              stato in `tauri::State`; traducono i callback della pipeline in eventi
  audio_toolkit/         cattura, ricampionamento, VAD, segmentatore, decodifica, mixer, writer Ogg/Opus, Forma d'onda
  engine/                trait `TranscriptionEngine`, motore transcribe-cpp, pipeline di Trascrizione
  transcript.rs          documento di una Trascrizione e rendering in Markdown o testo semplice
  tape.rs                il Tape: scrittura, lettura, riscrittura atomica, audio del mix e Forma d'onda dentro lo zip
  library.rs             la Libreria: Raccolte, Tape, indice SQLite, operazioni sui file e Cestino
  player.rs              il protocollo `tape` del player (il `mix.ogg` di un Tape con le richieste `Range`) e la Forma d'onda
  mcp.rs                 il server MCP degli Assistenti (`memotape.exe --mcp`, ADR-0012): stdio, sola lettura
  updates.rs             il controllo aggiornamenti: l'ultima release di GitHub contro la versione in esecuzione
docs/brand/              il logo: SVG, PNG e `LINEE-GUIDA.md`; `genera.py` li rigenera, con `src-tauri/icons/`
src-tauri/resources/     risorse del bundle (`silero_vad.onnx`, `licenses/` con i testi delle licenze)
src-tauri/runtime-libs/  DLL copiate da `build.rs` (ONNX Runtime, transcribe.cpp e backend ggml, runtime VC++) e messe
                         accanto all'exe dal bundle, e in dev in `target/debug` (ignorata da git)
src-tauri/tests/fixtures/ audio per i test: `parlato-it.wav` (sintesi vocale di Windows) e lo stesso parlato in
                         `parlato-it.mp4` (H.264 + AAC, 30 KB, fatto con `Windows.Media.Editing` di Media Foundation);
                         `parlato-due-voci.wav` (25 s, 16 kHz: Elsa e Cosimo di `System.Speech` che si alternano)
```

## Regole del codice

- Il frontend chiama il backend solo con `commands` ed `events` di `@/bindings` e con il protocollo `tape` del player; le eccezioni (finestra, drop dei file) sono in `docs/sviluppo/frontend.md`.
- `audio_toolkit` ed `engine` non dipendono da Tauri; nei test le uniche seam finte sono `TranscriptionEngine`, `VoiceDetector` e il motore di `LoadedModel`.
- Nei tipi esportati a TypeScript niente `u64`/`i64`: usa `u32` o `f64`.
- Interfaccia: solo i token di `DESIGN.md`, mai colori fissi; testi in tutte e sei le lingue di `src/locales/`.
- La logica pura del frontend vive nelle feature, con un `*.test.ts` accanto. Niente test sui componenti. Eccezione voluta: il reducer composto della finestra principale sta in `app/routes/home-state.ts`, con `home-state.test.ts`, perché compone due feature (Attività e vista della Sorgente) per una route.
- Errori applicativi: un unico enum serializzato con un codice; il frontend mappa ogni codice a un messaggio tradotto.

## Riferimento per area

Prima di modificare o provare un'area, leggi il suo documento in `docs/sviluppo/`; aggiornalo nello stesso commit quando cambi il comportamento che descrive.

| Area | Documento |
|---|---|
| Registrazione, mixer, Ogg/Opus, Trascrizione dal vivo, sessione | `registrazione.md` |
| Pipeline, motore, Trascrizione di un file, Attività, modelli, Lingua del parlato, Parziali | `trascrizione.md` |
| Riconosci i parlanti (Sortformer, Nemotron 3) | `diarizzazione.md` |
| Tape, Markdown, Libreria, ricerca, vista del Tape, player | `tape-e-libreria.md` |
| Frontend: modulo dell'Attività, drop dei file, route | `frontend.md` |
| Impostazioni, lingua, tema, Assistenti (MCP), aggiornamenti | `impostazioni-e-servizi.md` |
| Errori di build, versioni bloccate, dipendenze | `insidie-toolchain.md`, `insidie-audio.md` |
| Provare l'app (CDP), verifiche a mano, hardware reale | `verifica-manuale.md` |
| Installer NSIS e sua verifica | `installer.md` |

## Agent skills

### Issue tracker

Le issue sono file markdown locali in `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Le cinque etichette canoniche predefinite (needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: un `CONTEXT.md` e `docs/adr/` alla root. See `docs/agents/domain.md`.
