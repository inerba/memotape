# 03: Un'Attività blocca le scritture dell'Assistente su quel Tape

**What to build:** mentre l'app registra, trascrive o riconosce i parlanti su un Tape, l'Assistente non può scriverlo: lo strumento risponde che c'è un'Attività in corso. L'app tiene un lock per Tape (file in `.memotape`) per tutta l'Attività e lo rilascia anche con errore, Annulla o chiusura dell'app. Le Attività su altri Tape non bloccano l'Assistente. Vedi `../spec.md` (storie 30–32, decisione «Lock dell'Attività»).

**Blocked by:** 02

**Status:** resolved

- [x] Con il lock dell'Attività preso dal test come lo prende l'app, gli strumenti che scrivono rifiutano con l'errore di Attività in corso e il Tape resta identico
- [x] Un lock su un altro Tape non blocca la scrittura
- [x] Il lock si rilascia a fine Attività, anche con errore o Annulla
- [x] I sei controlli verdi
