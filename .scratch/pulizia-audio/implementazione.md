# Implementazione sequenziale della pulizia audio

Revisione successiva, richiesta dall'utente il 6 ottobre 2026: usare DFN3
diretto senza miscela aggiuntiva. Vedi ADR-0019, la revisione della spec e
`dfn3-diretto/README.md`. Le matrici precedenti restano evidenze della versione
`pcm-snr-v1`, non attestazioni della qualità o conservazione voce di `direct-v1`.

Richiesta dell'utente del 6 ottobre 2026: implementare i sette ticket uno alla volta, con conversazioni nuove, modello 6.1 Sol e ragionamento scelto tra basso, medio e alto. La richiesta approva la suddivisione dei ticket. Ogni conversazione parte dal ticket e dalla spec, senza fork della conversazione precedente.

Tutti lavorano sul checkout locale esistente, preservando le modifiche precedenti e senza commit o rilascio autonomi come stabilito dalla spec. La skill implement viene applicata per TDD alle seam concordate, controlli e code-review.

| Ticket | Modello | Ragionamento | Motivo | Stato |
| --- | --- | --- | --- | --- |
| 01 | gpt-6.1-sol | high | Contratto PCM condiviso e preservazione dei tempi su tutti i percorsi | Completato: 01a110d3-e03c-7a30-aa79-f10dfc9c363e; sei controlli verdi, review senza problemi residui |
| 02 | gpt-6.1-sol | high | Runtime nativo, bundle, DSP e percorso completo file/Tape | Implementazione terminata, stato partial: 01a110f3-1e47-75d0-9063-f924feaa8182; verifiche finali interrotte su richiesta dell'utente, bundle e licenza pesi aperti |
| 03 | gpt-6.1-sol | high | Due Ingressi, tempi, transizioni, bypass e cattura reale | Implementazione e review concluse, partial: 01a1118a-4bb6-7161-8c05-62d6ec3b395d; sei controlli verdi e smoke nativi passati, collaudo manuale Tauri/ascolto aperto; handoff-03.md |
| 04 | gpt-6.1-sol | high | Riscrittura atomica e riuso degli intervalli trattati | Implementazione e review concluse, partial: 01a111c3-fb81-7551-a54c-1869984e0325; sei controlli verdi, 118 test frontend e 284 Rust, smoke DFN3 passato; collaudo manuale Tauri/ascolto e disco pieno aperti; handoff-04.md |
| 05 | gpt-6.1-sol | high | Taratura prudente e regressioni con tre motori ASR | Implementazione e review concluse, partial: 01a111e6-0b38-7063-8cb0-49dc32b7f352; sei controlli verdi, 119 frontend/290 Rust, criterio sintetico Bilanciato passato con tre ASR; corpus umano/ascolto/UI aperti; handoff-05.md |
| 06 | gpt-6.1-sol | high | Configurazione prima delle code e carico live a due Ingressi | Implementazione e review concluse, partial: 01a11206-a2b7-7d73-9d2c-5056119f19c4; sei controlli verdi, 119 frontend/296 Rust, matrice streaming tre ASR e WASAPI release passati; corpus umano/ascolto/UI e Diarizzazione finale reale aperti; handoff-06.md |
| 07 | gpt-6.1-sol | medium | Controlli UI su contratti già implementati e collaudo al volo | Implementazione e review concluse, partial: 01a1122c-7f30-7ac2-99b3-8a90f0710a37; sei controlli verdi, 126 frontend/296 Rust, stato condiviso e concorrenza verificati sulle seam; collaudo Tauri, tastiera/Narrator, round-trip UI e ascolto aperti; handoff-07.md |

Non vengono avviati due ticket contemporaneamente, anche quando le dipendenze lo consentirebbero. Un ticket successivo parte dopo verifica dell'esito del precedente; eventuali criteri mancanti vengono riportati senza considerarli passati.

## Verifica della redistribuzione dei pesi

Durante il ticket 02 è emersa un'incertezza sulla licenza dei pesi, distinta dalla licenza MIT/Apache del codice. Il README upstream limita la dichiarazione esplicita a «All code»; le richieste di chiarimento sui pesi restano aperte ([697](https://github.com/Rikorose/DeepFilterNet/issues/697), [700](https://github.com/Rikorose/DeepFilterNet/issues/700), [709](https://github.com/Rikorose/DeepFilterNet/issues/709)). Verifica del coordinatore del 6 ottobre 2026. Il criterio della spec relativo alla redistribuzione dei pesi non può essere considerato verificato sulla sola base della licenza del codice. L'implementazione e le prove locali proseguono; nessuna pubblicazione o rilascio è autorizzata.

## Ripresa del ticket 02

Nella stessa conversazione è comparso un nuovo turno iniziale senza i precedenti. Il coordinatore ha consegnato un checkpoint in `handoff-02.md`, con baseline originale, prove già osservate e verifiche ancora necessarie. Il codice parziale e i log sono stati preservati. Questo non costituisce completamento del ticket né avvio del successivo.

## Runtime dei controlli

Bun già installato, recuperato dai comandi verificati del ticket 01: `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`. Nei terminali che non lo trovano nel PATH usare il percorso assoluto, senza sostituire `bun test` con un runner diverso. Il ticket 02 sta verificando un target Cargo isolato in `.scratch`, per preservare l'app già aperta nel target debug usuale; il comando riuscito deve essere riportato nel suo handoff per i ticket successivi.

Comando Cargo riuscito del ticket 02: impostare per il solo processo `CARGO_TARGET_DIR=D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust` e usare `cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false ...`. Non modificare `.cargo/config.toml` dell'utente.

L'utente ha chiesto nella conversazione 02 di fermare le prove ripetute e chiudere quel task. Le prove reali già raccolte restano valide per il codice a cui si riferiscono; i controlli mancanti sono espliciti nel ticket. I ticket seguenti riutilizzano il runtime e il corpus, verificano le nuove modifiche senza ripetere tutta la calibrazione precedente e mantengono distinta l'implementazione dalla verifica completa per il rilascio.
