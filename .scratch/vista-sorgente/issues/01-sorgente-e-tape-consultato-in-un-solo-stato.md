# 01: Sorgente e Tape consultato in un solo stato

**What to build:** Aprire, correggere, spostare e cestinare un Tape aggiorna con un'azione sola tutte le sue viste (Sorgente e Tape consultato), senza copie da tenere allineate a mano. Il comportamento visibile resta quello di oggi. Spec: `../spec.md`; ADR-0028.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Nasce il modulo puro della vista della Sorgente nella feature `source` (percorso della Sorgente e Tape consultato con testo e informazioni) e un reducer composto della route che lo unisce al modulo dell'Attività in un solo `useReducer`.
- [x] **Sorgente aperta** è un'azione sola: aggiorna percorso, testo e informazioni e mette lo Status a "pronto" solo senza un'Attività in corso; senza percorso vale per il Cestino. Le coppie di dispatch `sourceOpened` + `status` spariscono.
- [x] **Tape consultato aperto**, **Tape modificato** (testo e/o informazioni) e **Tape spostato** sono azioni sole applicate a ogni vista con quel percorso; con una sessione aperta o in chiusura la Sorgente non cambia. Spariscono `visibleSource` e gli aggiornamenti doppi di correzione, rinomina, unione di Turni, data e spostamento.
- [x] Il Parlante in rinomina diventa stato locale della vista del Tape.
- [x] Test del modulo e del reducer composto per le storie 9–12 e 15 della spec, attraverso le loro interfacce; nessun test sui componenti.
- [x] I sei controlli passano.
- [ ] Prova manuale con `bun tauri dev`: correzione e rinomina sulla Sorgente; consultare e correggere un Tape durante una Registrazione; spostare e cestinare un Tape aperto e uno consultato.

## Comments

- 2026-10-08, implementazione: `src/features/source/view.ts` (percorso della Sorgente e Tape consultato, azioni `sourceOpened`, `tapeConsulted`, `tapeChanged`, `tapeMoved`, `tapeTrashed`) e `src/app/routes/home-state.ts` (reducer composto `home` e hook `useHomeState`); `useActivity` diventa `useActivityEvents(dispatch, onRecordingStarted)`, che registra solo i listener. Le azioni `sourceLoaded`, `sourceChanged` e `moved` del modulo dell'Attività restano interne al reducer composto.
  - Deviazioni: "un'Attività in corso" per lo Status pronto è la sessione aperta; durante una sessione una Sorgente aperta con un Tape (l'esito) ne prende il testo, senza Tape (Cestino, Ogg tenuto dopo una Registrazione) non tocca l'Attività. Il Cestino è un'azione `tapeTrashed`, che per la Sorgente vale come Sorgente aperta senza percorso. La rinomina locale del Parlante si annulla anche quando la vista smette di essere modificabile (Trascrivi e Riconosci sulla Sorgente). La chiusura del Tape consultato resta negli effetti di `home.tsx` e in "Attività in corso" fino al ticket 02, che la porta in `activityShown` e nelle reazioni alle fasi di `home-state.ts`.
  - Prova manuale da fare: correzione e rinomina sulla Sorgente; consultare e correggere un Tape durante una Registrazione; spostare e cestinare un Tape aperto e uno consultato.
