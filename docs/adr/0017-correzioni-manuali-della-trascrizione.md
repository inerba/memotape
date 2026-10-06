---
status: accepted
---

# Una nuova Diarizzazione conserva le correzioni manuali

La Diarizzazione può attribuire un intervento al Parlante sbagliato. L'utente corregge il solo Turno selezionato unendolo a quello sopra o sotto, modifica il testo oppure dà un nome al Parlante. Queste correzioni vengono registrate nel Tape e mostrate con «Corretto a mano» sotto il titolo; una nuova Diarizzazione le conserva e aggiorna le altre attribuzioni. Decisione approvata il 6 ottobre 2026.

L'Unione di Turni conserva le Frasi, il loro testo e gli intervalli audio: riassegna solo il Turno scelto e non unisce tutte le occorrenze di due Parlanti. Gli Ingressi restano distinti, secondo ADR-0015. Una Frase attribuita a mano non può essere suddivisa o riassegnata dalla nuova analisi; una correzione del solo testo non fissa invece il Parlante.

Conservare un nome non significa associare il vecchio numero di Parlante al nuovo numero uguale. Le identità della nuova analisi possono cambiare: i nomi manuali e le attribuzioni protette devono conservare il loro significato, senza nominare automaticamente una voce diversa. I metadati distinguono testo corretto, attribuzione corretta e nome personalizzato; un solo indicatore generico non basta a sapere cosa proteggere.

## Implementazione

Le Frasi del Tape v1 hanno i campi facoltativi `testo_corretto` e `parlante_corretto`, falsi nei Tape precedenti. I nomi personalizzati restano nella mappa `parlanti`. `TapeInfo.correttoAMano` deriva da questi dati; titolo e data non lo modificano.

La nuova Diarizzazione conserva le Frasi attribuite a mano e i loro riferimenti. Per le altre, una corrispondenza temporale uno-a-uno tra voce vecchia e nuova conserva l'identità personalizzata. Quando la corrispondenza è ambigua, il nome rimane nel Tape con il numero riservato, ma non viene assegnato a una voce nuova: questa mostra l'etichetta generica. Non si deduce un'identità dalla sola coincidenza dei numeri. Le eventuali nuove parti hanno nuovi id senza rinumerare le Frasi esistenti.

La sostituzione per Trascrivi di nuovo usa la riscrittura annullabile del Tape: Annulla durante la copia impedisce la sostituzione, conservando anche le correzioni manuali.

## Alternative considerate

- Avvisare e poi sovrascrivere le attribuzioni manuali: approccio più semplice, ma obbliga l'utente a ripetere le correzioni dopo ogni analisi.
- Unire globalmente due Parlanti: corregge anche interventi che potrebbero essere stati attribuiti bene. La correzione richiesta è locale al Turno scelto.
- Unire fisicamente le Frasi: cambia paragrafi e riferimenti audio senza risolvere meglio l'errore di attribuzione.

## Conseguenze

- Diarizza e Diarizza di nuovo avvisano della presenza di Correzioni manuali e le conservano. Il testo corretto resta; le sole correzioni del testo non impediscono di aggiornare il Parlante.
- Trascrivi di nuovo è l'azione che può sostituire testo, attribuzioni e nomi, dopo un avviso esplicito. Il dato di correzione si rimuove soltanto quando la nuova Trascrizione sostituisce con successo il contenuto precedente.
- Il Tape deve registrare la provenienza delle correzioni e conservarla nelle riscritture e nelle copie. I Tape precedenti restano leggibili; l'assenza di tempi ASR non prova che il testo sia stato corretto a mano.
- La voce «Annulla» del menu chiude la scelta prima dell'unione. Una funzione separata per annullare un'unione già salvata resta fuori da questa richiesta.
