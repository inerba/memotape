# 05: Scorciatoie da tastiera

**What to build:** le scorciatoie della tabella in `../spec.md`, «5 · Scorciatoie», i tooltip e il pannello.

**Blocked by:** 04.

**Status:** done

- [x] Una mappa pura delle scorciatoie (azione, tasti, abilitata per stato), con test; un solo listener globale.
- [x] Ctrl+N, Ctrl+O, Ctrl+P, Ctrl+Maiusc+C non arrivano a WebView2: verificare nell'app che non aprano finestre, la stampa o l'ispettore.
- [x] Frecce ±5 s, `[` `]` velocità, Ctrl+J ritorno, solo fuori dai campi; Ctrl+B senza effetto finché non c'è il ticket 07.
- [x] Tooltip con i tasti sui pulsanti interessati; pannello con Ctrl+/ e collegamento in Impostazioni → Generale.
- [x] Sei lingue, documenti aggiornati, i sei controlli passano.

## Comments

### 2026-10-10

- Mappa pura in `features/shortcuts/shortcuts.ts` (13 test), listener unico e pannello `<dialog>` in `shortcuts-provider.tsx`; le azioni le collega il componente del pulsante con `useScorciatoia`, finché il pulsante è attivo.
- Tutte le combinazioni con Ctrl fanno sempre `preventDefault`, non solo le quattro della spec: è una regola sola e nessuna delle altre (K, B, comma, J, /) serve alla WebView2. Basta la pagina, niente lato Rust: Tauri 2.12 non espone `AreBrowserAcceleratorKeysEnabled`, che spegnerebbe anche F12.
- Verificato con tasti veri (SendInput) sull'app in dev: senza il listener Ctrl+P apre la stampa e Ctrl+Maiusc+C l'ispettore; con il `preventDefault` della pagina nessuno dei due, né finestre nuove con Ctrl+N o Ctrl+O (che già senza listener non aprivano nulla). Ctrl+P è stato provato con il listener vero fuori da una Registrazione; Ctrl+N, Ctrl+O e Ctrl+Maiusc+C con uno stub che fa lo stesso `preventDefault`, per non avviare una Registrazione né scrivere negli appunti dell'utente.
- Via CDP: pannello da Ctrl+/ e da Impostazioni → Generale nei due temi, Esc lo chiude, Ctrl+N in Impostazioni non fa nulla, `]` `[` e → ← sul player, tooltip e `aria-keyshortcuts`.
- Deviazioni: tooltip con il `title` nativo («Copia tutto il testo (Ctrl+Maiusc+C)»), non disegnati come nell'anteprima; il pannello non elenca Esc dell'editor (non è nella tabella della spec) né Ctrl+B finché non c'è il ticket 07. `[` e `]` valgono anche con AltGr (tastiera italiana); Ctrl+/ anche con Maiusc.
- Non verificati: Ctrl+P durante una Registrazione vera, Ctrl+J con tasti veri, la tastiera italiana fisica per `[`, `]` e Ctrl+/ (provati solo con eventi sintetici e nei test), 880 px.
