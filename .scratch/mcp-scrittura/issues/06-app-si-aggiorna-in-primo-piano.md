# 06: L'app si aggiorna quando torna in primo piano

**What to build:** quando la finestra di Memotape torna in primo piano, l'app riallinea l'indice della Libreria (come all'avvio) e ricarica il Tape aperto se la sua data di modifica è cambiata su disco. Una bozza aperta nell'editor del Turno non si perde: il salvataggio fallisce sul testo atteso e la bozza resta con l'errore. Nessun watcher del file system. Vedi `../spec.md` (storie 35–37, decisione «App al ritorno in primo piano»).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Modificando, rinominando o cestinando un Tape da fuori con l'app aperta, al ritorno in primo piano Libreria e ricerca sono aggiornate
- [ ] Il Tape aperto, cambiato da fuori, si ricarica al ritorno in primo piano; se non è cambiato non si ricarica
- [ ] Logica pura (se ricaricare) con un `*.test.ts` accanto, se ce n'è; il resto verificato a mano con `bun run app:prova` e CDP (esiti annotati in fondo al ticket)
- [ ] `docs/sviluppo/tape-e-libreria.md` aggiornato
- [ ] I sei controlli verdi
