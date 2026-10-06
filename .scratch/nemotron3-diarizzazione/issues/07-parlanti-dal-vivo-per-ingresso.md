# 07: Riconoscere i Parlanti dal vivo con gli Ingressi separati

**What to build:** Registrando da Entrambi, i Parlanti dell'Audio di sistema restano distinti dal Microfono. Il Microfono rappresenta una persona sola, salvo la sua casella per riconoscere più Parlanti; ogni Ingresso conserva identità, testo e stato propri.

**Blocked by:** 05 — Mostrare i Parlanti provvisori dal vivo su un Ingresso.

**Status:** done

**Modello consigliato:** GPT-6.1 Sol (`gpt-6.1-sol`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Estensione del percorso dal vivo già verificato a due Ingressi, mantenendo separati identità, stato e guasti.

- [x] La Registrazione da Entrambi conserva sempre mix e tracce degli Ingressi; Diarizzazione e Trascrizione lavorano per Ingresso, mai attribuendo le voci dal mix.
- [x] Audio di sistema e Microfono non condividono identità o cache. Si distinguono fino a 8 Parlanti per ogni Ingresso effettivamente diarizzato, senza promettere identità comuni fra Ingressi.
- [x] Il Microfono resta una persona sola con la sua casella spenta; attivandola si apre un'istanza e uno stream distinti. Il vincolo di un solo stream per modello è rispettato.
- [x] Frasi, Parziali e revisioni portano sempre l'Ingresso corretto. Le regole del ticket 06 sono riutilizzate quando quel ticket è integrato, senza imporne il completamento per provare la separazione degli Ingressi.
- [x] Un guasto o ritardo del diarizer di un Ingresso non arresta l'altro Ingresso, la cattura o la Trascrizione. Avvisi e analisi finale descrivono correttamente l'esito dei singoli Ingressi.
- [x] Pausa, Stop, salvataggio, copia/esportazione e riapertura conservano le identità separate; i Tape vecchi con solo mix non vengono presentati come separabili.
- [x] Test coprono due Ingressi, casella del Microfono, guasto di un solo diarizer e riapertura; i sei controlli passano. Uno smoke nativo misura anche memoria e carico con una e due istanze.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.


## Esecuzione autorizzata

Il 5 ottobre 2026 sono stati autorizzati i ticket 06, 07 e 08 in sequenza, dopo il completamento verificato dello 05, ciascuno con un agente pulito GPT-6.1 Sol e sforzo high. Avviare solo dopo la chiusura dei prerequisiti; nessun commit o pubblicazione e autorizzato.


## Esito dell’implementazione — 6 ottobre 2026

Worktree di esecuzione: `C:/Users/inerba/.codex/worktrees/5e50/sbobino`. Prerequisiti 05 e 06 conclusi; modifiche 01–06 preservate. Nessun commit, push, cambio di branch, modifica delle Impostazioni utente o pubblicazione di installer/pesi. Il ticket 08 non è stato implementato.

### Comportamento verificato

`record` collega il diarizer a ogni Ingresso presente in `Settings::parlanti_registrazione`. Da Entrambi ASR e Diarizzazione usano Microfono e Sistema; il mix resta soltanto audio di ascolto. Ogni `LiveDiarizer::run` carica un distinto modello/sessione/stream Nemotron 3: coda limitata, cache, identità e token di guasto sono indipendenti. Con casella spenta il Microfono segue ASR senza attribuzione provvisoria o Parlante non determinato; resta una persona sola rinominabile. Sortformer mantiene il percorso a posteriori. Resta il limite di 8 Parlanti per Ingresso del runtime e dei test del core; la qualità su otto voci reali non è attestata.

Snapshot, originali ASR, id delle divisioni e revisioni del ticket06 restano per Ingresso. `LiveDiarizationFailed` ora contiene anche `ingresso`; gli avvisi nominano l’Ingresso, si conservano durante completamento e analisi finale e mantengono la priorità degli errori di cattura/ASR. Le descrizioni nelle sei lingue indicano il realtime su ogni Ingresso selezionato.

La seconda analisi legge sequenzialmente gli Ogg salvati, con un modello/stream indipendente per ciascun Ingresso, e tenta anche il secondo dopo un guasto del primo. L’esito è fissato al termine di ogni analisi nativa: un Annulla durante l’altro Ingresso conserva i successi già ottenuti. `finalize_live_ingressi` riunisce o divide dagli originali ASR soltanto gli Ingressi riusciti, senza nuova Trascrizione; gli altri conservano la proiezione disponibile. La precedente implementazione alternativa `finalize_live` è eliminata: produzione, unit e smoke usano lo stesso finalizzatore.

`Diarizzazione::ingressi` è un campo facoltativo compatibile con il Tape v1 e contiene esiti indipendenti; l’esito complessivo mantiene compatibilità con i lettori correnti. Stato finale, informazioni del Tape, Copia testo e Markdown nominano gli esiti degli Ingressi. Un Tape precedente senza `ingressi` mantiene l’esito complessivo; un Tape con solo mix riapre con `ingressiSeparati: false`. Nessuna riscrittura automatica.

Test significativi coprono cache/identità/revisioni separate e coda guasta di un solo Ingresso, copia dei due Parziali con numeri di Parlante uguali, checkbox del Microfono, finale con un guasto, `[Microfono Ok, Sistema Cancelled]`, salvataggio/riapertura ed esportazione, compatibilità vecchi Tape. Le prove di Pausa/Stop e originali/snapshot06 restano nella suite.

### Sei controlli conclusivi

| Comando | Esito | Log |
|---|---|---|
| `bun run typecheck` | Verde | `checks-07-typecheck.log` |
| `bun run test` | 106 passati, 0 falliti | `checks-07-frontend.log` |
| `bun run check` | Verde, 107 file | `checks-07-biome.log` |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | Verde | `checks-07-fmt.log` |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Verde | `checks-07-clippy.log` |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 233 passati, 0 falliti, 7 smoke ignorati | `checks-07-rust.log` |

Log nella cartella `.scratch/nemotron3-diarizzazione/`. Rust sempre in sequenza. Binding generati dal builder tauri-specta tramite il test `i_bindings_committati_sono_aggiornati`: dopo il fallimento atteso per il contratto cambiato è stato copiato il file generato temporaneo in `src/bindings.ts`; nessuna modifica manuale. Il test del generatore passa nella suite conclusiva.

### Smoke nativo: una e due istanze

Ambiente: Windows x64, Vulkan, AMD Ryzen 7 3700X (8 core/16 processori logici), NVIDIA GeForce RTX 2070 SUPER; runtime fissato a `e6672a8672913b47f1571c66c54bee789d028416`, come nel ticket01. Nessuna modifica a Impostazioni o dispositivi. GGUF locale:

`MEMOTAPE_NEMOTRON3_MODEL=D:/local/tauri/sbobino-deps/nemotron3-proof/Nemotron-3-Diarization-BF16.gguf`

`cargo test --locked --manifest-path src-tauri/Cargo.toml due_ingressi_nativi -- --ignored --test-threads=1 --nocapture`

Passato dopo le correzioni di review in 154,81 s. Tre scenari sequenziali, sempre con due ASR reali: un diarizer solo sul Sistema; due diarizer entrambi sani; due diarizer con 4 s di ritardo indotto soltanto sul Microfono. La fixture sintetica da 25.675 ms è duplicata sulle due tracce, alimentata a tempo reale con 1 s di sospensione dell’alimentazione. I modelli sono precaricati/riscaldati prima dell’alimentazione; la prova non misura l’avvio della Registrazione nell’app.

| Scenario | Caricamento + riscaldamento | Working set caricati / picco campionato | Private bytes caricati / picco campionato | Wall dei flussi | CPU del processo / core equivalenti |
|---|---|---|---|---|---|
| 1 diarizer, 2 ASR, sani | 8.945 ms | 488 / 541 MiB | 1.572 / 2.485 MiB | 26,70 s | 31,98 s / 1,20 |
| 2 diarizer, 2 ASR, sani | 14.294 ms | 711 / 714 MiB | 1.807 / 3.085 MiB | 26,73 s | 34,30 s / 1,28 |
| 2 diarizer, 2 ASR, guasto Microfono | 14.198 ms | 719 / 746 MiB | 1.819 / 3.074 MiB | 26,68 s | 33,45 s / 1,25 |

Misure `Get-Process` sul solo processo Rust: CPU cumulativa divisa per wall, working set/private bytes della memoria host, campioni ogni 5 s. Il primo caso parte freddo; i successivi nello stesso processo conservano allocazioni/runtime già inizializzati. Non sono picchi continui, VRAM, utilizzo GPU o benchmark generalizzabili; la media del caso con guasto non rappresenta due diarizer attivi per tutta la durata.

ASR conserva testo per entrambi gli Ingressi oltre 20 s anche nel caso guasto; Sistema completa lo stream mentre Microfono termina con `LiveDiarizationLagging`. Con casella spenta il Microfono resta senza attribuzioni provvisorie. I casi sani eseguono realmente l’analisi finale degli Ingressi richiesti. Nel caso guasto la prova combina l’analisi finale reale di Sistema con un errore finale iniettato del Microfono, verificando consolidamento indipendente. Tape riaperto con Frasi/esiti distinti; mix, Microfono e Sistema recuperati dallo zip coincidono byte per byte con gli Ogg scritti. Log `native-07.log`.

### Regressione nativa WASAPI del ticket06

`cargo test --locked --manifest-path src-tauri/Cargo.toml un_ingresso_nativo -- --ignored --test-threads=1 --nocapture`

Passata sul nuovo finalizzatore unico in 45,36 s: 28.055 ms registrati, 32 revisioni, Parziali temporizzati attribuiti e divisioni dal vivo; Pausa, guasto isolato, testo dopo il guasto, audio conservato e analisi finale reale/Tape riaperto coerenti. Log `native-07-regressione.log`.

Le prove attestano integrazione nativa e integrità sul parlato sintetico. Restano al ticket08 la qualità sull’italiano reale, 5–8 voci reali, misure audio/testo → etichetta renderizzata, carico GPU/VRAM, avvio/cattura contemporanea dei due dispositivi nell’app e Registrazioni lunghe. Nessuna dichiarazione di verifica della UI o di latenza 1–2 s.

### Revisione e diff isolato

Due agenti GPT-6.1 Sol high in sola lettura, skill `code-review`: Standards 0 violazioni documentate/0 smell sostanziali residui; Spec 0 rilievi concreti residui. Risolti il vecchio finalizzatore presente solo nei test e Annulla globale che invalidava l’Ingresso già riuscito; entrambe le correzioni e il nuovo test di Annulla sono state rilette. Report `review-07.md`.

`07.diff` confronta la baseline pre-ticket in `0acd/.scratch/nemotron3-diarizzazione/baselines/07`, non HEAD. Sorgenti e documenti semanticamente modificati elencati in `07-files.json`; gli altri file nuovi sono log e resoconti delle verifiche. Nessun ripristino della baseline, commit o pubblicazione. Checklist conclusa senza blocchi tecnici residui nel perimetro07; il coordinatore può avviare08 separatamente.
