# Insidie della toolchain

Versioni bloccate e trappole di Tauri, specta, TypeScript, Biome, shadcn, reqwest, zip e download. Riferimento per gli agenti, spostato da `AGENTS.md` il 10 ottobre 2026: aggiornalo quando cambia il comportamento che descrive.

- **Tauri**: crate `tauri` e pacchetti `@tauri-apps/*` devono condividere major e minor (oggi 2.12), altrimenti la CLI si rifiuta di partire. I plugin seguono la stessa regola (`tauri-plugin-log` 2.10 con `@tauri-apps/plugin-log` 2.10). Tauri 3 è in alpha: le versioni npm sono pinnate esatte per non agganciare prerelease.
- **specta**: `specta`, `tauri-specta` (`=2.0.0-rc.25`) e `specta-typescript` (`=0.0.12`) restano pinnati con `=`; un aggiornamento può cambiare il formato di `bindings.ts`.
- **Tipi esportati**: niente `u64`/`i64`, bloccano l'export dei bindings. Usa `u32` (tutti i modelli stanno sotto i 4 GiB) o `f64`.
- **`bindings.ts`**: si rigenera solo in debug all'avvio dell'app (`bun tauri dev`). Il test `i_bindings_committati_sono_aggiornati` fallisce se il file committato non corrisponde ai comandi Rust: rigeneralo e committalo.
- **`cargo test` e il manifest Windows**: `tauri-plugin-dialog` richiede Common Controls v6. `build.rs` incorpora `windows-app-manifest.xml` in tutti gli eseguibili, test compresi; senza, i test escono con `STATUS_ENTRYPOINT_NOT_FOUND`.
- **TypeScript**: resta su 6.0.3, non la 7 (binari nativi, API JS sperimentale). In TS 6 `types` vale `[]` per default: i tipi globali (`bun`, `node`) vanno elencati nel tsconfig.
- **react-router**: resta su 7.18.x, non la 8.
- **Biome**: senza un `"**"` iniziale negli `includes` Biome non controlla nessun file (era il bug del template). Il core di ultracite 7.12 lo mette già, quindi in `biome.jsonc` restano solo le negazioni (`components/ui`, `bindings.ts`): ripetere `"**"` fa scattare `noBiomeFirstException`, e `ultracite fix` lo toglie. Se aggiorni ultracite, verifica che un `debugger` in `src/` venga ancora segnalato.
- **shadcn**: `components.json` è scritto a mano (stile `new-york`, base `neutral`); si usa solo `shadcn add`, perché `init` ora parte dai preset. I componenti importano `cn` dal pacchetto `cn`: è voluto (changelog shadcn del 2026-09-03), e `@/lib/utils` lo ri-esporta. Non riscrivere l'import.
- **Animazioni**: `tw-animate-css` è importato in `global.css`. `tailwindcss-animate` è il plugin di Tailwind 3 e non serve.
- **Line ending**: `.gitattributes` forza LF, come si aspetta Biome.
- **reqwest**: usa `native-tls` (Schannel) senza feature di default. Il default rustls + aws-lc richiede NASM sulla build MSVC.
- **zip**: 8.6 senza feature di default (AES, bzip2, zstd, lzma…), solo `deflate-flate2-zlib-rs`, in Rust puro. La 9 è in pre-release. `mix.ogg` deve restare `Stored`: `tape::Mix` lo legge come finestra sul file e rifiuta una voce compressa.
- **Download e Annulla**: il trasferimento corre in un `tokio::select!` contro il `Notify` di Annulla. Il `.partial` si cancella solo dopo l'uscita dal `select!`, quando il future e il suo file sono già chiusi: su Windows un file aperto non si cancella. Il `select!` serve perché un trasferimento fermo non restituisce chunk, quindi un flag controllato a ogni chunk non scatterebbe. Durante la verifica l'hash gira in un thread bloccante con il file aperto: Annulla si controlla a hash finito, prima del rename. Nel codice e nei documenti il file scaricato a metà è il "`.partial`" o il "file incompleto", non il "parziale": Parziale è un termine del glossario.
