# 02: L'impostazione del Vocabolario

**What to build:** Il campo `vocabolario` (lista di Termini, `#[serde(default)]`, vuota) in `Settings`, validato da Rust a ogni salvataggio, con bindings e schema del frontend aggiornati. Regole in `../spec.md`, sezione "Impostazioni e interfaccia" (la parte di Rust e del modello dati, non la UI).

**Blocked by:** nessuno.

**Status:** ready-for-agent

- [ ] `Settings::vocabolario: Vec<String>`; un file di prima senza il campo lo legge vuoto senza perdere il resto (come gli altri campi facoltativi).
- [ ] Al salvataggio (`set_settings`/`SettingsStore`) i Termini perdono gli spazi in testa e in coda, quelli vuoti si tolgono e i doppioni esatti pure; un Termine con `<|` o `|>` si rifiuta con un `AppError` dedicato (codice nuovo, messaggio tradotto nelle sei lingue), così una Trascrizione con Whisper non fallisce.
- [ ] `src/bindings.ts` rigenerato (`cargo test -- --ignored rigenera_bindings_di_sviluppo` in `src-tauri`), `settingsSchema` e `DEFAULT_SETTINGS` in `src/features/settings/` allineati.
- [ ] Test (con `tdd`): predefinito vuoto, file di prima, normalizzazione, rifiuto di `<|`.
- [ ] AGENTS.md: il campo nell'elenco delle impostazioni con `#[serde(default)]`.
- [ ] I sei controlli passano.

## Comments
