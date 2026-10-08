# 01: Via lo strato dei Parlanti dal vivo, resta lo snapshot finale

**What to build:** La Registrazione si comporta come oggi, ma il core non contiene più il percorso realtime dei Parlanti, irraggiungibile dalla revisione del 6 ottobre di ADR-0016. A fine Registrazione il frontend riceve, per ogni Ingresso con Frasi, un solo snapshot finale con sessione, Ingresso e Frasi. Decisione: ADR-0016, revisione dell'8 ottobre; spec: `../spec.md`.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Spariscono dal core: `LiveSession` e il suo ramo in `transcribe_live`; `LiveDiarizer`, `LiveTranscript`, la coda dei frame verso il diarizer e `OfflineDiarizer::diarize_live`; `LiveFeed::set_diarization`; `LiveSource.diarizer`; `Transcript::live_asr` e il suo ramo in `finalize_live_ingressi`; l'evento `LiveDiarizationFailed`; il banco del ticket 08 e gli smoke nativi dei ticket 05–07.
- [x] Restano: l'analisi finale dopo Stop sugli Ogg, `diarize::divide`, la prenotazione del diarizer all'avvio con l'errore nell'esito finale (PRODUCT.md, storia sul diarizer assente), la lettura di `parlante_provvisorio` e `Diarizzazione::ingressi` nei Tape già salvati.
- [x] `LiveTranscriptUpdated` ha solo `sessionId`, `ingresso`, `phrases`; `final_speakers` lo emette una volta per Ingresso con Frasi, solo per una Registrazione. `bindings.ts` è rigenerato e committato.
- [x] Il frontend compila con la forma nuova: lo snapshot finale sostituisce le Frasi del suo Ingresso e un secondo snapshot dello stesso Ingresso nella stessa sessione si scarta; il listener di `liveDiarizationFailed`, `withLiveDiarizationError` con i suoi test e le chiavi i18n rimaste senza uso spariscono. Il resto del protocollo lo riscrive il ticket 02.
- [x] AGENTS.md non descrive più i Parlanti dal vivo dei ticket 05–07, le revisioni per Ingresso, `live-diarization-failed` né lo smoke `un_ingresso_nativo`; una riga descrive lo snapshot finale.
- [x] I sei controlli passano.
- [x] Prova manuale con `bun tauri dev`: Registrazione da Entrambi con Trascrivi dal vivo e Riconosci i parlanti, Stop, testo finale con i Parlanti nella vista e nel Tape riaperto.

## Comments

- 2026-10-08, implementazione: eliminati `live_diarization` (con gli smoke nativi dei ticket 05–07), il banco `windows_benchmark` del ticket 08, `LiveSession`, `LiveDiarizationFailed`, `OfflineDiarizer::diarize_live`, `LiveFeed::set_diarization`, `LiveSource.diarizer`, `Transcript::live_asr` e anche gli errori `liveDiarizationLagging`/`liveDiarizationUnavailable`, rimasti senza produttori, con le loro chiavi i18n. `pipeline::transcribe` e `LiveFrames::backlog` restano solo per i test (`#[cfg(test)]`): fuori dai test si usa `transcribe_protected`. `bindings.ts` rigenerato con il test ignorato `rigenera_bindings_di_sviluppo`. Nel frontend `Conversation.revisions`/`liveFinished` diventano `finali` (Ingressi con snapshot finale); sparisce anche `Status.diarizerErrors`, che solo `withLiveDiarizationError` riempiva. Sei controlli verdi. **Da fare:** la prova manuale con `bun tauri dev` (Entrambi, Trascrivi dal vivo, Riconosci i parlanti, Stop, Tape riaperto) non è stata eseguita.
- 2026-10-08, prova manuale via CDP su `bun tauri dev` (branch di integrazione `f6ec8a7c`): Trascrivi su un file (Tape creato e riaperto) e Annulla a 400 ms (nessun Tape); Registrazione da Entrambi con Trascrivi dal vivo e Riconosci i parlanti sull'Audio di sistema (fixture a due voci suonata dalle casse): testo dal vivo, Pausa con timer fermo e Riprendi, Stop → completamento → analisi finale → Tape riaperto con 2 Parlanti; correzione del Tape precedente consultato durante la Registrazione salvata nel Tape senza toccare il testo dal vivo; Annulla della Preparazione torna al Tape precedente senza crearne uno; Riconosci i parlanti su un Tape corretto a mano con conferma, correzione conservata. Non provati: guasto della pulizia audio, fallback Ogg. Tape di prova nel Cestino, impostazioni ripristinate.
