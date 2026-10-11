# 04: Unione di Turni, nomi dei Parlanti e data dall'Assistente

**What to build:** l'Assistente unisce un Turno a quello adiacente (`merge_turn`), dà un nome a un Parlante (`rename_speaker`) e cambia data e ora del Tape (`set_date`), con le stesse regole dell'app. Vedi `../spec.md` (storie 14–16).

**Blocked by:** 02

**Status:** resolved

- [x] Ognuno dei tre strumenti scrive quello che dice, verificato rileggendo il Tape
- [x] Richieste non valide o obsolete (Turno non adiacente, Ingressi diversi, data non valida) rifiutate senza scrivere
- [x] Con il permesso a modificare spento rifiutano
- [x] I sei controlli verdi
