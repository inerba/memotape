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
| `bun tauri build` | installer NSIS in `src-tauri/target/release/bundle/nsis/` (vedi "Installer") |

Ogni ticket si chiude con i sei controlli verdi (da `typecheck` a `cargo test`; `bun tauri build` non è un controllo). Lancia una sola build Rust alla volta: condividono `src-tauri/target`.
Il pre-commit esegue `bunx ultracite fix` sui file staged (lint-staged).
Commit con prefissi convenzionali (`feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`).

## Prerequisiti di build

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
    LOCALAPPDATA = { value = "src-tauri/target", relative = true, force = true }
    VULKAN_SDK = { value = 'C:\VulkanSDK\<versione>', force = false }
    ORT_LIB_LOCATION = 'D:\percorso\onnxruntime-win-x64-1.24.2\lib'
    ORT_PREFER_DYNAMIC_LINK = "1"
    ```

    `LOCALAPPDATA` serve a `transcribe-cpp-sys`, che compila passando da una junction corta in `%LOCALAPPDATA%\tcs` (vedi Insidie). `relative = true` risolve il percorso rispetto alla cartella che contiene `.cargo`.
- Per trascrivere serve il modello scelto (Nemotron di default) scaricato da Impostazioni → Trascrizione in `%APPDATA%\it.sbobino.desktop\models` (`app_data_dir/models`). Senza il file Trascrivi mostra l'errore "modello assente" con il nome del modello e il link a Impostazioni.
  - Lo smoke test con i modelli veri si lancia a mano: `cargo test -- --ignored` in `src-tauri`. Richiede tutti e tre i modelli e con ciascuno trascrive `parlato-it.wav` (Lingua del parlato `it`) e `parlato-it.mp4` (Automatica).

## Architettura

```
src/                     frontend React, struttura bulletproof-react, alias @/ → src/
  app/                   router, provider, routes/, global.css con i token shadcn
  components/ui/         componenti shadcn (solo `shadcn add`, mai modificati a mano)
  features/<feature>/    source, transcription, recording, models, settings, status, updates, about
  lib/                   utilità condivise, i18n
  locales/               traduzioni delle sei lingue; it.json è il riferimento
  bindings.ts            generato da tauri-specta, committato, non si modifica a mano
src-tauri/src/
  lib.rs                 builder tauri-specta (comandi, eventi) e avvio di Tauri
  error.rs               `AppError`, l'enum degli errori applicativi
  commands/              comandi sottili: validano gli argomenti e delegano ai manager
  managers/              stato in `tauri::State`; traducono i callback della pipeline in eventi
  audio_toolkit/         cattura, ricampionamento, VAD, segmentatore, decodifica, mixer, writer Ogg/Opus
  engine/                trait `TranscriptionEngine`, motore transcribe-cpp, pipeline di un file
src-tauri/resources/     risorse del bundle (`silero_vad.onnx`, `licenses/` con i testi delle licenze)
src-tauri/runtime-libs/  DLL copiate da `build.rs` (ONNX Runtime, transcribe.cpp e backend ggml, runtime VC++) e messe
                         accanto all'exe dal bundle, e in dev in `target/debug` (ignorata da git)
src-tauri/tests/fixtures/ audio per i test: `parlato-it.wav` (sintesi vocale di Windows) e lo stesso parlato in
                         `parlato-it.mp4` (H.264 + AAC, 30 KB, fatto con `Windows.Media.Editing` di Media Foundation)
```

- La pipeline di un file (`engine::pipeline`) è a trazione: il motore legge i frame della Frase da un iteratore, e ogni `next()` decodifica, ricampiona e passa per VAD e segmentatore solo quanto serve. Così il motore in streaming riceve l'audio mentre la Frase è ancora in corso.
- `TranscriptionEngine::transcribe` riceve i frame della Frase, la Lingua del parlato (codice dell'app o `None`) e la callback dei Parziali, e restituisce il testo o un `EngineError` (`Busy`, `Cancelled`, `Internal`). `TranscribeCpp` sceglie da `capabilities().supports_streaming`: per Nemotron apre uno stream per Frase (`feed` a ogni frame, Parziale = `text().display()` quando cambia, testo = `text().full` dopo `finalize`); per Whisper e Parakeet raccoglie la Frase e chiama `run`, senza Parziali. `Busy` (un altro stream tiene il lease del modello) diventa `modelInUse`, che non scarta il motore; con una sola Attività alla volta non capita (ADR-0003).

- `audio_toolkit` ed `engine` non dipendono da Tauri: si testano senza `AppHandle`. Le uniche seam finte nei test sono `TranscriptionEngine` e `VoiceDetector`, più il motore generico di `LoadedModel` (vedi sotto).
- Il frontend chiama il backend solo tramite `commands` ed `events` di `@/bindings`. Anche i plugin passano da un comando Rust: `open_source` usa `tauri-plugin-opener` lato Rust, quindi non servono il pacchetto npm né permessi nella capability.
- La pipeline comunica con un solo sink di `PipelineEvent` (progresso, Parziali e Frasi). Il manager li traduce in `transcription-progress`, `transcript-partial` e `transcript-phrase`, poi salva il TXT; l'esito (`TranscriptionOutcome`: `saved` con percorso e caratteri, oppure `noSpeech` senza TXT) o l'`AppError` tornano come risultato del comando `transcribe`.
- Una sola Attività alla volta: `managers::activity::Activity` (in `tauri::State`) dà un guard con `begin(stop)`, e chi arriva secondo riceve `activityInProgress`. `stop` è la closure che ferma l'Attività, chiamata da `Activity::cancel`: così `activity` non dipende da transcribe-cpp. La Trascrizione passa una closure che preme il `CancelToken` di transcribe-cpp, installato sulla sessione e passato alla pipeline: `cancel_transcription` lo preme, il motore interrompe la Frase in corso (Nemotron ha `Feature::Cancellation`) e la pipeline esce con `AppError::Cancelled` al blocco successivo, senza emettere la fine né salvare il TXT. Il frontend mostra `cancelled` come esito, non come errore. La Registrazione passa una closure che equivale a Stop.
- Registrazione (`managers::recording`): registra dagli ingressi della sorgente di registrazione delle impostazioni (`mic`, `system` o `both`: microfono, audio di sistema, entrambi mixati). Il comando `record(prefix)` è l'Attività e dura fino alla fine, come `transcribe`; il risultato `RecordingSaved { path, error }` porta il file, che il frontend rende la Sorgente, ed `error` (`deviceDisconnected` con il nome) se si è fermata da sola. `pause_recording(paused)` e `stop_recording` agiscono su flag atomici in `Recorder` (`tauri::State`). In un solo thread bloccante: una `audio_toolkit::capture::Capture` per ingresso (cpal; `Kind::System` è il loopback) → `audio_toolkit::mixer::Mixer` → `audio_toolkit::ogg_opus::OggOpusWriter`, scritto mentre si registra.
  - La callback cpal non alloca: prende un `Vec` dal pool della sua cattura (`sync_channel` pre-riempito), copia i campioni convertiti in f32 e lo manda al worker con l'indice dell'ingresso e il timestamp `capture` in ns, su un canale comune a tutte le catture (`capture::channel`). Senza buffer libero il blocco si perde, e il mixer ne fa un buco di silenzio.
  - `Mixer` lavora sulla linea del tempo della sessione, tutta in QPC: l'origine è `Capture::now()` (`Stream::now`, lo stesso orologio dei timestamp) appena aperte le catture, e la posizione di un blocco è timestamp − origine − pause. Il worker chiama `advance(now, paused)` a ogni giro (almeno ogni 100 ms) e poi `push` dei blocchi.
    - Ogni ingresso resta entro 20 ms dai suoi timestamp: un vuoto più lungo diventa silenzio, un anticipo più lungo (blocchi sovrapposti a quanto già scritto, o catturati prima dell'origine) si scarta. Così si corregge anche la deriva rispetto a QPC, a scatti da 20 ms.
    - `advance` riempie di silenzio l'ingresso rimasto indietro di oltre 500 ms (`LAG_NS`): è il loopback a riproduzione ferma. Con solo l'Audio di sistema, nel silenzio il timer resta indietro di 0,5 s e poi recupera.
    - La pausa si misura con `now` tra l'`advance` che la vede iniziare e quello che la vede finire; i blocchi arrivati in pausa si scartano. Un salto di `now` oltre 10 s (PC sospeso) vale come una pausa.
    - Il timer (`elapsed_ms`) è la fine dell'ingresso più avanti sulla linea del tempo. `finish(stop)` completa tutti gli ingressi fino all'istante di Stop, quindi file e timer coincidono.
    - Canali: il mono si duplica in stereo; il loopback a più canali si scende in stereo con centrale e surround a −3 dB (ordine WASAPI assunto), in mono si fa la media di sinistra e destra. Poi ogni ingresso si ricampiona con il suo `resample::Resampler` (generico per frequenza e canali; `FrameResampler` della Trascrizione ci si appoggia) e si somma con clamp a [−1, 1].
    - A fine Registrazione il log dice per ingresso i ms ricevuti contro i ms di timestamp (la deriva, se la consegna è continua) e i ms di silenzio inseriti e scartati.
  - `OggOpusWriter`: encoder alla frequenza delle impostazioni (`Application::Audio`), frame da 20 ms, pre-skip = lookahead × 48000 / Fs, granule = pacchetti × 960, una pagina chiusa ogni 50 pacchetti (1 s), ultimo pacchetto con `EndStream` e granule = pre-skip + durata esatta a 48 kHz (end trimming). L'`OpusHead` porta come input rate la frequenza delle impostazioni (quella dell'encoder), non quella del dispositivo. I test rileggono il file con `audio_toolkit::decode` e ritrovano la durata al millisecondo per ogni frequenza e numero di canali, anche passando dal mixer.
  - `recording-tick { elapsedMs, levels }` ogni 100 ms: `levels` è `{ microphone, system }`, il picco (0–1) di ogni ingresso registrato e `null` per l'altro. Il frontend mostra un indicatore per ingresso (`meters`) in dBFS da −60 a 0 (`levelPercent`); prima del primo tick li prende dalla sorgente di registrazione (`silentLevels`).
  - Il nome è `recording_path` (funzione pura); il prefisso tradotto arriva dal frontend (`recording.prefix`), e il comando rifiuta i caratteri non ammessi nei nomi di file. Il file si crea con `create_new`, dopo l'apertura dei dispositivi: un dispositivo assente (`microphoneMissing`, `outputDeviceMissing`) non lascia file vuoti.
  - Gli errori della callback fermano la Registrazione come Stop, tranne `Xrun`, `RealtimeDenied` e `DeviceChanged`. Su WASAPI uno stream sul dispositivo predefinito non segue il cambio di predefinito: arriva `StreamInvalidated` (o `DeviceNotAvailable` se non resta nessun microfono), e anche questo salva e mostra `deviceDisconnected` ("non è più disponibile, scollegato o cambiato in Windows"). Vale per entrambi gli ingressi, con il nome del dispositivo che si è fermato. Un errore di scrittura (disco pieno) ferma allo stesso modo con `unwritableFolder` nel risultato.
- Modelli (`managers::models`): `models.json`, accanto al modulo e incluso con `include_str!`, è l'unica fonte di URL, SHA-256, dimensione, modalità, licenza e modello predefinito; il nome del file è l'ultimo segmento dell'URL. Il core (`download`, `disk_state`, `delete`) non usa `AppHandle` e si testa contro un server HTTP sulla loopback. Il manager `Models` tiene per modello il download in corso o l'errore dell'ultimo, e li traduce in `model-download-progress` e `model-state-changed`. `download_model` torna subito: l'esito arriva con gli eventi e resta in `list_models`, e Impostazioni rilegge `list_models` a ogni `model-state-changed`.
- Impostazioni (`managers::settings`): `settings.json` in `app_data_dir` (`%APPDATA%\it.sbobino.desktop`), letto e validato da Rust all'avvio in `SettingsStore`; se manca, è corrotto o ha un valore fuori elenco valgono i predefiniti, mentre un modello non più nel catalogo torna solo lui al predefinito. Si scrive passando da un `.json.tmp` rinominato. Ci sono già tutti i campi della spec; Impostazioni mostra Registrazione e audio ("Registra da" = sorgente di registrazione, microfono da `list_microphones`, dispositivo di uscita da `list_output_devices`, bitrate, canali, frequenza), Trascrizione e Generale (Cartella predefinita: `recordings_folder` dà quella in uso, `pick_folder` apre il dialog; Lingua dell'interfaccia) e Informazioni. Microfono e dispositivo di uscita si salvano con l'id di cpal (`wasapi:…`); `null` è il predefinito di sistema, e un id non più collegato dà `microphoneMissing` o `outputDeviceMissing` invece di registrare da un altro dispositivo. Il frontend lo legge con `get_settings` in `main.tsx` prima del primo render e lo tiene in `SettingsProvider` (`useSettings`); `set_settings` valida e salva. La Lingua del parlato è `auto` o una delle sei lingue dell'app.
  - Un `settings.json` che esiste ma non si legge (permessi, file bloccato, una cartella al suo posto) dà `unreadableSettings` da `get_settings`, finché un salvataggio non riesce; un file non UTF-8 vale come corrotto. Se `get_settings` fallisce, anche per un errore di IPC, `main.tsx` parte con `DEFAULT_SETTINGS` (copia di `Settings::default`: tienili allineati) e la status bar mostra l'errore.
- Lingua dell'interfaccia: `null` nelle impostazioni = la lingua di Windows (`system_language`, da `sys-locale`, mappata da `Language::from_locale`; inglese se non è tra le sei). `main.tsx` chiama `i18n.changeLanguage` prima del render, quindi una nuova scelta vale dal prossimo avvio. Senza backend usa `languageOf(navigator.language)`, lo specchio in TS. `src/lib/i18n.test.ts` controlla che ogni lingua abbia le chiavi e le variabili dell'italiano e le forme plurali di `Intl.PluralRules` per quella lingua (`_one/_many/_other` per it/fr/es, `_one/_few/_many/_other` per pl): una chiave nuova va aggiunta in tutti e sei i file.
- Tema: segue Windows con `prefers-color-scheme`. I token scuri di shadcn stanno in `@media (prefers-color-scheme: dark)` dentro `global.css`, ed è anche la variante `dark:` predefinita di Tailwind 4: niente classe `.dark`. Nei componenti si usano solo i token (`bg-background`, `text-destructive`…), mai colori fissi.
- Informazioni (`features/about`): versione da `app_version` e i componenti di `credits.ts`, con nome e licenza dei modelli presi da `models.json`. I testi sono in `src-tauri/resources/licenses`: il frontend li carica con `import.meta.glob(..., { query: "?raw" })` solo quando si apre un testo, e `tauri.windows.conf.json` li copia nel bundle in `licenses/`. Un componente nuovo vuole il suo file lì (lo controlla `credits.test.ts`).
- Modello caricato (`managers::loaded_model::LoadedModel`, dentro `Models`): il motore del modello scelto resta in memoria tra una Trascrizione e l'altra e si ricarica solo se cambia la scelta o il modello viene eliminato (`evict` prima di cancellare il file). Chi lo usa lo prende in prestito (`Models::take` / `release`); chi arriva intanto aspetta su una `Condvar`, quindi due caricamenti non si sovrappongono. `managers::transcription::preload` lo carica in background all'avvio, quando cambia il modello scelto e a fine download: serve anche a conoscere le lingue del modello. In caricamento o in uso il modello è `inUse` in `ModelInfo`: Elimina è disabilitato e il backend risponde `modelInUse`. Dopo un errore `internal` il motore si scarta e si ricarica alla Trascrizione successiva.
- Lingua del parlato: il manager la passa a `transcribe_file`, che la dà al motore a ogni Frase; `TranscribeCpp::set_cancel_token` installa solo il `CancelToken`. `engine::resolve_language` trasforma il codice dell'app nel codice offerto dal modello (`it` → `it-IT` per Nemotron), e senza corrispondenza passa `None` (automatico). Il frontend fa lo stesso filtro in `speechLanguageChoice` sulle `languages` di `ModelInfo`, che restano `null` finché il modello non è stato caricato una volta.
- Parziali: `transcript-partial` e `transcript-phrase` condividono `phraseId`. Il frontend tiene il Parziale a parte e lo mostra come ultima riga dell'area (`withPartial`); la Frase con lo stesso id lo sostituisce (`afterPhrase`). Una Frase finita vuota manda un Parziale vuoto e il suo id passa alla successiva. A fine Trascrizione (anche annullata) il Parziale sparisce, e quelli arrivati dopo la risposta di `transcribe` si ignorano: Tauri non ordina gli eventi rispetto alla risposta di un comando.
- Impostazioni è la route figlia `/settings` della finestra principale e la copre con un livello `fixed`: la finestra resta montata (con `inert`), così testo, eventi e una Trascrizione in corso non si perdono.
- La logica pura del frontend vive nelle feature, con un `*.test.ts` accanto. Niente test sui componenti.
- Errori applicativi: un unico enum serializzato con un codice; il frontend mappa ogni codice a un messaggio tradotto.

## Installer

- `bun tauri build` compila frontend ed exe in release e produce solo l'installer NSIS (`bundle.targets: ["nsis"]`): `src-tauri/target/release/bundle/nsis/Sbobino_<versione>_x64-setup.exe`, circa 15 MB (90 MB una volta installato, di cui 44 di `ggml-vulkan.dll`). La prima volta la CLI di Tauri scarica NSIS e `nsis_tauri_utils`; la prima release compila transcribe-cpp da capo (circa 6 minuti). È una build Rust come le altre: niente `tauri dev` attivo intanto.
- Installa per l'utente corrente (`installMode` predefinito `currentUser`) in `%LOCALAPPDATA%\Sbobino`, senza UAC: `sbobino.exe` con accanto le DLL di `runtime-libs/`, `resources\silero_vad.onnx` e `licenses\*.txt`. L'elenco dei file è in `src-tauri/target/release/nsis/x64/installer.nsi`; `7z l` sull'installer mostra cosa contiene davvero.
- **Editore**: `bundle.publisher` è il segnaposto "EDITORE DA DEFINIRE" (si vede in App installate ed è nella chiave `HKCU\Software\<editore>\Sbobino`). Va deciso prima della prima release pubblica: cambiarlo dopo lascia la chiave del vecchio editore sui PC già installati. L'installer non è firmato (fuori perimetro).
- WebView2: l'installer usa il bootstrapper predefinito, quindi su un Windows 10 che non ce l'ha la scarica durante l'installazione (serve la rete). Windows 11 ce l'ha già.
- La disinstallazione toglie i file installati e lascia impostazioni e modelli (`%APPDATA%\it.sbobino.desktop`) e i dati della WebView, a meno di spuntare "Delete the application data" nel dialog; con `/S` non si cancellano.

### Verifica dell'installer

Sulla macchina di sviluppo, senza toccare l'app di sviluppo:

1. Installa in silenzio in una cartella di prova (per esempio nella scratchpad): `Start-Process <setup.exe> -ArgumentList "/S","/D=<cartella>" -Wait`. `/D=` va per ultimo e senza virgolette. Prima di installare e disinstallare l'installer chiude un `sbobino.exe` in esecuzione solo se è quello di `<cartella>`, non quello di `target\debug`.
2. Avvia `<cartella>\sbobino.exe` da PowerShell dopo aver tolto `ORT_LIB_LOCATION`, `VULKAN_SDK` e `ORT_PREFER_DYNAMIC_LINK` e aver ripulito il `PATH` dal Vulkan SDK. Imposta `WEBVIEW2_USER_DATA_FOLDER` su una cartella di prova, così la WebView non condivide i dati con l'app di sviluppo, e `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9223`.
3. `(Get-Process -Id <pid>).Modules`: `onnxruntime.dll`, `transcribe.dll`, `ggml*.dll`, `msvcp140*.dll` e `vcruntime140*.dll` devono venire da `<cartella>`, `vulkan-1.dll` da System32. Il log (`%LOCALAPPDATA%\it.sbobino.desktop\logs\Sbobino.log`, lo stesso dell'app di sviluppo) riporta `backend di transcribe-cpp: Vulkan0 (…), CPU (…)` e il caricamento del modello.
4. Via CDP sulla 9223 (vedi "Pilotare l'app"): `transcribe` su una copia di `parlato-it.wav` fuori dal repo, poi `record`, qualche secondo e `stop_recording`. Cancella la Registrazione di prova da `Documenti\Sbobino`.
5. Chiudi l'app e lancia `<cartella>\uninstall.exe /S`: la cartella e la chiave `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Sbobino` spariscono, i modelli restano.

L'app installata usa la stessa `%APPDATA%\it.sbobino.desktop` dell'app di sviluppo, quindi anche i modelli già scaricati: `app_data_dir` viene da `FOLDERID_RoamingAppData` e una variabile d'ambiente non lo sposta. Durante la prova non cambiare le impostazioni. La finestra dell'app installata compare sullo schermo dell'utente, che può usarla: un `activityInProgress` inatteso viene da lì.

Esito del 2026-10-03 (Ryzen 7 3700X, RTX 2070 SUPER): tutte le DLL caricate dalla cartella d'installazione, backend Vulkan0 e CPU (modulo `ggml-cpu-haswell`), Nemotron caricato in 1,1 s, fixture trascritta in 1,3 s con il TXT salvato, Registrazione di 4 s salvata e ritrascritta, disinstallazione pulita. Manca la prova su un PC pulito (vedi "Da verificare sull'hardware reale").

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
- **Download e Annulla**: il trasferimento corre in un `tokio::select!` contro il `Notify` di Annulla. Il `.partial` si cancella solo dopo l'uscita dal `select!`, quando il future e il suo file sono già chiusi: su Windows un file aperto non si cancella. Il `select!` serve perché un trasferimento fermo non restituisce chunk, quindi un flag controllato a ogni chunk non scatterebbe. Durante la verifica l'hash gira in un thread bloccante con il file aperto: Annulla si controlla a hash finito, prima del rename. Nel codice e nei documenti il file scaricato a metà è il "`.partial`" o il "file incompleto", non il "parziale": Parziale è un termine del glossario.

### Audio e motore

- **rubato 5** non ha `FftFixedIn`: si usa `Fft` con `FixedSync::Input` e buffer `audioadapter`.
- **Symphonia** non ha una feature `opus`: Opus passa da `symphonia-adapter-libopus`.
- **opus** 0.4 compila libopus 1.6.1 da sorgente con CMake (via `opusic-sys`, link statico): niente DLL da distribuire. L'encoder accetta solo 8/12/16/24/48 kHz, quindi la frequenza del dispositivo (spesso 44,1 kHz) si ricampiona sempre prima.
- **cpal 0.18**: `DeviceTrait::name()` non c'è più, il nome è `device.to_string()` e l'id stabile è `device.id()` (`Display`/`FromStr`, formato `wasapi:{…}`). Gli stream non partono senza `play()`. Gli stream di cattura WASAPI accettano solo il formato nativo (di solito f32 alla frequenza del mix): `capture` gestisce f32, i16 e i32.
- **chrono**: è già nel grafo tramite Tauri, con `clock`; serve per la data e ora locali nel nome delle Registrazioni.
- **ONNX Runtime**: niente build prebuilt di `ort` (compilata AVX2, va in crash all'avvio sulle CPU pre-Haswell) e niente `DirectML.dll`. Si usa l'ONNX Runtime ufficiale 1.24.2 in link dinamico (`ORT_LIB_LOCATION` + `ORT_PREFER_DYNAMIC_LINK=1`): `build.rs` copia `onnxruntime.dll` in `target/<profilo>`, in `target/<profilo>/deps` (per i test) e in `runtime-libs/` per il bundle. Senza quella copia Windows carica l'`onnxruntime.dll` di System32 (Windows ML), più vecchio, e l'avvio fallisce.
- **Silero**: il file è `silero_vad.onnx` dal tag v4.0 di snakers4/silero-vad (1 807 522 byte, SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`). vad-rs accetta solo 8 o 16 kHz.
- **transcribe-cpp**: la feature di default è `metal`, quindi `default-features = false` con `vulkan`.
  - Con `vulkan` serve il Vulkan SDK: `build.rs` aggiunge `%VULKAN_SDK%\Lib` al percorso del linker per `vulkan-1.lib`. La prima build nativa dura diversi minuti.
  - `transcribe-cpp-sys` compila passando da una junction NTFS corta in `%LOCALAPPDATA%\tcs`, perché MSBuild ignora `LongPathsEnabled`. **Dentro l'app desktop di Claude `%LOCALAPPDATA%` è virtualizzato e CMake fallisce con "os error 267"**: per questo il `.cargo/config.toml` locale sposta `LOCALAPPDATA` in `src-tauri/target`. Se la junction fallisce comunque, usa un `CARGO_TARGET_DIR` corto (es. `C:\tc-target`).
  - Una build Rust alla volta: due build contemporanee condividono la cartella CMake. Se si corrompe, cancella `src-tauri/target/debug/build/transcribe-cpp-sys-*`.
  - **`dynamic-backends`, in dev come in release.** La build statica sarebbe ottimizzata per la CPU della macchina di build. Con `dynamic-backends` ci sono `transcribe.dll`, `ggml.dll`, `ggml-base.dll`, un `ggml-cpu-<livello>.dll` per livello di ISA (da `x64`/`sse42` a `alderlake`/`icelake`, scelto a runtime) e `ggml-vulkan.dll`. `TranscribeCpp::load` chiama `init_backends_default` una volta per processo, che cerca i moduli accanto a `transcribe.dll`; il log dice quali dispositivi si sono registrati. In dev funziona uguale (smoke test con Vulkan), quindi c'è una sola configurazione e quello che si prova in dev è quello che si installa. Cambiare le feature ricompila CMake da capo (circa 4 minuti e mezzo).
  - transcribe-cpp-sys copia le sue DLL in `target/<profilo>` e `deps`; `build.rs` le prende da `DEP_TRANSCRIBE_CPP_RUNTIME_DIR`/`MODULE_DIR` e le mette in `runtime-libs/`, togliendo quelle che non servono più. Debug e release scrivono nella stessa cartella, ma va bene: la sys compila CMake in `Release` in entrambi i profili. `build.rs` copia solo i file cambiati: tauri-build mette `rerun-if-changed` sulle risorse, e riscrivere le DLL a ogni build farebbe ripartire build script e compilazione ogni volta (`fs::copy` su Windows conserva la data).
  - `ggml-vulkan.dll` dipende da `vulkan-1.dll`, il loader che installano i driver della GPU: non si distribuisce. Se manca, il modulo non si carica e transcribe-cpp usa la CPU.
  - **`vcomp140.dll` non serve**: OpenMP è spento in 0.2.4 e `dumpbin /dependents` non lo trova in nessuna DLL. Servono invece `msvcp140.dll`, `msvcp140_1.dll`, `vcruntime140.dll` e `vcruntime140_1.dll` (ggml, transcribe e ONNX Runtime sono C++ con `/MD`): vanno nel bundle dal redistribuibile di VS, perché un PC pulito può non averli. Devono essere almeno della versione del toolset che ha linkato le DLL (14.44 per ONNX Runtime 1.24.2 e per transcribe.cpp qui): `build.rs` rifiuta un redistribuibile più vecchio. L'exe di Sbobino invece linka statico il vcruntime (lo fa tauri-build) e usa solo l'UCRT di Windows.
- **Loopback WASAPI**: è uno stream di input aperto sul dispositivo di uscita, con la config da `default_output_config()` (il mix format, di solito f32 48 kHz stereo); `default_input_config()` lì dà errore. Verificato su Windows 11 (26200) con un'uscita USB Focusrite:
  - a riproduzione ferma il loopback non consegna nessun pacchetto, nemmeno di silenzio: timer e file avanzano solo grazie a `advance` con l'orologio QPC;
  - all'inizio e alla fine di ogni riproduzione arriva spesso un `Xrun` (discontinuità), che si ignora: il buco diventa silenzio;
  - mic e loopback hanno lo stesso orologio (QPC) e si allineano per timestamp, non per numero di campioni.
- **Deriva misurata** (Windows 11, sessione Entrambi di 4 min con il microfono USB Anker PowerConf C200 e l'uscita Focusrite): il microfono ha consegnato 235 700 ms di audio in 235 697 ms di QPC (+3 ms, circa 13 ppm); il loopback, dopo una discontinuità da 20 ms all'avvio della riproduzione, nessuna correzione per 234 s, quindi meno di 20 ms. Mic e uscita non si sono scostati di più di 20 ms tra loro. All'avvio il mixer scarta i blocchi del microfono catturati prima dell'origine (100–170 ms, il tempo di aprire il loopback).
- **Cartelle dell'app**: l'identifier Tauri è `it.sbobino.desktop`, non `sbobino`: `%APPDATA%\sbobino` appartiene a una vecchia app con lo stesso nome e va lasciata intatta. Quindi `app_data_dir` è `%APPDATA%\it.sbobino.desktop` (roaming), mentre `app_local_data_dir` è `%LOCALAPPDATA%\it.sbobino.desktop` ed è anche la cartella dati della webview. Tauri sconsiglia un identifier che finisce in `.app`.
- **Progresso**: la percentuale viene da frame decodificati / `Track::num_frames`. Il demuxer MKV di Symphonia 0.6 non imposta `num_frames`, quindi per i MKV l'avanzamento è sempre indeterminato; idem per i WAV "in streaming" (lunghezze `0xFFFFFFFF`), che finiscono con un `UnexpectedEof` trattato come fine del file.

- **Lingue del modello**: transcribe-cpp 0.2.4 non legge i metadati senza caricare il modello, quindi `capabilities().languages` si conosce solo dopo un `Model::load`. Per questo il modello scelto si carica all'avvio. Oggi tutti e tre i modelli accettano le sei lingue dell'app: Nemotron come locale (`it-IT`, `pl-PL`…), Whisper e Parakeet come codici.
- **Prima Trascrizione con Whisper**: dopo il caricamento la prima Trascrizione dura parecchio di più (16 s contro 0,8 s sulla fixture, RTX 2070 SUPER con Vulkan), anche senza ricaricare il modello. Sembra il riscaldamento di Vulkan, non un ricaricamento: il log `<id> caricato in …` di `managers::models` dice quando il modello si carica davvero.

### Frontend e verifica manuale

- **Numeri in italiano**: con `{{count, number}}` Intl non raggruppa le migliaia sotto 10 000 (`minimumGroupingDigits` 2 nel CLDR italiano): 1234 resta "1234", 12345 diventa "12.345".
- **Watcher di `tauri dev`**: qualunque file cambi sotto `src-tauri/`, anche un TXT creato in `tests/fixtures` da una Trascrizione di prova, ricompila e riavvia l'app. I download in corso si interrompono e il `.partial` resta. Per le prove trascrivi copie delle fixture fuori da `src-tauri`.
- **Pilotare l'app**: con `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` la WebView2 di `bun tauri dev` si comanda via CDP. Il dialog di Sfoglia è nativo e UI Automation da PowerShell 5.1 vede i suoi controlli solo come `Pane`: si compila con Win32, `WM_SETTEXT` sull'`Edit` dentro il controllo 1148 e `BM_CLICK` sul pulsante 1 del dialog `#32770` "Apri". `SendKeys` non arriva, perché la finestra di Claude tiene il foreground. Per chiamare un comando senza passare dalla UI (per esempio un secondo `transcribe` durante un'Attività) c'è `window.__TAURI_INTERNALS__.invoke("transcribe", { source })`.
- **Vedere i Parziali**: con la GPU Nemotron trascrive molto più veloce del tempo reale e un Parziale resta a schermo per circa 250 ms. Per verificarli registra il valore dell'area a ogni `requestAnimationFrame` via CDP e usa un parlato lungo (frasi da 10–15 s), per esempio generato con `System.Speech` (voce "Microsoft Elsa Desktop") in una cartella fuori dal repo.
- **Chiudi tutto dopo la verifica**: fermare `bun tauri dev` può lasciare vivo il server Vite sulla porta 1420. Chiudi l'app e il Vite, e prima di lanciare una build controlla che non ci sia già un `tauri dev` attivo (anche dell'utente): il suo watcher ricompila a ogni modifica sotto `src-tauri/`.
- **`%APPDATA%` nell'app desktop di Claude**: anche il roaming è virtualizzato (MSIX). Le scritture di un processo lanciato da Claude finiscono in un livello privato, le letture vedono quel livello sopra il file reale. Spostare o cancellare `settings.json` da una shell di Claude può far ricomparire il file reale, più vecchio: per simulare un file illeggibile tienilo aperto in esclusiva (`[IO.File]::Open(p, 'Open', 'ReadWrite', 'None')` in PowerShell) mentre l'app parte, e non toccare il file reale.
- **Stub dei comandi via CDP**: `window.__TAURI_INTERNALS__` e il suo `invoke` non sono né scrivibili né configurabili, quindi non si può far fallire un comando dalla pagina. Il tema invece si prova con `Emulation.setEmulatedMedia` (`prefers-color-scheme`), nella stessa sessione CDP dello screenshot.
- **Selettori nativi via CDP**: per un `<select>` o un `<textarea>` controllato da React non basta assegnare `value`. Si usa il setter del prototipo (`Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value").set.call(el, v)`) e poi si emette un evento `change` (o `input`) con `bubbles: true`.

### Da verificare sull'hardware reale

- La deriva su sessioni di ore e con altri dispositivi (Bluetooth): il mixer la corregge a scatti da 20 ms (silenzio inserito o audio scartato), da sostituire con un resampler asincrono se gli scatti si sentono. Un dispositivo che consegna con oltre 500 ms di ritardo verrebbe scartato per intero (`LAG_NS`).
- Il loopback su un'uscita 5.1/7.1 (downmix con l'ordine dei canali assunto) e su un'uscita che cambia formato a metà Registrazione.
- Il pre-skip dell'OpusHead quando l'encoder lavora sotto i 48 kHz: Symphonia (test) e ffprobe ritrovano la durata esatta, `opusinfo` non è stato provato.
- L'installer su un PC Windows x64 pulito (una VM va bene), da fare a mano: senza Visual Studio né redistribuibile VC++, senza Vulkan (solo CPU) e se possibile con una CPU pre-AVX2 (moduli `ggml-cpu-sse42`/`x64`, ONNX Runtime ufficiale). Installa con doppio clic, scarica Nemotron da Impostazioni → Trascrizione, trascrivi un file e fai una Registrazione. La verifica di "Installer" gira sulla macchina di sviluppo, che ha già runtime VC++ e Vulkan.

## Agent skills

### Issue tracker

Le issue sono file markdown locali in `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Le cinque etichette canoniche predefinite (needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: un `CONTEXT.md` e `docs/adr/` alla root. See `docs/agents/domain.md`.
