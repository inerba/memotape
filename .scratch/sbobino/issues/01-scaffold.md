# 01: Scaffold

**What to build:** il progetto Sbobino nasce dal template MrLightful/create-tauri-react nella root del repo, con una storia git pulita e lo stack completo della spec (sezione "Versioni e configurazione"). L'avvio con `bun tauri dev` mostra una home minimale, che chiama un comando Tauri tipizzato da `bindings.ts` e ne visualizza il risultato. I file già presenti (`CONTEXT.md`, `docs/`, `.scratch/`) restano.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `git init` con un primo commit pulito, senza la storia del template, e commit con prefissi convenzionali
- [ ] Stack alle versioni della spec:
  - `tauri` e `@tauri-apps/*` 2.12.x;
  - specta e tauri-specta pinnati a rc.25 con `=`;
  - React 19, TypeScript 6.0.3, Vite 8, react-router 7.18.x;
  - shadcn new-york con `components.json` scritto a mano e il pacchetto `cn`;
  - Tailwind 4 con `tw-animate-css` importato;
  - Inter Variable, i18next con il solo italiano;
  - ultracite/Biome con `includes` corretti.
- [ ] Il `bun.lock` è rigenerato
- [ ] Struttura del backend con `commands/`, `managers/`, `audio_toolkit/` e `engine/`, e frontend bulletproof-react con l'alias `@/`
- [ ] `bindings.ts` generato in debug, e la home chiama un comando tipizzato
- [ ] Identifier Tauri `sbobino`
- [ ] `PRODUCT.md` riporta i requisiti di prodotto della spec e da qui in poi è la fonte di verità
- [ ] `AGENTS.md` contiene:
  - comandi e prerequisiti di build;
  - architettura;
  - tutte le Insidie, con le correzioni della spec ("Further Notes");
  - il blocco "Agent skills" spostato da `CLAUDE.md`.
- [ ] `CLAUDE.md` contiene solo `@AGENTS.md`, e `README.md` è per gli sviluppatori
- [ ] Husky + lint-staged eseguono `bunx ultracite fix` sui file staged
- [ ] Passano `typecheck`, `test` (con almeno un test reale), `check`, `format:backend`, `lint:backend` e `cargo test`
