# 08: Diarizzazione delle Registrazioni

**What to build:** in Impostazioni → Registrazione l'utente sceglie cosa diarizzare: il mix, oppure, con Ingressi separati, Microfono e/o Audio di sistema. Dopo Stop, finito lo smaltimento della coda, la Diarizzazione gira sull'audio scelto e i Parlanti compaiono nelle Frasi già trascritte, nel Bino e nel Markdown.

**Blocked by:** 06, 07

**Status:** ready-for-agent

- [ ] Impostazioni `parlanti_mix`, `parlanti_microfono` e `parlanti_sistema` (default false), mostrate secondo la modalità e gli Ingressi
- [ ] Dopo Stop: smaltimento, poi Diarizzazione degli audio scelti, poi composizione del Bino. Lo stato "Riconoscimento dei parlanti…" è nella status bar, con Annulla
- [ ] Le etichette combinate (`Microfono · Parlante N`) nella conversazione e nel `.md`, e i Parlanti salvati nel Bino
- [ ] Aprendo e ritrascrivendo un Bino, la Diarizzazione segue `parlanti_file` sul `mix.ogg`
- [ ] Test: attribuzione per Ingresso, etichette combinate nel renderer
- [ ] Verifica in `bun tauri dev` con Ingressi separati e diarizzazione del solo Audio di sistema

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
