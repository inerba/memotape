# 08: Prompt «Sistema il Tape» e prova completa

**What to build:** il server MCP offre il prompt «Sistema il Tape» con argomento il Tape, che guida l'Assistente in tre passi: correzione leggera Turno per Turno solo dove ci sono errori evidenti; Taglio dei Silenzi con un solo `cut_tape`; titolo, solo se quello attuale è generico, al massimo 100 caratteri, nella Lingua del parlato, sull'argomento, senza data. Testo esatto della ricetta in ADR-0031 e `../spec.md`. Poi la prova completa con un Assistente vero.

**Blocked by:** 04, 05, 07

**Status:** ready-for-agent

- [ ] Il server elenca il prompt e lo restituisce con il Tape indicato (test dalla seam del `Server`)
- [ ] Prova a mano: Claude Code collegato all'exe di debug (`--mcp`, cartella dati di prova) esegue «Sistema il Tape» su un Tape di prova; esito annotato in fondo al ticket
- [ ] `docs/sviluppo/impostazioni-e-servizi.md` descrive strumenti e prompt
- [ ] I sei controlli verdi
