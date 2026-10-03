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
