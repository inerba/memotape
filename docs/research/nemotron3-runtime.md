# Nemotron 3: runtime e modello locale

Prova del ticket [01](../../.scratch/nemotron3-diarizzazione/issues/01-runtime-e-modello-locale.md),
eseguita il 2026-10-05. Questo documento riguarda il runtime e gli artefatti locali;
non attesta l'integrazione dei Parlanti dal vivo nella Registrazione.

## Sorgenti e conversione

- Runtime, binding Rust e sys: [transcribe.cpp PR #175](https://github.com/handy-computer/transcribe.cpp/pull/175),
  commit `e6672a8672913b47f1571c66c54bee789d028416`.
- Checkpoint: [`nvidia/Nemotron-3-Diarization`](https://huggingface.co/nvidia/Nemotron-3-Diarization/tree/f667ed73aee57d40cc39428eb768b4fd87a0a29e),
  revisione `f667ed73aee57d40cc39428eb768b4fd87a0a29e`, file `Nemotron-3-Diarization.nemo`.
- Converter: `scripts/convert-nemotron3_diar.py` del commit del runtime, senza modifiche.
- Ambiente: Ubuntu in WSL2, Python 3.12.3, uv 0.12.23. `uv sync --frozen` usa il
  `uv.lock` upstream in `scripts/envs/nemotron3_diar`. Questo ambiente upstream
  esclude Windows; WSL serve alla preparazione, le inferenze si eseguono su Windows x64.
- Versioni effettive: Torch 2.14.0, NumPy 2.5.3, GGUF 0.19.0,
  NeMo al commit `cf724ac337d1ebc7d0dda1e23fb80916f52927a5`, Transformers al commit
  `4b28d51d0d5f17ec20c23a187d0475a8e68810c8`. L'ambiente completo è in
  [conversion-freeze.txt](nemotron3-runtime/conversion-freeze.txt).

| Artefatto | Byte | SHA-256 |
| --- | ---: | --- |
| Checkpoint NVIDIA `.nemo` | 198676480 | `867c53f552998f772e5b5e5c082962ae85ee7ca5669c2bc17d7f615133d4e96d` |
| GGUF BF16 locale | 198937280 | `4b11ce10e009fedf496cc9f879dc634e67605ecbf240463a155e0128657019e3` |
| Converter | 15364 | `981ae25bfd38d3ac06f06b50589e1a87a6be251568f549cdadfb8fd105ca648b` |
| Lock Python upstream | 227407 | `961dd0ce6aa7b6067d8c67809cff242d5c4592cd41c6c370af8bceecafef2336` |

Il download del checkpoint è stato confrontato con il digest LFS pubblicato da
NVIDIA. Due conversioni nello stesso ambiente hanno prodotto lo stesso SHA-256
del GGUF. Il converter emette 357 tensori: 228 F32, 128 BF16 e 1 F16; scarta sei
tensori di teste usate soltanto in addestramento. "BF16" è il nome del formato di
riferimento upstream, non significa che ogni tensore abbia quel dtype.

Comandi della conversione, dalla sorgente upstream fissata (percorsi locali di
questa prova):

```bash
export UV_PROJECT_ENVIRONMENT=/tmp/memotape-nemotron3-env
export UV_CACHE_DIR=/tmp/memotape-nemotron3-cache
export UV_LINK_MODE=copy
uv sync --frozen --project scripts/envs/nemotron3_diar
/tmp/memotape-nemotron3-env/bin/python scripts/convert-nemotron3_diar.py \
  ../Nemotron-3-Diarization.nemo \
  --repo-id nvidia/Nemotron-3-Diarization \
  --revision f667ed73aee57d40cc39428eb768b4fd87a0a29e \
  --out ../Nemotron-3-Diarization-BF16.gguf
```

Il `.nemo` va prima scaricato dall'URL `resolve` della revisione sopra e verificato
contro hash e dimensione della tabella: passando un file locale, `--revision`
registra il contesto del comando ma non valida da solo la provenienza del file.
La cartella dell'uscita viene scritta dal converter in `stt.variant`: per ottenere
lo stesso hash di questa prova deve chiamarsi `nemotron3-proof`.

Non sono stati pubblicati pesi né aggiunti download al catalogo dell'app.
Il checkpoint e il GGUF verificato sono conservati localmente in
`D:\local\tauri\sbobino-deps\nemotron3-proof`, fuori da Git e dai target di Cargo;
la sorgente upstream fissata è nella sottocartella `transcribe.cpp`.

## Build Windows e provenienza delle DLL

Hardware: AMD Ryzen 7 3700X, NVIDIA GeForce RTX 2070 SUPER. Rust 1.96.1 MSVC,
Visual Studio 2022/toolset 14.44.35207, CMake 4.3.4, Vulkan SDK 1.4.357.0.
Il probe in [`tools/nemotron3-runtime`](../../tools/nemotron3-runtime/README.md)
è un crate separato con lock e revisione Git esplicita. La prima compilazione
non usa `TRANSCRIBE_DIR`; C++ e DLL vengono dalla sorgente di quel commit.
Le DLL del probe hanno una directory distinta da quelle dell'app.

Il probe ha verificato binding e runtime `0.2.3`, commit nativo `e6672a8`, ABI
`ae25d09c2b7b325b`. Il numero di versione del branch non è un'indicazione di
compatibilità con il crate pubblicato `0.2.4`. Gli hash delle DLL del probe sono
in [probe-dlls.json](nemotron3-runtime/probe-dlls.json).
Tutte le 13 DLL transcribe/ggml preparate da `build.rs` in `runtime-libs` dell'app
coincidono byte per byte con quelle del probe. Per evitare una seconda compilazione
C++ identica, i controlli dell'app hanno usato `TRANSCRIBE_DIR` nella sola config
Cargo locale, puntato all'install prefix prodotto dal probe (`out`, con
`lib/transcribe-link.json`: versione `0.2.3`, commit `e6672a8`). Senza quell'override
Cargo compila la stessa sorgente Git fissata nel manifest e nel lock dell'app.

La configurazione locale copiata dal checkout principale inizialmente usava
`LOCALAPPDATA = { value = "src-tauri/target", relative = true }`: Cargo conserva
il separatore `/`, che `mklink` nel commit fissato rifiuta. Inoltre la junction
deve avere un percorso realmente breve per MSBuild. Una junction in AppData
risultava leggibile da PowerShell ma CMake non riusciva a usare le sue directory.
La configurazione **locale e ignorata da Git** usa quindi come base
`D:\local\tauri\sbobino-deps`, producendo junction univoche in `tcs`.
Non è stato modificato il build script upstream.

## Esiti nativi

Gli smoke usano un processo distinto per backend/modello/modalità, in sequenza,
con selezione esplicita CPU/Vulkan e controllo del dispositivo effettivo.
Il runner verifica prima dimensione e SHA-256 dei quattro modelli del catalogo,
del BF16 e delle due fixture; rifiuta artefatti sostituiti.
La fixture di Diarizzazione è `parlato-due-voci.wav` (25,675 s, sintetica);
quella ASR è `parlato-it.wav` (8,960 s). Gli hash delle fixture sono
rispettivamente `c300d92aed389cbf6764e6acb3323f89f23de2d0f6f9c589ea88bd55dd7db42a`
e `56c338e08e699a5a99749b75b1da25fad005b0ab1a5236b66aea0a310a2b9465`.

Il BF16 ha superato offline con `VeryHighLatency` e streaming con `LowLatency`
su entrambi i backend. Ogni esito contiene due Parlanti e segmenti con tempi
validi; in streaming compaiono segmenti prima di `finalize`.
L'accettazione richiede due voci alternate A/B/A/B nelle quattro finestre
centrali della fixture, almeno 14 s coperti, assenza di sovrapposizioni oltre
200 ms e limiti temporali entro l'audio più 80 ms di padding. Ogni ASR deve
restituire tutte le parole attese, ignorando solo forma e punteggiatura;
l'hint di lingua e l'eventuale rilevamento devono essere coerenti con l'italiano.
Whisper in automatica deve rilevare `it`.
Tutte le 20 esecuzioni native sono riuscite. Gli esiti completi (capacità, testo,
tempi, segmenti, timestamp e codici di uscita) sono in
[smoke-results.json](nemotron3-runtime/smoke-results.json).

| Modello e modalità | CPU | Vulkan | Osservazioni sulla fixture |
| --- | --- | --- | --- |
| Nemotron 3 offline | riuscito | riuscito | 2 Parlanti |
| Nemotron 3 streaming | riuscito | riuscito | 2 Parlanti, segmenti prima di `finalize` |
| Nemotron ASR offline, `it` e automatica | riuscito | riuscito | testo italiano; 43 token, 16 parole |
| Nemotron ASR streaming, `it` e automatica | riuscito | riuscito | 6 cambi di Parziale prima di `finalize`; 43 token, nessuna parola |
| Parakeet offline, automatica | riuscito | riuscito | testo italiano; 30 token, 14 parole |
| Whisper offline, `it` e automatica | riuscito | riuscito | testo italiano; 2 segmenti, nessun token/parola |
| Sortformer offline | riuscito | riuscito | 2 Parlanti |

La dipendenza dell'app viene ora fissata alla revisione provata. Il catalogo,
i modelli predefiniti e l'interfaccia non cambiano; l'integrazione Nemotron 3
nella pipeline è oggetto dei ticket successivi.

Questi smoke non misurano la qualità sul parlato italiano reale, la latenza
audio → etichetta dell'app, la concorrenza ASR/Diarizzazione o Registrazioni lunghe.
I tempi di esecuzione includono chiamate a freddo e non sono benchmark ripetuti.

## Controlli del repository

I sei controlli sono riusciti: `bun run typecheck`, `bun run test` (89 test),
`bun run check`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`,
`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`,
`cargo test --manifest-path src-tauri/Cargo.toml` (192 test, 2 smoke ignorati).
Le compilazioni Rust sono state eseguite in sequenza.
I due smoke ignorati sono poi stati lanciati separatamente con
`cargo test --locked --manifest-path src-tauri/Cargo.toml -- --ignored --test-threads=1 --nocapture`:
entrambi riusciti. Verificano la pipeline dell'app con i tre ASR su WAV (`it`) e
MP4/AAC (automatica), e l'attribuzione di due Parlanti alternati con Sortformer.
Anche `cargo fmt` e `cargo clippy --all-targets -- -D warnings` del crate separato
del probe sono riusciti.
La revisione ha fatto rafforzare questi criteri: due test prima fallivano sui
falsi positivi «trascrizione testo» e un segmento minimo. I cinque test del
probe ora passano, includendo lingua errata, segmenti oltre la durata,
sovrapposizioni, copertura insufficiente e alternanza errata.
Il controllo negativo del runner rifiuta dimensioni e digest errati.
La matrice finale è stata ripetuta dopo queste correzioni: 20/20 riusciti.
Il riesame Standards e Spec si è chiuso senza rilievi residui; anche i
riferimenti al runtime e le insidie di build in `AGENTS.md` sono aggiornati.

Il clone upstream era inizialmente sotto `src-tauri/target`: `bun test src`
includeva anche i test TypeScript del clone e falliva per il loro bundle assente.
Dopo lo spostamento del clone fuori dal repository, la suite frontend passa.
Non è stato cambiato il comando del repository né escluso alcun suo test.

## Tempi ASR nel commit fissato

La sorgente pubblica espone `Capabilities::max_timestamp_kind` e
`Transcript::{timestamp_kind, segments, words, tokens}`. I tempi sono relativi
all'audio passato alla chiamata, in millisecondi; nella pipeline dell'app andrà
aggiunto l'inizio della Frase.

- Nemotron ASR e Parakeet hanno capacità `Token`. Il runtime ricava i tempi
  dall'indice di emissione e dal salto del decoder, sulla griglia dell'encoder;
  non sono un allineamento fonetico. Token distinti possono sovrapporsi o avere
  durata zero. Le parole offline sono aggregazioni dei token su confini del
  tokenizer, non una seconda stima dei tempi.
- Lo snapshot Nemotron in streaming espone token ma, nella prova, nessuna parola
  o segmento di testo. Il risultato offline espone anche parole e un segmento.
  La granularità token non garantisce quindi la presenza di `words` dal vivo.
- I tempi possono includere padding: il segmento offline di Nemotron termina a
  9040 ms sulla fixture da 8960 ms. Non vanno interpretati come limiti esatti del
  parlato o della Sorgente.
- Whisper dichiara `Segment`; non si devono promettere tempi di parola/token
  sufficienti a separare una Frase con più voci.
- Nemotron 3 è un diarizer: restituisce `speaker_segments`, senza testo, token o
  parole. `timestamp_kind: None` riguarda il testo e non l'assenza dei tempi dei
  Parlanti. Nel GGUF il passo d'uscita del diarizer è 10 ms; questa risoluzione
  numerica non certifica l'accuratezza dei confini delle voci.

Riferimenti al commit immutabile:
[risultati Rust](https://github.com/handy-computer/transcribe.cpp/blob/e6672a8672913b47f1571c66c54bee789d028416/bindings/rust/transcribe-cpp/src/result.rs),
[tempi Parakeet/Nemotron ASR](https://github.com/handy-computer/transcribe.cpp/blob/e6672a8672913b47f1571c66c54bee789d028416/src/arch/parakeet/model.cpp),
[capacità Whisper](https://github.com/handy-computer/transcribe.cpp/blob/e6672a8672913b47f1571c66c54bee789d028416/src/arch/whisper/capabilities.cpp),
[converter](https://github.com/handy-computer/transcribe.cpp/blob/e6672a8672913b47f1571c66c54bee789d028416/scripts/convert-nemotron3_diar.py).
