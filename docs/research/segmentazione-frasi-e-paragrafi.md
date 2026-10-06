# Segmentazione di Frasi, turni e paragrafi: confronto con i concorrenti

Data delle verifiche: 2026-10-06.

Scopo: rimettere in discussione le regole di Memotape dopo la segnalazione di blocchi troppo grandi sia nella Registrazione dal vivo sia nella Trascrizione di file. Questa ricerca non cambia i requisiti né il codice.

Legenda: **[V]** verificato su una fonte primaria; **[I]** interpretazione; **[P]** proposta da discutere.

## Risultati verificati

| Prodotto | Comportamento documentato | Limite della verifica |
| --- | --- | --- |
| Otter | Raggruppa automaticamente paragrafi della stessa voce; in modifica Invio/Return crea un nuovo paragrafo. L'identificazione dei Parlanti parte dopo Stop o dopo l'importazione; Notetaker per Zoom può usare i nomi dei partecipanti dal vivo. | Non pubblica nella pagina le soglie per silenzi, durata o numero di parole. |
| Sonix | Invio nel punto in cui comincia una nuova voce crea un paragrafo con timestamp; si può assegnare il nome. Un comando separato unisce paragrafi adiacenti con lo stesso nome del Parlante. | Le istruzioni non definiscono una soglia generale della paragrafazione automatica. |
| Descript | Si può assegnare un Parlante a una selezione di testo e spostare l'etichetta prima di una parola. La wordbar consente di correggere inizio e fine di ogni parola rispetto all'audio. | Queste funzioni non dimostrano quale algoritmo produca i paragrafi iniziali. |
| Sonix, sottotitoli | La divisione in sottotitoli avviene dalla trascrizione e ha preset per frase o per parola, oltre a vincoli su caratteri, righe, durata, velocità di lettura e pause. | Sono regole dei sottotitoli, non una prova della regola dei paragrafi dell'editor. |

**[V] Fonti:**

- Otter: [Speaker Identification Overview](https://help.otter.ai/hc/en-us/articles/21665587209367-Speaker-Identification-Overview).
- Sonix: [How to add a new speaker](https://sonix.ai/resources/sonix-tutorials-add-new-speaker-sonix/).
- Sonix: [How do I fix speaker labeling issues?](https://help.sonix.ai/en/articles/5141425-how-do-i-fix-speaker-labeling-issues). La modalità specifica che elimina tutte le etichette condensa in paragrafi di circa 500 parole: non è la soglia predefinita generale.
- Descript: [Speaker labels](https://help.descript.com/script-editing/speaker-labels).
- Descript: [The wordbar](https://help.descript.com/script-editing/the-wordbar).
- Sonix: [How do I create subtitles?](https://help.sonix.ai/en/articles/4649596-how-do-i-create-subtitles-in-sonix).

Per i prodotti locali e il codice di WhisperX vedere anche [la ricerca dedicata](segmentazione-concorrenti-locali.md).

## Memotape oggi

**[V]** Codice letto nel checkout corrente, comprese le modifiche locali già presenti:

- `src-tauri/src/audio_toolkit/segmenter.rs`, `Params::default`: prefill 300 ms, onset 60 ms, hangover 700 ms, massimo 18.000 ms, soglia di parlato 0,4. I frame sono da 30 ms: 700 ms diventano 24 frame, cioè 720 ms.
- `src/features/transcription/phrases.ts`, `turnsOf`: le Frasi consecutive con la stessa etichetta si uniscono; quando esistono etichette, la pausa non interrompe l'unione. Non c'è un limite di parole o durata del blocco. Senza etichette una pausa stimata oltre 2.000 ms apre un paragrafo.
- `src-tauri/src/engine/diarize.rs`, `assign_configured`: Nemotron 3 può dividere usando i tempi del testo; Sortformer assegna la Frase intera in base alla maggiore sovrapposizione temporale.
- `PRODUCT.md`, revisione del 6 ottobre 2026: durante la Registrazione si esegue solo ASR; la Diarizzazione usa l'audio salvato dopo Stop.

## Implicazioni

**[I]** Le fonti mostrano che la struttura leggibile è modificabile oltre all'attribuzione della voce. Non provano che tutti i concorrenti usino lo stesso algoritmo, né che producano risultati migliori in italiano.

**[I]** In Memotape la Frase audio, il periodo linguistico e il paragrafo visibile hanno confini diversi. La fusione illimitata per etichetta può annullare visivamente una divisione audio corretta. Ridurre solo l'hangover non risolve questa fusione.

## Proposte da discutere

1. **[P]** Conservare i tagli audio necessari all'ASR come dettaglio della pipeline e decidere separatamente i confini leggibili del testo.
2. **[P]** Permettere più paragrafi nello stesso turno di un Parlante. Un cambio di voce apre un turno; una pausa lunga o un limite di lunghezza può aprire un paragrafo senza cambiare identità.
3. **[P]** Provare paragrafi di 2–4 periodi o circa 60–100 parole come obiettivo morbido, cercando un confine naturale. Sono valori iniziali da provare, non valori documentati dei concorrenti; non usare un taglio cieco alla parola numero N.
4. **[P]** Usare insieme fine di periodo, pausa e lunghezza. Il punto da solo è ambiguo con abbreviazioni e numeri; la punteggiatura ASR può essere incompleta o errata.
5. **[P]** Dal vivo lasciare rettificabile la parte di testo in corso ed evitare di riparagrafare continuamente il testo già stabilizzato. Dopo Stop e su file usare le stesse regole finali, integrate con i cambi di Parlante riconosciuti.
6. **[P]** Consentire divisione e unione manuali dei paragrafi. I confini manuali devono sopravvivere alla riapertura e alle esportazioni.
7. **[P]** Non inventare tempi per nuove unità testuali: dividere la struttura di lettura non autorizza a interpolare timestamp. Senza tempi affidabili mantenere l'aggancio audio della Frase originale. Se si cambia il testo, considerare l'invalidazione dei tempi già prevista dal prodotto.

## Decisioni ancora aperte

- Vista principale: un periodo per riga oppure paragrafi brevi?
- Il limite morbido segue periodi, parole, durata o una combinazione?
- Correzione e salto audio restano per Frase originale oppure devono diventare per periodo linguistico?
- I paragrafi manuali diventano parte del Tape oppure solo una preferenza di visualizzazione?

Non sono state eseguite prove pratiche nelle UI dei concorrenti né misure sull'italiano reale. Questa nota descrive funzioni documentate e propone criteri; non attesta qualità, latenza o soglie interne non pubblicate.
