# 01: Una Nota si scrive e resta nel Tape

**What to build:** il percorso completo più stretto. Nel Tape aperto, accanto a Trascrizione e Parlanti, c'è la scheda «Note» con un editor CodeMirror 6 semplice (Markdown grezzo, senza resa ibrida). In un Tape senza Note la scheda mostra subito l'editor vuoto con il segnaposto; il primo carattere crea la Nota «Note», che si salva da sola dopo 1 s di pausa e uscendo dal campo nella voce `note.json` del Tape (ADR-0032), sotto il lock di riscrittura e con il controllo del testo atteso. Riaprendo il Tape la Nota ricompare. Vedi `../spec.md` (formato, comandi, errori) e `PRODUCT.md` («Note»).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Un Tape senza `note.json` ha zero Note; salvare una Nota riscrive solo `note.json` e lascia le altre voci byte-identiche
- [ ] Il salvataggio con un testo atteso diverso da quello su disco è rifiutato con un codice di `AppError` dedicato, mappato a un messaggio tradotto
- [ ] Correzione di un Turno, nome di un Parlante e ritrascrizione conservano `note.json`; le Note non accendono «Corretto a mano»
- [ ] La scheda «Note» c'è nel Tape aperto, con il testo nella colonna da lettura e i token di `DESIGN.md`; «Salvate» sotto la scheda dopo il salvataggio
- [ ] Testi nelle sei lingue di `src/locales/`
- [ ] `docs/sviluppo/tape-e-libreria.md` descrive `note.json`
- [ ] I sei controlli verdi
