# 01: Rinomina interna del backend, il Bino diventa Tape nel Rust

**What to build:** nel Rust il documento si chiama Tape:
- i moduli `bino` e `managers::pending_bino` diventano `tape` e `managers::pending_tape`;
- tipi, funzioni, costanti, commenti e test che non escono nei bindings passano da `Bino`/`bino`/`Bini`/`bini` a `Tape`/`tape`/`tapes` (plurale `tapes` negli identificatori, ADR-0014), compresi `TAPES_PER_RICERCA`, `FRASI_PER_TAPE` e `file_to_tape`;
- la tabella dell'indice della Libreria diventa `tapes` e `SCHEMA` sale di uno, così l'indice si ricostruisce;
- il server MCP ha `list_tapes`, e descrizioni, errori e campi delle risposte strutturate dicono Tape (`bini` diventa `tapes`).

**Non** cambiano il contratto IPC (comandi, tipi esportati, eventi, codici d'errore), l'estensione `.bino`, il nome dell'app e i testi tradotti. Per l'utente l'app si comporta come prima e il frontend non si tocca.

Spec: `.scratch/rebrand-memotape/spec.md` (storie 21, 24, 25).

**Blocked by:** None (can start immediately). Prima si committa il lavoro in corso (la Continuazione) e si apre il branch `rebrand-memotape`.

**Status:** ready-for-agent

- [ ] Nel Rust nessun identificatore, commento o nome di test usa `bino`/`bini`. Fanno eccezione il contratto IPC, che passa da `bindings.ts`, e le stringhe dell'estensione `.bino`
- [ ] `tools/list` del server MCP restituisce `list_tapes`, e nessuno strumento o descrizione parla di Bini
- [ ] Un indice della versione precedente si ricostruisce (lo verifica il test che c'è già)
- [ ] `bindings.ts` non cambia: il test sui bindings resta verde senza rigenerarlo
- [ ] AGENTS.md usa i nomi nuovi dei moduli e dei tipi Rust che cita
- [ ] I sei controlli sono verdi
