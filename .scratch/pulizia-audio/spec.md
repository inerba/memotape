# DeepFilterNet3 e protezione del parlato per Ingresso

Status: ready-for-agent

Data: 6 ottobre 2026. Specifica pubblicata con `to-spec`; punti di verifica
confermati dall'utente. Descrive requisiti da implementare, non funzionalità
già disponibili. Non avvia automaticamente l'implementazione né autorizza
commit o rilascio.

## Problem Statement

Durante una Registrazione da Entrambi, il Microfono può produrre Frasi anche
quando l'utente non parla: Whisper inventa, per esempio, «grazie» e Parakeet
produce testo inglese. Respiri e rumori possono aprire una Frase che il modello
trascrive come parlato. Il problema deve essere affrontato anche nella
Trascrizione di file e Tape, preservando voce bassa e risposte brevi.

L'utente vuole inoltre migliorare il rumore dell'audio che ascolta e conserva.
Un filtro applicato soltanto alla Trascrizione non soddisfa il requisito.
Silero è già presente, ma la sua presenza non garantisce l'assenza di false
Frasi e un denoiser non sostituisce il riconoscimento del parlato.

Il problema precedente dei gruppi di testo troppo grandi è stato affrontato
nella vista delle Frasi; questa spec non cambia i confini del testo per motivi
di impaginazione e non introduce nuove regole di aggregazione.

## Solution

Integrare DeepFilterNet3 in locale, nel backend Rust, come pulizia dell'audio
indipendente per Ingresso. È spenta per default. Quando attiva, lo stesso audio
ripulito viene salvato, riprodotto e usato da Silero, Trascrizione e analisi
finale dei Parlanti. Nel Tape non si conserva una seconda copia originale.

Separatamente, aggiungere una protezione prudente contro le false Frasi con
quattro livelli di sensibilità del parlato: **Spento**, **Più sensibile**,
**Bilanciato** e **Più selettivo**. Bilanciato è il predefinito. Spento disattiva
solo la protezione aggiuntiva, mantenendo Silero e il suo comportamento attuale.
Non spegne DeepFilterNet3: i due controlli sono indipendenti.

Microfono, Audio di sistema e File e audio misto hanno profili distinti.
Impostazioni e barra della Registrazione condividono i valori persistenti.
I cambi durante un'Attività valgono dall'audio successivo, senza ritoccare
retroattivamente testo o audio già elaborati.

## User Stories

1. Come utente, voglio pulire il rumore del Microfono con DeepFilterNet3, così ascolto e conservo una voce più chiara.
2. Come utente, voglio attivare la pulizia separatamente sull'Audio di sistema, così posso lasciare intatto un audio già filtrato dalla chiamata.
3. Come utente, voglio trovare la pulizia spenta al primo avvio, così scelgo su quali Ingressi usarla.
4. Come utente, voglio che la pulizia funzioni interamente sul mio PC, così nessun audio viene inviato a servizi esterni.
5. Come utente, voglio attivare la pulizia senza installare Python o configurare un servizio, così il controllo è utilizzabile nell'app.
6. Come utente, voglio ascoltare nel player lo stesso audio ripulito usato per la Trascrizione, così posso verificare le Frasi su ciò che è stato salvato.
7. Come utente, voglio conservare soltanto l'audio elaborato quando la pulizia è attiva, così il Tape non contiene una copia originale aggiuntiva.
8. Come utente, voglio sapere che spegnere la pulizia non recupera l'audio originale già elaborato, così comprendo il significato del controllo.
9. Come utente, voglio che Microfono e Audio di sistema restino distinti nel Tape e nel testo, così la pulizia di uno non altera l'altro.
10. Come utente, voglio pulire una Registrazione anche senza Trascrivi dal vivo, così la qualità dell'audio non dipende dal riconoscimento del testo.
11. Come utente, voglio usare la pulizia durante la Trascrizione di file audio e video, così anche il Tape importato contiene il risultato ripulito.
12. Come utente, voglio un profilo File e audio misto per file e Tape senza Ingressi separati, così non vengono attribuite loro impostazioni di Microfono o Audio di sistema inesistenti.
13. Come utente, voglio applicare i profili dei rispettivi Ingressi quando trascrivo un Tape separato, così mantengo le mie preferenze su ciascuna traccia.
14. Come utente, voglio evitare che un Tape già ripulito venga filtrato di nuovo a ogni Trascrizione, così non degrado progressivamente la voce.
15. Come utente, voglio una protezione automatica contro le false Frasi, così silenzio, respiri e rumori non vengono trascritti abitualmente come parole.
16. Come utente, voglio preservare risposte come «sì» e «no», così il filtro non elimina parole corrette solo perché brevi.
17. Come utente, voglio preservare il parlato a bassa voce, così non devo alzare la voce per farmi trascrivere.
18. Come utente, voglio una sensibilità distinta per ciascun Ingresso, così posso rendere il Microfono più selettivo senza perdere il parlato dell'Audio di sistema.
19. Come utente, voglio quattro livelli con spiegazioni comprensibili, così scelgo il compromesso fra voce debole e false attivazioni.
20. Come utente, voglio Bilanciato come sensibilità predefinita, così parto da una protezione prudente senza doverla regolare.
21. Come utente, voglio che Spento mantenga Silero e disattivi soltanto la protezione aggiuntiva, così torno al comportamento precedente della selezione del parlato.
22. Come utente, voglio regolare pulizia e sensibilità separatamente, così posso usare uno dei due strumenti senza obbligatoriamente attivare l'altro.
23. Come utente, voglio cambiare i controlli di ciascun Ingresso dalla barra della Registrazione, così correggo il comportamento senza fermare la conversazione.
24. Come utente, voglio che un cambio valga sull'audio successivo anche durante una Registrazione, così non devo aspettare una nuova sessione.
25. Come utente, voglio che barra e Impostazioni mostrino lo stesso valore e lo conservino al riavvio, così non devo riconfigurarlo ogni volta.
26. Come utente, voglio che i cambi fatti in Pausa valgano alla ripresa, così posso preparare i controlli senza produrre rumori nel Tape.
27. Come utente, voglio mantenere durata, tempi delle Frasi e sincronizzazione fra Ingressi, così il filtro non sposta il testo rispetto al player.
28. Come utente, voglio conservare anche le ultime parole prima di Pausa e Stop, così i buffer del filtro non tagliano la fine della conversazione.
29. Come utente, voglio un avviso comprensibile se la pulizia non può proseguire, così non credo di salvare audio ripulito quando il filtro si è interrotto.
30. Come utente, voglio che un guasto della pulizia non faccia perdere la Registrazione, così conservo audio e testo già acquisiti.
31. Come utente, voglio che un file o Tape non venga sostituito parzialmente se annullo la Trascrizione, così mantengo una Sorgente utilizzabile.
32. Come utente, voglio continuare ad aprire i Tape precedenti, così l'integrazione non richiede una migrazione della Libreria.
33. Come utente, voglio controlli accessibili e tradotti nelle lingue dell'app, così posso usarli anche con tastiera e lettore di schermo.
34. Come utente, voglio che la Registrazione da Entrambi tenga il passo con la Trascrizione, così il miglioramento dell'audio non introduce interruzioni o code crescenti.

## Implementation Decisions

### Runtime e distribuzione

- Il candidato scelto è **DeepFilterNet3 standard**, non DFN2 né la variante low latency. La prima integrazione usa libDF 0.5.6, revisione `978576aa8400552a4ce9730838c635aa30db5e61`, con Tract 0.19.16 fissato e feature di inferenza necessarie. Nessun aggiornamento automatico delle dipendenze o dei pesi durante l'uso.
- Usare il runtime Rust originale verificato, senza Python, PyTorch, processo esterno o chiamate a Hugging Face. ONNX Runtime usato da Silero resta distinto da Tract.
- Il modello standard verificato pesa 7 983 136 byte e ha SHA-256 `c94d91f70911001c946e0fabb4aa9adc37045f45a03b56008cb0c8244cb63616`. Origine, hash, dimensione e runtime vanno fissati insieme e verificati prima dell'adozione. Il peso del modello non è una stima di RAM o installer.
- Distribuire i pesi con l'app, rendendo il controllo utilizzabile senza un nuovo download manuale. Verificare licenze e attribuzioni sia del codice sia dei pesi; includere quelle richieste nel bundle e in Informazioni. Un artefatto assente o incompatibile produce un errore esplicito, senza sostituzione con un altro modello.
- Verificare il grafo completo delle dipendenze di Memotape e la build MSVC. La prova separata non attesta già la compatibilità dell'intera app o l'installer.

### Un solo componente audio, stato per Ingresso

- Aggiungere al core audio un componente che riceve blocchi PCM e produce blocchi elaborati, gestendo ricampionamento, buffer, cambi di configurazione e chiusura. Registrazione e Trascrizione di file usano lo stesso contratto. Il core resta indipendente da Tauri.
- La pulizia lavora internamente a 48 kHz, a hop di 480 campioni, mantenendo lo stato tra i blocchi. La frequenza di salvataggio e i canali scelti dall'utente restano quelli della Registrazione: non vengono cambiati per adattarsi al modello.
- Ogni Ingresso ha uno stato indipendente. L'Audio di sistema stereo non diventa involontariamente mono; la gestione multicanale deve conservare formato e allineamento. Il profilo File e audio misto non tenta di ricostruire Ingressi già sommati.
- Nella Registrazione applicare la pulizia nel worker, prima della somma degli Ingressi e prima della diramazione verso Ogg e Trascrizione. La callback WASAPI resta priva di inferenza e attese.
- Per i file applicare lo stadio comune dopo la decodifica e prima della diramazione verso copia audio e ASR. La Forma d'onda si ricava dall'audio effettivamente salvato.
- Il Guadagno continua a valere per audio salvato e Trascrizione. L'ordine con la pulizia deve essere esplicito e verificato: conversione/Guadagno dell'Ingresso, pulizia, somma e destinazioni. Non aggiungere AGC o normalizzazione automatica come effetto collaterale.
- La Diarizzazione delle Registrazioni resta soltanto dopo Stop e completamento ASR, sull'audio salvato. Nessun nuovo modello dei Parlanti durante la cattura.

### Tempi, cambi al volo ed errori

- Compensare il ritardo effettivo della catena preservando i campioni utili, l'origine dei tempi e la durata. La latenza non coincide con l'hop da 10 ms. Anche il percorso senza pulizia deve restare allineato quando l'altro Ingresso è filtrato o quando il controllo cambia.
- Gestire buchi di silenzio, pausa e riavvio senza contaminazione fra sessioni. A Pausa e Stop conservare l'audio precedente e scaricare la coda utile. Verificare la scorciatoia del runtime sui frame a energia molto bassa: l'invio di zeri non è da solo una prova dello scarico dei buffer.
- La configurazione viene applicata all'audio successivo nello stadio di acquisizione/decodifica, prima delle code ASR; non al momento in cui il motore consuma una Frase già accodata. Le revisioni della configurazione non ritrattano Parziali o Frasi già pubblicati.
- I cambi di pulizia devono avere continuità di tempi e una transizione senza scatti evidenti. La disattivazione non elimina la coda del segmento precedente. I cambi di sensibilità influenzano la nuova evidenza VAD e non troncano parole già riconosciute.
- Impostazioni e Registrazione usano lo stesso stato persistente, con aggiornamento coerente anche a cavallo dell'avvio. In Pausa i cambi valgono dalla ripresa; dopo Stop non modificano il Tape appena acquisito.
- Se il filtro si guasta durante una Registrazione, preservare l'audio acquisito, proseguire con bypass per il solo Ingresso interessato e mostrare l'avviso nella sessione corretta. Il bypass deve mantenere la linea del tempo; non perdere campioni, introdurre code illimitate o bloccare la callback. Il Tape registra dove la pulizia è stata effettivamente applicata.
- Nei file, errore o Annulla impediscono la sostituzione finale del Tape. Una pulizia richiesta che non riesce non viene presentata come riuscita.

### Impostazioni e protezione del parlato

- Tre profili persistenti: Microfono, Audio di sistema, File e audio misto. Ogni profilo contiene attivazione della pulizia (default falso) e sensibilità del parlato (default Bilanciato). I dati precedenti ricevono questi default senza perdere gli altri valori.
- L'intensità interna della pulizia parte da una taratura prudente. Limite di attenuazione, soglie SNR e post-filter sono strumenti di calibrazione, non la traduzione dei quattro livelli di sensibilità. Non aggiungere un altro selettore di intensità senza un requisito distinto.
- Spento usa Silero e i parametri attuali della selezione del parlato, disabilitando soltanto le nuove decisioni di scarto. Quando la pulizia è attiva, Silero riceve comunque l'audio elaborato.
- La protezione aggiuntiva combina evidenza del parlato di Silero e caratteristiche della Frase; eventuale confidenza ASR è utilizzabile solo quando il runtime la espone con un significato verificabile. Una soglia energetica assoluta, una durata minima rigida o una lista di parole vietate non possono essere l'unico criterio di scarto.
- Nessuna rimozione di «grazie», parole inglesi o altre espressioni soltanto per il loro testo. Le parole reali brevi o deboli del corpus di regressione devono essere preservate nei livelli prudenti.
- Calibrare i parametri con campioni etichettati e fissare valori riproducibili per i tre livelli attivi. Più sensibile privilegia la voce debole, Bilanciato riduce le false attivazioni prudentemente, Più selettivo dichiara il possibile costo sulle parole brevi/deboli. Non promettere zero allucinazioni su qualsiasi audio.
- La sensibilità vale con DeepFilterNet3 acceso o spento e con tutti i motori ASR disponibili. Si applica dal vivo, nei file e nei Tape secondo l'Ingresso effettivo.

### Tape e interfaccia

- Il Tape mantiene il formato compatibile con i documenti precedenti e aggiunge metadati facoltativi della pulizia, per Ingresso e intervalli: algoritmo/versione e tratti effettivamente elaborati. Servono a descrivere cambi al volo e bypass per guasto e a evitare nuove passate di pulizia sul medesimo audio.
- In una Registrazione o importazione, si conserva solo il risultato del percorso scelto; nessuna traccia originale aggiuntiva, anche quando la pulizia viene accesa a metà. I tratti con pulizia spenta restano non trattati.
- Trascrivi su un Tape già elaborato riusa i tratti puliti senza ripulirli. Se la pulizia è attiva su tratti non trattati, produce audio, mix, Forma d'onda e testo coerenti e li sostituisce atomicamente al termine; errore o Annulla conservano il Tape precedente. Spegnere il controllo non annulla un trattamento già salvato.
- Aprire un Tape, cambiare le Impostazioni o riprodurlo non avvia elaborazione né riscrittura automatica. I Tape vecchi restano leggibili e non vengono migrati in apertura. L'assenza dei nuovi metadati non prova che un audio esterno non sia mai stato filtrato.
- In Impostazioni mostrare i controlli per profilo con spiegazioni brevi. Nella barra della Registrazione mostrare pulizia e sensibilità accanto ai controlli del relativo Ingresso, mantenendoli distinti dal Guadagno.
- Un cambio dalla barra aggiorna Impostazioni e viceversa. Controlli, stato corrente e avvisi devono essere accessibili con tastiera e lettore di schermo e tradotti nelle sei lingue dell'app.
- Aggiornare i contratti generati tramite il generatore dell'app; mantenere tutti gli accessi frontend ai comandi/eventi applicativi. Aggiornare i requisiti di prodotto e le decisioni architetturali durante l'implementazione, distinguendo ciò che è implementato da ciò che è soltanto misurato in una prova.

## Testing Decisions

I test devono verificare audio, durata, testo, persistenza ed errori osservabili.
Non verificare nomi di funzioni interne, numero di hop o il dettaglio delle
formule come sostituti del comportamento finale.

- **Punto principale nel core:** verificare dal blocco audio fino alle destinazioni audio e Trascrizione. Riutilizzare le seam esistenti `TranscriptionEngine` e `VoiceDetector`, i test del mixer e i test di creazione/riapertura dei Tape. Introdurre al massimo una nuova seam del processore audio, abbastanza alta da simulare un'elaborazione e un ritardo riconoscibili; non una seam per ciascuna operazione DSP.
- **Coerenza del percorso:** usare un processore deterministico nei test di orchestrazione per dimostrare che tracce, mix, copia audio, Forma d'onda e audio consegnato all'ASR derivano dallo stesso risultato. Confrontare PCM prima della compressione e usare tolleranze adatte a Opus dopo la riapertura, senza richiedere PCM bit-identico a un codec lossy.
- **Tempi e ciclo di vita:** ingressi a frequenze diverse, mono/stereo, loopback fermo, buchi, avvio e cambio al volo, Pausa/Riprendi, Stop durante parlato, impulsi e parole ai confini, più sessioni. Verificare durata e allineamento con la precisione già richiesta dai test dell'app; nessun taglio sistematico della coda del filtro.
- **Protezione del parlato:** prove deterministiche con le seam esistenti per livelli e cambi di configurazione; nessuna nuova Frase su silenzio digitale, nessuno scarto delle Frasi valide di regressione nei livelli prudenti, modalità Spento equivalente alla selezione Silero precedente quando la pulizia è spenta.
- **Runtime reale:** smoke con pesi DFN3 verificati e campioni di silenzio, respiri, rumore continuo/intermittente, risposte brevi e voce bassa italiana. Il corpus separa parlato e non parlato e conserva gli originali soltanto nel banco di prova. Confrontare falsi avvii e Frasi, parole corrette perse e qualità d'ascolto con il comportamento precedente, con Whisper, Parakeet e Nemotron. I dati di prova non diventano una copia originale obbligatoria nei Tape degli utenti.
- **Taratura:** fissare un corpus annotato e riportare i risultati per Ingresso e livello. Bilanciato deve ridurre le false Frasi del corpus senza introdurre perdite nelle risposte brevi e nella voce debole designate come regressioni. Non sostituire queste verifiche con punteggi percettivi pubblicati o con il solo output finito della prova tecnica.
- **Carico nativo:** misurare in release due Ingressi, pulizia attiva, ASR dal vivo e salvataggio, con intervalli di parlato e silenzio. Riportare tempo per secondo di audio, memoria e andamento delle code; il tratto stabile deve tenere il passo senza accumulo crescente, perdita di buffer o disallineamenti. La Diarizzazione parte dopo Stop. Qualificare durata della prova e hardware, senza estendere i risultati ad altri PC.
- **Tape:** riaprire registrazioni, importazioni, Tape misti/separati e vecchi Tape; verificare metadati per intervallo, assenza della seconda copia audio, nessuna doppia pulizia, riscrittura atomica e conservazione del precedente con Annulla, errore o disco pieno. Verificare player, Frasi, ricerca, copia e analisi finale dei Parlanti dopo una pulizia di audio esistente.
- **Impostazioni e UI:** riutilizzare i test dei profili e delle azioni del frontend per default, validazione, persistenza, aggiornamento dalla barra, eventi di sessioni precedenti e accessibilità. Aggiungere un collaudo nativo dei cambi durante cattura/Pausa: i test puri del frontend non lo sostituiscono.
- **Controlli di chiusura:** typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust tutti verdi. Una sola build Rust alla volta. Gli smoke con motori ASR reali vanno eseguiti separatamente secondo i vincoli esistenti. Registrare anche gli esiti manuali mancanti, senza considerarli passati.

I tre gruppi di verifica sono stati confermati dall'utente il 6 ottobre 2026:
percorso audio/Tape e tempi; DFN3 reale e carico con due Ingressi/ASR; controlli
persistenti e modifiche al volo.

## Out of Scope

- RNNoise, DeepFilterNet2, selettore tra denoiser e variante DFN3 low latency.
- Addestramento o modifica dei pesi, GPU per il denoiser e servizi cloud.
- Cancellazione dell'eco con riferimento dell'Audio di sistema, separazione delle voci sovrapposte, dereverberazione dedicata, AGC e normalizzazione automatica.
- Ripristino dell'audio originale eliminato dalla pulizia, archivio di originali nei Tape e migrazione automatica della Libreria.
- Pulizia applicata solo al player o solo a una copia ASR.
- Revisione della segmentazione linguistica, dell'aggregazione delle Frasi o della Diarizzazione dal vivo.
- Continuazioni, identità fra sessioni e nuove azioni autonome di pulizia senza Trascrivi.
- Commit, pubblicazione di installer e rilascio: non sono autorizzati da questa spec.

## Further Notes

La prova tecnica precedente ha caricato DFN3 con il runtime originale su
Windows/MSVC e processato una fixture parlata pulita di 8,960 s in 3,175 s,
RTF 0,354 in debug, su un solo Ingresso. Conservazione della durata e output
finito sono stati verificati. Sono esclusi caricamento, ricampionamento,
cattura, salvataggio e ASR dalla misura di elaborazione. Non è una verifica
di qualità su voce bassa, respiri o conversazioni reali, né un benchmark dei
due Ingressi dell'app.

La scelta di DFN3 sostituisce la scelta provvisoria di RNNoise nella
conversazione. Restano valide tutte le decisioni di comportamento già
concordate; la qualità percepita nella demo DFN2 non viene presentata come
una prova della medesima qualità del runtime DFN3.

Fonti tecniche: [runtime libDF 0.5.6](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs),
[modello DFN3 ufficiale](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/models/DeepFilterNet3_onnx.tar.gz),
[paper DFN3](https://arxiv.org/abs/2305.08227).
Il repository conserva separatamente il resoconto della prova Rust e il
confronto fra i candidati. Questa spec non trasforma PESQ, STOI o MOS in una
garanzia sulle allucinazioni dei modelli ASR.
