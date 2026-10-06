# Verifica delle correzioni manuali

6 ottobre 2026. Implementazione della specifica approvata in `spec.md`, confrontata con lo snapshot precedente all'implementazione. Il checkout conteneva già modifiche ad altre funzionalità: `implementation.diff` isola questa richiesta.

## Controlli automatici

| Controllo | Esito |
|---|---|
| `bun run typecheck` | Verde |
| `bun run test` | 114 test superati, nessun errore |
| `bun run check` | Verde, 110 file controllati |
| `cargo fmt --check` | Verde |
| `cargo clippy --all-targets -- -D warnings` | Verde |
| `cargo test` | 249 test superati, nessun errore, 13 smoke nativi ignorati come previsto |

Clippy e i test Rust usano l'override **temporaneo del solo processo** `TAURI_CONFIG={"bundle":{"resources":[]}}`: l'app di sviluppo aperta blocca la copia delle DLL eseguita da tauri-build. Le DLL richieste dai test restano preparate da build.rs; nessuna configurazione del progetto o delle Impostazioni viene cambiata. La suite Rust completa è eseguita fuori dalla sandbox perché il test preesistente del Cestino Windows fallisce nella sandbox e passa fuori. Biome esclude `.scratch` e gli output `target`, che contenevano centinaia di JSON generati non formattati.

I nuovi test coprono persistenza del testo corretto, nessuna correzione per testo invariato, copie e riapertura, unione sopra/sotto del solo Turno completo, rifiuto atomico di richieste obsolete o non valide, Ingressi distinti, sorgente sconosciuta, ricerca FTS dopo l'unione, attribuzioni manuali protette durante l'analisi, solo testo modificato ancora riattribuibile e nomi conservati dopo una permutazione dei numeri.

## Prova interattiva

`ui-smoke.html` monta **i componenti di produzione** TranscriptView e TapeHeader in Vite, con dati di prova e callback simulate. Non accede ai Tape dell'utente né alle sue Impostazioni. È un harness di verifica temporaneo, escluso dai controlli e dalla build dell'app.

- Riprodotto il difetto originale: clic su Mario apriva più input, perdeva il fuoco e spostava la sezione a `scrollTop=2022`. Dopo la correzione: un solo input, fuoco sul campo e `scrollTop=0`.
- Rinomina con Invio ed Esc; Invio senza modificare il nome non salva e non accende il badge.
- Modifica del testo con Invio ed Esc; selezione da tastiera visibile nei temi chiaro e scuro, cursore con il colore del testo. Le selezioni usano salvia piena e testo a contrasto.
- Menu prima di Copia turno, con sopra/sotto/Annulla; prima e ultima direzione disabilitate correttamente; Esc chiude il popover.
- Unione sopra e sotto: Mario–Anna–Mario diventa un Turno di Mario con tre paragrafi; l'intervento successivo di Anna rimane Anna. Testo e riferimenti dei paragrafi restano distinti.
- Badge dopo la correzione salvata tramite callback; l'effettiva persistenza nel Tape e nelle copie è verificata dai test Rust.

La prova usa il browser integrato di Codex su Windows, **non la WebView2 della finestra Memotape**. La validazione manuale nella finestra nativa e gli smoke con modelli reali restano da eseguire. Non sono attestazioni di qualità della diarizzazione su audio reale.

## Review

- **Standards:** review parallela prevista dalla skill code-review. Rilievi iniziali risolti: selezione/caret documentati e menu condiviso conforme a DESIGN.md. Ricontrollo senza violazioni residue.
- **Spec:** review conclusa sul diff aggiornato, senza problemi concreti e riproducibili. Corretti i due casi rilevati durante la review: Invio su un nome invariato non salva metadati e Annulla durante la copia della nuova Trascrizione conserva il Tape originale. Rimane soltanto il limite di verifica nativa descritto sopra.

Esito finale: Standards 0 rilievi residui; Spec 0 rilievi residui. Le due review non attestano la UI nativa o la qualità dei modelli reali.

Nessun commit né pubblicazione eseguiti.
