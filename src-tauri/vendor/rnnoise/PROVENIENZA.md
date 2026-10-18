# RNNoise 0.2

Sorgenti C della libreria di [xiph/rnnoise](https://gitlab.xiph.org/xiph/rnnoise), tag `v0.2`
(commit `904a876dce1f9ab8860c0a5000ed151f9f6eef58`), licenza BSD-3-Clause (`COPYING`, copiato
anche in `resources/licenses/rnnoise.txt`). `build.rs` li compila con `cc`.

## Cosa c'è

- `include/rnnoise.h`, `COPYING`, `AUTHORS` come nel tag.
- `src/` e `src/x86/`: i sorgenti di `RNNOISE_SOURCES` del `Makefile.am`, con le varianti
  SSE4.1 e AVX2 e la scelta a runtime (`x86_dnn_map.c`, `x86cpu.c`). Esclusi gli esempi, gli
  strumenti (`dump_features.c`, `dump_rnnoise_tables.c`, `write_weights.c`), gli script e il
  training.
- `src/rnnoise_data.h` e `src/rnnoise_data.c` vengono dal tarball del modello
  `rnnoise_data-0b50c45.tar.gz` (SHA-256
  `4ac81c5c0884ec4bd5907026aaae16209b7b76cd9d7f71af582094a2f98f4b43`). Di `rnnoise_data.c`
  restano solo le parti fuori dai blocchi `#ifndef USE_WEIGHTS_FILE`, cioè `init_rnnoise`: i
  pesi (29 MB di sorgente) stanno nel file binario `resources/models/rnnoise-0b50c45.bin`.
- I pesi binari sono quelli della build ufficiale, con `DISABLE_DEBUG_FLOAT`: solo int8 dove il
  modello è quantizzato (1,4 MB). La prova d'ascolto usava anche le copie float, che hanno la
  precedenza (5,5 MB). Sul campione del ristorante le due uscite differiscono di −43 dB, con la
  stessa attenuazione (−2,7 dB) e metà della CPU per l'int8 (RTF 0,009 contro 0,019, AVX2).

## Modifiche

- `src/denoise.c`, `rnnoise_model_from_buffer`: inizializza `model->file` a `NULL`. Nella 0.2
  manca, e `rnnoise_model_free` chiamava `fclose` su memoria non inizializzata (crash).
