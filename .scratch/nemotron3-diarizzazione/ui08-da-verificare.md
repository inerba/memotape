# Collaudo UI nativo da completare da una persona — ticket 08

Usare la build del worktree `5e50`, senza installer o pubblicazione. Conservare
prima i valori delle Impostazioni e ripristinarli dopo la prova. Usare una Raccolta
di prova e registrazioni autorizzate. Annotare build, PC, data, modello, backend,
Ingressi e lingua; registrare uno screen video locale a 60 fps con audio, mai inviato
ad un servizio remoto. Questo elenco è una procedura: nessun passo è già attestato.

1. Aprire Impostazioni → Trascrizione: Sortformer è predefinito per configurazioni
   precedenti, Nemotron 3 sperimentale e percorso GGUF leggibile. Scegliere Nemotron
   ASR, Trascrivi dal vivo e Riconosci i parlanti. Ripetere da Microfono, Sistema,
   Entrambi con casella Microfono spenta e accesa.
2. Registrare conversazione italiana annotata da 2–4 voci, alternanze rapide,
   silenzi/rumore/overlap; ripetere separatamente da 5–8. Segnare nell'audio gli
   intervalli delle voci prima di valutare il modello. Osservare testo ed etichette
   provvisorie, rettifiche e divisioni/riunioni senza duplicazioni o parole perse;
   overlap/tempi insufficienti devono restare non determinati.
3. Dal video misurare per le stesse unità: fine del parlato → testo renderizzato,
   fine del parlato → prima etichetta determinata renderizzata, prima comparsa del
   testo → etichetta. Riportare campioni senza etichetta, p50/p95/p99/max e ritardo
   crescente; distinguerli dalle misure callback del banco. Non selezionare solo
   il primo caso rapido. Valutare obiettivo 1–2 s soltanto con queste misure.
4. Pausa 10 s/Riprendi: audio/timer escludono la pausa, sessione/identità restano.
   Da Entrambi lo stesso numero sulle due tracce non indica identità comune.
   Stop mostra smaltimento e Analisi finale dei parlanti; Copia testo/Copia turno
   durante il completamento riflettono lo snapshot visto. Rinomina/correzione
   sono disabilitate finché finisce. Le etichette finali possono cambiare, l'ASR
   non riparte.
5. Ripetere Annulla nell'analisi finale (anche mentre il secondo Ingresso lavora):
   Tape con audio e testo, esito non completato e provvisorietà riconoscibile;
   successo del primo Ingresso conservato. Guasto e saturazione isolati devono
   mostrare avviso che nomina l'Ingresso, mentre altra ASR/audio proseguono.
   Il banco copre un ritardo artificiale; riprodurre in UI richiede una build di
   prova con fault esplicito o un carico locale osservabile, senza alterare i pesi.
6. Riaprire Tape finale e annullato, ascoltare Player e Riascolta con salti/tempi;
   verificare testo, Ingressi, nomi, esiti, intervalli, durata. Correggere una Frase,
   rinominare, Copia testo/Copia turno, Esporta Markdown e riaprire: stesso contenuto
   visibile/coperto, provvisorio e non determinato distinti nelle uscite.
7. Aprire un Tape v1 precedente senza nuovi campi: stessi nomi/testo/audio,
   nessuna riscrittura o separazione inventata del mix. Ripetere avvisi, copie,
   esportazioni, layout e riapertura in it/en/fr/es/de/pl (core già coperto dai
   controlli automatici non significa UI di queste sei lingue già verificata).
8. Registrare una sessione almeno 60 min su CPU e Vulkan, con uno e due Ingressi,
   senza audio perso o crescita incontrollata delle code; monitorare CPU/memoria,
   durata e smaltimento. Conservare log e hash dei Tape. I cinque minuti del banco
   sono stress prolungato, non certificano Registrazioni di un'ora.

Esito per passo: passato/fallito/non eseguito, evidenza locale e problema preciso.
Ripristinare Impostazioni annotate; Sortformer resta disponibile e non viene
promosso automaticamente alcun modello. Nessun commit o pubblicazione.