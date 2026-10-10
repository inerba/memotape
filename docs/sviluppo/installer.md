# Installer

Build NSIS, cosa installa, editore, WebView2, disinstallazione e verifica sulla macchina di sviluppo. Riferimento per gli agenti, spostato da `AGENTS.md` il 10 ottobre 2026: aggiornalo quando cambia il comportamento che descrive.

- `bun tauri build` compila frontend ed exe in release e produce solo l'installer NSIS (`bundle.targets: ["nsis"]`): `src-tauri/target/release/bundle/nsis/Memotape_<versione>_x64-setup.exe`, circa 15 MB (90 MB una volta installato, di cui 44 di `ggml-vulkan.dll`). La prima volta la CLI di Tauri scarica NSIS e `nsis_tauri_utils`; la prima release compila transcribe-cpp da capo (circa 6 minuti). È una build Rust come le altre: niente `tauri dev` attivo intanto.
- Installa per l'utente corrente (`installMode` predefinito `currentUser`) in `%LOCALAPPDATA%\Memotape`, senza UAC: `memotape.exe` con accanto le DLL di `runtime-libs/`, `resources\silero_vad.onnx` e `licenses\*.txt`. L'elenco dei file è in `src-tauri/target/release/nsis/x64/installer.nsi`; `7z l` sull'installer mostra cosa contiene davvero.
- **Editore**: `bundle.publisher` è il segnaposto "EDITORE DA DEFINIRE" (si vede in App installate ed è nella chiave `HKCU\Software\<editore>\Memotape`). Va deciso prima della prima release pubblica: cambiarlo dopo lascia la chiave del vecchio editore sui PC già installati. L'installer non è firmato (fuori perimetro).
- WebView2: l'installer usa il bootstrapper predefinito, quindi su un Windows 10 che non ce l'ha la scarica durante l'installazione (serve la rete). Windows 11 ce l'ha già.
- La disinstallazione toglie i file installati e lascia impostazioni e modelli (`%APPDATA%\it.memotape.desktop`) e i dati della WebView, a meno di spuntare "Delete the application data" nel dialog; con `/S` non si cancellano.

## Verifica dell'installer

Sulla macchina di sviluppo, senza toccare l'app di sviluppo:

1. Installa in silenzio in una cartella di prova (per esempio nella scratchpad): `Start-Process <setup.exe> -ArgumentList "/S","/D=<cartella>" -Wait`. `/D=` va per ultimo e senza virgolette. Prima di installare e disinstallare l'installer chiude un `memotape.exe` in esecuzione solo se è quello di `<cartella>`, non quello di `target\debug`.
2. Avvia `<cartella>\memotape.exe` da PowerShell dopo aver tolto `ORT_LIB_LOCATION`, `VULKAN_SDK` e `ORT_PREFER_DYNAMIC_LINK` e aver ripulito il `PATH` dal Vulkan SDK. Imposta `WEBVIEW2_USER_DATA_FOLDER` su una cartella di prova, così la WebView non condivide i dati con l'app di sviluppo, e `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9223`.
3. `(Get-Process -Id <pid>).Modules`: `onnxruntime.dll`, `transcribe.dll`, `ggml*.dll`, `msvcp140*.dll` e `vcruntime140*.dll` devono venire da `<cartella>`, `vulkan-1.dll` da System32. Il log (`%LOCALAPPDATA%\it.memotape.desktop\logs\Memotape.log`, lo stesso dell'app di sviluppo) riporta `backend di transcribe-cpp: Vulkan0 (…), CPU (…)` e il caricamento del modello.
4. Via CDP sulla 9223 (vedi «Pilotare l'app» in `verifica-manuale.md`): `transcribe` su una copia di `parlato-it.wav` fuori dal repo, poi `record`, qualche secondo e `stop_recording`. Cancella il Tape della Trascrizione e la Registrazione di prova da `Documenti\Memotape`.
5. Chiudi l'app e lancia `<cartella>\uninstall.exe /S`: la cartella e la chiave `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Memotape` spariscono, i modelli restano.

L'app installata usa la stessa `%APPDATA%\it.memotape.desktop` dell'app di sviluppo, quindi anche i modelli già scaricati: `app_data_dir` viene da `FOLDERID_RoamingAppData` e una variabile d'ambiente non lo sposta. Durante la prova non cambiare le impostazioni. La finestra dell'app installata compare sullo schermo dell'utente, che può usarla: un `activityInProgress` inatteso viene da lì.

Esito del 2026-10-03 (Ryzen 7 3700X, RTX 2070 SUPER): tutte le DLL caricate dalla cartella d'installazione, backend Vulkan0 e CPU (modulo `ggml-cpu-haswell`), Nemotron caricato in 1,1 s, fixture trascritta in 1,3 s con il TXT salvato, Registrazione di 4 s salvata e ritrascritta, disinstallazione pulita. Manca la prova su un PC pulito (vedi «Da verificare sull'hardware reale» in `verifica-manuale.md`).
