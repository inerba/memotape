# 04: Dividere il testo al cambio di Parlante con tempi affidabili

**What to build:** Nel risultato di un file o Tape, due persone che si alternano dentro la stessa Frase diventano tratti leggibili con il proprio Parlante e il proprio intervallo audio. Il testo resta integro; Whisper applica il comportamento conservativo concordato.

**Blocked by:** 02 — Scegliere Nemotron e riconoscere i Parlanti nei file.

**Status:** done

**Modello consigliato:** GPT-6 Astra (`gpt-6-astra`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Allineamento dei tempi ASR con i Parlanti e divisione del testo senza alterarne contenuto, ordine o identità.

- [x] La Trascrizione conserva i tempi realmente esposti da Nemotron/Parakeet e li colloca sulla linea del tempo della Sorgente; non inventa timestamp distribuendo le parole lungo la durata.
- [x] Le divisioni seguono cambi di Parlante sostenuti da tempi affidabili; concatenando le parti si conservano testo, ordine e punteggiatura senza duplicazioni o perdite.
- [x] Ogni Frase risultante ha id univoco per Ingresso e intervallo valido. Player, Turni, correzioni, indice della Libreria, copia, esportazione e riapertura agiscono sullo stesso risultato.
- [x] Whisper conserva i segmenti disponibili; un tratto con più voci non separabili è Parlante non determinato. Parakeet e Whisper continuano a mostrare il testo alla chiusura della Frase.
- [x] Voci simultanee o tempi insufficienti non producono una falsa separazione delle parole né un Parlante aggiuntivo per rappresentare l'incertezza.
- [x] Il cambiamento del risultato ASR mantiene verdi i percorsi esistenti mediante adattamento compatibile, senza una migrazione indiscriminata dei Tape vecchi.
- [x] Test significativi coprono alternanze, tempi relativi/assoluti, sovrapposizioni e conservazione del testo; i sei controlli passano. Uno smoke con modelli reali verifica i tempi usati.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.

## Comments

### Implementazione del 2026-10-05 nel worktree `5e50`

Il ticket è stato copiato dal worktree di progettazione `0acd`; il prerequisito 02 è già `done` qui. `TranscriptionEngine` restituisce `AsrResult`: testo e intervalli indivisibili con offset UTF-8. Il wrapper richiede la granularità offerta dal modello e conserva parole di Nemotron/Parakeet, token dello stream raggruppati ai confini di parola e segmenti di Whisper. Gli offset vengono verificati contro il testo finale; se le righe non coincidono, si conserva il testo senza inventare un allineamento. Le due pipeline traslano dal primo frame effettivamente dato al motore, includendo il prefill del VAD.

Lo smoke ha rilevato che l'ultimo intervallo di Nemotron può superare di 70 ms l'audio della fixture, per il passo encoder di 80 ms del runtime fissato. Solo questo padding viene intersecato con l'audio reale; tempi esterni oltre un passo, invertiti o disordinati non producono divisioni. Nessuna parola viene distribuita artificialmente lungo la Frase.

`assign_configured` applica con Nemotron la divisione ai confini affidabili e mantiene testo, ordine, spazi, accenti e punteggiatura. Una parola o un segmento con due voci rimane non determinato; le parti attribuibili intorno possono avere i propri Parlanti. Sortformer conserva la regola precedente. Le Frasi vengono riordinate per inizio, e `Document::new` assegna identità univoche al risultato definitivo. Questa regola vale anche nell'analisi finale dopo Stop, senza aggiungere ancora le rettifiche durante la Registrazione.

Il Tape v1 aggiunge `tempi` facoltativo; i documenti precedenti restano leggibili e non vengono riscritti automaticamente. La correzione elimina i tempi legati al testo originale, conservando intervallo audio e Parlante. Vista, Player, Turni, ricerca, copia ed esportazione ricevono il risultato attraverso la riapertura già prevista. Quando cambiano le divisioni, a ASR terminata si ripubblicano tutte le `TranscriptPhrase` con identità per Ingresso: le vecchie righe vengono sostituite e si aggiungono le parti, anche nel fallback Ogg se non si riesce a salvare il Tape. Con layout invariato restano gli eventi delle sole etichette. Il comando riapre il Tape completo, se salvato. Il contratto degli eventi pubblici resta invariato e il controllo dei bindings passa.

Test prima rosso e poi verde per la divisione di una Frase con due voci, per il padding effettivamente rilevato e per la sostituzione delle Frasi ripubblicate nel frontend. `withPhrase` sostituisce per la coppia Ingresso/id; il test verifica anche idempotenza, Turni, Copia turno e conservazione delle Frasi prima e dopo la rimozione dei Parziali. Test del core coprono tempi assoluti, testo esatto, sovrapposizioni, segmenti Whisper, dati mancanti o invalidi e Sortformer. Una verifica integrata con due Ingressi passa da salvataggio e riapertura, copia/Markdown, indice della Libreria e correzione, controllando audio e Forma d'onda. Nessuna nuova seam finta oltre a `TranscriptionEngine` e `VoiceDetector`.

Controlli completati:

- `bun run typecheck`: verde.
- `bun run test`: 95 passati, 0 falliti.
- `bun run check`: verde, 103 file.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`: verde.
- `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: verde.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml`: 217 passati, 5 ignorati, 0 falliti, nell'ambiente Windows ordinario per il test del Cestino.

Smoke nativi separati, in sequenza su Windows x64/Vulkan, RTX 2070 SUPER:

- `tempi_asr_reali_e_divisione_del_testo_sui_tre_modelli`: passato in 9,09 s. Fixture `parlato-it.wav`, Nemotron/Parakeet con 14 tratti di parola, Whisper con 2 segmenti; verificati tempi assoluti, testo conservato e divisioni con turni di controllo. Nemotron emette 6 Parziali con callback, Parakeet/Whisper nessuno.
- `nemotron3_analisi_finale_salva_e_riapre_la_registrazione` e `nemotron3_trascrive_un_file_e_riapre_il_tape_con_i_parlanti`: entrambi passati, 22,93 s complessivi, con il modello locale del ticket 01 e Nemotron ASR. Verificati salvataggio, riapertura, Parlanti, testo, audio e Forma d'onda.

Queste prove non attestano qualità su italiano reale, riconoscimento di 8 voci o interazioni UI. Le revisioni confrontano lo stato iniziale del worktree con le sole modifiche del ticket 04:

- Standards: nessuna violazione residua delle regole del repository. Rafforzato il commento di `assign_configured` per dichiarare divisione e riordinamento; i suggerimenti P3 sulla tupla interna dei tempi e sul nome della funzione non richiedono nuove astrazioni per questo ticket.
- Spec: corretto il P2 sulle duplicazioni nel frontend dovute alla ripubblicazione delle Frasi finali. La revisione della correzione conferma la sostituzione per Ingresso/id, la conservazione del testo nel fallback Ogg e nessun blocco residuo.

Ticket chiuso dopo i sei controlli e gli smoke nativi. Nessun commit o pubblicazione, come previsto dalla spec.

