# Memotape: requisiti di prodotto

Fonte di verità dei requisiti. Nasce dalla spec v1 (`.scratch/sbobino/spec.md`); da qui in poi un requisito cambia qui, non nella spec. I termini in maiuscolo (Sorgente, Attività, Frase, Parziale…) sono definiti in `CONTEXT.md`.

## Problema

Chi deve sbobinare lezioni, riunioni, interviste o video oggi carica i file su servizi online, che chiedono account e chiavi, mandano l'audio a terzi e non funzionano senza rete. Spesso non c'è nemmeno un modo semplice per registrare insieme la propria voce e l'audio del computer, per esempio in una videochiamata, e poi trascriverli. L'utente vuole un'app Windows che faccia tutto sul proprio PC: aprire un file audio o video, oppure registrare, e ottenere il testo, senza account, senza chiavi e senza rete una volta scaricato il modello.

## Soluzione

Memotape è un'app desktop solo Windows x64 (Tauri 2 + React).
- **Sorgente.** L'utente apre un file audio, video o Tape con "Apri file" o dalla Libreria, oppure lo crea con una Registrazione da microfono, audio di sistema o entrambi. Il file diventa la Sorgente.
- **Libreria.** La cartella in cui Memotape salva i Tape, sempre visibile nella barra laterale; le sue cartelle sono le Raccolte, per esempio una per cliente (ADR-0008). Un campo di ricerca trova le parole nei titoli, nelle Frasi e nei nomi dei Parlanti.
- **Trascrizione.** Trascrivi riconosce il parlato in locale con uno di tre modelli (Nemotron Streaming consigliato, Whisper Large v3 Turbo, Parakeet TDT v3). Il testo di un file compare tutto insieme in un'area dedicata a Trascrizione finita, mentre quello di una Registrazione con Trascrivi dal vivo compare Frase per Frase, con Nemotron anche come Parziale mentre la Frase è in corso. Un file trascritto diventa un Tape nella Raccolta scelta, con l'audio e il testo con i tempi (ADR-0009); il testo di un Tape si legge a turni, si corregge Frase per Frase e si esporta in Markdown.
- **Assistenti.** Con il permesso dell'utente, Claude o Codex cercano e leggono i Tape della Libreria attraverso un server MCP, anche con Memotape chiuso (ADR-0012).
- **Impostazioni.** Restano salvate tra un avvio e l'altro. L'interfaccia è disponibile in sei lingue.

Nessun ffmpeg: la decodifica è in Rust (Symphonia), le Registrazioni sono in OGG/Opus scritte in Rust e "Estrai solo audio" non fa parte del prodotto (ADR-0002). La Trascrizione parte solo dopo Stop (ADR-0003).

<!-- impeccable:product-schema 1 -->

Le sezioni da "Platform" a "Product Principles" sono il contesto di prodotto per il lavoro di design (skill impeccable): i titoli restano in inglese perché lo strumento li riconosce per nome.

## Platform

web

L'interfaccia è HTML/React dentro la WebView2 di un'app desktop Tauri, solo Windows 10/11 x64; non è un sito e non ha una versione mobile.

## Users

- **Professionisti** che sbobinano riunioni e videochiamate: registrano microfono e audio di sistema insieme e vogliono sapere chi ha detto cosa (Parlanti).
- **Studenti** che trascrivono lezioni registrate o seguite dal vivo.
- **Pubblico generico**: chiunque abbia un file audio o video da trasformare in testo, senza competenze tecniche.

Nessuna priorità stabilita tra i tre gruppi.

## Product Purpose

Trasformare in testo un file audio o video, o una Registrazione fatta nell'app, tutto sul PC dell'utente. Successo: il testo compare mentre si registra (con Trascrivi dal vivo) o appena finisce la Trascrizione di un file, si salva in un Tape nella Libreria e si copia o si esporta altrove, senza account, chiavi né rete una volta scaricato il modello.

## Positioning

Tutto in locale: l'audio non lascia il PC, niente account né chiavi, e dopo il download del modello funziona offline. È la prima differenza rispetto ai servizi online; la Registrazione di voce e audio di sistema con il testo dal vivo viene dopo.

## Operating Context

- Finestra desktop da 1000×700, accanto ad altre app: una videochiamata durante la Registrazione, l'editor in cui incollare il testo, Esplora file per i Tape (doppio clic apre Memotape).
- Lavori lunghi in background: download di modelli da centinaia di MB, Trascrizioni di ore, completamento e Riconoscimento dei parlanti dopo Stop. Una sola Attività alla volta; la sezione Attività della barra laterale dice sempre la fase.
- Il testo di un Tape si legge a turni e si corregge Frase per Frase; Copia testo ed Esporta Markdown… sono le uscite.

## Capabilities and Constraints

- Le funzioni sono le storie utente qui sotto; i termini sono quelli di `CONTEXT.md` (Sorgente, Attività, Frase, Parziale, Parlante, Tape…) e vanno usati uguali nell'interfaccia.
- Sei lingue dell'interfaccia (it di riferimento, en, fr, es, de, pl): le etichette devono reggere i testi più lunghi di tedesco e polacco.
- Tema chiaro o scuro da Windows, oppure scelto in Impostazioni; dialog di sistema nativi per file e cartelle.
- La rete serve solo per scaricare i modelli e per il controllo aggiornamenti.
- Da decidere: editore dell'installer (oggi "EDITORE DA DEFINIRE"), firma dell'installer, licenza e modalità di distribuzione dell'app.

## Brand Commitments

- Nome **Memotape**, memo + tape, la musicassetta delle note vocali (ADR-0014); il file delle Registrazioni è il **Tape** (`.tape`, tipo "Tape (Memotape)" in Esplora file).
- Icona dell'app in `src-tauri/icons/`, usata anche per i Tape.
- Voce sobria e chiara: frasi brevi e precise, senza battute, anche negli errori e nelle conferme.

## Evidence on Hand

- Audio di prova in `src-tauri/tests/fixtures/` (sintesi vocale, non materiale pubblicabile).
- Le misure di velocità in `AGENTS.md` sono di una sola macchina di sviluppo: non sono benchmark da mostrare.
- Non esistono testimonianze, clienti, numeri d'uso o recensioni: non vanno inventati.

## Product Principles

1. L'audio resta sul PC: nessuna funzione chiede account, chiavi o servizi di terzi.
2. Il lavoro dell'utente non si perde: conferma prima di sostituire il testo, nessuna sovrascrittura, la Registrazione si salva anche quando qualcosa si guasta.
3. La finestra mostra solo ciò che serve all'Attività in corso: la fase si legge sempre nella sezione Attività della barra laterale, errori ed esiti in un avviso.
4. Si comporta come un'app Windows: tema, lingua, Esplora file e dialog di sistema.
5. Parole sobrie e coerenti con il glossario, uguali nelle sei lingue.

## Storie utente

### Sorgente

1. Come utente, voglio un pulsante "Apri file" che apra il dialog di sistema, così scelgo un file senza digitare percorsi.
2. Come utente, voglio che il dialog proponga solo le estensioni accettate (MP3, WAV, M4A, FLAC, OGG, OPUS, WEBM, MPGA, MPEG, AIFF, MP4, MKV, MOV, M4V), così non scelgo file inutili.
3. Come utente, voglio vedere il nome della Sorgente nella finestra, così so su cosa sto lavorando.
4. Come utente, voglio cliccare il nome della Sorgente e aprirla con il programma associato, così la ascolto o la guardo prima di trascrivere.
5. Come utente, voglio vedere il percorso completo della Sorgente passando il mouse sul suo nome, così so dove si trova.
6. Come utente, voglio che dopo la scelta di un file mi venga proposta l'azione Trascrivi, così so cosa posso fare.
7. Come utente, voglio un errore dedicato se il file contiene un codec audio non supportato (per esempio AC-3 dentro un MKV o un `.mpeg` che è un video MPEG-PS), così capisco che il problema è il formato e non l'app.
8. Come utente, voglio un errore dedicato se il file non esiste più o non è leggibile, così so che va scelto di nuovo.

### Trascrizione

9. Come utente, voglio premere Trascrivi su un file audio e vedere comparire il testo, così ottengo la sbobinatura.
10. Come utente, voglio trascrivere anche un video MP4, MOV, M4V o MKV senza prima estrarne l'audio, così risparmio un passaggio.
11. Come utente, voglio il testo a turni, con le Frasi di un turno una dopo l'altra, così si legge facilmente.
12. Come utente con Nemotron, voglio vedere il Parziale della Frase in corso mentre viene riconosciuta, così seguo il lavoro in tempo reale.
13. Come utente con Whisper o Parakeet, voglio vedere ogni Frase appena è conclusa, così vedo comunque l'avanzamento.
14. Come utente, voglio la percentuale di avanzamento nella sezione Attività e in fondo alla vista quando la durata è nota, così so quanto manca.
15. Come utente, voglio un avanzamento senza percentuale quando la durata non è nota, così so comunque che l'app sta lavorando.
16. Come utente, voglio che Trascrivi su un file audio o video crei `<nome del file>.tape` nella Raccolta scelta (" 2", " 3"… se esiste già), con l'audio in Opus alle impostazioni di Registrazione e il testo con i tempi, così ritrovo la trascrizione nella Libreria e il file originale resta com'è. Se la Trascrizione non trova parlato il Tape non si crea e un avviso dice "Nessun parlato rilevato".
17. Come utente, voglio che a fine Trascrizione il Tape diventi la Sorgente e un avviso mostri il suo percorso, così so dove trovarlo.
18. Come utente, voglio un pulsante "Copia testo" che copi negli appunti il testo del Tape aperto, con le correzioni e i nomi dei Parlanti, o quello della Trascrizione in corso, in testo semplice o in Markdown secondo le Impostazioni, così incollo il testo altrove.
19. Come utente, voglio una conferma prima che Trascrivi su un Tape ne sostituisca il testo, correzioni comprese, così non lo perdo per errore.
20. Come utente, voglio un pulsante "Annulla" durante la Trascrizione, così fermo un lavoro lungo avviato per sbaglio.
21. Come utente, voglio che dopo Annulla il testo già comparso di un file resti visibile ma che nessun Tape venga creato o cambiato, così un Tape ha solo Trascrizioni complete.
22. Come utente, voglio scegliere la Lingua del parlato con un selettore accanto a Trascrivi, così aiuto il modello quando il riconoscimento automatico sbaglia.
23. Come utente, voglio che il selettore della Lingua del parlato abbia "Automatica" come default e offra tutte le lingue riconosciute dal modello selezionato, in ordine di nome, così non scelgo combinazioni impossibili e non sono limitato alle lingue dell'interfaccia.
24. Come utente, voglio che la Lingua del parlato scelta resti salvata tra un avvio e l'altro, così non la reimposto ogni volta.
25. Come utente, voglio che la Trascrizione funzioni senza rete una volta scaricato il modello, così lavoro anche offline.
26. Come utente, voglio che senza un modello scaricato Trascrivi mostri un errore dedicato con un link a Impostazioni → Trascrizione, così so cosa fare.
27. Come utente, voglio che mentre una Trascrizione è in corso Registra e Sfoglia siano disabilitati, così non avvio due Attività insieme.
28. Come utente, voglio che una Trascrizione che fallisce mostri un errore dedicato in un avviso, così capisco cosa è successo.

### Modelli

29. Come utente, voglio vedere in Impostazioni → Trascrizione i tre modelli con dimensione del download e modalità del testo (in streaming o a fine frase), così scelgo consapevolmente.
30. Come utente, voglio che Nemotron sia indicato come consigliato e selezionato per default, così parto dalla scelta migliore.
31. Come utente, voglio scaricare un modello vedendo l'avanzamento in percentuale, così so quanto manca.
32. Come utente, voglio che un modello diventi utilizzabile solo dopo la verifica d'integrità (dimensione e SHA-256), così non trascrivo con un file corrotto.
33. Come utente, voglio un errore dedicato se la verifica fallisce, così riprovo il download.
34. Come utente, voglio annullare un download in corso e che il file parziale venga cancellato, così libero spazio.
35. Come utente, voglio che un download interrotto (connessione persa, app chiusa) riprenda da dove si era fermato, così non riscarico centinaia di MB.
36. Come utente, voglio eliminare un modello scaricato, così libero spazio su disco.
37. Come utente, voglio che "Elimina" sia disabilitato per il modello che una Trascrizione in corso sta usando, così non la rompo.
38. Come utente, voglio che il download continui in background mentre registro o trascrivo con un altro modello, così non aspetto.
39. Come utente, voglio scegliere quale modello usare, così confronto qualità e velocità.

### Registrazione

40. Come utente, voglio registrare dal Microfono, dall'Audio di sistema o da Entrambi, così catturo la mia voce, una videochiamata o tutte e due.
41. Come utente, voglio che con "Entrambi" le due sorgenti finiscano mixate in un solo file, così ho un'unica traccia da trascrivere.
42. Come utente, voglio usare i dispositivi predefiniti oppure sceglierne uno tra quelli rilevati, così registro dal microfono o dalle cuffie giuste.
43. Come utente, voglio un timer della durata registrata che non conti le pause, così so quanto dura il file.
44. Come utente, voglio un indicatore di livello per ciascuna sorgente attiva, così vedo che entrambe stanno arrivando.
45. Come utente, voglio Pausa e Riprendi nella stessa sessione, così salto le parti che non mi interessano senza creare più file.
46. Come utente, voglio che il file non contenga vuoti per le pause, così l'ascolto e la Trascrizione sono continui.
47. Come utente, voglio che Stop salvi il file nella Cartella predefinita come `Registrazione <data ora>.ogg`, con il prefisso nella Lingua dell'interfaccia, così trovo le registrazioni in ordine.
48. Come utente, voglio che se quel nome esiste già venga aggiunto " 2", " 3"…, così nulla viene mai sovrascritto.
49. Come utente, voglio che dopo Stop la Registrazione diventi la Sorgente, così posso subito premere Trascrivi.
50. Come utente, voglio che con l'Audio di sistema la registrazione continui anche quando il PC è in silenzio, inserendo silenzio nel file, così timer e audio restano allineati.
51. Come utente, voglio che se un dispositivo si scollega durante la Registrazione questa si fermi come con Stop, salvando quanto registrato, e un avviso mostri l'errore con il nome del dispositivo, così non perdo nulla.
52. Come utente, voglio che la Registrazione usi il bitrate, i canali e la frequenza delle Impostazioni, così controllo qualità e dimensione.
53. Come utente, voglio che durante una Registrazione Sfoglia e Trascrivi siano disabilitati, così non avvio due Attività insieme.
73. Come utente, voglio regolare il Guadagno di ciascun Ingresso accanto al suo indicatore, anche durante la Registrazione, così alzo un microfono troppo basso o abbasso un audio di sistema troppo forte senza fermarmi.
75. Come utente che registra da Entrambi, voglio che Microfono e Audio di sistema non finiscano mai nello stesso Parlante né nella stessa Frase, dal vivo, trascrivendo dopo o ritrascrivendo il Tape, così chi parla al microfono resta distinto da chi arriva dalla videochiamata (ADR-0015).
76. Come utente che registra da Entrambi, voglio che il Tape contenga sempre l'audio di ogni Ingresso, anche senza Trascrivi dal vivo, così posso trascriverlo dopo per Ingresso.
77. Come utente, voglio che con "Riconosci i parlanti" si distinguano le voci dell'Audio di sistema e che il Microfono resti una persona sola, salvo che attivi la sua casella (più persone allo stesso microfono), così la videochiamata ha i suoi Parlanti e io resto "Microfono". Ritrascrivendo un Tape da Entrambi vale la stessa regola, con la casella del Microfono di Impostazioni → Registrazione.
78. Come utente, voglio che una Continuazione da Entrambi su un Tape senza l'audio degli Ingressi lo aggiunga, con silenzio per la parte già registrata, così le Frasi nuove sono separate per Ingresso.

### Impostazioni

54. Come utente, voglio scegliere la sorgente di registrazione predefinita e i dispositivi, così non li reimposto ogni volta.
55. Come utente, voglio scegliere il bitrate tra 16, 24, 32, 48, 64, 96, 128, 192 e 320 kbps, così adatto la qualità.
56. Come utente, voglio scegliere mono o stereo e la frequenza tra 8 000, 16 000, 24 000 e 48 000 Hz, così adatto il file all'uso.
57. Come utente, voglio come predefiniti 16 kbps, mono, 16 kHz, così ho subito un audio sufficiente per la voce e Tape piccoli. Chi ha già le impostazioni salvate le tiene.
58. Come utente, voglio scegliere la Cartella della Libreria, che in mancanza è `Documenti\Memotape` e viene creata se non esiste, così so dove finiscono i Tape.
59. Come utente, voglio scegliere la Lingua dell'interfaccia tra it, en, fr, es, de e pl, con un avviso che si applica al riavvio, così uso l'app nella mia lingua.
60. Come utente al primo avvio, voglio l'interfaccia nella lingua del sistema se è tra le sei, altrimenti in inglese, così non devo cercare l'impostazione.
61. Come utente, voglio che tutte le impostazioni restino salvate tra un avvio e l'altro, così l'app riparte come l'ho lasciata.
62. Come utente, voglio che un file impostazioni corrotto non impedisca l'avvio ma riporti ai valori predefiniti, così l'app parte sempre.
63. Come utente, voglio una sezione Informazioni con la versione dell'app e le licenze dei componenti (modelli, Symphonia, ONNX Runtime, Silero, transcribe-cpp), così l'app rispetta le attribuzioni richieste.
74. Come utente, voglio impostare il Guadagno di ciascun Ingresso anche in Impostazioni, prima di registrare, così parto già con i livelli giusti.

### Finestra e aggiornamenti

64. Come utente, voglio una barra laterale sempre visibile (Nuova registrazione, Importa un file, ricerca, Attività, Recenti, Libreria, Impostazioni) e un pannello centrale che mostra il Tape aperto come un documento, la Registrazione in corso o la Libreria, così la finestra mostra solo ciò che serve.
65. Come utente, voglio che la sezione Attività mostri sempre la fase in corso e la percentuale quando c'è, e che errori ed esiti compaiano in un avviso con messaggi dedicati, così so sempre cosa succede.
66. Come utente, voglio che all'avvio, se c'è connessione, l'app controlli se esiste una versione più recente e mi proponga il link per scaricarla, così resto aggiornato.
67. Come utente offline, voglio che il controllo aggiornamenti fallisca in silenzio, così non vedo errori inutili.
68. Come utente, voglio che il tema chiaro o scuro segua quello di Windows, così l'app si integra con il sistema; in Impostazioni → Generale posso invece sceglierlo chiaro o scuro, e la scelta si applica subito.

### Sviluppo e distribuzione

69. Come sviluppatore, voglio che `typecheck`, `test`, `check`, `format:backend`, `lint:backend` e `cargo test` passino a ogni milestone, così il progetto resta sano.
70. Come sviluppatore, voglio che i comandi e gli eventi Tauri siano tipizzati da un `bindings.ts` generato da Rust, così frontend e backend non divergono.
71. Come sviluppatore, voglio che `AGENTS.md` documenti comandi, prerequisiti di build, architettura e insidie, così un agente o un collega riparte senza chiedere.
72. Come utente finale, voglio un installer NSIS che includa tutto il necessario (DLL di runtime, modello Silero, testi delle licenze), così installo e uso l'app su qualsiasi PC Windows x64 recente.

## Stato

- **M1 (tracer bullet)**: storie 1, 2, 9, 11 e 13. Sfoglia sceglie un file audio, Trascrivi mostra le Frasi una per riga mentre arrivano. Il modello è sempre Nemotron, il predefinito del catalogo (la scelta arriva con il ticket 06). Il nome della Sorgente compare già, non ancora cliccabile (storia 4).
- **M2, prima parte (ticket 03)**: storie 3–5, 7, 8, 10, 14–18, 28, 64 e 65.
  - Il nome della Sorgente si clicca per aprirla con il programma associato; il percorso sta nella status bar.
  - Si trascrivono anche i video MP4, MOV, M4V e MKV, senza file intermedi.
  - La status bar mostra la fase, la percentuale (nei MKV, che non dichiarano la durata, un avanzamento senza percentuale), l'esito con caratteri e percorso del TXT, e gli errori dedicati.
  - Il TXT si salva accanto alla Sorgente. "Copia testo" copia l'area.
  - La sezione Trascrizione compare con la prima Trascrizione.
  - Storia 6: Trascrivi è già l'azione proposta, si abilita appena c'è una Sorgente.
- **M2, seconda parte (ticket 04)**: storie 19–21 e 27.
  - Durante la Trascrizione c'è "Annulla": il testo già comparso resta, il TXT non si salva e la status bar lo dice. Con Nemotron si interrompe anche la Frase in corso.
  - Se l'area contiene testo, anche modificato a mano, Trascrivi chiede conferma prima di sostituirlo.
  - Una sola Attività alla volta: durante la Trascrizione Sfoglia e Trascrivi sono disabilitati, e il backend rifiuta una seconda Attività con l'errore "Attività in corso". Registra si aggancerà allo stesso controllo (ticket 08).
  - Una Trascrizione senza Frasi non crea il TXT e mostra "Nessun parlato rilevato".
- **M3, prima parte (ticket 05)**: storie 29, 31–36 e 38; della 30 c'è l'indicazione "Consigliato".
  - Impostazioni si apre dall'icona accanto a Trascrivi, sopra la finestra principale, che non perde testo né Trascrizione in corso. Per ora contiene solo la sezione Trascrizione.
  - Ogni modello mostra la dimensione in MB (binari, come Esplora file), la modalità del testo (in streaming o a fine frase), la licenza e lo stato: non scaricato, download in corso con percentuale, interrotto con percentuale, verifica, scaricato.
  - Scarica, Annulla (cancella il file incompleto, anche durante la verifica), Riprendi (da dove si era fermato, anche dopo la chiusura dell'app) ed Elimina, con conferma; Elimina toglie anche un file incompleto e, durante un download, vale come Annulla. Si possono scaricare più modelli insieme.
  - Il download continua mentre si usa il resto dell'app. Un download fallito o un modello corrotto (SHA-256 errato, file incompleto cancellato) mostrano un errore dedicato sulla riga del modello, che resta finché non si riprova.
- **M3, seconda parte (ticket 06)**: storie 22–26, 30, 37, 39 e 62; della 61 restano salvati modello e Lingua del parlato, gli altri campi esistono già nel file e diventano scelte con i ticket 08 e 10.
  - In Impostazioni → Trascrizione si sceglie il modello, Nemotron di default. Si può scegliere anche un modello non scaricato: Trascrivi mostra "modello assente" con il link a Impostazioni → Trascrizione.
  - Il selettore della Lingua del parlato sta accanto a Trascrivi: "Automatica" e tutte le lingue del modello scelto (32 per Nemotron, 99 per Whisper, 25 per Parakeet), con il nome nella Lingua dell'interfaccia. Finché il modello non è caricato offre solo la scelta salvata.
  - Con una Lingua del parlato scelta, una Frase con lettere di un'altra scrittura (il cirillico in una Trascrizione in italiano; il latino vale sempre) si scarta: è un'invenzione del modello su audio incomprensibile. Parakeet non usa la lingua indicata e la riconosce Frase per Frase: il menu lo dice, e la scelta serve solo a questo filtro. Whisper scarta la Frase quando la sua confidenza è bassa, invece di ritentare a temperature più alte, che producevano parole a caso in più lingue.
  - Le impostazioni stanno in `settings.json` nella cartella dati dell'app. Un file mancante, corrotto o non valido riporta ai predefiniti senza bloccare l'avvio.
  - Il modello scelto si carica in background all'avvio e quando cambia la scelta, e resta caricato tra una Trascrizione e l'altra: dalla seconda in poi la Trascrizione parte subito. Si ricarica solo se cambia la scelta o se il modello viene eliminato.
  - Elimina è disabilitato ("In uso") per il modello che una Trascrizione sta usando o che si sta caricando.
- **M4 (ticket 07)**: storia 12.
  - Con Nemotron il testo della Frase in corso compare come Parziale nell'ultima riga dell'area e si aggiorna mentre il modello ascolta; a fine Frase diventa definitivo e la Frase successiva parte su una riga nuova. Con Whisper e Parakeet ogni Frase compare intera a fine Frase, come prima.
  - Annulla toglie il Parziale della Frase interrotta: restano solo le Frasi concluse, quelle già comparse.
- **M5 (ticket 08)**: storie 42 (microfono), 43, 45–49, 51–53 e 55–58; della 40 e della 44 c'è il Microfono, della 54 la scelta del microfono, della 61 anche le impostazioni audio e la Cartella predefinita.
  - "Registra" accanto a Sfoglia registra dal microfono predefinito di sistema o da quello scelto in Impostazioni. Durante la Registrazione compaiono il timer, l'indicatore di livello (in decibel, giallo e rosso vicino alla saturazione), Pausa/Riprendi e Stop; Sfoglia, Trascrivi e la Lingua del parlato sono disabilitati.
  - Il timer non conta le pause e il file non ha vuoti al loro posto. Un'interruzione dell'audio dal dispositivo diventa silenzio, così il file resta allineato al tempo.
  - Stop salva `Registrazione AAAA-MM-GG HH-MM-SS.ogg` (OGG/Opus) nella Cartella predefinita, con " 2", " 3"… se il nome c'è già, e il file diventa la Sorgente: Trascrivi è subito disponibile. Il file si scrive mentre si registra.
  - Se il microfono si scollega (o cambia il microfono predefinito di Windows) la Registrazione si ferma e si salva come con Stop, e la status bar mostra l'errore con il nome del dispositivo. Un microfono scelto ma non collegato dà un errore dedicato invece di registrare da un altro.
  - Impostazioni ha le sezioni Registrazione e audio (microfono, bitrate tra i nove valori, mono/stereo, 8/16/24/48 kHz; predefiniti 32 kbps, mono, 48 kHz) e Generale (Cartella predefinita, `Documenti\Memotape` se non scelta, creata se manca). Le scelte valgono dalla Registrazione successiva.
- **M6 (ticket 09)**: storie 40, 41, 44 e 50; della 42 e della 54 anche il dispositivo di uscita e la sorgente di registrazione.
  - In Impostazioni → Registrazione e audio, "Registra da" sceglie Microfono, Audio di sistema o Entrambi (predefinito Microfono), e "Dispositivo di uscita" sceglie da quale uscita prendere l'audio di sistema (predefinito di sistema). Registra usa la scelta salvata.
  - Con Entrambi microfono e audio di sistema finiscono mixati in un solo file, allineati nel tempo. Durante la Registrazione c'è un indicatore di livello per ciascuno.
  - Quando il PC è in silenzio la Registrazione dell'audio di sistema continua: il file si riempie di silenzio e resta lungo quanto il timer. Con solo l'Audio di sistema il timer può restare indietro di mezzo secondo durante il silenzio e poi recuperare.
  - Un'uscita scelta ma non collegata dà un errore dedicato; se si scollega durante la Registrazione questa si ferma e si salva come per il microfono.
- **M7 (ticket 10)**: storie 59, 60, 63 e 68; con la Lingua dell'interfaccia anche la 61 è completa, e la 62 copre anche un file illeggibile.
  - L'interfaccia è in italiano, inglese, francese, spagnolo, tedesco e polacco, con le forme plurali di ogni lingua. In Impostazioni → Generale si sceglie la Lingua dell'interfaccia, con l'avviso che vale dal prossimo avvio. Finché non la si sceglie vale la lingua di Windows se è tra le sei, altrimenti l'inglese. Anche il prefisso delle Registrazioni segue la Lingua dell'interfaccia.
  - Il tema chiaro o scuro segue quello di Windows, compresi i controlli nativi come i selettori.
  - Impostazioni → Informazioni mostra la versione dell'app e, per ogni componente, autore, licenza, ruolo e fonte, con il testo della licenza da aprire: Parakeet (CC BY 4.0, con l'attribuzione e la conversione GGUF di handy-computer), Nemotron (OpenMDW-1.1), Whisper (MIT di OpenAI), Silero VAD, Symphonia (MPL-2.0), transcribe.cpp (con ggml e miniz), ONNX Runtime (con gli avvisi di terze parti) e vad-rs. I testi sono anche file del bundle, nella cartella `licenses`.
  - Se il file impostazioni esiste ma non si legge (per esempio bloccato da un altro programma), l'app parte con i valori predefiniti e la lingua di Windows, e la status bar mostra l'errore. Se il file è bloccato solo per un attimo (fino a circa un secondo) si legge lo stesso. Le impostazioni salvate non si perdono: finché il file non si legge una modifica non si salva e la status bar lo dice; appena si legge, la modifica si aggiunge alle impostazioni del file.
- **M9 (ticket 12)**: storia 72.
  - `Memotape_<versione>_x64-setup.exe` (NSIS) installa l'app per l'utente corrente, senza diritti di amministratore, in `%LOCALAPPDATA%\Memotape`, con il collegamento nel menu Start. Accanto all'exe mette l'ONNX Runtime, transcribe.cpp con i suoi backend (CPU scelta a runtime per il processore della macchina, Vulkan se c'è una GPU con driver Vulkan) e il runtime VC++, quindi non serve installare altro (su un Windows 10 senza WebView2 l'installer la scarica); con i file arrivano Silero e i testi delle licenze.
  - La disinstallazione toglie l'app e lascia impostazioni e modelli scaricati, a meno di spuntare la cancellazione dei dati dell'app.
  - L'editore dell'installer non è ancora deciso: oggi è il segnaposto "EDITORE DA DEFINIRE".
- **V1 (ticket v2/02)**: storie 1–12 della spec v2 (`.scratch/sbobino-v2/spec.md`) sul mix; il testo si salvava nel TXT (dalla V3 è Markdown), il Tape arriva con V2.
  - La casella "Trascrivi dal vivo" accanto a Registra, spenta per default e salvata tra un avvio e l'altro, fa comparire il testo mentre si registra: con Nemotron i Parziali, con Whisper e Parakeet ogni Frase a fine Frase. Durante la Registrazione la casella è bloccata.
  - La Registrazione non rallenta mai: se il motore resta indietro le Frasi vanno in coda. Dopo Stop la status bar mostra "Completamento della trascrizione…" con l'avanzamento e Annulla finché la coda non è vuota; annullando, la Registrazione resta salvata e il testo no.
  - Pausa chiude la Frase in corso, Riprendi continua nella stessa sessione; il testo non contiene l'audio in pausa.
  - A fine Registrazione il testo si salva in `<Registrazione> trascrizione 1.md` accanto alla Registrazione (dalla V2 il Tape) e la status bar lo dice.
  - Con testo nell'area, Registra con la casella attiva chiede conferma prima di sostituirlo.
  - Se il modello scelto non è scaricato o non si carica la Registrazione parte comunque, con un avviso e il link alle Impostazioni.
- **V3 (ticket v2/03)**: storie 26–28 e 30 della spec v2; la 29 (turni con Parlanti e Ingressi) è arrivata con i ticket v2/06 e v2/07.
  - Il risultato di una Trascrizione, anche dal vivo, è `<nome Sorgente> trascrizione <N>.md` invece del TXT, con la stessa regola del primo N libero. Ha come titolo il nome della Sorgente e un'intestazione con data e ora, durata, modello e Lingua del parlato, nella Lingua dell'interfaccia.
  - Le Frasi si uniscono in paragrafi, e se ne apre uno nuovo dopo oltre 2 s di silenzio.
  - In Impostazioni → Generale, "Copia testo come" sceglie testo semplice (predefinito) o Markdown. Copia testo copia l'ultima Trascrizione, anche annullata o in corso, nel formato scelto; se il testo nell'area è stato modificato a mano lo copia com'è.
- **V2, Tape (ticket v2/04 e v2/05)**: storie 18–25 della spec v2. Sostituisce la storia 47 e il punto di M5 sul file `.ogg`.
  - Stop salva un solo `Registrazione <data ora>.tape` nella Cartella predefinita, con " 2", " 3"… se il nome c'è già, che diventa la Sorgente. Contiene l'audio del mix e il testo con i suoi metadati; con la Trascrizione dal vivo accanto c'è anche il Markdown, che prende il nome dal Tape.
  - Durante la Registrazione l'audio si scrive in un Ogg nella cartella nascosta `.memotape` dentro la Cartella predefinita: dopo un crash è lì. A Stop, finita la coda della Trascrizione dal vivo, diventa il Tape e la cartella sparisce se resta vuota. Se il Tape non si può scrivere, la Registrazione resta come `Registrazione <data ora>.ogg` nella Cartella predefinita e diventa la Sorgente, con l'errore nella status bar.
  - Annullando il completamento della trascrizione dopo Stop il Tape si salva lo stesso, con le Frasi già pronte e segnato come incompleto, mentre il Markdown no. Senza Trascrizione dal vivo il Tape non ha testo.
  - Sfoglia accetta anche i `.tape`: un Tape si apre con il suo testo nell'area, senza ritrascrivere, e Copia testo lo rende con la sua intestazione. Se l'area contiene già testo chiede conferma. Un Tape di una versione più nuova dell'app dà un errore che chiede di aggiornare.
  - Trascrivi su un Tape ne decodifica l'audio e, dopo la conferma (chiesta sempre, anche con l'area vuota, e che avvisa che cambia anche il Tape), sostituisce il testo dentro il Tape e salva un nuovo Markdown accanto. Annullando, il Tape non cambia.
  - Il clic sul nome di un Tape lo mostra in Esplora file invece di aprirlo. Le vecchie Registrazioni `.ogg` si aprono ancora come Sorgente.
  - L'installer associa `.tape` a Memotape (tipo "Tape (Memotape)", con l'icona dell'app) e la disinstallazione toglie l'associazione. Il doppio clic su un Tape in Esplora file avvia Memotape con quel Tape come Sorgente; se Memotape è già aperto, il Tape arriva alla finestra esistente, che torna in primo piano, e non si apre una seconda finestra. Come con Sfoglia, se l'area contiene testo chiede conferma. Durante una Trascrizione o una Registrazione, o con una conferma aperta, il Tape si apre quando finiscono; se è aperta Impostazioni, si chiude.
- **V4, Ingressi separati (ticket v2/06)**: storie 13–17 della spec v2 e la parte della 29 sugli Ingressi.
  - In Impostazioni → Registrazione, "Trascrizione dal vivo" sceglie tra Mix (predefinito) e Ingressi separati; è attivo solo con "Registra da" su Entrambi e vale solo con "Trascrivi dal vivo" attiva. Superato dalle storie 75–78 (ADR-0015): da Entrambi si trascrive sempre per Ingresso e il selettore sparisce.
  - Con Ingressi separati microfono e audio di sistema si trascrivono in parallelo, ognuno con la sua istanza del modello (il doppio della memoria), caricata all'inizio della Registrazione e liberata alla fine. Se un Ingresso si guasta compare l'avviso e l'altro continua; il Markdown non si salva.
  - Il testo è una conversazione: le Frasi in ordine di inizio, non di arrivo, e ogni turno di un Ingresso comincia con l'etichetta "Microfono:" o "Audio di sistema:" su una riga. Con Nemotron ogni Ingresso ha i suoi Parziali, al loro posto.
  - Il Tape contiene anche `microfono.ogg` e `sistema.ogg` ed è segnato come Ingressi separati; riaperto mostra la conversazione. Il Markdown e Copia testo hanno un paragrafo per turno, `**Microfono:**` e `**Audio di sistema:**`. Trascrivi su un Tape trascrive sempre il mix (superato dalla storia 75: un Tape con l'audio degli Ingressi si ritrascrive per Ingresso).
- **V5, Diarizzazione dei file (ticket v2/07)**: storie 31 e 34–38 della spec v2 per i file e la parte della 29 sui Parlanti; le Registrazioni (32, 33) sono arrivate con il ticket v2/08.
  - Impostazioni → Trascrizione ha, separato dai modelli di trascrizione, "Riconoscimento dei parlanti" con Sortformer 4spk v2.1 (133 MB, NVIDIA Open Model License): download con percentuale, verifica, ripresa ed Elimina come gli altri, ma non si sceglie. Informazioni ne mostra la licenza.
  - La casella "Riconosci i parlanti" accanto a Trascrivi, spenta per default e salvata tra un avvio e l'altro, dice nel suggerimento che si riconoscono al massimo 4 Parlanti. Senza Sortformer scaricato Trascrivi mostra subito l'errore "il modello per Riconosci i parlanti non è scaricato" con il link alle Impostazioni, e durante la Trascrizione Sortformer non si può eliminare.
  - Finita la Trascrizione, nella stessa Attività, la status bar dice "Riconoscimento dei parlanti…" e Annulla resta disponibile (annullando non si salva il Markdown). Ogni Frase va al Parlante che parla di più durante la Frase, numerato per ordine di comparsa; nell'area e nel Markdown ogni turno comincia con "Parlante N:". Una Frase in cui nessuno parla resta senza etichetta.
  - Trascrivi su un Tape con la casella attiva salva i Parlanti dentro il Tape, e riaprendolo tornano.
- **V5, Diarizzazione delle Registrazioni (ticket v2/08)**: storie 32 e 33 della spec v2.
  - Impostazioni → Registrazione ha "Riconosci i parlanti": una casella "Sul mix", oppure, con Ingressi separati e "Registra da" su Entrambi, una per Microfono e una per Audio di sistema. Valgono con "Trascrivi dal vivo" attiva; spente per default.
  - Dopo Stop, finito il completamento della trascrizione, la status bar dice "Riconoscimento dei parlanti…" (con Annulla) e la Diarizzazione gira sull'audio scelto. Le Frasi già comparse prendono le etichette "Parlante N" o, con gli Ingressi separati, "Audio di sistema · Parlante N", numerate per Ingresso; lo stesso nel Markdown, in Copia testo e nel Tape, che riaperto le mostra.
  - Senza Sortformer scaricato la Registrazione parte lo stesso: la status bar avvisa che il modello per Riconosci i parlanti non è scaricato, con il link alle Impostazioni, e il testo resta senza Parlanti. Annullando durante il Riconoscimento dei parlanti il Tape si salva con le Frasi senza Parlanti e il Markdown no.
- **V6, Rinomina dei Parlanti (ticket v2/09)**: storia 39 della spec v2.
  - A Trascrizione finita, sotto l'area, "Parlanti:" elenca i Parlanti del testo in ordine di comparsa. Un clic su uno, o sulla riga della sua etichetta nell'area, apre sul posto un campo con il nome: Invio conferma, Esc o un clic altrove annulla; un nome vuoto, o già di un altro Parlante dello stesso Ingresso, non si conferma.
  - Il nome sostituisce "Parlante N" solo per quel Parlante di quell'Ingresso ("Audio di sistema · Mario"), in tutte le sue Frasi: nell'area, in Copia testo, nel Tape (se la Sorgente è un Tape, in `parlanti`) e nell'ultimo Markdown prodotto per quella Sorgente, che si riscrive da capo invece di crearne uno nuovo (le modifiche fatte a mano nel Markdown si perdono). Di un Tape riaperto vale il suo Markdown modificato per ultimo; un Markdown cancellato non si ricrea.
  - Se il testo nell'area è stato modificato a mano, la rinomina cambia solo le righe delle etichette e lascia il resto com'è.
  - Un Tape riaperto mostra i nomi. Ritrascrivere un Tape li toglie, perché i Parlanti si rinumerano.
- **V7, Trascrivi più veloce (ticket v2/10)**: per Trascrivi su un file o su un Tape sostituisce le storie 12 e 13 e il punto di M4. Cambia la 21, la 18 e il punto di V3 su Copia testo. La Trascrizione dal vivo non cambia (ADR-0007).
  - Durante la Trascrizione di un file l'area resta vuota e la status bar mostra la percentuale e Annulla; il testo compare tutto insieme a Trascrizione finita. Con Nemotron non ci sono più Parziali.
  - Annullando, l'area mostra le Frasi già trascritte e Copia testo le copia; il Markdown non si salva, come prima.
  - Copia testo è disabilitato mentre la Trascrizione di un file è in corso.
  - La Trascrizione è più veloce: su 10 minuti di riunione Nemotron passa da 93 a 32 s, Parakeet da 20 a 15 s, Whisper da 42 a 38 s (RTX 2070 SUPER).
  - Anche la prima Trascrizione dopo l'avvio, o dopo il cambio di modello, parte senza l'attesa del riscaldamento del modello: il riscaldamento avviene nel caricamento in background.
- **V8, Libreria e barra laterale (ticket libreria/01)**: storie 1–16, 18, 19, 21 e 23–34 della spec v3 (`.scratch/sbobino-libreria/spec.md`); della 17, 20 e 22 la parte senza la nuova vista del Tape (ticket libreria/03), della 100 e della 101 aprire, leggere, copiare, rinominare, spostare ed eliminare altri Tape durante un'Attività, e poi 103, 104, 106–108. Cambiano le storie 1, 58 e 64.
  - La finestra ha una barra laterale: Registra, Apri file e Trascrivi dal vivo; "Attività in corso" (timer o percentuale, riporta alla sua vista); il selettore della Raccolta (Tutta la Libreria, Senza raccolta, le Raccolte, Nuova Raccolta…) con Rinomina ed Elimina; i Tape della Raccolta dal più recente in Oggi, Ieri, Questa settimana e poi per mese, con titolo, ora e durata, e quello aperto evidenziato; in fondo "Tutti i Tape della Raccolta" (elenco completo ordinabile per data o titolo, con Sposta in… ed Elimina) e Impostazioni.
  - La Libreria è la Cartella della Libreria (prima Cartella predefinita); le Raccolte sono le sue cartelle di primo livello, i Tape nella radice sono Senza raccolta, quelli più in profondità contano nella Raccolta che li contiene. Quello che si fa in Esplora file si vede quando l'app torna in primo piano o si apre un Tape. Cambiare cartella mostra i Tape della nuova senza spostare quelli della vecchia.
  - Sopra il testo di un Tape aperto c'è il suo titolo, il nome del file: un clic lo rinomina (vuoto, caratteri non ammessi da Windows o un titolo già nella stessa cartella non si confermano). Accanto ci sono Mostra in Esplora file, Sposta in… ed Elimina, che dopo una conferma lo manda nel Cestino di Windows (su un volume senza Cestino rifiuta). Un Tape fuori dalla Libreria si apre come prima e ha "Aggiungi alla Libreria…", che lo sposta nella Raccolta scelta.
  - Le Raccolte si creano, si rinominano (con la cartella) e si eliminano solo se vuote, con gli stessi controlli sul nome. La Raccolta scelta resta al prossimo avvio; una sparita vale Tutta la Libreria.
  - Una Registrazione salva il Tape nella Raccolta scelta (con Tutta la Libreria o Senza raccolta nella radice); a Stop compare nella barra laterale e resta aperto. Durante la Registrazione la sua Raccolta non si rinomina né si elimina.
  - Durante un'Attività gli altri Tape si aprono in sola lettura, si copiano, si rinominano, si spostano e si eliminano; il Tape su cui lavora l'Attività no. Finita l'Attività torna la sua vista, con il Tape della Registrazione aperto.
  - Aprire un Tape chiede conferma solo se il testo nell'area è stato modificato a mano.
- **V9, Ricerca (ticket libreria/02)**: storie 35–40, 42, 44 e 45 della spec v3; della 41 la parte senza player (ticket libreria/04), della 43 la rinomina dei Parlanti (la correzione del testo arriva con il ticket libreria/03), della 100 cercare durante un'Attività.
  - Nella barra laterale, sotto il selettore della Raccolta, c'è un campo di ricerca. Trova le parole nei titoli, nel testo delle Frasi e nei nomi dati ai Parlanti, senza maiuscole né accenti e anche dall'inizio ("prev" trova "preventivo"); con più parole, tutte nella stessa Frase.
  - Cerca nella Raccolta scelta (o in Senza raccolta, o in Tutta la Libreria); in fondo ai risultati "Cerca in tutta la Libreria" allarga la ricerca, e i risultati mostrano allora la Raccolta di ogni Tape.
  - I risultati sono per Tape, prima quelli che parlano di più dell'argomento, ognuno con fino a cinque Frasi trovate: il tempo e un estratto con le parole evidenziate (con il nome del Parlante davanti se la parola è lì). Al massimo 50 Tape.
  - Il clic su una Frase apre il Tape, la seleziona nell'area e la porta in vista; il clic sul titolo apre il Tape dall'inizio. Durante un'Attività il Tape si consulta accanto, come dalla barra laterale.
  - Un Parlante rinominato si trova subito con il nome nuovo. L'indice resta sul PC, accanto a quello dei Tape, e se si perde si ricostruisce dai Tape.
- **V10, Vista di un Tape e file in un Tape (ticket libreria/03)**: storie 46–74 della spec v3; della 17 consultare un Tape fuori dalla Libreria con il testo (il player arriva con il ticket libreria/04), della 20 e della 22 le azioni nella vista, della 43 la correzione del testo, della 101 correggere altri Tape durante un'Attività. Cambiano le storie 11, 16–19, 21 e 64, il punto di V6 sul Markdown riscritto alla rinomina e quello di V8 sulla conferma all'apertura.
  - Trascrivi su un file audio o video crea `<nome del file>.tape` nella Raccolta scelta (con Tutta la Libreria in Senza raccolta), con " 2", " 3"… se c'è già: l'audio, ricodificato in Opus con bitrate, canali e frequenza delle Impostazioni di Registrazione, e il testo con i tempi delle Frasi. Il file originale non si tocca e il suo nome resta nelle informazioni. Finita la Trascrizione il Tape è la Sorgente. Annullata, guasta o senza parlato: nessun Tape.
  - Il Markdown non si salva più da solo: "Esporta Markdown…" lo salva dove si sceglie con il dialog di sistema, proponendo `<titolo>.md`. Rinominare un Parlante non riscrive più nessun Markdown. Solo se il Tape di una Registrazione non si scrive, il Markdown della Trascrizione dal vivo si salva accanto all'Ogg, come prima.
  - La vista di un Tape ha il titolo; una riga di informazioni (data e ora, durata, Raccolta o "fuori dalla Libreria", modello, Lingua del parlato, Ingressi separati, incompleto, il file d'origine); le azioni Trascrivi ▾, Copia testo, Esporta Markdown…, Mostra in Esplora file e "…" con Sposta in… (o Aggiungi alla Libreria…) ed Elimina; i Parlanti; il testo a turni, con l'etichetta del Parlante o dell'Ingresso a ogni turno, che un clic rinomina.
  - Un clic su una Frase la rende modificabile: Invio o l'uscita dalla Frase salvano nel Tape (i tempi, i Parlanti e l'audio restano), Esc ripristina il testo. Una Frase svuotata resta. Se il salvataggio non riesce il testo resta scritto e la status bar dice l'errore. La correzione si ritrova riaprendo il Tape e si trova subito con la ricerca. Su un Tape di un'ora il salvataggio dura circa 50 ms. Il testo non si corregge durante l'Attività che lo produce; gli altri Tape sì.
  - Trascrivi ▾ sceglie modello, Lingua del parlato e Riconosci i parlanti, e il pulsante dice la Lingua scelta ("Trascrivi · Italiano"); Registra ▾ sceglie Trascrivi dal vivo e Riconosci i parlanti della Registrazione. La conferma di Trascrivi si chiede solo su un Tape e avvisa che correzioni e nomi dei Parlanti si perdono; aprire un Tape o registrare non chiedono più conferma, perché non c'è più testo modificato a mano da perdere.
  - Copia testo copia sempre le Frasi: del Tape aperto, con correzioni e nomi, oppure della Trascrizione in corso o appena annullata.
- **V11, Player e testo collegato all'audio (ticket libreria/04)**: storie 75–99 della spec v3; della 17 il player di un Tape fuori dalla Libreria, della 41 il player portato alla Frase trovata.
  - In fondo alla vista di un Tape c'è un player che riproduce il mix, anche con gli Ingressi separati: Play/Pausa, indietro e avanti di 10 s, posizione, durata, barra di avanzamento e velocità 1×, 1,25×, 1,5× e 2×. Spazio fa Play/Pausa quando non si scrive. Il player è disabilitato durante una Registrazione e si ferma aprendo un altro Tape; un file audio o video non trascritto non ce l'ha. Lo spostamento arriva al punto chiesto entro 2 ms, con il mix a 16 e 48 kHz, mono e stereo.
  - Durante l'ascolto la Frase in riproduzione si evidenzia (più di una se si sovrappongono, con gli Ingressi separati) e resta in vista; nel silenzio resta la precedente. Scorrendo il testo a mano lo scorrimento si ferma e compare "Segui l'audio", che lo riprende; lo riprendono anche la barra del player e il salto a una Frase. Mentre si corregge una Frase il testo non scorre da solo.
  - Ogni Frase ha il pulsante del suo tempo (`12:34`), visibile passando il mouse, con il focus da tastiera, per la Frase in riproduzione e all'inizio di ogni turno: porta il player all'inizio della Frase senza cambiare Play/Pausa. Il clic sul testo serve solo a correggerlo. I tempi non entrano in Copia testo né nel Markdown.
  - Il clic su una Frase trovata con la ricerca porta lì anche il player, in pausa.
  - Correzioni e rinomine dei Parlanti si salvano anche mentre il player suona: il Tape non resta aperto tra una lettura dell'audio e l'altra.
- **V12, ridisegno della finestra (contratto in `.impeccable/surfaces/src-app-routes-home-tsx.md`)**: cambiano le storie 5, 14, 16, 17, 28, 51, 64 e 65. Dove le voci precedenti dicono "status bar", da qui vale la sezione Attività per la fase e l'avviso per errori ed esiti.
  - Aspetto chiaro e caldo (carta tiepida, inchiostro bruno, salvia per l'audio in ascolto) e una variante scura calda; segue sempre il tema di Windows. Titoli in Commissioner, testo in Inter.
  - La finestra non ha la cornice di Windows: Riduci a icona, Ingrandisci e Chiudi sono disegnati nell'app, e si trascina dalla barra in alto e dal marchio. Il riquadro di Snap Layouts sul pulsante Ingrandisci non c'è.
  - Barra laterale: Nuova registrazione ▾ (Registra da, Trascrivi dal vivo e Riconosci i parlanti nel menu), Importa un file, ricerca in tutta la Libreria (Ctrl+K, Esc la svuota), Attività (con l'avanzamento, un clic riporta alla sua vista), Recenti di tutta la Libreria per giorno, in fondo Libreria con il numero dei Tape, Impostazioni e "Solo sul tuo PC".
  - Niente status bar: la fase sta in Attività e in fondo alla vista (avanzamento con Annulla); errori ed esiti compaiono in un avviso sopra il pannello centrale, con il link alle Impostazioni quando serve. Gli esiti spariscono da soli dopo qualche secondo, gli errori restano finché non si chiudono.
  - Le Raccolte stanno nella Libreria: Tutta la Libreria, Senza raccolta, le Raccolte e Nuova Raccolta, con Rinomina ed Elimina di quella scelta. La Raccolta scelta è anche quella in cui vanno le Registrazioni e i file importati, e la Libreria lo dice. In alto il percorso "Raccolta / titolo" porta alla Libreria su quella Raccolta.
  - Un Tape si legge come un documento: titolo grande (un clic lo rinomina), giorno, ora e durata, etichette (file d'origine o Registrazione, Parlanti, modello e Lingua del parlato, incompleto, fuori dalla Libreria). In alto Copia testo e "…" con Esporta Markdown…, Mostra in Esplora file, Sposta in…, Trascrivi di nuovo con le sue scelte ed Elimina.
  - Schede Trascrizione e Parlanti. Ogni turno ha il pallino del colore della voce, il nome (un clic lo rinomina sul posto), il tempo (porta lì il player) e ▶ (lo avvia da lì). Il tempo di ogni altra Frase compare sopra di lei passando il mouse. La scheda Parlanti elenca ogni Parlante con il colore, il tempo di parola, i turni e ▶ sul primo intervento.
  - Il player sta in fondo: ±10 s, Play/Pausa, tempo, forma d'onda del mix che fa da barra di avanzamento (la parte ascoltata in salvia), durata, velocità e volume (ricordato su questo PC). Il turno in ascolto ha il fondo salvia, le barre che si muovono e Riascolta; la Frase in ascolto è evidenziata. "Segui l'audio" è un interruttore sopra il testo; scorrendo a mano si spegne e compare "Torna al punto in ascolto", con la freccia verso la Frase.
  - Senza Sorgente il pannello centrale invita a registrare o importare un file. Un Tape senza testo dice che va trascritto e offre Trascrivi ▾. Durante la Registrazione il documento "Nuova registrazione" cresce con il testo dal vivo e in fondo c'è la barra con timer, livelli, Pausa e Stop.
- **V13, data della registrazione, ordinamento e trascinamento nella Libreria**:
  - La data di un Tape è l'ora in cui è stato registrato: per una Registrazione l'inizio, per un file importato la sua data di modifica (prima era l'inizio della Trascrizione; i Tape già creati restano come sono). Un clic sulla riga "giorno · ora · durata" del Tape aperto apre il campo data e ora: Invio o l'uscita dal campo salvano, Esc annulla. Il nome del file non cambia, Recenti e Libreria si riordinano. Non si cambia mentre un'Attività lavora su quel Tape.
  - Nella Libreria le intestazioni Titolo, Data e Durata ordinano l'elenco: il primo clic va dalla più recente o più lunga (il titolo dalla A), il secondo inverte. Si parte dalla Data più recente e la scelta si ricorda su questo PC. Il menu "Ordina per" non c'è più.
  - Il titolo di una riga della Libreria si trascina su una pillola delle Raccolte, compresa Senza raccolta, per spostare il Tape: le pillole che lo accettano hanno il bordo tratteggiato e quella sotto il cursore si evidenzia. Tutta la Libreria e la Raccolta in cui il Tape sta già non lo accettano. Sposta in… resta per la tastiera.
- **V14, importa trascinando un file (ticket libreria/05, ADR-0011)**:
  - Un file audio, video o Tape trascinato da Esplora file in un punto qualsiasi della finestra si apre come con Importa un file: un file diventa la Sorgente e aspetta Trascrivi, un Tape si apre (fuori dalla Libreria resta dov'è). Funziona anche con Impostazioni aperto, che si chiude.
  - Mentre si trascina, un velo su tutta la finestra dice "Rilascia per aprire" oppure perché il rilascio non farà nulla: più di un file, un formato non accettato o una cartella (con i formati accettati), un file audio o video durante una Registrazione o una Trascrizione. In quel caso un Tape si apre in consultazione, come dalla Libreria. Con la conferma di Trascrivi di nuovo aperta il rilascio non conta.
  - Il trascinamento di un Tape su una pillola delle Raccolte di V13 non c'è più: su Windows non convive con il drop dei file. Per spostare un Tape c'è Sposta in….
- **V15, la Libreria per gli Assistenti (ADR-0012, ricerca in `docs/research/server-mcp.md`)**:
  - Claude Code, Claude Desktop e Codex collegano Memotape come server MCP locale: lo avviano loro con `memotape.exe --mcp` e funziona anche con l'app chiusa.
  - In Impostazioni → Assistenti l'interruttore "Consenti agli Assistenti di leggere la Libreria", spento di default, con la nota che il testo letto va ai server dell'Assistente. Da spento ogni richiesta risponde con un errore che dice dove accenderlo. Sotto, "Copia" per il comando di Claude Code, la tabella per `~/.codex/config.toml` e il blocco per `claude_desktop_config.json`, con il percorso vero dell'exe.
  - Un Assistente può: cercare nella Libreria come il campo di ricerca (titoli, Frasi, nomi dei Parlanti; tutta la Libreria o una Raccolta); elencare Raccolte e Tape con titolo, data e durata, anche tra due date; leggere le Frasi intorno a una Frase trovata, con tempi, Ingresso e Parlante; leggere il testo intero di un Tape, come Copia testo, a pagine. Nessuna risposta supera circa 10 000 token.
  - Vede solo la Libreria: un Tape si indica con il percorso relativo alla Libreria. Non modifica nulla e non avvia Attività.
  - Legge l'indice che l'app tiene allineato: un Tape spostato a mano con l'app chiusa non si trova finché l'app non si riapre.
  - Ogni richiesta lascia una riga nel log di Memotape.
- **V16, Copia turno**:
  - Ogni Turno ha in fondo alla sua riga, dalla parte opposta al nome, il pulsantino "Copia turno". Si vede passando il mouse sul Turno o con il focus da tastiera al suo interno; sul Turno in ascolto Riascolta gli sta a sinistra.
  - Copia sempre in testo semplice, qualunque sia "Copia testo come": il nome mostrato nel Turno seguito da due punti (`Mario: …`, con gli Ingressi separati `Microfono · Mario: …`) e le sue Frasi una dopo l'altra, con le correzioni. Senza nome, solo le Frasi. Il Parziale in corso si copia com'è al momento del clic: anche un Turno fatto solo di un Parziale ha il pulsante.
  - Per circa 1,5 s l'icona diventa una spunta e il tooltip dice "Copiato"; nessun avviso.
  - C'è ovunque si vede il testo a turni: Registrazione dal vivo, Tape aperto (anche consultato durante un'Attività), file trascritto o annullato.

- **V17, Guadagno degli Ingressi (storie 73 e 74)**:
  - Ogni Ingresso ha il suo Guadagno, indipendente dall'altro (per esempio Microfono +6 dB e Audio di sistema 0 dB), da −12 a +24 dB a passi di 3 dB; il predefinito è 0 dB. Si sceglie da una tendina: "0 dB", gli altri con il segno ("+6 dB", "−3 dB").
  - Nella barra della Registrazione c'è una tendina accanto all'indicatore di ogni Ingresso registrato. Si cambia anche in Pausa (vale dalla ripresa), non dopo Stop. Il cambio passa in pochi millisecondi, senza scatti udibili.
  - In Impostazioni → Registrazione e audio la tendina "Guadagno" sta sotto il suo dispositivo e si disattiva con lui quando l'Ingresso non si registra. Il menu ▾ di Nuova registrazione non lo mostra.
  - È un solo valore per Ingresso: cambiarlo dalla barra lo cambia anche in Impostazioni e vale per le Registrazioni successive. Non dipende dal dispositivo scelto.
  - Il Guadagno cambia l'audio prima che i due Ingressi si uniscano: il mix salvato, l'audio di ogni Ingresso con gli Ingressi separati e quello che riceve la Trascrizione dal vivo. Non tocca il volume di Windows né le altre app.
  - L'indicatore mostra il livello dopo il Guadagno. I picchi oltre il massimo si tagliano come oggi; il rosso dell'indicatore avvisa di abbassare.
  - Il Tape non ricorda il Guadagno usato: è già dentro l'audio.

## Fuori dal perimetro

- "Estrai solo audio" e qualsiasi conversione video. Niente ffmpeg (ADR-0002).
- I formati AVI, WMV, FLV, TS, MTS, MPEG-PS e i codec AC-3, E-AC-3, HE-AAC, WMA, DTS.
- La Trascrizione di più file in coda (la Trascrizione durante la Registrazione è arrivata con la v2, ADR-0004).
- Timestamp nel testo copiato o esportato, traduzione, prompt iniziale di Whisper e Lingue del parlato oltre le sei dell'interfaccia.
- Editor avanzato del testo, esportazioni diverse dal Markdown.
- Installazione automatica degli aggiornamenti (`tauri-plugin-updater`) e firma del codice.
- macOS, Linux, Windows ARM.
- Assistenti che modificano i Tape o avviano Attività, ricerca per significato, server MCP su HTTP e pacchetto `.mcpb` per Claude Desktop.
- La scelta della GPU: `transcribe-cpp` usa Vulkan se disponibile, altrimenti la CPU.
