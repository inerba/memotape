# 07: Silenzi e Taglio dall'Assistente

**What to build:** l'Assistente chiede i Silenzi di un Tape (`find_silences`) e riceve per ciascuno inizio, fine, posizione (inizio, in mezzo, fine) e il Taglio che l'app proporrebbe, con la stessa regola della modalità Taglio portata in Rust (almeno 5 s di silenzio stimato fra le Frasi di tutti gli Ingressi; tutto all'inizio e alla fine; 1 s per lato in mezzo, a metà se corto). Poi applica i Tagli in una sola operazione (`cut_tape`, `destructive_hint`): l'originale va sempre nel Cestino, e su un volume senza Cestino rifiuta. Vedi `../spec.md` (storie 23–29) e ADR-0030.

**Blocked by:** 02

**Status:** resolved

- [x] `find_silences` sugli stessi casi di `taglio.test.ts`: inizio, fine, in mezzo sotto e sopra i 5 s, Ingressi che si accavallano; un Tape senza Frasi non ha Silenzi
- [x] `cut_tape` con i Tagli proposti accorcia il Tape e sposta le Frasi; l'originale è nel Cestino
- [x] Su `noRecycleBin` rifiuta e il Tape resta com'è
- [x] Con il permesso a modificare spento rifiutano
- [x] I sei controlli verdi
