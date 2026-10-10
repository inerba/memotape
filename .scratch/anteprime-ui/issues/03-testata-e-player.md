# 03: Testata e player

**What to build:** titolo a 32 px, popover Dettagli, player su una riga. Regole in `../spec.md`, «3 · Testata e player».

**Blocked by:** 02.

**Status:** done

- [x] `tape-header.tsx`: titolo e campo di rinomina a 2rem, `text-wrap: pretty`.
- [x] Chip «Dettagli» con popover (come gli altri popover nativi di `popover-menu.tsx`): origine, modello, Lingua del parlato, Ingressi. Avvisi, Corretto a mano e Parlanti restano visibili.
- [x] `player.tsx`: una riga, senza nome del file, volume a comparsa, Segui l'audio come pillola; tolto dalla barra delle schede.
- [x] DESIGN.md (Display, Player, barra delle schede), i sei controlli passano.

## Comments

### 2026-10-10

- Le righe di Dettagli vengono da `tapeDetails` (pura, in `library.ts`, test in `library.test.ts`); senza testo niente modello né lingua. Ingressi dice «Mix» o «Microfono + audio di sistema». La riga «Stato» dell'anteprima non c'è: la spec non la chiede e gli avvisi restano visibili.
- «fuori dalla Libreria» resta visibile come etichetta (la spec non lo nomina).
- `PopoverMenu` accetta `size` per un pulsante con il testo (Dettagli). Il volume è un `PopoverMenu` con Silenzia e il cursore; Segui l'audio è una pillola `aria-pressed` («Segui», nome intero nel tooltip e per i lettori di schermo), visibile anche nella scheda Parlanti.
- Il titolo del Tape resta a 2rem anche nelle finestre basse; il display del `DocumentHeader` (Libreria, dal vivo, file) resta a 2.75rem. Play da 44 a 40 px.
- Tolte le chiavi `tape.file`, `player.file` e `player.recording`, non più usate.
