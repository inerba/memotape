# 03: Il Vocabolario nella Trascrizione

**What to build:** Ogni Trascrizione usa il Vocabolario delle impostazioni com'era al suo avvio: con Whisper nel prompt iniziale, con ogni modello la correzione del ticket 01 sulle Frasi concluse, mai sui Parziali. Regole in `../spec.md`, sezioni "Dominio" e "Whisper".

**Blocked by:** 01, 02.

**Status:** done

- [x] `TranscribeCpp` riceve i Termini all'avvio della Trascrizione, accanto a `set_cancel_token` in `run_pipeline` (`managers/transcription.rs`); vale per file, Trascrivi su un Tape (anche gli Ingressi in parallelo con `load_instance`) e Trascrivi dal vivo. Verificare che ogni percorso passi da lì.
- [x] Whisper: `initial_prompt` = Termini uniti da `", "`, nelle stesse `WhisperRunOptions` di `confident_only()`; niente prompt senza Termini.
- [x] La correzione si applica nel `keep` di `TranscribeCpp::transcribe`, dopo il filtro `foreign_script`, sul risultato con tempi relativi; i Parziali non passano di lì.
- [x] Test (con `tdd`) dove la seam lo permette (motore finto della pipeline o funzione estratta); lo smoke con i modelli veri resta manuale.
- [x] AGENTS.md: una riga nell'Architettura sul Vocabolario (prompt di Whisper, correzione in `keep`, snapshot all'avvio).
- [x] I sei controlli passano.

## Comments

- 2026-10-08, dal ticket 02: `Settings::load` non scarta un Termine con `<|`/`|>` scritto a mano in `settings.json`; escluderlo dal prompt di Whisper qui.
- 2026-10-08, implementazione (test-first): `TranscribeCpp::set_termini` copia i Termini nel motore; `run_pipeline` lo chiama subito dopo `set_cancel_token` con `settings.vocabolario`, dove `settings` è la fotografia presa all'avvio da `transcribe` (file e Tape, anche l'istanza in più di `load_instance` e gli Ingressi in sequenza) e da `record` per `transcribe_live`. Nell'app non ci sono altri percorsi verso `TranscribeCpp::transcribe` (`preload` carica e rilascia soltanto). `run_pipeline` ora ha 8 argomenti: `#[expect(clippy::too_many_arguments)]` con la ragione. `confident_only(termini)` mette nell'`initial_prompt` i Termini uniti da `", "`, saltando quelli con `<|`/`|>`; senza Termini validi niente prompt. Il `keep` è diventato la funzione `concluded(transcript, language, termini)`: filtro `foreign_script`, `timed_result`, poi `vocabolario::correct`; il ramo dei Parziali usa ancora solo `timed_result`. Tolto l'`#[allow(dead_code)]` su `engine::vocabolario`.
  - Test unitari in `engine::transcribe_cpp::tests`: prompt con Termini, senza Termini, con `<|`; Frase conclusa corretta con i tempi uniti, scrittura estranea scartata anche con Termini, invariata senza Termini. Che i Parziali non siano corretti non ha un test automatico (servirebbe uno stream nativo): lo garantisce la struttura, `concluded` si chiama solo sul risultato finale.
  - Smoke nuovo `engine::pipeline::tests::i_tre_modelli_usano_il_vocabolario` (`--ignored --test-threads=1`): con il Termine "Trascrizzione" su `parlato-it.wav` tutti e tre i modelli scrivono "oggi parliamo di Trascrizzione". Lanciato e verde, come lo smoke esistente `i_tre_modelli_trascrivono_il_parlato_italiano`.
  - Non provato: la UI (nessun `tauri dev`), la Trascrizione dal vivo e Trascrivi su un Tape con i modelli veri (lo smoke copre solo il percorso dei file), i falsi positivi su audio reale, un prompt oltre i 223 token.
- 2026-10-08, code review (branch `feat/vocabolario-review`): un solo predicato per `<|`/`|>`, `engine::vocabolario::has_special_token`, usato da `managers::settings` e dal prompt di Whisper. `confident_only` è diventata `whisper_options` (soglie e prompt), `concluded` è `finish_phrase` (filtro della scrittura, tempi, correzione) e la closure `keep` è `finish`; commenti e AGENTS.md aggiornati.
