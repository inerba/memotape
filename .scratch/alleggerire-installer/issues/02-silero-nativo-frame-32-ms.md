# 02: Silero v6.2 nativo con frame da 32 ms

**What to build:** Trascrivere un file, un Tape e dal vivo con Silero VAD v6.2 eseguito da transcribe-cpp al posto di Silero v4 su `vad-rs`, con la pipeline a frame da 512 campioni (32 ms). Il GGUF è una risorsa del bundle verificata prima del caricamento; un modello Silero per pipeline. Spec: `../spec.md`, sezioni "VAD" e "Frame".

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Una nuova implementazione di `VoiceDetector` sopra `VadSession` (`stream_feed` di un frame, `stream_reset`), backend CPU, un `Model` caricato per pipeline.
- [ ] Il GGUF `silero-vad-v6.2-F32.gguf` (1 241 056 byte, SHA-256 `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`, revisione Hugging Face fissata) sostituisce `silero_vad.onnx` tra le risorse; dimensione e hash verificati prima del caricamento, con errori distinti per assenza e incompatibilità, tradotti nelle sei lingue.
- [ ] Una costante per il frame in campioni (512) e una in millisecondi (32); ogni `480` e ogni «30 ms» scritto a mano in pipeline, protezione, dal vivo e livello deriva da lì. Prefill, onset, hangover e durata massima restano in millisecondi.
- [ ] Soglie invariate in questo ticket (la riverifica è il ticket 03).
- [ ] Test dell'adattatore con il modello vero: parlato di `parlato-it.wav` sopra soglia, silenzio sotto, `reset` riproducibile, errore con modello assente o non valido. I test con il detector finto passano con i frame nuovi; quelli che caricavano `silero_vad.onnx` usano il GGUF.
- [ ] Gli smoke `--ignored` dei modelli scaricati trascrivono ancora `parlato-it.wav` e `parlato-it.mp4`.
- [ ] `vad-rs` può restare nel manifest finché il ticket 04 non lo toglie, ma l'app non lo usa più.
- [ ] I sei controlli passano.
