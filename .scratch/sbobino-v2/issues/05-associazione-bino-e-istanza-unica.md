# 05: Associazione `.bino` e istanza unica

**What to build:** con l'app installata, il doppio clic su un `.bino` in Esplora file apre Sbobino con quel Bino come Sorgente. Se Sbobino è già aperto, il file arriva alla finestra esistente invece di aprirne un'altra.

**Blocked by:** 04

**Status:** done

- [x] `fileAssociations` di Tauri per `.bino` nel bundle NSIS, con descrizione e icona dell'app
- [x] `tauri-plugin-single-instance`: un secondo avvio con un percorso porta la finestra esistente in primo piano e apre il file come Sorgente, con la conferma se l'area contiene testo
- [x] Primo avvio con un percorso negli argomenti: il file si apre come Sorgente
- [x] Verifica con l'installer, nella procedura isolata di `AGENTS.md`: installa, doppio clic su un Bino ad app chiusa e ad app aperta, disinstalla (l'associazione deve sparire)

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.

## Esito

Verificato il 2026-10-03, dopo le correzioni della code review. In `bun tauri dev`: avvio con un Bino negli argomenti (percorso con spazi) → Bino aperto con il suo testo; secondo avvio di `target\debug\sbobino.exe` con un altro Bino, a finestra ridotta a icona → il processo esce, la finestra torna in primo piano e chiede la conferma. Con Impostazioni aperta la finestra torna a `/` con la conferma; un terzo Bino arrivato con la conferma aperta aspetta e chiede la sua dopo Sostituisci. Durante una Registrazione il Bino arrivato aspetta e a Stop compare la conferma.

Con l'installer, nella procedura isolata: l'installer scrive `HKCU\Software\Classes\.bino` → `Sbobino.Bino` ("Bino (Sbobino)", icona `sbobino.exe,0`, comando `sbobino.exe "%1"`). Aprire un Bino con ShellExecute (lo stesso percorso del doppio clic) ad app chiusa avvia l'app installata con il Bino; ad app aperta e ridotta a icona resta un solo processo, la finestra torna in primo piano e chiede conferma. Installato due volte di seguito (il backup diventa `Sbobino.Bino`) e poi disinstallato con `uninstall.exe /S`: `.bino`, `Sbobino.Bino` e la chiave di disinstallazione spariscono, i modelli restano. Il doppio clic con il mouse in Esplora file non è stato provato da qui.
