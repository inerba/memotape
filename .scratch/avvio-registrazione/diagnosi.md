# Attesa prima della Registrazione

Indagine in sola lettura del 7 ottobre 2026 sul checkout principale. Nessuna
nuova build o Registrazione eseguita; i tempi sotto provengono da prove locali
già presenti, non dalla riproduzione del ritardo riferito in questa sessione.

## Risultati

| Passaggio | Evidenza attuale | Implicazione |
| --- | --- | --- |
| Interfaccia | `src/app/routes/home.tsx:647` attende `flush`; a riga 667 imposta `recording` prima di `commands.record` a riga 670. | L'attesa dei salvataggi non ha una fase propria; poi la UI anticipa il reale avvio dell'audio. |
| Prenotazione dei Parlanti | `src-tauri/src/managers/recording.rs:193-234` attende la prenotazione; Nemotron locale passa dalla verifica di dimensione e SHA-256 in `engine/local_diarizer.rs:14-39`. | Una lettura completa del modello locale da 198.937.280 byte può precedere l'audio. Costo non misurato qui; non è l'analisi dei Parlanti, che resta dopo Stop. |
| Trascrizione dal vivo | `recording.rs:235-253` avvia ASR in un thread parallelo; `transcription.rs:919-940` prepara le istanze necessarie, incluse quelle aggiuntive non conservate fra sessioni. | Il caricamento ASR ritarda il testo e può contendere risorse, ma non è un'attesa sequenziale obbligatoria prima dell'audio. |
| Pulizia audio | `recording.rs:671-709` prepara un runtime per Ingresso, in sequenza; `cleaning.rs:115-123` lo prepara anche da spento; `deepfilter.rs:256-323` attende l'inizializzazione del worker. | Lavoro costoso ripetuto sul percorso critico di ogni avvio. Due Ingressi comportano due preparazioni. |
| Dispositivi e scrittura | `recording.rs:710-785` apre le catture, prepara mixer e writer Ogg; i tick arrivano dal ciclo successivo. | La conferma dell'avvio va ancorata alla disponibilità effettiva dell'intero percorso audio, non all'invio del comando. |

## Tempi già disponibili

Il file `.scratch/diagnosi-registrazione/dfn3-performance.log` riporta:

- DFN3 con worker, 48 kHz, un Ingresso mono: inizializzazione **1,1312 s**;
- DFN3 con worker, 48 kHz, due Ingressi stereo: inizializzazione **2,3572 s**;
- DFN3 con worker, 16 kHz, due Ingressi mono: inizializzazione **2,5183 s**.

Sono misure del componente nel test profile locale. Non sono tempi dal clic
all'avvio nella UI, né garantiscono la stessa durata su altri PC o configurazioni.
Il log reale dell'app del 7 ottobre mostra inizializzazioni DFN ripetute prima
dei messaggi dei dispositivi, ma non misura clic, durata delle singole fasi o
comparsa della UI. Non permette di attribuire tutto il ritardo a DFN3.

## Audio e annullamento

Prima dell'apertura dei dispositivi non si cattura audio. Il canale di `LiveFeed`
accoda i frame per ASR (`engine/live.rs:31-63`): il testo può arrivare dopo senza
far dipendere la scrittura del Tape dalla prontezza del motore. La coda senza
limite richiede attenzione se il motore resta bloccato o molto indietro.

Il pool delle callback WASAPI è finito: non basta anticipare la scritta nella
UI per garantire che il primo parlato sia conservato. Il precedente problema
del mixer che sostituiva audio in coda con silenzio è documentato e corretto in
`.scratch/diagnosi-registrazione/README.md`; non è dimostrato che sia la causa
del ritardo di avvio riferito ora.

La cancellazione imposta il flag di Stop, ma il percorso attuale non lo osserva
durante preparazione DFN e apertura dispositivi: il ciclo lo legge dopo.
Servono confini di annullamento e un ordine definito fra avvio e annullamento,
oltre al pulsante nella UI. Le chiamate native non interrompibili non devono
produrre un avvio tardivo quando terminano.

## Interventi da progettare e verificare

1. Distinguere richiesta, preparazione e avvio confermato nella UI, fin dal clic.
2. Preparare in background e riusare il lavoro costoso DFN, con stato audio nuovo
   e indipendente per sessione e Ingresso. Valutare compatibilità del formato e
   invalidazione delle risorse; nessun `unsafe Send` per forzare il riuso.
3. Non attendere la pulizia spenta. Quando è richiesta, attenderne la disponibilità
   oppure gestirne il guasto secondo la scelta esplicita di prodotto.
4. Conservare ASR parallela alla cattura; misurare l'utilità del riuso delle
   istanze aggiuntive prima di aumentarne la memoria residente.
5. Misurare la verifica del diarizer locale e valutare come spostarla fuori dal
   percorso critico senza eludere l'integrità, accettare file cambiati o spostare
   l'analisi dei Parlanti prima di Stop.

La scelta di preparare e riusare DFN modifica il compromesso di ADR-0021:
quell'ADR preparava tutto a ogni Registra per rendere subito disponibile il
controllo durante la cattura. Le misure future sono in `verifica.md`.
