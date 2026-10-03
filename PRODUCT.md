# Sbobino: requisiti di prodotto

Fonte di verità dei requisiti. Nasce dalla spec v1 (`.scratch/sbobino/spec.md`); da qui in poi un requisito cambia qui, non nella spec. I termini in maiuscolo (Sorgente, Attività, Frase, Parziale…) sono definiti in `CONTEXT.md`.

## Problema

Chi deve sbobinare lezioni, riunioni, interviste o video oggi carica i file su servizi online, che chiedono account e chiavi, mandano l'audio a terzi e non funzionano senza rete. Spesso non c'è nemmeno un modo semplice per registrare insieme la propria voce e l'audio del computer, per esempio in una videochiamata, e poi trascriverli. L'utente vuole un'app Windows che faccia tutto sul proprio PC: aprire un file audio o video, oppure registrare, e ottenere il testo, senza account, senza chiavi e senza rete una volta scaricato il modello.

## Soluzione

Sbobino è un'app desktop solo Windows x64 (Tauri 2 + React).
- **Sorgente.** L'utente apre un file audio o video con "Sfoglia", oppure lo crea con una Registrazione da microfono, audio di sistema o entrambi. Il file diventa la Sorgente.
- **Trascrizione.** Trascrivi riconosce il parlato in locale con uno di tre modelli (Nemotron Streaming consigliato, Whisper Large v3 Turbo, Parakeet TDT v3). Il testo compare Frase per Frase in un'area dedicata: con Nemotron anche come Parziale, mentre la Frase è in corso. Alla fine il testo viene salvato in un TXT accanto alla Sorgente.
- **Impostazioni.** Restano salvate tra un avvio e l'altro. L'interfaccia è disponibile in sei lingue.

Nessun ffmpeg: la decodifica è in Rust (Symphonia), le Registrazioni sono in OGG/Opus scritte in Rust e "Estrai solo audio" non fa parte del prodotto (ADR-0002). La Trascrizione parte solo dopo Stop (ADR-0003).

## Storie utente

### Sorgente

1. Come utente, voglio un pulsante "Sfoglia" che apra il dialog di sistema, così scelgo un file senza digitare percorsi.
2. Come utente, voglio che il dialog proponga solo le estensioni accettate (MP3, WAV, M4A, FLAC, OGG, OPUS, WEBM, MPGA, MPEG, AIFF, MP4, MKV, MOV, M4V), così non scelgo file inutili.
3. Come utente, voglio vedere il nome della Sorgente nella finestra, così so su cosa sto lavorando.
4. Come utente, voglio cliccare il nome della Sorgente e aprirla con il programma associato, così la ascolto o la guardo prima di trascrivere.
5. Come utente, voglio vedere il percorso completo della Sorgente nella status bar, così so dove si trova.
6. Come utente, voglio che dopo la scelta di un file mi venga proposta l'azione Trascrivi, così so cosa posso fare.
7. Come utente, voglio un errore dedicato se il file contiene un codec audio non supportato (per esempio AC-3 dentro un MKV o un `.mpeg` che è un video MPEG-PS), così capisco che il problema è il formato e non l'app.
8. Come utente, voglio un errore dedicato se il file non esiste più o non è leggibile, così so che va scelto di nuovo.

### Trascrizione

9. Come utente, voglio premere Trascrivi su un file audio e vedere comparire il testo, così ottengo la sbobinatura.
10. Come utente, voglio trascrivere anche un video MP4, MOV, M4V o MKV senza prima estrarne l'audio, così risparmio un passaggio.
11. Come utente, voglio una Frase per riga nell'area di testo, così il testo si legge e si modifica facilmente.
12. Come utente con Nemotron, voglio vedere il Parziale della Frase in corso mentre viene riconosciuta, così seguo il lavoro in tempo reale.
13. Come utente con Whisper o Parakeet, voglio vedere ogni Frase appena è conclusa, così vedo comunque l'avanzamento.
14. Come utente, voglio la percentuale di avanzamento nella status bar quando la durata è nota, così so quanto manca.
15. Come utente, voglio un avanzamento senza percentuale quando la durata non è nota, così so comunque che l'app sta lavorando.
16. Come utente, voglio che a fine Trascrizione il testo venga salvato in `<nome Sorgente> trascrizione <N>.txt` accanto alla Sorgente, con N il primo numero libero, così non perdo il risultato e non sovrascrivo trascrizioni precedenti.
17. Come utente, voglio che la status bar mostri a fine Trascrizione che è finita, il numero di caratteri e il percorso del TXT, così so dove trovarlo.
18. Come utente, voglio un pulsante "Copia testo" che copi l'area negli appunti, così incollo il testo altrove.
19. Come utente, voglio una conferma prima che una nuova Trascrizione sostituisca il testo già presente nell'area, così non lo perdo per errore.
20. Come utente, voglio un pulsante "Annulla" durante la Trascrizione, così fermo un lavoro lungo avviato per sbaglio.
21. Come utente, voglio che dopo Annulla il testo già comparso resti nell'area ma che nessun TXT venga salvato, così il TXT esiste solo per Trascrizioni complete.
22. Come utente, voglio scegliere la Lingua del parlato con un selettore accanto a Trascrivi, così aiuto il modello quando il riconoscimento automatico sbaglia.
23. Come utente, voglio che il selettore della Lingua del parlato abbia "Automatica" come default e offra solo le lingue tra it, en, fr, es, de e pl supportate dal modello selezionato, così non scelgo combinazioni impossibili.
24. Come utente, voglio che la Lingua del parlato scelta resti salvata tra un avvio e l'altro, così non la reimposto ogni volta.
25. Come utente, voglio che la Trascrizione funzioni senza rete una volta scaricato il modello, così lavoro anche offline.
26. Come utente, voglio che senza un modello scaricato Trascrivi mostri un errore dedicato con un link a Impostazioni → Trascrizione, così so cosa fare.
27. Come utente, voglio che mentre una Trascrizione è in corso Registra e Sfoglia siano disabilitati, così non avvio due Attività insieme.
28. Come utente, voglio che una Trascrizione che fallisce mostri un errore dedicato nella status bar, così capisco cosa è successo.

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
51. Come utente, voglio che se un dispositivo si scollega durante la Registrazione questa si fermi come con Stop, salvando quanto registrato, e la status bar mostri un errore con il nome del dispositivo, così non perdo nulla.
52. Come utente, voglio che la Registrazione usi il bitrate, i canali e la frequenza delle Impostazioni, così controllo qualità e dimensione.
53. Come utente, voglio che durante una Registrazione Sfoglia e Trascrivi siano disabilitati, così non avvio due Attività insieme.

### Impostazioni

54. Come utente, voglio scegliere la sorgente di registrazione predefinita e i dispositivi, così non li reimposto ogni volta.
55. Come utente, voglio scegliere il bitrate tra 16, 24, 32, 48, 64, 96, 128, 192 e 320 kbps, così adatto la qualità.
56. Come utente, voglio scegliere mono o stereo e la frequenza tra 8 000, 16 000, 24 000 e 48 000 Hz, così adatto il file all'uso.
57. Come utente, voglio come predefiniti 32 kbps, mono, 48 kHz, così ho subito un buon compromesso per la voce.
58. Come utente, voglio scegliere la Cartella predefinita, che in mancanza è `Documenti\Sbobino` e viene creata se non esiste, così so dove finiscono le Registrazioni.
59. Come utente, voglio scegliere la Lingua dell'interfaccia tra it, en, fr, es, de e pl, con un avviso che si applica al riavvio, così uso l'app nella mia lingua.
60. Come utente al primo avvio, voglio l'interfaccia nella lingua del sistema se è tra le sei, altrimenti in inglese, così non devo cercare l'impostazione.
61. Come utente, voglio che tutte le impostazioni restino salvate tra un avvio e l'altro, così l'app riparte come l'ho lasciata.
62. Come utente, voglio che un file impostazioni corrotto non impedisca l'avvio ma riporti ai valori predefiniti, così l'app parte sempre.
63. Come utente, voglio una sezione Informazioni con la versione dell'app e le licenze dei componenti (modelli, Symphonia, ONNX Runtime, Silero, transcribe-cpp), così l'app rispetta le attribuzioni richieste.

### Finestra e aggiornamenti

64. Come utente, voglio che le sezioni visibili seguano l'Attività (Sorgente, Registrazione, Trascrizione), così la finestra mostra solo ciò che serve.
65. Come utente, voglio che la status bar mostri sempre la fase in corso, la percentuale quando c'è e gli errori con messaggi dedicati, così so sempre cosa succede.
66. Come utente, voglio che all'avvio, se c'è connessione, l'app controlli se esiste una versione più recente e mi proponga il link per scaricarla, così resto aggiornato.
67. Come utente offline, voglio che il controllo aggiornamenti fallisca in silenzio, così non vedo errori inutili.
68. Come utente, voglio che il tema chiaro o scuro segua quello di Windows, così l'app si integra con il sistema.

### Sviluppo e distribuzione

69. Come sviluppatore, voglio che `typecheck`, `test`, `check`, `format:backend`, `lint:backend` e `cargo test` passino a ogni milestone, così il progetto resta sano.
70. Come sviluppatore, voglio che i comandi e gli eventi Tauri siano tipizzati da un `bindings.ts` generato da Rust, così frontend e backend non divergono.
71. Come sviluppatore, voglio che `AGENTS.md` documenti comandi, prerequisiti di build, architettura e insidie, così un agente o un collega riparte senza chiedere.
72. Come utente finale, voglio un installer NSIS che includa tutto il necessario (DLL di runtime, modello Silero, testi delle licenze), così installo e uso l'app su qualsiasi PC Windows x64 recente.

## Stato

- **M1 (tracer bullet)**: storie 1, 2, 9, 11 e 13. Sfoglia sceglie un file audio, Trascrivi mostra le Frasi una per riga mentre arrivano. Il modello è sempre Nemotron e si mette a mano nella cartella dei modelli (`AGENTS.md`) finché non arriva il download (storie 29–39). Il nome della Sorgente compare già, non ancora cliccabile (storia 4).
- **M2, prima parte (ticket 03)**: storie 3–5, 7, 8, 10, 14–18, 28, 64 e 65.
  - Il nome della Sorgente si clicca per aprirla con il programma associato; il percorso sta nella status bar.
  - Si trascrivono anche i video MP4, MOV, M4V e MKV, senza file intermedi.
  - La status bar mostra la fase, la percentuale (nei MKV, che non dichiarano la durata, un avanzamento senza percentuale), l'esito con caratteri e percorso del TXT, e gli errori dedicati.
  - Il TXT si salva accanto alla Sorgente. "Copia testo" copia l'area.
  - La sezione Trascrizione compare con la prima Trascrizione.
  - Annulla, la conferma di sostituzione e una sola Attività alla volta (storie 19–21, 27) arrivano con il ticket 04. Storia 6: Trascrivi è già l'azione proposta, si abilita appena c'è una Sorgente.

## Fuori dal perimetro

- "Estrai solo audio" e qualsiasi conversione video. Niente ffmpeg (ADR-0002).
- I formati AVI, WMV, FLV, TS, MTS, MPEG-PS e i codec AC-3, E-AC-3, HE-AAC, WMA, DTS.
- La Trascrizione durante la Registrazione (ADR-0003) e la Trascrizione di più file in coda.
- Timestamp nel testo, diarizzazione, traduzione, prompt iniziale di Whisper e Lingue del parlato oltre le sei dell'interfaccia.
- Storico delle trascrizioni, editor avanzato del testo, esportazioni diverse dal TXT.
- Installazione automatica degli aggiornamenti (`tauri-plugin-updater`) e firma del codice.
- macOS, Linux, Windows ARM.
- La scelta della GPU: `transcribe-cpp` usa Vulkan se disponibile, altrimenti la CPU.
