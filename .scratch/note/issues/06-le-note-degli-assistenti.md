# 06: Le Note degli Assistenti

**What to build:** un Assistente legge e scrive le Note (ADR-0031). Con il permesso di lettura `list_notes` (nome, autore, ora) e `read_note`; con quello di modifica `write_note(tape, nome, testo, atteso)`, che crea la Nota se non c'è o la riscrive se il testo attuale è quello atteso, e `rename_note`; nessuna eliminazione. L'autore dell'ultima scrittura (utente, o Assistente con il nome del client MCP) e l'ora si salvano e si vedono sotto la riga dei nomi: «Assistente (Claude) · 17:52», «Tu · 18:03». Al ritorno in primo piano l'app rilegge le Note del Tape aperto: quelle senza bozza si aggiornano, su una con bozza un avviso offre «Tieni le mie» o «Usa le sue». Il prompt MCP «Scrivi le Note» crea una Nota nuova «Riepilogo» (nella Lingua del parlato, «Riepilogo 2» se c'è già) con Decisioni, Da fare e Aperto, ogni punto con il suo tempo.

**Blocked by:** 03, 04

**Status:** ready-for-agent

- [ ] Gli strumenti rispettano i due permessi, il lock su file e `activityInProgress`, con errori leggibili per nome già usato, testo atteso obsoleto e Nota non trovata
- [ ] Test degli strumenti come quelli di ADR-0031 in `mcp.rs`, compreso l'autore con il nome del client
- [ ] Un test o una verifica a mano prova l'avviso «Tieni le mie» / «Usa le sue» su una Nota con bozza
- [ ] Il prompt ha la ricetta di `PRODUCT.md` e non tocca le Note esistenti
- [ ] `docs/sviluppo/impostazioni-e-servizi.md` elenca strumenti e prompt
- [ ] Testi nelle sei lingue
- [ ] I sei controlli verdi
