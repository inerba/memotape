# 02: Tracer bullet: file audio → testo

**What to build:** l'utente preme Sfoglia, sceglie un file audio, preme Trascrivi e vede comparire il testo, una Frase per riga, nell'area dedicata. Il percorso attraversa tutta la pipeline vera:
- decodifica con Symphonia (+ adapter libopus);
- resampler rubato verso frame da 30 ms a 16 kHz;
- `VoiceDetector` Silero (vad-rs su ONNX Runtime ufficiale 1.24.2, caricato dinamicamente);
- segmentatore con i parametri di partenza della spec;
- `TranscriptionEngine` con `transcribe-cpp` (Nemotron in modalità `run`, feature `vulkan`);
- evento `transcript-phrase`.

Il modello Nemotron Q5_K_M si mette a mano nella cartella dei modelli, seguendo la procedura in `AGENTS.md`.

**Blocked by:** 01 (Scaffold)

**Status:** ready-for-agent

- [ ] Sfoglia usa `tauri-plugin-dialog`, chiamato da Rust, con il filtro sulle estensioni accettate (spec, storia 2)
- [ ] Trascrivi su un MP3 o WAV con parlato reale mostra le Frasi in ordine, una per riga, mentre arrivano
- [ ] I trait `TranscriptionEngine` e `VoiceDetector` sono le uniche seam finte nei test. La pipeline è Tauri-free e alla velocità del calcolo
- [ ] Test della pipeline (WAV generato, motore finto, detector finto) e del segmentatore (prefill, onset, hangover, taglio a 18 s)
- [ ] Silero `silero_vad.onnx` (tag v4.0, SHA verificato) incluso come risorsa. Le DLL di ORT e dei backend sono copiate accanto all'exe
- [ ] Smoke test `#[ignore]` con Nemotron vero
- [ ] `AGENTS.md` documenta i prerequisiti (CMake, Vulkan SDK, zip ORT, `.cargo/config.toml` locale) e dove mettere il modello. `PRODUCT.md` è aggiornato
- [ ] I controlli del ticket 01 passano
