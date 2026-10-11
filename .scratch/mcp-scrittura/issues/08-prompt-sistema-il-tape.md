# 08: Prompt «Sistema il Tape» e prova completa

**What to build:** il server MCP offre il prompt «Sistema il Tape» con argomento il Tape, che guida l'Assistente in tre passi: correzione leggera Turno per Turno solo dove ci sono errori evidenti; Taglio dei Silenzi con un solo `cut_tape`; titolo, solo se quello attuale è generico, al massimo 100 caratteri, nella Lingua del parlato, sull'argomento, senza data. Testo esatto della ricetta in ADR-0031 e `../spec.md`. Poi la prova completa con un Assistente vero.

**Blocked by:** 04, 05, 07

**Status:** resolved

- [x] Il server elenca il prompt e lo restituisce con il Tape indicato (test dalla seam del `Server`)
- [x] Prova a mano: Claude Code collegato all'exe di debug (`--mcp`, cartella dati di prova) esegue «Sistema il Tape» su un Tape di prova; esito annotato in fondo al ticket
- [x] `docs/sviluppo/impostazioni-e-servizi.md` descrive strumenti e prompt
- [x] I sei controlli verdi

## Comments

- Il prompt è `fix_tape` (titolo «Sistema il Tape», argomento `tape`): un messaggio `user` con la ricetta di ADR-0031, in `fix_tape_recipe`. Le capabilities del server hanno `prompts` e `tools` (`#[prompt_handler]` sotto `#[tool_handler]`: nell'ordine inverso `prompts` non compare).
- Prova a mano, 11 ottobre 2026, exe di debug con `MEMOTAPE_DATA_DIR` su una cartella temporanea: settings con i due consensi accesi, Libreria con `Registrazione 2026-10-11 10-00-00.tape` (40 s, quattro Frasi di due Parlanti con errori: minuscole, punteggiatura, «allora allora», «eh», «si si», «venerdi»; Silenzi all'inizio, 13 s in mezzo e alla fine), indice da `Library::open` + `sync` in un test temporaneo poi tolto.
  - JSON-RPC su stdio: `initialize` dà `capabilities` `{prompts, tools}`; `prompts/list` elenca `fix_tape` con `tape` obbligatorio; `prompts/get` restituisce la ricetta con il Tape; poi `read_turns`, due `edit_turn`, un terzo `edit_turn` con il testo atteso vecchio rifiutato («changed since you read it»), `find_silences` (tre Silenzi), un `cut_tape` (40 000 → 16 993 ms, originale nel Cestino), `find_silences` vuoto, `rename_tape` che dà il percorso nuovo, `read_turns` sul percorso nuovo con il testo corretto. Ogni chiamata ha la sua riga `Assistente:` in `Memotape.log`.
  - Claude Code 2.1.289 (`claude -p --mcp-config … --strict-mcp-config --allowedTools mcp__memotape`): lo slash command `/mcp__memotape__fix_tape` è elencato, ma in `-p` non chiama `prompts/get` (nessuna riga `prompt fix_tape` nel log; il modello dice di non avere la ricetta e, senza, chiede conferma prima del Taglio). Passando come messaggio il testo di `prompts/get`, Claude esegue la ricetta per intero e nell'ordine: `read_turns`, `edit_turn` sui due Turni, `find_silences`, un solo `cut_tape` con i tre Tagli proposti, `rename_tape` in «Preventivo impianto fotovoltaico Ferrara» (generico il titolo di partenza, niente data, in italiano). Da provare nella sessione interattiva di Claude Code o in Claude Desktop, dove lo slash command del prompt si espande.
  - Dopo la prima prova la ricetta dice anche che l'originale di un Taglio va nel Cestino di Windows, da cui si ripristina: senza, Claude lo chiamava «irreversibile».
  - Le prove hanno mandato nel Cestino di Windows quattro Tape di prova della cartella temporanea.
