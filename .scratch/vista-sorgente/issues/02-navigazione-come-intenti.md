# 02: La navigazione come intenti, senza effetti

**What to build:** Quale vista c'è al centro e quale Tape è evidenziato nella barra laterale li decide il modulo della vista della Sorgente, a partire da intenti puri. La finestra principale non ha più effetti di navigazione né la catena che sceglie la vista. Il comportamento visibile resta quello di oggi. Spec: `../spec.md`.

**Blocked by:** 01 — Sorgente e Tape consultato in un solo stato.

**Status:** ready-for-agent

- [ ] Intenti nel modulo: apri dalla Libreria (con Frase facoltativa: consulta durante un'Attività, riparte dall'inizio se è già la Sorgente, torna alla vista dell'Attività se è la Sorgente su cui lavora Trascrivi o Riconosci), mostra l'Attività, mostra la Libreria, mostra la Home, Tape richiesto dal doppio clic, file rilasciato (con il verdetto puro esistente). Frase evidenziata, revisione, Home e Libreria aperte passano nel modulo.
- [ ] Due funzioni pure sullo stato composto danno la vista centrale (Preparazione, Libreria, Home, Tape consultato, vista dal vivo, Tape Sorgente, file, niente) e il Tape selezionato; sostituiscono le funzioni e la catena attuali della finestra.
- [ ] Le reazioni all'avvio della Registrazione e alla fine dell'Attività stanno nel reducer composto; spariscono i due effetti.
- [ ] Il Tape in attesa è stato del modulo; una funzione pura dice quando aprirlo (nessuna Attività, nessuna conferma aperta) e l'effetto si limita a eseguire l'apertura e a tornare alla finestra principale.
- [ ] Test per le storie 1–8, 13, 14, 16–18 della spec attraverso le interfacce dei moduli; nessun test sui componenti.
- [ ] AGENTS.md: la descrizione delle viste di `home.tsx` rimanda al modulo della vista della Sorgente e al Tape consultato.
- [ ] I sei controlli passano.
- [ ] Prova manuale con `bun tauri dev`: aprire dalla barra laterale, dalla ricerca e dalla Libreria; riaprire il Tape già aperto; consultare durante Trascrivi e Registrazione e tornare all'Attività; Tape del doppio clic durante un'Attività; Home e Libreria.

## Comments
