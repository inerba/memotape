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

**Status:** done

- [x] Sfoglia usa `tauri-plugin-dialog`, chiamato da Rust, con il filtro sulle estensioni accettate (spec, storia 2)
- [x] Trascrivi su un MP3 o WAV con parlato reale mostra le Frasi in ordine, una per riga, mentre arrivano
- [x] I trait `TranscriptionEngine` e `VoiceDetector` sono le uniche seam finte nei test. La pipeline è Tauri-free e alla velocità del calcolo
- [x] Test della pipeline (WAV generato, motore finto, detector finto) e del segmentatore (prefill, onset, hangover, taglio a 18 s)
- [x] Silero `silero_vad.onnx` (tag v4.0, SHA verificato) incluso come risorsa. Le DLL di ORT e dei backend sono copiate accanto all'exe
- [x] Smoke test `#[ignore]` con Nemotron vero
- [x] `AGENTS.md` documenta i prerequisiti (CMake, Vulkan SDK, zip ORT, `.cargo/config.toml` locale) e dove mettere il modello. `PRODUCT.md` è aggiornato
- [x] I controlli del ticket 01 passano

## Note di chiusura

- Verificato in `bun tauri dev` su un WAV con parlato italiano (`src-tauri/tests/fixtures/parlato-it.wav`, sintesi vocale di Windows). L'MP3 non è stato provato perché sulla macchina non c'è un encoder MP3; il decoder MP3 di Symphonia è comunque abilitato.
- L'interfaccia di `TranscriptionEngine` è ridotta di proposito: riceve solo l'iteratore dei frame della Frase. Lingua del parlato, callback dei Parziali, capability ed errori `Busy`/`Cancelled` arrivano con i ticket 04, 06 e 07.
- Con la build statica di `transcribe-cpp` non ci sono DLL dei backend da copiare: arrivano con `dynamic-backends` (ticket 12). `build.rs` copia oggi la DLL di ORT.
