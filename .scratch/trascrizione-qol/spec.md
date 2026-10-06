# Trascrizione: correzioni manuali e qualità d'uso

Stato: implementata il 6 ottobre 2026; sei controlli automatici verdi e verifica interattiva dei componenti nel browser, nei due temi. Resta da verificare la finestra Windows nativa con un Tape reale. PRODUCT.md è la fonte di verità, CONTEXT.md definisce i termini e ADR-0017 registra la scelta di conservare le correzioni durante una nuova Diarizzazione. Esiti e limiti in `verifica.md`.

## Richieste confermate

- Durante la correzione del testo di una Frase, il cursore e la selezione devono essere chiaramente visibili.
- Il clic sul nome del Parlante nella Trascrizione deve permettere la rinomina sul posto. Lo spostamento nella pagina senza possibilità di rinominare è il difetto segnalato; la causa resta da verificare nell'interfaccia.
- Unisci serve a correggere un errore della Diarizzazione: il blocco selezionato viene attribuito al Parlante del Turno immediatamente sopra o sotto e unito a quel Turno. Per esempio, un intervento etichettato Anna viene unito al Turno sopra di Mario perché anche quell'intervento era di Mario.
- L'azione riguarda solo il blocco selezionato. Non riassegna gli altri interventi dello stesso Parlante nel Tape.
- Il comando propone l'unione sopra, l'unione sotto e Annulla, che chiude la scelta senza eseguire l'operazione. Il pulsante è desiderato prima di Copia turno.
- Le correzioni manuali, incluse la modifica del testo e l'unione per correggere l'attribuzione, devono essere registrate nel Tape. Il Tape riaperto deve ricordarle.
- Sotto il titolo, tra le informazioni che mostrano gli Ingressi e il numero di Parlanti, deve comparire la dicitura «Corretto a mano» quando ci sono correzioni manuali.
- Prima di rifare la Diarizzazione o la Trascrizione, l'app deve avvisare che ci sono correzioni manuali.
- Anche la rinomina di un Parlante conta come Correzione manuale.
- Una nuova Diarizzazione conserva testo corretto, attribuzioni manuali e nomi personalizzati; aggiorna le altre attribuzioni. La nuova Trascrizione può sostituirli soltanto dopo un avviso esplicito e la conferma dell'utente.

## Vincoli esistenti

- Frase e Turno mantengono i significati di CONTEXT.md: il Turno raggruppa Frasi consecutive dello stesso Ingresso e Parlante. L'unione richiesta riguarda il Turno selezionato, non la concatenazione di due singoli paragrafi.
- Microfono e Audio di sistema restano Ingressi distinti, secondo PRODUCT.md e ADR-0015.
- Le modifiche già presenti nel checkout restano il riferimento per la successiva implementazione.

## Comportamento dell'Unione di Turni

- Il pulsante Unisci sta nella riga del Turno, prima di Copia turno. Le voci sono «Unisci al turno sopra», «Unisci al turno sotto» e «Annulla».
- Il bersaglio è il Turno immediatamente adiacente nella vista, senza saltare interventi intermedi. Se non esiste, appartiene a un altro Ingresso o non ha un Parlante utilizzabile, la relativa direzione è disabilitata.
- La correzione conserva testo, paragrafi, intervalli, tempi ASR e riferimenti delle Frasi selezionate. Cambia la loro attribuzione e le registra come attribuite a mano. Non crea un'unica Frase con l'intervallo complessivo dei due Turni.
- Un Turno non attribuito può essere unito a un Turno con un Parlante noto dello stesso Ingresso. L'identità scelta dall'utente è manuale, non un'ipotesi provvisoria del modello.
- L'azione segue i vincoli di modifica del Tape: nessun Parziale e nessuna modifica mentre un'Attività lavora su quel Tape.
- Il risultato, i metadati manuali e l'indice si aggiornano insieme dopo il salvataggio. Un errore non deve lasciare metà del Turno riassegnato né mostrare una correzione come salvata.
- Annulla chiude il menu senza modifiche. Un comando separato per annullare un'unione già salvata è fuori perimetro.

## Persistenza e nuova analisi

- Il Tape distingue le correzioni del testo, le attribuzioni manuali e i nomi personalizzati: una modifica del solo testo non protegge automaticamente il Parlante.
- Le Frasi attribuite a mano non si dividono e non si riassegnano durante una nuova Diarizzazione. Le altre continuano a seguire i tempi ASR e i vincoli esistenti. Una correzione testuale continua a rimuovere soltanto i tempi riferiti al testo originale, come oggi.
- I nomi personalizzati vengono conservati senza assegnarli a una voce diversa per una coincidenza di numerazione tra analisi. Le attribuzioni protette restano collegate all'identità scelta dall'utente, anche se la nuova analisi rinumera le altre voci.
- «Corretto a mano» compare quando esiste una modifica manuale salvata al testo, all'attribuzione o al nome di un Parlante. Non compare solo perché si cambia titolo o data o si apre e chiude un campo senza modificarlo.
- Una nuova Diarizzazione conserva l'indicatore. Una nuova Trascrizione lo rimuove solo dopo la sostituzione riuscita del contenuto; annullamento o errore conservano il Tape precedente.
- Le copie, la riapertura, gli spostamenti e le riscritture per altri metadati conservano il dato. I Tape precedenti restano leggibili; non si deducono vecchie correzioni testuali dall'assenza di tempi ASR. I nomi personalizzati già presenti sono interventi manuali riconoscibili e vanno conservati.

## Avvisi

- **Diarizza / Diarizza di nuovo:** «Questo Tape contiene correzioni manuali. Il testo corretto, le attribuzioni modificate a mano e i nomi personalizzati saranno conservati. La Diarizzazione aggiornerà le altre attribuzioni.» Azioni: «Annulla» e «Diarizza».
- **Trascrivi di nuovo:** «Questo Tape contiene correzioni manuali. Una nuova Trascrizione sostituirà il testo, le attribuzioni e i nomi dei Parlanti, comprese le correzioni fatte a mano.» Azioni: «Annulla» e «Trascrivi di nuovo».
- Gli avvisi integrano la conferma esistente, senza due conferme consecutive per la stessa azione. Le sei lingue devono comunicare le stesse conseguenze.

## Criteri di accettazione per l'implementazione

1. Nei temi chiaro e scuro, clic sul testo, spostamento del cursore e selezione con mouse o tastiera restano chiaramente visibili. Invio e uscita dal campo salvano; Esc ripristina il testo, secondo il comportamento esistente.
2. Il clic sul nome del Parlante rende visibile e utilizzabile l'input sul posto, senza cambiare scheda o spostare inutilmente il documento. Verificare il difetto nella UI reale, senza attribuirlo preventivamente a una causa CSS o di navigazione.
3. Nell'esempio Mario–Anna–Mario, l'unione sopra attribuisce solo il Turno centrale a Mario. Un altro Turno di Anna altrove resta Anna; parole, paragrafi e salti audio delle Frasi conservano i loro riferimenti.
4. L'unione sotto funziona allo stesso modo. Prima/ultima posizione, Ingresso diverso e destinazione senza Parlante utilizzabile disabilitano soltanto la direzione non disponibile.
5. Correzione del testo, unione e rinomina fanno comparire l'indicatore dopo il salvataggio; riaprire o copiare il Tape lo conserva. Un errore di salvataggio non viene rappresentato come successo.
6. Diarizza mostra un unico avviso, conserva testo, attribuzioni protette e nomi, e può aggiornare gli altri Turni. La coincidenza di un numero di Parlante non trasferisce un nome manuale a una voce diversa. L'indicatore resta.
7. Trascrivi di nuovo avvisa della perdita delle correzioni. Annullare la conferma non avvia l'Attività. Annullamento o errore dell'Attività conserva le correzioni precedenti; la sostituzione riuscita rimuove il precedente dato manuale.
8. Copia testo, Copia turno, Markdown, ricerca e riapertura riflettono l'attribuzione corretta. I Tape precedenti restano leggibili.
9. Alla chiusura dei ticket: i sei controlli del repository verdi e una verifica manuale della UI Windows per caret, selezione, rinomina e menu. Nessuno di questi controlli è stato eseguito nella fase di documentazione.

## Fonti

- Conversazione del 6 ottobre 2026.
- PRODUCT.md: correzione delle Frasi, rinomina sul posto, Copia turno, Diarizzazione del testo esistente.
- CONTEXT.md: Frase, Turno, Ingresso, Parlante.
- docs/adr/0015-entrambi-si-trascrive-sempre-per-ingresso.md.
- docs/adr/0017-correzioni-manuali-della-trascrizione.md.
