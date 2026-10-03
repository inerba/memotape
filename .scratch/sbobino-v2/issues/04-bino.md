# 04: Bino

**What to build:** a Stop la Registrazione produce `Registrazione <data ora>.bino` nella Cartella predefinita, insieme al `.md` accanto. Con Sfoglia l'utente apre un Bino e ritrova il testo senza ritrascrivere. Può anche ritrascriverlo: il nuovo testo sostituisce quello dentro il Bino, previa conferma.

**Blocked by:** 02, 03

**Status:** done

- [x] Modulo Bino senza Tauri, con scrittura e lettura. Zip (crate `zip`) con `mix.ogg` salvato senza ricompressione e `trascrizione.json` nello schema v1 della spec (`version`, `creato`, `durata_ms`, `modalita`, `modello`, `lingua_parlato`, `completa`, `parlanti`, `frasi` con tempi, Ingresso e Parlante)
- [x] Durante la Registrazione gli Ogg si scrivono in temporanei in una cartella nascosta accanto alla destinazione. A Stop, dopo lo smaltimento, si compone il Bino e si cancellano i temporanei. Se si annulla il completamento, il Bino si salva con `completa: false` e il `.md` no
- [x] Sfoglia accetta `.bino`: il testo viene dal JSON. Trascrivi decodifica `mix.ogg` e riscrive il JSON in modo atomico. `version` futura → errore dedicato `unsupportedBino`; i campi sconosciuti si ignorano
- [x] Il clic sul nome di un Bino lo mostra nella cartella. Le vecchie Registrazioni `.ogg` si aprono ancora
- [x] Test: scrittura e lettura, campi sconosciuti, versione futura, `mix.ogg` decodificabile, riscrittura atomica, `completa: false`
- [x] Verifica in `bun tauri dev`: registra con la Trascrizione dal vivo, apri il Bino, ritrascrivilo

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.

## Esito

Verificato in `bun tauri dev` il 2026-10-03 (Whisper Large v3 Turbo, Registra da Entrambi, fixture TTS riprodotta sugli altoparlanti): durante la Registrazione l'Ogg sta in `Documenti\Sbobino\.sbobino` (nascosta); a Stop c'è un solo `.bino` (`mix.ogg` 48 272 byte non compresso, JSON compresso, `completa: true`) con il `.md` accanto, e la cartella nascosta sparisce. Sfoglia sul Bino con testo modificato nell'area chiede conferma e carica le due Frasi; Copia testo rende l'intestazione del Bino. Trascrivi con la Lingua del parlato Italiano riscrive il JSON (`lingua_parlato: "it"`, audio identico, nessun `.tmp`) e salva `trascrizione 2.md`. Annulla subito dopo Stop: Bino con `completa: false` e la Frase già pronta, niente `.md`. Clic sul nome: Esplora file con il Bino selezionato. Un `.ogg` vecchio si apre come prima; un Bino con `version: 2` mostra l'errore e lascia Sorgente e testo com'erano.
