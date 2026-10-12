# 13: Tasti del browser nella WebView

**What to build:** nella build di release i tasti del browser che non sono scorciatoie dell'app non agiscono più sulla WebView: Ctrl+R e F5 non ricaricano la pagina (perdendo lo stato), Alt+← e Alt+→ non navigano nella cronologia. È il seguito del menu del clic destro tolto in `feat/menu-webview` (`lib/webview.ts`, vedi `docs/sviluppo/frontend.md`).

**Status:** done

Da decidere:

- Meccanismo:
  - (a) WebView2 `AreBrowserAcceleratorKeysEnabled = false` via `with_webview` in `startup.rs`: toglie tutti gli acceleratori del browser, ma va verificato che Ctrl+C/V/X/Z/A nei campi restino;
  - (b) un filtro `keydown` nel frontend, accanto a `scorciatoia` (`features/shortcuts/shortcuts.ts`, che già ferma i Ctrl delle scorciatoie dell'app).
- Elenco dei tasti: oltre a Ctrl+R, F5, Alt+←/→, anche Ctrl+F (cerca nella pagina), Ctrl+più/meno e Ctrl+rotella (zoom)?
- In `bun tauri dev` restano attivi, come il menu (comodo ricaricare)?

- [x] Tasti scelti bloccati, anche in sviluppo, Taglia/Copia/Incolla/Annulla nei campi intatti
- [x] `docs/sviluppo/frontend.md` aggiornato (riga «Menu del clic destro»), controlli verdi

## Comments

Deciso il 12 ottobre 2026: filtro JS (`bloccaTastiDelBrowser` in `src/lib/webview.ts`), non WebView2 `AreBrowserAcceleratorKeysEnabled`, che spegnerebbe anche Ctrl+F e F12 e non copre i tasti laterali del mouse. Bloccati F5, Ctrl+R, Ctrl+Maiusc+R, Alt+←/→, i tasti media e i tasti laterali del mouse, anche in sviluppo. Ctrl+F resta. Lo zoom da tastiera era già spento (`zoom_hotkeys_enabled: false` di Tauri).
