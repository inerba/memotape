# 07: Barra laterale richiudibile

**What to build:** la barra laterale che si chiude in una striscia di icone. Regole in `../spec.md`, «7 · Barra laterale richiudibile».

**Blocked by:** 06.

**Status:** done

- [x] Pulsante nella riga del titolo e Ctrl+B; stato in `localStorage` (con try/catch), parte aperta.
- [x] Striscia da 60 px con tooltip; Cerca e Recenti riaprono la barra (Cerca con il focus nel campo), le altre icone agiscono subito; pallino dell'Attività sui Recenti.
- [x] DESIGN.md (Layout, Navigation), i sei controlli passano; prova a 880 px.

## Comments

### 2026-10-10

- «Parte sempre aperta» letto come: aperta al primo avvio (o senza memoria del browser), poi come l'ha lasciata l'utente (`memotape.barraLaterale`). Nessuna chiusura automatica, nemmeno sotto i 1000 px (il «Da decidere» delle anteprime resta escluso).
- Il pulsante della barra è uno solo, assoluto in alto a sinistra del pannello centrale (`SidebarToggle`), sopra la riga del titolo di ogni vista, come i pulsanti di Windows a destra: le righe lasciano `SIDEBAR_TOGGLE_PADDING` e restano trascinabili intorno.
- Ctrl+N passa da `RecordMenu` a `Sidebar`, che resta montata anche chiusa: così Ctrl+N vale con la striscia. L'icona Nuova registrazione registra subito con le scelte salvate, senza il menu ▾. Anche Ctrl+K riapre la barra, come l'icona Cerca; Recenti porta il focus sul Tape aperto (o sul primo Recente).
- Pallino dei Recenti: salvia, mattone se `liveError` (Trascrizione dal vivo fermata, `activitySummary(...).guasta`), come dice la spec; una Registrazione sana resta salvia, a differenza del pallino mattone della scheda Attività.
- Provato nell'app a 880 e 1200 px nei due temi (tema forzato solo nel DOM). Non provati: il pallino con un'Attività vera, il trascinamento reale della finestra intorno al pulsante. Durante la prova nella finestra è comparsa una Registrazione di 4 s (`Registrazione 2026-10-10 16-15-04.tape`) avviata e fermata a mano, non dai passi CDP: è rimasta nella Libreria.
