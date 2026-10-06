# 06: Rettificare e dividere il testo già comparso dal vivo

**What to build:** Mentre la Registrazione prosegue, le nuove informazioni sui Parlanti correggono e dividono il testo già comparso. Frasi e Parziali restano coerenti anche quando gli aggiornamenti arrivano in ritardo.

**Blocked by:** 04 — Dividere il testo al cambio di Parlante con tempi affidabili; 05 — Mostrare i Parlanti provvisori dal vivo su un Ingresso.

**Status:** done

**Modello consigliato:** GPT-6.1 Sol (`gpt-6.1-sol`), su scelta esplicita per questa esecuzione.

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Revisioni fuori ordine, divisioni retroattive e passaggio dai Parziali alle Frasi devono conservare una vista coerente.

**Quando aumentare lo sforzo:** valutare `xhigh` solo se restano problemi concreti di concorrenza o ordinamento che `high` non risolve; registrare il caso e verificare se lo sforzo aggiuntivo aiuta.

- [x] La regola di divisione del ticket 04 si applica al testo dal vivo quando i tempi sono affidabili; il testo incerto resta esplicitamente non determinato.
- [x] Un aggiornamento può rettificare il Parlante e la suddivisione di testo già emesso, preservando ordine, testo e punteggiatura; il passaggio da Parziale a Frase non duplica o lascia testo fantasma.
- [x] Identità e revisioni per Ingresso impediscono a eventi vecchi o fuori ordine di sovrascrivere un risultato più recente; il comportamento è deterministico e testato.
- [x] Turni e colori seguono le attribuzioni correnti; Copia testo e Copia turno rappresentano la stessa versione visibile e non presentano un'etichetta provvisoria come definitiva.
- [x] L'analisi finale riesamina le attribuzioni usando i tempi conservati senza rieseguire ASR. Il Tape finale, il testo in vista e quello riaperto hanno le stesse Frasi e gli stessi tempi.
- [x] Whisper mantiene i segmenti conservativi e Parakeet non acquista Parziali inventati; il risultato non separa parole simultanee senza evidenza.
- [x] Test coprono rettifiche, divisioni, eventi fuori ordine e passaggio Parziale/Frase; i sei controlli passano e uno smoke nativo verifica l'intero flusso.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.


## Esecuzione autorizzata

Il 5 ottobre 2026 sono stati autorizzati i ticket 06, 07 e 08 in sequenza, dopo il completamento verificato dello 05, ciascuno con un agente pulito GPT-6.1 Sol e sforzo high. Avviare solo dopo la chiusura dei prerequisiti; nessun commit o pubblicazione e autorizzato.

## Comments

### Implementazione e verifiche del 2026-10-06 nel worktree `5e50`

Prerequisiti 04 e 05 verificati `done`; esecuzione autorizzata in sequenza 06 → 07 → 08. Implementato soltanto 06, senza commit o pubblicazione, preservando le modifiche già presenti dei ticket 01–05. Confronto isolato rispetto allo snapshot pre-ticket in `0acd/.scratch/nemotron3-diarizzazione/baselines/06`, mai usato per ripristinare sorgenti.

Il contratto `TranscriptionEngine` porta testo e tempi ASR anche nei Parziali. Lo stream Nemotron conserva il testo di visualizzazione esistente e allinea lo snapshot strutturato: se testo e timestamp non coincidono, conserva il testo senza tempi. Gli aggiornamenti di risultato possono rettificare anche i tempi. La pipeline li trasla dal primo frame della Frase e interseca soltanto il padding encoder ammesso dal ticket 04; Whisper e Parakeet continuano a usare `run` e non emettono Parziali.

`LiveTranscript` conserva Frasi ASR originali e Parziale corrente separatamente dalla proiezione visibile. Riusa `diarize::divide`, con l'orizzonte dei turni disponibile: le parole/segmenti ancora scoperti o con voci simultanee restano non determinati. Nuovi turni possono dividere, rettificare e riunire testo già mostrato. Testo UTF-8, punteggiatura e ordine rimangono identici. Gli id delle parti appartengono al confine testuale della Frase ASR: una riunione e una nuova divisione riusano le identità. La Frase conclusa sostituisce tutte le parti del Parziale; un Parziale tardivo della Frase già conclusa viene ignorato.

`LiveTranscriptUpdated` ora contiene `partials`, tutte le parti del Parziale insieme a tutte le Frasi dell'Ingresso. ASR e diarizer condividono una revisione; il frontend sostituisce l'Ingresso in modo atomico. Sessione e revisioni filtrano eventi superati; dopo il primo snapshot di un Ingresso anche eventi legacy senza revisione vengono ignorati. Il terminale `finished` usa `u32::MAX`, viene accettato nel fallback Ogg anche dopo la risposta a Stop, non può regredire per un terminale duplicato e chiude soltanto il proprio Ingresso. Le emissioni terminali di Registrazione non sono seguite da rettifiche legacy. Restano i filtri del ticket 05 per UUID, progresso, fase, guasti, timer e livelli.

Turni, colori e Copia turno derivano dalla revisione corrente. Copia testo passa al comando il testo visibile al click, con le parti del Parziale e le etichette provvisorie, anche se il backend ha già una revisione successiva. Il pulsante è disponibile anche quando è comparso soltanto il primo Parziale. Il backend rende lo snapshot nelle Impostazioni di copia senza modificarlo né salvarlo come testo ASR concluso.

`Transcript::live_asr` conserva solo testo e tempi originali, senza PCM. Dopo Stop `finalize_live` riesamina una copia di quegli originali e può riunire una divisione provvisoria, senza chiamare di nuovo ASR. Successo consolida il nuovo documento; Annulla o errore conserva la proiezione disponibile. Terminale, Tape e riapertura condividono Frasi, tempi e attribuzioni. Il campo interno degli originali non viene scritto nel Tape v1.

Test significativi: divisione/riunione del Parziale e della Frase, identità riutilizzate, orizzonte in ritardo, voci simultanee non separabili, segmenti Whisper conservativi, traslazione e limiti dei tempi Parziali, consolidamento finale senza ASR, Annulla/errore, snapshot visibile in entrambe le modalità di copia, eventi fuori ordine e legacy, terminali duplicati, finalizzazione indipendente per Ingresso. Non sono state aggiunte seam finte per il diarizer.

#### Sei controlli conclusivi

| Comando | Esito | Log |
|---|---|---|
| `bun run typecheck` | Verde | `checks-06-typecheck.log` |
| `bun run test` | 103 passati, 0 falliti | `checks-06-frontend.log` |
| `bun run check` | Verde, 106 file | `checks-06-biome.log` |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Verde | `checks-06-fmt.log` |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Verde | `checks-06-clippy.log` |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 229 passati, 0 falliti, 6 smoke ignorati | `checks-06-rust.log` |

Log nella cartella `.scratch/nemotron3-diarizzazione/`. Una sola compilazione Rust per volta. I binding sono generati con l'export del builder Rust usato dal test `i_bindings_committati_sono_aggiornati`: dopo il primo fallimento atteso per contratto cambiato, il file generato temporaneo è stato copiato in `src/bindings.ts`; il test del generatore passa nella suite finale. Nessuna modifica manuale dei tipi. Durante il giro conclusivo sono stati corretti un warning Clippy di closure ridondante e il formato della nuova lista JSON di evidenze; i controlli sono poi passati.

#### Smoke nativo, distinto dai test ordinari

`MEMOTAPE_NEMOTRON3_MODEL=D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf`

`cargo test --locked --manifest-path src-tauri/Cargo.toml un_ingresso_nativo -- --ignored --test-threads=1 --nocapture`

Windows x64, Vulkan, RTX 2070 SUPER, un Audio di sistema WASAPI sull'uscita predefinita; fixture sintetica a due voci, circa 1 s di Pausa, guasto del consumer dopo 9 s di turni per saturare la coda. Passato in 45,20 s: 28.059 ms salvati, 33 revisioni osservate, Parziali con tempi affidabili e almeno una divisione dal vivo. Ogni snapshot conserva esattamente il testo degli originali e ha id univoci tra Frasi e parti del Parziale. ASR prosegue dopo il guasto, il Tape conserva l'Ogg byte per byte e la durata decodificata coincide col timer. Verificati salvataggio provvisorio con analisi annullata, nuova analisi finale sull'audio salvato senza ASR, Tape finale riaperto con Frasi, id, tempi, Parlanti e ambiguità coerenti. Log: `native-06.log`. Nessuna modifica alle Impostazioni dell'utente.

`cargo test --locked --manifest-path src-tauri/Cargo.toml tempi_asr_reali_e_divisione -- --ignored --test-threads=1 --nocapture`

Regressione sequenziale dei tre ASR reali, passata in 10,41 s. Nemotron: 6 Parziali richiesti, con almeno un risultato temporizzato affidabile, 14 tratti finali. Parakeet: 0 Parziali e 14 parole. Whisper: 0 Parziali e 2 segmenti indivisibili. Verificate traslazione dei tempi e divisione conservativa del testo. Log: `native-06-asr.log`.

Queste prove verificano integrazione nativa e integrità sul parlato sintetico; non attestano interazioni UI, latenza audio/testo → etichetta renderizzata di 1–2 s, qualità su italiano reale, otto voci, sessioni lunghe o il realtime da Entrambi. Il perimetro di Entrambi resta al ticket 07.

#### Revisione sul diff isolato

Due agenti puliti GPT-6.1 Sol, high, secondo `code-review`, in sola lettura: Standards 0 violazioni documentate e 0 smell sostanziali; Spec 0 rilievi concreti residui e nessuno scope creep. Entrambi hanno confermato anche gli ultimi scostamenti (semplificazione Clippy, test dei tempi Parziali, smoke ASR e pulsante Copia testo sul solo Parziale). Report in `review-06.md`.

Sorgenti/documenti modificati dal solo ticket elencati in `06-files.json`; diff completo rispetto alla baseline in `06.diff`. Ulteriori nuovi file sono esclusivamente log e resoconti di queste verifiche. Nessun commit, push, modifica di branch/configurazione o pubblicazione; 07 e 08 non avviati.
