# 04: ONNX Runtime fuori da build, bundle e documenti

**What to build:** Un installer e una build senza ONNX Runtime: niente `onnxruntime.dll`, `vad-rs`, `ort`, `silero_vad.onnx` né `ORT_LIB_LOCATION`. Crediti e licenze riflettono i componenti distribuiti. Spec: `../spec.md`, sezioni "Build e bundle" e "Documentazione".

**Blocked by:** 02

**Status:** ready-for-human

- [x] Via `vad-rs` dal manifest (e `ort` dal lock); lo script di build non chiede più `ORT_LIB_LOCATION` e non copia `onnxruntime.dll`; le DLL vecchie spariscono da `runtime-libs/` con la pulizia esistente. Restano le DLL del runtime VC++.
- [x] Via le licenze di ONNX Runtime (comprese le note di terze parti) e di `vad-rs`; la licenza di Silero passa a v6.2 con l'attribuzione del GGUF. I crediti in Informazioni seguono, con i loro test; il ruolo «onnx» sparisce se non resta nessun componente che lo usa, con le chiavi di traduzione.
- [x] `bun run worktree:setup` e i suoi test non dipendono più dalle variabili ORT.
- [x] Aggiornati `AGENTS.md`, `README.md`, `PRODUCT.md` dove nominano ONNX Runtime o Silero v4, e in `docs/sviluppo/` prerequisiti, insidie audio, installer e verifica manuale.
- [ ] Verifica dell'installer come in `docs/sviluppo/installer.md`: nessun `onnxruntime.dll` nell'installer né tra i moduli caricati, Trascrizione della fixture e Registrazione di prova riuscite. Dimensione dell'installer e dell'installato riportate nei commenti, contro 29,6 MB e 126,5 MB della 0.5.3.
- [x] I sei controlli passano su un worktree nuovo preparato senza le variabili ORT.

**Esito:** fatto tutto tranne la verifica dal vivo dell'installer, che resta da fare a mano (per questo `ready-for-human`).
- `vad-rs` è uscito dal manifest; dal lock spariscono `ort`, `ort-sys`, `vad-rs` e 14 dipendenze transitive, senza aggiornamenti. `build.rs` non legge più `ORT_LIB_LOCATION` e non copia `onnxruntime.dll`: la pulizia esistente l'ha tolta da `runtime-libs/`. Le DLL VC++ restano.
- Le licenze di ONNX Runtime (con le note di terze parti) e di `vad-rs` sono uscite. `silero-vad.txt` è Silero v6.2, con il NOTICE e la revisione del GGUF di handy-computer. I ruoli `onnx` e `vadRs` sono spariti dai crediti, dal test e dalle sei lingue.
- `worktree:setup` non copia più le variabili `ORT_*` della config principale (test in `scripts/worktree-setup.test.ts`). Il worktree, ripreparato con lo script nuovo, non ha variabili ORT, e i sei controlli passano lì senza ORT anche nell'ambiente.
- Documenti aggiornati: `AGENTS.md`, `README.md`, `PRODUCT.md`, prerequisiti, insidie audio, installer, verifica manuale, ADR-0034, commento di `.gitignore`.
- Installer, da `bun tauri build` senza variabili ORT:
  - `Memotape_0.5.3_x64-setup.exe` pesa 26 048 604 byte (24,84 MiB), contro 29,6 MB della 0.5.3;
  - i file installati, estratti con `7z`, fanno 111 203 644 byte (più `uninstall.exe`), contro 126,5 MB;
  - nell'installer non ci sono `onnxruntime.dll` né `silero_vad.onnx`, c'è `resources\silero-vad-v6.2-F32.gguf`;
  - `dumpbin /dependents memotape.exe` elenca `transcribe.dll` e nessun `onnxruntime.dll`.
- **Non fatto:** l'installazione con moduli caricati, Trascrizione della fixture e Registrazione di prova. Sulla macchina c'è un Memotape installato e aperto in `%LOCALAPPDATA%\Memotape` (una build del branch RNNoise di oggi). Installare quella di prova avrebbe sovrascritto e poi tolto la sua chiave di disinstallazione in HKCU, l'istanza unica avrebbe girato l'avvio all'app aperta, e la Trascrizione avrebbe scritto nella Libreria vera. Da fare a mano, o con l'app dell'utente chiusa e il suo consenso (`docs/sviluppo/installer.md`).
