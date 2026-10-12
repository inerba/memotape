# 13: Tasti del browser nella WebView

**What to build:** nella build di release i tasti del browser che non sono scorciatoie dell'app non agiscono più sulla WebView: Ctrl+R e F5 non ricaricano la pagina (perdendo lo stato), Alt+← e Alt+→ non navigano nella cronologia. È il seguito del menu del clic destro tolto in `feat/menu-webview` (`lib/context-menu.ts`, vedi `docs/sviluppo/frontend.md`).

**Status:** needs-triage

Da decidere:

- Meccanismo:
  - (a) WebView2 `AreBrowserAcceleratorKeysEnabled = false` via `with_webview` in `startup.rs`: toglie tutti gli acceleratori del browser, ma va verificato che Ctrl+C/V/X/Z/A nei campi restino;
  - (b) un filtro `keydown` nel frontend, accanto a `scorciatoia` (`features/shortcuts/shortcuts.ts`, che già ferma i Ctrl delle scorciatoie dell'app).
- Elenco dei tasti: oltre a Ctrl+R, F5, Alt+←/→, anche Ctrl+F (cerca nella pagina), Ctrl+più/meno e Ctrl+rotella (zoom)?
- In `bun tauri dev` restano attivi, come il menu (comodo ricaricare)?

- [ ] Tasti scelti bloccati nella build di release, Taglia/Copia/Incolla/Annulla nei campi intatti
- [ ] `docs/sviluppo/frontend.md` aggiornato (riga «Menu del clic destro»), controlli verdi
