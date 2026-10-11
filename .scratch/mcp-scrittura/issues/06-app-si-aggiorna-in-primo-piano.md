# 06: L'app si aggiorna quando torna in primo piano

**What to build:** quando la finestra di Memotape torna in primo piano, l'app riallinea l'indice della Libreria (come all'avvio) e ricarica il Tape aperto se la sua data di modifica è cambiata su disco. Una bozza aperta nell'editor del Turno non si perde: il salvataggio fallisce sul testo atteso e la bozza resta con l'errore. Nessun watcher del file system. Vedi `../spec.md` (storie 35–37, decisione «App al ritorno in primo piano»).

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] Modificando, rinominando o cestinando un Tape da fuori con l'app aperta, al ritorno in primo piano Libreria e ricerca sono aggiornate
- [x] Il Tape aperto, cambiato da fuori, si ricarica al ritorno in primo piano; se non è cambiato non si ricarica
- [x] Logica pura (se ricaricare) con un `*.test.ts` accanto, se ce n'è; il resto verificato a mano con `bun run app:prova` e CDP (esiti annotati in fondo al ticket)
- [x] `docs/sviluppo/tape-e-libreria.md` aggiornato
- [x] I sei controlli verdi

## Comments

11 ottobre 2026, verifica a mano (exe di debug con `MEMOTAPE_DATA_DIR` di prova, Vite del worktree, CDP; ritorno in primo piano con riduci a icona e ripristina della finestra):

- Tape aperto e non cambiato: al ritorno solo `tape_modificato`, nessun `open_tape`.
- Testo della prima Frase riscritto da fuori (zip riscritto con tmp + rename, come un Assistente) e un altro Tape rinominato in Esplora file: al ritorno il Tape aperto mostra il testo nuovo, i Recenti il titolo nuovo, `library_search` trova la parola nuova.
- Durata cambiata da fuori (come un Taglio): il player si rimonta con `?v=1` e la riga dice la durata nuova.
- Bozza nell'editor del Turno mentre il Turno cambia da fuori: all'uscita dal campo il salvataggio fallisce («il testo del Turno è cambiato»), la bozza resta nel campo e il Tape tiene il testo di fuori.
- Tape aperto spostato fuori dalla Libreria: al ritorno la vista si chiude (Home) e sparisce dai Recenti.

Nota: con l'app di `tauri dev` dell'utente aperta da `target\debug`, `tauri-build` non riesce a ricopiare le DLL di `runtime-libs` (os error 32) né cargo a sostituire `memotape.exe`; i controlli Rust sono passati con `TAURI_CONFIG={"bundle":{"resources":null}}` e l'app di prova è partita dall'exe in `target\debug\deps`, con `devUrl` sulla 1430 per servire il frontend del worktree.
