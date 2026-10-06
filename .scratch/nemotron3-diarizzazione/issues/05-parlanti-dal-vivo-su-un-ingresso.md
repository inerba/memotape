# 05: Mostrare i Parlanti provvisori dal vivo su un Ingresso

**What to build:** Durante una Registrazione da un solo Ingresso, Nemotron mostra il Parlante provvisorio sul testo disponibile. Pausa/Riprendi conserva le identità; Stop porta all'analisi finale. Il guasto o il ritardo del diarizer non ferma audio e testo.

**Blocked by:** 03 — Concludere la Diarizzazione dopo Stop senza perdere il Tape.

**Status:** done

**Modello consigliato:** GPT-6 Astra (`gpt-6-astra`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Streaming concorrente, code limitate, cache, Pausa e isolamento dei guasti richiedono ragionamento sul comportamento nel tempo.

**Quando aumentare lo sforzo:** valutare `xhigh` solo se restano problemi concreti di concorrenza o ordinamento che `high` non risolve; registrare il caso e verificare se lo sforzo aggiuntivo aiuta.

- [x] Con Trascrivi dal vivo e Riconosci i parlanti attivi e Nemotron selezionato, il testo disponibile, compresi i Parziali supportati dall'ASR, riceve un'attribuzione riconoscibile come provvisoria.
- [x] ASR e Diarizzazione ricevono lo stesso audio mono sulla linea del tempo salvata, prima che il VAD elimini i silenzi, senza consumare i frame l'uno dell'altro e senza rallentare la cattura.
- [x] La sessione usa il preset a bassa latenza verificato. Sono osservabili separatamente il ritardo del diarizer e quello dell'ASR; l'obiettivo 1–2 secondi non è dichiarato raggiunto dalla sola configurazione del preset.
- [x] Pausa/Riprendi mantiene cache e identità, con tempi delle Frasi coerenti con l'audio salvato e senza silenzio artificiale della pausa. Ogni nuova Registrazione avvia identità nuove.
- [x] La coda del diarizer ha un limite e una condizione verificabile di ritardo; saturazione o guasto interrompe il solo riconoscimento realtime, mostra un avviso e lascia continuare cattura e ASR. Si ritenta a posteriori quando possibile, senza scartare frame e proseguire con tempi falsi.
- [x] Stop chiude lo stream, conserva le attribuzioni disponibili e usa l'analisi finale del ticket 03. La rinomina resta indisponibile fino al termine dell'analisi finale.
- [x] Il contratto frontend/backend porta attribuzioni e stato provvisorio con identità/revisione per Ingresso; i binding sono rigenerati. Test coprono ritardi, Pausa, Stop ed errori; i sei controlli passano.
- [x] Una prova nativa registra da un singolo Ingresso e verifica che il Tape riascoltato conservi audio e testo anche dopo un guasto simulato del diarizer.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.


## Comments

### Implementazione del 2026-10-05 nel worktree `5e50`

Ticket copiato dal worktree di progettazione `0acd`; 03 e 04 sono già `done` qui. La Registrazione da un solo Ingresso affianca Nemotron 3 alla Trascrizione quando entrambe le caselle sono attive. La scelta fissata dal lease crea un `LiveDiarizer` con modello e canale propri; Sortformer e Registrazioni da Entrambi mantengono il percorso a posteriori.

`LiveFeed` manda ad ASR e diarizer gli stessi frame mono a 16 kHz prima del VAD, inclusi i silenzi. La copia verso il diarizer passa da `try_send` in un canale da 100 frame (3 s, circa 192 KB); la cattura non attende mai il modello. Coda piena o audio vecchio oltre 3 s, anche dopo il calcolo, interrompono l’intera sessione realtime con `liveDiarizationLagging`: non si salta un frame continuando con tempi falsi. Un guasto diventa `liveDiarizationUnavailable`; la Trascrizione continua e il modello si ritenta sull’audio salvato a Stop.

Lo stream usa `LowLatency` del commit già verificato nel ticket 01. Pausa/Riprendi non lo finalizza né ne crea un altro: gli stessi frame, senza audio artificiale della pausa, proseguono nella stessa cache. Ogni Registrazione crea sessione e identità nuove. Stop chiude il canale, smaltisce i frame e completa lo stream prima della seconda analisi `VeryHighLatency` del ticket 03. Un guasto del realtime non trasforma l’esito ASR in errore né cancella il risultato disponibile; Annulla nell’analisi finale mantiene le attribuzioni provvisorie.

`LiveTranscriptUpdated` porta lo snapshot di un Ingresso con UUID della Registrazione, id ASR, revisione, Frasi e Parziale attribuito. Il frontend crea l’UUID prima di Registra e lo passa al comando; anche gli eventi precedenti di testo, attribuzione finale, guasto, progresso, timer/livelli e avvio dell’analisi finale sono correlati alla sessione. Gli eventi di una Registrazione terminata non entrano nella successiva né sostituiscono una Sorgente riaperta. Gli aggiornamenti ASR e del diarizer condividono la revisione; una rettifica vecchia non ripristina testo o Parziali già superati. I turni disponibili attribuiscono in modo conservativo: con tempi non ancora coperti o più voci si mostra Parlante non determinato. Lo snapshot `finished` contiene tutto il testo finale e chiude le rettifiche, anche nel fallback Ogg e anche se arriva dopo la risposta a Stop, quando i Parziali non sono più accettati. Le divisioni durante la Registrazione restano nei ticket successivi. I binding sono generati dal backend; vista e Copia turno mostrano le etichette provvisorie dei Parziali. Rinomina e correzione restano bloccate dall’Attività già prevista.

I messaggi nelle sei lingue spiegano che audio e Trascrizione continuano e si ritenterà a Stop. L’avviso resta leggibile durante lo smaltimento e l’analisi finale; un evento del diarizer non nasconde un errore ASR già presente. I log separano la coda ASR, la fine dell’audio con testo disponibile, la fine dell’audio con turni disponibili e il tempo in coda/calcolo del diarizer. Non sono una misura della latenza di rendering della UI e non attestano l’obiettivo 1–2 secondi.

Test del core e del frontend verificano audio duplicato esatto, silenzi, Pausa senza frame aggiuntivi, saturazione, coda vecchia senza saturazione, Stop, Annulla, turni prima o dopo l’ASR, rettifiche e conservazione del testo, eventi fuori ordine, snapshot finale e priorità degli errori. Restano le seam già documentate; non è stato aggiunto un diarizer finto.

Smoke nativo manuale, separato dai test ordinari:

`MEMOTAPE_NEMOTRON3_MODEL=D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf`

`cargo test --locked --manifest-path src-tauri/Cargo.toml un_ingresso_nativo -- --ignored --test-threads=1 --nocapture`

Windows x64, Vulkan, RTX 2070 SUPER, uscita predefinita WASAPI. Riproduce la fixture sintetica a due voci e registra un solo Audio di sistema per 29 s, con circa 1 s di Pausa. Dopo 9 s di turni introduce un arresto del consumer di 4 s: la coda satura mentre ASR e cattura proseguono. Nell’ultima esecuzione: 28 046 ms salvati, 5 Frasi, un Parziale attribuito prima del guasto, testo presente anche oltre 20 s e dopo il guasto. Verificati Tape riaperto, Ogg byte per byte identico, decodifica con durata coerente con il timer e audio non muto, stato di analisi annullata con Trascrizione completa; successivo tentativo finale riuscito senza modificare il testo. Durata dello smoke 45,82 s.

Il primo tentativo aveva una parola attesa estranea alla fixture; il secondo assumeva 16 kHz anche per il decoder Opus, che restituisce il proprio sample rate. Le asserzioni sono state corrette contro contenuto reale e formato decodificato, poi la prova è passata e ripetuta dopo la revisione. Nessun cambiamento alle Impostazioni dell’utente. Questa prova non verifica interazioni UI, qualità su italiano reale, 8 voci, sessioni lunghe o latenza 1–2 s nella vista.

I controlli e l’esito finale della revisione sono riportati sotto dopo l’ultima verifica. Nessun commit o pubblicazione, come previsto dalla spec approvata.

### Controlli conclusivi del 2026-10-06

| Controllo | Esito |
|---|---|
| `bun run typecheck` | Verde |
| `bun run test` | 101 passati, 0 falliti |
| `bun run check` | Verde, 105 file |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Verde |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Verde |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 224 passati, 0 falliti, 6 smoke ignorati |

Il test dei binding conferma che `src/bindings.ts` coincide con il file generato dal backend. Il log della suite Rust è `checks-05-rust.log`; il log della prova nativa è `native-05.log`, nella cartella di questo progetto di ticket.

#### Standards

Nessuna violazione documentata o smell sostanziale residuo. Risolte le osservazioni iniziali su orchestrazione, costruzione ripetuta degli eventi, nome e responsabilità del diarizer, tipo dei Parziali e complessità dello stato di completamento. Il diff è isolato rispetto alle modifiche dei ticket precedenti, già presenti all’avvio. La verifica conclusiva sui filtri di progresso, fase e timer/livelli non ha rilievi. Totale Standards: 0 residui.

#### Spec

La prima revisione ha individuato due problemi concreti: eventi di una Registrazione precedente potevano entrare nella nuova conversazione, e il risultato terminale poteva essere ignorato dopo la risposta di Stop nel fallback Ogg. Corretti con UUID di sessione, filtro negli aggiornamenti pubblici della conversazione e accettazione del risultato `finished` dopo Stop. Due test di regressione sono stati eseguiti rossi prima della correzione e verdi dopo. La seconda revisione ha richiesto lo stesso isolamento per progresso e timer/livelli: anche questi eventi, insieme all’avvio dell’analisi finale, ora hanno la sessione e sono filtrati sia da Home sia dal pannello di Registrazione. Il pannello riparte da zero con una nuova sessione. Altri due test verificano la fase di una nuova Registrazione e gli indicatori con vecchi eventi prima e dopo quelli nuovi. La revisione conclusiva Spec non rileva problemi funzionali residui concreti. Totale Spec: 4 rilievi corretti, 0 residui.
