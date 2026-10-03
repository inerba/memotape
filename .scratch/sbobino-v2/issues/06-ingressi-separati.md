# 06: Ingressi separati

**What to build:** l'utente che registra da Entrambi sceglie in Impostazioni → Registrazione la modalità "Ingressi separati". Microfono e audio di sistema si trascrivono in parallelo, ognuno con la sua istanza del modello, e il testo appare come una conversazione: un messaggio per Frase con l'etichetta "Microfono" o "Audio di sistema", in ordine di inizio. Il Bino contiene anche l'audio di ogni Ingresso.

**Blocked by:** 04

**Status:** done

- [x] Impostazione `modalita_dal_vivo: mix | ingressi_separati` (default mix), attiva solo con Entrambi
- [x] Il mixer espone un flusso allineato per ogni Ingresso (silenzio nei buchi, pause escluse), che alimenta una pipeline e un Ogg per Ingresso
- [x] Il manager del modello fornisce una seconda istanza dello stesso modello per la durata della Registrazione, poi la libera
- [x] Eventi delle Frasi con `ingresso`. Vista a conversazione nel frontend, ordinata per `inizio_ms` e raggruppata per voce consecutiva, con logica pura testata
- [x] Il Bino ha `microfono.ogg` e `sistema.ogg` e `modalita: ingressi_separati`; riaperto, mostra la conversazione. Il `.md` ha turni `**Microfono:**` e `**Audio di sistema:**`
- [x] Test: flusso per Ingresso allineato al mix, due fonti con Frasi in ordine di inizio
- [x] Verifica in `bun tauri dev` con microfono e audio di sistema insieme

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.

## Esito

- La vista a conversazione è il testo dell'area (etichetta su una riga a ogni cambio di voce), così resta modificabile e Copia testo funziona come prima.
- Verifica del 2026-10-03 in `bun tauri dev` (Nemotron, Entrambi, microfono KLIM Talk e uscita Focusrite): seconda istanza caricata in 0,8 s all'avvio della Registrazione, Parziali e Frasi dell'audio di sistema con l'etichetta, Bino con `mix.ogg`, `microfono.ogg`, `sistema.ogg` non compressi e `modalita: ingressi_separati`, Markdown con `**Audio di sistema:**`. Il microfono era in cuffia e non ha avuto parlato: la conversazione a due voci si è vista riaprendo una copia del Bino con una Frase del microfono aggiunta, e in Copia testo. Resta da provare parlando davvero al microfono durante una call.
