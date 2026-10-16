# Ricerca: come Handy affronta la qualità della Trascrizione

Data: 2026-10-15. Scopo: capire cosa fa Handy contro due difetti visti in Memotape con Parakeet:

1. un Termine del Vocabolario che compare dove non c'entra («slide deck» al posto di «scelte»);
2. parlato italiano, soprattutto a voce bassa, trascritto come inglese.

Il documento raccoglie solo fatti. Le **Proposte** sono marcate e la scelta spetta all'utente.

Legenda: **[V]** = verificato sul sorgente (file e riga). **[I]** = inferenza da confermare.

## 0. Fonti

| Fonte | Riferimento |
| --- | --- |
| Handy | `https://github.com/cjpais/Handy`, HEAD `9e57a0d7929fcc6417ea0d6e54c1b57f62635edd` (2026-10-15, "long form chunking with vad (#2282)"), versione 0.9.8, `transcribe-cpp = "0.4.0"` |
| transcribe.cpp | `https://github.com/handy-computer/transcribe.cpp`, HEAD `90cd63d53b3103d137dcf463ec62338a66f56174` (2026-10-15); tag `v0.4.0` del 2026-10-12 |
| Memotape | `transcribe-cpp` da git, rev `e6672a8672913b47f1571c66c54bee789d028416` (2026-09-26, "metal improvements"), **precedente** a `v0.3.0` |

I percorsi di Handy sono relativi a `src-tauri/src/`, quelli di transcribe.cpp alla root del repo.

---

## 1. Il Vocabolario: Handy usa lo stesso algoritmo

### Fatti
- **[V] La correzione approssimata è la stessa di Memotape.** `apply_custom_words` (`audio_toolkit/text.rs:150`) usa la distanza di Levenshtein divisa per la lunghezza maggiore. Moltiplica il punteggio per 0,3 se il Soundex coincide (`text.rs:117-125`), salta i candidati che differiscono di oltre il 25% o di oltre 2 caratteri (`text.rs:95-104`), prova n-grammi fino a 3 parole senza attraversare la punteggiatura (`text.rs:163-178`). La soglia predefinita è 0,18 (`settings.rs:627`, test in `settings.rs:1413`). Memotape usa gli stessi valori ([vocabolario.rs:9](../../src-tauri/src/engine/vocabolario.rs), `best_match`).
- **[V] In Handy la soglia si può cambiare.** `word_correction_threshold` è un'impostazione dell'utente (`settings.rs:446-447`). In Memotape `SOGLIA` è una costante.
- **[V] Con i modelli che accettano un vocabolario, Handy non corregge.** Se il modello ha `Feature::Vocabulary` (`engine_supervisor/worker.rs:317`), i Termini vanno nel campo `vocabulary` di `RunOptions` (`managers/transcription.rs:1175`, `1418-1425`). La correzione approssimata gira solo se i Termini non sono già andati al modello (`transcription.rs:1505-1513`, `!custom_words_already_prompted`).
- **[V] Il campo `vocabulary` è arrivato con transcribe.cpp 0.3.0.** Il commit `e85b30e` "generic prompting in interface (#178)" del 2026-09-30 è contenuto in `v0.3.0` e `v0.4.0`. La revisione di Memotape è del 26 settembre, quindi non ce l'ha.
- **[V] I modelli con vocabolario nativo** (`docs/prompting.md`) sono Whisper, Qwen3-ASR, Granite 4.0-1b/4.1-2b, Fun-ASR-Nano e MOSS. Parakeet, Nemotron, Canary e Cohere non ce l'hanno.
- **[V] Nessuna protezione contro le parole comuni.** Né Handy né Memotape controllano se il tratto da sostituire è già una parola valida della lingua. Il Soundex è quello americano, pensato per l'inglese (`text.rs:117-120`).

### Cosa significa per «slide deck»
**[I]** La sostituzione non può essere partita da «scelte» da sola: le chiavi `scelte` e `slidedeck` differiscono di 3 caratteri, oltre il limite. Parakeet aveva già scritto qualcosa di inglese e simile, che il Vocabolario ha agganciato. Handy avrebbe fatto lo stesso con Parakeet. Il primo errore è la lingua (sezione 2), la correzione lo rende visibile.

---

## 2. La lingua: Handy non la forza su Parakeet, cambia modello

### Fatti
- **[V] Parakeet TDT v3 non accetta un'indicazione di lingua.** In transcribe.cpp la lingua dei modelli parakeet passa da una tabella di prompt (`resolve_prompt_id`, `src/arch/parakeet/model.cpp:147-175`). Senza tabella (`!hp.has_prompt`) un'indicazione esplicita dà `UNSUPPORTED_LANGUAGE` (`model.cpp:1245-1252`). La tabella ce l'ha Nemotron 3.5 (locale `it-IT`), non Parakeet v3, che riconosce la lingua da solo. Memotape lo sa già (`accepts_language` falso, [mod.rs:68](../../src-tauri/src/engine/mod.rs)).
- **[V] Handy passa la lingua solo se il modello la offre**, altrimenti trascrive in automatico (`transcribe_cpp_run_plan`, `managers/transcription.rs:1433-1460`). È la stessa regola di Memotape (`resolve_language`). Con Parakeet v3, quindi, anche Handy lascia scegliere la lingua al modello.
- **[V] Handy non consiglia più Parakeet v3.** Nel catalogo (`catalog/catalog.json`) i modelli consigliati sono `parakeet-unified-en-0.6b`, `nemotron-3.5-asr-streaming-0.6b`, `canary-180m-flash`, `cohere-transcribe-03-2026` e `whisper-medium`. Quelli che hanno l'italiano sono **Nemotron 3.5, Cohere Transcribe e Whisper medium**. `parakeet-tdt-0.6b-v3` resta nel catalogo, ma non è consigliato.
- **[V] Il riconoscimento della lingua dal testo serve solo per le pulizie.** `detect_output_language` (`audio_toolkit/lang_id.rs`, crate `whatlang 0.16`) accetta una lingua solo con confidenza ≥ 0,9 ed è sicuro su circa il 66% delle frasi (`lang_id.rs:11-19`). Si usa per togliere le parole riempitive e per la scrittura cinese (`transcription.rs:1471-1490`), mai per ritrascrivere una Frase.
- **[V] transcribe.cpp 0.4.0 ha un modello che riconosce la lingua dall'audio.** È il ruolo LANGID (`docs/langid.md`, `docs/migrating-to-0.4.md:28-31`), con VoxLingua107 ECAPA-TDNN e 107 lingue. Si può limitare a un insieme (`allowed`) e restituisce la probabilità di ogni lingua. Vuole almeno 500 ms di audio e ascolta al massimo i primi 30 s. Non ha un'etichetta "nessun parlato". **Handy non lo usa** (`grep langid` trova solo `lang_id.rs`, quello testuale).
- **[V] Niente normalizzazione del volume in Handy.** Nessuna funzione di guadagno, RMS o loudness in `audio_toolkit` e `managers`. La voce bassa arriva al modello così com'è.
- **[V] File lunghi tagliati nelle pause.** Dal 2026-10-15 (#2282) Handy taglia l'audio lungo in blocchi di circa 30 s (massimo 40, minimo 5) dove c'è una pausa, perché "most models' accuracy degrades past 30-40 s of audio in one run" (`audio_toolkit/chunking.rs:1-35`). Memotape trascrive già una Frase alla volta (ADR-0007).
- **[V] Post-elaborazione con un LLM, facoltativa.** È un provider esterno con un prompt che corregge ortografia e punteggiatura e conserva la lingua originale (`settings.rs:811`, `actions.rs:122-190`). Non è locale per impostazione predefinita.

### Modelli di transcribe.cpp che accettano la lingua italiana come vincolo
WER sul test FLEURS, in Q8_0 (schede in `docs/models/`). FLEURS è parlato letto e pulito: non misura la voce bassa né le frasi in due lingue.

| Modello | Lingua forzabile | WER it | Vocabolario nativo | Tempi | Note |
| --- | --- | ---: | --- | --- | --- |
| Parakeet TDT v3 (oggi predefinito) | no | 3,02% | no | token | CC-BY-4.0 |
| Nemotron 3.5 streaming | sì (`it-IT`) | 5,78% | no | token | già nel catalogo di Memotape |
| Cohere Transcribe 03-2026 | sì, obbligatoria (niente rilevamento) | 3,24% | no | nessuno | massimo 400 s per chiamata; Apache-2.0 |
| Canary 1b-v2 | sì (lingua esplicita) | 3,10% | no (solo prefisso) | nessuno nel port v1 | CC-BY-4.0 |
| Qwen3-ASR 1.7B | sì (indicazione nel prompt) | 2,68% | sì, più un prompt di contesto | nessuno | 2,19 GB in Q8_0; Apache-2.0 |
| Whisper large v3 turbo | sì, ma non vincolante | — | sì (prompt) | segmento | già nel catalogo di Memotape |

Fonti: `parakeet-tdt-0.6b-v3.md:54-73`, `cohere-transcribe-03-2026.md:28-33`, `63-77`, `canary-1b-v2.md:8-36`, `70-86`, `qwen3-asr-1.7b.md:6-20`, `58-76`, `src/arch/qwen3_asr/model.cpp:499-540`, `592-593`, `nemotron-3.5-asr-streaming-0.6b.md:96` (letta nella ricerca precedente).

---

## 3. Riepilogo: cosa fa Handy che Memotape non fa

| Leva | Handy | Memotape |
| --- | --- | --- |
| Correzione approssimata dei Termini | stessa, soglia regolabile | stessa, soglia fissa |
| Vocabolario passato al modello | sì, con transcribe-cpp ≥ 0.3 | solo il prompt di Whisper |
| Lingua forzata su Parakeet v3 | no (impossibile) | no (impossibile) |
| Modello consigliato per l'italiano | Nemotron 3.5, Cohere, Whisper medium | Parakeet v3 |
| Lingua riconosciuta dall'audio (LANGID) | no | no |
| Normalizzazione del volume | no | no |
| Ritrascrizione delle Frasi nella lingua sbagliata | no | no (scarta solo le scritture estranee) |
| Pulizia con un LLM | facoltativa, esterna | no |

In breve: Handy non ha un rimedio per Parakeet. Lo ha tolto dai consigliati e propone modelli a cui si può imporre la lingua.

## 4. Proposte (nessuna decisione)

1. **Modello predefinito per l'italiano.** Tornare a un modello che accetta la lingua: Nemotron 3.5, già presente, oppure aggiungere Cohere Transcribe o Canary 1b-v2. Per Cohere e Canary va verificato quanto pesa la mancanza dei tempi per parola: oggi servono ai tempi delle Frasi e al Vocabolario (`fits_tempi`).
2. **Aggiornare transcribe-cpp a 0.4.0**, per avere `vocabulary` nativo e LANGID. Prima va letto `docs/migrating-to-0.4.md`: l'API è cambiata (ruoli, nuovi errori).
3. **Vocabolario più prudente.** Non sostituire un tratto che è già una parola della Lingua del parlato, oppure rendere la soglia regolabile come in Handy. **[I]** Una lista di parole italiane comuni coprirebbe il caso «scelte».
4. **Frasi nella lingua sbagliata.** Con Parakeet in automatico si potrebbe controllare ogni Frase con LANGID (`allowed` = la Lingua del parlato più l'inglese) o con il riconoscimento dal testo, e ritrascrivere con Nemotron forzato solo le Frasi sospette. Handy non lo fa: andrebbe progettato e misurato.
5. **Volume.** Normalizzare il volume della Frase prima dell'ASR. Né Handy né Memotape lo fanno, quindi l'effetto sulla voce bassa è da misurare **[I]**.

## 5. Punti aperti

- Il caso «slide deck» non è riprodotto: serve il testo grezzo di Parakeet per quella Frase.
- Non ho misurato su audio italiano a voce bassa nessuno dei modelli della tabella. I WER FLEURS non bastano per scegliere.
- Non ho verificato il comportamento di Qwen3-ASR con un'indicazione di lingua sbagliata o sulle Frasi brevi.
