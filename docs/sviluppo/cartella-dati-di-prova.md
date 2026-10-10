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

## Avvio

Da PowerShell, nel repo, con Vite (`bun run dev`, porta 1420) già avviato e `target\debug\memotape.exe` compilato (`cargo build --manifest-path src-tauri/Cargo.toml`). Scegli una porta CDP libera: `Get-NetTCPConnection -State Listen -LocalPort 9231` non deve trovare nulla.

```powershell
$env:MEMOTAPE_DATA_DIR = "$env:TEMP\memotape-prova"
$env:WEBVIEW2_USER_DATA_FOLDER = "$env:TEMP\memotape-prova-webview"
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9231"
Start-Process src-tauri\target\debug\memotape.exe -WorkingDirectory src-tauri -PassThru
```

Poi pilota l'app via CDP sulla 9231 (vedi "Pilotare l'app" in `AGENTS.md`). Non usare `bun tauri dev`: il suo watcher ricompila e riavvia a ogni modifica sotto `src-tauri/`.

## Pulizia

Chiudi l'app di prova (per pid, non per nome: `memotape.exe` può essere anche quella dell'utente) e il Vite sulla 1420, poi cancella `<dir>` e la cartella della WebView. Per verificare che i dati veri non cambino, confronta `Get-FileHash "$env:APPDATA\it.memotape.desktop\settings.json"` prima e dopo.
