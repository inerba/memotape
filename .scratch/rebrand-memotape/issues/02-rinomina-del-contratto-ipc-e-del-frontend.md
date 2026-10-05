# 02: Rinomina del contratto IPC e del frontend

**What to build:** anche il contratto tra backend e frontend e tutto il frontend parlano di Tape:
- comandi `open_tape`, `tape_text`, `tape_peaks`, `rename_tape`, `move_tape`, `trash_tape`, `take_pending_tape` e gli altri con `bino` nel nome;
- tipi esportati `TapeInfo`, `OpenedTape`, `TapeEntry`…, con i campi `bini` che diventano `tapes` (per esempio in `LibraryList`);
- evento `tape-requested`;
- codici d'errore `unsupportedTape` e `tapeNotFound`;
- `bindings.ts` rigenerato e committato;
- nel frontend componenti, hook, funzioni pure, test e file (`bino-*` diventa `tape-*`): `TapePane`, `TapeMenu`, `TapeHeader`, `AllTapes`, `useTapeOperations`, `isTape`, `sortTapes`…;
- le chiavi i18n `bino`, `binoNotFound` e `unsupportedBino` diventano `tape`, `tapeNotFound` e `unsupportedTape`, nelle sei lingue.

Restano come sono i **valori** tradotti ("Bino"), l'estensione `.bino` e il nome Sbobino: l'utente non vede differenze.

Spec: `.scratch/rebrand-memotape/spec.md` (storie 24, 25, 28).

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] In `src/` e nei comandi Rust nessun identificatore, nome di file, chiave i18n o commento usa `bino`/`bini`. Fanno eccezione i valori tradotti e le stringhe dell'estensione `.bino`
- [ ] `bindings.ts` è rigenerato con `bun tauri dev` e committato, e `i_bindings_committati_sono_aggiornati` è verde
- [ ] I test di i18n trovano le chiavi rinominate in tutte e sei le lingue, comprese le forme plurali di `tape.parlanti`
- [ ] L'app di sviluppo apre un Bino, lo riproduce, lo corregge, lo rinomina, lo sposta e lo cerca come prima
- [ ] AGENTS.md usa i nomi nuovi di comandi, eventi e componenti
- [ ] I sei controlli sono verdi
