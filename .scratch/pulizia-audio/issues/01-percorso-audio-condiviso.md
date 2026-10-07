# 01: Preparare un percorso audio condiviso senza cambiare il risultato

**What to build:** preparare il punto comune di elaborazione dei blocchi audio, così la successiva pulizia potrà raggiungere audio salvato e Trascrizione senza creare percorsi divergenti. Questo è il solo ticket di preparazione: non introduce ancora controlli o un denoiser.

**Blocked by:** None (can start immediately).

**Status:** done

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [x] Registrazione, importazione di file audio/video e Trascrizione di Tape attraversano un contratto condiviso per blocchi PCM, con un comportamento iniziale di passaggio invariato. Audio salvato, tracce, mix, Forma d'onda e ASR conservano il risultato attuale.
- [x] Il componente del core gestisce formato, posizione temporale, chiusura e confini di configurazione senza dipendere da Tauri. L'interfaccia consente un successivo processore con ritardo e stato per Ingresso; non introduce una nuova astrazione per ciascuna operazione DSP.
- [x] L'elaborazione avviene prima della diramazione verso salvataggio e ASR, e prima della somma degli Ingressi. L'ordine conversione/Guadagno, elaborazione, somma, destinazioni è esplicito; il Guadagno mantiene il comportamento precedente.
- [x] Frequenze diverse, mono/stereo, buchi, loopback fermo, Pausa/Riprendi e Stop mantengono durata e allineamento secondo le tolleranze già richieste dall'app. La callback WASAPI non esegue elaborazione o attese aggiuntive.
- [x] Test dal blocco alle destinazioni usano le seam esistenti di motore e VAD e al massimo una nuova seam alta del processore. Un processore deterministico con trasformazione e ritardo riconoscibili dimostra che le destinazioni ricevono lo stesso risultato e che la chiusura conserva la coda utile.
- [x] Confronti PCM prima del codec e confronti con tolleranza dopo Opus verificano il comportamento osservabile; non si richiede identità bit per bit a un codec lossy.
- [x] La Diarizzazione della Registrazione resta dopo Stop e completamento ASR. Nessuna modifica a segmentazione linguistica, aggregazione delle Frasi o comportamento dei Parlanti.
- [x] Le decisioni architetturali documentano il contratto introdotto. Il prodotto non dichiara disponibile una pulizia ancora assente.
- [x] Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust sono verdi; una sola build Rust alla volta. Il resoconto distingue controlli automatici ed eventuali prove native mancanti. Nessun commit o rilascio autonomo.

## Comments

Nessuna implementazione avviata. Ticket di preparazione richiesto prima delle successive integrazioni verticali.

### Implementazione del 6 ottobre 2026

Baseline iniziale salvata prima degli edit in
`C:\Users\inerba\.codex\visualizations\2026\10\06\01a110d3-e03c-7a30-aa79-f10dfc9c363e\baseline`:
212 file di codice, contratti, spec e documentazione, HEAD e stato Git.
La review confronta questa baseline con il risultato del ticket, escludendo le
modifiche preesistenti; nessun commit. Il watcher `tauri dev`, Vite e Cargo di
questo checkout sono stati arrestati prima della compilazione.

Introdotti `AudioProcessor` e `PcmStream` nel core: formato interleaved, posizione
in frame, limite della coda, scarico a Pausa/cambio/Stop e passaggio invariato
con `Bypass`. Mixer per Ingresso e blocchi decodificati di file/Tape usano il
contratto prima di somma e destinazioni. L'errore del contratto non diventa una
chiusura riuscita. La Diarizzazione resta dopo Stop e smaltimento ASR.
Decisione in `docs/adr/0018-percorso-pcm-condiviso.md`; nessun denoiser, UI,
nuovo requisito di prodotto, modello o dipendenza introdotti.

Prove pertinenti già eseguite: 42 test `audio_toolkit` verdi; processore
deterministico con dimezzamento e ritardo, formato/posizione e cambio di
processore; mixer con frequenze 8/16/24/48 kHz, Ingressi 44,1/48 kHz mono/stereo,
buchi, loopback fermo, Pausa/Riprendi e Stop; percorso file con confronto PCM
ASR, Forma d'onda e Ogg riletto; percorso Registrazione fino al Tape separato
riaperto, con mix, tracce e ASR coerenti. Opus: durata esatta e RMSE < 0,015.

I sei controlli sono verdi sul codice finale, dopo le correzioni della review:

| Controllo | Esito |
|---|---|
| `bun run typecheck` | Verde |
| `bun run test` | 114 passati, 0 falliti |
| `bun run check` | Verde, 115 file controllati |
| `bun run format:backend` | Verde |
| `bun run lint:backend` | Verde, `--all-targets -- -D warnings` |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 256 passati, 0 falliti, 13 ignorati |

La suite Rust completa è stata eseguita fuori dal sandbox predefinito: al suo
interno il test esistente del Cestino falliva perché Windows non rendeva
disponibile il Cestino del volume. Il test isolato e la suite completa sono
passati fuori dal sandbox; nessuna modifica al codice per aggirare il problema.

Review indipendenti sul diff rispetto alla baseline:

- **Spec:** nessun finding; verifica ripetuta sullo stato finale del percorso file.
- **Standards:** risolte entrambe le osservazioni, sullo stato del ticket e sui
  tre campi opzionali correlati del percorso file. `FileProcessing` rende espliciti
  gli stati `Pending`, `Active` e `Finished`, senza gli `expect` precedenti.
  La seconda verifica conferma la risoluzione; nessun finding residuo.

La formattazione ha richiesto copie temporanee e sostituzione atomica
dei soli file del ticket perché un lettore aveva sezioni mappate aperte;
`cargo fmt --check` è poi passato senza modificare file estranei.

Limiti: nessuna cattura WASAPI o prova manuale della UI eseguita in questa
conversazione; nessun modello ASR/DFN3 reale o misura di carico nativo, né
installer. I 13 smoke test ignorati non sono stati eseguiti. Sono prove del
contratto e delle destinazioni nel core. Nessun ticket successivo avviato,
nessun commit o rilascio.
