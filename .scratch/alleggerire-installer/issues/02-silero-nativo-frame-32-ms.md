# 02: Silero v6.2 nativo con frame da 32 ms

**What to build:** Trascrivere un file, un Tape e dal vivo con Silero VAD v6.2 eseguito da transcribe-cpp al posto di Silero v4 su `vad-rs`, con la pipeline a frame da 512 campioni (32 ms). Il GGUF è una risorsa del bundle verificata prima del caricamento; un modello Silero per pipeline. Spec: `../spec.md`, sezioni "VAD" e "Frame".

**Blocked by:** 01

**Status:** done

- [x] Una nuova implementazione di `VoiceDetector` sopra `VadSession` (`stream_feed` di un frame, `stream_reset`), backend CPU, un `Model` caricato per pipeline.
- [x] Il GGUF `silero-vad-v6.2-F32.gguf` (1 241 056 byte, SHA-256 `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`, revisione Hugging Face fissata) sostituisce `silero_vad.onnx` tra le risorse; dimensione e hash verificati prima del caricamento, con errori distinti per assenza e incompatibilità, tradotti nelle sei lingue.
- [x] Una costante per il frame in campioni (512) e una in millisecondi (32); ogni `480` e ogni «30 ms» scritto a mano in pipeline, protezione, dal vivo e livello deriva da lì. Prefill, onset, hangover e durata massima restano in millisecondi.
- [x] Soglie invariate in questo ticket (la riverifica è il ticket 03).
- [x] Test dell'adattatore con il modello vero: parlato di `parlato-it.wav` sopra soglia, silenzio sotto, `reset` riproducibile, errore con modello assente o non valido. I test con il detector finto passano con i frame nuovi; quelli che caricavano `silero_vad.onnx` usano il GGUF.
- [x] Gli smoke `--ignored` dei modelli scaricati trascrivono ancora `parlato-it.wav` e `parlato-it.mp4`.
- [x] `vad-rs` può restare nel manifest finché il ticket 04 non lo toglie, ma l'app non lo usa più.
- [x] I sei controlli passano.

**Esito:** `audio_toolkit::vad::Silero` sta sopra `VadSession` (`stream_feed` di un frame, `stream_reset`), un `Model` per `run_pipeline`, backend lasciato ad `AUTO` (la CPU per il VAD). Il GGUF è `src-tauri/resources/silero-vad-v6.2-F32.gguf`, scaricato dalla revisione `a9b2d61c20ae4ddf3644edb34969e8185a485448` di `handy-computer/silero-vad-v6.2-gguf` (dimensione e SHA-256 controllati); `silero_vad.onnx` è uscito dal repo e dal bundle. Errori nuovi `VadModelMissing` e `VadModelIncompatible`. `resample::FRAME_SAMPLES` = 512 e `resample::FRAME_MS` = 32 (derivato, con un assert in compilazione); `segmenter::FRAME_MS` non c'è più; la finestra delle osservazioni della protezione (prima 612 e 12 frame) si ricava dal segmentatore; `MAX_QUEUED_FEEDS` resta 60 s. I test con il detector finto attendono multipli di `FRAME_MS`; due fixture sintetiche sono passate a 0,96 s e a 0,64 s di silenzio iniziale, così restano allineate ai frame. Lo smoke `i_modelli_scaricati_trascrivono_il_parlato_italiano` (`--ignored`) trascrive `parlato-it.wav` e `parlato-it.mp4` con Nemotron, Cohere, Whisper medium, Parakeet e Whisper large-v3 turbo. I test ignorati `live_protection_native` non sono partiti, perché nel worktree manca il corpus `.scratch/pulizia-audio/verification-05/corpus` (non è versionato): girano nel ticket 03 insieme agli altri test del corpus. `vad-rs` resta nel manifest per il ticket 04. Documenti aggiornati: `insidie-audio.md` (Silero), `trascrizione.md` (frame), `installer.md` (risorsa); il resto dei documenti va nei ticket 03 e 04.
