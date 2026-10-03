# 01: Pipeline a frame con i tempi delle Frasi

**What to build:** lavoro preparatorio, senza cambi visibili per l'utente.
- La pipeline di Trascrizione accetta qualunque fonte di frame a 16 kHz mono, non più solo un file.
- La Trascrizione di un file diventa un involucro sottile sopra la pipeline generica.
- Ogni Frase e ogni Parziale portano `inizio_ms` e `fine_ms` sulla linea del tempo della Sorgente, insieme all'`id`.
- La pipeline accetta un segnale di "chiusura della Frase", che servirà alla Pausa dal vivo.

Trascrivi su un file deve funzionare esattamente come prima.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Pipeline generica su una fonte di frame; Trascrivi su un file passa da lì
- [ ] `inizio_ms`/`fine_ms` contati in frame da 30 ms, presenti negli eventi `transcript-phrase` e `transcript-partial` e in `bindings.ts`
- [ ] Segnale di chiusura della Frase: la Frase in corso si chiude come a fine parlato
- [ ] Test con fonte sintetica: tempi corretti, chiusura forzata; i test esistenti restano verdi
- [ ] Lo smoke test `#[ignore]` con i tre modelli passa, e Trascrivi in `bun tauri dev` è invariato

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
