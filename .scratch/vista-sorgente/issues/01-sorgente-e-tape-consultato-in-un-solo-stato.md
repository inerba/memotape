# 01: Sorgente e Tape consultato in un solo stato

**What to build:** Aprire, correggere, spostare e cestinare un Tape aggiorna con un'azione sola tutte le sue viste (Sorgente e Tape consultato), senza copie da tenere allineate a mano. Il comportamento visibile resta quello di oggi. Spec: `../spec.md`; ADR-0028.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Nasce il modulo puro della vista della Sorgente nella feature `source` (percorso della Sorgente e Tape consultato con testo e informazioni) e un reducer composto della route che lo unisce al modulo dell'Attività in un solo `useReducer`.
- [ ] **Sorgente aperta** è un'azione sola: aggiorna percorso, testo e informazioni e mette lo Status a "pronto" solo senza un'Attività in corso; senza percorso vale per il Cestino. Le coppie di dispatch `sourceOpened` + `status` spariscono.
- [ ] **Tape consultato aperto**, **Tape modificato** (testo e/o informazioni) e **Tape spostato** sono azioni sole applicate a ogni vista con quel percorso; con una sessione aperta o in chiusura la Sorgente non cambia. Spariscono `visibleSource` e gli aggiornamenti doppi di correzione, rinomina, unione di Turni, data e spostamento.
- [ ] Il Parlante in rinomina diventa stato locale della vista del Tape.
- [ ] Test del modulo e del reducer composto per le storie 9–12 e 15 della spec, attraverso le loro interfacce; nessun test sui componenti.
- [ ] I sei controlli passano.
- [ ] Prova manuale con `bun tauri dev`: correzione e rinomina sulla Sorgente; consultare e correggere un Tape durante una Registrazione; spostare e cestinare un Tape aperto e uno consultato.

## Comments
