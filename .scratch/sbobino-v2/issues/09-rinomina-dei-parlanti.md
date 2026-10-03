# 09: Rinomina dei Parlanti

**What to build:** dopo una Trascrizione con Parlanti, l'utente clicca su un'etichetta "Parlante N" e la rinomina, per esempio in "Mario". Il nome vale per tutte le Frasi di quel Parlante, nell'area di testo, nel Bino e nell'ultimo Markdown della Sorgente.

**Blocked by:** 08

**Status:** ready-for-agent

- [ ] Clic su un'etichetta: modifica inline del nome, con Invio per confermare ed Esc per annullare; nomi vuoti non ammessi
- [ ] La rinomina vale per il Parlante di quell'Ingresso (`<ingresso>:<n>`), si salva in `parlanti` nel Bino se la Sorgente è un Bino, e riscrive l'ultimo `.md` prodotto per quella Sorgente
- [ ] Un Bino riaperto mostra i nomi
- [ ] Test: renderer con i nomi, scrittura e lettura di `parlanti` nel Bino, logica pura della rinomina nel frontend
- [ ] Verifica in `bun tauri dev`

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
