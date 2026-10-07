# Verifica dell'editor continuo del Turno

Data: 6 ottobre 2026. Spec: [spec.md](spec.md). Decisione persistente: [ADR-0020](../../docs/adr/0020-testo-continuo-del-turno.md).

## Risultato

Ogni Turno modificabile usa un campo di testo nativo continuo. Cursore, selezione e cancellazione attraversano le Frasi dello stesso Turno. Invio inserisce un a capo; l'uscita dal campo salva; Esc ripristina la bozza presente al focus; Ctrl+Z usa l'annullamento nativo durante la scrittura.

Il Tape conserva la correzione completa con i riferimenti alle Frasi originali. Audio, Forma d'onda, intervalli e identificativi restano disponibili. I tempi ASR del testo sostituito vengono rimossi. Un Turno svuotato rimane modificabile con «Blocco senza testo», senza esportare questa indicazione.

Copia e Unisci attendono lo stesso salvataggio in corso. La risposta a una modifica non sostituisce la Sorgente se nel frattempo si è aperto un altro Tape. Un errore lascia la bozza nel campo e mostra un messaggio; un secondo tentativo può salvarla.

## Controlli del repository

Esito finale: tutti e sei i controlli verdi, dopo le correzioni della review.

| Controllo | Esito | Log |
| --- | --- | --- |
| bun run typecheck | Superato | typecheck.log |
| bun run test | 117 superati, 0 falliti | test.log |
| bun run check | 116 file, nessun errore | check.log |
| cargo fmt --check | Superato | format-backend.log |
| cargo clippy --all-targets -- -D warnings | Superato | lint-backend.log |
| cargo test | 272 superati, 0 falliti, 20 ignorati | test-backend.log |

Il caso del Turno vuoto unito sopra/sotto è stato prima riprodotto con test falliti, poi corretto e verificato. I test frontend conservano anche a capo iniziali e finali intenzionali del testo vicino.

Le dipendenze Rust sono quelle fissate dal lockfile. Durante le prove si usa soltanto nel processo `TAURI_CONFIG={"bundle":{"resources":[]}}`, per evitare di ricopiare DLL già caricate dall'istanza nativa. La suite Rust completa viene eseguita fuori dal sandbox per il test del Cestino Windows.

## Prove nella finestra Windows

È stata avviata un'istanza debug isolata di Memotape con WebView2 reale, cartella del profilo dedicata e porta CDP 9228. La fixture è un Tape con audio Ogg/Opus reale, tre Frasi di Mario e un Turno di Anna. I salvataggi passano attraverso i comandi Rust dell'app; non sono simulati nel browser.

Fixture e immagini sono in `C:\Users\inerba\.codex\visualizations\2026\10\06\01a11078-9a68-7d91-b4aa-1d056bb836f9`. Le Registrazioni dell'utente non sono state modificate. La sessione nativa usa un server Vite temporaneo che esclude le directory delle altre compilazioni da `.scratch`, evitando ricaricamenti estranei alla prova.

| Prova | Esito ed evidenza |
| --- | --- |
| Clic all'inizio, nel mezzo e alla fine | 12 casi superati: due temi × audio fermo/in ascolto × tre posizioni. Inserimento nel punto cliccato e Ctrl+Z verificati. `native-matrix-results.json` |
| Cursore visibile e lampeggiante | Sequenze di sei immagini per combinazione. Confronto tra fotogrammi stabili: differenza di un solo tratto verticale di 1 × 20 pixel nell'editor, in tutte le quattro combinazioni. `native-blink-results.json` |
| Selezione | Trascinamento del mouse su tre righe e Shift+frecce oltre il separatore; immagini in entrambi i temi, anche durante l'ascolto. |
| Backspace e Canc | Rimuovono il separatore tra prima e seconda Frase. Ctrl+Z ripristina il testo. `native-results.json` |
| Invio, Esc, incolla | Invio resta nell'editor; Esc scarta la modifica corrente; incolla multilinea, blur e riapertura conservano testo e a capo. `native-results.json` |
| Player durante la modifica | Posizione del cursore, selezione e Ctrl+Z restano validi durante l'avanzamento dell'audio. `native-results.json` |
| Salvataggio fallito | Ostacolo temporaneo alla riscrittura del Tape: nessuna variazione del file, bozza recuperabile, errore visibile, Esc ripristina la bozza precedente, nuovo tentativo riuscito. `native-failure-empty-results.json` |
| Nessuna modifica e Turno vuoto | Focus/blur lascia il Tape identico byte per byte; cancellare tutto salva il vuoto e conserva il Turno vicino. Placeholder «Blocco senza testo» visibile. `native-failure-empty-results.json` |
| Copia e Unisci durante il salvataggio | Ritardo di 250 ms applicato al solo IPC di prova: Copia turno usa il nuovo testo; Unisci attende il salvataggio e conserva la correzione. `native-actions-results.json` |
| Evidenziazione del testo corretto | Durante l'audio si evidenzia il Turno intero; nessuna evidenziazione fittizia delle Frasi; selezione stabile. `native-highlight-results.json` e `editor-corrected-highlight.png` |

Gli script ripetibili sono `cdp.ts`, `create-native-fixture.py`, `native-editor-test.ts`, `native-caret-matrix.ts`, `native-failure-empty.ts`, `native-actions-test.ts` e `native-highlight.ts`, in questa directory. I file `native-*-results.json` contengono i risultati osservati, non dati di prodotto.

## Persistenza, uscite e nuove analisi

I test al confine pubblico di modifica/riapertura del Tape verificano salvataggio atomico, no-op, riferimenti obsoleti, errore di scrittura, testo multilinea e vuoto, località della correzione e contenuto dell'audio/Forma d'onda. Il rendering di Copia testo e Markdown usa la correzione completa senza riemettere le Frasi originali.

Il test `editor_turno_ricerca_e_assistenti_leggono_solo_la_correzione` verifica FTS, ricerca e `read_around`: il vecchio testo e il placeholder del vuoto non diventano risultati. La correzione è contenuta nel Tape stesso, quindi viene conservata dalle copie del documento.

`editor_turno_ridiarizzazione_conserva_correzioni_senza_inventare_allineamenti` include testo unito, multilinea e vuoto, nuove attribuzioni, attribuzioni protette e attribuzioni protette ambigue. Il nuovo modello può aggiornare voci non protette. Se divide una correzione fra voci discordanti, la vista mostra il gruppo come Parlante non determinato, conservando le nuove attribuzioni nei riferimenti originali: non esiste un allineamento attendibile per distribuire le parole corrette fra le voci. L'ADR-0020 registra questa scelta.

La riconciliazione delle identità usa anche l'evidenza analizzata dei tratti protetti, senza cambiarne l'attribuzione manuale. Un tratto protetto ambiguo non trasferisce il suo nome a una voce automatica priva di corrispondenza affidabile.

Gli avvisi «Corretto a mano», Diarizza di nuovo e Trascrivi di nuovo riutilizzano il contratto precedente. La suite completa include le relative regressioni. Non è stata eseguita una nuova ASR con modelli reali dalla finestra di prova.

## Review

Sono state eseguite due review indipendenti, Standards e Spec, sul confronto con la copia del checkout prima dell'implementazione. Le modifiche concorrenti della pulizia audio sono escluse dal confronto della funzione.

- **Standards:** corretti selezione/focus, duplicazione dell'applicazione di `OpenedTape`, copertura delle funzioni pure e documentazione precedente. Rimosse anche le classi di focus generiche ridondanti; resta `focus-visible`.
- **Spec:** corretta la corsa tra blur e Copia/Unisci. Documentata la proiezione non determinata con voci discordanti. Aggiunta la prova del Turno svuotato e poi unito sopra/sotto: non deve aggiungere separatori alle uscite, mentre gli a capo presenti nel testo non vuoto restano intatti.

Il riesame finale della Spec non rileva residui funzionali. Tutti i rilievi ricevuti sono stati gestiti; resoconto affiancato in [review.md](review.md).

## Limiti delle prove

Le prove native attestano l'editor e il salvataggio nella WebView2 Windows di sviluppo su questa macchina. Non attestano un nuovo installer, l'aggiornamento dell'app già installata, altre macchine o la qualità dei modelli su parlato reale. I 20 test ignorati richiedono modelli, cattura o prove esplicite e non sono stati eseguiti in questa sessione.

I Tape precedenti restano leggibili senza migrazione. Una vecchia versione dell'app non interpreta il nuovo metadato del testo continuo, come precisato nell'ADR-0020.

Nessun commit o rilascio eseguito.
