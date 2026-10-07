# Avvio della Registrazione

Data: 7 ottobre 2026. Decisioni di prodotto confermate con `grill-with-docs`.
Implementazione autorizzata dall'utente con `$implement` e svolta nel checkout locale.

## Decisioni confermate

- L'audio deve partire appena possibile. La Trascrizione dal vivo può mostrare
  il testo dopo, conservando tutto l'audio dall'avvio confermato.
- L'attesa e la conferma «Registrazione avviata, puoi parlare» compaiono nella
  barra della Registrazione, al posto dei controlli durante la preparazione.
- Durante la preparazione è disponibile Annulla: torna alla schermata precedente
  senza creare un Tape. Dall'avvio effettivo il comando è Stop e salva l'audio.
- La conferma è visiva, insieme all'avvio del timer; nessun segnale sonoro.
- I componenti della Pulizia audio vengono preparati in background e conservati
  per il riuso mentre l'app è aperta. L'utente accetta il maggiore uso di memoria;
  costo e risparmio devono essere misurati. Ogni sessione usa stato audio nuovo.
- Se la pulizia è richiesta all'avvio, la Registrazione attende che sia pronta
  affinché sia applicata dall'inizio. Se è spenta, la sua preparazione non deve
  ritardare l'avvio dell'audio.
- Se la pulizia viene accesa durante la Registrazione mentre il filtro è ancora
  in preparazione, l'audio continua senza interruzioni. La barra mostra
  «Preparazione pulizia…» e il filtro si attiva appena pronto, solo sull'audio
  successivo. La richiesta non viene mostrata come trattamento già attivo.
- Se pulizia o Trascrizione dal vivo falliscono, l'audio continua a essere
  registrato con un avviso esplicito. Un errore di dispositivo o l'impossibilità
  di scrivere l'audio impediscono l'avvio. Restano le regole esistenti di
  arresto e recupero dell'audio se il guasto arriva a Registrazione iniziata.

## Comportamento della barra

La Preparazione della Registrazione inizia al clic, comprende l'attesa dei
salvataggi delle Impostazioni e impedisce doppi avvii e Attività incompatibili.
Mostra un indicatore indeterminato, Annulla e un messaggio relativo alla fase
effettiva: «Preparazione registrazione…», «Salvataggio impostazioni…»,
«Preparazione pulizia audio…» oppure «Apertura dispositivi audio…».
Le fasi già pronte si saltano, senza sequenze o percentuali simulate.

«Registrazione avviata, puoi parlare» appare solo quando il backend conferma
che tutti gli Ingressi richiesti e il percorso di scrittura sono pronti.
Compaiono timer, livelli e controlli della Registrazione. La conferma non
dipende dal primo testo, da una soglia di volume o da un conto alla rovescia.
Anche Audio di sistema silenzioso deve poter partire. L'indicazione di
Registrazione attiva resta riconoscibile dopo la conferma iniziale.

La preparazione ASR non tiene aperto il preloader quando l'audio è pronto.
Un'eventuale indicazione «Preparazione trascrizione…» resta secondaria alla
conferma che l'audio è in Registrazione. Messaggi e annunci accessibili
seguono i cambi di fase, non ogni tick del timer, nelle sei lingue dell'app.

Annulla ripristina la vista precedente senza perdere testo o Correzioni manuali
e senza creare Tape. Se l'avvio ha già vinto la corsa nel backend, l'azione
diventa Stop e conserva l'audio; non elimina una Registrazione già iniziata.
Una preparazione annullata non può far partire dispositivi in ritardo.

## Preparazione e riuso

La preparazione in background riguarda i componenti della Pulizia audio per
gli Ingressi configurati; non apre microfoni o loopback prima di Registra.
Si conserva il lavoro costoso riutilizzabile, con stato audio indipendente
per sessione e Ingresso e invalidazione delle risorse incompatibili dopo
cambi di configurazione. Non si accumula una cache senza limite di formati.

Se si spegne la pulizia prima che sia pronta, il completamento tardivo non
deve attivarla. Stop o Annulla rendono inapplicabile ogni attivazione pendente
per quella sessione. In Pausa l'attivazione vale sull'audio dalla ripresa.
Un guasto lascia l'Ingresso in bypass e lo segnala, come previsto dall'app.

Il riuso delle istanze ASR aggiuntive e lo spostamento della verifica del
diarizer sono possibili ottimizzazioni da valutare con le misure, non scelte
già obbligatorie. La Diarizzazione resta dopo Stop; l'integrità dei modelli
e l'audio salvato non vengono sacrificati per abbreviare l'attesa.

## Evidenze prima dell'implementazione

- `src/app/routes/home.tsx`, funzione `record`: attende `flush`, poi imposta
  `phase: "recording"` prima di invocare `commands.record`. L'interfaccia non
  distingue ancora la richiesta di avvio dall'effettiva Registrazione.
- ADR-0021 prevede la preparazione dei runtime di Pulizia audio prima di aprire
  i dispositivi, anche con pulizia spenta, per permettere l'accensione durante
  la Registrazione senza caricare e ottimizzare il modello in quel momento.
  Ridurre questa attesa richiede di riesaminare esplicitamente tale compromesso.
- Questi fatti non quantificano ancora il ritardo segnalato dall'utente.

La diagnosi dettagliata e le misure locali preesistenti sono in `diagnosi.md`;
il piano di misura e collaudo è in `verifica.md`. Nessuna promessa di latenza
assoluta o risparmio di memoria precede queste verifiche.

## Documenti e stato

Requisiti in `PRODUCT.md`, termine in `CONTEXT.md`, compromesso architetturale
in ADR-0026. Le copie di PRODUCT.md e CONTEXT.md precedenti a questo aggiornamento
sono in `documenti-prima/`; `documenti.diff` mostra soltanto queste aggiunte.
Implementazione e prove automatiche sono riportate in `verifica.md` e nel
ticket `issues/01-avvio-e-filtri-pronti.md`. Le prove native di cattura e
parlato reale restano esplicitamente da collaudare.
