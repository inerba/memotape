# 06: Libreria, menu contestuale e accessibilità

**What to build:** menu contestuale, tabella da tastiera, nomi accessibili. Regole in `../spec.md`, «6 · Libreria e menu contestuale».

**Blocked by:** 05.

**Status:** done

- [x] Un menu condiviso (clic destro, tasto Menu, pulsante «…») sulle righe della Libreria e sui Recenti della barra laterale, con le azioni di `useTapeOperations`; Copia testo con `tape_text`.
- [x] Tabella con un solo elemento nel Tab e le frecce; Invio, F2, Canc (con la conferma di oggi).
- [x] Colonna del titolo a larghezza piena.
- [x] «Elimina {{titolo}}» e verso dell'ordinamento per colonna, nelle sei lingue.
- [x] Documenti aggiornati, i sei controlli passano.

## Comments

### 2026-10-10

- Logica pura test-first in `library.ts` (`rigaDopo`, `azioniDelTape`, `versoDellOrdine`) e in `shortcuts.ts` (`tastoDellaTabella`): Invio, F2 e Canc stanno nella mappa delle scorciatoie con `nellaTabella`, così il pannello li elenca (gruppo Libreria) e il menu mostra i loro nomi; li gestisce l'elemento con il focus, non il listener della finestra.
- Il menu è `TapeContextMenu` (`tape-context-menu.tsx`) su `PopoverMenu`, esteso con `lazy`, `tabIndex` e `openMenuAt`: il clic destro (e il tasto Menu, che arriva anche lui come `contextmenu`) apre il menu nel punto, tramite un'ancora invisibile. Sposta in ▸ è un popover annidato.
- Rinomina apre il campo sul posto, nella riga o nel Recente. Nei Recenti valgono F2 e Canc (Invio è già il clic).
- Deviazioni dall'anteprima: nella riga restano il Cestino («Elimina {{titolo}}», come chiede la spec) e «…», non la cartella di Sposta in…, che sta nel menu; `MoveSelect` è stato tolto. Pagina su/giù salta di 10 righe fisse. La data della tabella resta nel formato medio di oggi.
- Il Tape bloccato è solo quello di Trascrivi su un Tape (`lavorato`), come fa il backend con `Activity::write`.
- Dalla prova nell'app: tabella con bordi separati (con quelli uniti Chromium non disegna il contorno della riga col focus); sotto i 672 px di tabella (finestra a 880 px) la colonna Raccolta si nasconde, se no il titolo spariva; chiusa la conferma del Cestino il focus torna dov'era (riga o «…»).
