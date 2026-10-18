---
status: accepted
---

# RNNoise 0.2 al posto di DeepFilterNet3

Decisione del 18 ottobre 2026, dopo l'inventario dell'installer 0.5.3 e una prova
d'ascolto alla cieca. Spec e ticket in `.scratch/rnnoise-al-posto-di-dfn3/`, prova e misure in
`.scratch/prova-denoiser/esito.md`. Sostituisce la scelta del modello dell'ADR-0019. Restano
il percorso PCM condiviso (ADR-0018), la pulizia per Ingresso (ADR-0021) e il riuso dei tratti
puliti (ADR-0022, per quanto non superato dall'ADR-0029).

## Perché

DeepFilterNet3 su tract costava circa 11 MB compressi dell'installer (8 MB di modello e il
codice di tract nell'exe) e 0,083–0,088 secondi di CPU per secondo di audio. Sulla voce molto
debole del corpus (la fixture attenuata di 30 dB) la sua uscita diretta era silenzio digitale
(−217 dB): Silero non trovava nessuna Frase.

Alla cieca, sul campione del ristorante dell'ADR-0019, l'utente ha giudicato RNNoise 0.2
«quasi ai livelli» di DFN3. RNNoise 0.1 (quello di OBS) attenuava poco e apriva una falsa Frase
sul rumore intermittente. Con RNNoise 0.2, misurato con Silero v4 a soglia 0,4:

- la voce a −30 dB resta parlato;
- «sì» e «no» si conservano;
- il rumore intermittente scende di 30 dB senza false Frasi, il soffio di 72 dB.

Il banco della prova differiva dalla produzione in tre punti: compilava RNNoise con AVX2
fisso, usava le copie float dei pesi e compensava un solo frame di ritardo (le sue misure
d'inizio Frase sono spostate di 10 ms). Lì costava 0,015 secondi di CPU per secondo di audio;
in produzione, con l'int8 e la scelta a runtime, 0,009, su un Ryzen 7 3700X con AVX2.

## Cosa

- RNNoise ufficiale di xiph, tag `v0.2` (commit `904a876dce1f9ab8860c0a5000ed151f9f6eef58`),
  licenza BSD-3-Clause.
  - I sorgenti C della libreria sono in `src-tauri/vendor/rnnoise` e si compilano con `cc`,
    senza bindgen né download, con la scelta SIMD a runtime della build ufficiale: generica
    SSE2, SSE4.1, AVX2.
  - Una riga corretta: `rnnoise_model_from_buffer` non inizializzava `model->file`.
- Modello normale `0b50c45`. Il file dei pesi è prodotto una volta con `write_weights` dal
  tarball ufficiale (SHA-256 `4ac81c5c0884ec4bd5907026aaae16209b7b76cd9d7f71af582094a2f98f4b43`).
  - È nella build ufficiale, con `DISABLE_DEBUG_FLOAT`: 1 401 600 byte, SHA-256
    `47edcad7baeffb6442d9bfe8ea3b3ae728b50055c3abca1d69b4e31a806da27d`.
  - Dimensione e hash si verificano prima del caricamento, con gli errori distinti per assenza
    e incompatibilità.
  - La prova d'ascolto usava anche le copie float dei pesi (5,5 MB): sul ristorante le due
    uscite differiscono di −43 dB, con la stessa attenuazione e metà CPU per l'int8.
- L'adattatore `AudioProcessor` segue lo schema dell'ADR-0019:
  - ricampiona a 48 kHz e torna al formato della Sorgente;
  - tiene uno stato per canale ed elabora frame da 480 campioni in scala int16;
  - scarica la coda completa a ogni confine;
  - usa l'uscita diretta del modello.

  Il ritardo è di due frame, non uno come nella 0.1: la 0.2 sintetizza lo spettro del frame
  precedente. La probabilità di voce di RNNoise non si usa: il VAD resta Silero.
- RNNoise si carica in circa 2 ms e il suo stato non è legato al thread. Nella Registrazione
  sparisce la preparazione in anticipo dell'ADR-0026: il processore nasce al primo blocco con
  la pulizia accesa.
- I nuovi tratti hanno `algoritmo` `rnnoise` e versione
  `v0.2/904a876dce1f9ab8860c0a5000ed151f9f6eef58/0b50c45/direct-v1`. I tratti `deepfilternet3`
  dei Tape esistenti restano e si leggono.
- Escono `deep_filter`, i crate tract, il modello DFN3, le loro licenze e le ottimizzazioni del
  profilo dev dedicate. L'installer passa da 29,6 a 19,7 MB e l'app installata da 126,5 a
  103,4 MB (`bun tauri build` del 18 ottobre, senza le modifiche della spec parallela
  `alleggerire-installer`).

## Conseguenze e limiti

- Con un rumore molto forte ci si aspetta che RNNoise degradi anche la voce, come ogni filtro
  a guadagno per banda; non è misurato. Non c'è un controllo d'intensità.
- Sulla voce a −30 dB pulita Silero trova 2 Frasi contro le 3 dell'originale (si perde il
  primo «Buongiorno a tutti»), e nessun modello ASR scaricato ne restituisce testo. Senza
  pulizia ne trascrivono solo «Buongiorno a tutti». I test con i modelli veri chiedono che la
  voce debole resti una Frase per Silero, non che arrivi nella Trascrizione.
- La prova su una CPU senza AVX2 è tra le verifiche sull'hardware reale
  (`docs/sviluppo/verifica-manuale.md`).
