# 05: Catalogo e download dei modelli

**What to build:** in Impostazioni → Trascrizione l'utente vede i tre modelli (Nemotron consigliato, Whisper Large v3 Turbo, Parakeet TDT v3) con la dimensione del download e la modalità del testo. Può scaricarli vedendo la percentuale, annullare un download ed eliminare un modello. Un modello è utilizzabile solo dopo la verifica d'integrità, e un download interrotto riprende da dove si era fermato.

**Blocked by:** 02 (Tracer bullet)

**Status:** done

- [x] `models.json` incluso in compilazione, con URL fissati alla revision HF, SHA-256, dimensione, modalità e licenza (valori in `docs/research/transcribe-cpp-e-handy.md` §8)
- [x] Download in `app_data_dir/models`:
  - scrittura in `.partial`;
  - ripresa con `Range`;
  - verifica di dimensione e SHA-256, poi rename atomico.
- [x] `reqwest` con `native-tls`
- [x] Evento `model-download-progress`, al massimo 10 al secondo, e `model-state-changed`. Nei tipi esportati niente `u64`
- [x] Annulla cancella il parziale. Un'interruzione lo conserva. Uno SHA errato cancella il parziale e mostra un errore dedicato
- [x] "Elimina" rimuove il modello scaricato
- [x] Il download continua in background mentre si usa il resto dell'app
- [x] Test con un server HTTP locale sulla loopback: percentuale, ripresa, SHA errato, Annulla, interruzione, Elimina
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
