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
Esegue i sei controlli del progetto in sequenza, chiede la password della chiave
degli aggiornamenti (`$env:USERPROFILE\.tauri\memotape.key`, ADR-0036) e genera, firmati:

```text
src-tauri/target/release/bundle/nsis/Memotape_0.2.0_x64-setup.exe
src-tauri/target/release/bundle/nsis/Memotape_0.2.0_x64-setup.exe.sig
src-tauri/target/release/bundle/nsis/latest.json
```

`latest.json` è il manifest che le app leggono da
`releases/latest/download/latest.json`: versione, data, firma dell'installer e
suo indirizzo nella release `vX.Y.Z`. La password non si salva: senza la chiave,
o con la password sbagliata, la build si ferma. Se la chiave o la password si
perdono, le app installate non accettano più aggiornamenti (vedi l'ADR).

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
e creano una GitHub Release in bozza con installer, firma e `latest.json`
allegati e le note di `.scratch/release-X.Y.Z.md`. L'ultimo comando richiede
GitHub CLI (`gh`) autenticata. Controlla la bozza e pubblicala dal sito: solo
allora le app la vedono.

Un tag da solo non pubblica l'installer. L'Aggiornamento di Memotape legge
`latest.json` dall'ultima GitHub Release pubblicata: bozze e prerelease non
contano, e una release senza `latest.json` non arriva alle app.
Prima della pubblicazione risolvi il segnaposto `bundle.publisher`
(`EDITORE DA DEFINIRE`) nella configurazione Tauri. La build non verifica
l'installazione su un PC Windows pulito.
