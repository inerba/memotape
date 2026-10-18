# 01: Pulizia RNNoise 0.2 su file e Tape

**What to build:** Trascrivi su un file audio o video, o su un Tape, con la pulizia accesa usa RNNoise 0.2 al posto di DFN3, con lo stesso comportamento di oggi per l'utente. Le Registrazioni restano su DFN3 fino al ticket 02. Spec: `../spec.md`, sezioni "Modello e build", "Il processore", "Tratti puliti e Tape".

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Sorgenti C della libreria di xiph/rnnoise al tag `v0.2` (`904a876d`) nel repo, con licenza BSD-3-Clause e provenienza annotata; compilati con `cc` dallo script di build, con la scelta SIMD a runtime (generica, SSE4.1, AVX2). Niente bindgen, niente download durante la build.
- [ ] Il file dei pesi prodotto con `write_weights` dal modello `0b50c45` (tarball SHA-256 `4ac81c5c0884ec4bd5907026aaae16209b7b76cd9d7f71af582094a2f98f4b43`) tra le risorse del bundle; dimensione e SHA-256 del file fissati nel codice e verificati prima del caricamento, con gli errori distinti di oggi per assenza e incompatibilità. Procedura per rigenerarlo nel documento di sviluppo.
- [ ] Un adattatore `AudioProcessor` RNNoise: ricampionamento interno a 48 kHz e ritorno al formato della Sorgente, stato per canale, frame da 480 in scala int16, compensazione del ritardo, scarico completo a Pausa, Configurazione e Finish, uscita diretta, storia di sola lettura che scalda lo stato a inizio tratto (ADR-0022).
- [ ] `ConfiguredCleaning` per file e Tape costruisce l'adattatore RNNoise; i nuovi tratti hanno `algoritmo` «rnnoise» e una `versione` con tag, commit, modello e uscita diretta. I tratti DFN3 dei Tape esistenti si leggono e non si ripuliscono.
- [ ] Test dell'adattatore con il modello vero: durata identica a 16, 44,1 e 48 kHz e in stereo, impulso allineato, coda completa ai tre confini, voce della fixture conservata, silenzio digitale invariato, errori con pesi assenti o hash sbagliato.
- [ ] Regressione: sul campione del corpus con la voce attenuata di 30 dB, dopo la pulizia Silero trova ancora una Frase.
- [ ] I test con il modello vero della pulizia nella Trascrizione di file e Tape passano con RNNoise senza cambiare cosa verificano.
- [ ] I sei controlli passano.
