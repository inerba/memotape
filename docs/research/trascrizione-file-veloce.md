# Ricerca: Trascrivi su un file più veloce

Data: 2026-10-04. Scopo: capire dove va il tempo di Trascrivi su un file, e se trascrivere tutto
in una chiamata o a gruppi di Frasi lo rende più veloce. Le decisioni prese da qui sono in
ADR-0007.

Legenda: **[V]** = verificato su fonte primaria (file e riga, o misura). **[I]** = inferenza.

## 0. Fonti e macchina

| Fonte | Riferimento |
| --- | --- |
| `transcribe-cpp` / `-sys` | crate 0.2.4, sorgenti nel registry cargo locale |
| Codice dell'app | commit `1bf5d34` |
| Macchina | Ryzen 7 3700X, RTX 2070 SUPER (8 GB), Vulkan; test in `--release` |
| Audio | `D:\Video\2026-05-22 11-49-57.mp4`: riunione in italiano, 618,6 s, 103 Frasi, 526,2 s di parlato |

Path abbreviati: `tc/…` = crate `transcribe-cpp-0.2.4`, `tcs/…` = crate `transcribe-cpp-sys-0.2.4`.

## 1. Misura

Un test `#[ignore]` temporaneo (tolto dopo la misura), Lingua del parlato `it`:
1. solo decodifica e ricampionamento, con `transcribe_file`, un motore che scarta i frame e un VAD
   sempre silenzioso;
2. solo Silero e segmentatore sull'audio già decodificato, raccogliendo le Frasi;
3. per ogni modello, dopo una Frase di riscaldamento:
   - "oggi": `transcribe_file` com'è, con decodifica, VAD e motore in sequenza;
   - `Session::run` su ogni Frase già pronta (solo motore);
   - `Session::run_batch` a gruppi di 4, 8, 16 e 32 Frasi.

Memoria della GPU campionata con `nvidia-smi` ogni 500 ms (base 3,3–4,2 GB occupati da altre app).

**[V] Risultati** (secondi; Parakeet misurato due volte):

| | oggi | `run` per Frase, solo motore | `run_batch` |
| --- | --- | --- | --- |
| Decodifica | 1,4–1,5 | | |
| VAD + segmentatore | 4,3–5,5 | | |
| Nemotron | 93,5 | 39,4 | il processo si chiude: `GGML_ASSERT(ggml_can_repeat(b, a))` (`ggml.c:2264`) |
| Parakeet | 19,6 / 24,0 | 12,6 / 15,4 | stesso assert; con le Frasi allungate di silenzio alla stessa lunghezza `GGML_ASSERT(ggml_nelements(a) == ne0*ne1*ne2*ne3)` (`ggml.c:3769`) |
| Whisper | 41,6 | 35,6 | da 4: 38,8 (gruppo più lento 2,3) · da 8: 37,8 (4,1) · da 16: 40,9 (9,3) · da 32: 59,9 (23,2), picco 7,7 GB |

- **[V]** Con Whisper e Parakeet "oggi" ≈ decodifica + VAD + motore: le tre fasi si sommano, perché
  la pipeline a trazione legge la fonte solo quando il motore chiede frame. Decodifica e VAD sono
  circa 6 s su 10 minuti: il 30% del tempo con Parakeet, il 14% con Whisper.
- **[V]** Nemotron sui file oggi usa lo stream (un `feed` ogni 30 ms, per i Parziali): 93,5 s. Con
  `run` sulla Frase intera 39,4 s, 2,4 volte meno. Il testo è praticamente uguale (1179 parole
  contro 1171): punteggiatura, qualche parola diversa in un verso o nell'altro, mezza riga persa
  in un punto.
- **[V]** `run_batch` non serve: con Parakeet e Nemotron (famiglia parakeet) chiude il processo in
  0.2.4, con Whisper non è più veloce di `run` e un gruppo dura di più.

## 2. Una chiamata sola sul file intero

- **[V]** Niente progresso: l'API C non ha callback di avanzamento, solo log e abort
  (`tcs/include/transcribe.h:397`, `:1655`). `Session::run` è una chiamata FFI bloccante
  (`tc/src/session.rs:150`).
- **[V]** Annulla: Whisper controlla l'abort a ogni finestra da 30 s e a ogni token
  (`tcs/src/arch/whisper/model.cpp:1624`, `:2192`); Parakeet e Nemotron offline solo prima di
  partire ("the single observation point on this path", `tcs/src/arch/parakeet/model.cpp:969-972`).
  Un'ora con Parakeet non si annullerebbe.
- **[V]** Memoria: Parakeet offline fa un solo passaggio dell'encoder sull'audio intero
  (`parakeet/model.cpp:1345-1373`), con buffer di attenzione O(T²) (`parakeet/multitalker.cpp:73`).
  Whisper divide comunque in finestre da 30 s (`whisper/model.cpp:1622-1630`).
- **[V]** Il risultato ha i tempi: segmenti per Whisper, token e parole per Parakeet
  (`tc/src/result.rs:17-75`; `whisper/capabilities.cpp:18-21`, `parakeet/capabilities.cpp:26-29`).
- **[I]** Senza VAD Whisper trascrive anche i silenzi (8% di questo file, molto di più in una
  lezione con pause) e tende a inventarvi testo.

## 2b. Dopo ADR-0007

**[V]** Stessa macchina e stesso file:
- Nemotron 31,6 s;
- Parakeet 15,1 s;
- Whisper 37,7 s.

Dettagli in `.scratch/sbobino-v2/issues/10-trascrivi-piu-veloce.md`.

## 3. Altro

- **[V]** `run` è ammesso su un modello in streaming: il dispatcher controlla `supports_streaming`
  solo a `stream_begin` (`tcs/src/transcribe.cpp:1778`). Il wrapper Rust rifiuta `run` con `Busy`
  se c'è uno stream attivo sullo stesso modello (`tc/src/session.rs:157-167`).
- **[V]** Una Frase dura al massimo 18 s (`max_phrase_ms`, `src-tauri/src/audio_toolkit/segmenter.rs:19-28`):
  con `run` per Frase, Annulla con Nemotron e Parakeet aspetta al massimo una Frase (circa 0,4 s
  con Nemotron su questa macchina).
- **[I]** La prima Trascrizione dopo il caricamento paga il riscaldamento di Vulkan (con Whisper 16 s
  contro 0,8 s sulla fixture, vedi `AGENTS.md`). Le misure sopra lo escludono con una Frase di
  riscaldamento.
