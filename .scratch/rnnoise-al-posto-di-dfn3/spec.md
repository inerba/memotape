# RNNoise 0.2 al posto di DeepFilterNet3

Status: ready-for-agent

Data: 18 ottobre 2026. Decisioni concordate in una sessione di grilling dopo l'inventario dell'installer 0.5.3 e la prova d'ascolto alla cieca in `.scratch/prova-denoiser/esito.md`. Procede in parallelo a `.scratch/alleggerire-installer/spec.md` (VAD e ONNX Runtime). Sostituisce la scelta del modello dell'ADR-0019; il percorso PCM condiviso (ADR-0018), la pulizia per Ingresso (ADR-0021) e il riuso dei tratti puliti (ADR-0022) restano.

## Problem Statement

La pulizia dell'audio usa DeepFilterNet3 su tract: circa 11 MB dell'installer (8 MB di modello più circa 12 MiB di codice nell'exe) e una CPU di 0,085 secondi per secondo di audio. In più, sulla voce molto debole (il campione attenuato di 30 dB del corpus) DFN3 restituisce silenzio digitale: Silero non trova nessuna Frase e la voce dell'utente sparisce dal Tape.

## Solution

La pulizia passa a RNNoise 0.2 ufficiale di xiph, il modello che l'utente ha ascoltato alla cieca e giudicato quasi al livello di DFN3 sul campione del ristorante. Conserva la voce debole, elimina il rumore intermittente e il soffio, costa circa un sesto della CPU e circa 5 MB nell'installer invece di 11. Per l'utente la pulizia si accende e si spegne negli stessi punti e con gli stessi default; cambia il nome del modello nei testi e nei crediti. I Tape già puliti con DFN3 restano come sono.

## User Stories

1. Come utente, voglio un installer più leggero di circa 6 MB, così scaricare e aggiornare Memotape costa meno.
2. Come utente, voglio che la pulizia riduca il rumore di un ristorante con una qualità vicina a quella di oggi, così non perdo il beneficio che ho scelto.
3. Come utente, voglio che la pulizia non cancelli una voce molto debole, così le Frasi dette piano restano nel Tape e nella Trascrizione.
4. Come utente, voglio che il rumore intermittente e il soffio vengano eliminati, così Silero non apre false Frasi.
5. Come utente, voglio che «sì» e «no» brevi restino dopo la pulizia, così le risposte corte non spariscono.
6. Come utente, voglio accendere la pulizia nel dialog di Trascrivi come oggi, inizialmente spenta, così il flusso non cambia.
7. Come utente, voglio accendere la pulizia per Microfono e Audio di sistema in modo indipendente in una Registrazione, così posso pulire solo l'Ingresso rumoroso.
8. Come utente, voglio accendere o spegnere la pulizia durante una Registrazione, anche in Pausa, con effetto dall'audio successivo, così posso reagire al rumore.
9. Come utente, voglio che la Registrazione parta subito anche con la pulizia accesa, così non aspetto il caricamento di un modello.
10. Come utente, voglio che la pulizia funzioni anche senza Trascrivi dal vivo, così la Registrazione salvata è pulita comunque.
11. Come utente, voglio che il Tape conservi l'audio elaborato come oggi, così player, Forma d'onda e Parlanti si riferiscono a quello che ho sentito.
12. Come utente, voglio che un Tape già pulito con DFN3 non venga ripulito quando lo ritrascrivo, così non accumulo due puliture sullo stesso audio.
13. Come utente, voglio che la pulizia funzioni su ogni PC x64, anche con una CPU senza AVX2, così l'app non si chiude su una macchina vecchia.
14. Come utente, voglio che la pulizia di un file audio o video vada più veloce di prima, così Trascrivi finisce prima con la pulizia accesa.
15. Come utente, voglio un errore chiaro e tradotto se il modello della pulizia manca o è corrotto, così so di dover reinstallare.
16. Come utente, voglio che un guasto della pulizia durante una Registrazione lasci passare l'audio originale senza fermarla, come oggi, così non perdo la Registrazione.
17. Come utente, voglio che i testi della pulizia e i crediti in Informazioni nominino RNNoise e la sua licenza, così so cosa elabora il mio audio.
18. Come utente, voglio che la pulizia resti locale e senza download, così funziona offline dal primo avvio.
19. Come sviluppatore, voglio compilare RNNoise dai sorgenti nel repo, senza libclang e senza download durante la build, così la build resta riproducibile.
20. Come sviluppatore, voglio che il modello sia un file di pesi tra le risorse con hash verificato, così non versiono 29 MB di sorgente C e un file corrotto si riconosce.
21. Come sviluppatore, voglio che la versione esatta di RNNoise e del modello sia scritta nei tratti puliti del Tape, così si sa con cosa è stato elaborato un audio.
22. Come sviluppatore, voglio che tract, `df` e il modello DFN3 escano dal grafo e dal bundle, così l'exe e la build si alleggeriscono.
23. Come sviluppatore, voglio un ADR che sostituisca l'ADR-0019 con la prova d'ascolto e le misure, così un agente futuro sa perché DFN3 non c'è più.
24. Come sviluppatore, voglio che i test della pulizia di oggi (file, Tape, Registrazioni, dal vivo) girino con RNNoise, così la sostituzione è verificata sugli stessi casi.
25. Come sviluppatore, voglio che `PRODUCT.md` e i documenti di sviluppo descrivano RNNoise, così la fonte di verità resta allineata.

## Implementation Decisions

### Modello e build

- RNNoise ufficiale xiph, tag `v0.2` (commit `904a876dce1f9ab8860c0a5000ed151f9f6eef58`), licenza BSD-3-Clause. I sorgenti C della libreria (senza esempi, script di training e pesi) si copiano nel repo e si compilano con `cc` dallo script di build. Nessun crate di binding, nessun bindgen: le poche funzioni C si dichiarano a mano.
- Scelta SIMD a runtime come nella build ufficiale: versione generica più le varianti SSE4.1 e AVX2, scelte dal rilevamento della CPU di RNNoise. Il banco di prova usava AVX2 fisso, che non va in produzione.
- Modello normale `0b50c45` (non il «little»): è quello ascoltato. Dal tarball ufficiale (`rnnoise_data-0b50c45.tar.gz`, 22 270 507 byte, SHA-256 `4ac81c5c0884ec4bd5907026aaae16209b7b76cd9d7f71af582094a2f98f4b43`) si produce una volta il file binario dei pesi con lo strumento `write_weights` di RNNoise; il file va tra le risorse del bundle al posto del modello DFN3 e si carica con le funzioni di caricamento da file della 0.2. Dimensione e SHA-256 del file si fissano nel codice e si verificano prima del caricamento, con gli errori distinti di oggi per assenza e incompatibilità. La procedura per rigenerarlo va scritta nel documento di sviluppo.

### Il processore

- Il seam è `AudioProcessor`: l'adattatore RNNoise prende il posto di `DeepFilter` in `ConfiguredCleaning` e nella preparazione della Registrazione, senza cambiare il contratto.
- Lo stesso schema di oggi: ricampionamento interno a 48 kHz e ritorno al formato della Sorgente, stato indipendente per canale, frame da 480 campioni in scala int16, compensazione del ritardo, scarico completo della coda a ogni confine (Pausa, Configurazione, Finish), uscita diretta del modello senza miscela con l'originale.
- La storia di sola lettura all'inizio di un nuovo tratto (ADR-0022) scalda lo stato di RNNoise senza riconsegnare audio.
- Lo stato C di RNNoise non è legato al thread: l'adattatore può essere `Send` senza il thread dedicato che serve oggi a tract. La preparazione in anticipo della Registrazione resta se il caricamento dei pesi è misurabile; si semplifica se diventa istantaneo, mantenendo il comportamento (la Registrazione parte senza aspettare).
- La probabilità VAD che RNNoise restituisce non si usa: Silero resta il solo VAD.

### Tratti puliti e Tape

- I nuovi tratti hanno `algoritmo` «rnnoise» e una `versione` che identifica tag, commit, modello e uscita diretta. I tratti DFN3 dei Tape esistenti restano e si leggono; il riuso li protegge già qualunque sia la versione (ADR-0022), quindi un Tape già pulito non si ripulisce.

### Cosa esce

- `deep_filter`, i quattro crate tract fissati, `ndarray` se non serve ad altro, il modello DFN3 dalle risorse, le licenze di DFN3 e di tract; restano le dipendenze che altri usano.
- Le ottimizzazioni del profilo dev dedicate a DFN3 e tract (servono a non perdere blocchi WASAPI in debug) si tolgono o si spostano su RNNoise solo se la misura in debug lo richiede.

### Testi e documenti

- Messaggi di errore della pulizia, descrizioni della pulizia (file e Registrazione) e crediti nelle sei lingue: RNNoise al posto di DeepFilterNet3.
- `PRODUCT.md` (sezione Trascrizione e le storie della pulizia), `CONTEXT.md` se nomina DFN3, `AGENTS.md`, i documenti di sviluppo della trascrizione e della registrazione, la verifica dell'installer.
- Un ADR nuovo che sostituisce la scelta del modello dell'ADR-0019 (che diventa «superato in parte»): prova d'ascolto, misure di Silero e CPU, peso, versione e hash.

## Testing Decisions

- Un buon test guarda il PCM consegnato alle destinazioni: durata esatta, allineamento, campioni finiti, coda completa ai confini, stati separati per canale e Ingresso. Non guarda le chiamate C.
- Seam unico: `AudioProcessor`. I test che oggi usano il processore finto (`DelayedScale`) restano. I test con il modello vero che oggi caricano DFN3 passano a RNNoise senza cambiare cosa verificano: le prove della pulizia nella Trascrizione di file e Tape, nella Registrazione con due Ingressi, nella protezione dal vivo nativa e il corpus della protezione.
- Test dell'adattatore con il modello vero: durata identica all'ingresso a 16, 44,1 e 48 kHz e in stereo; ritardo compensato (un impulso esce allineato); scarico completo a Pausa, Configurazione e Finish; voce della fixture conservata e silenzio digitale invariato; errore distinto con pesi assenti o con hash sbagliato.
- Regressione della prova d'ascolto: sul campione del corpus con la voce attenuata di 30 dB, Silero trova ancora una Frase dopo la pulizia (DFN3 la perdeva).
- Un test che fissa la scelta SIMD non serve; la verifica manuale su una macchina senza AVX2 va in `docs/sviluppo/verifica-manuale.md` tra le prove su hardware reale.
- Verifica dell'installer: nessun file DFN3, la pulizia accesa su un file e in una Registrazione di prova, dimensioni riportate.

## Out of Scope

- Il modello «little» di RNNoise 0.2 o i modelli successivi di `main`.
- Usare la probabilità VAD di RNNoise al posto di Silero.
- Un controllo d'intensità della pulizia.
- Ripulire automaticamente i Tape puliti con DFN3.
- Il VAD e ONNX Runtime (spec `alleggerire-installer`).

## Further Notes

- Prova e misure: `.scratch/prova-denoiser/esito.md`. CPU sul ristorante (Ryzen 7 3700X, release): DFN3 0,083–0,088 RTF, RNNoise 0.2 0,015 con AVX2. Libreria statica di RNNoise 0.2 con il modello normale: 5,7 MB, circa 5 MB compressi.
- Il banco compilava RNNoise 0.2 con MSVC e `cc` senza modifiche ai sorgenti, a partire dall'elenco dei sorgenti del `Makefile.am` del tag.
