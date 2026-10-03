# Ricerca: transcribe-cpp e Handy come riferimento per Sbobino

Data: 2026-10-02. Scopo: raccogliere fatti verificati per la pipeline di Sbobino
(cpal, poi mono, poi rubato, poi frame da 30 ms a 16 kHz, poi Silero VAD v4, poi
`TranscriptionEngine` su `transcribe-cpp`). Non contiene decisioni: le proposte
sono marcate **Proposta** e la scelta finale spetta all'utente. Da Handy si
prendono idee, mai nome, logo o asset.

Legenda: **[V]** = verificato su fonte primaria (file e riga, URL o output di
comando). **[I]** = inferenza mia, da confermare.

## 0. Fonti e versioni fissate

| Fonte | Riferimento |
| --- | --- |
| Handy (clone `--depth 50`) | `https://github.com/cjpais/Handy`, HEAD `5ec58f696354fcf64ae831102e673779e0249717` (2026-10-02, "Update PULL_REQUEST_TEMPLATE.md"), versione app 0.9.7 |
| Permalink Handy | `https://github.com/cjpais/Handy/blob/5ec58f696354fcf64ae831102e673779e0249717/<path>#L<n>`. Sotto i path sono abbreviati, relativi alla root del repo |
| transcribe-cpp / -sys | crate 0.2.4 (pubblicato 2026-09-25, licenza MIT), sorgente `.crate` scaricato da `static.crates.io` e letto |
| Repo transcribe.cpp | `https://github.com/handy-computer/transcribe.cpp`, tag `v0.2.4` -> commit `4807edaf210d0d7e8a6f7fb2a44b65966a2797f0`. Clone HEAD `e85b30edac87533168863283c1e595bf39bd7d15` (2026-09-30, quindi **dopo** 0.2.4) |
| vad-rs (fork) | `https://github.com/cjpais/vad-rs` commit `2a412ed858695b9251f3f5a1a20d95b59fa7c498` (versione 0.1.6), fork di `thewh1teagle/vad-rs` |
| rubato | Handy usa 0.16.2. L'ultima versione su crates.io è 5.0.1 (2026-10-01). Entrambe lette dal `.crate` |
| cpal | Handy usa 0.16.0. L'ultima è 0.18.2 (2026-08-16), letta dal `.crate` |
| ort | Handy (tramite vad-rs) usa `=2.0.0-rc.12`. L'ultima è 2.0.0-rc.13 (2026-07-28) |

Comando usato per le versioni: `curl https://crates.io/api/v1/crates/<nome>`
(campi `max_stable_version`, `newest_version`, `versions[].license`).

---

## 1. Handy: cattura audio con cpal

### Fatti
- **[V] Versioni.** `cpal = "0.16.0"` e `rtrb = "0.4.0"` (`src-tauri/Cargo.toml:51-52`). La lock risolve cpal 0.16.0 (`src-tauri/Cargo.lock:1059-1060`).
- **[V] Host.** Su Windows usa `cpal::default_host()`, cioè WASAPI. Forza ALSA solo su Linux (`src-tauri/src/audio_toolkit/utils.rs:3-12`).
- **[V] Enumerazione.** `list_input_devices()` e `list_output_devices()` usano `device.name()` e confrontano con il nome del default per marcare `is_default`. L'indice è la posizione nell'enumerazione (`src-tauri/src/audio_toolkit/audio/device.rs:10-52`).
- **[V] Configurazione.** Usa la frequenza nativa del dispositivo (`default_input_config().sample_rate()`) e non forza 16 kHz. Tra le config che supportano quella frequenza sceglie il formato con priorità F32 > I16 > I32 > altri. Fallback: la config di default (`src-tauri/src/audio_toolkit/audio/recorder.rs:555-608`). La config scelta è in cache per nome dispositivo, perché le query HAL costano 40-85 ms (`recorder.rs:97-103`, `209-220`, `311-313`). La cache si invalida a ogni apertura fallita (`recorder.rs:341-344`).
- **[V] Formati.** Gestisce U8, I8, I16, I32 e F32 via `build_stream::<T>`. Gli altri formati danno errore "Unsupported sample format" (`recorder.rs:249-293`).
- **[V] Thread.** `open()` lancia un worker thread che possiede lo `cpal::Stream`. L'esito dell'init torna su un `sync_channel(1)` (`recorder.rs:183-184`, `204-349`, `351-373`). I comandi `Start`, `Stop` e `Shutdown` passano su `mpsc` (`recorder.rs:23-29`).
- **[V] Dal callback al consumer.** Ring SPSC `rtrb` wait-free con capacità di 2 s (`AUDIO_RING_SECONDS = 2`). Le pagine vengono pre-toccate per ridurre i page fault (`recorder.rs:31-36`, `438-454`). Il callback deve restare "allocation-, lock-, logging-, and blocking-free" (`recorder.rs:38-39`, `483-484`). Il consumer fa polling ogni 10 ms e drena al massimo 50 ms per giro (`recorder.rs:34-35`, `914-1043`). Gli overrun si contano in un `AtomicU64` (`recorder.rs:537-542`). Nota: Handy usa un ring buffer, non un canale. Il "canale" della pipeline di Sbobino corrisponde a questo ring.
- **[V] Mono.** Il downmix si fa nel callback, prima del ring. Con 1 canale copia. Con un canale selezionato prende `frame[channel]`. Altrimenti fa la media di tutti i canali (`recorder.rs:511-533`, `456-458`).
- **[V] Errori del dispositivo.** L'error callback imposta solo un `AtomicBool`. Il rebuild avviene al prossimo `open()` (`recorder.rs:470-479`, `403-413`, `1036-1041`).
- **[V] Stop senza perdite.** Una pausa con handshake atomico (`pause_requested` e `pause_acknowledged`) inoltra un ultimo blocco di confine, poi drena tutto il ring. Il timeout è 2 s (`recorder.rs:960-1009`, `611-615`).
- **[V] Timestamp dei buffer: non usati.** Il callback ignora `&cpal::InputCallbackInfo` (`recorder.rs:460`). Handy misura solo latenze diagnostiche con `Instant` sul consumer (`recorder.rs:321-323`, `810-818`).
- **[V] Loopback: assente in Handy.** Registra solo input. L'unico uso di `eRender` è il mute di sistema via `IAudioEndpointVolume` (`src-tauri/src/managers/audio.rs:23-40`).

### cpal 0.18.2 (ultima versione): cosa cambia per Sbobino
- **[V] Loopback WASAPI.** Se il device è di render (`eRender`), `build_input_stream` aggiunge `AUDCLNT_STREAMFLAGS_LOOPBACK` (`cpal-0.18.2/src/host/wasapi/device.rs:854-856`). Quindi il loopback si ottiene aprendo uno stream di **input** su un **output device**. Verificato solo sul sorgente 0.18.2. **[I]** È probabile che valga anche per 0.16, ma non l'ho controllato.
- **[V] Timestamp.** `InputCallbackInfo` contiene un `InputStreamTimestamp { callback, capture }` (`cpal-0.18.2/src/timestamp.rs:43-69`). Su WASAPI la sorgente è `QueryPerformanceCounter()` (tabella in `timestamp.rs:15-31`). `capture` deriva dal `qpc_position` restituito da `IAudioCaptureClient::GetBuffer` (`src/host/wasapi/stream.rs:802-853`, `939-949`). Avvertenza esplicita: tra stream diversi le origini dei clock non sono garantite condivise (`timestamp.rs:10-13`). **[I]** Su WASAPI entrambe le sorgenti usano QPC, quindi il timestamp `capture` dovrebbe permettere di allineare microfono e loopback. Da verificare sul campo.
- **[V] Breaking change 0.16 -> 0.17.** `SampleRate` diventa un alias di `u32`, quindi il codice di Handy `config.sample_rate().0` (`recorder.rs:223`) non compilerebbe più. `device.name()` è deprecato a favore di `description()` e `id()` (`cpal-0.18.2/UPGRADING.md:397-411`).
- **[V] Breaking change 0.17 -> 0.18.** Errore unificato `cpal::Error` con `kind()`. `build_*_stream` riceve `StreamConfig` per valore. `device.name()` va sostituito. Se `F32` manca, `I32`/`I24` hanno priorità su `I16` (`UPGRADING.md:1-30`). Firma: `build_input_stream(config: StreamConfig, data_cb, err_cb, timeout)` (`cpal-0.18.2/src/traits.rs:261-267`).
- **[V] Stream thread-safe.** Da 0.17 `Stream` WASAPI implementa `Send` e `Sync` (`cpal-0.18.2/CHANGELOG.md:356`).

**Proposta.** Due worker con lo stesso schema di Handy (ring SPSC, poi consumer),
uno per il microfono e uno per il loopback. Il loopback si apre come input stream
su `default_output_device()`. Conviene salvare il `capture` timestamp del primo
blocco di ogni sorgente come àncora temporale dei segmenti. Il downmix va nel
callback come fa Handy.

---

## 2. Handy: resampling con rubato

### Fatti
- **[V] Versione e tipo.** `rubato = "0.16.2"` (`src-tauri/Cargo.toml:54`). Usa `FftFixedIn<f32>`, sincrono FFT a input fisso (`src-tauri/src/audio_toolkit/audio/resampler.rs:1`, `8`).
- **[V] Chunk.** `RESAMPLER_CHUNK_SIZE = 1024` frame di input, fisso e non derivato dal GCD (`resampler.rs:4-5`, `26-27`). Costruttore: `FftFixedIn::<f32>::new(in_hz, out_hz, 1024, 1, 1)`, cioè `sub_chunks = 1` e 1 canale. Il resampler si crea solo se `in_hz != out_hz` (`resampler.rs:29-32`).
- **[V] Framing.** `FrameResampler` accumula l'input fino a 1024 campioni, chiama `process(&[&in_buf], None)` ed emette frame di dimensione fissa. A 16 kHz con 30 ms sono 480 campioni (`resampler.rs:22-24`, `47-76`, `145-157`). La dimensione del frame la decide il VAD attivo, con 30 ms di default (`recorder.rs:730-742`).
- **[V] Svuotamento a fine registrazione.** `process_partial` sul residuo, poi chunk di zeri finché `out_count >= in_count*out/in + output_delay()` (al massimo 8 giri). Il frame finale viene riempito di zeri (`resampler.rs:78-129`). `reset()` tra una registrazione e l'altra evita che l'overlap FFT trascini audio vecchio (`resampler.rs:131-143`).
- **[V] `FftFixedIn` non esiste più nella versione corrente.** Nel CHANGELOG di rubato v1.0.0 c'è "Merged the FixedIn, FixedOut and FixedInOut resamplers into single types" e "New API using the AudioAdapter crate" (`rubato-5.0.1/README.md:602-606`). In 5.0.1 il resampler FFT sincrono è `Fft<T>` con `FixedSync::{Input, Output, Both}` (`rubato-5.0.1/src/synchro.rs:39-46`, `53`). Firma: `Fft::new(sample_rate_input, sample_rate_output, chunk_size, nbr_channels, fixed)`. Il parametro `sub_chunks` è stato rimosso e ora vale `(chunk_size/256).max(1)`. `Fft::new_custom(...)` espone `sub_chunks` e la finestra (`synchro.rs:214-232`; migrazione in `README.md:486-494`). `process(&input, Option<&Indexing>)` riceve buffer `audioadapter`, e `Indexing::partial_len` sostituisce `process_partial` (`README.md` v4.0.0, righe ~553-560; `src/lib.rs:93-145`). Per il real-time si usa `process_into_buffer` con buffer pre-allocati (`README.md` "Real-time considerations"). Dipendenze nuove: `audioadapter 5.0` e `audioadapter-buffers 5.1`. La feature di default `fft_resampler` resta (`rubato-5.0.1/Cargo.toml:34-41`). Licenza: da 4.0 `MIT OR Apache-2.0`, prima `MIT` (crates.io API).

**Proposta.** Ci sono due strade, entrambe valide:

(a) Fissare `rubato = "0.16.2"` e riusare `FftFixedIn`, come fa Handy, che è collaudato.
(b) Adottare 5.0.1 con `Fft::<f32>::new(in, 16000, 1024, 1, FixedSync::Input)` e `InterleavedSlice`/`SequentialSlice` di `audioadapter-buffers`.

La pipeline prevista nomina `FftFixedIn`. Con (b) va rinominata.

---

## 3. Handy: VAD (Silero v4 tramite il fork vad-rs) e macchina a stati

### Fatti
- **[V] Dipendenza.** `vad-rs = { git = "https://github.com/cjpais/vad-rs", default-features = false }` (`src-tauri/Cargo.toml:59`). La lock fissa il commit `2a412ed858695b9251f3f5a1a20d95b59fa7c498`, versione 0.1.6 (`src-tauri/Cargo.lock:7581-7589`). Quel commit è il merge della PR #2 "update-ort-rc12" del 2026-03-14 (`git log` del fork).
- **[V] Fork vad-rs.** `ort = "=2.0.0-rc.12"`, `ndarray = "0.17"`. Feature: `default = ["helpers"]` (helpers porta `samplerate` ed `ebur128`, che Handy disattiva), più `directml`, `coreml` e `load-dynamic` (`vad-rs/Cargo.toml`). La sessione ORT usa `GraphOptimizationLevel::Level3`, `intra_threads = 1` e `inter_threads = 1` (`vad-rs/src/session.rs:7-13`). Accetta solo 8000 o 16000 Hz (`vad-rs/src/vad.rs:18-20`).
- **[V] Interfaccia Silero v4.** Input `input`, `sr`, `h`, `c` e output `output`, `hn`, `cn`, con `h`/`c` di forma `(2,1,64)` (`vad-rs/src/vad.rs:22-23`, `41-55`). È la firma del modello v4: il v5 usa un unico tensore `state`. **[I]** Questo fork quindi **non** è compatibile con Silero v5. `reset()` azzera `h` e `c` (`vad.rs:61-64`).
- **[V] Modello incluso in Handy.** `src-tauri/resources/models/silero_vad_v4.onnx`, 1 807 522 byte, SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28` (output di `sha256sum` e `stat`). È identico byte per byte a quello del tag v4.0 di snakers4 (sezione 7). Si risolve come risorsa Tauri `resources/models/silero_vad_v4.onnx` (`src-tauri/src/managers/audio.rs:288-294`).
- **[V] Frame.** 30 ms, cioè 480 campioni a 16 kHz. Frame di dimensione diversa vengono rifiutati (`src-tauri/src/audio_toolkit/vad/silero.rs:9-11`, `34-39`). È speech se `prob > threshold` (`silero.rs:46`). `reset()` azzera lo stato LSTM a ogni sessione (`silero.rs:57-61`).
- **[V] Soglie.** `SILERO_VAD_THRESHOLD = 0.3`. C'è anche un backend alternativo Earshot (pykeio, Rust puro, frame da 256 campioni = 16 ms) con soglia 0.5 (`src-tauri/src/managers/audio.rs:20-21`, `286-304`). Il CLI di debug usa 0.5 (`src-tauri/src/audio_toolkit/bin/cli.rs:181`).
- **[V] Tempi della macchina a stati** (`src-tauri/src/audio_toolkit/vad/mod.rs:5-8`): `VAD_PREFILL_MS = 450`, `VAD_ONSET_MS = 60`, `VAD_OFFLINE_HANGOVER_MS = 450`, `VAD_STREAMING_HANGOVER_MS = 1650`. La conversione in frame arrotonda per eccesso (`mod.rs:12-17`). Con Silero (480 campioni) diventano prefill 15, onset 2, hangover 15 (offline) e 55 (streaming), verificati dal test in `mod.rs:86-92`.
- **[V] `SmoothedVad`** (`src-tauri/src/audio_toolkit/vad/smoothed.rs`):
  - buffer circolare di `prefill_frames + 1` frame (`smoothed.rs:55-64`);
  - silenzio -> voce: incrementa `onset_counter`. Quando raggiunge `onset_frames` entra in speech, imposta `hangover_counter = hangover_frames` ed emette **tutto il buffer** (pre-roll più frame corrente) come un unico `Speech` (`smoothed.rs:74-93`);
  - voce -> voce: ricarica l'hangover ed emette il frame (`smoothed.rs:96-100`);
  - voce -> silenzio: finché `hangover_counter > 0` decrementa ed emette, poi esce da speech (`smoothed.rs:103-112`);
  - silenzio -> silenzio: azzera l'onset (`smoothed.rs:115-118`);
  - l'hangover si cambia per sessione con `set_hangover_frames` (`smoothed.rs:126-128`, `recorder.rs:784-789`).
- **[V] Politica per sessione.** `VadPolicy::{Disabled, Offline, Streaming}` (`recorder.rs:49-58`). Si sceglie `Streaming` se il modello selezionato supporta lo streaming, `Disabled` se il VAD è spento (`src-tauri/src/actions.rs:519-532`).
- **[V] Nessuna durata massima.** In Handy il VAD **non** segmenta in frasi. Filtra i frame di non-parlato ed emette i frame `Speech` in un unico buffer per registrazione (`recorder.rs:633-664`, `872-911`), oppure li inoltra in tempo reale allo stream (`audio_cb`). Una registrazione corrisponde a una trascrizione. La "durata massima frase" della pipeline di Sbobino **non ha equivalente** in Handy e va progettata da zero.

**Proposta.** Si possono riprendere i valori di Handy (soglia 0.3, prefill 450 ms,
onset 60 ms, hangover 450 ms per la frase intera e 1650 ms per lo streaming) come
punto di partenza, più una nuova regola di durata massima. **[I]** Per esempio un
taglio forzato al primo frame non-speech dopo N secondi, o un taglio netto a M
secondi. I valori sono da tarare. Il cambio frase va emesso alla fine
dell'hangover (transizione `in_speech: true -> false`), un punto che `SmoothedVad`
oggi non espone come evento: serve un'API in più. Earshot è un'alternativa senza
ONNX Runtime (vedi la sezione 6 su ORT).

---

## 4. Handy: uso di transcribe-cpp

### Fatti
- **[V] Feature per Windows x64.** `transcribe-cpp = { version = "0.2.4", default-features = false, features = ["dynamic-backends", "vulkan"] }` (`src-tauri/Cargo.toml:144-148`). La dipendenza base è `default-features = false` (`Cargo.toml:82`), perché il default della crate è `metal`. Windows ARM64 linka statico solo CPU (`Cargo.toml:131-151`).
- **[V] Ruolo rispetto a transcribe-rs.** Commento in `Cargo.toml:78-80`: "Whisper-family models run through transcribe-cpp (GGUF/ggml…); transcribe-rs is now ONNX-only". In pratica `EngineType::TranscribeCpp` copre **tutti** i GGUF del catalogo, compresi Parakeet e Nemotron (`src-tauri/src/catalog/mod.rs:80`, `managers/model.rs:28-31`). `EngineType::Parakeet` resta per i vecchi modelli ONNX int8 scaricati da `blob.handy.computer` tramite transcribe-rs (`managers/model.rs:721-788`, `transcription.rs:639-648`).
- **[V] Init backend all'avvio, una volta sola, prima di qualsiasi load.** `transcribe_cpp::init_logging()` e poi `init_backends_default()` (`src-tauri/src/managers/transcription.rs:1884-1897`), chiamati in `lib.rs:210` e `lib.rs:917`. Su Windows x64 emulato su ARM64 la GPU si disabilita (`transcription.rs:1888-1895`).
- **[V] Caricamento del modello.** `Model::load_with(&path, &ModelOptions { backend, device })`, poi `model.session()` con `SessionOptions` di default (`transcription.rs:591-607`). Backend: `Auto` (best device con fallback CPU) oppure `Cpu`. Il device esatto si risolve da `device_id` persistito (`transcription.rs:565-586`, `1979-1995`, `1997-2016`). Dopo il load legge `session.model().capabilities()` e aggiorna le capability del registry (`transcription.rs:612-619`). Prima del nuovo load fa il drop del motore precedente per non tenere in RAM due modelli (`transcription.rs:543-550`).
- **[V] Frase intera.** `session.run(&audio, &RunOptions { task, language, target_language, family, .. })` su tutto il buffer della registrazione (`transcription.rs:1328-1351`). Il prompt iniziale (custom words) viene passato solo se `model.arch() == "whisper"`, perché le altre architetture rifiutano l'estensione whisper con INVALID_ARG (`transcription.rs:1291-1316`). La lingua passata è solo una di quelle in `capabilities().languages`, altrimenti `None` (auto) (`transcription.rs:1741-1767`).
- **[V] Streaming.** Si usa solo se `caps.supports_streaming`. `session.stream(&run_options, &StreamOptions::default())` usa `CommitPolicy::Auto` e nessuna estensione di famiglia (`transcription.rs:969-977`). A ogni frame VAD: `stream.feed(&pcm)`. Se `committed_changed` o `tentative_changed` è vero, emette l'evento `StreamTextEvent { committed, tentative }` (`transcription.rs:989-1006`, `60-67`). Alla fine: `stream.finalize()` e testo `stream.text().full` (`transcription.rs:1015-1045`). Se la finalize fallisce si ricade sulla trascrizione batch (`transcription.rs:1046-1053`). `Cancel` diventa `stream.reset()` (`transcription.rs:1064-1066`).
- **[V] Threading.** Il motore sta in `Arc<Mutex<Option<LoadedEngine>>>` (`transcription.rs:249`). Per lo streaming un thread dedicato **estrae** l'engine dal mutex (lease) e lo restituisce alla fine (`transcription.rs:814-833`, `854-886`). L'audio arriva tramite `StreamRouter` (un `mpsc` di `StreamCmd::{Feed, Finalize, Cancel}`), e l'ordine FIFO garantisce che ogni frame sia processato prima della finalize (`transcription.rs:98-177`). La batch fa lo stesso: prende l'engine, rilascia il lock e usa `catch_unwind` per non avvelenare il mutex (`transcription.rs:1257-1276`, `1295`). C'è un watcher di inattività che scarica il modello (`transcription.rs:302-371`).
- **[V] Gestione di `Busy`: nessuna esplicita.** `Error::Busy` non compare in Handy (`grep "Busy"` trova solo l'enum `BusyAction` del coordinator, che non c'entra). Handy evita `Busy` **per costruzione**: un solo modello, un solo `Session`, e il lease dell'engine esclude una batch concorrente durante lo stream. Il commento in `transcription.rs:854-857` dice: "structurally excluding any concurrent batch transcription (which transcribe-cpp's compute_lock would refuse anyway)".
- **[V] Lingua per i modelli con locale BCP-47 (Nemotron).** `effective_language` risolve un intento "it" nel codice esatto offerto dal modello (es. `it-IT`), e non passa mai "auto" ai modelli senza language detection (test in `managers/model.rs:2695-2712`, logica in `model.rs:290-318`).

---

## 5. Crate `transcribe-cpp` 0.2.4: API

Fonti: sorgente `transcribe-cpp-0.2.4.crate`, docs.rs
`https://docs.rs/transcribe-cpp/0.2.4/transcribe_cpp/` (HTTP 200, 2026-10-02) e
ctx7 `/handy-computer/transcribe.cpp`.

### Formato audio, tipi e thread-safety
- **[V]** PCM **16 kHz mono f32 in [-1, 1]** (`src/session.rs:146`, `src/lib.rs:16`). `Capabilities::native_sample_rate` è esposto (`src/model.rs:52-66`). Esiste l'errore `TRANSCRIBE_ERR_SAMPLE_RATE`, mappato su `InvalidArgument` (`src/error.rs`, `error_for_status`).
- **[V]** `Model` è `Send + Sync`, clonabile e basato su `Arc`. Il modello nativo si libera solo quando sono droppati tutti i `Model` e tutti i `Session` (`src/lib.rs:22-26`, `src/model.rs:69-90`).
- **[V]** `Session` è `Send` ma **non** `Sync`. `run` e `stream` prendono `&mut self` (`src/session.rs:82-85`).
- **[V]** In 0.x la libreria C ammette **al massimo una compute in volo per modello**, su tutte le sessioni. La crate lo impone con un `compute_lock: Mutex<bool>` per modello (`src/lib.rs:29-33`, `src/model.rs:69-77`). Per un vero parallelismo serve un `Model` per worker (`lib.rs:32-33`).
- **[V]** `SessionOptions { n_threads (0 = default di libreria), kv_type, n_ctx }` (`src/model.rs:291-310`). Handy usa i default.

### Modalità 1: frase intera (`run`), per Whisper e Parakeet TDT v3
- **[V]** `Session::run(&mut self, pcm: &[f32], options: &RunOptions) -> Result<Transcript>` (`src/session.rs:150-182`). `RunOptions` ha `task`, `timestamps`, `pnc`, `itn`, `diarize`, `language: Option<String>` (None = autodetect), `target_language`, `keep_special_tags`, `spec_k_drafts` e `family: Option<RunExtension>` (es. `WhisperRunOptions { initial_prompt, .. }`) (`session.rs:28-63`, `src/family.rs:109-112`).
- **[V]** `Transcript` contiene `text`, `raw_text`, `language`, `segments`, `words`, `tokens`, `timings` e `speaker_segments` (`session.rs:300-319`).
- **[V]** `run_batch(&[&[f32]], ..)` restituisce un `Result` per utterance (`session.rs:189-241`).
- **[V] Niente parziali nella modalità run.** L'header C espone solo il callback di log e quello di abort (`transcribe-cpp-sys-0.2.4/include/*.h:397`, `1655`). L'abort si installa con `Session::set_cancel_token(&CancelToken)` (`session.rs:114-127`) e ha effetto solo se il modello supporta `Feature::Cancellation` (`src/types.rs:199-215`). Quindi i parziali "via callback" del trait `TranscriptionEngine` si possono avere **solo** con l'API stream.
- **[V] Limiti di lunghezza.** Whisper e tutta la famiglia parakeet (compreso `nemotron-3.5-asr-streaming-0.6b`) hanno `max_audio_ms = 0`, cioè nessun limite pratico: Whisper finestra internamente a 30 s, Parakeet ha un encoder senza limite (`transcribe.cpp/docs/input-limits.md`, sezione "Chunked / unbounded", letto su HEAD `e85b30e`).

### Modalità 2: stream, per Nemotron
- **[V]** `Session::stream(&mut self, run: &RunOptions, stream: &StreamOptions) -> Result<Stream<'_>>`. Lo `Stream` prende in prestito la sessione per tutta la sua vita (`session.rs:263-296`).
- **[V]** `StreamOptions { commit_policy: CommitPolicy (Auto | OnFinalize | StablePrefix), stable_prefix_agreement_n (0 = 3), family: Option<StreamExtension> }` (`src/streaming.rs:13-23`, `src/types.rs:233-252`). `StreamExtension::ParakeetStream(ParakeetStreamOptions { att_context_right: Option<i32> })` e `ParakeetBuffered { left_ms, chunk_ms, right_ms }` (`src/family.rs:44-54`, `117-122`).
- **[V]** `Stream::feed(&[f32]) -> Result<StreamUpdate>` (`session.rs:524-541`). `StreamUpdate` contiene `committed_changed`, `tentative_changed`, `revision`, `input_received_ms`, `audio_committed_ms`, `buffered_ms` e `is_final` (`streaming.rs:25-44`). Altri metodi: `text() -> StreamText { full, committed, tentative }`, dove `committed` è solo in append e senza flicker (`session.rs:588-593`, `streaming.rs:61-76`), `finalize()` (`session.rs:545-567`), `reset()` (`session.rs:573-585`), `snapshot()`, `state()`, `revision()`, `last_status()` (`session.rs:597-623`).
- **[V] Nemotron 3.5.** L'encoder è cache-aware con 4 latenze addestrate, `att_context_size` [56,0]/[56,3]/[56,6]/[56,13] = 0/240/480/1040 ms di lookahead. Il percorso offline (`run`) usa [56,13]. Lo streaming a R=13 produce un risultato **byte-identico** all'offline. Da CLI: `--stream-chunk-ms 1120 --stream-att-right {0,3,6,13}` (`transcribe.cpp/docs/models/nemotron-3.5-asr-streaming-0.6b.md:24-32`, `181-185`). La lingua si sceglie per chiamata con locale come `it-IT`, e il modo `auto` emette un tag `<ll-RR>` che viene rimosso dal testo (`stesso file:18-22`, `170-176`). Nel codice (HEAD `e85b30e`) un hint vuoto o null va allo slot auto e un hint sconosciuto produce `UNSUPPORTED_LANGUAGE`. Il dizionario contiene sia `en-US` sia l'alias `en` (`transcribe.cpp/src/arch/parakeet/model.cpp:143-170`). WER FLEURS it (Q8_0): 5.78% (`nemotron-...md:96`).

### `Error::Busy`
- **[V] Definizione** (`src/error.rs:76-83`): "Another session of the same model already has a compute operation in flight … at most one run / batch / active stream across ALL of a model's sessions at a time. Finish or drop the in-flight stream (one-shot runs/batches just queue) … or give each concurrent worker its own model." Nasce lato Rust, quindi `raw_status()` vale 0.
- **[V] Quando si verifica.**
  - `run`/`run_batch` restituiscono `Busy` **solo** se c'è uno stream attivo su quel modello (`session.rs:157-167`, `202-212`). Se c'è un altro `run` in corso, aspettano sul mutex (fanno coda).
  - `stream()` restituisce `Busy` se c'è già uno stream attivo (`session.rs:276-285`). Il lease si prende a `stream_begin` e si rilascia a `finalize`, a `reset` o al drop dello `Stream` (`session.rs:290`, `503-520`, `559-564`, `581-584`).
  - ctx7 (README dei binding Swift e TypeScript) lo conferma: "An active `Stream` holds the model's compute lease until `finalize`, `reset`, or drop. Other runs/streams on that model fail with … busy".
- **[V] Come lo evita Handy.** Per costruzione, con un solo motore in lease (sezione 4).
- **Proposta** per Sbobino, che ha due sorgenti (microfono e loopback) e un trait "una frase per chiamata":
  - (a) **un solo worker di trascrizione** con coda di frasi `(sorgente, pcm)`. Le chiamate `run` fanno già coda sul `compute_lock`, quindi niente `Busy`. Per Nemotron però si perdono i parziali (lo stream è uno solo alla volta);
  - (b) **un `Model` per sorgente**: doppia RAM/VRAM (circa 2 x 560 MB in Q5_K_M per Nemotron), ma due stream in parallelo;
  - (c) stream per frase su un unico modello, con `finalize` alla fine di ogni frase. Se l'altra sorgente prova a trascrivere mentre lo stream è aperto riceve `Busy`. Serve quindi un retry o una serializzazione a livello di frase.

  **[I]** In ogni caso conviene mappare `Busy` su un errore riprovabile del trait.

### Feature flag (crate `transcribe-cpp` e `-sys`)
- **[V]** `default = ["metal"]`. Le altre feature sono `vulkan`, `cuda`, `rocm`, `openmp`, `shared` e `dynamic-backends` (implica `shared` e la sys `dynamic-backends`) (`transcribe-cpp-0.2.4/Cargo.toml`, sezione `[features]`). Corrispondenza con le opzioni CMake: `shared` -> `TRANSCRIBE_BUILD_SHARED=ON`, `dynamic-backends` -> in più `TRANSCRIBE_GGML_BACKEND_DL=ON` e, su x86, `GGML_CPU_ALL_VARIANTS=ON` + `TRANSCRIBE_X86_CONSERVATIVE=ON`, `vulkan` -> `TRANSCRIBE_VULKAN=ON` (`transcribe-cpp-sys-0.2.4/bindings/rust/sys/build.rs:16-33`, `157-168`).
- **[V] OpenMP disattivato per default su tutte le piattaforme in 0.2.4.** Si attiva solo con la feature `openmp` (`build.rs:197-216`). Nota: un commento in `Handy/.github/workflows/build.yml:291-296` dice ancora che la sys "force-sets GGML_OPENMP=ON on Windows", e Handy impacchetta `vcomp140.dll` (`build.rs` di Handy, righe 76-80). **[I]** È un retaggio di versioni precedenti, da verificare.
- **[V] Uso statico o dinamico.** Con il build statico di default non c'è nulla da distribuire. Con `shared`/`dynamic-backends` la crate espone `DEP_TRANSCRIBE_CPP_RUNTIME_DIR` e `DEP_TRANSCRIBE_CPP_MODULE_DIR` al `build.rs` del consumer (`transcribe-cpp-0.2.4/README.md`, sezione "Packaging a distributable"; sys `build.rs:488-507`). In un build `dynamic-backends`, `init_backends_default()` cerca i moduli nella cartella della libtranscribe caricata (`src/backend.rs:138-157`).

### Requisiti di build su Windows
- **[V]** Servono un toolchain C++ e **CMake**. libclang non serve, perché i binding generati sono committati. Nessuna dipendenza zlib (`transcribe-cpp-sys/bindings/rust/sys/README.md`, "Build prerequisites").
- **[V]** Con la feature `vulkan` serve il **Vulkan SDK**, con `VULKAN_SDK` nel PATH di un terminale nuovo (stesso README, "Windows Vulkan builds"; Handy `BUILD.md:45-53`, `winget install KhronosGroup.VulkanSDK`). La CI di Handy usa Vulkan SDK `1.4.309.0` (`.github/workflows/build.yml:156-164`). Aggiunge anche `spirv-headers` via vcpkg in `CMAKE_PREFIX_PATH`, perché `find_package(SPIRV-Headers CONFIG REQUIRED)` falliva con l'SDK della CI (`build.yml:426-447`). **[I]** Con un SDK installato in locale potrebbe non servire.
- **[V] Junction `%LOCALAPPDATA%\tcs`.** Il build nativo passa da una junction NTFS corta `%LOCALAPPDATA%\tcs\<hash FNV-1a di OUT_DIR>` che punta a `OUT_DIR`, creata con `cmd /C mklink /J` senza diritti di admin (`build.rs:235-249`, `257-347`). Se `LOCALAPPDATA` manca usa `TEMP`. Se la creazione fallisce stampa un warning e compila nell'`OUT_DIR` profondo. La soluzione di ripiego è `CARGO_TARGET_DIR` corto, es. `C:\tc-target` (sys README; Handy `BUILD.md:234-262`). Il motivo: `LongPathsEnabled` non aiuta perché il FileTracker di MSBuild lo ignora (FTK1011) (`build.rs:257-261`, `BUILD.md:236-240`).
- **[V]** Su MSVC la sys forza `/O2 /Ob2 /DNDEBUG`, altrimenti ggml verrebbe compilato senza ottimizzazioni (`build.rs:135-151`).
- **[I] Build concorrenti.** La junction dipende solo da `OUT_DIR`. Due cargo sulla stessa target dir sono già serializzati dal lock di cargo, quindi il problema segnalato riguarda probabilmente build CMake parallele lanciate da processi diversi (es. rust-analyzer e un terminale) sullo stesso albero di build. Non verificato. Coerente con la cautela "una build alla volta".

### Licenza
- **[V]** transcribe-cpp e -sys: **MIT** (`Cargo.toml` `license = "MIT"`; repo `LICENSE`: "Copyright (c) 2026 The transcribe.cpp authors"). Componenti vendorizzati: **ggml (MIT)** e **miniz (MIT)** (`transcribe.cpp/THIRD-PARTY-LICENSES.md:1-15`).

---

## 6. Handy: download dei modelli, build e bundle, struttura Tauri

### Download dei modelli
- **[V] Dove sono definiti.** `src-tauri/src/catalog/catalog.json` (catalog_version 2, generato il 2026-08-17 da `scripts/gen_catalog.py` a partire dall'org HF `handy-computer`) è compilato nel binario con `include_str!` (`src-tauri/src/catalog/mod.rs:1-14`, `112-116`). Per ogni modello ci sono `id` (repo HF), `revision` (commit fissato), `files[] { filename, quant, size_bytes, sha256 }` e `default_quant` (`mod.rs:36-61`). Il default di Handy è **Q8_0**, non Q5_K_M (vedi la sezione 8).
- **[V] URL.** HF a revision fissata ("`resolve/<sha>` is immutable", `mod.rs:80-84`). Il mirror di fallback è `{mirror}/{repo_id}/{revision}/{filename}` con `mirrors = ["https://blob.handy.computer"]` (`mod.rs:26-34`, `143-170`). Il mirror si usa solo se c'è uno SHA-256, perché senza hash niente host non fidato (`mod.rs:155-159`).
- **[V] Trasporto HF.** Fork `hf-hub` (`cjpais/hf-hub`, branch `cancellable-downloads`) (`Cargo.toml:90-92`), cache HF condivisa (`HF_HOME` o `~/.cache/huggingface/hub`) (`managers/model.rs:328-333`). Fino a 4 tentativi con stream `[4,1,1,1]`, watchdog di stallo a 60 s e ripresa dal marker `.sync.part` (`model.rs:1945-1960` circa e `download.rs:22-26`). **[I]** Su questo percorso non ho trovato un ricalcolo SHA-256 esplicito da parte di Handy: `verify_sha256` si chiama solo nel downloader HTTP (`grep compute_sha256|verify_sha256`).
- **[V] Downloader HTTP con ripresa** (mirror e modelli via URL), in `src-tauri/src/managers/model/download.rs:184-380`:
  - file temporaneo `<models_dir>/<filename>.partial` e rename atomico finale (`model.rs:2169`, `2186-2188`);
  - `models_dir = app_data_dir/models` (`model.rs:534-536`);
  - ripresa con `Range: bytes=<n>-`. Un 200 a una Range riparte da zero, un 206 deve partire esattamente all'offset, su 416 si cancella (`download.rs:222-280`);
  - un partial già completo viene verificato senza nuova richiesta, uno troppo grande viene cancellato (`download.rs:193-208`);
  - controlla `Content-Length` contro la dimensione del catalogo e taglia al primo byte in eccesso (`download.rs:281-291`, `339-352`);
  - timeout di connessione 15 s e di stallo 60 s, sempre in corsa con il token di cancel (`download.rs:19-26`, `226-233`, `324-338`);
  - SHA-256 a blocchi di 64 KB in `spawn_blocking`. Se non combacia **cancella il partial** (`download.rs:52-120`);
  - progress con eventi `model-download-progress` (`DownloadProgress { model_id, downloaded, total, percentage }`, al massimo 10 al secondo), `model-verification-started`/`-completed` e `model-download-complete` (`download.rs:122-157`, `305-321`; `model.rs:320-326`, `2150-2153`).

### Build e bundle per Windows
- **[V] `build.rs` di Handy** (`src-tauri/build.rs:1-38`):
  - `stage_transcribe_runtime_libs()` copia le DLL da `DEP_TRANSCRIBE_CPP_RUNTIME_DIR` e `..._MODULE_DIR` in `src-tauri/transcribe-libs/`. Ricrea la cartella pulita e fallisce se è vuota, con il commento "registers zero compute devices" (`build.rs:149-261`);
  - `stage_onnxruntime_dll()` copia `onnxruntime.dll` da `ORT_LIB_LOCATION` se è impostato `ORT_PREFER_DYNAMIC_LINK` (`build.rs:105-147`);
  - `stage_vc_runtime_dlls()` copia `msvcp140*`, `vcruntime140*` e `vcomp140*` da `HANDY_VC_REDIST_DIRS` (issue #1527) (`build.rs:40-103`).
- **[V] Bundle.** `tauri.windows.conf.json` mappa `"resources": { "resources": "resources", "transcribe-libs": "." }`, così le DLL finiscono accanto all'exe (`src-tauri/tauri.windows.conf.json:1-8`). `tauri.conf.json` include `resources/**/*` (dentro c'è `silero_vad_v4.onnx`) e un template NSIS custom `nsis/installer.nsi` con modalità portable (`tauri.conf.json:30`, `72-77`; `nsis/installer.nsi:1-6`). Nel template NSIS non c'è niente di specifico per DLL, Vulkan o ONNX (`grep` vuoto).
- **[V] DirectML / `ort`: rimosso.** Commento in `Cargo.toml:108-115`: "ONNX Runtime on Windows is CPU-only (no ort-directml)". La build prebuilt di pyke è compilata con `/arch:AVX2` e "crashes at process startup on any pre-Haswell CPU". In CI Handy linka **dinamicamente** l'ONNX Runtime ufficiale Microsoft 1.24.2 (`onnxruntime-win-x64-1.24.2.zip` dalle release GitHub, impostando `ORT_LIB_LOCATION` e `ORT_PREFER_DYNAMIC_LINK=1`) (`.github/workflows/build.yml:401-424`). Quindi Handy **non** distribuisce `DirectML.dll`. Il fork vad-rs ha ancora una feature `directml = ["ort/directml"]` (`vad-rs/Cargo.toml`). Feature di default di `ort` rc.12: `std, ndarray, tracing, download-binaries, tls-native, copy-dylibs, api-24` (crates.io API). **[I]** Usando vad-rs con i default di `ort`, Sbobino erediterebbe la build prebuilt AVX2 e il relativo rischio di crash all'avvio su CPU pre-Haswell.
- **[V] `dynamic-backends`.** Ha senso su x86 per `GGML_CPU_ALL_VARIANTS`, cioè moduli CPU per ISA scelti a runtime, più `ggml-vulkan` come modulo (`Cargo.toml:131-148`).
- **[V] Long path / CI.** Ora sono solo margine extra: dalla 0.1.3 la sys usa la junction (`build.yml:59-72`).

### Struttura Tauri
- **[V] Manager** in `src-tauri/src/managers/`: `audio`, `gguf_meta`, `history`, `model`, `model_capabilities` e `transcription` (`managers/mod.rs`). Si creano in `lib.rs:195-208` e si registrano con `app_handle.manage(...)` (`lib.rs:216-220`). Il `StreamRouter` del transcription manager viene passato all'`AudioRecordingManager` (`lib.rs:203-205`).
- **[V] Comandi** in `src-tauri/src/commands/{audio,history,models,transcription,mod}.rs`, con `#[tauri::command] #[specta::specta]` (es. `commands/transcription.rs:13-40`).
- **[V] tauri-specta.** `specta = "=2.0.0-rc.22"`, `specta-typescript = "0.0.9"`, `tauri-specta = { version = "=2.0.0-rc.21", features = ["derive","typescript"] }` (`Cargo.toml:86-88`). `Builder::<tauri::Wry>::new().commands(collect_commands![...]).events(collect_events![HistoryUpdatePayload, StreamTextEvent, StreamPhaseEvent])` (`lib.rs:649-772`). Export TypeScript in `../src/bindings.ts` **solo in debug** (`lib.rs:774-780`). Montaggio con `specta_builder.mount_events(app)` (`lib.rs:897`). Gli eventi tipizzati derivano `tauri_specta::Event` (`transcription.rs:63-67`, `90-96`). Altri eventi (es. `model-state-changed`, `model-download-progress`) usano `app_handle.emit` non tipizzato.

---

## 7. Silero VAD v4 (snakers4/silero-vad, tag v4.0)

| Campo | Valore | Fonte |
| --- | --- | --- |
| Tag | `v4.0`: oggetto tag `7a176cc294a2c40615458e50895ed9703782638d`, commit `915dd3d639b8333a52e001af095f87c5b7f1e0ac` | `api.github.com/repos/snakers4/silero-vad/git/refs/tags/v4.0` e `/git/tags/7a176cc…` **[V]** |
| Percorso nel repo | `files/silero_vad.onnx` (blob git `e6db48d6e2a0797a2ec173c008384f7710189344`) | `api.github.com/.../contents/files?ref=v4.0` **[V]** |
| URL raw al tag | `https://raw.githubusercontent.com/snakers4/silero-vad/v4.0/files/silero_vad.onnx` | scaricato **[V]** |
| URL fissato al commit | `https://raw.githubusercontent.com/snakers4/silero-vad/915dd3d639b8333a52e001af095f87c5b7f1e0ac/files/silero_vad.onnx` | scaricato, stesso hash **[V]** |
| SHA-256 | `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28` | `sha256sum` **[V]** (uguale al file di Handy) |
| Dimensione | 1 807 522 byte | `stat` e API contents **[V]** |
| Licenza | MIT ("Copyright (c) 2020-present Silero Team") | `raw.githubusercontent.com/snakers4/silero-vad/v4.0/LICENSE` e API license **[V]** |

Nota: Handy lo rinomina `silero_vad_v4.onnx`. Il file è piccolo e MIT.
**Proposta:** includerlo nelle risorse invece di scaricarlo, come fa Handy.

---

## 8. Modelli GGUF Q5_K_M (org HF `handy-computer`)

Verifica fatta con `GET https://huggingface.co/api/models/handy-computer/<repo>/tree/<revision>`
(`lfs.oid` = SHA-256, `size`) e `HEAD .../resolve/<revision>/<file>`. Le risposte
HEAD hanno restituito `302`, `X-Repo-Commit` uguale alla revision,
`X-Linked-Size` e `X-Linked-ETag` uguali allo SHA-256, `Accept-Ranges: bytes`
(quindi la ripresa è possibile). I tre valori coincidono con `catalog.json` di
Handy (righe 64, 229, 2098) e, a oggi, anche con `main`.

| Modello | Repo / revision fissata | File | Byte | SHA-256 | Modalità | Licenza |
| --- | --- | --- | ---: | --- | --- | --- |
| Nemotron Streaming 3.5 0.6B | `handy-computer/nemotron-3.5-asr-streaming-0.6b-gguf` @ `6d44e540bc31b0de1dbe174a3cea87f53a7f22fb` (main oggi: `8139c4ec14bdc45c361adf8d57c27c28e7478272`) | `nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf` | 559 647 200 | `86429e8c4f7fdcf9b3312269ad1ca6669478ba7805331c4aea7a2e33e9910d65` | stream (anche `run` offline) | `other` = **OpenMDW-1.1** (card HF di handy-computer e di `nvidia/nemotron-3.5-asr-streaming-0.6b`) |
| Whisper Large v3 Turbo | `handy-computer/whisper-large-v3-turbo-gguf` @ `5eaf945c7978e564bae5b28a5b1639dd93c2bfb1` (main: `ceea6c8a94a21ab85be244d311e874a39344dbf5`) | `whisper-large-v3-turbo-Q5_K_M.gguf` | 619 628 128 | `977b5db4e004349dffd1ab9caa10ba5aaba3fc3edd3ba72cadb84328a3203e36` | frase | card handy-computer: **apache-2.0**; upstream `openai/whisper-large-v3-turbo`: **mit** (discrepanza, vedi incertezze) |
| Parakeet TDT v3 0.6B | `handy-computer/parakeet-tdt-0.6b-v3-gguf` @ `85ac09ea12fc4b1112fa76810059364bc6adc9de` (main: `90f082450fcbacdb54e5900c44ef697c9ea59622`) | `parakeet-tdt-0.6b-v3-Q5_K_M.gguf` | 548 946 272 | `cc722e76adc1a629fc0b2535de879d99b8160d07ad4c0215e2ca7d7ea0ae4b8f` | frase | **cc-by-4.0** (handy-computer e `nvidia/parakeet-tdt-0.6b-v3`) |

Lingue (catalog di Handy): tutti e tre includono l'italiano. Nemotron 28 lingue,
Parakeet v3 25 lingue europee, Whisper Turbo 100 (`catalog.json`; script
`python` sul file). Capability dal catalog: Nemotron `streaming: true,
lang_detect: true`, Parakeet v3 `streaming: false, lang_detect: true`, Whisper
Turbo `streaming: false, translate: false, lang_detect: true`.

**Proposta: contenuto di `models.json`** (URL fissati al commit, quindi immutabili):

```json
{
  "version": 1,
  "models": [
    {
      "id": "nemotron-3.5-streaming-0.6b-q5km",
      "nome": "Nemotron Streaming 3.5 0.6B (Q5_K_M)",
      "url": "https://huggingface.co/handy-computer/nemotron-3.5-asr-streaming-0.6b-gguf/resolve/6d44e540bc31b0de1dbe174a3cea87f53a7f22fb/nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf",
      "sha256": "86429e8c4f7fdcf9b3312269ad1ca6669478ba7805331c4aea7a2e33e9910d65",
      "size": 559647200,
      "modalita": "stream",
      "licenza": "OpenMDW-1.1"
    },
    {
      "id": "whisper-large-v3-turbo-q5km",
      "nome": "Whisper Large v3 Turbo (Q5_K_M)",
      "url": "https://huggingface.co/handy-computer/whisper-large-v3-turbo-gguf/resolve/5eaf945c7978e564bae5b28a5b1639dd93c2bfb1/whisper-large-v3-turbo-Q5_K_M.gguf",
      "sha256": "977b5db4e004349dffd1ab9caa10ba5aaba3fc3edd3ba72cadb84328a3203e36",
      "size": 619628128,
      "modalita": "frase",
      "licenza": "MIT (upstream OpenAI) / apache-2.0 (card GGUF)"
    },
    {
      "id": "parakeet-tdt-0.6b-v3-q5km",
      "nome": "Parakeet TDT v3 0.6B (Q5_K_M)",
      "url": "https://huggingface.co/handy-computer/parakeet-tdt-0.6b-v3-gguf/resolve/85ac09ea12fc4b1112fa76810059364bc6adc9de/parakeet-tdt-0.6b-v3-Q5_K_M.gguf",
      "sha256": "cc722e76adc1a629fc0b2535de879d99b8160d07ad4c0215e2ca7d7ea0ae4b8f",
      "size": 548946272,
      "modalita": "frase",
      "licenza": "CC-BY-4.0"
    }
  ]
}
```

**[I]** Il campo `licenza` è facoltativo ma utile per mostrare l'attribuzione
nell'interfaccia. Per Nemotron, la modalità "stream" non esclude `run`: è
byte-identico a R=13 (sezione 5).

---

## 9. Licenze rilevanti per la distribuzione

| Componente | Licenza | Obblighi principali | Fonte |
| --- | --- | --- | --- |
| Handy (solo come riferimento di idee) | MIT, "Copyright (c) 2025 CJ Pais" | Se si copia codice: mantenere copyright e licenza. Nome, logo e asset sono esclusi per scelta del progetto | `Handy/LICENSE`, GitHub API **[V]** |
| transcribe-cpp / transcribe-cpp-sys | MIT | Includere licenza e notice | crates.io, `transcribe.cpp/LICENSE` **[V]** |
| ggml, miniz (vendorizzati in -sys) | MIT, MIT | Includere i testi (la sys li porta con sé) | `transcribe.cpp/THIRD-PARTY-LICENSES.md:8-11` **[V]** |
| vad-rs (fork cjpais e upstream thewh1teagle) | MIT dichiarata in `Cargo.toml`. Nessun file `LICENSE` nel fork, e l'API GitHub non rileva una licenza né sul fork né sull'upstream | Includere la notice MIT. **[I]** Manca un testo con titolare del copyright | `vad-rs/Cargo.toml`, `api.github.com/repos/{cjpais,thewh1teagle}/vad-rs` **[V]** |
| ort (pyke) | MIT OR Apache-2.0 | Licenza e notice | crates.io **[V]** |
| ONNX Runtime (Microsoft) | MIT | Includere `LICENSE` e i ThirdPartyNotices dello zip | `raw.githubusercontent.com/microsoft/onnxruntime/main/LICENSE` **[V]**; ThirdPartyNotices **[I]** non verificati |
| rubato | 0.16.2: MIT; da 4.0: MIT OR Apache-2.0 | Licenza | crates.io **[V]** |
| cpal | Apache-2.0 | Licenza e NOTICE se presente | crates.io **[V]** |
| earshot (alternativa VAD) | MIT OR Apache-2.0 | Licenza | crates.io **[V]** |
| Silero VAD v4 | MIT | Includere la licenza | sezione 7 **[V]** |
| Nemotron 3.5 ASR Streaming | OpenMDW-1.1 | Chi distribuisce i Model Materials deve conservare una copia dell'accordo e le notice di origine. La licenza decade se si avvia una causa di brevetto o copyright sui materiali. Nessun vincolo sugli output | `https://openmdw.ai/license/1-1/` (testo letto) **[V]** |
| Parakeet TDT 0.6B v3 | CC-BY-4.0 | Attribuzione (nome, link alla licenza, indicazione delle modifiche, qui la conversione GGUF/quantizzazione) | card HF **[V]** |
| Whisper Large v3 Turbo | upstream MIT (OpenAI). La card GGUF dice apache-2.0 | Includere la licenza | card HF **[V]** |

**[I]** Se i modelli si scaricano al primo avvio e non stanno nell'installer, chi
"distribuisce" è Hugging Face/handy-computer. Mostrare comunque le licenze
nell'interfaccia è prudente. Per Nemotron, OpenMDW lega gli obblighi alla
distribuzione.

---

## 10. Idee da Handy riutilizzabili (proposte, nessuna decisione)

1. Callback cpal senza allocazioni, ring SPSC `rtrb` da 2 s, downmix nel callback, consumer con polling a 10 ms (sezione 1).
2. Usare la frequenza nativa del dispositivo e ricampionare dopo, invece di forzare 16 kHz su WASAPI (sezione 1).
3. `FrameResampler`: chunk da 1024, frame fissi per il VAD, svuotamento corretto rispetto a `output_delay()`, `reset()` tra sessioni (sezione 2).
4. `SmoothedVad` con prefill, onset e hangover espressi in ms e convertiti in frame (sezione 3). Per Sbobino serve in più l'evento "fine frase" e la durata massima.
5. Init dei backend una volta prima di ogni load. Lease del motore in un thread dedicato, `catch_unwind` attorno alle chiamate native (sezione 4).
6. Downloader con `.partial`, Range e ripresa, verifica di dimensione e SHA-256, eventi di progresso limitati a 10 al secondo, URL fissati a revision (sezione 6).
7. `build.rs` che copia le DLL da `DEP_TRANSCRIBE_CPP_RUNTIME_DIR`/`MODULE_DIR` in una cartella mappata da `tauri.windows.conf.json` su `"."` (sezione 6).
8. tauri-specta per comandi ed eventi tipizzati, export dei binding solo in debug (sezione 6).

---

## 11. Incertezze e punti aperti

1. **Loopback cpal 0.16.** Il flag `AUDCLNT_STREAMFLAGS_LOOPBACK` l'ho verificato solo sul sorgente cpal 0.18.2. Non so se Sbobino resterà su 0.16 come Handy o passerà a 0.18 (breaking change in sezione 1).
2. **Allineamento microfono/loopback.** Tutti e due i timestamp `capture` vengono da QPC su WASAPI. Che i due stream siano davvero confrontabili è un'inferenza: cpal non lo garantisce tra stream diversi (`timestamp.rs:10-13`).
3. **Verifica SHA sul percorso hf-hub di Handy.** Non ho trovato un ricalcolo SHA-256 esplicito. Il fork `cjpais/hf-hub` potrebbe verificarlo internamente (non letto).
4. **OpenMP / `vcomp140.dll`.** La sys 0.2.4 tiene OpenMP spento per default, ma la CI di Handy dice il contrario e impacchetta `vcomp140.dll`. Da chiarire se a Sbobino serve.
5. **SPIRV-Headers via vcpkg.** Necessario in CI ad Handy con l'SDK 1.4.309.0. Con un SDK LunarG locale completo potrebbe non servire. Non verificato perché non ho eseguito build.
6. **Licenza di Whisper Turbo.** La card GGUF dice apache-2.0, l'upstream OpenAI dice MIT. Conviene citare la licenza upstream.
7. **vad-rs senza file LICENSE.** Licenza dichiarata solo in `Cargo.toml`, nessun titolare del copyright esplicito.
8. **Docs del repo transcribe.cpp.** Le note su Nemotron e `input-limits.md` le ho lette su HEAD `e85b30e`, posteriore a v0.2.4. Per esempio `input-limits.md` cita `TRANSCRIBE_ERR_OUTPUT_REPETITION`, che non ha una variante dedicata nell'enum `Error` della crate 0.2.4 (finirebbe in `Error::Other`). Il comportamento esatto in 0.2.4 va confermato sul tag `v0.2.4` (`4807eda`).
9. **Opzioni di streaming per Nemotron.** Handy usa `StreamOptions::default()`, senza `ParakeetStreamOptions { att_context_right }`. Non ho verificato quale R e quale chunk applichi il default della libreria (la CLI documenta 1120 ms e R in {0,3,6,13}).
10. **Codici lingua.** Il catalog di Handy elenca `it`, mentre la doc di Nemotron usa locale come `it-IT`. Il codice C++ (HEAD) accetta entrambi perché ha alias. Conviene leggere `capabilities().languages` a runtime.
11. **Concorrenza nel build CMake di transcribe-cpp-sys.** Il motivo per cui build concorrenti non funzionano è un'inferenza (junction per `OUT_DIR`). La regola pratica resta "una build alla volta".
12. **Rischio AVX2 di `ort` con vad-rs.** È dedotto dal commento di Handy e dalle feature di default di `ort`. Non l'ho testato.
