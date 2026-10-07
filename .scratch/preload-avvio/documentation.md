# Preload di avvio — documentazione finale del 7 ottobre 2026

## Perimetro

Ho riesaminato il contratto approvato in `spec.md`, i requisiti in `PRODUCT.md`,
la guida visiva in `DESIGN.md`, la guida del marchio e il sidecar incumbent
`.impeccable/design.json`, insieme al codice shipped del preload e ai report già
prodotti. La superficie resta quella concordata: marchio originale, solo il
mozzo in rotazione costante di 2,8 s, nessuna barretta o percentuale, messaggio
di attesa dopo 10 s, movimento ridotto rispettato, controlli della finestra
disponibili e rimozione al primo contenuto reale.

## Persistenza del tema

Il finding P1 della revisione precedente è risolto.

- `startup::create_main` legge le impostazioni prima di creare la WebView,
  passa la scelta in `window.__MEMOTAPE_STARTUP__.theme`, applica `Tema::apply`
  alla finestra nativa e imposta il fondo prima di mostrarla.
- `index.html` imposta `html[data-theme]` prima della prima pittura. Il preload
  usa la scelta esplicita per carta, inchiostro e varianti del marchio; il suo
  `MutationObserver` rimuove solo `data-startup-theme`, lasciando `data-theme`
  attivo quando arriva il primo contenuto React.
- `SettingsProvider` riallinea lo stesso attributo in `useLayoutEffect` e ascolta
  `prefers-color-scheme` soltanto quando l'impostazione è `Sistema`.
- `global.css` usa `:root[data-theme="dark"]` per i token e per la variante
  `dark:`; `BrandMark` seleziona le SVG chiare/scure con quella stessa variante.

## Evidenze controllate

`native-report.json` mostra `savedTheme: "scuro"`, Windows chiaro,
`readyDark: true` e il fondo scuro atteso dopo il montaggio React.
`ui-report.json` copre chiaro e scuro con preferenza Windows opposta, `Sistema`
con cambio scuro → chiaro, il minimo 880 × 600, italiano/polacco, movimento
ridotto, messaggio lento senza salto, errore con `Ricarica` e controlli della
finestra. I capture `startup-native.png`, `startup-native-ready.png`, le
varianti `startup-ready-*`, `startup-slow-*` e `startup-error-1200.png` sono
presenti e coerenti con questi esiti. `verifica.md` registra i sei controlli
standard verdi, la build frontend e la build Rust; il limite dichiarato resta
il mancato collaudo dell'installer e di altre macchine.

## Drift deliberatamente non ampliato

Non ho rigenerato `.impeccable/design.json`: il sidecar incumbent resta la
fonte già approvata per i token e le componenti; questa passata aggiorna solo
la frase locale di `DESIGN.md` che descriveva erroneamente il tema come gestito
unicamente dalla media query. Nessun colore, token, regola di brand o codice di
prodotto è stato modificato.

Esito della documentazione: tema persistente verificato, restante perimetro
chiaro, nessun finding visivo aggiuntivo canonizzato.
