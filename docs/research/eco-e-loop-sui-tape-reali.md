# Ricerca: l'eco del prompt di Whisper e il loop di Cohere sui Tape reali

Data: 2026-10-16. Scopo: misurare, sui due Tape dell'utente, perché Whisper scrive Frasi fatte solo di «Glossary», «Slaitech, Laravel» e simili, e perché Cohere Transcribe ferma la Trascrizione con `Error::OutputRepetition`. Il documento raccoglie fatti e misure, non propone modifiche al codice.

Legenda: **[V]** = misurato o letto nel sorgente (file e riga). **[I]** = inferenza da confermare.

## 0. Fonti e versioni

| Fonte | Riferimento |
| --- | --- |
| Memotape | worktree di `8928f230` (versione 0.5.3), `transcribe-cpp` dal tag `v0.4.0` |
| transcribe.cpp | tag `v0.4.0`, commit `f1dcd15858c4bbba87ca97862b4e448b2cf15b8f` (da `src-tauri/Cargo.lock`), checkout in `~/.cargo/git/checkouts/transcribe.cpp-0f9edbe501cad03c/f1dcd15` |
| Whisper Medium | `whisper-medium-Q8_0.gguf`, `handy-computer/whisper-medium-gguf` rev `ec78f06` |
| Whisper Large v3 Turbo | `whisper-large-v3-turbo-Q5_K_M.gguf`, rev `5eaf945` |
| Nemotron 3.5 | `nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf`, rev `6d44e54` |
| Cohere Transcribe | `cohere-transcribe-03-2026-Q5_K_M.gguf`, rev `dfa4ade` |
| Silero | `src-tauri/resources/silero_vad.onnx`, soglia del segmentatore 0,4 (`audio_toolkit/segmenter.rs:26`) |
| Hardware | RTX 2070 SUPER su Vulkan, `GGML_VK_DISABLE_COOPMAT2=1` come nell'app (`engine/transcribe_cpp.rs:330`) |
| Tape | `Registrazione 2026-10-16 15-11-13.tape` (56,4 s, Ingressi separati) e `Registrazione 2026-10-16 15-04-07.tape` (91,9 s, Ingressi separati), copiati in una cartella temporanea; gli originali non sono stati toccati (a fine lavoro hanno lo stesso SHA-256 delle copie) |

I percorsi di transcribe.cpp sono relativi alla root del suo repo, quelli di Memotape a `src-tauri/src/`.

**Il Vocabolario.** Oggi `settings.json` ha `"vocabolario": []` [V]: l'utente l'ha svuotato dopo le Trascrizioni. Le misure usano i Termini indicati nella richiesta, `["Slaitech", "Laravel"]`.

**Storia dei due Tape, dal log** (`%LOCALAPPDATA%\it.memotape.desktop\logs\Memotape.log`, ore UTC) [V]:
- 15-11-13: registrato con Whisper Medium dal vivo (13:11:15), ritrascritto con Whisper Large v3 Turbo (13:27:17), poi due tentativi con Cohere (13:27:38 e 13:31:40), finiti senza Tape. Il documento attuale è quello di Turbo (`"modello": "whisper-large-v3-turbo-q5km"`). Il log non contiene l'errore di Cohere: Memotape non lo scrive.
- 15-04-07: registrato con Nemotron dal vivo; nessuna ritrascrizione nel log.

## 1. Metodo

Il codice di prova era in un worktree temporaneo e non è nel repository (legge la traccia di Whisper da un puntatore privato di `Session`): `engine/pipeline/eco_e_loop.rs`, incluso da `engine/pipeline.rs` con `#[cfg(test)] mod eco_e_loop`. Era un test `#[ignore]`:

```
ECO_TAPE="<a.tape>;<b.tape>" ECO_OUT=<cartella> ECO_FASI=whisper-medium,whisper-turbo,nemotron,cohere \
cargo test --manifest-path src-tauri/Cargo.toml --lib eco_e_loop -- --ignored --test-threads=1 --nocapture
```

`ECO_SOLO=microfono|sistema` limita i modelli a un Ingresso.

1. **Le Frasi come le riceve il motore** [V]. Per ogni Ingresso il test chiama `transcribe_decoded(Decoder::open_ingresso(tape, ingresso), …)` con Silero, come Trascrivi su un Tape (`managers/transcription.rs:437-438`, senza pulizia né Sensibilità). Al posto del motore c'è un registratore che conserva i campioni di ogni Frase. Così ogni modello riceve esattamente l'audio che riceverebbe nell'app, con prefill e hangover. Le Frasi del Tape 15-11-13 tornano con gli stessi `inizio_ms` del documento. Quelle del 15-04-07 no: il suo documento viene dalla Trascrizione dal vivo, che taglia diversamente.
2. **Audio.** RMS e picco della Frase in dBFS. Media della probabilità di Silero sui frame della Frase e quota di frame sopra 0,4 (la sequenza di Silero è quella della pipeline). Le stesse misure sull'altro Ingresso nello stesso intervallo.
3. **Rientro delle casse.** Due misure per ogni Frase del Microfono. La correlazione normalizzata della forma d'onda (a 8 kHz) fra Microfono e Audio di sistema in ritardo da −50 a +400 ms. La correlazione degli inviluppi da 10 ms in dB. Poi, con numpy sull'audio intero a 16 kHz, la coerenza spettrale fra 300 e 3400 Hz con l'Audio di sistema in ritardo.
4. **Whisper.** `Session::run` con le opzioni che l'app costruisce (`engine/transcribe_cpp.rs:114-122`): `timestamps` dal modello e lingua `it`. Le varianti:

| Nome | Prompt | Soglie |
| --- | --- | --- |
| `vocab_ns0` | `vocabulary: ["Slaitech", "Laravel"]` (prompt ` Glossary: Slaitech, Laravel`) | `no_speech_thold: 0`, **quello che fa Memotape oggi** |
| `vocab1_ns0` | `vocabulary: ["Laravel"]` | `no_speech_thold: 0` |
| `vuoto_ns0` | nessuno | `no_speech_thold: 0` |
| `initial_ns0` | `WhisperRunOptions::initial_prompt: "Slaitech, Laravel"` (il Memotape di prima) | `no_speech_thold: 0` |
| `contesto_ns0` | `RunOptions::prompt: "Slaitech, Laravel."` (`Feature::ContextPrompt`, senza «Glossary:») | `no_speech_thold: 0` |
| `frase_ns0` | `RunOptions::prompt: "Si parla di Slaitech e di Laravel."` | `no_speech_thold: 0` |
| `vocab_def`, `vuoto_def`, `initial_def` | come sopra | predefinite di Whisper (0,6 e −1) |

5. **Traccia di decodifica di Whisper** [V]. I binding Rust 0.4.0 non la espongono: c'è solo l'FFI `sys::transcribe_get_whisper_chunk_count` e `sys::transcribe_get_whisper_chunk_trace` (`bindings/rust/sys/src/transcribe_sys.rs:1891-1941`, `include/transcribe/whisper.h:174-211`), che vogliono il puntatore nativo della sessione. `Session` lo tiene privato (`bindings/rust/transcribe-cpp/src/session.rs:85-93`, campi `ptr`, `model: Arc`, `cancel: Option<Arc>`). Il test lo ricava dalla disposizione in memoria: verifica le dimensioni e sceglie la parola diversa da zero e dall'`Arc` del modello. Funziona solo senza `CancelToken` installato ed è un ripiego da test, non da app. Per ogni finestra da 30 s la traccia dà `no_speech_prob`, `avg_logprob`, `compression_ratio`, `temperature_used`, `n_fallbacks` e `no_speech_triggered`.
6. **Nemotron e Cohere.** `Session::run` con lingua `it` (`it-IT` per Nemotron, da `resolve_language`), come l'app. Su `Error::OutputRepetition` il test registra `Error::partial()`, poi trascrive tre alternative: le due metà divise nel punto più silenzioso (minimo dell'RMS su 3 frame da 30 ms fra il 25% e il 75% della Frase); la Frase senza i 300 ms di prefill e senza la coda sotto −40 dBFS; la stessa Frase con `TRANSCRIBE_NO_REPETITION_GUARD=1`, in un processo a parte.
7. **Audio amplificato.** Le Frasi del Microfono portate a −25 dBFS di RMS e ritrascritte, per vedere se sotto il rumore c'è parlato comprensibile.
8. **Riferimento per il parlato vero.** Per le Frasi dell'Audio di sistema il testo di Nemotron serve da riferimento indipendente (Nemotron non riceve prompt). La somiglianza è `difflib.SequenceMatcher` sulle parole. Una Frase vale «persa o sostituita» sotto 0,3. Contano solo le Frasi con almeno 3 parole in Nemotron.

**Ripetibilità** [V]. Due esecuzioni complete hanno dato gli stessi testi per tutte le varianti decodificate a temperatura 0. Cambiano solo i valori delle Frasi accettate a temperatura > 0 (il seed predefinito è 0, cioè casuale, `whisper.h:139-145`). Il loop di Cohere si ripete identico in due esecuzioni.

## 2. Problema 1: l'eco del prompt di Whisper

### 2.1 Cosa c'è nel Microfono

**Tabella A.** audio delle Frasi del Microfono (Tape 15-11-13)

| Frase (ms) | Durata | RMS dBFS | Picco dBFS | Silero media / sopra 0,4 | Sistema RMS dBFS | Sistema Silero sopra 0,4 | r onda (lag ms) | r inviluppo (lag ms) |
|---|---|---|---|---|---|---|---|---|
| 2250–3390 | 1140 | -71.9 | -46.3 | 0.09 / 0.11 | -37.7 | 0.24 | 0.046 (5) | 0.35 (-40) |
| 3390–4380 | 990 | -73.3 | -57.3 | 0.15 / 0.15 | -77.7 | 0.00 | 0.070 (2) | 0.44 (320) |
| 4470–5550 | 1080 | -76.0 | -62.5 | 0.09 / 0.06 | -24.3 | 0.00 | -0.218 (14) | 0.33 (270) |
| 8670–9870 | 1200 | -74.4 | -58.3 | 0.18 / 0.15 | -45.1 | 0.00 | -0.027 (107) | 0.69 (0) |
| 10620–13530 | 2910 | -73.0 | -54.3 | 0.31 / 0.32 | -17.3 | 0.64 | -0.117 (129) | 0.21 (260) |
| 13710–17310 | 3600 | -71.3 | -52.0 | 0.38 / 0.39 | -15.2 | 0.47 | -0.080 (129) | 0.27 (140) |
| 17310–18630 | 1320 | -70.2 | -57.8 | 0.30 / 0.39 | -15.5 | 0.39 | -0.059 (129) | 0.52 (-50) |
| 18630–20010 | 1380 | -68.4 | -38.9 | 0.37 / 0.39 | -51.5 | 0.00 | 0.024 (107) | 0.52 (110) |
| 20700–23160 | 2460 | -71.2 | -48.1 | 0.38 / 0.40 | -14.2 | 0.46 | 0.033 (128) | 0.03 (210) |
| 23250–25410 | 2160 | -62.3 | -33.6 | 0.31 / 0.26 | -18.4 | 0.82 | -0.028 (270) | 0.11 (-40) |
| 26490–29250 | 2760 | -70.3 | -53.0 | 0.46 / 0.46 | -23.9 | 0.85 | 0.070 (128) | 0.36 (330) |
| 31260–33000 | 1740 | -67.1 | -48.3 | 0.41 / 0.41 | -20.7 | 0.52 | 0.086 (396) | 0.38 (400) |

- **[V] Il Microfono del Tape 15-11-13 non contiene parlato.** Per tutti i 56 s l'RMS al secondo sta fra −66 e −83 dBFS, con un massimo di −55. Da 33 s in poi, e a tratti a 25 e 29-30 s, i campioni sono zeri esatti (−114 dBFS dopo l'Opus). Il picco dell'intera traccia è −26 dBFS. Le 12 Frasi del Microfono hanno RMS fra −62,3 e −76,0 dBFS. Le Frasi dell'Audio di sistema hanno RMS fra −13 e −39 dBFS.
- **[V] Silero vede «parlato» nel rumore di fondo.** Sui frame non nulli del Microfono la media è 0,28, il 30% dei frame supera 0,4 e il 90° percentile è 0,77. La correlazione fra livello e probabilità è 0,50. Bastano 60 ms sopra soglia per aprire una Frase. Lo spettro medio è concentrato fra 100 e 2000 Hz (−37/−47 dB), sopra 4 kHz scende a −71 dB.
- **[V] Non è rientro delle casse.** La coerenza fra Audio di sistema e Microfono sui primi 31 s è 0,006 con 129 ms di ritardo, 0,003 a 0 ms, 0,009 a 1 s e 0,001 a 3 s. È rumore di stima: il Microfono non dipende linearmente da quello che esce dalle casse (meno dell'1% della sua potenza). La correlazione della forma d'onda ha il suo massimo a 128,9 ms (|r| = 0,028 contro un 99° percentile di 0,008), e quel ritardo torna in 6 Frasi su 12. Un accoppiamento minimo c'è, ma vale meno dell'1% della potenza. Le correlazioni degli inviluppi per Frase (0,03-0,69) non distinguono: su 1-3 s di rumore sono instabili anche a ritardi casuali (−0,27/+0,03 a ritardi di 1-5 s).
- **[V] Mentre il Microfono «parla», l'Audio di sistema parla in 7 Frasi su 12** (oltre il 39% dei frame sopra soglia), e in 4 tace (0%).
- **[V] Nessun modello sente parlato, nemmeno amplificato** (tabella C). Nemotron dà testo vuoto su tutte le 12 Frasi; amplificate a −25 dBFS, dà «okay» e «Sì.» su 2. Su 26490 ms Whisper Medium e Cohere, con l'audio amplificato, scrivono la stessa breve domanda («Capisci? …»): forse lì c'è una voce lontana di 1 s.
- **[I] Cosa produce quel livello.** Probabilmente il microfono era silenziato o con il guadagno al minimo (gli zeri esatti a tratti fanno pensare a un muto del dispositivo o a un noise gate del driver). In ogni caso le Frasi del Microfono sono rumore, non voce dell'utente né eco.

### 2.2 Whisper sulle Frasi del Microfono

**Tabella B.** Whisper Medium sulle Frasi del Microfono (testo; no_speech_prob / avg_logprob)

| Frase (ms) | Documento | vocab_ns0 | vuoto_ns0 | initial_ns0 | contesto_ns0 | frase_ns0 | vocab_def | vuoto_def |
|---|---|---|---|---|---|---|---|---|
| 2250 | Glossary, Laravel | Glossary (0.83 / -0.86) | Grazie. (0.88 / -0.93) | Slaitech, Laravel (0.77 / -0.50) | Slaitech, Laravel. (0.88 / -0.40) | ∅ (0.96 / -1.06) | Glossary (0.83 / -0.86) | Grazie. (0.88 / -0.93) |
| 3390 | Glossary, Laravel | Glossary (0.85 / -0.74) | Grazie. (0.85 / -0.94) | Slaitech, Laravel (0.79 / -0.51) | Slaitech, Laravel. (0.87 / -0.43) | Grazie. (0.96 / -0.87) | Glossary (0.85 / -0.74) | Grazie. (0.85 / -0.94) |
| 4470 | Glossary, Laravel | Glossary (0.83 / -0.72) | Grazie. (0.91 / -0.86) | Slaitech, Laravel (0.79 / -0.43) | Slaitech, Laravel. (0.88 / -0.31) | Grazie. (0.96 / -0.83) | Glossary (0.83 / -0.72) | Grazie. (0.91 / -0.86) |
| 8670 | Glossary, Laravel | Glossary (0.86 / -0.76) | Sottotitoli creati dalla comunità Amara.org (0.88 / -0.33) | Slaitech, Laravel (0.81 / -0.45) | Slaitech, Laravel. (0.92 / -0.37) | Sottotitoli creati dalla comunità Amara.org (0.95 / -0.37) | Glossary (0.86 / -0.76) | Sottotitoli creati dalla comunità Amara.org (0.88 / -0.33) |
| 10620 | Grazie a tutti. | Glossary (0.77 / -0.64) | ∅ (0.88 / -1.63) | Slaitech, Laravel (0.73 / -0.40) | Slaitech, Laravel. (0.82 / -0.34) | ∅ (0.89 / -1.00) | Glossary (0.77 / -0.64) | ∅ (0.88 / -1.63) |
| 13710 | Glossary, Laravel | Glossary (0.87 / -0.82) | ∅ (0.92 / -1.13) | Slaitech, Laravel (0.83 / -0.51) | Slaitech, Laravel. (0.91 / -0.42) | Slaitech e di Laravel. (0.93 / -0.78) | Glossary (0.87 / -0.82) | ∅ (0.92 / -1.13) |
| 17310 | Glossary, Laravel | Glossary (0.89 / -0.75) | Grazie. (0.87 / -0.92) | Slaitech, Laravel (0.85 / -0.49) | Slaitech, Laravel. (0.93 / -0.41) | Grazie. (0.97 / -0.93) | Glossary (0.89 / -0.75) | Grazie. (0.87 / -0.92) |
| 18630 | Glossary, Laravel | Glossary (0.89 / -0.76) | ∅ (0.87 / -1.28) | Slaitech, Laravel (0.85 / -0.48) | Slaitech, Laravel. (0.90 / -0.44) | Siamo in Slaitech e di Laravel. (0.95 / -0.99) | Glossary (0.89 / -0.76) | ∅ (0.87 / -1.28) |
| 20700 | Glossary, Laravel | Slaitech, Laravel (0.88 / -0.57) | Grazie. (0.88 / -0.88) | Slaitech, Laravel (0.85 / -0.47) | Slaitech, Laravel. (0.91 / -0.36) | Grazie. (0.95 / -0.85) | Slaitech, Laravel (0.88 / -0.57) | Grazie. (0.88 / -0.88) |
| 23250 | Grazie. | Slaitech, Laravel (0.81 / -0.65) | Grazie. (0.85 / -0.85) | Slaitech, Laravel (0.73 / -0.48) | Slaitech, Laravel. (0.91 / -0.46) | Grazie. (0.94 / -0.80) | Slaitech, Laravel (0.81 / -0.65) | Grazie. (0.85 / -0.85) |
| 26490 | Glossary | Slaitech, Laravel (0.71 / -0.48) | ∅ (0.93 / -1.50) | Slaitech, Laravel (0.67 / -0.36) | Slaitech, Laravel. (0.77 / -0.35) | Slaitech. (0.81 / -0.82) | Slaitech, Laravel (0.71 / -0.48) | ∅ (0.93 / -1.50) |
| 31260 | — | Glossary (0.79 / -0.83) | ∅ (0.88 / -1.05) | Slaitech, Laravel (0.76 / -0.46) | Slaitech, Laravel. (0.83 / -0.54) | Ehi! (0.95 / -0.81) | Glossary (0.79 / -0.83) | ∅ (0.88 / -1.05) |

**Tabella C.** Turbo, Nemotron, Cohere e audio amplificato sulle Frasi del Microfono

| Frase (ms) | Turbo vocab_ns0 | Turbo vuoto_ns0 | Turbo contesto_ns0 | Nemotron | Cohere | Medium vuoto amplificata | Nemotron amplificata | Cohere amplificata |
|---|---|---|---|---|---|---|---|---|
| 2250 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Il bianco è sempre stato bianco. | ∅ | ∅ | Grazie. |
| 3390 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Grazie. | ∅ | ∅ | Grazie. |
| 4470 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Grazie. | Ok | ∅ | Grazie. |
| 8670 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Il suo nome è Jonathan Hartmann. | ∅ | ∅ | Grazie. |
| 10620 | Grazie a tutti. | Grazie. | Grazie. | ∅ | Il film è stato pubblicato in due feste. | ∅ | ∅ | Il giro di Pietro è fissato a Pietro. |
| 13710 | Glossary, Laravel | Grazie. | Grazie. | ∅ | E poi, per finire, non posso fare. | ∅ | ∅ | Vediamo, è come fare qui, un miglio, un miglio, un po' su stato. |
| 17310 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Grazie. | ∅ | okay | Grazie. |
| 18630 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Il resto è uguale. | ∅ | Sì. | Grazie. |
| 20700 | Glossary, Laravel | Grazie. | Grazie. | ∅ | Il suo compleanno è stato scoperto da un giovane amico e amico. | Grazie. | ∅ | Visita a Cecchia |
| 23250 | Grazie. | Grazie. | Grazie. | ∅ | Grazie a tutti. | Grazie. | ∅ | Il giro di Maria Cristina è stato completamente sottovalutato. |
| 26490 | Glossary | Grazie. | Grazie. | ∅ | Avvisti, Redmi e Cherokee. | Capisci? E' lì che è! | ∅ | Capisci, è il lecce! |
| 31260 | ∅ | Grazie. | Ok. | ∅ | Il giro di Pisa è completamente in alto. | Eh! | ∅ | Ecc. |

- **[V] Il documento è riprodotto.** Turbo con `vocab_ns0` dà lo stesso testo del documento su tutte le 11 Frasi del Microfono che ci sono («Glossary, Laravel» 8 volte, «Glossary», «Grazie a tutti.», «Grazie.»), e testo vuoto su 31260, che infatti nel documento manca.
- **[V] Ogni prompt viene ricopiato da Medium.** Con `vocab`, `vocab1`, `initial` e `contesto` tutte le 12 Frasi diventano il prompt («Glossary», «Slaitech, Laravel», «Slaitech, Laravel.»). `frase_ns0` lo ricopia in 2 («Slaitech e di Laravel.») e nelle altre scrive «Grazie.» o «Sottotitoli creati dalla comunità Amara.org». Turbo ricopia il Glossary in 9 su 12, l'`initial_prompt` in 1, il prompt di contesto in 0.
- **[V] Senza prompt l'invenzione resta.** Medium `vuoto_ns0` scarta 5 Frasi e scrive «Grazie.» in 6 e «Sottotitoli creati dalla comunità Amara.org» in 1. Turbo `vuoto_ns0` scrive «Grazie.» in tutte le 12. Cohere inventa frasi italiane intere in 8 su 12 («Il suo nome è Jonathan Hartmann.»). Solo Nemotron tace.
- **[V] La soglia no-speech non scatta col prompt.** La regola è `no_speech_prob > no_speech_thold && avg_logprob < logprob_thold` (`src/arch/whisper/model.cpp:2488-2489`). Copiare il prompt è una decodifica sicura: `avg_logprob` fra −0,31 e −0,86, sempre sopra −1, quindi la Frase passa sia con `no_speech_thold` 0 sia con 0,6. Senza prompt, Medium ha `avg_logprob` sotto −1 in 5 Frasi e le scarta (con 0 e con 0,6 il risultato è identico su tutte le 12). Le soglie predefinite non cambiano nessun testo rispetto a `ns0` su queste Frasi.
- **[V] `no_speech_prob` di Medium è alto sul rumore** (0,67-0,97 in ogni variante senza amplificazione), ma da solo non separa. Nel 15-04-07, Frasi di parlato vero hanno 0,81-0,83 con il Glossary («appunto di fare…», «Va bene, ma…») e 0,69 senza prompt («scrivere tutta l'anagrafica…»).
- **[V] Turbo ha `no_speech_prob` sempre ≈ 0** (al massimo 2,5e-9 su tutte le Frasi e varianti, rumore compreso), quindi con Turbo la soglia no-speech non può mai scattare. [I] È una proprietà del modello (large-v3-turbo) o del suo GGUF, da verificare contro il riferimento di Transformers.

### 2.3 Il prompt sul parlato vero (Audio di sistema)

**Tabella D.** Frasi dell'Audio di sistema (prime parole; no_speech_prob / avg_logprob di Medium)

| Tape | Frase (ms) | RMS dBFS | Silero sopra 0,4 | Nemotron | Medium vocab_ns0 | Medium vuoto_ns0 | Medium contesto_ns0 | Turbo vocab_ns0 | Turbo vuoto_ns0 |
|---|---|---|---|---|---|---|---|---|---|
| 15-11-13 | 2310–3870 | -39.1 | 0.17 | ∅ | Slaitech, Laravel (0.52 / -0.53) | ∅ (0.53 / -1.11) | Slaitech, Laravel. | ∅ | Grazie. |
| 15-11-13 | 7710–8790 | -30.2 | 0.06 | ∅ | Slaitech (0.64 / -0.60) | Grazie. (0.35 / -0.75) | Grazie. | ∅ | Grazie. |
| 15-11-13 | 9720–12630 | -17.3 | 0.65 | Lo se seguiamo… | Non si seguiamo… (0.02 / -0.65) | Non si seguiamo… (0.10 / -0.79) | Non si seguiamo… | Glossary, Laravel | Se seguiamo questa… |
| 15-11-13 | 12630–18960 | -15.6 | 0.47 | quindi centrale di… | Questo è stato… (0.30 / -0.80) | quindi questo è… (0.13 / -0.65) | Questo è stato… | quindi centrale questo… | quindi questo è… |
| 15-11-13 | 21450–32100 | -17.9 | 0.79 | già c'è la… | Slaitech, Laravel (0.05 / -0.82) | Allora, sì, questo… (0.07 / -0.34) | Sì, già c'è… | Sì, già c'è… | Allora, sì, questo… |
| 15-11-13 | 32100–34080 | -25.1 | 0.58 | Il Daniele | ∅ (0.79 / -1.61) | ∅ (0.82 / -1.48) | E' lì, Daniela. | E lì, Daniele... | e lì Daniele… |
| 15-11-13 | 34980–46650 | -14.3 | 0.64 | cioè noi non… | Non so come… (0.04 / -0.76) | Non so come… (0.04 / -0.93) | Non so come… | ∅ | io non so… |
| 15-11-13 | 49440–52500 | -16.2 | 0.56 | Daniele che per… | Forza di Daniele… (0.11 / -0.64) | forse di Daniele… (0.07 / -0.57) | Forza di Daniele… | forse di Daniele… | forse di Daniele… |
| 15-11-13 | 52500–53550 | -31.9 | 0.09 | Sì. | ∅ (0.66 / -6.75) | Sì. (0.27 / -0.87) | Sì. | that's all right. | Grazie. |
| 15-11-13 | 53550–56430 | -13.0 | 0.76 | qua invece questo… | ∅ (0.53 / -1.05) | e questo è… (0.52 / -0.91) | Questa è quella… | Qua invece, questo… | Qua invece, questo… |
| 15-04-07 | 0–5490 | -21.7 | 0.58 | scrivere il tutta… | ∅ (0.32 / -1.03) | iscrivere il tutt'altra… (0.69 / -0.49) | Scrivere il tutt'altra… | Scrivere tutta l'anagrafica… | scrivere tutta l'anagrafica… |
| 15-04-07 | 5640–12480 | -23.7 | 0.62 | persa scartata acquistata… | Slaitech, Laravel (0.17 / -0.85) | se è persa,… (0.06 / -0.37) | Slaitech, Laravel. | Glossary, Laravel | se è persa,… |
| 15-04-07 | 13290–14460 | -29.8 | 0.13 | Okay. | Slaitech, Laravel (0.71 / -0.58) | Grazie. (0.66 / -0.89) | Slaitech, Laravel. | Glossary, Laravel | Grazie. |
| 15-04-07 | 14460–30750 | -15.0 | 0.75 | Allora vi antico… | ∅ (0.04 / -1.08) | Allora vi antico… (0.03 / -0.79) | Lui che dici… | Allora, vi anticipo...… | Allora, vi anticipo...… |
| 15-04-07 | 30930–33000 | -18.2 | 0.42 | ∅ | Slaitech, Slaitech, Slaitech (0.03 / -0.23) | Ciao a tutti,… (0.29 / -0.50) | Slaite, Slaite, Slaite. | Slaitech, Slaitech, Slaitech | Sachi, sachi, sachi. |
| 15-04-07 | 34440–36270 | -26.3 | 0.15 | ∅ | Glossary (0.66 / -0.70) | Ok (0.51 / -0.88) | Ok. | Slaitech, Laravel | Grazie. |
| 15-04-07 | 36270–37260 | -25.0 | 0.06 | ∅ | Buon appetito! (0.18 / -0.53) | Buon appetito! (0.16 / -0.52) | Buon appetito. | Glossary, Laravel | Buongiorno. |
| 15-04-07 | 37530–41340 | -14.5 | 0.62 | Non so forse… | Glossary (0.12 / -0.90) | Non so, forse… (0.11 / -0.22) | Non so, forse… | Non so, forse… | non so, forse… |
| 15-04-07 | 42000–44490 | -25.1 | 0.51 | di lunetti | Di lunedì (0.45 / -0.47) | di lunedì (0.34 / -0.43) | Di lunedì. | di lunedì | di lunedì |
| 15-04-07 | 44940–47280 | -27.9 | 0.38 | prossimo spiego quarto | ∅ (0.03 / -1.28) | prossimo spoglio del… (0.03 / -0.92) | Prossimo, spugno il… | Glossary, Laravel | prossimo spiego il… |
| 15-04-07 | 50100–51780 | -30.7 | 0.21 | ∅ | Glossary (0.63 / -0.71) | ∅ (0.63 / -1.03) | Slaitech, Laravel. | Grazie. | Grazie. |
| 15-04-07 | 54270–55830 | -16.4 | 0.35 | Allora | Allora... (0.34 / -0.60) | allora (0.18 / -0.67) | Allora... | Allora | allora |
| 15-04-07 | 55920–59700 | -13.9 | 0.60 | Lo ro mi… | Loro mi hanno… (0.07 / -0.46) | Loro mi hanno… (0.17 / -0.53) | Loro mi hanno… | Loro mi hanno… | Loro mi hanno… |
| 15-04-07 | 59700–64200 | -14.6 | 0.73 | Purtroppo io ero… | Purtroppo ero incasinato,… (0.14 / -0.51) | Purtroppo ero incasinato,… (0.06 / -0.55) | Purtroppo ero incasinato,… | Slaitech, Laravel | purtroppo io ero… |
| 15-04-07 | 64200–65460 | -15.1 | 0.43 | e | Slaitech (0.52 / -0.62) | ∅ (0.68 / -1.31) | Slaitech. | è... | è |
| 15-04-07 | 65880–71070 | -18.5 | 0.64 | quindi lunedì alle… | ∅ (0.14 / -1.34) | e quindi ecco… (0.07 / -0.80) | Per quando? L'uomo… | Per quando? Per… | Per quando? Per… |
| 15-04-07 | 71250–72750 | -19.0 | 0.26 | ∅ | Slaitech, Laravel (0.75 / -0.66) | ∅ (0.49 / -1.05) | Ehm... | ∅ | e |
| 15-04-07 | 75030–76290 | -14.9 | 0.19 | Allora. | Allora, grazie. (0.59 / -0.86) | allora (0.42 / -0.70) | Allora... | Allora, grazie. | allora grazie |
| 15-04-07 | 76290–78270 | -16.4 | 0.32 | appunto di fare… | Slaitech, Laravel (0.83 / -0.67) | appunto di fare… (0.50 / -0.74) | Slaitech, Laravel. | e la cosa… | appunto di fare… |
| 15-04-07 | 78270–79740 | -13.8 | 0.29 | per | ∅ (0.84 / -1.24) | ∅ (0.94 / -1.07) | ∅ | per | per |
| 15-04-07 | 80430–89430 | -15.3 | 0.50 | anche provare a… | Slaitech, Laravel (0.35 / -0.94) | Ho anche provato… (0.14 / -0.43) | Ho anche provato… | Glossary, Laravel | anche provare a… |
| 15-04-07 | 89430–91110 | -13.8 | 0.18 | Va bene, ma… | Slaitech, Laravel (0.81 / -0.54) | va bene ma… (0.44 / -0.67) | Va bene, ma… | va bene ma… | va bene ma… |
| 15-04-07 | 91110–91920 | -20.0 | 0.37 | ∅ | Glossary (0.52 / -0.92) | ∅ (0.39 / -1.11) | Slaitech, Laravel. | Glossary | più o |

Somiglianza con Nemotron sulle Frasi dell'Audio di sistema con almeno 3 parole (6 nel 15-11-13, 11 nel 15-04-07) [V]:

| Variante | Medium 15-11-13 | Medium 15-04-07 | Turbo 15-11-13 | Turbo 15-04-07 |
| --- | --- | --- | --- | --- |
| `vocab_ns0` (oggi) | 0,36, perse 2 | **0,11, perse 9 su 11** | 0,61, perse 2 | 0,47, perse 5 |
| `vocab1_ns0` | 0,57, perse 1 | 0,12, perse 9 | 0,70, perse 1 | 0,47, perse 4 |
| `initial_ns0` | 0,34, perse 2 | 0,27, perse 6 | 0,77, perse 0 | 0,69, perse 1 |
| `contesto_ns0` | 0,62, perse 0 | 0,55, perse 2 | 0,69, perse 1 | 0,78, perse 0 |
| `frase_ns0` | 0,62, perse 0 | 0,50, perse 3 | 0,68, perse 1 | 0,78, perse 0 |
| `vuoto_ns0` | 0,67, perse 0 | 0,71, perse 0 | 0,77, perse 0 | 0,81, perse 0 |
| `vocab_def` | 0,36, perse 2 | 0,11, perse 9 | 0,61, perse 2 | 0,47, perse 5 |

Eco contate su tutte le Frasi di Sistema (testo fatto solo di Glossary/Termini) [V]: con `vocab_ns0` Medium 3/10 e **12/23**, Turbo 1/10 e 9/23. Senza prompt 0 in entrambi.

- **[V] Il Glossary sostituisce anche parlato forte e lungo.** Esempi: 10,6 s a −17,9 dBFS («Sì, già c'è…») diventano «Slaitech, Laravel» con Medium; 9 s a −15,3 dBFS («anche provare a…») diventano «Slaitech, Laravel» con Medium e «Glossary, Laravel» con Turbo; 6,8 s («persa scartata…») lo stesso. Il problema non è solo il rumore del Microfono.
- **[V] Basta un solo Termine.** `vocab1_ns0` (solo «Laravel») perde quanto `vocab_ns0`: dipende dal formato `Glossary: …`, non da «Slaitech».
- **[V] Il prompt fa anche perdere Frasi.** Con `vocab_ns0` Medium svuota 4 Frasi del 15-04-07 che senza prompt hanno testo. Turbo scarta la Frase 34980-46650 del 15-11-13 (11,7 s di parlato vero): a temperatura 0 entra in loop, il primo ritentativo (T 0,2) ha `avg_logprob` −7,43 e `compression_ratio` 8,39, e con `no_speech_thold` 0 la regola di salto scatta subito, senza i ritentativi successivi. Quella Frase manca dal documento attuale del Tape. Senza prompt Turbo la trascrive intera a temperatura 0.
- **[V] Il prompt passato come contesto ricopia meno del Glossary**, ma non risolve. Con Turbo ha 0 eco sul Microfono e 3 su 23 nel 15-04-07; perde la Frase 34980 del 15-11-13 (loop, come col Glossary) e nessuna del 15-04-07. Con Medium ricopia ancora 12/12 sul Microfono e 6/23 nel 15-04-07.
- **[V] Dove si genera il prompt.** transcribe.cpp mette i Termini nello slot `<|startofprev|>` come ` Glossary: a, b` (`src/arch/whisper/model.cpp:982-1016`), come fa Transformers con `prompt_ids`: lo stesso posto dell'`initial_prompt`, che infatti ricopia anche lui. [I] L'eco è il comportamento noto di Whisper sulle finestre quasi vuote: una Frase di 1-10 s occupa una piccola parte della finestra di 30 s, il resto è silenzio, e il modello continua il testo precedente che ha nel contesto. Il formato «Glossary:» sembra peggiorare la cosa (Turbo: 9 eco contro 1 dell'`initial_prompt` e 0 del contesto).

### 2.4 Cosa funziona e cosa no contro l'eco

| Alternativa | Esito sulle misure |
| --- | --- |
| **Gate sul livello** prima dell'ASR | **Separa con margine su questi Tape** [V]: le Frasi del Microfono hanno RMS ≤ −62,3 dBFS e picco ≤ −33,6 dBFS; tutte le 33 Frasi dell'Audio di sistema hanno RMS ≥ −39,1 (quelle con parlato vero ≥ −31,9) e picco ≥ −19,6. Una soglia RMS di −50 dBFS toglierebbe 12/12 Frasi di rumore e 0 di parlato. Toglie anche gli «Grazie.» e le frasi inventate da Cohere e Whisper senza prompt, che nessun'altra alternativa toglie. [I] Il limite: una voce lontana o un microfono con guadagno basso potrebbero stare sotto soglia; la soglia va calibrata su registrazioni di voce debole (cfr. la protezione del parlato, ADR-0023/0024). |
| Gate su Silero (media o quota sopra soglia) | **No** [V]: il rumore del Microfono ha media 0,09-0,46 e quota 0,06-0,46; parlato vero dell'Audio di sistema scende a 0,09-0,19 di quota («Sì.», «Va bene, ma…», «Allora.», confermati da Nemotron). Le distribuzioni si sovrappongono. |
| Gate sul rientro (correlazione con l'Audio di sistema) | **Non applicabile qui** [V]: le Frasi non sono eco (coerenza 0,006). Non distinguerebbe il rumore dal parlato. |
| Soglie no-speech predefinite (0,6 / −1) al posto di `no_speech_thold: 0` | **No** [V]: con prompt nessuna Frase di eco scatta (`avg_logprob` > −1); senza prompt il risultato è identico a `ns0` sul Microfono. Con Turbo `no_speech_prob` ≈ 0 non scatta mai. |
| Scarto su `no_speech_prob` letto dalla traccia (> 0,6 da solo) | **No** [V]: con Medium toglierebbe tutte le 12 Frasi di rumore, ma anche Frasi vere («Va bene, ma…» ha 0,81 con il Glossary); con Turbo non vede nulla. Richiede inoltre il puntatore nativo della sessione (§1, punto 5). |
| Ritentare la Frase senza prompt se il testo è solo prompt (Termini o «Glossary») | **Toglie l'eco** [V]. Con Medium la somiglianza nel 15-04-07 passa da 0,11 a 0,52, ma restano 4 Frasi perse: quelle che il prompt aveva svuotato. Ritentando anche le Frasi vuote si arriva a 0,70 con Medium (0 perse; senza prompt 0,71) e 0,77 con Turbo (1 persa; senza prompt 0,81). Sul Microfono restano le invenzioni senza prompt («Grazie.»). Costo: una seconda `run` per ogni Frase ritentata. |
| Termini come prompt di contesto (`RunOptions::prompt`) invece del Glossary | **In parte** [V]: con Turbo nessuna Frase persa e 0 eco sul Microfono; con Medium eco su 12/12 Frasi di rumore e 6/23 nel 15-04-07. |
| `initial_prompt` (il Memotape di prima) | **No** con Medium (ricopia 12/12 e 11/23); meglio del Glossary con Turbo (1 eco sul Microfono, 2 nel 15-04-07). Non si può più combinare con `vocabulary` (`bindings/rust/transcribe-cpp/src/family.rs:23`). |
| Nessun prompt | La qualità più alta su tutte le misure di parlato vero (0,67-0,81), ma il Vocabolario non arriva al modello. [I] Su questi Tape nessuno pronuncia «Slaitech» o «Laravel», quindi il beneficio del Vocabolario non si è potuto misurare. |

## 3. Problema 2: il loop di Cohere

### 3.1 Dove va in loop

- **[V] Su 45 Frasi (2 Tape, entrambi gli Ingressi) una sola va in loop**, sempre la stessa, in due esecuzioni con la guardia attiva: la Frase dell'Audio di sistema 34980-46650 ms del Tape 15-11-13. Le altre 44 finiscono con `Ok`. [I] È la Frase che ha fermato le due Trascrizioni dell'utente: il log non registra l'errore, ma il Tape e il modello sono gli stessi, e il loop è deterministico.

| Misura | Valore |
| --- | --- |
| Durata | 11 670 ms |
| Livello | RMS −14,3 dBFS, picco +0,13 dBFS (il decoder Opus supera 0 dBFS), Silero media 0,58, frame sopra soglia 64% |
| Contenuto (Whisper Medium, senza prompt) | «Non so come farlo. No, no, no, devi appunto…», con «no» ripetuto davvero dal parlante; segue un nome |
| Nemotron | va in loop anche lui, senza errore: «… come come come come come come come come come come come come come farlo…» (13 volte) |
| Cohere, errore | `Error::OutputRepetition`, «decode stopped when the output began repeating itself (status 19)», `was_truncated` vero, in 355 ms |
| Messaggio di transcribe.cpp | `cohere run: output began repeating a 2-token block; decode stopped with the repeats dropped (10 tokens kept)` |
| `Error::partial()` | «Io non so come farlo. Se no,»: circa un terzo del contenuto, perde la seconda metà |

### 3.2 Le alternative

| Alternativa | Esito [V] |
| --- | --- |
| **Dividere nel punto più silenzioso** (6255 ms) e trascrivere le due metà con Cohere | Entrambe `Ok`: «Io non so come farlo, se si deve stare.» + «Devi appunto, lì ci sarà Daniele oppure Alessio.» Il contenuto c'è tutto. «No, no, no» diventa «se si deve stare», che è un errore di riconoscimento, non un loop. 258 + 255 ms. |
| Ritrascriverla senza i 300 ms di prefill e la coda sotto −40 dBFS | `Ok`: «Io non so come farlo. Se no, no, no, no, devi appunto, devi appunto lì. Ci sarà Daniele oppure la.» È la versione più fedele, «no» ripetuto compreso. Basta cambiare di poco la finestra per uscire dal loop. 309 ms. |
| Ritrascriverla con un altro modello | Whisper Medium (tutte le varianti) e Turbo senza prompt: testo completo. Turbo con il Glossary: Frase scartata (§2.3). Nemotron: testo con «come» ripetuto 13 volte. |
| Tenere il parziale | Perde la seconda metà della Frase (circa 7 s di parlato). |
| `TRANSCRIBE_NO_REPETITION_GUARD=1` | La decodifica arriva al budget di 512 token: «Io non so come farlo. Se no, no, no, no, …» con circa 250 «no», poi `Error::OutputTruncated` (status 18), 1025 ms. Memotape lo tratterebbe allo stesso modo (errore interno), quindi non serve. |

### 3.3 Cosa offre transcribe.cpp 0.4.0 per Cohere

- **[V] Nessuna opzione di decodifica per Cohere.** `RunExtension` ha solo la variante `Whisper` (`bindings/rust/transcribe-cpp/src/family.rs:180-182`). Cohere dichiara solo `TRANSCRIBE_FEATURE_CANCELLATION` (`src/arch/cohere/capabilities.cpp:17`): niente vocabolario, prompt o prefisso (`docs/prompting.md:13-22`). La decodifica è greedy, con l'argmax letto dalla GPU (`src/arch/cohere/model.cpp:1215-1217`, `1300-1302`): niente temperatura, niente penalità di ripetizione. Il prompt è fisso: lingua, `<|pnc|>`, `<|noitn|>`, `<|notimestamp|>` (`model.cpp:856-860`).
- **[V] La guardia contro le ripetizioni.** `stop_on_repetition` (`src/transcribe-repetition-guard.h:92`) ferma la decodifica quando la coda ripete un blocco: per un blocco di 2 token servono 64 token ripetuti, cioè 32 copie (`k_stop_bar`, riga 40). Toglie le copie, ne tiene una, e la `run` restituisce `TRANSCRIBE_ERR_OUTPUT_REPETITION` con il parziale (`docs/input-limits.md:175`). L'unica manopola è la variabile d'ambiente `TRANSCRIBE_NO_REPETITION_GUARD`, letta una volta per processo (riga 83, `docs/environment-variables.md:32`), che la spegne. Il budget minimo è di 512 token (`k_gen_reserve`, `model.cpp:422`).
- **[V] Il parziale è sempre disponibile** su `OutputRepetition` e `OutputTruncated` (`bindings/rust/transcribe-cpp/src/error.rs:69-73`, `274-278`; `session.rs:180-189`).
- **[V] Memotape trasforma entrambi in errore interno.** `catch_native` (`engine/transcribe_cpp.rs:343-351`) mappa tutto tranne `Aborted` e `Busy` in `EngineError::Internal`, e la pipeline esce al primo errore (`engine/pipeline.rs:466-470`). Una Frase su 45 basta a non salvare il Tape.

## 4. Incertezze e cosa non è stato misurato

- **[I] Perché il Microfono è a −70 dBFS.** Muto del dispositivo, guadagno al minimo o noise gate del driver: dal file non si distingue, e non ho potuto ascoltarlo.
- **La Trascrizione dal vivo con Whisper Medium** (il primo documento del 15-11-13) non è riproducibile: il documento attuale l'ha sostituita, e dal vivo i tagli e la cronologia dei frame possono differire. Ho misurato le Frasi di Trascrivi su un Tape.
- **Il Vocabolario dell'utente al momento delle Trascrizioni** non si conosce: oggi è vuoto, e ho usato `Slaitech, Laravel`. Il documento contiene «Glossary, Laravel» (e mai «Slaitech»): è compatibile con questi Termini, perché Turbo produce esattamente quel testo con entrambi.
- **Il beneficio del Vocabolario non è misurato**: nessuno dei due Tape contiene i Termini.
- **La soglia del gate sul livello** è ricavata da due Tape con un solo microfono e un solo Audio di sistema. Mancano registrazioni con una voce debole o lontana per sapere quanto margine c'è sotto −40 dBFS.
- **`no_speech_prob` di Turbo ≈ 0**: non ho verificato se è il modello o la conversione GGUF.
- **Il loop di Cohere** si è presentato una sola volta: la divisione e il taglio dei bordi funzionano su questa Frase, ma su un campione di 1 non si può dire se funzionino sempre. Le metà possono andare in loop a loro volta. Il test non ritenta oltre.
- **La traccia di Whisper** si legge solo con un ripiego (il puntatore privato di `Session`); un uso nell'app richiederebbe che i binding la espongano.

## Esito

Decisioni del 16 ottobre 2026 (ADR-0033), da queste misure:

- Nessun modello riceve più i Termini nel prompt: si applicano solo con la correzione dopo la Frase.
- Una Frase sotto −50 dBFS di RMS non va al modello (`audio_toolkit::livello::rumore`).
- Con `OutputRepetition` o `OutputTruncated` la Frase si divide nel punto più silenzioso e si ritrascrive, al massimo in quattro pezzi; un pezzo che fallisce ancora tiene il parziale seguito da «[incomprensibile]» (`engine::transcribe_cpp::run_diviso`).
- Una parola ripetuta più di 4 volte di seguito resta 3 volte (`engine::vocabolario::riduci_ripetizioni`).

Verifica [V]: l'Audio di sistema del Tape 15-11-13 ritrascritto con Cohere arriva in fondo (10 Frasi) e la Frase 34980–46650 ms esce intera.
