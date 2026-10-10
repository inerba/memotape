# Cartella dati di prova

Per provare l'app di debug senza toccare i dati dell'utente: registrare, trascrivere, rinominare, cestinare. Vale solo nelle build di debug (`cfg(debug_assertions)`); in release `MEMOTAPE_DATA_DIR` non ha effetto. Codice: `src-tauri/src/cartelle.rs`.

## Cosa sposta

Con `MEMOTAPE_DATA_DIR=<dir>` (relativa alla cwd se non assoluta):

| Dato | Senza override | Con override |
|---|---|---|
| `settings.json` e `.json.tmp` | `%APPDATA%\it.memotape.desktop` | `<dir>` |
| Indice della Libreria | `%LOCALAPPDATA%\it.memotape.desktop\libreria` | `<dir>\indice` |
| Cartella della Libreria predefinita | `Documenti\Memotape` | `<dir>\Libreria` |

`indice` e `Libreria` si creano all'avvio, `settings.json` al primo salvataggio. La Cartella della Libreria predefinita vale solo se `settings.json` non ne indica una: non scegliere in Impostazioni la Libreria vera dell'utente. Il server MCP (`memotape.exe --mcp`) segue lo stesso override. All'avvio il log dice `cartella dati di prova attiva: <dir>`.

## Cosa resta condiviso

- **Modelli**: quelli veri in `%APPDATA%\it.memotape.desktop\models`, per non riscaricarli. Con l'override Scarica ed Elimina modello rispondono `internal` e non toccano i file: l'app dell'utente può avere lo stesso modello caricato o in download (stesso `.partial`).
- **Log**: `%LOCALAPPDATA%\it.memotape.desktop\logs\Memotape.log`, lo stesso dell'app dell'utente.
- **Dati della WebView** (`localStorage`): in `%LOCALAPPDATA%\it.memotape.desktop` se non imposti `WEBVIEW2_USER_DATA_FOLDER`.
- **Istanza unica**: con l'override il plugin non si carica, quindi l'app di prova parte anche con quella dell'utente aperta (e non riceve i Tape aperti da Esplora file).

## Avvio e pulizia

Con `target\debug\memotape.exe` compilato (`cargo build --manifest-path src-tauri/Cargo.toml`):

```
bun run app:prova
```

Lo script (`scripts/app-prova.ts`) sceglie una porta CDP libera tra 9300 e 9399, crea `%TEMP%\memotape-prova-<porta>` con `dati` (`MEMOTAPE_DATA_DIR`) e `webview` (`WEBVIEW2_USER_DATA_FOLDER`), avvia Vite se la 1420 è libera, lancia l'exe e aspetta che il frontend sia montato (un Vite appena avviato può metterci un minuto). Si ferma, chiudendo l'app, se l'exe non usa la cartella di prova (exe compilato prima di `MEMOTAPE_DATA_DIR`). Stampa porta, pid e cartella. Se Vite era già attivo lo usa e avvisa: può servire il frontend di un altro checkout.

Pilota l'app con `bun scripts/cdp.ts <porta> eval "<js>"` o `shot <file.png> [w h] [dark|light]`; da un altro script importa `valuta` e `fotografa` da `scripts/cdp.ts`, invece di passare il JavaScript a un processo figlio. Non usare `bun tauri dev`: il suo watcher ricompila e riavvia a ogni modifica sotto `src-tauri/`.

Alla fine `bun run app:prova stop <porta>` chiude app, WebView2 e il Vite che ha avviato, e cancella la cartella. Per verificare che i dati veri non cambino, confronta `Get-FileHash "$env:APPDATA\it.memotape.desktop\settings.json"` prima e dopo.
