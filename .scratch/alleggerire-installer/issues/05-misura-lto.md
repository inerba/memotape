# 05: Misura di LTO sull'exe

**What to build:** La decisione, basata su una misura, se il profilo release usa `lto = true` e `codegen-units = 1`. Spec: `../spec.md`, sezione "LTO".

**Blocked by:** 04

**Status:** done

- [x] Due build release dello stesso commit, con e senza il profilo; si riportano dimensione dell'exe, dell'installer e tempo di build.
- [x] Il profilo si adotta solo se l'exe cala di almeno il 10%; nessun `opt-level = "s"`, `panic = "abort"` o `strip`.
- [ ] (non applicabile: profilo non adottato) Se adottato: una Trascrizione della fixture e una Registrazione di prova riuscite con l'installer nuovo, e la durata della build release aggiornata in `docs/sviluppo/installer.md`.
- [x] Esito e numeri nei commenti di questo ticket anche se il profilo non si adotta.
- [x] I sei controlli passano.

**Esito: LTO non si adotta.** Con `lto = true` e `codegen-units = 1` l'exe cala del 5,3%, sotto la soglia del 10% della spec. L'installer cresce di poco e la build release dura quasi il doppio. `Cargo.toml` non cambia.

Due build release da zero dello stesso commit (`fb33300d`), il 18 ottobre 2026, sulla macchina di sviluppo (Ryzen 7 3700X), una dopo l'altra. Ognuna ha una cartella target nuova e separata, così `src-tauri/target/release` condiviso non viene toccato. Le variabili ORT sono tolte dall'ambiente con `env -u`. Il profilo di oggi è quello predefinito di Cargo per `release` (nel manifest non c'è `[profile.release]`). La variante LTO passa dalle variabili d'ambiente, senza modificare il manifest. Dalla root del worktree, in Git Bash:

```
env -u ORT_LIB_LOCATION -u ORT_PREFER_DYNAMIC_LINK CARGO_TARGET_DIR='D:\local\tauri\sbobino-lto\oggi' bun tauri build
env -u ORT_LIB_LOCATION -u ORT_PREFER_DYNAMIC_LINK CARGO_PROFILE_RELEASE_LTO=true CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR='D:\local\tauri\sbobino-lto\lto' bun tauri build
```

| | Profilo di oggi | `lto = true`, `codegen-units = 1` | Differenza |
| --- | --- | --- | --- |
| `memotape.exe` | 43 190 784 byte | 40 886 784 byte | −2 304 000 (−5,3%) |
| Installer NSIS | 26 047 270 byte (24,84 MiB) | 26 250 516 byte (25,03 MiB) | +203 246 (+0,8%) |
| Build Rust (`Finished release`) | 7 min 27 s | 15 min 06 s | +103% |
| `bun tauri build` intero (frontend, Rust, NSIS) | 499 s | 959 s | +92% |

- I tempi sono di build da zero, transcribe-cpp compreso. Non si può escludere carico da altre sessioni sulla stessa macchina.
- Che LTO fosse attivo lo dicono l'exe più piccolo e la build più lunga: la riga `Finished` dice `[optimized]` in tutte e due, e il log non stampa le variabili `CARGO_PROFILE_RELEASE_*`.
- Una prima prova con la cartella target nello scratchpad era fallita: il percorso era troppo lungo per CMake di `opusic-sys`.
- Le due cartelle di prova sono state cancellate.
- L'exe di oggi ha la stessa dimensione di quello del ticket 04 (43 190 784 byte). L'hash non è stato confrontato.
- **Installer in `D:\local\tauri\sbobino\src-tauri\target\release\bundle\nsis\` (il target condiviso):** è ancora quello del ticket 04 (26 048 604 byte, 18:58), con il profilo di oggi, senza LTO. Questo ticket non l'ha rigenerato.

Perché l'installer cresce mentre l'exe cala: è un'ipotesi, non misurata. Il codice ottimizzato con LTO si comprime peggio con LZMA, e nell'installer pesano soprattutto DLL e modelli, che LTO non tocca.

I sei controlli, rieseguiti dopo le misure sullo stesso commit, sono verdi: questo ticket non cambia il codice.
