# Ticket 08 — verifica Windows del 6 ottobre 2026

**Aggiornamento dopo le prove create su richiesta:** [addendum con cinque confronti reali eseguiti](report08-addendum.md). I capitoli sotto documentano la prima fase, precedente alla costruzione del corpus; i requisiti mancanti sul corpus sono aggiornati nell’addendum. Stato attuale ready-for-human, non done.

**Esito: ready-for-human, non done.** Il realtime su CPU non supera questa prova; Vulkan raggiunge p95 di circa 1,73 s nei callback del core sulla fixture sintetica, con massimi oltre 2 s. Non sono disponibili campioni italiani reali con annotazioni indipendenti e non è stato verificato il rendering dell’app. Nessuna conclusione di qualità italiana, DER reale o promozione del modello.

## Ambiente e riproducibilità

Worktree `C:\Users\inerba\.codex\worktrees\5e50\sbobino`; Windows 11 Pro 10.0.26200, Ryzen 7 3700X (8 core/16 thread), RTX 2070 SUPER 8192 MiB, driver 591.86. Bun 1.4.0, rustc/cargo 1.96.1, Python 3.14.3. Dati verificati in `benchmark08/environment.json`, inclusi percorsi e hash delle DLL. Build debug MSVC; non installer/release. Runtime transcribe-cpp 0.2.3 fissato a `e6672a8672913b47f1571c66c54bee789d028416`, runtime effettivo `e6672a8`. DLL transcribe SHA-256 `97e20118624825899f319627cbe9d10f4c10fb3e1969a55f67edebce5e4687be`.

| Modello | Byte | SHA-256 |
|---|---:|---|
| Nemotron 3 Diarization BF16 locale | 198937280 | `4b11ce10e009fedf496cc9f879dc634e67605ecbf240463a155e0128657019e3` |
| Nemotron 3.5 ASR streaming 0.6B Q5_K_M | 559647200 | `86429e8c4f7fdcf9b3312269ad1ca6669478ba7805331c4aea7a2e33e9910d65` |
| Sortformer 4spk v2.1 Q8_0 | 139310336 | `a5dacdc650790266c7a362e54e6bf51952015487edaa606c4e11632bc32442a9` |

ASR italiano, Silero del repository, frame 30 ms mono 16 kHz. Backend imposto e verificato CPU oppure Vulkan0; non fallback implicito. Nemotron 3 `LowLatency` dal vivo, `VeryHighLatency` nell’analisi finale. Il modello ASR e i diarizer sono caricati prima del clock audio; l’avvio è riportato separatamente. Nessuna modifica persistente delle Impostazioni, ACL o Git. Sortformer resta il default e disponibile.

## Banco nativo e significato dei ritardi

`engine/windows_benchmark.rs` è solo codice di test Windows. Attraversa feed/pipeline ASR, diarizzazione incrementale, rettifiche della proiezione e finalizzatore di produzione, scrittura/rilettura Tape. Quattro processi distinti per CPU/Vulkan × uno/due Ingressi; 300 s alimentati al ritmo reale. Ripete la fixture Windows TTS `parlato-due-voci.wav` (25,675 s, Elsa/Cosimo), SHA-256 `c300d92aed389cbf6764e6acb3323f89f23de2d0f6f9c589ea88bd55dd7db42a`. Le due tracce sono duplicati della stessa fixture; il mix del banco è una di esse. Non prova cattura di due dispositivi o qualità sul parlato reale. Pausa di 1 s a metà, `ClosePhrase` effettivo, esclusa dai 300 s salvati.

Per ogni unità temporizzata che sopravvive nell’ASR finale con lo stesso testo/intervallo si misura: consegna al feed del frame della sua fine → testo ASR disponibile; stesso frame → prima attribuzione determinata; testo disponibile → attribuzione. I tempi audio sono quelli stimati dall’ASR, arrotondati al frame del feed. Osservazioni su clock monotono nei callback del core, senza rendering frontend né timestamp acustici di dispositivo. L’etichetta è la prima determinata, non una voce corretta verificata. Si escludono dalle distribuzioni le unità senza etichetta, riportate nel denominatore; nessun campione mancante viene trasformato in zero. Le revisioni temporanee che non corrispondono a un’unità finale sono escluse. p50/p95/p99 nearest rank, min e singole unità nei JSON. Zero testo→etichetta significa che i turni erano già pronti quando quel testo è comparso.

| Caso/Ingresso | Unità etichettate/totali | Audio→testo p50/p95/p99/max ms | Audio→etichetta p50/p95/p99/max ms | Testo→etichetta p50/p95/p99/max ms |
|---|---:|---|---|---|
| Vulkan 1 mix | 506/506 | 858 / 1452 / 1538 / 1589 | 1009 / 1723 / 2058 / 2271 | 0 / 1260 / 1798 / 1894 |
| Vulkan 2 microfono | 506/506 | 915 / 1496 / 1582 / 1620 | 1031 / 1730 / 2022 / 2312 | 0 / 1284 / 1800 / 1895 |
| Vulkan 2 sistema | 506/506 | 908 / 1504 / 1571 / 1634 | 1035 / 1728 / 2021 / 2322 | 0 / 1293 / 1786 / 1881 |
| CPU 1 mix | 131/508 | 997 / 1573 / 1899 / 2326 | 1544 / 3698 / 4882 / 4972 | 354 / 2421 / 3702 / 4295 |
| CPU 2 microfono | 50/508 | 1142 / 2520 / 4400 / 4949 | 2210 / 4638 / 4949 / 4949 | 0 / 472 / 1634 / 1634 |
| CPU 2 sistema | 43/508 | 1133 / 3452 / 5417 / 6280 | 2036 / 5440 / 6280 / 6280 | 0 / 480 / 489 / 489 |

**CPU: prova non riuscita.** Il modello attiva `LiveDiarizationLagging` realmente, senza fault artificiale: una pipeline nel caso 1 e entrambe nel caso 2. Mancano 377/508, 458/508 e 465/508 etichette rispettivamente. Nessuna unità etichettata negli ultimi due blocchi di 100 s. Il basso testo→etichetta CPU2 riguarda le poche unità iniziali e non compensa l’assenza delle altre. ASR/audio proseguono e il finale riesce. **Vulkan:** p95 audio→attribuzione entro 2 s in questo smoke, p99 circa/oltre 2 s e massimi 2,27–2,32 s. Non si certifica il target 1–2 s su ogni parola, sul parlato reale o nella UI. p95 per blocchi di 100 s: Vulkan1 1720/1584/1756 ms; Vulkan2 microfono 1730/1619/1737, sistema 1728/1601/1737. Nessuna crescita monotona evidente in cinque minuti, senza garanzia su un’ora.

La frontiera dei turni è un indicatore della distanza dal feed, non profondità misurata di ogni coda: p95 892/903/906 ms Vulkan; 3095/3240/3129 ms CPU prima dell’interruzione. Le code ASR sono state drenate a Stop; non è strumentata la profondità massima dei canali né ogni stadio del rendering.

## Carico, memoria, Stop e integrità

Campionamento host del solo processo ~1 s, intera GPU tramite nvidia-smi ~3 s. CPU media = CPU cumulativa / wall campionata, in core, include avvio/stream/finale. WS e private bytes sono picchi campionati, non picchi continui. VRAM e utilizzo GPU sotto indicati sono dell’intera scheda, includono altre app: non sono memoria attribuita al processo. Il banco trattiene anche le osservazioni per misurare, quindi la memoria non è quella esclusiva della UI di produzione.

| Caso | CPU media core | WS picco MiB | Private picco MiB | WS mediana 30–60 s → 270–300 s MiB | GPU globale picco MiB / uso medio % | Avvio ms | Stop→drain ms | Analisi finale ms |
|---|---:|---:|---:|---|---|---:|---:|---:|
| Vulkan 1 | 0,59 | 556 | 1654 | 427 → 465 | 3971 / 12,82 | 1181 | 325 | 32451 |
| Vulkan 2 | 1,25 | 568 | 3159 | 556 → 568 | 4773 / 13,14 | 2254 | 257 | 63640 |
| CPU 1 | 2,98 | 1636 | 2193 | 1630 → 1634 | 3346 / 0,45 | 1299 | 198 | 40448 |
| CPU 2 | 4,62 | 3188 | 4307 | 3184 → 3187 | 3343 / 0,56 | 2873 | 533 | 81605 |

Tutti i processi terminano senza timeout/crash. Finale completo per ciascun Ingresso, ASR originale preservata per Ingresso, Ogg byte-identici dopo scrittura Tape, durata decodificata 300 s usando il rate Opus reale 48 kHz, rilettura Document identica e stesso numero di Frasi alla riapertura. Picchi e mediana non dimostrano assenza di leak su una Registrazione lunga; CPU stabilizza anche perché il diarizer è stato fermato dalla protezione. Gli 81,605 s del finale CPU2 e 63,640 s Vulkan2 sono analisi sequenziali di due tracce da 300 s.

`benchmark08/summary300.json` esplicita 4 casi completati e `all_realtime_healthy: false`; `audit300.json` verifica identità/runtime/integrità di ogni report. La versione del runner avviata prima dell’esito CPU controllava l’exit del test; native exit 0 significa integrità verificata, non realtime sano. Il runner finale rifiuta binari privi del test, report assenti/incoerenti, timeout e qualsiasi errore live, continuando gli altri casi e terminando con exit 1 in tali casi. La protezione negativa è provata con il binario `main` che contiene zero test; prova positiva successiva registrata separatamente.

## Confronto qualità pronto, ma non eseguito sul reale

`ticket08_confronto_stessa_asr` trascrive una sola volta e riusa esattamente PCM/testo/tempi italiani per Sortformer e Nemotron 3, stesso Ingresso/backend. Smoke Vulkan riuscito sulla fixture TTS: RTTM, ASR congelata, proiezioni e unità in `benchmark08/comparison-synthetic`. Nessuna reference indipendente: `no_reference_scored: true`, nessun DER o errore di attribuzione pubblicato. Il confronto è offline; non confronta le latenze realtime Sortformer/Nemotron. Le prove con 5–8 voci, overlap, rumore e alternanze reali non sono svolte.

`metrics.py` implementa DER esatto ad intervalli, collar zero, overlap incluso, regione annotata esplicita, miss/false alarm/confusion e denominatore in speaker-secondi, mapping ottimale uno-a-uno fino a otto voci. Separatamente valuta attribuzione delle medesime unità ASR congelate: corretto/errato/non determinato/reference ambiguo, escludendo e contando unità fuori/a cavallo della stessa regione DER. Non calcola WER, non crea groundtruth dal modello. Otto test con attesi manuali indipendenti verificano permutazione voci, errori, overlap/duplicati, regioni, otto voci/limite, RTTM errato e separazione degli errori di testo.

Ricerca mirata, nessun audio locale trasmesso: [KIParla](https://kiparla.it/en/search/) vieta download/registrazione dell’audio; [EVALITA 2009](https://www.evalita.it/campaigns/evalita-2009/data-distribution/) rimanda agli organizzatori; [MagicHub](https://magichub.com/datasets/italian-conversational-speech-corpus/) richiede accesso; [VoxPopuli ufficiale](https://github.com/facebookresearch/voxpopuli) documenta speaker per segmenti ASR e grandi download (44–75 GB), senza reference pronta di conversazione/overlap adatta al ticket. Non sono stati scaricati corpora enormi né inventate annotazioni. Non significa che un corpus valido non esista: manca un campione immediatamente accessibile con licenza audio e annotazioni verificate. Il coordinatore ha richiesto eventuali percorsi locali all’utente; nessuno ricevuto al momento del resoconto.

## Prove native sul flusso e limiti

`native08-flow.log`: vero runtime Vulkan su Tape da 300 s, `CancelToken` durante `diarize_saved` restituisce `Cancelled`, passato al `finalize` di produzione; testo e provvisorietà conservati, audio invariato dopo rewrite. Player core Range 100–199 = 206 e byte corretti, correzione con tempi/ID conservati, rinomina, copia testo/Markdown nelle sei lingue, riapertura, etichetta provvisoria tradotta; Tape v1 senza nuovi campi apre senza riscrittura e con audio identico. Questo attraversa manager/core di testo e Tape ma non l’orchestrazione completa della Registrazione o gli eventi della vista; gli stati provvisori sono preparati dal banco prima del vero errore nativo. Non è playback ascoltato, clipboard OS, dialogo Esporta o verifica UI.

`native08-wasapi.log`: smoke reale della cattura loopback WASAPI dell’uscita predefinita, fixture riprodotta localmente, Pausa e Stop; 45,75 s di test, 28054 ms salvati, 5 Frasi, 32 revisioni, Parziali con tempi/suddivisioni, guasto isolato, analisi finale e Tape con audio identico. Non è cattura da microfono né collaudo UI.

`native08-dual.log`: tre scenari sequenziali con due ASR reali e uno/due diarizer su tracce dalla fixture; 155,50 s totali. Scenari sani: `(Sistema, Ok)` con un diarizer e `(Microfono, Ok), (Sistema, Ok)` con due. Il fault deliberato sul Microfono produce `LiveDiarizationLagging` soltanto lì; Sistema continua sano, audio/testo dei due Ingressi e tutti e tre gli Ogg/Tape restano integri, finale dei soli Ingressi richiesti verificato. Non equivale a due dispositivi reali. I dati spot di memoria/carico di questo vecchio smoke non sostituiscono il campionamento continuo della matrice; il processo riusa allocazioni.

`runner08-positive.log`: runner finale provato su una nuova sessione Vulkan1 da 13 s; exit 0, `validReport: true`, `liveDiarizationHealthy: true`, 23,59 s process wall. `benchmark08/runner-guard.log`: main senza test respinto prima dei casi. Gli smoke con modelli reali sono tutti sequenziali.

Le API CUA native sono disabilitate in questa sessione. Nessuna attivazione/interazione UI dell’app né sei layout localizzati attestati. `ui08-da-verificare.md` è una procedura concreta per una persona, comprende due dispositivi, rettifiche/suddivisioni, ambiguità/finale/Annulla durante secondo Ingresso, copia/export/player/riapertura/legacy e almeno 60 minuti CPU/Vulkan × uno/due Ingressi. Una sessione da cinque minuti è stress prolungato, non soddisfa la prova di Registrazione lunga.

## Comandi, controlli e revisione

Da root 5e50, uno smoke/build Rust alla volta. Percorsi e variabili di prova nel [README del banco](../../tools/nemotron3-benchmark/README.md):

```powershell
cargo test --locked --manifest-path src-tauri/Cargo.toml metriche_distinguono
python -m unittest discover -s tools/nemotron3-benchmark -v
./tools/nemotron3-benchmark/run.ps1 -Seconds 300
python tools/nemotron3-benchmark/summarize.py
cargo test --locked --manifest-path src-tauri/Cargo.toml ticket08_confronto_stessa_asr -- --ignored --test-threads=1 --nocapture
cargo test --locked --manifest-path src-tauri/Cargo.toml ticket08_flusso_tape -- --ignored --test-threads=1 --nocapture
cargo test --locked --manifest-path src-tauri/Cargo.toml un_ingresso_nativo -- --ignored --test-threads=1 --nocapture
cargo test --locked --manifest-path src-tauri/Cargo.toml due_ingressi_nativi -- --ignored --test-threads=1 --nocapture
```

Cartelle di caso non sovrascritte; conservare precedenti o usare durata diversa. Il runner documenta l’eseguibile e rifiuta quello sbagliato. Le prime prove 10/11 s hanno scoperto due errori nel banco (rate di decodifica Opus e confronto dell’ASR dopo consumo del finalizzatore), corretti prima della matrice; log conservati. Quattro preflight da 12 s sono riusciti. Nessuna modifica a modello/runtime per mascherare gli esiti.

La revisione finale è in `review-08.md`: Standards e Spec, due agenti GPT-6.1 Sol high in sola lettura, finding risolte e requisiti mancanti separati. I sei controlli finali sono tutti verdi, con log separati sotto. Diff isolato `08.diff`, elenco `08-files.json`: manifest di 194 file della baseline pre-08 verificato con SHA-256, nessun ripristino. Le modifiche 01–07 restano integralmente; sul loro codice si aggiunge soltanto l’inclusione del modulo di test in `transcribe_cpp.rs`. Nessun contratto Tauri cambia, nessun binding da rigenerare. Nessun commit/push/reset/checkout/publishing. Continuazione, identità globali, allineatore Whisper e distribuzione restano esclusi.

### Controlli finali effettivamente eseguiti

| Controllo | Esito | Evidenza locale |
|---|---|---|
| `bun run typecheck` | verde | `checks08-typecheck.log` |
| `bun run test` | 106 pass, 0 fail | `checks08-frontend.log` |
| `bun run check` | verde, 159 file | `checks08-biome.log` |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | verde | `checks08-fmt.log` |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | verde | `checks08-clippy.log` |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 234 pass, 0 fail, 10 ignored | `checks08-rust.log` |

Gli ignored non sono dedotti da cargo test: benchmark, comparatore, flow, WASAPI e dual sono eseguiti esplicitamente nei log native/runner. Otto test Python aggiuntivi passati in `metrics08-tests.log`. Prima esecuzione Biome segnalava soltanto formattazione JSON degli artefatti nuovi; formattati esclusivamente quegli artefatti, nessuna configurazione/allentamento del controllo.

Restano aperti: confronto reale annotato 2–4/5–8 con DER e errori di attribuzione, condizioni reali/overlap/rumore, rendering UI/copia turno/clipboard/playback, due dispositivi, Annulla nell’orchestrazione del secondo Ingresso e Registrazioni lunghe. Il risultato CPU negativo è già ottenuto e non richiede un’annotazione per essere riportato. Requisiti incompleti e fallimento realtime sono entrambi espliciti nel tracker ready-for-human.
