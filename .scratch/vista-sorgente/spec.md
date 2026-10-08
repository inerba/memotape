# La vista della Sorgente fuori dalla finestra principale

**Status:** done

Data: 2026-10-08. Candidato 6 della revisione architetturale, prima parte; grilling dello stesso giorno. Segue ADR-0028 (riga sulle conseguenze). Ticket: `issues/01-sorgente-e-tape-consultato-in-un-solo-stato.md`, `issues/02-navigazione-come-intenti.md`.

## Problema

Per l'utente aprire un Tape, consultarne un altro durante un'Attività, correggerlo, spostarlo o cestinarlo deve lasciare sempre la vista coerente: il testo giusto al centro, il Tape giusto evidenziato nella barra laterale, nessuna correzione persa o applicata alla copia sbagliata. Oggi la finestra principale tiene il percorso della Sorgente, il Tape consultato, la Frase evidenziata, il Parlante in rinomina, Home e Libreria aperte in variabili separate, e ogni apertura le azzera a mano con piccole differenze. Il testo e le informazioni della Sorgente stanno invece nel modulo dell'Attività: ogni modifica a un Tape deve chiedersi se quel percorso è la Sorgente e aggiornare a mano anche il Tape consultato. La navigazione verso un Tape del doppio clic o un file rilasciato e le reazioni all'avvio e alla fine di un'Attività vivono in effetti.

Per chi sviluppa nessuna di queste regole è testata, e la deriva fra le copie è lo stesso tipo di errore che ha prodotto il bug del testo dal vivo corretto per sbaglio (spec `attivita-frontend`).

## Soluzione

La vista della Sorgente diventa un modulo puro, composto con il modulo dell'Attività in un solo stato. Riceve intenti (apri, consulta, mostra la Libreria, Tape modificato, spostato, nel Cestino…) e dice quale vista c'è al centro e quale Tape è selezionato nella barra laterale. Le chiamate al backend restano alla finestra, che passa il risultato. Per l'utente il comportamento resta quello di oggi.

## Storie

1. Come utente, voglio che aprire un Tape dalla barra laterale, dall'elenco della Libreria, dalla ricerca o con il doppio clic in Esplora file lo renda la Sorgente con il suo testo, così ogni strada porta allo stesso risultato.
2. Come utente, voglio che aprire un Tape da un risultato della ricerca evidenzi la Frase trovata e la porti in vista.
3. Come utente, voglio che cliccare il Tape già aperto lo riporti all'inizio del testo, o alla Frase trovata.
4. Come utente, voglio che durante un'Attività un Tape della Libreria si apra come Tape consultato, accanto alla vista dell'Attività, senza cambiare la Sorgente.
5. Come utente, voglio che durante Trascrivi o Riconosci i parlanti aprire proprio la Sorgente riporti alla vista dell'Attività invece di consultarla.
6. Come utente, voglio che "Attività in corso" nella barra laterale riporti alla vista dell'Attività e tolga l'evidenziazione del Tape consultato.
7. Come utente, voglio che a fine Attività la vista torni alla Sorgente con il suo esito, chiudendo il Tape consultato.
8. Come utente, voglio che all'avvio di una Registrazione la sua vista prenda il posto di Home, Libreria e Tape consultato.
9. Come utente, voglio che correggere un Turno, rinominare un Parlante, unire Turni o cambiare la data valga per tutte le viste di quel Tape (Sorgente e consultato) e solo per quelle.
10. Come utente, voglio che durante una Registrazione o una Trascrizione le modifiche al Tape consultato non tocchino il testo dell'Attività.
11. Come utente, voglio che un Tape aperto e poi spostato o rinominato resti aperto con il percorso nuovo, come Sorgente o come Tape consultato.
12. Come utente, voglio che un Tape mandato nel Cestino si chiuda se era aperto, mentre la vista di un'Attività in corso resta.
13. Come utente, voglio che un Tape del doppio clic in Esplora file aspetti la fine dell'Attività e la chiusura di una conferma, e poi si apra sulla finestra principale anche se c'era Impostazioni sopra.
14. Come utente, voglio che un file rilasciato si apra come con Apri file, e durante un'Attività che un Tape rilasciato si apra come Tape consultato.
15. Come utente, voglio che Apri file renda il file la Sorgente, pronto da trascrivere.
16. Come utente, voglio che la barra laterale evidenzi il Tape che vedo al centro, e nessuno quando vedo la Libreria o la Registrazione.
17. Come utente, voglio che Home e Libreria si aprano e si chiudano senza perdere la Sorgente.
18. Come utente, voglio che la rinomina di un Parlante in corso si annulli quando apro un altro Tape o parte una Registrazione.
19. Come chi sviluppa Memotape, voglio che le regole della vista stiano in un modulo puro testato con `bun test`.
20. Come chi sviluppa Memotape, voglio che ogni modifica a un Tape sia un'azione sola, così non esistono copie da aggiornare a mano.
21. Come chi sviluppa Memotape, voglio che la finestra principale non contenga più effetti di navigazione né la catena che sceglie la vista centrale.

## Decisioni di implementazione

- **Due moduli, uno stato.** Il modulo della vista della Sorgente (feature `source`) contiene percorso della Sorgente, Tape consultato (percorso, testo, informazioni), Frase evidenziata, revisione, Home e Libreria aperte, Tape in attesa. Il modulo dell'Attività resta com'è. Un reducer composto nel livello della route li unisce in un solo `useReducer`; l'hook della route registra i listener dell'Attività come oggi.
- **Sorgente aperta**: una sola azione (percorso, e il Tape se è un Tape; senza percorso per il Cestino). La vista aggiorna percorso, Frase evidenziata, Home e Libreria; l'Attività aggiorna testo e informazioni e mette lo Status a "pronto" solo senza un'Attività in corso. L'errore di apertura resta un'azione a parte. Sostituisce le coppie `sourceOpened` + `status`.
- **Tape modificato**: una sola azione (percorso, e trasformazioni facoltative di testo e informazioni) applicata al Tape consultato se il percorso coincide e alla Sorgente se coincide il suo percorso, con la regola dell'Attività che la ignora durante una sessione. Sostituisce le funzioni che oggi aggiornano due copie.
- **Tape spostato**: una sola azione (da, a) per Sorgente, Tape consultato e Attività.
- **Navigazione come intenti puri**: apri dalla Libreria (con Frase facoltativa; consulta durante un'Attività, riparte dall'inizio se è già la Sorgente, torna alla vista dell'Attività se è la Sorgente su cui lavora Trascrivi o Riconosci), Tape consultato aperto, mostra l'Attività, mostra la Libreria, mostra la Home, Tape richiesto (doppio clic), file rilasciato (con il verdetto puro già esistente).
- **Vista centrale e selezione**: due funzioni pure sullo stato composto dicono quale vista c'è al centro (Preparazione, Libreria, Home, Tape consultato, vista dal vivo, Tape Sorgente, file, niente) e quale Tape è selezionato nella barra laterale. Sostituiscono le funzioni e la catena attuali della finestra.
- **Reazioni alle fasi**: dopo ogni azione il reducer composto confronta lo Status prima e dopo: all'avvio della Registrazione chiude Home, Libreria, Tape consultato e Frase evidenziata; a fine Attività chiude il Tape consultato. Spariscono i due effetti.
- **Tape in attesa**: stato del modulo; una funzione pura dice quando si può aprire (nessuna Attività, nessuna conferma aperta). L'effetto della finestra si limita a eseguire l'apertura e a tornare alla finestra principale.
- **Revisione**: il contatore che fa da chiave della vista del Tape passa nel modulo e cresce quando si riapre la Sorgente.
- **Parlante in rinomina**: diventa stato locale della vista del Tape, che si rimonta già da sola al cambio di Tape o di revisione.
- **IO fuori**: le chiamate al backend (aprire un Tape, scegliere un file, spostare, cestinare, correggere) restano alla finestra principale, che manda al reducer il risultato.
- **Glossario**: aggiunto **Tape consultato**. Nessun ADR nuovo; una riga in ADR-0028.

## Decisioni sui test

- Un buon test applica una sequenza di intenti e risultati come li produrrebbe l'utente e controlla vista centrale, Tape selezionato, testo e informazioni visibili; non controlla campi interni oltre quanto descrive il comportamento.
- Test del modulo della vista: vista centrale in ogni combinazione; Tape selezionato; apri dalla Libreria con e senza Attività; stesso Tape due volte; Tape spostato e nel Cestino, anche durante un'Attività; Tape in attesa che si apre solo senza Attività e senza conferma; file rilasciato con e senza Attività.
- Test del reducer composto: Sorgente aperta con e senza Attività (Status); Tape modificato sulla Sorgente, sul consultato e su entrambi, ignorato durante una sessione; reazioni all'avvio della Registrazione e alla fine dell'Attività.
- Precedenti: i test del modulo dell'Attività (`*.test.ts` accanto al codice, `bun test`). Nessun test sui componenti.
- Prova manuale con `bun tauri dev` (via CDP come nella spec `attivita-frontend`): aprire dalla barra laterale, dalla ricerca e dalla Libreria; consultare e correggere un Tape durante una Registrazione; spostare e cestinare un Tape aperto; Tape del doppio clic durante un'Attività; Home e Libreria.

## Fuori perimetro

- Secondo passo del candidato 6: conferme di Trascrivi e Diarizza, `cancelling`, ciclo di Registrazione, Trascrivi e Riconosci, avvisi e loro timer.
- Stato puramente visivo della finestra: copia riuscita, trascinamento in corso, aggiornamento disponibile.
- Il ridisegno della finestra a ogni tick della Registrazione.
- Qualunque cambio del comportamento visibile.

## Note

- La scelta fra consultare e aprire dipende dall'Attività in corso, quindi la vista legge lo Status dallo stato composto e non ne tiene una copia.
