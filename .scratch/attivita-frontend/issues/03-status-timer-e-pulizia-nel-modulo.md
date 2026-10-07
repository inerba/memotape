# 03: Status, timer e pulizia passano dal modulo dell'Attività

**What to build:** La fase in Attività, la percentuale, il timer della barra laterale e gli avvisi della pulizia audio seguono solo la sessione in corso, decisa dallo stesso modulo che governa il testo. La finestra principale non tiene più identificatori di sessione propri. Decisione: ADR-0028; spec: `../spec.md`.

**Blocked by:** 02 — Il testo dell'Attività passa dal modulo.

**Status:** ready-for-agent

- [ ] Il modulo possiede Status, tempo trascorso, preparazioni e guasti della pulizia e la chiusura del loro avviso; gestisce fase della Registrazione, progresso, inizio dell'analisi dei Parlanti, guasto della Trascrizione dal vivo (toglie i Parziali), tick, preparazione e guasto della pulizia, pausa e pulizia spenta per un Ingresso.
- [ ] Avvio, esito e ripristino portano anche lo Status: annullare la Preparazione o una Registrazione non partita riporta fase, avvisi di pulizia e sessione di prima. Le transizioni e gli avvisi esistenti dello Status restano dove sono e il modulo le usa.
- [ ] La finestra principale non ha più `recordingSession`, né `useState` di Status, timer e pulizia; i `setStatus` letterali diventano dispatch. `pendingRecording`, il ripristino dello stato precedente e le conferme restano alla finestra.
- [ ] La promessa "pronto" copre tutti i listener dell'hook, compresi inizio dell'analisi e Parlanti assegnati.
- [ ] La barra della Registrazione riceve preparazioni e guasti già filtrati e confronta direttamente l'identificatore di sessione nel proprio ascolto dei livelli; il confronto di sessione condiviso e il filtro della pulizia spariscono con i loro test.
- [ ] AGENTS.md: il paragrafo su `sessionId` e revisioni nel frontend diventa due righe sul modulo dell'Attività e sugli stati della sessione.
- [ ] I sei controlli passano. Prova manuale con `bun tauri dev`: Preparazione annullata, Registrazione con pulizia e guasto simulato o reale, Pausa, timer, completamento dopo Stop, Trascrivi su un file, Riconosci i parlanti su un Tape.

## Comments
