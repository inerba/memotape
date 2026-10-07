## verdict

- P1 tema scelto che si perde dopo il preload — **resolved**. Il tema esplicito ora resta su `html[data-theme]` dal bootstrap iniziale fino al contenuto React; `SettingsProvider` lo riallinea in `useLayoutEffect` e ascolta Windows solo per `Sistema`, mentre CSS e `BrandMark` usano lo stesso attributo. La prova nativa con `savedTheme: scuro` e preferenza Windows chiara ha dato `readyDark: true` e `readyBackground: oklch(0.205 0.008 65)`; le prove browser hanno coperto entrambi i temi con OS opposto e il cambio `Sistema` scuro → chiaro.
- Recapture richiesti — **passed**. Preload, messaggio lento, errore `Ricarica`, ready state e viewport 880 × 600 sono coerenti nelle immagini finali; il native probe ha anche confermato i controlli della finestra prima dell'arrivo del bundle React.

## remaining

clear

disposition: ship
