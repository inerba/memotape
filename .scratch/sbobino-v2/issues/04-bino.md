# 04: Bino

**What to build:** a Stop la Registrazione produce `Registrazione <data ora>.bino` nella Cartella predefinita, insieme al `.md` accanto. Con Sfoglia l'utente apre un Bino e ritrova il testo senza ritrascrivere. Può anche ritrascriverlo: il nuovo testo sostituisce quello dentro il Bino, previa conferma.

**Blocked by:** 02, 03

**Status:** ready-for-agent

- [ ] Modulo Bino senza Tauri, con scrittura e lettura. Zip (crate `zip`) con `mix.ogg` salvato senza ricompressione e `trascrizione.json` nello schema v1 della spec (`version`, `creato`, `durata_ms`, `modalita`, `modello`, `lingua_parlato`, `completa`, `parlanti`, `frasi` con tempi, Ingresso e Parlante)
- [ ] Durante la Registrazione gli Ogg si scrivono in temporanei in una cartella nascosta accanto alla destinazione. A Stop, dopo lo smaltimento, si compone il Bino e si cancellano i temporanei. Se si annulla il completamento, il Bino si salva con `completa: false` e il `.md` no
- [ ] Sfoglia accetta `.bino`: il testo viene dal JSON. Trascrivi decodifica `mix.ogg` e riscrive il JSON in modo atomico. `version` futura → errore dedicato `unsupportedBino`; i campi sconosciuti si ignorano
- [ ] Il clic sul nome di un Bino lo mostra nella cartella. Le vecchie Registrazioni `.ogg` si aprono ancora
- [ ] Test: scrittura e lettura, campi sconosciuti, versione futura, `mix.ogg` decodificabile, riscrittura atomica, `completa: false`
- [ ] Verifica in `bun tauri dev`: registra con la Trascrizione dal vivo, apri il Bino, ritrascrivilo

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
