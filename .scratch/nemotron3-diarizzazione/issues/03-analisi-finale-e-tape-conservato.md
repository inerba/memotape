# 03: Concludere la Diarizzazione dopo Stop senza perdere il Tape

**What to build:** Dopo una Registrazione con Trascrivi dal vivo l'utente vede la fase di analisi finale dei Parlanti. Il Tape conserva audio e testo anche se questa analisi viene annullata o fallisce, e ne mostra chiaramente l'esito.

**Blocked by:** 02 — Scegliere Nemotron e riconoscere i Parlanti nei file.

**Status:** done

**Modello consigliato:** GPT-6 Astra (`gpt-6-astra`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Coordinamento fra Stop, Annulla, errori e salvataggio del Tape; audio e testo devono sopravvivere a ogni esito.

- [x] Dopo Stop, quando audio e testo sono pronti, Nemotron rilegge l'audio salvato con il preset più accurato verificato; non esegue una nuova Trascrizione e non trattiene l'intero PCM solo per questa analisi.
- [x] L'Attività mostra la fase e il controllo Annulla. Il Tape è salvato prima di dichiarare conclusa l'Attività; rinomina e correzione non interferiscono con il risultato ancora in lavorazione.
- [x] La nuova assegnazione aggiorna i Parlanti senza perdere o riformulare testo e senza alterare gli Ingressi. Il completamento del futuro stream realtime viene collegato nel ticket 05.
- [x] Annulla o errore della sola Diarizzazione finale conserva audio, testo e attribuzioni disponibili e mostra Diarizzazione non completata; non cancella né maschera errori di cattura o ASR.
- [x] Completezza della Trascrizione ed esito della Diarizzazione sono distinti. Il Tape registra modello della Diarizzazione, esito e attribuzioni provvisorie tramite metadati facoltativi compatibili con i Tape vecchi.
- [x] Riapertura, vista, Copia testo, Copia turno e Markdown distinguono attribuzioni provvisorie e non determinate; un errore non trasforma un'attribuzione incerta in definitiva.
- [x] Con Trascrivi dal vivo spento non si avvia un nuovo riconoscimento implicito. Con Sortformer resta il completamento a posteriori già previsto.
- [x] Test coprono successo, Annulla, guasto e riapertura del Tape; i sei controlli passano e uno smoke nativo verifica il salvataggio effettivo.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.


## Comments

### Implementazione del 2026-10-05 nel worktree `5e50`

La copia del ticket viene dal worktree di progettazione `0acd`; il prerequisito 02 è già `done` in questo worktree. La selezione del diarizer ora vale anche per la Registrazione con Trascrivi dal vivo, fissata all’avvio senza fallback. `transcribe_live` esegue ASR; dopo Stop `record` attende gli Ogg e il testo, quindi analizza solo gli Ingressi richiesti rileggendo l’audio salvato. Nemotron usa lo stream `VeryHighLatency` con decodifica a blocchi; la Registrazione non accumula PCM per la Diarizzazione. Sortformer mantiene il contratto offline.

Annulla cambia bersaglio sotto un lock: durante cattura/smaltimento annulla l’ASR, nell’analisi finale usa un token separato. `completa` fotografa l’esito ASR prima del cambio di fase. Successo, Annulla ed errore dell’analisi finale conservano audio, testo e attribuzioni disponibili; i turni finali vengono applicati solo dopo l’analisi di tutti gli Ingressi e l’ultimo controllo Annulla. L’Attività resta occupata fino al salvataggio del Tape e impedisce scritture sul risultato in lavorazione.

Il Tape v1 conserva `diarizzazione` facoltativa (modello ed esito) e `parlante_provvisorio` facoltativo sulle Frasi. I Tape precedenti restano leggibili e non sono riscritti. La vista e il Markdown mostrano «Diarizzazione non completata» dopo Annulla o guasto; etichette, Copia testo, Copia turno e lettura MCP indicano le attribuzioni provvisorie. I tratti non determinati restano distinti. Gli errori di cattura e ASR hanno priorità sugli esiti della Diarizzazione. Le sei lingue sono aggiornate; i bindings sono stati rigenerati dall’esportatore tauri-specta, non modificati a mano.

Verifica con test prima rossi e poi verdi alle interfacce del core e del Tape: conservazione e riapertura con attribuzioni provvisorie, successo/Annulla/guasto, annullamento dopo l’ultimo Ingresso e guasto del secondo Ingresso senza rettifiche parziali. Sono coperti anche Annulla/guasto su Frasi senza attribuzione e testo vuoto senza una falsa analisi completata. Test frontend coprono etichette provvisorie nella vista e nell’elenco Parlanti, Copia turno e priorità degli errori; un test MCP verifica la lettura del nome provvisorio.

Controlli completati:

- `bun run typecheck`: verde.
- `bun run test`: 94 passati, 0 falliti.
- `bun run check`: verde, 103 file controllati.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`: verde.
- `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: verde.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml`: 207 passati, 4 ignorati, 0 falliti. Fuori sandbox per il test del Cestino Windows: nella sandbox il solo test del Cestino dava «il volume non ha un Cestino»; il test isolato e la suite completa passano nell’ambiente Windows ordinario.

Smoke nativo distinto, in sequenza su Windows x64/Vulkan, RTX 2070 SUPER: `MEMOTAPE_NEMOTRON3_MODEL=D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf`, `cargo test --locked --manifest-path src-tauri/Cargo.toml nemotron3_analisi_finale_salva -- --ignored --test-threads=1 --nocapture`. Superato in 13,78 s: ASR reale sulla fixture sintetica a due voci, seconda analisi sul suo Ogg, Parlanti 1/2, salvataggio con `save_tape` e riapertura con identico audio, testo, Forma d’onda e metadati. Questo non attesta interazioni UI, Registrazione dai dispositivi reali, qualità dell’italiano reale o prestazioni su 8 voci: sono verifiche separate. L’attribuzione durante la Registrazione e le divisioni del testo restano nei ticket successivi.

Nessun commit o pubblicazione, come richiesto dalla spec approvata. La revisione del ticket confronta una copia dello stato iniziale del worktree con i cambiamenti di questo turno; non include il lavoro già presente del ticket 02.

### Revisione e correzioni finali

La revisione Spec ha individuato due casi limite, entrambi corretti: nessuna falsa «Completata» se il testo non contiene Frasi, e nessuna trasformazione preventiva in «Parlante non determinato» su Annulla o guasto. La seconda lettura ha individuato anche Annulla ASR senza Frasi: ora il controllo precede evento e thread finale, e `Ok([])` non produce un esito neppure con token già annullato. La regressione è passata da rossa a verde. Se la prenotazione del modello è fallita, il Tape conserva «Fallita» anche senza testo; con modello disponibile e nessuna analisi l’esito è assente.

La revisione Standards ha richiesto di rispettare le seam previste da AGENTS: il consolidamento ora riceve turni già calcolati, mentre il manager esegue il diarizer. I test passano dati ed errori al consolidamento, senza simulare un motore aggiuntivo. Sono condivisi caricamento del diarizer e scelta della regola di assegnazione; `reserve_configured_diarizer` descrive anche l’uso nelle Registrazioni.

**Spec, verifica finale:** nessun rilievo bloccante residuo; confermati conservazione, cancellazione, metadati, copie/riapertura e Sortformer.

**Standards, verifica finale:** nessuna violazione residua e nessun code smell rilevante. Corretta anche la nota documentale di PRODUCT: disponibilità e compatibilità riguardano il diarizer selezionato, non solo Sortformer.
