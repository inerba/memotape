# 08: Ricerca ed esportazione

**What to build:** le Note si trovano e si portano fuori. La ricerca della Libreria trova le parole delle Note; il risultato dice la Nota («Riepilogo · …preventivo aggiornato…») e apre il Tape sulla scheda Note, su quella Nota. Esporta Markdown… mette ogni Nota come «## nome», in ordine, prima della Trascrizione; Copia testo resta la sola Trascrizione.

**Blocked by:** 03

**Status:** resolved

- [x] L'indice ha le righe delle Note (una per Nota, con il nome) e la versione dello schema alzata, così si ricostruisce; il riallineamento rilegge `note.json`
- [x] Test Rust: una parola di una Nota si trova e il risultato porta il nome della Nota; l'export Markdown ha le Note prima della Trascrizione
- [x] Un risultato in una Nota apre quella Nota
- [x] `docs/sviluppo/tape-e-libreria.md` aggiornato
- [x] I sei controlli verdi
