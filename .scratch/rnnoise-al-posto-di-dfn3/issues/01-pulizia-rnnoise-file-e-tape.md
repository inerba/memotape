# 01: Pulizia RNNoise 0.2 su file e Tape

**What to build:** Trascrivi su un file audio o video, o su un Tape, con la pulizia accesa usa RNNoise 0.2 al posto di DFN3, con lo stesso comportamento di oggi per l'utente. Le Registrazioni restano su DFN3 fino al ticket 02. Spec: `../spec.md`, sezioni "Modello e build", "Il processore", "Tratti puliti e Tape".

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [x] Sorgenti C della libreria di xiph/rnnoise al tag `v0.2` (`904a876d`) nel repo, con licenza BSD-3-Clause e provenienza annotata; compilati con `cc` dallo script di build, con la scelta SIMD a runtime (generica, SSE4.1, AVX2). Niente bindgen, niente download durante la build.
- [x] Il file dei pesi prodotto con `write_weights` dal modello `0b50c45` (tarball SHA-256 `4ac81c5c0884ec4bd5907026aaae16209b7b76cd9d7f71af582094a2f98f4b43`) tra le risorse del bundle; dimensione e SHA-256 del file fissati nel codice e verificati prima del caricamento, con gli errori distinti di oggi per assenza e incompatibilità. Procedura per rigenerarlo nel documento di sviluppo.
- [x] Un adattatore `AudioProcessor` RNNoise: ricampionamento interno a 48 kHz e ritorno al formato della Sorgente, stato per canale, frame da 480 in scala int16, compensazione del ritardo, scarico completo a Pausa, Configurazione e Finish, uscita diretta, storia di sola lettura che scalda lo stato a inizio tratto (ADR-0022).
- [x] `ConfiguredCleaning` per file e Tape costruisce l'adattatore RNNoise; i nuovi tratti hanno `algoritmo` «rnnoise» e una `versione` con tag, commit, modello e uscita diretta. I tratti DFN3 dei Tape esistenti si leggono e non si ripuliscono.
- [x] Test dell'adattatore con il modello vero: durata identica a 16, 44,1 e 48 kHz e in stereo, impulso allineato, coda completa ai tre confini, voce della fixture conservata, silenzio digitale invariato, errori con pesi assenti o hash sbagliato.
- [x] Regressione: sul campione del corpus con la voce attenuata di 30 dB, dopo la pulizia Silero trova ancora una Frase.
- [ ] I test con il modello vero della pulizia nella Trascrizione di file e Tape passano con RNNoise senza cambiare cosa verificano.
- [x] I sei controlli passano.

## Note dell'implementazione (18 ottobre 2026)

- **Storia di sola lettura**: non esiste più. L'ADR-0029 ha tolto la pulizia di Trascrivi su un Tape e con essa il contesto dell'ADR-0022; si pulisce solo un file importato. «File e Tape» vale quindi come «file importati, e i tratti DFN3 dei Tape esistenti si leggono».
- **Ritardo**: la 0.2 ritarda di due frame (960 campioni), non uno come la 0.1 e OBS: sintetizza lo spettro del frame precedente (`delayed_X`). Il test dell'impulso lo verifica.
- **Pesi**: blob della build ufficiale con `DISABLE_DEBUG_FLOAT`, 1 401 600 byte. Quello con le copie float, come nel banco d'ascolto, è di 5,5 MB e usa il percorso float. Sul ristorante: differenza −43 dB, stessa attenuazione, metà CPU (RTF 0,009 contro 0,019).
- **Patch**: una riga in `rnnoise_model_from_buffer`, che nella 0.2 non inizializza `model->file` (crash in `rnnoise_model_free`); in `vendor/rnnoise/PROVENIENZA.md`.
- **Test con i modelli veri** (5 ASR scaricati):
  - importazione audio/video: verde;
  - corpus della protezione: verde;
  - `rnnoise_corpus_prudente_confronta_pulizia_accesa_e_spenta`: resta rosso sulla voce a −30 dB già con la pulizia **spenta**, perché ogni ASR trascrive solo «Buongiorno a tutti». Con RNNoise Silero trova le Frasi (la regressione richiesta), ma nessun ASR restituisce testo: a livello identico (−0,14 dB, correlazione 0,977) RNNoise abbassa sotto soglia la probabilità di Silero sul primo «Buongiorno a tutti» a inizio file, l'unico tratto che gli ASR trascrivevano. DFN3 lì dava silenzio digitale.
