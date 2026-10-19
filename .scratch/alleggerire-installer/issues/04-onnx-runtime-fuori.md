# 04: ONNX Runtime fuori da build, bundle e documenti

**What to build:** Un installer e una build senza ONNX Runtime: niente `onnxruntime.dll`, `vad-rs`, `ort`, `silero_vad.onnx` né `ORT_LIB_LOCATION`. Crediti e licenze riflettono i componenti distribuiti. Spec: `../spec.md`, sezioni "Build e bundle" e "Documentazione".

**Blocked by:** 02

**Status:** done

- [x] Via `vad-rs` dal manifest (e `ort` dal lock); lo script di build non chiede più `ORT_LIB_LOCATION` e non copia `onnxruntime.dll`; le DLL vecchie spariscono da `runtime-libs/` con la pulizia esistente. Restano le DLL del runtime VC++.
- [x] Via le licenze di ONNX Runtime (comprese le note di terze parti) e di `vad-rs`; la licenza di Silero passa a v6.2 con l'attribuzione del GGUF. I crediti in Informazioni seguono, con i loro test; il ruolo «onnx» sparisce se non resta nessun componente che lo usa, con le chiavi di traduzione.
- [x] `bun run worktree:setup` e i suoi test non dipendono più dalle variabili ORT.
- [x] Aggiornati `AGENTS.md`, `README.md`, `PRODUCT.md` dove nominano ONNX Runtime o Silero v4, e in `docs/sviluppo/` prerequisiti, insidie audio, installer e verifica manuale.
- [x] Verifica dell'installer come in `docs/sviluppo/installer.md`: nessun `onnxruntime.dll` nell'installer né tra i moduli caricati, Trascrizione della fixture e Registrazione di prova riuscite. Dimensione dell'installer e dell'installato riportate nei commenti, contro 29,6 MB e 126,5 MB della 0.5.3.
- [x] I sei controlli passano su un worktree nuovo preparato senza le variabili ORT.

**Esito:** fatto tutto tranne la verifica dal vivo dell'installer, che resta da fare a mano (per questo `ready-for-human`).
- `vad-rs` è uscito dal manifest; dal lock spariscono `ort`, `ort-sys`, `vad-rs` e 14 dipendenze transitive, senza aggiornamenti. `build.rs` non legge più `ORT_LIB_LOCATION` e non copia `onnxruntime.dll`: la pulizia esistente l'ha tolta da `runtime-libs/`. Le DLL VC++ restano.
- Le licenze di ONNX Runtime (con le note di terze parti) e di `vad-rs` sono uscite. `silero-vad.txt` è Silero v6.2, con il NOTICE e la revisione del GGUF di handy-computer. I ruoli `onnx` e `vadRs` sono spariti dai crediti, dal test e dalle sei lingue.
- `worktree:setup` non copia più le variabili `ORT_*` della config principale (test in `scripts/worktree-setup.test.ts`). Il worktree, ripreparato con lo script nuovo, non ha variabili ORT, e i sei controlli passano lì senza ORT anche nell'ambiente.
- Documenti aggiornati: `AGENTS.md`, `README.md`, `PRODUCT.md`, prerequisiti, insidie audio, installer, verifica manuale, ADR-0035, commento di `.gitignore`.
- Installer, da `bun tauri build` senza variabili ORT:
  - `Memotape_0.5.3_x64-setup.exe` pesa 26 048 604 byte (24,84 MiB), contro 29,6 MB della 0.5.3;
  - i file installati, estratti con `7z`, fanno 111 203 644 byte (più `uninstall.exe`), contro 126,5 MB;
  - nell'installer non ci sono `onnxruntime.dll` né `silero_vad.onnx`, c'è `resources\silero-vad-v6.2-F32.gguf`;
  - `dumpbin /dependents memotape.exe` elenca `transcribe.dll` e nessun `onnxruntime.dll`.
- **Non fatto:** l'installazione con moduli caricati, Trascrizione della fixture e Registrazione di prova. Sulla macchina c'è un Memotape installato e aperto in `%LOCALAPPDATA%\Memotape` (una build del branch RNNoise di oggi). Installare quella di prova avrebbe sovrascritto e poi tolto la sua chiave di disinstallazione in HKCU, l'istanza unica avrebbe girato l'avvio all'app aperta, e la Trascrizione avrebbe scritto nella Libreria vera. Da fare a mano, o con l'app dell'utente chiusa e il suo consenso (`docs/sviluppo/installer.md`).

**Verifica dal vivo (19 ottobre 2026, con il consenso dell'utente):** l'installer è quello sopra, stesso SHA-256 `f412e01f…947ae7`, profilo senza LTO. Installato con `/S` sopra l'installazione dell'utente in `%LOCALAPPDATA%\Memotape`, che era una build del branch RNNoise. Lì resta installato.
- **Avvio:** l'exe è partito senza `VULKAN_SDK` né Vulkan SDK nel `PATH`, senza variabili ORT, con una WebView di prova e la porta CDP 9223.
- **Moduli caricati:**
  - dalla cartella d'installazione: `transcribe.dll`, `ggml.dll`, `ggml-base.dll`, `ggml-vulkan.dll`, `ggml-cpu-haswell.dll`, `MSVCP140.dll`, `VCRUNTIME140.dll` e `VCRUNTIME140_1.dll`;
  - da System32: `vulkan-1.dll`;
  - nessun `onnxruntime.dll`.
- **Backend e modello:** backend Vulkan0 (RTX 2070 SUPER) e CPU, Parakeet caricato in 1,2 s.
- **Trascrizione:** `transcribe` su una copia di `parlato-it.wav` fuori dal repo ha salvato `parlato-it.tape` in 986 ms, con il testo «Buongiorno a tutti, oggi parliamo di trascrizione. Il computer trasforma la voce in testo.».
- **Registrazione:** `record` più `stop_recording` dopo 5 s ha salvato un Tape di 4 s, `error: null`, Trascrizione `noSpeech` (era silenzio).
- **Pulizia:** i due Tape di prova sono andati nel Cestino con `trash_tape`. L'app è stata chiusa e il log non ha errori.

**Scoperta, da decidere a parte:** installando sopra una versione precedente, l'installer NSIS non toglie i file che non distribuisce più. Nella cartella restano:
- `onnxruntime.dll` (14,1 MB), `resources\silero_vad.onnx` e le licenze di ONNX Runtime;
- qui anche `rnnoise.txt` e `resources\models\rnnoise-0b50c45.bin` della build RNNoise.

Il totale della cartella è 128 970 622 byte, contro i 111,2 MB di un'installazione pulita. L'app non carica `onnxruntime.dll`, ma chi aggiorna dalla 0.5.3 non recupera lo spazio finché non disinstalla. La correzione andrebbe nell'installer, per esempio un hook NSIS che cancella i file obsoleti in fase di installazione: è fuori da questo ticket.
