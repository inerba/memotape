# 12: Installer NSIS

**What to build:** un installer NSIS che installa Sbobino su un PC Windows x64 pulito. Dopo il download di un modello l'app trascrive e registra, senza altri prerequisiti.

**Blocked by:** 02, 03, 04, 05, 06, 07, 08, 09, 10

**Status:** done

- [x] `transcribe-cpp` con `dynamic-backends`, così il binario non è ottimizzato per la CPU della macchina di build. Le DLL dei backend sono copiate accanto all'exe
- [x] Le DLL dell'ONNX Runtime ufficiale 1.24.2 sono incluse (niente build prebuilt AVX2 di `ort`). Si verifica se serve `vcomp140.dll`
- [x] Silero e i testi delle licenze sono inclusi nel bundle
- [x] Editore dell'installer: se non è ancora deciso, resta un segnaposto e viene segnalato
- [x] Verifica manuale documentata in `AGENTS.md`: installazione, download di un modello, Trascrizione e Registrazione
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi

## Comments

- `dynamic-backends` vale anche in dev: una sola configurazione, motivata in `AGENTS.md` (Insidie, transcribe-cpp).
- `vcomp140.dll` non serve (OpenMP spento, nessuna DLL lo importa). Servono invece `msvcp140*` e `vcruntime140*`, inclusi app-local dal redistribuibile di Visual Studio.
- Editore: segnaposto "EDITORE DA DEFINIRE" in `bundle.publisher`, da decidere prima della prima release pubblica.
- La verifica sulla macchina di sviluppo (installazione silenziosa, DLL dal bundle, Trascrizione con un modello già scaricato, Registrazione, disinstallazione) è in `AGENTS.md` → "Installer". Resta all'umano la prova su un PC pulito, compreso il download di un modello: è in "Da verificare sull'hardware reale".
