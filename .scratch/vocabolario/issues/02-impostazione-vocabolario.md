# 02: L'impostazione del Vocabolario

**What to build:** Il campo `vocabolario` (lista di Termini, `#[serde(default)]`, vuota) in `Settings`, validato da Rust a ogni salvataggio, con bindings e schema del frontend aggiornati. Regole in `../spec.md`, sezione "Impostazioni e interfaccia" (la parte di Rust e del modello dati, non la UI).

**Blocked by:** nessuno.

**Status:** done

- [x] `Settings::vocabolario: Vec<String>`; un file di prima senza il campo lo legge vuoto senza perdere il resto (come gli altri campi facoltativi).
- [x] Al salvataggio (`set_settings`/`SettingsStore`) i Termini perdono gli spazi in testa e in coda, quelli vuoti si tolgono e i doppioni esatti pure; un Termine con `<|` o `|>` si rifiuta con un `AppError` dedicato (codice nuovo, messaggio tradotto nelle sei lingue), così una Trascrizione con Whisper non fallisce.
- [x] `src/bindings.ts` rigenerato (`cargo test -- --ignored rigenera_bindings_di_sviluppo` in `src-tauri`), `settingsSchema` e `DEFAULT_SETTINGS` in `src/features/settings/` allineati.
- [x] Test (con `tdd`): predefinito vuoto, file di prima, normalizzazione, rifiuto di `<|`.
- [x] AGENTS.md: il campo nell'elenco delle impostazioni con `#[serde(default)]`.
- [x] I sei controlli passano.

## Comments

- 2026-10-08: implementato test-first. `Settings::vocabolario: Vec<String>` con `#[serde(default)]`. Normalizzazione (`trim`, vuoti e doppioni esatti tolti, ordine conservato) e rifiuto stanno nella funzione `vocabolario` di `managers::settings`, chiamata da `SettingsStore::set`: valgono per ogni salvataggio e si testano senza Tauri; `set_settings` non cambia. Errore nuovo `AppError::InvalidTermine(termine)` (codice `invalidTermine`, il dettaglio è il Termine senza spazi), tradotto nelle sei lingue in `errors.codes`. Il primo Termine con `<|`/`|>` rifiuta tutto il salvataggio: impostazioni correnti e file restano com'erano. Schema zod: `vocabolario: z.array(z.string()).optional()`, senza regola su `<|`, perché lo schema rispecchia ciò che Rust accetta in lettura. Deviazione: `Settings::load` non filtra i Termini di un `settings.json` modificato a mano (il ticket chiede la validazione al salvataggio); se serve, il ticket 03 può scartarli prima del prompt di Whisper.
