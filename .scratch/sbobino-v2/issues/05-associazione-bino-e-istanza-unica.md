# 05: Associazione `.bino` e istanza unica

**What to build:** con l'app installata, il doppio clic su un `.bino` in Esplora file apre Sbobino con quel Bino come Sorgente. Se Sbobino è già aperto, il file arriva alla finestra esistente invece di aprirne un'altra.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] `fileAssociations` di Tauri per `.bino` nel bundle NSIS, con descrizione e icona dell'app
- [ ] `tauri-plugin-single-instance`: un secondo avvio con un percorso porta la finestra esistente in primo piano e apre il file come Sorgente, con la conferma se l'area contiene testo
- [ ] Primo avvio con un percorso negli argomenti: il file si apre come Sorgente
- [ ] Verifica con l'installer, nella procedura isolata di `AGENTS.md`: installa, doppio clic su un Bino ad app chiusa e ad app aperta, disinstalla (l'associazione deve sparire)

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
