Status: ready-for-agent

# Sbobino: spec v3 (Libreria, Raccolte, ricerca, player sincronizzato)

## Problem Statement

Chi usa Sbobino per lavoro fa molte call con clienti diversi, in giorni diversi. Oggi ogni Registrazione è un Bino sciolto nella Cartella predefinita con un nome fatto di data e ora, e l'app mostra solo la Sorgente aperta: per ritrovare "la call con Ferrara Quarzi in cui si è parlato del preventivo" bisogna aprire i Bini uno alla volta. Non c'è modo di cercare una parola in tutte le trascrizioni, né di tenere separate le call di un cliente da quelle di un altro.

Una call registrata con un altro strumento (l'MP4 di Teams) lascia solo un Markdown accanto al file: i tempi delle Frasi si perdono e la trascrizione non si riapre.

Il testo e l'audio non si parlano. Per verificare cosa ha detto davvero qualcuno bisogna aprire l'audio in un altro programma e cercare il punto a mano. Il testo si corregge in un'area libera, ma la correzione non arriva al Bino e si perde riaprendolo.

## Solution

- **Libreria e Raccolte.** La Libreria è la cartella in cui Sbobino salva i Bini, quella che finora era la Cartella predefinita. Le sue cartelle di primo livello sono le Raccolte, per esempio una per cliente. Un Bino fa parte della Libreria se sta lì dentro: quello che si vede nell'app è quello che si vede in Esplora file (ADR-0008).
- **Barra laterale.** È sempre visibile, con Registra, Apri file, l'Attività in corso, il selettore della Raccolta, la ricerca e i Bini della Raccolta raggruppati per data.
- **Ricerca nel testo.** Un indice SQLite FTS5, locale e ricostruibile, trova le parole nei titoli, nelle Frasi e nei nomi dei Parlanti. Il risultato apre il Bino sulla Frase trovata.
- **Ogni file trascritto diventa un Bino** nella Raccolta, con l'audio in Opus e il testo con i tempi (ADR-0009). Il Markdown non si salva più da solo: si esporta.
- **Vista di un Bino.** Ha il titolo rinominabile, le informazioni, la trascrizione a turni con la correzione per Frase salvata nel Bino, e un player fisso che riproduce il mix.
- **Testo e audio collegati.** Durante l'ascolto la Frase in riproduzione si evidenzia e resta visibile, finché l'utente non scorre per conto suo. Il pulsante del tempo di una Frase porta il player lì.

## User Stories

### Libreria e Raccolte

1. Come utente, voglio che i miei Bini stiano in una Libreria che l'app mi mostra sempre, così ritrovo le riunioni passate senza cercarle in Esplora file.
2. Come utente, voglio che la Libreria sia la cartella che prima era la Cartella predefinita (`Documenti\Sbobino` se non ne ho scelta un'altra), così i Bini che ho già fatto ci sono subito.
3. Come utente, voglio scegliere in Impostazioni → Generale la "Cartella della Libreria", così tengo la Libreria dove voglio, per esempio in una cartella sincronizzata.
4. Come utente, voglio che cambiare la Cartella della Libreria mostri i Bini della nuova cartella senza spostare quelli della vecchia, così nessun file si muove senza che lo chieda.
5. Come professionista, voglio raggruppare i Bini in Raccolte, per esempio una per cliente, così non mescolo le call di clienti diversi.
6. Come utente, voglio che una Raccolta sia una cartella dentro la Libreria, così ritrovo lo stesso ordine in Esplora file e nei miei backup.
7. Come utente, voglio creare una Raccolta dalla barra laterale, così inizio a ordinare senza uscire dall'app.
8. Come utente, voglio rinominare una Raccolta e che la sua cartella si rinomini con lei, così app e disco restano allineati.
9. Come utente, voglio che il nome di una Raccolta rifiuti i caratteri non ammessi da Windows e il nome di un'altra Raccolta, così non ottengo errori dal disco.
10. Come utente, voglio eliminare una Raccolta vuota, così tolgo quelle che non servono più.
11. Come utente, voglio che una Raccolta con dei Bini non si possa eliminare finché non la svuoto, così nessuna azione cancella riunioni in massa.
12. Come utente, voglio che i Bini nella cartella principale della Libreria compaiano come "Senza raccolta", così non devo per forza ordinare tutto.
13. Come utente, voglio che le Raccolte abbiano un solo livello, così l'ordine resta semplice; i Bini in sottocartelle più profonde create a mano contano nella Raccolta che le contiene.
14. Come utente, voglio spostare un Bino in un'altra Raccolta con "Sposta in…", così riordino le riunioni registrate nel posto sbagliato.
15. Come utente, voglio che un Bino che sposto, rinomino o cancello in Esplora file venga recepito dall'app, così non ci sono voci fantasma.
16. Come utente, voglio eliminare un Bino dall'app mandandolo nel Cestino di Windows dopo una conferma, così libero spazio senza perdere nulla per sbaglio.
17. Come utente, voglio aprire un Bino che sta fuori dalla Libreria (Download, ricevuto da un collega) e consultarlo con testo e player, senza che entri nella Libreria, così guardo un file senza archiviarlo.
18. Come utente, voglio "Aggiungi alla Libreria…" su un Bino aperto da fuori, che mi chiede la Raccolta e lo sposta lì, così archivio un Bino ricevuto.
19. Come utente, voglio che il titolo di un Bino sia il nome del suo file, così app ed Esplora file dicono la stessa cosa.
20. Come utente, voglio rinominare un Bino cliccando il titolo nella sua vista, così "Registrazione 2026-10-04 10-15-00" diventa "Ferrara Quarzi, preventivo fase 2".
21. Come utente, voglio che un titolo già usato nella stessa Raccolta, vuoto o con caratteri non ammessi non si confermi, così non sovrascrivo né rompo un file.
22. Come utente, voglio "Mostra in Esplora file" nella vista di un Bino, così lo trovo sul disco per mandarlo a qualcuno.

### Barra laterale

23. Come utente, voglio una barra laterale sempre visibile, così passo da una riunione all'altra senza perdere il punto.
24. Come utente, voglio in cima alla barra laterale Registra e Apri file (audio, video o Bino), così avvio il lavoro da un posto fisso.
25. Come utente, voglio una voce "Attività in corso" quando c'è una Registrazione o una Trascrizione, con lo stato in breve (timer o percentuale), così vedo a che punto è anche mentre leggo altro.
26. Come utente, voglio che un clic su "Attività in corso" mi riporti alla sua vista, così torno alla Registrazione dopo aver consultato una riunione vecchia.
27. Come utente, voglio un selettore della Raccolta con Tutta la Libreria, Senza raccolta, le mie Raccolte e "Nuova Raccolta", così scelgo su cosa lavoro.
28. Come utente, voglio che la barra laterale mostri i Bini della Raccolta scelta, dal più recente, raggruppati in Oggi, Ieri, Questa settimana e poi per mese, così trovo a colpo d'occhio la call di martedì.
29. Come utente, voglio che ogni Bino nella barra laterale mostri titolo, ora e durata, così lo riconosco senza aprirlo.
30. Come utente, voglio che la Raccolta scelta resti scelta al prossimo avvio, così riprendo da dove avevo lasciato.
31. Come utente, voglio in fondo alla barra laterale "Tutti i Bini della Raccolta", che apre l'elenco completo nell'area principale con ordinamento per data o titolo, così consulto anche le riunioni vecchie.
32. Come utente, voglio dall'elenco completo spostare ed eliminare i Bini, così riordino una Raccolta in un colpo d'occhio.
33. Come utente, voglio Impostazioni in fondo alla barra laterale, così le trovo sempre allo stesso posto.
34. Come utente, voglio che il Bino aperto sia evidenziato nella barra laterale, così so dove mi trovo.

### Ricerca

35. Come utente, voglio un campo di ricerca nella barra laterale, così ritrovo una riunione da una parola che ricordo.
36. Come utente, voglio che la ricerca guardi nei titoli, nel testo delle Frasi e nei nomi dei Parlanti, così trovo una riunione anche se ricordo solo chi c'era o cosa si è detto.
37. Come utente, voglio che la ricerca ignori maiuscole e accenti e trovi anche l'inizio di una parola ("prev" trova "preventivo"), così non devo scrivere la parola esatta.
38. Come utente, voglio che la ricerca parta dalla Raccolta scelta, così le call di un cliente non si mescolano con quelle degli altri.
39. Come utente, voglio in fondo ai risultati "Cerca in tutta la Libreria", così allargo la ricerca quando non ricordo con chi ne ho parlato.
40. Come utente, voglio che ogni risultato mostri il Bino e le Frasi trovate con un estratto e la parola evidenziata, così capisco quale riunione è quella giusta.
41. Come utente, voglio che il clic su una Frase trovata apra il Bino, scorra fino a lei, la evidenzi e porti lì il player senza farlo partire, così premo Play e sento proprio quel passaggio.
42. Come utente, voglio che i risultati siano ordinati per pertinenza, così il Bino che parla di più dell'argomento viene prima.
43. Come utente, voglio che una correzione del testo o la rinomina di un Parlante si possano cercare subito, così la ricerca trova quello che vedo.
44. Come utente, voglio che audio, trascrizioni e indice restino sul PC, così le riunioni dei clienti non escono dalla mia macchina.
45. Come utente, voglio che se l'indice si perde o si rovina l'app lo ricostruisca dai Bini, così non perdo nulla e non devo fare niente.

### Trascrizione di un file

46. Come utente, voglio che Trascrivi su un file audio o video crei un Bino nella Raccolta scelta, così anche la call registrata con Teams entra in "Ferrara Quarzi" come le altre.
47. Come utente, voglio che il Bino di un file contenga l'audio, così riascolto e cerco anche se sposto o cancello il file originale.
48. Come utente, voglio che il file originale non venga modificato né spostato, così resta dov'era per gli altri programmi.
49. Come utente, voglio che il Bino prenda il nome del file originale (con " 2", " 3"… se esiste già), così lo riconosco.
50. Come utente, voglio che il nome del file d'origine resti nelle informazioni del Bino, così so da dove viene anche dopo averlo rinominato.
51. Come utente, voglio che finita la Trascrizione la Sorgente diventi il Bino, così vedo subito testo e player.
52. Come utente, voglio che una Trascrizione annullata o senza parlato non crei nessun Bino, così la Libreria non si riempie di file vuoti o a metà.
53. Come utente, voglio che l'audio del Bino usi il bitrate, i canali e la frequenza delle impostazioni di Registrazione, così controllo qualità e spazio occupato.
54. Come utente, voglio che un file audio o video aperto e non ancora trascritto mostri il suo nome e Trascrivi, così so cosa fare.

### Opzioni e Markdown

55. Come utente, voglio scegliere modello, Lingua del parlato e Riconosci i parlanti da un menu sul pulsante Trascrivi, così le opzioni non occupano lo spazio della lettura.
56. Come utente, voglio che il pulsante Trascrivi dica in breve la scelta corrente ("Trascrivi · Italiano"), così so cosa succederà senza aprire il menu.
57. Come utente, voglio scegliere Trascrivi dal vivo e Riconosci i parlanti della Registrazione da un menu sul pulsante Registra, così li cambio prima di registrare senza aprire Impostazioni.
58. Come utente, voglio "Esporta Markdown…" che salva il Markdown dove scelgo con il dialog di sistema, così ho il documento quando mi serve, senza file doppi nelle Raccolte.
59. Come utente, voglio che Copia testo copi il testo del Bino aperto, con le correzioni e i nomi dei Parlanti, nel formato scelto in Impostazioni, così lo incollo altrove.

### Vista di un Bino

60. Come utente, voglio che selezionare un Bino ne mostri titolo, informazioni e trascrizione nell'area principale, così lo leggo subito.
61. Come utente, voglio una riga di informazioni con data e ora, durata, Raccolta, modello, Lingua del parlato, "Ingressi separati" se vale e "incompleto" se la Trascrizione non è finita, così so cosa sto guardando.
62. Come utente, voglio la trascrizione a turni, con l'etichetta del Parlante o dell'Ingresso all'inizio di ogni turno, così leggo chi ha detto cosa.
63. Come utente, voglio le Frasi di un turno una dopo l'altra in un testo leggibile, non come righe di un editor, così la lettura è scorrevole.
64. Come utente, voglio rinominare i Parlanti come oggi (dall'elenco o dall'etichetta del turno), così il testo dice "Mario" invece di "Parlante 2".
65. Come utente, voglio che la rinomina di un Parlante si salvi nel Bino, così la ritrovo riaprendolo.
66. Come utente, voglio che le azioni Trascrivi ▾, Copia testo, Esporta Markdown… e Mostra in Esplora file stiano sopra il testo, e Sposta in… ed Elimina in un menu "…", così le azioni rare non ingombrano.

### Correzione del testo

67. Come utente, voglio correggere una parola cliccando sul testo di una Frase, così sistemo gli errori del modello sul posto.
68. Come utente, voglio che la correzione si salvi nel Bino quando esco dalla Frase, così non la perdo chiudendo l'app.
69. Come utente, voglio che Esc annulli la correzione in corso e ripristini il testo della Frase, così torno indietro da una modifica sbagliata.
70. Come utente, voglio che correggere il testo non sposti la Frase nell'audio, così il collegamento con il player resta giusto.
71. Come utente, voglio che una Frase svuotata resti al suo posto, così il suo tempo non si perde e posso riscriverla.
72. Come utente, voglio che una correzione non riuscita (Bino in sola lettura, disco pieno) mi lasci il testo modificato con l'errore nella status bar, così posso riprovare.
73. Come utente, voglio che ritrascrivere un Bino mi avvisi che le correzioni si perdono, così decido consapevolmente.
74. Come utente, voglio che il testo di una Registrazione o di una Trascrizione in corso non si possa correggere finché non è finita, così non litigo con le Frasi che arrivano.

### Player

75. Come utente, voglio un player in fondo alla vista di un Bino, sempre visibile mentre scorro il testo, così riascolto senza perdere il punto.
76. Come utente, voglio che il player riproduca il mix, anche quando il Bino conserva gli Ingressi separati, così sento la conversazione completa.
77. Come utente, voglio Play/Pausa, la posizione corrente, la durata complessiva e una barra per spostarmi, così mi muovo nella riunione.
78. Come utente, voglio i pulsanti indietro e avanti di 10 secondi, così riascolto una frase sentita male.
79. Come utente, voglio la velocità 1×, 1,25×, 1,5× e 2×, così ripasso una riunione lunga in meno tempo.
80. Come utente, voglio che Spazio faccia Play/Pausa quando non sto scrivendo, così controllo l'ascolto dalla tastiera.
81. Come utente, voglio che il player sia disabilitato durante una Registrazione, così l'audio riascoltato non finisce nel file.
82. Come utente, voglio che aprire un altro Bino fermi l'ascolto del precedente, così non sento due riunioni.
83. Come utente, voglio che un file audio o video non ancora trascritto non abbia player, così il player compare solo dove testo e audio sono collegati.

### Testo e audio collegati

84. Come utente, voglio che durante l'ascolto la Frase in riproduzione sia evidenziata, così seguo il testo mentre ascolto.
85. Come utente, voglio che la Frase evidenziata resti visibile, scorrendo il testo da solo, così non devo inseguirla.
86. Come utente, voglio che spostando la barra del player l'evidenziazione salti alla Frase di quel punto, così vedo subito dove sono arrivato.
87. Come utente, voglio che nel silenzio tra due Frasi resti evidenziata la precedente, così il testo non sfarfalla.
88. Come utente con gli Ingressi separati, voglio che due Frasi che si sovrappongono nel tempo siano evidenziate entrambe e che lo scorrimento segua quella iniziata prima, così vedo chi parlava sopra chi.
89. Come utente, voglio poter scorrere il testo per leggere un altro passaggio mentre l'audio va avanti, senza che l'app mi riporti indietro, così leggo e ascolto cose diverse.
90. Come utente, voglio un comando "Segui l'audio" che compare quando ho scorso da solo e mi riporta alla Frase in riproduzione, così riprendo a seguire quando voglio.
91. Come utente, voglio che spostare la barra del player o saltare a una Frase riprenda a seguire l'audio, così non devo premere anche "Segui l'audio".
92. Come utente, voglio che mentre correggo una Frase il testo non scorra da solo, così la Frase non mi scappa da sotto il cursore.
93. Come utente, voglio accanto a ogni Frase un pulsante con il suo tempo (`12:34`), visibile passando il mouse, sempre per la Frase in riproduzione e all'inizio di ogni turno, così vedo dove sono nella riunione.
94. Come utente, voglio che il pulsante del tempo porti il player all'inizio della Frase, così riascolto proprio quel passaggio.
95. Come utente, voglio che con l'audio in pausa il salto lasci il player in pausa, e con l'audio in riproduzione continui da lì, così decido io quando ascoltare.
96. Come utente, voglio che un clic sul testo serva solo a correggerlo e non sposti mai il player, così una correzione non mi fa perdere il punto.
97. Come utente da tastiera, voglio raggiungere il pulsante del tempo con Tab e attivarlo con Invio, così uso il collegamento senza mouse.
98. Come utente, voglio che i tempi siano quelli dell'audio salvato, con le pause della Registrazione escluse, così il salto arriva sempre al punto giusto.
99. Come utente, voglio che i tempi non compaiano nel testo copiato o esportato, così il testo resta pulito.

### Attività e navigazione

100. Come utente, voglio aprire, leggere, cercare e copiare altri Bini mentre registro o trascrivo, così consulto una riunione vecchia durante una call.
101. Come utente, voglio correggere il testo e rinominare i Parlanti di altri Bini durante un'Attività, ma non del Bino su cui l'Attività sta lavorando, così non rompo il lavoro in corso.
102. Come utente, voglio che durante una Registrazione la sua vista mostri timer, livelli, Pausa, Stop e il testo dal vivo al posto del player, così controllo la Registrazione dove la vedo.
103. Come utente, voglio che il Bino prodotto da una Registrazione finisca nella Raccolta scelta al momento di Registra, o in "Senza raccolta" con Tutta la Libreria, così la call con un cliente finisce già nel suo posto.
104. Come utente, voglio che a Stop il nuovo Bino compaia nella barra laterale e resti aperto, così lo ritrovo subito.
105. Come utente, voglio che la status bar continui a dire fase, percentuale, esiti ed errori, così so sempre cosa succede.
106. Come utente, voglio che il doppio clic su un Bino in Esplora file lo apra nell'app come oggi, anche se non è nella Libreria, così il comportamento di Windows non cambia.

### Sviluppo

107. Come sviluppatore, voglio che la Libreria sia un modulo Rust senza Tauri, testato su cartelle temporanee, così la sincronizzazione con il disco è verificata senza l'app.
108. Come sviluppatore, voglio che il database sia una cache ricostruibile e mai la fonte di verità, così un suo errore non può far perdere dati.

## Implementation Decisions

Le decisioni di fondo sono negli ADR-0008 (Libreria come cartella con un indice) e ADR-0009 (ogni Trascrizione di un file diventa un Bino). Le scelte di dettaglio vengono dal grilling, in `decisioni.md` accanto a questa spec.

### Libreria (modulo nuovo, senza Tauri)

- È un modulo come `bino` e `transcript`, senza `AppHandle`. La sua interfaccia: si apre su una cartella della Libreria e un percorso del database. Poi:
  - allinea l'indice alla cartella;
  - elenca le Raccolte e i Bini di una Raccolta (o di tutta la Libreria, o dei Senza raccolta);
  - cerca, in una Raccolta o in tutta la Libreria;
  - crea, rinomina ed elimina una Raccolta vuota;
  - sposta, rinomina e manda nel Cestino un Bino;
  - aggiorna l'indice di un Bino appena riscritto.
- Il manager in `tauri::State` tiene un'istanza per la Cartella della Libreria corrente, la ricrea quando l'impostazione cambia ed emette `library-changed` dopo ogni allineamento o operazione che cambia l'elenco. Il frontend rilegge l'elenco a ogni `library-changed`.
- **Raccolte**: sono le cartelle di primo livello della Libreria, escluse quelle che iniziano con `.` (quindi anche `.sbobino`). Un Bino in una cartella più profonda appartiene alla Raccolta di primo livello che la contiene. "Senza raccolta" sono i Bini nella radice.
- I nomi di Raccolte e Bini rifiutano i caratteri non ammessi da Windows, i nomi riservati (`CON`, `NUL`…), il punto o lo spazio finale e il nome vuoto. Un nome già esistente dà un errore dedicato (`nameTaken`). Rinominare non cambia mai l'estensione `.bino`.
- **Cestino**: `SHFileOperationW` con `FOF_ALLOWUNDO`, tramite `windows-sys` che è già una dipendenza (aggiungere la feature della Shell). Niente cancellazioni definitive in nessun percorso dell'app.
- **Allineamento**: all'avvio, quando la finestra torna in primo piano e prima di aprire un Bino, la Libreria elenca i `.bino` della cartella (ricorsivo, saltando le cartelle che iniziano con `.`). Rilegge solo quelli nuovi o con data di modifica o dimensione cambiate, e toglie dall'indice quelli spariti. Un Bino illeggibile o di una versione futura si elenca con il nome del file e senza testo nell'indice, e aprirlo dà l'errore di oggi. Niente watcher del file system: spostamenti fatti in Esplora file con l'app in primo piano si vedono alla prossima occasione di allineamento, e `library-changed` aggiorna la barra laterale.
- **Database**: SQLite (`rusqlite` 0.40 con `bundled`) in `app_local_data_dir` (`%LOCALAPPDATA%\it.sbobino.desktop`). Un file per Cartella della Libreria, con il nome ricavato dal percorso, così cambiare cartella e tornare indietro non costa una ricostruzione. Due tabelle:
  - una tabella dei Bini: percorso relativo alla Libreria, Raccolta, titolo, `creato`, durata, data di modifica e dimensione del file, `completa`, `modalita`, modello, Lingua del parlato;
  - una tabella virtuale FTS5 con una riga per Frase: Bino, id della Frase, Ingresso, `inizio_ms`, testo e il nome mostrato del Parlante. Ha in più una riga per il titolo del Bino.
  - Tokenizer `unicode61 remove_diacritics 2`. La query dell'utente si spezza in parole, ognuna diventa un prefisso (`parola*`), unite in AND. Ordinamento con `bm25`, estratti con `snippet()`. I caratteri speciali di FTS5 nella query si neutralizzano.
- Il database ha una `user_version`. Se il numero non torna, il file è corrotto o non si apre, la Libreria lo cancella e ricostruisce l'indice dai Bini. Nessun dato esiste solo nel database.
- **Risultati della ricerca**: per Bino, ordinati per il miglior `bm25` delle sue righe, ognuno con le Frasi trovate (id, Ingresso, `inizio_ms`, estratto con la parola segnata). Un limite di Frasi per Bino e di Bini per pagina tiene la risposta piccola.

### Bino

- Lo schema resta alla `version` 1 con un campo in più, facoltativo: `origine`, il nome del file da cui viene un Bino trascritto da un file. Le versioni precedenti dell'app lo ignorano, perché la lettura ignora i campi sconosciuti.
- **Correzione di una Frase**: una funzione che riscrive il testo di una Frase, identificata da Ingresso e id come nei `TranscriptPhrase`, con `bino::rewrite`. Tempi, Parlante, nomi dei Parlanti e audio restano come sono. Dopo la riscrittura si aggiorna l'indice di quel Bino.
- **Finestra sul mix**: una funzione pura che, data una richiesta `Range` (o nessuna) e la finestra di `mix.ogg` nello zip (offset e lunghezza, da `bino::Mix`), dà stato (200/206/416), intestazioni (`Content-Range`, `Content-Length`, `Accept-Ranges`, `Content-Type: audio/ogg`) e i byte da leggere. Si usa dal protocollo personalizzato.

### Player e protocollo

- Il frontend riproduce con un elemento `<audio>` (posizione da `currentTime`, eventi `timeupdate`, `seeked`, `play` e `pause`). La sorgente è un protocollo personalizzato registrato con `register_asynchronous_uri_scheme_protocol` (per esempio `bino`), che serve il `mix.ogg` del Bino indicato nell'URL attraverso la finestra sul mix.
- Il protocollo accetta solo percorsi `.bino`, cioè il Bino aperto o uno nella Libreria, e legge solo la voce `mix.ogg`. Non è un modo per leggere file qualunque.
- Il salto alla Frase imposta `currentTime = inizio_ms / 1000` e non tocca lo stato di Play/Pausa.
- Il player è disabilitato mentre l'Attività in corso è una Registrazione, e si ferma quando si apre un altro Bino.
- Spazio fa Play/Pausa solo se il focus non è su un campo di testo o una Frase in modifica.

### Trascrizione di un file

- `transcribe` riceve anche la Raccolta di destinazione. Su un file audio o video, mentre `transcribe_file` decodifica, l'audio originale (alla sua frequenza e con i suoi canali) passa per il `Mixer` o il `Resampler` ed entra in un `OggOpusWriter` alle impostazioni di Registrazione. Si scrive un Ogg temporaneo nella cartella `.sbobino` della Libreria.
- A Trascrizione finita (Diarizzazione compresa) si compone il Bino `<nome del file>.bino` nella Raccolta, con " 2", " 3"… come `recording_path`, con `creato` = l'ora di inizio della Trascrizione e `origine`. Poi si cancella l'Ogg temporaneo.
- Annullata, guasta o senza parlato: l'Ogg temporaneo si cancella e il Bino non si crea.
- Il risultato `TranscriptionOutcome` porta il percorso del Bino invece del Markdown, e il frontend lo rende la Sorgente.
- Su un Bino Trascrivi resta com'è (riscrive il documento), salvo il Markdown, che non si salva più. La conferma dice anche che le correzioni del testo si perdono.

### Registrazione

- `record` riceve la Raccolta di destinazione. Il Bino si scrive lì e l'Ogg temporaneo resta in `.sbobino` nella radice della Libreria. Con "Tutta la Libreria" o "Senza raccolta" il Bino va nella radice.
- Dopo Stop la Trascrizione dal vivo non salva più il Markdown: il testo è nel Bino. Se il Bino non si scrive, l'Ogg passa in Libreria come oggi (`keep_ogg`) e, solo in quel caso, accanto si salva il Markdown, perché è l'unico posto in cui resta il testo.

### Markdown e Copia testo

- `transcript::render` resta com'è. "Esporta Markdown…" apre il dialog di salvataggio (`tauri-plugin-dialog`, nome proposto `<titolo>.md`) e scrive il render del Bino aperto, con correzioni e nomi dei Parlanti.
- `rename_parlante` riscrive il Bino e non riscrive più nessun Markdown. `LastTranscript` perde `md` e il calcolo del Markdown più recente (`latest_md`).
- Copia testo rende il Bino aperto, oppure la Trascrizione in corso o appena annullata, come oggi. Sparisce il caso del testo "modificato a mano" (`edited`): Copia testo usa sempre le Frasi.

### Impostazioni

- `recordings_folder` resta il nome del campo in `settings.json`, per i file esistenti, ma nell'interfaccia e nel codice nuovo è la Cartella della Libreria (etichetta "Cartella della Libreria" in Generale).
- Campo nuovo `raccolta`, con `#[serde(default)]`, quindi facoltativo nei bindings: la scelta del selettore (`null` = Tutta la Libreria, `""` = Senza raccolta, altrimenti il nome). Una Raccolta che non esiste più vale come Tutta la Libreria.
- Le scelte di Trascrivi ▾ e Registra ▾ (modello, Lingua del parlato, Riconosci i parlanti, Trascrivi dal vivo e Riconosci i parlanti della Registrazione) sono i campi che esistono già, salvati con `set_settings`.

### Frontend

- La finestra principale diventa un layout con la barra laterale e un'area principale. Le route sono: la vista di un Bino (o di un file aperto), l'elenco completo di una Raccolta e Impostazioni. Impostazioni resta un livello sopra la finestra, che resta montata: un'Attività in corso non si perde.
- L'area di testo (`textarea`) sparisce. La trascrizione è un elenco di turni, ognuno un elenco di Frasi. Il testo di una Frase si modifica sul posto (un `contentEditable` limitato al testo semplice, o un campo che compare al clic). Invio o l'uscita dalla Frase confermano, Esc annulla.
- La `Conversation` (`features/transcription/phrases.ts`) resta il modello delle Frasi. Si aggiunge `withTesto`, che applica una correzione. `conversationText` resta per Copia testo durante una Trascrizione.
- Moduli puri nuovi, con un `*.test.ts` accanto:
  - **sincronizzazione** (`features/player`): le Frasi da evidenziare a una data posizione (dentro una Frase; nel silenzio la precedente; con le Frasi sovrapposte tutte, con quella da seguire = la prima per inizio) e la macchina a stati di "Segui l'audio" (`following` / `free`; scorrimento manuale → `free`; salto o barra del player → `following`; modifica in corso → scorrimento sospeso);
  - **Libreria** (`features/library`): il raggruppamento per data della barra laterale (Oggi, Ieri, Questa settimana, poi per mese, relativo a una data "oggi" passata come argomento), il formato di ora, durata e posizione (`m:ss`, `h:mm:ss`) e la validazione dei nomi, specchio di quella Rust per un errore immediato.
- Lo scorrimento manuale si riconosce dagli eventi di input dell'utente (`wheel`, tasti di scorrimento, trascinamento della barra), non dallo `scroll`, che scatta anche con lo scorrimento automatico.
- Etichette nuove nelle sei lingue (Libreria, Raccolta, Senza raccolta, Tutta la Libreria, Segui l'audio, Esporta Markdown…, Aggiungi alla Libreria…, Sposta in…, Elimina, gruppi di date…), con le forme plurali dove servono.

### Comandi ed eventi (bindings)

Nomi indicativi:
- `library_list`, `library_search`;
- `create_raccolta`, `rename_raccolta`, `delete_raccolta`;
- `move_bino`, `rename_bino`, `trash_bino`, `add_to_library`;
- `edit_frase`, `export_markdown`;
- l'evento `library-changed`.

Ogni comando che scrive un Bino prende l'Attività per il tempo della scrittura, come `rename_parlante`, a meno che non ci sia già un'Attività in corso su un altro Bino. Il backend rifiuta con `activityInProgress` solo le scritture sul Bino dell'Attività in corso. Gli errori nuovi entrano in `AppError` con il loro codice e il messaggio tradotto: `nameTaken`, `invalidName`, `raccoltaNotEmpty`, `binoNotFound`.

## Testing Decisions

- **Cosa è un buon test**: verifica il comportamento osservabile dalla seam pubblica (file sul disco, risultati della ricerca, documento riletto, Frasi da evidenziare), non i dettagli interni come lo schema delle tabelle o le query SQL.
- **Seam 1, core Rust senza Tauri**, su cartelle temporanee con Bini veri (scritti con `bino::write`):
  - *Libreria*:
    - Raccolte e Senza raccolta da una cartella con sottocartelle, comprese quella profonda e `.sbobino`;
    - un Bino aggiunto, spostato, rinominato o cancellato "da Esplora file" (operazioni dirette sul file system) si vede dopo l'allineamento;
    - un Bino riscritto da fuori si rilegge solo se cambiano data o dimensione;
    - un database cancellato o corrotto si ricostruisce con gli stessi risultati;
    - creare, rinominare ed eliminare una Raccolta (piena: errore);
    - spostare e rinominare un Bino, con i nomi già usati o non validi;
    - un Bino illeggibile resta nell'elenco senza bloccare l'allineamento.
  - *Ricerca*: maiuscole e accenti ignorati, prefisso, più parole in AND, titolo, nome rinominato di un Parlante, ambito Raccolta contro tutta la Libreria, estratto con la parola segnata, una correzione trovata subito dopo l'aggiornamento, caratteri speciali di FTS5 nella query senza errori.
  - *Cestino*: un test che manda nel Cestino un file in una cartella temporanea e controlla che non sia più lì. Il contenuto del Cestino non si verifica.
  - *Bino*: la correzione di una Frase cambia solo quel testo, mentre tempi, Parlanti, `parlanti`, `origine` e i byte dell'audio restano uguali; `origine` si scrive e si rilegge; un Bino senza `origine` si legge come prima.
  - *Finestra sul mix*: richieste senza `Range`, `bytes=a-b`, `bytes=a-`, `bytes=-n`, oltre la fine (416); i byte restituiti coincidono con quelli di `mix.ogg` estratto.
  - *Trascrizione di un file*: con il motore finto, `parlato-it.mp4` e `parlato-it.wav` danno un Bino nella cartella indicata. Il `mix.ogg` si decodifica con la durata dell'originale, entro una tolleranza da misurare, alla frequenza e con i canali chiesti, e le Frasi hanno i tempi. Annullando non restano né Bino né Ogg temporaneo; senza parlato nessun Bino.
  - *Registrazione*: il Bino finisce nella Raccolta passata e, senza Raccolta, nella radice (test di `save_bino` / `recording_path` con la cartella).
- **Seam 2, logica pura del frontend** con `bun test`:
  - Frasi da evidenziare: posizione dentro, sul confine (`fine_ms` esclusa), nel silenzio, prima della prima Frase, dopo l'ultima, con Frasi sovrapposte di due Ingressi;
  - "Segui l'audio": la sequenza di scorrimento manuale, barra del player, salto e modifica;
  - raggruppamento per data a cavallo di mezzanotte, dell'inizio settimana e del cambio di mese;
  - formato dei tempi sotto e sopra l'ora;
  - validazione dei nomi;
  - `withTesto` sulla `Conversation`.
- **Verifica a mano** nell'app, via CDP come in AGENTS.md: riproduzione e spostamento nell'audio di un Bino vero (mix a 16 e 48 kHz, mono e stereo), salto da una Frase in pausa e in riproduzione, scorrimento automatico e "Segui l'audio", ricerca su una Libreria di prova con qualche decina di Bini, Cestino, Registrazione in una Raccolta, Trascrizione di un MP4 in una Raccolta.
- **Prior art**: i test di `bino` (scrittura, lettura, riscrittura, `temp_dir`, Ogg sintetici, `decoded_seconds`), di `managers::models` (cartelle temporanee, server sulla loopback), della pipeline con il motore finto e le fixture TTS, del `transcript`; nel frontend i test di `phrases.ts`, `settings.ts` e `status.ts`.

## Out of Scope

- Più Librerie separate tra cui passare (vault). Oggi si cambia la Cartella della Libreria in Impostazioni.
- Raccolte annidate, etichette, Bini in più Raccolte.
- Unire o dividere Frasi, aggiungere testo fuori da una Frase, modificare i tempi di una Frase.
- Annulla/Ripeti delle correzioni oltre l'Esc durante la modifica.
- Il player per i file audio o video non trascritti e per le vecchie Registrazioni `.ogg`.
- Riprodurre il solo microfono o il solo audio di sistema degli Ingressi separati.
- Volume nel player, forma d'onda, segnalibri, note sulle Frasi.
- Ricerca per parte di parola (trigram), ricerca con operatori, filtri per data o Parlante.
- Il watcher del file system: l'allineamento avviene nei momenti elencati sopra.
- Sincronizzazione tra PC e condivisione: la Libreria è una cartella, l'utente la gestisce con i suoi strumenti.
- Il Markdown salvato in automatico accanto al Bino.
- Rimangono fuori dal perimetro le voci di `PRODUCT.md` non toccate qui. "Storico delle trascrizioni" e i tempi nell'interfaccia escono dal "Fuori dal perimetro"; i tempi nel testo copiato o esportato restano fuori.

## Further Notes

- **PRODUCT.md** si aggiorna con le storie di questa spec quando i ticket le chiudono:
  - cambiano le storie 16 e 17 (niente Markdown automatico, il risultato è il Bino), 18 (Copia testo sempre dalle Frasi), 19 (la conferma vale per la Sorgente, non per l'area), 21, 47 e 58 (Cartella predefinita → Cartella della Libreria), 64 (la finestra ha la barra laterale) e il punto di V6 sulla riscrittura del Markdown alla rinomina;
  - dal "Fuori dal perimetro" esce "Storico delle trascrizioni";
  - i principi 3 e 4 restano: la status bar c'è ancora.
- **CONTEXT.md** ha già Libreria, Raccolta e le definizioni aggiornate di Bino, Sorgente e Frase. Nell'interfaccia "Cartella predefinita" diventa "Cartella della Libreria" nelle sei lingue.
- **Migrazione**: nessuna. I Bini esistenti nella Cartella predefinita sono già nella Libreria, come "Senza raccolta"; i Markdown accanto restano dove sono e l'app non li tocca più.
- **Tempi**: le Frasi hanno già `inizio_ms` e `fine_ms` sull'audio salvato, senza le pause. `inizio_ms` comprende 300 ms di prefill, quindi il salto parte poco prima del parlato, che è quello che serve per riascoltare.
- **Costo della correzione**: `bino::rewrite` copia il mix con `raw_copy_file` a ogni correzione. Su un Bino di un'ora a 32 kbps sono circa 14 MB, che vanno misurati. Se la riscrittura supera qualche centinaio di millisecondi, la correzione si salva in background e la Frase mostra lo stato di salvataggio.
- **Verifiche fatte (2026-10-04), da riportare in `docs/research/` con il primo ticket**:
  - *WebView2*: riproduce `audio/ogg; codecs=opus` nativamente (codec aperto, demuxer FFmpeg di Chromium con gestione del pre-roll in seek e del discard padding, quindi l'end trimming vale). L'input rate dell'OpusHead sotto i 48 kHz è solo informativo (RFC 7845 §5.1). Ogg non ha indice: lo spostamento fa più richieste `Range`, che il protocollo deve servire bene. La precisione dello spostamento si misura a mano via CDP.
  - *Protocollo*: l'esempio ufficiale `examples/streaming` di Tauri usa `register_asynchronous_uri_scheme_protocol` e il crate `http-range` (0.1.5), e risponde 206 con `Content-Range` limitando ogni risposta a circa 1 MB. Va adattato alla finestra della voce nello zip, con `Accept-Ranges: bytes` e `Content-Type: audio/ogg`. Su Windows l'URL è `http://<schema>.localhost/…`. wry passa il corpo come `Vec<u8>` intero, quindi il limite per risposta serve. Il protocollo `asset` gestisce `Range` ma solo su file interi, quindi non va bene per una voce dello zip.
  - *Il protocollo apre il Bino a ogni richiesta e non lo tiene aperto*: su Windows un file aperto bloccherebbe la riscrittura (correzioni, rinomina dei Parlanti), la rinomina e il Cestino mentre il player lo usa.
  - *rusqlite* 0.40.2 (libsqlite3-sys 0.38.2, SQLite 3.53.2): `bundled` compila con `cc` e `-DSQLITE_ENABLE_FTS5`, con bindings pregenerati, quindi niente bindgen, CMake o NASM; usa il CRT dinamico come il resto. `unicode61 remove_diacritics 2`, prefissi, `snippet()`, `highlight()` e `bm25()` ci sono. L'opzione `prefix='2 3'` velocizza i prefissi corti, da valutare con le misure.
- **Ticket**, in ordine suggerito:
  1. Libreria e indice (Rust) con la barra laterale minima e la Cartella della Libreria;
  2. Raccolte e operazioni sui Bini (crea, rinomina, sposta, Cestino, Aggiungi alla Libreria, titolo);
  3. ricerca;
  4. Trascrizione di un file in un Bino ed Esporta Markdown;
  5. vista di un Bino con turni, correzione per Frase e menu delle opzioni;
  6. player e protocollo;
  7. sincronizzazione testo-audio.
