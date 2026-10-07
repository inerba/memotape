# Rilasciare Memotape

Dalla root del repository, in PowerShell:

```powershell
.\scripts\release.ps1 -Version 0.2.0
```

Chiudi `tauri dev` e attendi la fine di altre build Rust. Servono i prerequisiti
di build elencati in `AGENTS.md`, dipendenze frontend installate e configurazione
locale `.cargo/config.toml`.

Lo script aggiorna `package.json`, `src-tauri/Cargo.toml` e la voce Memotape in
`src-tauri/Cargo.lock`. Tauri usa la versione Cargo quando la configurazione
non ne specifica una; se viene aggiunta, lo script aggiorna anche quella.
Esegue i sei controlli del progetto in sequenza, poi genera:

```text
src-tauri/target/release/bundle/nsis/Memotape_0.2.0_x64-setup.exe
```

Per vedere l'operazione senza modificare file o compilare:

```powershell
.\scripts\release.ps1 -Version 0.2.0 -WhatIf
```

`-SkipChecks` salta i controlli, se li hai gia eseguiti sulla stessa revisione.
Al primo errore lo script si ferma. Le modifiche di versione restano disponibili
per la revisione e puoi ripetere il comando; non ripristina file automaticamente.
Accetta versioni stabili `X.Y.Z`, impedisce downgrade e rifiuta tag locali esistenti.

## Tag e GitHub Release

Lo script non crea commit o tag e non pubblica. Controlla `git diff`, prova
l'installer, quindi includi le modifiche di versione nel commit da rilasciare.
Con il commit corretto in HEAD e il repository pulito, esegui i comandi stampati
dallo script: creano il tag annotato `vX.Y.Z`, inviano commit e tag a `origin`
e pubblicano una GitHub Release con l'installer allegato. L'ultimo comando
richiede GitHub CLI (`gh`) autenticata; puoi anche creare la release dal sito.

Un tag da solo non pubblica l'installer. Il controllo aggiornamenti di Memotape
legge l'ultima GitHub Release pubblicata, ignorando bozze e prerelease.
Prima della pubblicazione risolvi il segnaposto `bundle.publisher`
(`EDITORE DA DEFINIRE`) nella configurazione Tauri. La build non verifica
l'installazione su un PC Windows pulito.
