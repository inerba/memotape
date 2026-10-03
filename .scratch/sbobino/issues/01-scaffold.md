# 01: Scaffold

**What to build:** il progetto Sbobino nasce dal template MrLightful/create-tauri-react nella root del repo, con una storia git pulita e lo stack completo della spec (sezione "Versioni e configurazione"). L'avvio con `bun tauri dev` mostra una home minimale, che chiama un comando Tauri tipizzato da `bindings.ts` e ne visualizza il risultato. I file già presenti (`CONTEXT.md`, `docs/`, `.scratch/`) restano.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `git init` con un primo commit pulito, senza la storia del template, e commit con prefissi convenzionali
- [x] Stack alle versioni della spec:
  - `tauri` e `@tauri-apps/*` 2.12.x;
  - specta e tauri-specta pinnati a rc.25 con `=`;
  - React 19, TypeScript 6.0.3, Vite 8, react-router 7.18.x;
  - shadcn new-york con `components.json` scritto a mano e il pacchetto `cn`;
  - Tailwind 4 con `tw-animate-css` importato;
  - Inter Variable, i18next con il solo italiano;
  - ultracite/Biome con `includes` corretti.
- [x] Il `bun.lock` è rigenerato
- [x] Struttura del backend con `commands/`, `managers/`, `audio_toolkit/` e `engine/`, e frontend bulletproof-react con l'alias `@/`
- [x] `bindings.ts` generato in debug, e la home chiama un comando tipizzato
- [x] Identifier Tauri `sbobino`
- [x] `PRODUCT.md` riporta i requisiti di prodotto della spec e da qui in poi è la fonte di verità
- [x] `AGENTS.md` contiene:
  - comandi e prerequisiti di build;
  - architettura;
  - tutte le Insidie, con le correzioni della spec ("Further Notes");
  - il blocco "Agent skills" spostato da `CLAUDE.md`.
- [x] `CLAUDE.md` contiene solo `@AGENTS.md`, e `README.md` è per gli sviluppatori
- [x] Husky + lint-staged eseguono `bunx ultracite fix` sui file staged
- [x] Passano `typecheck`, `test` (con almeno un test reale), `check`, `format:backend`, `lint:backend` e `cargo test`

## Comments

- Il repo aveva già il commit `bed1f7d inizio` (CLAUDE.md e `docs/agents/`): la storia parte da lì, senza quella del template. Quel commit non ha un prefisso convenzionale e non è stato riscritto.
- Biome: la spec chiede `includes` `["**", "!**/components/ui", "!**/bindings.ts"]`, ma con ultracite 7.12 il `"**"` è già nel core esteso e ripeterlo dà `noBiomeFirstException` (e `ultracite fix` lo toglie). `biome.jsonc` tiene solo le negazioni; verificato che `src/` è controllato e che `components/ui` e `bindings.ts` sono esclusi. Insidia aggiornata in `AGENTS.md`.
- `cargo test` richiede il manifest Common Controls v6 incorporato anche nei test (per `tauri-plugin-dialog`): lo fa `build.rs`.
- Le Insidie del brief non sono nel repo: `AGENTS.md` raccoglie quelle della spec e delle ricerche in `docs/research/`.
