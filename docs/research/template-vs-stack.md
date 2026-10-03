# Template `create-tauri-react` a confronto con lo stack di Sbobino

Data della ricerca: 2026-10-02. Documento di ricerca: non contiene decisioni. Le proposte vanno approvate dall'utente.

Legenda: **[V]** = fatto verificato su una fonte primaria (file, comando, URL); **[I]** = inferenza non verificata.
I riferimenti `tpl/<file>:<riga>` puntano al clone del template in
`C:\Users\inerba\AppData\Local\Temp\claude\D--local-tauri-sbobino\c0b474c7-3cd4-4e08-86e1-1e8b858329ee\scratchpad\create-tauri-react`.

---

## 1. Commit analizzato e inventario

- Repo: https://github.com/MrLightful/create-tauri-react
- Commit: **`81c7247a872854c327a4e5591315cd6a3da97c9b`** (2026-10-02, "Bump the dependencies group in /src-tauri with 4 updates (#87)") [V] (`git log -1`)

### Cosa contiene [V] (`git ls-files`)

| Area | Contenuto |
|---|---|
| Frontend | `src/main.tsx`, `src/app/{index,provider,router}.tsx`, `src/app/routes/{home,not-found}.tsx`, `src/app/global.css`, `src/components/ui/{button,tooltip}.tsx`, `src/features/{built-with,errors,github-star-button}/`, `src/lib/{utils,create-env}.ts`, `src/config/env.ts`, `src/vite-env.d.ts` |
| Config frontend | `package.json`, `bun.lock`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `biome.jsonc`, `components.json`, `index.html`, `.env.example` |
| Git hooks | `.husky/pre-commit` (`bunx lint-staged`), config `lint-staged` dentro `package.json` |
| Backend | `src-tauri/Cargo.toml`, `Cargo.lock`, `build.rs`, `tauri.conf.json`, `capabilities/migrated.json`, `src/main.rs` (un solo file, comando `greet`) |
| CI | `.github/dependabot.yml` (npm + cargo, mensile) |

Nessun test, nessun `lib.rs`, nessuna cartella `commands/`, `managers/`, `audio_toolkit/` o `engine/`, nessun `tauri-specta`. [V]

### Problemi strutturali trovati nel template

1. **`bun.lock` non è allineato con `package.json`.** [V] L'ultima modifica a `bun.lock` è del 2026-04-01 (commit `b9f688b`, "Migrate vite v7 to v8"). Dependabot ha poi aggiornato solo `package.json`. Le differenze:
   - `react-router`: `^8.3.0` in `tpl/package.json:30` contro `7.13.2` risolto nel lock (`tpl/bun.lock:374`, workspace `^7.10.1`)
   - `@biomejs/biome`: `2.5.11` (`tpl/package.json:36`) contro `2.4.9` (`tpl/bun.lock:44`)
   - `ultracite`: `7.10.7` (`tpl/package.json:47`) contro `7.4.2` (`tpl/bun.lock:422`)
   - `lint-staged`: `^17.0.7` (`tpl/package.json:43`) contro `16.4.0` (`tpl/bun.lock:330`)

   [I] Un `bun install --frozen-lockfile` probabilmente fallirebbe. Conviene rigenerare il lock.
2. **`biome.jsonc` non controlla nessun file di `src/`.** `tpl/biome.jsonc:5-9` ha `"includes": ["!**/components/ui"]`, cioè solo una negazione senza un `"**"` iniziale. Test riprodotto in scratchpad con Biome 2.5.15 [V]:
   - con `["!**/components/ui"]`: `Checked 1 file`, cioè solo `biome.json`, più l'errore `lint/suspicious/noBiomeFirstException`. Il file `src/x.ts` (che contiene `debugger`) **non viene controllato**.
   - con `["**", "!**/components/ui", "!**/bindings.ts"]`: `src/x.ts` viene segnalato e i file esclusi vengono saltati.
   - La documentazione di ultracite lo conferma: «It is critical to include "**" as the first entry when using negated patterns» (https://github.com/haydenbleasel/ultracite/blob/main/apps/docs/docs/troubleshooting.mdx).
3. **`tw-animate-css` è installato ma non importato.** `tpl/src/app/global.css:1` contiene solo `@import "tailwindcss";`. Eppure `tooltip.tsx:45` usa `animate-in` e `fade-in-0`. Risulta installato anche `tailwindcss-animate` (`tpl/package.json:32`), che è il plugin di Tailwind v3 e qui non è usato. [V] su file. [I] Le animazioni dei componenti shadcn quindi non funzionano.
4. Il backend ha una struttura Tauri 1-style: un solo `main.rs`, `edition = "2021"` (`tpl/src-tauri/Cargo.toml:6`), la capability `migrated.json` che dice di essere «migrated from v1» (`tpl/src-tauri/capabilities/migrated.json:3`), e `identifier: "com.tauri.app"` (`tpl/src-tauri/tauri.conf.json:21`). [V]
5. I plugin usati sono `shell` e `process` (`tpl/src-tauri/src/main.rs:12-13`). Lo stack vuole invece `log` e `opener`. `plugin-process` serve solo per `relaunch()` in `tpl/src/features/errors/app-error.tsx:1,20`. [V]

---

## 2. Confronto voce per voce

| Voce dello stack | Stato nel template (versione) | Azione proposta | Fonte |
|---|---|---|---|
| Rust stable MSVC | non specificato (nessun `rust-toolchain.toml`) | aggiungere `rust-toolchain.toml` (opzionale) | `git ls-files`; stable = 1.99.0 da https://static.rust-lang.org/dist/channel-rust-stable.toml; rustc locale 1.96.1 |
| Crate `tauri` | `2.11.2` (`Cargo.toml:14`), lock `2.12.0` (`Cargo.lock:3086`) | aggiornare a 2.12.1 | crates.io API `tauri` |
| `tauri-build` | `2.0.3` (`Cargo.toml:11`), lock `2.7.1` | aggiornare a 2.7.1 | crates.io API |
| `tauri-plugin-log` | assente | aggiungere | crates.io / npm |
| `tauri-plugin-opener` | assente | aggiungere | crates.io / npm |
| `tauri-plugin-shell`, `tauri-plugin-process` | presenti (`Cargo.toml:17-18`, `package.json:21-22`) | togliere, oppure tenere `process` per `relaunch` (da decidere) | `main.rs:12-13`, `app-error.tsx:1` |
| `tauri-specta` + `specta` + `specta-typescript` | assenti | aggiungere, pinnati con `=` | §3 |
| Struttura `src-tauri/src/{commands,managers,audio_toolkit,engine}` | solo `main.rs` | aggiungere | `git ls-files` |
| `bindings.ts` generato in debug | assente | aggiungere | tauri-specta `src/lib.rs:48-56` |
| React 19 | `^19.2.1` (lock 19.2.4) | aggiornare a 19.3.0 | npm |
| TypeScript strict | `^6.0.2`, `strict: true` (`tsconfig.json:18`) | tenere 6.x o passare a 7 (vedi §4) | npm |
| Vite 8 | `^8.0.0` (lock 8.0.3) | aggiornare a 8.3.2 | npm |
| Alias `@/` → `src/` | presente (`vite.config.ts:25-29`, `tsconfig.json:24-26`) | tenere | file |
| bulletproof-react (`app/`, `features/`, `components/`, `lib/`) | presente (più `config/`) | tenere; ripulire le feature demo (`built-with`, `github-star-button`) | `git ls-files`, README |
| Test `*.test.ts` con `bun test` | assenti, nessuno script `test` | aggiungere | `package.json:6-15` |
| `react-router` 7 | `^8.3.0` in package.json, 7.13.2 nel lock | **decidere**: 7.18.4 (tag `version-7`) oppure adottare la 8 | npm dist-tags; changelog v8 |
| `react-hook-form` | `^7.68.0` (lock 7.72.0) | aggiornare | npm |
| `@hookform/resolvers` | assente | aggiungere (serve per collegare zod a RHF) | npm peer `zod ^3.25 \|\| ^4` |
| `zod` 4 | `^4.1.13` (lock 4.3.6) | aggiornare | npm |
| `react-error-boundary` | `^6.0.0` (lock 6.1.1) | aggiornare | npm |
| shadcn/ui `new-york`, `neutral`, `cssVariables`, lucide | presente (`components.json:3,9,10,20`) | tenere | file |
| `radix-ui` unificato | assente: pacchetti separati `@radix-ui/react-slot`, `@radix-ui/react-tooltip` (`package.json:17-18`, `button.tsx:1`, `tooltip.tsx:1`) | aggiungere `radix-ui` e togliere i pacchetti separati (`shadcn migrate radix`) | shadcn `cli.mdx` §migrate radix |
| `cn` helper | `src/lib/utils.ts` usa clsx + tailwind-merge | aggiornare: shadcn ora usa il pacchetto `cn` (vedi §4) | changelog shadcn 2026-09 |
| Tailwind 4 + `@tailwindcss/vite` | `^4.1.17` (lock 4.2.2), plugin attivo (`vite.config.ts:8`) | aggiornare a 4.3.3 | npm |
| Token in `src/app/global.css` | presente (`global.css:5-110`, oklch neutral) | tenere | file |
| `tw-animate-css` | installato `^1.4.0` ma **non importato** | aggiungere `@import "tw-animate-css";` | `global.css:1` |
| `tailwindcss-animate` | presente (`package.json:32`) | togliere | non usato in `src/` (grep) |
| Font `@fontsource-variable` | assente | aggiungere (font da scegliere) | grep `fontsource` vuoto |
| Bun | usato (`tauri.conf.json:3-4`, `.husky/pre-commit:3`) | tenere | file |
| Biome via ultracite | `extends ultracite/biome/core + react` (`biome.jsonc:3`) | aggiornare e **correggere `includes`**; escludere `bindings.ts` | test §1.2 |
| Husky + lint-staged con `bunx ultracite fix` | presente (`package.json:50-54`, `.husky/pre-commit`) | tenere, aggiornare le versioni | file |
| `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | assenti | aggiungere gli script | `package.json:6-15` |
| Script `typecheck` | presente (`tsc --noEmit`) | tenere | `package.json:14` |
| Script `check` | presente (`ultracite check`) | tenere (eventualmente unirlo a typecheck e test) | `package.json:11` |
| Script `test`, `format:backend`, `lint:backend` | assenti | aggiungere | `package.json` |
| `env` con zod (`src/config/env.ts`) | presente | tenere o togliere (Sbobino non ha API remote) | file |
| Dependabot | presente | tenere o togliere (da decidere) | `.github/dependabot.yml` |

---

## 3. Versioni correnti consigliate da fissare

### npm (`npm view <pkg> version`, eseguito il 2026-10-02) [V]

| Pacchetto | Ultima | Note |
|---|---|---|
| `@tauri-apps/api` | 2.12.1 | `latest`; esiste anche `next` = 3.0.0-alpha.2 |
| `@tauri-apps/cli` | 2.12.1 | `latest`; `next` = 3.0.0-alpha.4 |
| `@tauri-apps/plugin-log` | 2.10.0 | dipende da `@tauri-apps/api ^2.12.0` |
| `@tauri-apps/plugin-opener` | 2.7.0 | dipende da `@tauri-apps/api ^2.12.0` |
| `react` / `react-dom` | 19.3.0 | |
| `@types/react` / `@types/react-dom` | 19.3.0 | |
| `typescript` | 7.0.2 (`latest`) / 6.0.3 (ultima 6.x) | vedi §4 |
| `vite` | 8.3.2 | 8.0.0 pubblicata il 2026-03-12; engines `node ^20.19.0 \|\| >=22.12.0` |
| `@vitejs/plugin-react` | 6.1.1 | peer `vite ^8.0.0` |
| `@tailwindcss/vite` / `tailwindcss` | 4.3.3 | peer `vite ^5.2.0 \|\| ^6 \|\| ^7 \|\| ^8` |
| `tw-animate-css` | 1.4.0 | |
| `react-router` | 8.4.0 (`latest`) / **7.18.4** (`version-7`) | entrambe pubblicate il 2026-09-15 |
| `react-hook-form` | 7.89.0 | |
| `@hookform/resolvers` | 5.9.1 | peer `zod ^3.25.0 \|\| ^4.0.0`, `react-hook-form ^7.55.0` |
| `zod` | 4.6.5 | |
| `react-error-boundary` | 6.1.6 | peer `react ^18 \|\| ^19` |
| `radix-ui` | 1.6.7 | peer React fino a 19 |
| `lucide-react` | 1.50.0 | |
| `class-variance-authority` | 0.7.1 | |
| `cn` | 0.4.0 | maintainer `shadcn`, repo `github.com/shadcn-ui/cn` |
| `clsx` / `tailwind-merge` | 2.1.1 / 3.7.0 | non servono più se si adotta `cn` |
| `shadcn` (CLI, da usare con `bunx`) | 4.21.1 | |
| `ultracite` | 7.12.2 | peer `@biomejs/biome ^2.5.0`; devDependency interna `@biomejs/biome 2.5.14` |
| `@biomejs/biome` | 2.5.15 | |
| `husky` | 9.1.7 | engines `node >=18` |
| `lint-staged` | 17.6.0 | engines `node >=22.22.1` (Node locale: 25.2.0) |
| `@types/bun` | 1.4.2 | per i tipi di `bun:test` |
| `@fontsource-variable/inter`, `@fontsource-variable/geist` | 5.3.0 | esempi: il font va scelto |
| `bun` (runtime) | 1.4.2 su npm; locale 1.4.0 | |

### crates.io (`https://crates.io/api/v1/crates/<nome>` con User-Agent) [V]

| Crate | Ultima stabile / rc | Note |
|---|---|---|
| `tauri` | **2.12.1** (2026-09-30) | `rust_version = 1.90`; `max_version` = 3.0.0-alpha.4 (da evitare) |
| `tauri-build` | 2.7.1 | |
| `tauri-plugin-log` | 2.10.0 | richiede `tauri ^2.12` |
| `tauri-plugin-opener` | 2.7.0 | richiede `tauri ^2.12` |
| `tauri-specta` | **=2.0.0-rc.25** (2026-05-08) | ultima rc; la stabile 1.0.2 è per Tauri 1 |
| `specta` | **=2.0.0-rc.25** | |
| `specta-typescript` | **=0.0.12** | dipende da `specta =2.0.0-rc.25` |
| `serde` / `serde_json` | 1.0.229 / 1.0.151 | |

**Coppia Tauri corrente [V]:** crate `tauri 2.12.1` con `@tauri-apps/api 2.12.1` e `@tauri-apps/cli 2.12.1`, quindi la minor 2.12 è condivisa. La CLI fallisce se major e minor non coincidono: il messaggio è «Make sure the NPM package and Rust crate versions are on the same major/minor releases» (`crates/tauri-cli/src/info/plugins.rs:145-170`, branch `dev` @ `30da1fd6`). [I] Lo stesso controllo copre anche i plugin, quindi `tauri-plugin-log 2.10` va con `@tauri-apps/plugin-log 2.10` e `opener 2.7` con `opener 2.7`.

**Compatibilità di tauri-specta rc.25 [V]** (clone del tag `v2.0.0-rc.25` @ `e5e578fb`):
- `Cargo.toml:31-37,68-75`: `specta = "=2.0.0-rc.25"` con le feature `derive` e `function`, `specta-typescript = "0.0.12"`, `tauri = "2"` con la feature `specta`.
- Le feature di tauri-specta sono `derive` (serve per gli eventi), `typescript` e `javascript` (`Cargo.toml:20-24`).
- La feature `specta` di `tauri` 2.12.1 richiede `specta ^2.0.0-rc.16` (crates.io dependencies API), che è compatibile con rc.25.
- Comando ufficiale di installazione (`src/lib.rs:16-17`): `cargo add specta@=2.0.0-rc.25 specta-typescript@0.0.12` e `cargo add tauri-specta@=2.0.0-rc.25 --features derive,typescript`. [I] Conviene pinnare con `=` anche `specta-typescript`, visto che è 0.0.x.

**Export dei bindings in debug [V]** (`src/lib.rs:48-69`, `src/builder.rs:378-385`):
- Si crea `Builder::<tauri::Wry>::new().commands(collect_commands![...]).events(collect_events![...])`.
- Poi, sotto `#[cfg(debug_assertions)]`, si chiama `builder.export(Typescript::default(), "../src/bindings.ts").expect(...)`.
- Infine `tauri::Builder::default().invoke_handler(builder.invoke_handler()).setup(move |app| { builder.mount_events(app); Ok(()) })`.

**Eventi tipizzati [V]** (`src/lib.rs:112-164`):
- Lato Rust: `#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, tauri_specta::Event)]`, poi `.events(collect_events![MyEvent])` e `mount_events(app)` (obbligatorio). Per emettere: `MyEvent(..).emit(app)`.
- Lato frontend: `events.myEvent.listen(cb)` oppure `events.myEvent(window).listen(cb)`.

Comportamenti di default rilevanti [V]:
- `ErrorHandlingMode::Result` è il default (`src/builder.rs:15-21`): i comandi che restituiscono `Result` producono un tipo risultato, non un'eccezione.
- I tipi in stile BigInt (`u64`, `i64`) causano un errore di export, a meno di usare `dangerously_cast_bigints_to_number()` (`src/builder.rs:304-311`).

---

## 4. Incertezze e rischi

1. **react-router 7 o 8.** La richiesta dice 7, ma il template è già passato a `^8.3.0` (PR #82, 2026-08-05), senza aggiornare il lock. [V] La v8 richiede `react >=19.2.7` e Node >=22.22.0, è solo ESM e rimuove `react-router-dom` e diversi flag `future.*` (CHANGELOG `react-router@8.4.0`, righe 144-188). La 7.x è ancora mantenuta: 7.18.4 è uscita il 2026-09-15. [I] L'uso del template (`createBrowserRouter` e `lazy` in `src/app/router.tsx`) sembra compatibile con entrambe, ma non è stato provato. La decisione spetta all'utente.
2. **TypeScript 7.0.2 è `latest`.** [V] Il pacchetto distribuisce binari nativi per piattaforma (`@typescript/typescript-win32-x64`, ...) e l'export `"."` è solo `./lib/version.cjs`, con API sperimentali sotto `./unstable/*`. [I] Gli strumenti che importano l'API JS classica di `typescript` potrebbero rompersi. La CLI shadcn usa `ts-morph ^26`, che a quanto risulta include una propria copia di TS. Proposta prudente: `typescript 6.0.3`, oppure provare la 7 con `tsc --noEmit` prima di adottarla.
3. **shadcn e `import { cn } from "cn"`: non è un bug, è un cambiamento voluto.** [V] Il changelog «September 2026 - cn» (https://github.com/shadcn-ui/ui/blob/shadcn@4.21.1/apps/v4/content/docs/changelog/2026-09-cn.mdx, del 2026-09-03) dice che tutti i componenti importano `cn` dal pacchetto `cn`, che `init` genera un `lib/utils.ts` di una riga (`export { cn } from "cn"`) e che `npx shadcn migrate cn` converte i progetti esistenti. Il registry corrente lo conferma: `https://ui.shadcn.com/r/styles/new-york-v4/button.json` ha `dependencies: ["cn","radix-ui"]` e `import { cn } from "cn"`. Il pacchetto npm `cn` 0.4.0 è pubblicato da `shadcn`. [I] Se si vuole che i componenti importino `@/lib/utils`, bisogna riscrivere l'import a mano dopo ogni `add`. La strada più semplice è accettare `cn` e far ri-esportare `@/lib/utils` da `cn`. Nota [V]: la CLI 4.21.1 dipende da `cn ^0.2.4`, mentre l'ultima versione è 0.4.0.
4. **shadcn CLI v4 e lo stile `new-york`.** [V] `init` ora ruota attorno a `--preset`, `--base radix|base|aria` e `--template vite`, con default `--preset=nova` (`cli.mdx` righe 8-58). Lo stile nei preset è codificato come `radix-<style>` (`packages/shadcn/src/preset/preset.ts`). `components-json.mdx` §style dice ancora `"style": "new-york"` e che `default` è deprecato. L'indice `r/styles/index.json` elenca `new-york` e `default`. [I] Non è chiaro se `init` interattivo proponga ancora `new-york` o solo i preset (nova e altri). Proposta: scrivere `components.json` a mano (quello del template va già bene) e usare solo `shadcn add`, poi verificare che i file scaricati provengano da `new-york-v4`.
5. **`radix-ui` unificato.** [V] I componenti del registry new-york-v4 importano già da `radix-ui`, e `shadcn migrate radix` converte gli import `@radix-ui/react-*`. Rischio basso.
6. **Esclusioni Biome con file passati esplicitamente da lint-staged.** [V] Test con Biome 2.5.15: `biome check --no-errors-on-unmatched src/components/ui/y.ts src/bindings.ts` produce `Checked 0 files`, quindi le esclusioni valgono anche per i path espliciti. `ultracite fix` esegue `biome check --write --no-errors-on-unmatched <files|./>` (`packages/cli/src/commands/fix.ts:48-56`, ultracite @ `e948def3`, v7.12.2). Init non interattivo: `bunx ultracite init --pm bun --linter biome --frameworks react --integrations husky lint-staged --quiet` (SKILL.md di ultracite, via ctx7). [I] L'init potrebbe sovrascrivere `biome.jsonc` e la config di lint-staged: rivedere il diff dopo averlo lanciato.
7. **`bun test` e l'alias `@/`.** [V] Bun risolve nativamente `compilerOptions.paths` del tsconfig (docs `guides/runtime/tsconfig-paths.mdx`, `src/resolver/resolver.rs` `match_tsconfig_paths`), quindi non serve una config dedicata. [I] Restano due limiti:
   - `import.meta.env` (usato in `src/lib/create-env.ts:4`) esiste solo con Vite: sotto `bun test` è da verificare. Lo stesso vale per gli import di CSS e SVG.
   - Per i tipi di `bun:test` in `tsc` serve `@types/bun`, probabilmente nei `types` di un tsconfig dedicato ai test: non verificato con TS 6/7.
8. **Vite 8 è stabile.** [V] 8.0.0 è del 2026-03-12, 8.3.2 del 2026-10-01; `@vitejs/plugin-react 6.1.1` e `@tailwindcss/vite 4.3.3` dichiarano il supporto a Vite 8 nei peer. [I] Vite 8 usa rolldown (il lock del template mostra `rolldown 1.0.0-rc.12` come dipendenza di vite 8.0.3, `bun.lock:426`). La compatibilità con altri plugin non è stata verificata.
9. **Tauri 3 alpha in uscita.** [V] `tauri 3.0.0-alpha.4` (2026-10-01) e `@tauri-apps/* next` sono già pubblicati. Bisogna pinnare la major 2 ed evitare `^` aperti su npm che possano agganciare le prerelease. [I] Se Tauri 3 diventa stabile, tauri-specta rc.25 (`tauri = "2"`) non lo supporterà finché non esce una nuova rc.
10. **tauri-specta è ancora in rc**, l'ultima uscita (rc.25) risale al 2026-05-08. [V] La documentazione chiede esplicitamente il pin con `=` (`src/lib.rs:7-9`). [I] Gli aggiornamenti futuri possono cambiare il formato di `bindings.ts`, per esempio le fasi serde `_Serialize`/`_Deserialize` introdotte in rc.25 (`src/lib.rs:168-184`).
11. **Toolchain Rust.** [V] La stable da manifest è 1.99.0 (2026-09-28), quella locale 1.96.1; `tauri 2.12.1` richiede 1.90 e `tauri-specta` usa `edition = "2024"`, che richiede Rust 1.85 o superiore. [I] Va bene sia la versione locale sia l'ultima; il target MSVC non è stato verificato (`rustup show` non eseguito).
12. **Non verificati in questa ricerca:**
    - build effettiva (`cargo build`, `bun install`, `tauri dev`);
    - comportamento di `clippy -D warnings` sul codice generato da `tauri::generate_context!` e dalle macro specta;
    - se servono ancora `[features] custom-protocol` (`Cargo.toml:20-22`) e la struttura `lib.rs` + `main.rs` dei template Tauri 2 ufficiali;
    - avvisi della CLI sull'identifier `com.tauri.app`;
    - permessi `log:default` e `opener:default` nella capability (indicati dai docs Tauri v2 `plugin/logging.mdx` e `plugin/opener.mdx` via ctx7, non provati).
