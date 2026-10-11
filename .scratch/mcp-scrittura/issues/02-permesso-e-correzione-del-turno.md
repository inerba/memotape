# 02: Permesso a modificare, lettura dei Turni e correzione del Turno dall'Assistente

**What to build:** la prima fetta completa. In Impostazioni, sezione Assistenti, c'è il secondo interruttore «Consenti agli Assistenti di modificare la Libreria» (spento di default, disabilitato con la lettura spenta, testi nelle sei lingue). L'Assistente legge i Turni di un Tape a pagine (`read_turns`) e corregge un Turno con il testo atteso (`edit_turn`); la correzione è una Correzione manuale. Il server rilegge il permesso a ogni chiamata e non tocca mai l'indice. Vedi `../spec.md` (storie 1–13, 39, 45) e ADR-0031.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Nuovo booleano delle Impostazioni, falso di default e quando manca nel file (test in `settings.rs` e `settings.test.ts`)
- [ ] Con lettura spenta o modifica spenta `edit_turn` rifiuta e il Tape resta identico byte per byte
- [ ] `read_turns` restituisce per ogni Turno Ingresso, id delle Frasi, Parlante, inizio, fine e testo come l'editor del Turno, a pagine
- [ ] `read_turns` → `edit_turn` corregge; una seconda correzione con il testo atteso vecchio è rifiutata senza scrivere; il Tape risulta corretto a mano
- [ ] Percorsi assoluti, con `..` o fuori dalla Libreria rifiutati anche in scrittura
- [ ] Dopo la scrittura l'indice è identico
- [ ] `docs/sviluppo/impostazioni-e-servizi.md` e `PRODUCT.md` aggiornati
- [ ] I sei controlli verdi
