# 03: Più Note con nome

**What to build:** un Tape ha più Note, ognuna con un nome unico (anteprima `.impeccable/mocks/note-multiple.html`, variante A, riga di nomi). Sotto le schede una riga con i nomi in ordine di creazione, la Nota aperta con il fondo e in fondo «+ Nuova nota», che crea «Nota 2», «Nota 3»… con il nome subito in modifica. Doppio clic sul nome o F2 rinomina sul posto; il «…» ha Rinomina, Copia nota ed Elimina nota… con una conferma che dice che non si recupera. Una Nota nuova lasciata vuota sparisce uscendo dal Tape. La scheda mostra il numero di Note.

**Blocked by:** 01

**Status:** resolved

- [x] Creare, rinominare ed eliminare una Nota riscrivono solo `note.json`; l'ordine è quello di creazione
- [x] Nome vuoto, oltre 60 caratteri o già usato nel Tape rifiutato con un codice di `AppError` dedicato e un messaggio tradotto
- [x] Logica pura dei nomi predefiniti e unici in un modulo del frontend con il suo `*.test.ts`
- [x] La riga dei nomi è un elenco di schede per i lettori di schermo; il «…» si chiama «Altre azioni per {{nome}}»
- [x] Copia nota copia il Markdown della Nota e lo dice in un avviso
- [x] Testi nelle sei lingue
- [x] I sei controlli verdi
