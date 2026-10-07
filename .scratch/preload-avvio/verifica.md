# Preload di avvio — verifica del 7 ottobre 2026

Implementata la seconda proposta approvata: logo originale, solo mozzo in
rotazione, tema e lingua salvati, messaggio aggiuntivo dopo 10 secondi,
nessuna durata minima e rispetto del movimento ridotto.

| Controllo | Esito |
| --- | --- |
| `bun run typecheck` | Passato |
| `bun run test` | 134 passati, 0 falliti |
| `bun run check` | Passato, 132 file |
| `bun run format:backend` | Passato |
| `bun run lint:backend` | Passato, tutti i target, warning negati |
| `cargo test --locked` in `src-tauri` | 305 passati, 0 falliti, 35 ignorati |
| `bun run build` | Passato |
| `cargo build --locked` in `src-tauri` | Passato |

La build frontend mantiene l'avviso già presente sul chunk principale oltre
500 KB. Il chunk iniziale dei controlli è separato, circa 18 KB; il markup,
l'animazione e il timer del preload sono nel documento iniziale.

## Prove del comportamento

`ui-probe.cjs` serve il `dist` compilato in Edge e simula soltanto l'IPC:

- Tema chiaro con Windows simulato scuro e scuro con Windows simulato chiaro.
- Stesso tema nella Home dopo la rimozione del preload, anche con Windows opposto.
- Sistema: segue il cambio della preferenza di Windows da scuro a chiaro.
- Finestra 1200 × 800 e minimo 880 × 600, anche in polacco.
- Mozzo in movimento; movimento ridotto lo ferma.
- Riduci a icona, Ingrandisci e Chiudi inviano i comandi previsti.
- Messaggio di attesa lunga dopo 10 s, senza spostare il gruppo centrale.
- Rimozione del preload e dei suoi stili prima che la Libreria risponda.
- Mancato caricamento del modulo: avviso di errore e pulsante Ricarica,
  animazione ferma.

Esito completo: `ui-report.json`. Screenshot in `.impeccable/review/startup-*.png`.

`native-probe.cjs` avvia l'eseguibile debug aggiornato con WebView2 e backend
reali. Serve gli asset compilati, ritardando soltanto il chunk React di 5 s:

- La scelta salvata è Scuro, lingua automatica risolta in italiano.
- WebView2 segnala chiaro alla media query, ma il preload usa correttamente
  colori e immagini scuri grazie alla scelta esplicita iniettata da Rust.
- Ingrandisci/Ripristina funzionano prima dell'arrivo del chunk React.
- Il preload scompare all'arrivo della Home, senza errori JavaScript.
- La Home conserva lo stesso fondo scuro: `readyDark: true` e
  `readyBackground: oklch(0.205 0.008 65)`. Il tema esplicito resta nel documento;
  non dipende dalla media query della WebView2.
- App e server di prova chiusi al termine; preferenze salvate non modificate.

Esito completo: `native-report.json`, log `native.log`. La prova nativa copre
il tema attualmente salvato; gli altri temi sono coperti dalla prova browser.
Non è una verifica dell'installer né di altri PC.

Originali in `prima/`; diff limitato all'intervento in `delta.patch`.
Nessun commit, installazione o pubblicazione.

## Revisione finale

La revisione indipendente Impeccable ha chiesto di conservare il tema anche
nella Home. Correzione verificata con nuove catture e prove browser/native:
`verdict.md` riporta `resolved`, `remaining: clear`, disposizione `ship`.
La copertura della revisione è quella del preload e del passaggio alla Home.
