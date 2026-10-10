# 02: Recenti leggibili

**What to build:** Recenti con giorno abbreviato, durata a parole e limite. Regole in `../spec.md`, «2 · Recenti».

**Blocked by:** 01.

**Status:** done

- [x] Funzioni pure (con test): durata a parole («24 s», «59 min», «2 h 05 min») e data breve per gruppo («gio 8, 17:32», «31 lug, 11:10»), con `Intl` nella lingua dell'interfaccia.
- [x] Barra laterale: al massimo 10 Tape e «Tutti i N Tape nella Libreria» (chiave plurale nelle sei lingue); nessun limite durante la ricerca.
- [x] Recenti della Home con la stessa durata a parole; Libreria e player invariati.
- [x] Documenti aggiornati, i sei controlli passano.

## Comments

### 2026-10-10

- In `library.ts`: `durationWords` (unità brevi di `Intl.NumberFormat`, arrotondata: 59,6 s è «1 min», 59 min 40 s è «1 h 00 min»), `recentWhen` (l'ora in Oggi e Ieri, `weekday`+`day` nella settimana, `day`+`month` nei mesi, sempre con `clockText`) e `groupByDate(tapes, today, limit)` con `RECENTI_MAX` = 10. Test in `library.test.ts`, in italiano.
- Nelle altre lingue l'ordine e le unità sono quelli di `Intl` («8 Thu», «2 Std. 05 Min.», «2 godz. 05 min»).
- «Tutti i N Tape nella Libreria» compare solo con più di 10 Tape e apre la Libreria (`onShowAll`). Un Tape aperto oltre i 10 non è evidenziato nei Recenti.
- Home: cambia solo la durata dei tre Recenti (orologio da 14 px); la carta Riprendi non è tra i Recenti e resta in cifre.
- Non provato nell'app (CDP), né nei due temi né a 880 px.
