# 04: ONNX Runtime fuori da build, bundle e documenti

**What to build:** Un installer e una build senza ONNX Runtime: niente `onnxruntime.dll`, `vad-rs`, `ort`, `silero_vad.onnx` né `ORT_LIB_LOCATION`. Crediti e licenze riflettono i componenti distribuiti. Spec: `../spec.md`, sezioni "Build e bundle" e "Documentazione".

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Via `vad-rs` dal manifest (e `ort` dal lock); lo script di build non chiede più `ORT_LIB_LOCATION` e non copia `onnxruntime.dll`; le DLL vecchie spariscono da `runtime-libs/` con la pulizia esistente. Restano le DLL del runtime VC++.
- [ ] Via le licenze di ONNX Runtime (comprese le note di terze parti) e di `vad-rs`; la licenza di Silero passa a v6.2 con l'attribuzione del GGUF. I crediti in Informazioni seguono, con i loro test; il ruolo «onnx» sparisce se non resta nessun componente che lo usa, con le chiavi di traduzione.
- [ ] `bun run worktree:setup` e i suoi test non dipendono più dalle variabili ORT.
- [ ] Aggiornati `AGENTS.md`, `README.md`, `PRODUCT.md` dove nominano ONNX Runtime o Silero v4, e in `docs/sviluppo/` prerequisiti, insidie audio, installer e verifica manuale.
- [ ] Verifica dell'installer come in `docs/sviluppo/installer.md`: nessun `onnxruntime.dll` nell'installer né tra i moduli caricati, Trascrizione della fixture e Registrazione di prova riuscite. Dimensione dell'installer e dell'installato riportate nei commenti, contro 29,6 MB e 126,5 MB della 0.5.3.
- [ ] I sei controlli passano su un worktree nuovo preparato senza le variabili ORT.
