# 05: Titolo e Cestino dall'Assistente

**What to build:** l'Assistente rinomina un Tape (`rename_tape`) e riceve il percorso nuovo relativo alla Libreria; un titolo già usato o un nome non valido danno errori distinti. Manda un Tape nel Cestino di Windows (`trash_tape`, con `destructive_hint`). Il server non tocca l'indice; una lettura di un Tape che l'indice elenca ma non c'è più dice di aprire Memotape. Vedi `../spec.md` (storie 17–22, 38).

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] `rename_tape` restituisce il percorso nuovo; il file è rinominato; titolo già usato e nome non valido danno i loro errori
- [ ] `trash_tape` toglie il file dalla Libreria (come `eliminare_un_tape_lo_manda_nel_cestino`)
- [ ] Dopo rinomina e Cestino l'indice è identico a prima
- [ ] Leggere il vecchio percorso dà l'errore che chiede di aprire Memotape
- [ ] I sei controlli verdi
