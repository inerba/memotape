# 03: Il Vocabolario nella Trascrizione

**What to build:** Ogni Trascrizione usa il Vocabolario delle impostazioni com'era al suo avvio: con Whisper nel prompt iniziale, con ogni modello la correzione del ticket 01 sulle Frasi concluse, mai sui Parziali. Regole in `../spec.md`, sezioni "Dominio" e "Whisper".

**Blocked by:** 01, 02.

**Status:** ready-for-agent

- [ ] `TranscribeCpp` riceve i Termini all'avvio della Trascrizione, accanto a `set_cancel_token` in `run_pipeline` (`managers/transcription.rs`); vale per file, Trascrivi su un Tape (anche gli Ingressi in parallelo con `load_instance`) e Trascrivi dal vivo. Verificare che ogni percorso passi da lì.
- [ ] Whisper: `initial_prompt` = Termini uniti da `", "`, nelle stesse `WhisperRunOptions` di `confident_only()`; niente prompt senza Termini.
- [ ] La correzione si applica nel `keep` di `TranscribeCpp::transcribe`, dopo il filtro `foreign_script`, sul risultato con tempi relativi; i Parziali non passano di lì.
- [ ] Test (con `tdd`) dove la seam lo permette (motore finto della pipeline o funzione estratta); lo smoke con i modelli veri resta manuale.
- [ ] AGENTS.md: una riga nell'Architettura sul Vocabolario (prompt di Whisper, correzione in `keep`, snapshot all'avvio).
- [ ] I sei controlli passano.

## Comments
