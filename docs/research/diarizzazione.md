# Ricerca: diarizzazione opzionale (chi parla quando)

Data: 2026-10-03. Scopo: capire se e come Sbobino può aggiungere, come opzione, le etichette dei
parlanti alla Trascrizione. Si guarda come fa Handy, cosa offre `transcribe-cpp` 0.2.4 e quali
alternative esistono in Rust/ONNX. Il documento non prende decisioni: le proposte sono marcate
**Proposta** e la scelta spetta all'utente.

Legenda: **[V]** = verificato su fonte primaria (file e riga, URL o output di comando).
**[I]** = inferenza mia, da confermare.

## 0. Fonti e versioni

| Fonte | Riferimento |
| --- | --- |
| Handy (clone) | `5ec58f696354fcf64ae831102e673779e0249717`. Confrontato con l'HEAD upstream `ffbc950` ("transcribe 0.3.0 (#2203)", 2026-10-03) |
| `transcribe-cpp` / `-sys` | crate 0.2.4, sorgente `.crate` nel registry cargo locale |
| Repo transcribe.cpp | `https://github.com/handy-computer/transcribe.cpp`, tag `v0.2.4` → `4807eda`, tag `v0.3.0` → `077110e` (2026-10-03) |
| Crate alternativi | crates.io API (`/api/v1/crates/<nome>`), sorgenti `.crate` di `speakrs` 0.5.0 e `sherpa-onnx-sys` 1.13.8 |
| Modelli | API Hugging Face (`/api/models/<repo>` e `/tree/main`), release GitHub di k2-fsa/sherpa-onnx, model card |
| ctx7 | `/websites/rs_sherpa-onnx_sherpa_onnx` (API Rust di sherpa-onnx) |

Path abbreviati: `Handy/…` = root del clone di Handy. `tc/…` = crate `transcribe-cpp-0.2.4`.
`tcs/…` = crate `transcribe-cpp-sys-0.2.4`. `sb/…` = `D:\local\tauri\sbobino`.

---

## 1. Handy

### Fatti
- **[V] Handy non ha la diarizzazione.** Non imposta mai `RunOptions.diarize`. Le due
  costruzioni di `RunOptions` passano solo `task`, `language`, `target_language` e, per Whisper,
  `family`. Il resto è `..Default::default()`, quindi `Diarize::Default`, cioè spento
  (`Handy/src-tauri/src/managers/transcription.rs:945-950`, `:1328-1334`). Nel codice Rust e TS
  non compaiono `speaker_segments`, `Feature::Diarization` né UI di etichette o rinomina dei
  parlanti (grep di `diariz|speaker` su `src/` e `src-tauri/src/`).
- **[V] Il catalogo contiene un modello che diarizza, ma Handy lo usa come trascrittore puro.**
  `MOSS-Transcribe-Diarize 0.9B` (`handy-computer/moss-transcribe-diarize-gguf`, architettura
  `moss`, licenza Apache-2.0, lingue `["en","zh"]`, `streaming: false`, timestamp `segment`,
  default Q8_0 da 986.899.616 byte, SHA256 nel catalogo)
  (`Handy/src-tauri/src/catalog/catalog.json:1343-1374`). Con diarize spento `transcribe-cpp`
  toglie i marcatori dei parlanti dal testo (§2).
- **[V] Sortformer è escluso apposta.** Un test impone che nessun modello del catalogo abbia
  architettura `sortformer`: "Sortformer produces speaker segments, not transcription text"
  (`Handy/src-tauri/src/catalog/mod.rs:234-242`). Però `moss` e `sortformer` sono nell'elenco
  delle architetture riconosciute (`Handy/src-tauri/src/managers/model_capabilities.rs:29-51`,
  righe 49-50). Così un GGUF Sortformer importato a mano risulta "compatibile", ma Handy non ne
  legge l'output.
- **[V] Granite Speech 4.1 2B Plus** (che può diarizzare, §2) è in catalogo con lingue
  `en, fr, de, es, pt`, senza italiano (`catalog.json:768-794`). Anche per lui Handy non attiva
  diarize.
- **[V] HEAD upstream.** Dopo `5ec58f6` ci sono 6 commit. L'ultimo porta `transcribe-cpp` a 0.3.0
  e cambia solo `Cargo.toml` e `Cargo.lock`. Su `origin/HEAD` il grep dà gli stessi risultati:
  ancora nessuna diarizzazione.
- **[V] transcribe.cpp 0.3.0** non cambia la diarizzazione. Il diff `v0.2.4..v0.3.0` non tocca
  `src/arch/sortformer`, `include/transcribe/sortformer.h` né `multitalker.cpp`, e in
  `include/transcribe.h` non cambia nessuna riga con `diar` o `speaker`.

### Cosa se ne ricava
Da Handy non c'è niente da copiare per la diarizzazione. L'unica indicazione utile è indiretta.
Handy considera Sortformer "non un modello di trascrizione" e lo tiene fuori dal catalogo dei
modelli di trascrizione. Se Sbobino lo adotta, deve gestirlo come un modello accessorio, separato
dalla scelta del modello di trascrizione **[I]**.

---

## 2. `transcribe-cpp` 0.2.4

### API Rust
- **[V]** `RunOptions.diarize: Diarize` (`tc/src/session.rs:35`, default `Diarize::Default` a
  `:55`). `Diarize::{Default, Off, On}`: "Library default: speaker attribution is disabled for
  every family" (`tc/src/types.rs:128-147`).
- **[V]** `Feature::Diarization` esiste: "Produces structured speaker attribution"
  (`tc/src/types.rs:212-213`), e si interroga con `Model::supports` (`tc/src/model.rs:196`).
- **[V]** `Transcript.speaker_segments: Vec<SpeakerSegment>`, "empty unless diarization was
  enabled and found" (`tc/src/result.rs:32-33`). `Segment.speaker_id: i32` è "1-based speaker id;
  zero means no attribution" (`result.rs:52-53`).
- **[V]** `SpeakerSegment { t0_ms: i64, t1_ms: i64, speaker_id: i32, p: f32 }`. Tempi zero
  significano "attribuito ma senza tempi". `p` vale NaN se non disponibile (`result.rs:56-63`).
- **[V]** Estensione Sortformer: `RunExtension::Sortformer(SortformerStreamOptions { preset })`
  con `SortformerPreset::{Default, VeryHighLatency, HighLatency, LowLatency}`
  (`tc/src/family.rs:63-114`), esportate in `tc/src/lib.rs:66-79`. Esiste solo nello slot RUN:
  `StreamExtension` non ha una variante Sortformer (`family.rs:116-123`).

### Semantica C
- **[V]** `diarize` ON: "the model's speaker markers are parsed into structured results". Con un
  modello senza la feature, un valore non DEFAULT emette un WARN e il run procede come al solito
  (`tcs/include/transcribe.h:536-569`, `:1337-1347`; controllo in `tcs/src/transcribe.cpp:1413-1420`).
- **[V]** `transcribe_raw_text` conserva i marcatori grezzi (moss `[0.48][S01]`, granite
  `[Speaker N]:`). `full_text` è sempre pulito (`transcribe.h:2346-2357`).

### Quali modelli diarizzano (0.2.4)

| Famiglia / modello | Come | Lingue | Note | Fonte |
| --- | --- | --- | --- | --- |
| **sortformer** (`diar_streaming_sortformer_4spk-v2.1`) | Solo diarizzazione, nessun testo. DIARIZATION è un invariante della famiglia | catalogo: `en` (vedi §2.4) | max 4 parlanti | `tcs/src/arch/sortformer/capabilities.cpp:12-33` |
| **moss** (MOSS-Transcribe-Diarize 0.9B) | Il modello emette sempre i marcatori; diarize ON li analizza | en, zh | 0,99 GB Q8_0, solo `run` | `tcs/src/arch/moss/capabilities.cpp:12-22` |
| **granite** (solo `granite-speech-4.1-2b-plus`) | ON cambia il prompt (task SAA) | en, fr, de, es, pt | incompatibile con timestamp `word` nello stesso run | `tcs/src/arch/granite/model.cpp:255-264`, `:542-554` |
| **parakeet** solo come bundle multitalker (`multitalker-parakeet-streaming-0.6b-v1`, con Sortformer incorporato) | ON passa all'orchestratore multitalker: una decodifica per parlante attivo | en | solo `run`; `run_batch` e stream restano a parlante singolo | `tcs/src/arch/parakeet/model.cpp:551-570`, `:595-617`, `:1364-1370`, `:1652-1658`, `:2897-2903`; `multitalker.cpp:1-34` |

- **[V] I tre modelli di Sbobino non diarizzano.** Nel catalogo di transcribe.cpp `v0.2.4`,
  `nemotron-3.5-asr-streaming-0.6b`, `whisper-large-v3-turbo` e `parakeet-tdt-0.6b-v3` hanno
  `"diarize": {"supported": false}`. Per Parakeet la feature si accende solo se il GGUF contiene
  `stt.parakeet.diarizer.embedded` (`parakeet/model.cpp:557-569`, `:617`). Whisper non supporta
  tinydiarize (`whisper/bin_load.cpp:430`).
- **Risposta:** serve un **modello separato**. L'unico modello di sola diarizzazione è
  Sortformer. I modelli che diarizzano da soli (MOSS, Granite Plus, multitalker) non coprono
  l'italiano **[V]** (catalogo, colonna lingue).

### 2.1 Sortformer: comportamento in dettaglio
- **[V] Solo `run`, niente stream.** Nella vtable `stream_begin`, `stream_feed`,
  `stream_finalize` e `run_batch` sono `nullptr` (`tcs/src/arch/sortformer/model.cpp:1110-1123`).
  L'header lo dice: il nucleo è in streaming (speaker cache AOSC + FIFO), ma l'unico punto
  d'ingresso è `transcribe_run` sulla registrazione intera. Un futuro push-audio avrebbe un'altra
  estensione (`tcs/include/transcribe/sortformer.h:8-16`).
- **[V] Serve tutto il PCM in un solo buffer.** `run` calcola il mel dell'intero buffer
  (`model.cpp:1042-1046`), poi lo scorre a chunk con il nucleo streaming (`model.cpp:830-834`).
  Il forward offline a contesto pieno (O(T²)) gira solo nei dump di debug, perché "would OOM on
  the many-minute audio" (`model.cpp:1050-1061`).
- **[V] Lunghezza.** Nessun controllo `INPUT_TOO_LONG` nel codice Sortformer. Il catalogo dice
  `"long_form_strategy": "hard-cap"`, ma `docs/input-limits.md` non elenca la famiglia. La cache
  AOSC è stata verificata "bit-exact" su una riunione AMI di 39 minuti (doc del modello, §Numerical
  Validation). NVIDIA: "can handle recordings that are several hours long", ma "performance may
  degrade on very long recordings" (model card HF).
  **[I]** Memoria: PCM f32 a 16 kHz = 230 MB/ora, più il mel. Un file di 3 ore richiede quindi
  circa 1 GB solo per i buffer.
- **[V] Annullamento.** Il flag di abort si controlla a ogni chunk (`model.cpp:831-834`) e
  all'inizio del run (`:1021-1022`).
- **[V] Numero di parlanti.** Automatico, al massimo 4 (`max_speakers` dal GGUF, cap
  architetturale), numerati per ordine di comparsa. Non si può fissare (nessun parametro
  nell'estensione). Oltre 4 parlanti attivi, "will be merged into the 4 arrival-order slots"
  (`docs/models/diar_streaming_sortformer_4spk-v2.1.md`, Known Limitations).
- **[V] Formato dell'output.** Post-processing minimo: soglia fissa 0,5 sulle probabilità per
  frame da 80 ms, un segmento per ogni tratto contiguo sopra soglia (`model.cpp:673-701`, chiamata
  a `:1010`). I segmenti escono **ordinati per parlante, non per tempo**. `p` è sempre NaN. Non ci
  sono durata minima, unione dei buchi né il post-processing "dihard3" usato per i DER pubblicati
  ("simple offline segmentation; the DER-grade dihard3 post-processing is a later … concern",
  `model.cpp:673-676`). Le sovrapposizioni sono possibili: due parlanti attivi nello stesso frame
  producono due segmenti.
- **[V] Preset.**

  | Preset | Lookahead | DER AMI IHM (FA + dihard3) | Throughput (M4 CPU) |
  | --- | --- | --- | --- |
  | `VeryHighLatency` | ~30,4 s | 14,59% | ~25-31× realtime |
  | `HighLatency` | ~10 s | non misurato | — |
  | `LowLatency` | ~1,04 s | 14,80% (3 riunioni) | ~0,85-1,2× realtime |
  | `Default` | config del GGUF | — | — |

  Fonte: `docs/porting/families/sortformer.md` al tag `v0.2.4`. Sul CPU M4, Q8_0 fa 51-101×
  realtime e F16 44-81× (preset Default, campioni jfk e dots). Non ci sono misure su CPU x64 né su
  Vulkan.
- **[V] Quantizzazioni.** Ci sono solo F32, F16 e Q8_0. Le k-quant sono state ritirate perché un
  errore sui pesi "can deterministically flip a near-tie pick and permute speaker labels
  mid-stream" (Q5_K_M: da 9,47% a 32,13% di DER su una riunione).
- **[V] Stabilità delle etichette.** "Perturbation-sensitive label continuity": backend diversi
  (Metal o CPU) possono risolvere diversamente un quasi-pareggio e cambiare l'assegnazione delle
  etichette a metà registrazione (doc del modello).

### 2.2 File del modello Sortformer
Repo HF `handy-computer/diar_streaming_sortformer_4spk-v2.1-gguf`, revisione corrente di `main`
`ae4afbb5c3d33b71cf2dbf600022b655ee706dd0`, non gated **[V]** (API HF):

| File | Byte | SHA256 (LFS oid) |
| --- | ---: | --- |
| `diar_streaming_sortformer_4spk-v2.1-Q8_0.gguf` | 139.310.336 | `a5dacdc650790266c7a362e54e6bf51952015487edaa606c4e11632bc32442a9` |
| `diar_streaming_sortformer_4spk-v2.1-F16.gguf` | 236.606.560 | `62faec7b99ad23e323087597604b50728abe85089b6364970b019a845547bf99` |
| `diar_streaming_sortformer_4spk-v2.1-F32.gguf` | 470.910.560 | `61597ce9447539ca32ce6fefb923551ab6afbdc068223c5d19929296ad2aa380` |

URL nel formato di `sb/src-tauri/src/managers/models.json`:
`https://huggingface.co/handy-computer/diar_streaming_sortformer_4spk-v2.1-gguf/resolve/ae4afbb5c3d33b71cf2dbf600022b655ee706dd0/diar_streaming_sortformer_4spk-v2.1-Q8_0.gguf`.
Upstream: `nvidia/diar_streaming_sortformer_4spk-v2.1` @ `fafaab5`, 117,7 M parametri.

### 2.3 Licenza
- **[V]** NVIDIA Open Model License (catalogo: `"spdx": "other"`). Uso commerciale consentito.
  Chi ridistribuisce deve includere una copia della licenza e la nota "Licensed by NVIDIA
  Corporation under the NVIDIA Open Model License". La licenza termina se si aggirano i guardrail
  o si avvia un contenzioso sul modello (pagina della licenza su nvidia.com). Sbobino scarica il
  modello da HF invece di ridistribuirlo **[I]**, ma conviene comunque mostrare la licenza come
  già fa per gli altri modelli (campo `licenza` in `models.json`).

### 2.4 Qualità attesa in italiano
- **[V]** Model card NVIDIA: "trained on publicly available speech datasets, primarily in
  English", circa 5.000 ore (Fisher, AMI, VoxConverse, ICSI, AISHELL-4, DIHARD…). Dice anche
  "Performance may degrade on non-English speech" e "performance degrades on recordings with 5 and
  more speakers". DER a 1,04 s: CALLHOME 2 parlanti 6,65%, DIHARD III 1-4 parlanti 15,09%,
  AliMeeting (cinese) 12,60%.
- **[I]** La diarizzazione dipende meno dalla lingua del riconoscimento del parlato, e il training
  contiene già cinese (AISHELL-4, AliMeeting) con buoni risultati. Su 2-4 parlanti italiani in una
  stanza tranquilla è ragionevole aspettarsi un risultato utilizzabile. Ma nessuna fonte misura
  l'italiano: va provato su 2-3 registrazioni reali prima di decidere.

---

## 3. Alternative in Rust/ONNX

Vincolo di Sbobino **[V]**: `ort` è fissato a `=2.0.0-rc.12` da `vad-rs` (Cargo.toml di vad-rs
@ `2a412ed`, riga 14) e quindi a ORT API 24 / ONNX Runtime 1.24 (`ort-2.0.0-rc.12/Cargo.toml:32`).
È linkato in dinamico su `onnxruntime.dll` 1.24.2 da `ORT_LIB_LOCATION`, che `build.rs` copia
accanto all'exe (`sb/src-tauri/build.rs:31-47`). Nel processo può esserci un solo
`onnxruntime.dll` con quel nome **[I]**.

### 3.1 `speakrs` (port del pipeline pyannote community-1)
- **[V] Versione.** 0.5.0 su crates.io (2026-07-07), Apache-2.0. Repo `avencera/speakrs`, ultimo
  push 2026-10-02, 0 issue aperte: è mantenuto. Il README della HEAD parla già di 0.6 e di
  `ort` rc.13, ma su crates.io 0.6 non c'è.
- **[V] Pipeline.** Segmentazione pyannote 3.0, decodifica powerset, aggregazione overlap-add,
  embedding WeSpeaker ResNet34 con maschere, PLDA, clustering AHC + VBx (README, `cargo-rdme`).
  `PipelineConfig` espone `binarize`, `ahc`, `vbx`, `merge_gap`, `speaker_keep_threshold` e
  `reconstruct_method`, ma **non** un numero di parlanti fisso o min/max
  (`speakrs-0.5.0/src/pipeline/config.rs:21-47`). Il numero di parlanti è quindi automatico, con la
  soglia AHC.
- **[V] Compatibilità con ORT.** Dipende da `ort = "2.0.0-rc.12"` con feature `ndarray`
  (`Cargo.toml:147-149`), la stessa versione di Sbobino: si unifica senza un secondo runtime.
  `load-dynamic` è opzionale.
- **[V] Costi di build.** Su x86_64 le feature di default includono `ndarray-linalg` con
  `intel-mkl-static` (`Cargo.toml:65-69`, `:176-180`). In alternativa c'è `openblas-static` o
  `-system`, che richiede un toolchain C. `online` (default) aggiunge `hf-hub` con `native-tls`:
  per Sbobino va spento, scaricando i file da soli e caricandoli con `from_dir`.
  **[I]** MKL statico aumenta sensibilmente tempi di build e dimensione dell'exe.
- **[V] Modelli.** `avencera/speakrs-models` (HF, non gated, revisione fissata dall'SDK
  `5d24ffee75f13fb061fa6d10944a64e2dc1d5e6f`). Per CPU servono circa 60 MB:
  `segmentation-3.0.onnx` (5.916.308 B), `wespeaker-voxceleb-resnet34.onnx` più `.onnx.data`
  (26,9 + 26,7 MB) o le varianti `-tail`/`-fbank`, e i file `plda_*.npy` (~0,27 MB). Sono convertiti
  da pyannote community-1: **CC-BY-4.0** (richiede attribuzione). Su HF l'originale è gated (va
  chiesto l'accesso e accettati i termini), il mirror di avencera no. Il README del mirror
  scarica sull'utente il rispetto dei termini a monte.
- **[V] Offline e streaming.** Solo offline (`pipeline.run(&audio)` sull'audio intero; la "queue"
  è una coda di file, non uno streaming).
- **[V] Qualità.** pyannote community-1 (model card): AMI IHM 17,0%, VoxConverse 11,2%, REPERE
  (francese) 8,9%, AISHELL-4 11,7%, CALLHOME 26,7%. È addestrato su più lingue. speakrs dichiara
  7,1% di DER su VoxConverse dev contro il 7,2% di pyannote (dichiarazione dell'autore, non
  verificata). **[I]** Su 2-4 parlanti italiani è probabilmente l'opzione più robusta, e non ha
  il limite di 4 parlanti.

### 3.2 `sherpa-onnx` (binding ufficiale k2-fsa)
- **[V] Versione.** `sherpa-onnx` / `sherpa-onnx-sys` 1.13.8 (2026-09-11), Apache-2.0, molto
  usato (~490k download). `sherpa-rs` (thewh1teagle) è fermo a 0.6.8 (2025-10).
- **[V] API.** `OfflineSpeakerDiarization::create(&OfflineSpeakerDiarizationConfig)` e
  `process(&[f32])`. Il config contiene `segmentation` (pyannote), `embedding`, `clustering:
  FastClusteringConfig` (numero di cluster oppure soglia), `min_duration_on` e `min_duration_off`
  (docs.rs via ctx7). Il numero di parlanti si può **fissare** o lasciare alla soglia. Solo
  offline: non c'è diarizzazione online.
- **[V] ONNX Runtime.** Incompatibile con quello di Sbobino senza lavoro. `sherpa-onnx-sys`
  scarica a build-time un archivio precompilato da GitHub (`build.rs:13`, `:199-207`; si può
  evitare con `SHERPA_ONNX_LIB_DIR`/`SHERPA_ONNX_ARCHIVE_DIR`). Su Windows x64 ci sono due
  varianti:
  - `static` (default): `win-x64-static-MT-Release`, onnxruntime statico con CRT **/MT**
    (`build.rs:385-386`);
  - `shared`: copia le sue DLL, compreso un proprio `onnxruntime.dll`, nella cartella del profilo
    (`build.rs:403-404`, `:843-867`). Il cmake di sherpa-onnx @ `v1.13.8` usa onnxruntime-libs
    `v1.28.2` (`cmake/onnxruntime-win-x64.cmake:42-43`).

  **[I]** Con `static` probabilmente ci sono conflitti di simboli con l'`onnxruntime.lib`
  dinamico di `ort` e di CRT (/MT contro il /MD di Rust). Con `shared` le due DLL hanno lo stesso
  nome e se ne carica una sola. Andrebbe allineato tutto su un unico runtime: va verificato.
- **[V] Modelli** (release GitHub k2-fsa):
  - `sherpa-onnx-pyannote-segmentation-3-0.tar.bz2`: 6.958.444 B (pyannote segmentation-3.0,
    **MIT**, gated su HF ma non sul mirror k2-fsa).
  - Embedding, a scelta: `3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx`
    (28,3 MB; repo 3D-Speaker Apache-2.0, licenza della singola model card da verificare),
    `wespeaker_en_voxceleb_resnet34_LM.onnx` (26,5 MB, CC-BY-4.0), `nemo_en_titanet_small.onnx`
    (40,3 MB, CC-BY-4.0).
  - Da evitare: `sherpa-onnx-reverb-diarization-v1/v2` (Rev, licenza "other", gated).
- **[I] Qualità.** È la stessa segmentazione di pyannote 3.x, ma con un clustering più semplice
  (niente PLDA né VBx). Probabilmente è peggiore di community-1, ma accettabile su 2-4 parlanti
  ben separati.

### 3.3 `pyannote-rs`
- **[V] Versione.** 0.3.4 (2025-09-07), MIT. Repo fermo da settembre 2025, 18 issue aperte.
  Dipende da `ort ^2.0.0-rc.10`, `ndarray ^0.16` e `knf-rs`, che compila kaldi-native-fbank in
  C++ e richiede Clang e CMake (BUILDING.md).
- **[V] Algoritmo.** Segmentazione su finestre di 10 s non sovrapposte. Un embedding per ogni
  tratto continuo di parlato, poi assegnazione **online** con soglia di coseno contro il primo
  embedding salvato per ogni parlante. Ha un `max_speakers` (`src/identify.rs`). Non c'è
  clustering globale né gestione delle sovrapposizioni.
- **[I] Compatibilità.** Nel grafo di Sbobino `ort ^rc.10` si unifica a rc.12, che usa
  `ndarray 0.17` (`ort-2.0.0-rc.12/Cargo.toml:183-186`), mentre pyannote-rs passa array
  `ndarray 0.16`. È probabile che non compili senza fork.
- **[V] Qualità.** Secondo il README di speakrs (fonte di parte), pyannote-rs non produce output
  su 183 dei 216 file di VoxConverse dev e ha l'80% di DER sui restanti. **Da scartare.**

### 3.4 Altri crate visti, non approfonditi
`diarization` 0.1.0 e `diaric` 0.2.0 (findit-studio, 2026, clustering AHC→VBx e online, pyannote
segmentation-3.0 incluso) **[V]** (crates.io). Hanno pochi download (218 / 2.767): troppo giovani
per una dipendenza **[I]**.

---

## 4. Implicazioni per Sbobino

### 4.1 Stato attuale rilevante **[V]**
- Le Frasi non hanno tempi. `PipelineEvent::Phrase { id, text }`
  (`sb/src-tauri/src/engine/pipeline.rs:19-28`). Il segmentatore lavora a frame da 30 ms, con
  Frase massima 18 s e hangover di 700 ms (`sb/src-tauri/src/audio_toolkit/segmenter.rs:4-30`).
- Il TXT è `phrases.join("\n")` (`sb/src-tauri/src/managers/transcription.rs:156-166`).
- La Registrazione "Entrambi" **somma** gli Ingressi in un solo file
  (`sb/src-tauri/src/audio_toolkit/mixer.rs:1-3`). Il mixer però riceve ogni blocco con indice
  di Ingresso e timestamp QPC (`mixer.rs:198`).
- ADR 0003: la Trascrizione parte solo dopo Stop, mai durante la Registrazione
  (`sb/docs/adr/0003-trascrizione-solo-dopo-stop.md`).
- Timestamp dei modelli in transcribe.cpp (catalogo `v0.2.4`): Nemotron 3.5 e Parakeet TDT v3
  arrivano a `token`, Whisper Large v3 Turbo solo a `segment`.

### 4.2 Prima, dopo o insieme alla Trascrizione
- **Proposta: diarizzare prima, nella stessa Attività Trascrizione.** Sortformer non ha stream e
  vuole tutto il PCM. Quindi: (1) decodificare la Sorgente in un buffer mono 16 kHz, (2) fare il
  `run` di Sortformer (Q8_0, preset `VeryHighLatency` per i file) con progresso "Riconoscimento
  dei parlanti" e annullamento, (3) eseguire la pipeline attuale. A 25-100× realtime **[V su M4]**,
  un'ora di audio costa decine di secondi o qualche minuto di CPU in più **[I su x64]**.
- Il buffer intero può essere usato solo da Sortformer: la pipeline ASR continua a decodificare
  in streaming. Per file di molte ore la memoria cresce (§2.1). Tagliare il file a pezzi farebbe
  perdere la coerenza delle etichette tra un pezzo e l'altro, perché manca un confronto fra
  embedding **[I]**. **Proposta:** mettere un limite di durata con un messaggio chiaro, invece di
  spezzare il file.
- Farlo "dopo" porta allo stesso risultato. "Prima" permette in più di spezzare le Frasi al
  cambio di parlante (§4.3).

### 4.3 Allineare i parlanti alle Frasi
1. **Proposta, minima.** La pipeline conta i frame (30 ms) dall'inizio del file e dà a ogni
   Frase `t0`/`t1` (inizio con prefill, fine senza hangover). Il parlante è quello con la
   **massima sovrapposizione** temporale con i `SpeakerSegment` (ordinati per tempo, prima uniti
   i buchi brevi e tolti i segmenti sotto ~0,3 s, perché Sortformer non lo fa **[V]** §2.1). Senza
   sovrapposizione la Frase eredita il parlante precedente.
2. **Proposta, migliore e piccola.** Siccome la diarizzazione è già fatta, il segmentatore
   chiude anche la Frase quando il parlante attivo cambia (oltre che con il silenzio e i 18 s).
   Così una Frase non mescola due voci. È un ingresso in più nella macchina a stati pura del
   segmentatore.
3. **Più fine** (non necessario all'inizio): spezzare una Frase sui timestamp dei token
   (Nemotron e Parakeet `token`). Con Whisper non si può (`segment`).
- Le sovrapposizioni (due voci insieme) restano a una sola etichetta: quella con più tempo.

### 4.4 Durante la Registrazione dal vivo
- **[V]** Con `transcribe-cpp` 0.2.4/0.3.0 non si può: Sortformer non ha stream e il preset
  `LowLatency` vale solo dentro un `run`. L'ADR 0003 esclude comunque la trascrizione dal vivo.
- **[I]** Il clustering online sarebbe possibile solo con un'altra libreria (`diaric` "online",
  oppure un embedding ONNX con assegnazione a soglia tipo pyannote-rs, di qualità bassa) e con la
  trascrizione dal vivo, che oggi non c'è. **Proposta:** solo a posteriori.

### 4.5 "Entrambi": Ingressi separati come diarizzazione grezza
- **[V]** Oggi l'informazione si perde perché il mixer somma gli Ingressi.
- **[I] Costo quasi nullo, ma non è zero.** Durante la Registrazione il mixer o il worker può
  salvare accanto al file un piccolo file di attività per Ingresso (per esempio energia RMS o VAD
  per frame da 30 ms, con i tempi già allineati da QPC). Alla Trascrizione: Frase con energia
  dominante dal microfono = "Io", dall'audio di sistema = "Altri". Limiti:
  - senza cuffie il microfono riprende anche gli altoparlanti (eco, rientro), quindi serve il
    confronto dell'energia, non una semplice soglia;
  - "Altri" spesso sono più persone (riunione online), e il sistema non le distingue;
  - non vale per i file importati né per le Registrazioni solo microfono.
- **Proposta:** come euristica a parte ("Io / Altri") ha senso solo per le Registrazioni
  "Entrambi". La combinazione migliore è "microfono = Io" più Sortformer sul solo audio di sistema
  per separare gli altri. Questo però richiede di salvare gli Ingressi anche separati (per esempio
  un file stereo L=microfono, R=sistema, oppure un secondo file). È un cambio di formato da
  valutare dopo.

### 4.6 Formato del TXT
- **Proposta.** Le Frasi consecutive dello stesso parlante si uniscono in un paragrafo. Il cambio
  di parlante apre un nuovo paragrafo preceduto da una riga vuota:
  ```
  Parlante 1: Buongiorno a tutti, iniziamo. Oggi parliamo del bilancio.

  Parlante 2: Ho una domanda sul punto tre.
  ```
  Facoltativo: un tempo all'inizio del turno (`[00:12:34] Parlante 1: …`), che si ottiene
  gratis dal `t0` della prima Frase. Senza diarizzazione il TXT resta com'è oggi.
- **Rinomina.** Sortformer dà numeri per ordine di comparsa, non identità, e non riconosce le
  stesse persone tra file diversi **[V]**. **Proposta:** a fine Trascrizione, se ci sono
  parlanti, mostrare l'elenco "Parlante 1…N" con un campo nome. Rinominare riscrive il TXT appena
  salvato (sostituzione dell'etichetta a inizio paragrafo). Le etichette "Io/Altri" di §4.5 sono
  già nomi.
- Con l'opzione attiva, i nuovi termini vanno in `CONTEXT.md` (per esempio **Parlante**,
  **Turno**).

### 4.7 Download e verifica
- **[V]** Il formato di `models.json` (URL con revisione fissata, `sha256`, `size`, `licenza`)
  basta già per il GGUF Sortformer (§2.2). **[I]** Serve però un campo o una sezione che lo
  distingua dai modelli di trascrizione: non deve comparire nella scelta del modello, come fa
  Handy (§1). Si scarica alla prima attivazione dell'opzione.

---

## 5. Tabella comparativa

| Opzione | Nuove dipendenze | Modelli (dimensione, licenza) | Parlanti | Offline / stream | Italiano | Rischi principali |
| --- | --- | --- | --- | --- | --- | --- |
| **A. Sortformer via `transcribe-cpp`** | nessuna (API già nel crate 0.2.4) | 1 GGUF Q8_0, 139 MB, NVIDIA Open Model License (commerciale ok, nota di attribuzione) | auto, max 4, non impostabile | solo `run` sul file intero | non misurato; training "primarily English" + cinese | qualità in italiano da provare; post-processing grezzo (lo fa Sbobino); RAM proporzionale alla durata |
| **B. `speakrs` 0.5.0** | crate + `ndarray-linalg` con MKL statico (o OpenBLAS); stesso `ort` rc.12 | ~60 MB ONNX + PLDA, CC-BY-4.0 (pyannote community-1), mirror non gated | auto (soglia), senza limite fisso | solo offline | community-1 multilingue (REPERE fr 8,9%) | build più pesante; README già su 0.6/rc.13 (aggiornamento di ort accoppiato a vad-rs); attribuzione CC-BY |
| **C. `sherpa-onnx` 1.13.8** | crate + archivio nativo scaricato a build-time, con un proprio onnxruntime (1.28.x) | ~7 MB segmentazione (MIT) + ~28 MB embedding (Apache-2.0 / CC-BY-4.0) | fissabile o soglia | solo offline | dipende dall'embedding; nessun dato | conflitto con `onnxruntime.dll` 1.24.2 e CRT /MT: da risolvere prima di tutto |
| **D. `pyannote-rs` 0.3.4** | crate + C++ (knf-rs) | ~7 MB + ~29 MB (MIT / CC-BY-4.0) | soglia online, max N | offline | nessun dato | quasi abbandonato; probabile incompatibilità ndarray; qualità molto bassa |
| **E. Ingressi separati ("Entrambi")** | nessuna | nessuno | 2 (Io / Altri) | anche dal vivo | indipendente dalla lingua | solo Registrazioni "Entrambi"; eco senza cuffie; "Altri" non distinti |
| **F. ASR che diarizza (MOSS, Granite Plus, multitalker)** | nessuna | 0,6-2,3 GB | — | `run` | **no** (en/zh, en/fr/de/es/pt, en) | esclusa per la lingua |

## 6. Raccomandazione (proposta, decide l'utente)

**Partire con A (Sortformer via `transcribe-cpp`) come opzione "Riconosci i parlanti", a
posteriori e dentro la Trascrizione, con l'allineamento di §4.3 punti 1 e 2 e il TXT di §4.6.**

Motivi:
1. Nessuna nuova dipendenza nativa né un secondo ONNX Runtime. L'API (`RunOptions`,
   `RunExtension::Sortformer`, `Transcript::speaker_segments`) è già nel crate in uso e non
   cambia in 0.3.0.
2. Un solo file da 139 MB, scaricabile e verificabile con l'infrastruttura di `models.json`
   (URL con revisione fissata e SHA256 noti).
3. Il limite di 4 parlanti copre il caso d'uso indicato (2-4 parlanti) e il numero è automatico,
   quindi non serve un'altra impostazione.
4. Annullamento e progresso sono possibili (abort controllato a ogni chunk).

Condizione per proseguire: una **prova di qualità su 2-3 registrazioni italiane reali**
(lezione, riunione a 3-4 voci, intervista) prima di costruire la UI. Si può fare con
`transcribe-cli --batch-jsonl` di transcribe.cpp o con un piccolo esempio Rust, fuori dall'app.
Se il risultato non regge (etichette che si scambiano, parlanti uniti), il ripiego è **B
(`speakrs`)**: stessa versione di `ort`, pipeline pyannote completa e multilingue. Il prezzo è una
build più pesante (MKL) e l'attribuzione CC-BY-4.0. C e D sono sconsigliate per i motivi in
tabella.

L'euristica **E** ("Io / Altri") resta un'aggiunta separata e facoltativa per le Registrazioni
"Entrambi". Ha senso solo se si decide di salvare l'attività per Ingresso. La diarizzazione dal
vivo è fuori portata finché vale l'ADR 0003.

## 7. Punti aperti
- Velocità reale di Sortformer su CPU x64 e su Vulkan (le misure pubblicate sono solo Apple M4).
- Qualità in italiano (nessuna fonte primaria).
- Come si comporta Sortformer su file molto lunghi (oltre 2-3 ore): memoria e tenuta delle
  etichette.
- Se un `onnxruntime.dll` più recente (1.28) funziona con `ort` rc.12 / API 24. Serve solo per
  l'opzione C, ed è un'inferenza: l'API C di ORT è retrocompatibile, ma non è verificato.

---

## 8. Nemotron-3-Diarization (aggiornamento del 2026-10-03)

Questa sezione aggiorna le §5-6 alla luce di un modello uscito dopo la prima ricerca. Le sezioni
precedenti restano valide per Sortformer 4spk v2.1.

### 8.0 Fonti

| Fonte | Riferimento |
| --- | --- |
| Model card | `nvidia/Nemotron-3-Diarization`, revisione `f667ed73aee57d40cc39428eb768b4fd87a0a29e` (lastModified 2026-09-24): `README.md`, `config.json`, `processor_config.json`, `ASR_INTEGRATION_GUIDE.md`, API HF `?blobs=true` e `/tree/main` (stessi file, dimensioni e SHA; `main` = `f667ed7` al 2026-10-03) |
| Blog NVIDIA | `https://huggingface.co/blog/nvidia/nemotron-diarization` (2026-09-23) |
| Licenza | `https://openmdw.ai/license/1-1/` |
| transcribe.cpp | HEAD `main` = tag `v0.3.0` = `077110e`. Issue #170 "Add Nemotron 3 Diarization" (2026-09-24). **PR #175 "nemotron3 diarization port"**, branch `nemotron3-diar`, head `e6672a8` (2026-09-26), **aperta, non mergiata** |
| NeMo-Speech.cpp | `NVIDIA/NeMo-Speech.cpp`, release `v0.2.0` (2026-10-02), Apache-2.0, `include/nemo_speech/diar.h`, `docs/sdk.md` |
| Conversioni di terzi | API HF: `Glimpse-Dictation/Nemotron-3-Diarization-gguf` @ `273ebb0`, `onnx-community/Nemotron-3-Diarization-ONNX` @ `353b6f8` (header GGUF e grafo ONNX letti direttamente) |
| sherpa-onnx | issue #3497 (Sortformer) e #4006 (Nemotron-3-Diarization), entrambe aperte |
| Baseten | `baseten.co/blog/nvidia-nemotron-3-diarization/`: solo servizio cloud, non rilevante per l'uso locale |
| ctx7 | `/handy-computer/transcribe.cpp` (API `Stream::snapshot`) |

Path abbreviati: `tc175/…` = file sul branch `nemotron3-diar` della PR #175.

### 8.1 Il modello (model card) **[V]**
- **Data.** "Release Date: September 23, 2026" (il repo HF esiste dal 2026-09-01).
- **Architettura.** È della famiglia Sortformer, ma non è la stessa rete. NeMo lo carica con
  `SortformerEncLabelModel`, ordina i parlanti per arrivo e usa la stessa speaker cache AOSC e
  la stessa coda FIFO dello Streaming Sortformer. Cambiano però l'encoder (Transformer di 31
  strati con RoPE, d=512, 8 teste, invece di FastConformer più Transformer), l'uscita (10 ms
  invece di 80 ms, grazie a un upsampler Conv1D) e il silenzio in cache, che qui è un embedding
  appreso. 100M parametri. NeMo-Speech.cpp lo chiama preset "v3"
  (`diar.h`), Glimpse "Streaming Sortformer v3".
- **Parlanti.** Al massimo **8**, numerati per ordine di comparsa. L'uscita è un tensore
  `[T, 8]` di probabilità per frame da 10 ms.
- **Streaming e offline.** Lo stesso checkpoint lavora con quattro configurazioni consigliate,
  cioè latenze del buffer d'ingresso, senza il tempo di calcolo: 30,4 s (offline), 1,04 s,
  0,64 s e 0,32 s. Si può scendere fino a 80 ms, ma 0,32 s è il minimo consigliato. "With
  chunked inference, the maximum audio duration is not limited."
- **Audio.** 16 kHz, mono. Mel a 128 bande, hop 10 ms (`processor_config.json`).
- **Lingue di training.** Circa 10.000 ore di conversazioni reali più 82.611 ore di miscele
  simulate. Inglese e mandarino dominano. Ci sono poi DISPLACE (hindi, kannada, telugu,
  bengali), VoxConverse, DIHARD e CALLHOME (multilingue), **YODAS-v2** (5.000 ore
  pseudo-etichettate, multilingue) e "David AI [D12] … 21 languages" (19.216 ore di miscele
  multilingue). **Non c'è nessun dato o benchmark esplicito sull'italiano.**
  **[I]** YODAS e i 21 linguaggi di David AI contengono probabilmente anche italiano. Rispetto a
  v2.1, che era "primarily English", la base multilingue è molto più ampia.
- **Benchmark** (DER %, protocollo della card; tra parentesi v2.1 alla stessa latenza):

  | Dataset | 30,4 s | 1,04 s | 0,32 s |
  | --- | --- | --- | --- |
  | DIHARD III, 1-4 parlanti | 9,13 (13,98) | 9,47 (14,33) | 9,69 (14,37) |
  | DIHARD III, 5-9 parlanti | 27,58 (40,21) | 28,65 (41,39) | 29,49 (42,71) |
  | CALLHOME, 2 parlanti | 5,98 (5,68) | 6,98 (6,83) | 7,75 (7,92) |
  | CALLHOME, 3 parlanti | 9,26 (10,41) | 10,90 (11,26) | 11,84 (12,54) |
  | AMI test MHM | 9,25 (15,81) | 9,48 (16,36) | 10,05 (17,77) |
  | NOTSOFAR1 MHM, 3-4 parlanti | 5,25 (11,14) | 5,85 (12,03) | 6,57 (12,94) |

  Con 2 parlanti al telefono il guadagno è nullo: CALLHOME a 2 parlanti è leggermente peggiore.
  Il guadagno cresce con le riunioni e con 3 o più parlanti. La velocità è misurata solo su GPU
  (RTX PRO 5000, BF16, PyTorch). A batch 1, 1,04 s, eager, fa 38× realtime contro i 16× di v2.1.
- **Licenza.** OpenMDW-1.1. Uso commerciale consentito, modifica e ridistribuzione consentite.
  Chi ridistribuisce deve conservare "a copy of this agreement" e le note di copyright e di
  origine. Non ci sono restrizioni d'uso sugli output. C'è una clausola difensiva sui brevetti:
  la licenza termina se si avvia una causa. È meno onerosa della NVIDIA Open Model License di
  v2.1: non c'è la nota di attribuzione obbligatoria né la clausola sui guardrail.
- **File nel repo NVIDIA** (revisione `f667ed7`):

  | File | Byte | SHA-256 |
  | --- | ---: | --- |
  | `Nemotron-3-Diarization.nemo` (BF16) | 198.676.480 | `867c53f552998f772e5b5e5c082962ae85ee7ca5669c2bc17d7f615133d4e96d` |
  | `model.safetensors` (F32, per HF Transformers) | 396.954.592 | `c074d86335b3b794f8fa5edc25594558f128bdb3914d27806a3a5a2e44963cb6` |
  | `Nemotron-3-Diarization.q8_0.gguf` | 107.012.128 | `08456d9e22cd9a323c0364d98375f3746d6e68507ebb705cd46438c534c7a3a1` |

  Il GGUF di NVIDIA ha `general.architecture = sortformer` (metadati GGUF dell'API HF) ed è fatto
  per **NeMo-Speech.cpp**. transcribe.cpp non lo usa: "It is not consumed here"
  (`tc175/docs/porting/families/nemotron3_diar.md`, Notes). NVIDIA non pubblica un ONNX ufficiale.
- **Verifica della fonte secondaria (unite.ai).** Questi dati sono confermati: 100M, encoder
  Transformer, 8 parlanti, streaming e offline, latenze 0,32/0,64/1,04 s, OpenMDW-1.1, formato
  NeMo. Mancano due cose: c'è anche la modalità offline a 30,4 s, e oltre al `.nemo` NVIDIA
  pubblica safetensors (HF Transformers) e un GGUF Q8_0.

### 8.2 Supporto in transcribe.cpp / `transcribe-cpp`
- **[V] Né la 0.2.4 né la 0.3.0 lo supportano.** HEAD `main` coincide con `v0.3.0`. `git grep`
  non trova `nemotron3_diar` né "Nemotron-3-Diarization", e in `docs/models/` ci sono solo
  `diar_streaming_sortformer_4spk-v2.1.md` e `moss-transcribe-diarize.md`. L'architettura
  `sortformer` esistente non lo carica: "NEW FAMILY, NOT A SORTFORMER VARIANT … src/arch/sortformer
  hardcodes max_speakers=4 and an 80 ms output grid" (`tc175/reports/porting/nemotron3_diar/…/intake.json`,
  known_risks).
- **[V] Il port c'è, ma solo nella PR #175** (CJ Pais, maintainer, 59 file, +8.282 righe).
  L'issue #170 ha la risposta "will come in #175". La PR aggiunge la famiglia
  `src/arch/nemotron3_diar/` (`model.cpp`, `stream.cpp`), l'header
  `include/transcribe/nemotron3_diar.h` e i wrapper in tutti i binding, Rust compreso.
- **[V] Stato del port** (`tc175/docs/porting/families/nemotron3_diar.md`): "Stage 4 complete
  pending tolerance review". Il DER coincide con quello di NeMo: AMI 9,212% offline e 9,530% a
  1,04 s, uguali al riferimento. Restano aperte tre cose:
  - la sezione Benchmarks è "TODO";
  - le GPU (Vulkan compreso) sono demandate allo "Stage 6";
  - "License (`openmdw-1.1`) still to be interpreted before ship".
- **[V] Diarizzazione dal vivo prevista e testata.** Il modello si usa in due modi:
  `transcribe_run` sul file intero, oppure `transcribe_stream_begin/feed/finalize`. Nel secondo
  caso: "After every feed the speaker segments cover all audio processed so far: a finished turn
  is final, a turn still in progress is reported open-ended … Finalize flushes the tail; the
  final segments equal a transcribe_run over the same audio at the same preset"
  (`tc175/include/transcribe/nemotron3_diar.h:12-28`). Tutti e cinque i preset sono permessi nello
  slot STREAM. Il test di capacità "Push-audio live diarization" è MUST PASS e risulta PASS:
  output bit-identico al `run` a tutti i preset.
- **[V] Post-processing grezzo come in v2.1.** I segmenti nascono da una soglia 0,5 su frame da
  10 ms, senza durata minima né unione dei buchi (`tc175/src/arch/nemotron3_diar/nemotron3_diar.h:133`).
- **[V] Memoria e lunghezza.** Non c'è un limite di durata (`max_audio_ms = 0`). Il calcolo e
  la memoria del grafo per passo sono costanti: al massimo 541 frame encoder a 1,04 s e 684 a
  30,4 s. Con l'audio crescono solo il mel conservato (nel percorso `run`) e le probabilità a
  10 ms, "~11.5 MB per hour" (`tc175/docs/input-limits.md`). Nel push-audio il mel viene
  consumato man mano (`mel_tm`: "Mel frames not yet consumed").
- **[V] CPU.** Su CPU il loader converte in F32 i pesi BF16/F16, con "+~400 MB RAM". I pesi
  quantizzati girano invece con i kernel nativi, senza conversione (`tc175/src/arch/nemotron3_diar/model.cpp:660-684`).
  Throughput su M4 Max a 12 thread: "30.4 s preset RTF ~0.02; 1.04 s preset RTF ~0.40", perché
  ogni chunk da 0,72 s ricalcola circa 540 frame su 31 strati. Non ci sono misure su x64.
- **[V] GGUF per la PR.** Nell'org `handy-computer` su HF **non c'è** un GGUF di questo modello
  (ci sono solo `moss-transcribe-diarize-gguf` e `diar_streaming_sortformer_4spk-v2.1-gguf`).
  La PR lo converte in locale: `scripts/convert-nemotron3_diar.py` produce
  `Nemotron-3-Diarization-BF16.gguf` (arch `nemotron3_diar`, chiavi `stt.nemotron3_diar.*`). La
  policy di quantizzazione ha già una regola per `diar.sil_emb`, ma il DER è stato validato
  solo sul BF16.
- **[V] I GGUF di terzi non sono compatibili con la PR.**
  `Glimpse-Dictation/Nemotron-3-Diarization-gguf` @ `273ebb0e65377f74576768257cd11f00c248c39d`
  (Q8_0 105.936.416 B, SHA-256 `877ff9e77e829e30158528cfaabf56188fca11d05349e09821b3033a97d31688`)
  usa la stessa stringa arch `nemotron3_diar`, ma viene dal fork `LegendarySpy/transcribe.cpp`
  (branch `glimpse-diarization`) e ha chiavi diverse. Per esempio la PR legge
  `stt.nemotron3_diar.aosc.*` e `encoder.rope_base`, il file ha `stream.pred_score_threshold` e
  `encoder.rope_theta` (header GGUF letto contro le chiavi di `tc175/src/arch/nemotron3_diar/*.cpp`).
  **[I]** Con la PR il caricamento fallirebbe. `dawsonvosburg/…-gguf` ha lo stesso SHA ed è una
  copia. Il README di Glimpse dichiara che il Q8_0 equivale all'F32 su AMI (DER 9,23% contro
  9,22%): lo dice l'autore, non è verificato.

### 8.3 API Rust (dalla PR #175) **[V]**
- Nuovi tipi `Nemotron3DiarPreset::{Default, VeryHighLatency, LowLatency, VeryLowLatency,
  UltraLowLatency}` e `Nemotron3DiarOptions { preset }`. Si usano come
  `RunExtension::Nemotron3Diar(..)` e come **`StreamExtension::Nemotron3Diar(..)`**
  (`tc175/bindings/rust/transcribe-cpp/src/family.rs`). Il test Rust
  `nemotron3_diar_run_and_stream_extensions` fa `session.stream(&RunOptions::default(),
  &StreamOptions { family: Some(StreamExtension::Nemotron3Diar(opts)), .. })`, poi `feed` a pezzi
  e infine `finalize` (`tc175/bindings/rust/transcribe-cpp/tests/extensions.rs`).
- I segmenti incrementali si leggono con `Stream::snapshot()`. Chiama `materialize_run`, che
  riempie `Transcript.speaker_segments` da `transcribe_n_speaker_segments`
  (`tc175/bindings/rust/transcribe-cpp/src/session.rs:296-316`). `Stream` non ha un accessor
  dedicato ai parlanti. Il test Rust non controlla i segmenti durante il feed; lo fa il test C
  `nemotron3_diar_stream_unit` ("rows during feed, open turns extend").
- Vale sempre il vincolo di un solo stream attivo per `Model` ("a stream is already active on
  this model", `compute_lock` in `session.rs` di `v0.3.0`). Due Ingressi diarizzati dal vivo
  richiedono quindi due `Model` caricati, come per la trascrizione dal vivo dell'ADR-0004.

### 8.4 Da 0.2.4 a 0.3.0: cambiamenti incompatibili **[V]**
Il diff `v0.2.4..v0.3.0` contiene 8 commit: repetition guard, tekken, generic prompting,
"remove unk", fix di canary e serde.
- **C ABI.** `transcribe_run_params` cambia layout: `spec_k_drafts` si sposta e arrivano
  `vocabulary`, `n_vocabulary`, `prompt` e `prefix`. Cambiano anche `TRANSCRIBE_VERSION_*` e
  l'abihash. Il binding e la `transcribe.dll` vanno quindi aggiornati insieme.
- **Rust.**
  - `RunOptions` ha tre campi pubblici nuovi (`vocabulary`, `prompt`, `prefix`) e non è
    `#[non_exhaustive]`: un letterale senza `..Default::default()` non compila più.
  - `Error` ha la variante nuova `OutputRepetition { message, partial }`. L'enum è
    `#[non_exhaustive]`, quindi i match con un ramo di default restano validi.
  - Nuova feature opzionale `serde`.
- **Comportamento.** `OUTPUT_REPETITION` lo emettono solo canary, funasr_nano, granite,
  moonshine_streaming e voxtral, non i tre modelli di Sbobino. "remove unk" tocca anche
  `parakeet/model.cpp`, cioè il token `<unk>` in uscita da Nemotron 3.5 e Parakeet.
- **Per Sbobino** (`sb/src-tauri/src/engine/transcribe_cpp.rs:65-67`, `:118-122`): usa
  `..RunOptions::default()` e un match su `Error` con ramo `e => …`. **[I]** L'aggiornamento
  a 0.3.0 dovrebbe compilare senza modifiche al codice. Resta da verificare con una build.

### 8.5 Alternative se non si aspetta la PR
1. **NeMo-Speech.cpp (NVIDIA, Apache-2.0)** **[V]**. È il runtime ufficiale indicato dalla model
   card. La v0.2.0 (2026-10-02) aggiunge Nemotron 3 Diarization e "Live diarized
   transcription". Ci sono archivi Windows x86_64 per CPU, CUDA e Vulkan. L'API C stabile di
   `diar.h` offre:
   - `nemo_speech_diar_stream_open` / `_push_f32` / `_finish` per lo streaming dal vivo, con
     risultati "valid on a live stream at any point";
   - `nemo_speech_diar_offline_f32`;
   - `nemo_speech_diar_segments`, con post-processing configurabile (onset/offset, pad,
     min_gap, min_duration, cioè la semantica NeMo che transcribe.cpp non ha);
   - la gestione dei flussi lunghi: oltre circa 20 minuti compatta le probabilità vecchie in
     segmenti finali.

   Usa il Q8_0 di NVIDIA (106 MB) senza conversioni. Lo svantaggio: non c'è un crate Rust, quindi
   servono FFI con bindgen e il packaging delle DLL accanto all'exe.
   **[I]** NeMo-Speech.cpp porta con sé un proprio ggml (llama.cpp b11151) e transcribe.cpp il
   suo. Due copie di `ggml*.dll` con lo stesso nome nello stesso processo sono un rischio
   concreto, da verificare prima di tutto (stesso problema di `onnxruntime.dll` per sherpa-onnx).
2. **ONNX + `ort` (già nel progetto)** **[V]**. `onnx-community/Nemotron-3-Diarization-ONNX`
   @ `353b6f8ad2cac3580e982d7fbdf0a010786b0406`, licenza openmdw-1.1, opset 21 più
   `com.microsoft`. Il grafo calcola **un solo passo**: ingressi `input_features [B, T, 128]`,
   `cached_embeds [B, N, 512]` e `attention_mask`; uscite `logits [B, T, 8]`, `chunk_embeds` e
   `silence_embeds`. In Rust bisognerebbe scrivere il mel NeMo, la FIFO e soprattutto la
   compressione AOSC. Il port di transcribe.cpp mostra che è delicata: top-k con pareggi a 1e-7,
   da replicare "bit-for-bit" per non scambiare le etichette (546 righe in `stream.cpp`). Le
   varianti, per `.onnx_data`:

   | Variante | Byte | SHA-256 |
   | --- | ---: | --- |
   | fp32 | 398.184.448 | `c293d9b5930eb9f6172f095ced052c0d1bbdbeb2594a115497583ca209b1dbd6` |
   | fp16 | 199.287.808 | `affecf841c462d78c86b56a6eee6216b119809285bcbdb4d5c4d541810e9d317` |
   | int8 (`quantized`) | 120.479.872 | `002d7483e1c865c35c82220fdb378f185ff213c6d35922b38ae421c8ec72c338` |
   | q4 | 82.768.000 | `4513ed877d7cb83944bcb27c747221a7180e03f790d1c703e50af01792b7b8f4` |

   **[I]** Funziona con ORT 1.24 (opset 21 supportato), ma è il lavoro più grande e più
   rischioso. Ha senso solo se transcribe.cpp non mergia la PR.
3. **sherpa-onnx** **[V]**: oggi non supporta né Sortformer né Nemotron-3-Diarization (feature
   request #3497 e #4006 aperte). Restano anche i problemi di runtime della §3.2.
4. **Fork Glimpse** (`LegendarySpy/transcribe.cpp@glimpse-diarization` più il crate
   `glimpse-speech`) **[V esistenza, I idoneità]**: è un port indipendente con GGUF Q8_0
   pubblicati, ma è un fork di terzi con un formato diverso da quello che arriverà upstream.
   Sconsigliato come dipendenza.

### 8.6 Conseguenze per Sbobino

**Diarizzazione dal vivo durante la Registrazione**
- **[V]** Con `transcribe-cpp` 0.2.4 o 0.3.0 non si può: il modello non è supportato.
- **[V]** Con la PR #175 mergiata si può: uno `Stream` di diarizzazione per ogni flusso audio e
  `snapshot().speaker_segments` dopo ogni `feed`. I turni chiusi sono definitivi, quello aperto
  si allunga. Con la trascrizione dal vivo dell'ADR-0004 ogni Frase (con `inizio_ms`/`fine_ms`)
  riceve il Parlante quando la diarizzazione ha coperto il suo intervallo. Si usa la regola di
  massima sovrapposizione della §4.3.
- **[I] Il costo decide il preset.**
  - A **1,04 s** il modello lavora a RTF ~0,40 su un M4 Max con 12 thread. Su un portatile x64
    medio, insieme all'ASR dal vivo e magari a due Ingressi, rischia di non stare al passo.
  - A **30,4 s** in stream (`VeryHighLatency` nello slot STREAM, permesso e bit-identico al run)
    costa RTF ~0,02, quasi nulla. I Parlanti arrivano però con circa 30 s di ritardo, e l'output
    finale è quello offline, il più accurato. Questo ritardo si sposa bene con l'ADR-0004: le
    Frasi sono già in coda e il Bino riceve i Parlanti definitivi a Stop.
  - **Proposta:** usare `VeryHighLatency` come default della modalità dal vivo e `LowLatency`
    solo come opzione per PC potenti.
- **Ingressi separati, due istanze.** Si può fare, ma per l'Ingresso microfono serve a poco: di
  solito c'è una sola voce, "Io". **[I] Proposta:** diarizzare dal vivo solo l'audio di sistema,
  oppure il mix quando non ci sono Ingressi separati. Così basta un'istanza e il microfono resta
  "Io" (§4.5). Con due istanze raddoppiano RAM e CPU, per il vincolo di un solo stream per `Model`.

**Costi**

| Voce | Valore | Stato |
| --- | --- | --- |
| Disco | Q8_0 ~106 MB (NVIDIA o Glimpse); BF16 della PR ~200 MB (stima dal `.nemo` BF16) | V / I |
| RAM pesi su CPU | Q8_0: circa la dimensione del file, senza conversione. BF16: convertito in F32, "+~400 MB" | V |
| RAM che cresce con la durata | push-audio: probabilità 11,5 MB/ora più i segmenti. `run`: in più il mel intero (128 × f32 ogni 10 ms ≈ 184 MB/ora) e il PCM se lo tiene il chiamante | V (11,5 MB) / I (calcolo del mel) |
| Grafo per passo | costante: al massimo 541-684 frame × d=512 × 31 strati, ordine delle decine di MB | I |
| CPU | 30,4 s: RTF ~0,02. 1,04 s: RTF ~0,40 (M4 Max, 12 thread) | V su M4, I su x64 |
| VRAM (Vulkan) | non misurata: GPU allo Stage 6. Su GPU i pesi restano BF16 (~200 MB) | V (stato) / I (valore) |

**Conviene rispetto a Sortformer 4spk v2.1?**
- **[V]**
  - Qualità: DER più basso dal 30% al 50% su riunioni e 3+ parlanti, conteggio dei parlanti
    molto migliore, a parità o quasi su 2 parlanti al telefono.
  - Parlanti: 8 invece di 4.
  - Streaming vero nell'API (nella PR).
  - Licenza più semplice.
  - File più piccolo (106 MB contro 139 MB in Q8_0) e base di training più multilingue.
- **[V] Svantaggi.** Il supporto non è rilasciato: PR aperta, senza GGUF ufficiale `handy-computer`
  né benchmark x64/Vulkan. A 1,04 s costa molto più di 30,4 s.
- **[I]** Per Sbobino è la scelta migliore appena la PR entra in una release di `transcribe-cpp`.
  L'integrazione è quasi identica a quella prevista per Sortformer, perché `speaker_segments`,
  `RunExtension` e la gestione del modello accessorio sono gli stessi. Conviene quindi non
  investire in Sortformer v2.1 adesso.

### 8.7 Tabella comparativa aggiornata

| Opzione | Dipendenze | Modello | Parlanti | Offline / dal vivo | Italiano | Stato / rischi |
| --- | --- | --- | --- | --- | --- | --- |
| **A'. Nemotron-3-Diarization via `transcribe-cpp`** (PR #175) | nessuna nuova (aggiornamento del crate quando esce) | GGUF da convertire (BF16 ~200 MB) o Q8_0 se pubblicato. OpenMDW-1.1 | auto, max 8 | `run` e `Session::stream` (segmenti incrementali) | non misurato; training multilingue (YODAS, 21 lingue) | PR aperta; nessun GGUF ufficiale; GPU e benchmark non fatti; licenza "da interpretare" per il maintainer |
| A. Sortformer v2.1 via `transcribe-cpp` | nessuna (0.2.4) | Q8_0 139 MB, NVIDIA OML | auto, max 4 | solo `run` | non misurato; "primarily English" | disponibile oggi; qualità inferiore (DER AMI 15,8% contro 9,3%) |
| G. NeMo-Speech.cpp (C API) | DLL native più FFI scritta a mano | Q8_0 NVIDIA 107 MB (`08456d9e…`) | auto, max 8 | offline e stream push, post-processing NeMo | come A' | possibile conflitto tra due ggml nello stesso processo; packaging; niente crate |
| H. ONNX + `ort` | nessuna nuova | ONNX int8 120 MB / fp16 199 MB | auto, max 8 | quello che si implementa | come A' | mel, FIFO e AOSC da scrivere in Rust; rischio di scambio delle etichette |
| B. `speakrs` | §5 | §5 | soglia | solo offline | community-1 multilingue | §5 |
| C/D. sherpa-onnx / pyannote-rs | §5 | §5 | §5 | solo offline (sherpa: nessun Sortformer, #3497/#4006) | — | §5 |
| E. Ingressi separati (Io / Altri) | nessuna | nessuno | 2 | dal vivo | indipendente | §5; si combina con A' sull'audio di sistema |

### 8.8 Raccomandazione aggiornata (proposta, decide l'utente)

**Puntare su Nemotron-3-Diarization via `transcribe-cpp` (A') invece che su Sortformer v2.1, e
aspettare il merge della PR #175 in una release prima di scrivere codice di diarizzazione.**

1. Il port upstream è del maintainer, ha la parità di DER con NeMo ed espone già in Rust sia
   `run` sia lo streaming. Rispetto al piano della §6 cambia solo il nome dell'estensione, quindi
   il lavoro di §4.2-4.7 resta valido.
2. A posteriori: `run` con `VeryHighLatency`, come previsto in §4.2.
3. Dal vivo, solo con la Trascrizione dal vivo attiva: uno `Stream` a `VeryHighLatency` (RTF
   ~0,02, Parlanti in ritardo di circa 30 s) sul mix o solo sull'audio di sistema, con "Io" per il
   microfono. `LowLatency` (1,04 s) va offerto solo se una misura su x64 mostra margine. Va
   aggiornata l'ADR-0004, che oggi dice "Diarizzazione solo a posteriori".
4. Il passaggio a 0.3.0 si può fare subito e separatamente: per Sbobino non ci sono
   incompatibilità di sorgente (§8.4). Così il salto alla release con Nemotron sarà piccolo.

Cose da fare prima di decidere **[I]**:
- seguire la PR #175 e verificare se `handy-computer` pubblicherà un GGUF (ed eventualmente un
  Q8_0) con URL e SHA fissabili in `models.json`;
- nel frattempo fare la prova di qualità sull'italiano della §6 con il CLI di NeMo-Speech.cpp
  v0.2.0 per Windows (`nemo-speech diarize file.wav`, Q8_0 di NVIDIA), fuori dall'app. Si può
  provare anche `--live` per sentire la latenza reale su x64;
- se la PR resta ferma a lungo, il ripiego è G (NeMo-Speech.cpp via FFI), dopo aver verificato
  la convivenza delle due copie di ggml. H (ONNX) solo come ultima risorsa.

### 8.9 Punti aperti
- Data di merge della PR #175 e pubblicazione del GGUF ufficiale.
- Accuratezza del Q8_0 nel formato upstream: la PR ha validato solo il BF16.
- RTF reale su CPU x64 a 30,4 s e a 1,04 s, e su Vulkan.
- Qualità sull'italiano: nessuna fonte primaria.
- Convivenza di NeMo-Speech.cpp e transcribe.cpp (due ggml) nello stesso processo, solo per G.
