# 02: Editor ibrido

**What to build:** l'editor delle Note diventa come il Live Preview di Obsidian (ADR-0032, anteprima `.impeccable/mocks/note-anteprime.html`, variante A). Fuori dalla riga del cursore il Markdown si legge impaginato: titoletti, grassetto, corsivo, link, elenchi, caselle da spuntare, codice inline; sulla riga del cursore si vedono i segni e si correggono. Selezionando compare una barretta sospesa (grassetto, corsivo, link, elenco, da fare) nello stile della barretta del Turno. Il testo salvato resta il Markdown scritto, carattere per carattere.

**Blocked by:** 01

**Status:** resolved

- [x] Plugin di decorazioni nostro su `@codemirror/lang-markdown`: segni nascosti fuori dalla riga del cursore, visibili sulla riga attiva
- [x] Un clic su una casella la spunta o la toglie cambiando solo `[ ]`/`[x]` nel testo
- [x] Un link si apre nel browser di Windows e non nella WebView
- [x] Nel campo Ctrl+B, Ctrl+I e Ctrl+K formattano e non arrivano a barra laterale e ricerca; fuori dal campo restano come prima
- [x] Il pannello «Scorciatoie da tastiera» ha il gruppo Note
- [ ] Cursore, selezione e focus visibili nei due temi (verifica a mano nella finestra Windows): resta da fare nell'app, vedi `docs/sviluppo/verifica-manuale.md`; provati in un browser solo il tema chiaro, le decorazioni, la casella, il link e i tasti
- [x] I sei controlli verdi
