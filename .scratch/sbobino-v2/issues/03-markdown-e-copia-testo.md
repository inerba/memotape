# 03: Markdown e Copia testo

**What to build:** il risultato di una Trascrizione è `<nome Sorgente> trascrizione <N>.md` invece del TXT, formattato con un'intestazione e paragrafi. "Copia testo" copia testo semplice o Markdown secondo un'impostazione.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Renderer puro, con due uscite: Markdown e testo semplice. Riceve i metadati (titolo = nome della Sorgente, data, durata, modello, Lingua del parlato) e le Frasi con i tempi. Prevede già etichette di Ingresso e Parlante, che arriveranno con i ticket 06 e 07
- [ ] Paragrafo nuovo quando la pausa tra due Frasi supera i 2 s. Con le etichette, un paragrafo per turno (`**Etichetta:**`), con le Frasi consecutive della stessa voce unite
- [ ] `.md` con il primo N libero accanto alla Sorgente; il TXT sparisce del tutto, e il risultato del comando di Trascrizione restituisce il percorso del `.md`
- [ ] Impostazione `copia_come: testo | markdown` (default testo) in Impostazioni; "Copia testo" usa il renderer tramite un comando
- [ ] Test del renderer: intestazione, soglia di 2 s, turni e unione, testo semplice
- [ ] Verifica in `bun tauri dev`: il `.md` si apre bene in un visualizzatore Markdown, e Copia testo funziona nei due modi

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
