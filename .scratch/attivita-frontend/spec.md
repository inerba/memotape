# Gli eventi dell'Attività in un solo modulo del frontend

**Status:** done

Data: 2026-10-08. Nasce dalla revisione architetturale (candidato 1) e dal grilling dello stesso giorno. Decisioni: ADR-0016 (revisione dell'8 ottobre) e ADR-0028. Ticket: `issues/01-via-lo-strato-dei-parlanti-dal-vivo.md`, `issues/02-modulo-dell-attivita-nel-frontend.md`, `issues/03-status-timer-e-pulizia-nel-modulo.md`.

## Problema

Per l'utente il testo di una Registrazione o di una Trascrizione deve essere quello giusto, anche quando gli eventi arrivano in ritardo, dopo Stop o dopo aver aperto un altro Tape. Oggi la finestra principale decide quali eventi accettare con tre identificatori di sessione e un flag, sparsi in variabili che comandi diversi azzerano o ripristinano in punti diversi. Un evento tardivo viene scartato solo perché riaprire il Tape ricostruisce il testo; nel fallback Ogg lo stesso evento passa perché quel passaggio non avviene. Il testo dal vivo è anche il documento della Sorgente che si corregge: correggere, durante una Registrazione, il Tape consultato che era la Sorgente precedente può sostituire il testo dal vivo con quello del vecchio Tape.

Per chi sviluppa, nessuna di queste regole è testata: i test coprono le trasformazioni del testo e dello stato, non come la finestra le combina. In più il frontend gestisce un protocollo di revisioni per Ingresso pensato per i Parlanti dal vivo, un percorso del backend che dalla revisione del 6 ottobre di ADR-0016 non è più raggiungibile.

## Soluzione

Il frontend riceve gli eventi dell'Attività in corso in un solo modulo puro, con una sessione che ha stati espliciti: nessuna, aperta, in chiusura, chiusa. Il modulo decide quali eventi entrano, possiede Status, testo, timer e avvisi di pulizia, e accetta le modifiche alla Sorgente solo quando nessuna Attività sta scrivendo il testo. La finestra principale registra i listener, inoltra eventi e azioni e mostra lo stato. Il backend perde lo strato dei Parlanti dal vivo e manda, a fine Registrazione, un solo snapshot finale per Ingresso. Per l'utente il comportamento resta quello di oggi, senza i casi sbagliati.

## Storie

1. Come utente, voglio che il testo dal vivo di una Registrazione mostri solo Frasi e Parziali di quella Registrazione, così non vedo testo di un'Attività precedente.
2. Come utente, voglio che i Parziali compaiano solo se ho attivato Trascrivi dal vivo per quella Registrazione, così la vista corrisponde alla scelta fatta all'avvio e non a un'impostazione cambiata dopo.
3. Come utente, voglio che dopo Stop, con il Tape salvato, la vista mostri il Tape riaperto dal disco e ignori eventi arrivati in ritardo, così testo e Tape coincidono.
4. Come utente, voglio che quando il Tape non si scrive e resta solo l'Ogg il testo finale con i Parlanti arrivi comunque nella vista, anche se giunge dopo la risposta di Stop, così non perdo l'analisi finale.
5. Come utente, voglio che un secondo snapshot finale dello stesso Ingresso non sostituisca il primo, così il testo non cambia sotto i miei occhi.
6. Come utente, voglio che aprire un altro Tape o un file chiuda la sessione precedente, così nessun evento tardivo di quella sessione modifica la nuova Sorgente.
7. Come utente, voglio che le Frasi tardive di una Trascrizione di un file non entrino nel Tape riaperto alla fine, così vedo esattamente il Tape salvato.
8. Come utente, voglio che le attribuzioni tardive di Riconosci i parlanti non si applichino a un Tape aperto dopo, così i Parlanti restano quelli salvati.
9. Come utente, voglio che correggere un Turno del Tape che consulto durante una Registrazione non tocchi il testo dal vivo, anche se quel Tape era la Sorgente precedente.
10. Come utente, voglio che rinominare un Parlante o cambiare la data di quel Tape consultato non tocchi il testo dal vivo.
11. Come utente, voglio che correzioni e rinomine sul Tape aperto come Sorgente, senza Attività in corso, si vedano subito, come oggi.
12. Come utente, voglio che la fase in Attività (Preparazione, Registrazione, Pausa, completamento, analisi dei Parlanti) segua gli eventi della Registrazione in corso e nessun altro.
13. Come utente, voglio che il timer della barra laterale mostri il tempo della Registrazione in corso e riparta da zero a ogni nuova Registrazione.
14. Come utente, voglio che la percentuale di una Trascrizione o del completamento dopo Stop venga solo dall'Attività in corso.
15. Come utente, voglio che un guasto della Trascrizione dal vivo mostri l'avviso e tolga i Parziali, mentre la Registrazione continua.
16. Come utente, voglio che un guasto della pulizia audio di un Ingresso resti segnalato per quella Registrazione finché non lo chiudo, e che chiuderlo non tolga l'indicazione di bypass sulla barra.
17. Come utente, voglio che spegnere la pulizia di un Ingresso tolga subito l'indicazione "preparazione della pulizia" di quell'Ingresso.
18. Come utente, voglio che annullare la Preparazione della Registrazione mi riporti a fase, avvisi e Sorgente di prima, senza testo nuovo.
19. Come utente, voglio che una Registrazione non partita per un errore del dispositivo mi riporti allo stato precedente con l'errore in un avviso.
20. Come utente, voglio che premere Registra subito dopo un salvataggio delle impostazioni non perda i primi eventi della Registrazione, perché i listener sono pronti prima che il comando parta.
21. Come utente, voglio che la barra della Registrazione mostri livelli, preparazione e bypass solo dell'Ingresso e della Registrazione in corso.
22. Come utente, voglio che durante Trascrivi su un file la vista resti senza testo fino alla fine, come oggi, e che dopo Annulla si vedano le Frasi arrivate.
23. Come utente, voglio che Copia testo durante e dopo una Registrazione copi lo stesso testo che vedo.
24. Come utente, voglio che il nome predefinito del Microfono compaia nel testo dal vivo dal primo istante della Registrazione, come nel Tape che nascerà.
25. Come utente, voglio che un diarizer assente o incompatibile, con Riconosci i parlanti attivo per la Registrazione, venga segnalato dopo Stop come Diarizzazione non completata, come oggi.
26. Come utente, voglio che i Tape salvati con attribuzioni provvisorie o esiti per Ingresso continuino ad aprirsi e mostrarsi come prima.
27. Come chi sviluppa Memotape, voglio che le regole di accettazione degli eventi stiano in un solo modulo, così un bug di sessione si corregge in un posto.
28. Come chi sviluppa Memotape, voglio testare quelle regole con `bun test` senza Tauri né componenti, così le regressioni si vedono nei sei controlli.
29. Come chi sviluppa Memotape, voglio che lo stato della sessione sia esplicito, così non devo sapere quale setter azzera quale identificatore.
30. Come chi sviluppa Memotape, voglio che la finestra principale non possieda più Status, testo dell'Attività, timer e pulizia, così diventa più corta e più facile da leggere.
31. Come chi sviluppa Memotape, voglio che lo snapshot finale sia l'unica forma di `LiveTranscriptUpdated`, così il frontend non gestisce revisioni che nessuno produce.
32. Come chi sviluppa Memotape, voglio che il codice dei Parlanti dal vivo non raggiungibile sparisca dal core, così `transcribe_live` e l'analisi finale hanno un solo percorso.
33. Come chi sviluppa Memotape, voglio che AGENTS.md descriva il comportamento reale, senza i percorsi dei ticket 05–07 né un evento che non viene emesso.
34. Come agente che implementa, voglio un test che fallisce sul codice attuale per il caso della correzione durante la Registrazione, così so che il bug è chiuso.

## Decisioni di implementazione

- **Due passi, in quest'ordine.** Prima il backend elimina lo strato dei Parlanti dal vivo e riduce l'evento; poi il frontend introduce il modulo. Il secondo dipende dalla forma nuova dell'evento nei bindings.
- **Backend, cosa sparisce:** la sessione dei Parlanti dal vivo nel manager della Trascrizione e il suo ramo in `transcribe_live`; il diarizer dal vivo, il suo testo dal vivo, la coda dei frame verso il diarizer e lo stream realtime di Nemotron; il diarizer facoltativo delle fonti dal vivo; il testo ASR conservato per l'analisi dei Parlanti dal vivo e il suo ramo nell'analisi finale; l'evento `LiveDiarizationFailed`; il banco del ticket 08 e gli smoke nativi dei ticket 05–07.
- **Backend, cosa resta:** l'analisi finale dopo Stop sugli Ogg, la divisione delle Frasi ai cambi di Parlante, la prenotazione del diarizer all'avvio con l'errore nell'esito finale, la lettura dei campi facoltativi del Tape v1 già salvati.
- **Contratto dell'evento:** `LiveTranscriptUpdated` porta solo identificatore di sessione, Ingresso e Frasi. Si emette una volta per Ingresso con Frasi, a fine Registrazione, dopo l'analisi finale. Trascrivi e Riconosci i parlanti non lo usano.
- **Il modulo dell'Attività** (feature `activity`) è un reducer puro: stato e azione in ingresso, stato in uscita. Lo stato contiene Status, sessione, Conversation, tempo trascorso, preparazioni e guasti della pulizia, chiusura dell'avviso di pulizia.
- **La sessione** ha quattro stati: nessuna, aperta, in chiusura, chiusa. Porta l'identificatore (assente per Trascrivi e Riconosci i parlanti) e se accetta Parziali (fissato all'avvio dalla richiesta, non letto dalle impostazioni correnti). Aperta accetta gli eventi con lo stesso identificatore; in chiusura accetta solo lo snapshot finale di quella sessione, una volta per Ingresso; chiusa scarta tutto. Una Trascrizione passa da aperta a chiusa con l'esito; una Registrazione passa a in chiusura con l'esito e a chiusa all'apertura di un'altra Sorgente.
- **Azioni dagli eventi:** fase della Registrazione, preparazione della pulizia, Frase, Parziale, snapshot finale, progresso, inizio dell'analisi dei Parlanti, Parlanti assegnati, guasto della Trascrizione dal vivo, tick, guasto della pulizia.
- **Azioni dai comandi:** avvio (tipo di Attività, identificatore, Parziali, nome del Microfono), esito (lo Status già calcolato dalle funzioni di esito esistenti), ripristino (annullamento della Preparazione o mancata partenza), pausa, Sorgente aperta (Frasi e nomi di un Tape, oppure vuota per un file), Sorgente modificata (una trasformazione pura della Conversation, applicata solo senza sessione aperta o in chiusura), pulizia spenta per un Ingresso.
- **L'hook** registra tutti i listener dell'Attività, inoltra al reducer e offre una promessa "pronto" che copre tutti i listener; l'avvio della Registrazione la aspetta prima del comando, come oggi. La richiesta di un Tape dall'esterno e il drop dei file restano alla finestra principale.
- **La finestra principale** conserva la logica dei comandi: richiesta della Registrazione in corso, ripristino dello stato precedente, conferme, Tape consultato. Invece di impostare lo Status a mano, invia le azioni.
- **Le trasformazioni della Conversation** (Frase, Parziale, Parlanti, nomi, turni) restano pure e perdono il controllo di sessione, che passa al modulo. Lo Status conserva le sue transizioni e la composizione degli avvisi; il modulo le chiama.
- **La barra della Registrazione** riceve preparazioni e guasti già filtrati e conserva il proprio ascolto dei livelli a 40 ms, confrontando direttamente l'identificatore di sessione: i livelli non passano dalla finestra principale.
- **Il glossario non cambia.** "Attività" esiste già; l'identificatore di sessione resta un dettaglio tecnico della Registrazione.

## Decisioni sui test

- Un buon test parte da uno stato, applica una sequenza di azioni come le produrrebbero eventi e comandi reali e controlla lo stato visibile (Status, Frasi, Parziali, timer, avvisi). Non controlla campi interni della sessione oltre a quanto serve a descrivere il comportamento.
- **Unico seam nuovo:** l'interfaccia del reducer dell'Attività. Si testano lì: accettazione e rifiuto per sessione; Parziali solo con Trascrivi dal vivo; snapshot finale accettato in chiusura e una sola volta per Ingresso; eventi tardivi dopo l'esito di una Trascrizione; chiusura all'apertura di un'altra Sorgente; modifica della Sorgente ignorata durante una Registrazione (il primo test, rosso sul codice attuale); ripristino dopo annullamento della Preparazione; guasto dal vivo che toglie i Parziali; pulizia per Ingresso e chiusura dell'avviso; timer azzerato all'avvio.
- **Si sostituisce, non si somma:** i casi di sessione dei test della Conversation passano nei test del modulo; quelli delle revisioni intermedie si eliminano; i test della funzione di confronto di sessione e del filtro della pulizia spariscono con le funzioni. Restano i test delle trasformazioni della Conversation (senza sessione) e dello Status.
- **Backend:** nessun seam nuovo. Restano i test dell'analisi finale e delle fonti dal vivo, aggiornati senza il ramo eliminato; la forma dell'evento si verifica con il test dei bindings committati. Un seam per gli eventi Tauri appartiene al candidato "Trascrizione per Ingresso".
- **Precedenti nel codice:** i test della Conversation e dello Status (`*.test.ts` accanto al codice, `bun test`), i test della Preparazione della Registrazione, il test dei bindings committati.
- **Verifica manuale** con `bun tauri dev`: Registrazione con Trascrivi dal vivo e Stop con Tape salvato; correzione del Tape precedente consultato durante la Registrazione; Trascrivi su un file; Riconosci i parlanti su un Tape; annullamento della Preparazione.

## Fuori perimetro

- Il ciclo di vita dei comandi dell'Attività (richiesta della Registrazione, ripristino, conferme, Tape consultato e vista della Sorgente): è il candidato 6 della revisione.
- Un identificatore per Trascrivi e Riconosci i parlanti, e qualunque cambio ai loro comandi.
- Lo spostamento dell'orchestrazione Rust in un modulo testabile e un seam per gli eventi Tauri (candidato 3), la Diarizzazione finale unificata (candidato 2), la protezione del parlato nella fonte di frame (candidato 4).
- Ridurre la frequenza con cui il timer aggiorna la finestra principale.
- Nuove funzioni per l'utente: il comportamento visibile resta quello di oggi, tolti i casi sbagliati.

## Note

- Il percorso realtime eliminato resta nella storia git e nei report dei ticket 05–08 di `.scratch/nemotron3-diarizzazione/`.
- Il caso della correzione durante la Registrazione è dedotto dal codice e non riprodotto: il primo test lo conferma o lo smentisce. Se lo smentisce, la regola "modifiche solo senza sessione attiva" resta comunque.
