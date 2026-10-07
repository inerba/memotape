# 02: Il testo dell'Attività passa dal modulo, con sessione esplicita e modifiche protette

**What to build:** Il testo che arriva da una Registrazione, da Trascrivi o da Riconosci i parlanti entra nella vista solo se appartiene alla sessione in corso, decisa da un modulo puro dell'Attività con stati espliciti (nessuna, aperta, in chiusura, chiusa). Correggere o rinominare il Tape consultato durante una Registrazione non tocca più il testo dal vivo. Lo Status resta per ora alla finestra principale. Decisione: ADR-0028; spec: `../spec.md`.

**Blocked by:** 01 — Via lo strato dei Parlanti dal vivo, resta lo snapshot finale.

**Status:** ready-for-agent

- [ ] Primo test, rosso sul codice attuale: durante una Registrazione, una modifica al Tape consultato con lo stesso percorso della Sorgente precedente non cambia il testo dal vivo.
- [ ] Nasce la feature `activity`: un reducer puro con sessione (stato, identificatore assente per Trascrivi e Riconosci i parlanti, Parziali sì/no fissato all'avvio) e Conversation, e un hook che registra i listener del testo (Frase, Parziale, snapshot finale, Parlanti assegnati) e offre la promessa "pronto".
- [ ] Azioni dai comandi: avvio, esito, ripristino, Sorgente aperta (Frasi e nomi di un Tape, o vuota per un file), Sorgente modificata (trasformazione pura della Conversation, ignorata con una sessione aperta o in chiusura).
- [ ] Gli eventi tardivi si scartano per stato: dopo l'esito una Trascrizione è chiusa; una Registrazione resta in chiusura e accetta solo il proprio snapshot finale, una volta per Ingresso, finché non si apre un'altra Sorgente. Il nome predefinito del Microfono è nel testo dal primo istante della Registrazione.
- [ ] La finestra principale non ha più `acceptPartials` né il proprio `useState` della Conversation; aperture, correzioni, rinomine e Tape riaperto passano dalle due azioni della Sorgente. `browsed` resta com'è.
- [ ] Test sostituiti, non sommati: i casi di sessione dei test della Conversation passano nei test del modulo; le trasformazioni della Conversation perdono il controllo di sessione e restano testate senza.
- [ ] I sei controlli passano. Prova manuale con `bun tauri dev`: Registrazione con e senza Trascrivi dal vivo, correzione del Tape precedente consultato durante la Registrazione, Stop con Tape salvato, Trascrivi su un file con Annulla, Riconosci i parlanti su un Tape.

## Comments
