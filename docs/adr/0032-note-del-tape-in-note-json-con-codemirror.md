---
status: accepted
---

# Le Note stanno in `note.json` dentro il Tape e si scrivono come Markdown grezzo con CodeMirror 6

Data: 11 ottobre 2026. Termini in `CONTEXT.md` (Nota, Tempo nella Nota).

Un Tape ha zero o più Note con un nome, scritte dall'utente o da un Assistente (ADR-0031). Le Note di un Tape stanno in una sola voce `note.json` dello zip: un elenco ordinato con nome, testo Markdown così com'è, autore dell'ultima scrittura (l'utente o un Assistente con il nome del client MCP) e ora. Una voce sola fa di ogni salvataggio una sola sostituzione atomica con `replace`, sotto lo stesso lock delle altre riscritture, e lascia `trascrizione.json` e la sua `version` come sono: le versioni vecchie dell'app copiano la voce senza leggerla. Un Taglio la riscrive (sposta i Tempi nella Nota) insieme all'audio e al documento.

L'editor è CodeMirror 6 con `@codemirror/lang-markdown` e un nostro plugin di decorazioni che nasconde i segni del Markdown fuori dalla riga del cursore, come il Live Preview di Obsidian. CodeMirror modifica il testo e non lo ricostruisce, quindi il Markdown di un Assistente resta identico carattere per carattere quando l'utente corregge una parola.

## Alternative scartate

- Un file `.md` per Nota nello zip (`note/<id>.md`) con l'indice in `trascrizione.json`: due voci da sostituire insieme a ogni salvataggio e una `version` da alzare.
- Un editor visuale (Tiptap con `@tiptap/markdown`, Milkdown): più facile per chi non conosce il Markdown, ma l'andata e ritorno tra Markdown e documento riscrive il testo dell'Assistente (stelle, spazi, numeri degli elenchi) ed è il più pesante.
- Un `<textarea>` con una vista impaginata a parte («Leggi, poi Modifica»): il più leggero, ma un clic in più per correggere e un modo di scrivere diverso dall'editor del Turno, che non ha modalità.
- OverType (textarea con lo stile sovrapposto): leggera, ma lascia sempre visibili i segni del Markdown.
- `codemirror-markdown-hybrid`: fa già il Live Preview, ma è piccola e ferma da gennaio 2026; il plugin nostro è poco codice e resta sotto controllo.

## Conseguenze

- Durante la Registrazione il Tape non esiste: le Note stanno in una bozza nella cartella temporanea degli Ogg e allo Stop entrano nel Tape; senza Tape diventano file `.md` accanto all'Ogg.
- L'indice della Libreria aggiunge il testo delle Note alla ricerca e lo rilegge nel riallineamento.
