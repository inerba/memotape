# 04: La lista dei Termini in Impostazioni

**What to build:** In Impostazioni → Trascrizione la lista del Vocabolario: campo con Aggiungi, pillole con ×, salvataggio immediato. Regole in `../spec.md`, sezione "Impostazioni e interfaccia"; testi in `PRODUCT.md`, storie 81–83.

**Blocked by:** 02.

**Status:** ready-for-agent

- [ ] Logica pura in `src/features/settings/` (con `*.test.ts` accanto): aggiungere uno o più Termini da un testo (una riga per Termine), con trim, righe vuote ignorate, doppioni senza distinguere maiuscole e accenti (`normalize("NFD")`), rifiuto di `<|`/`|>`, conteggio degli scartati; rimuovere un Termine.
- [ ] Componente: campo + **Aggiungi** (Invio aggiunge; incollare più righe ne aggiunge più), messaggio accanto al campo per scarti e doppioni con il testo che resta, pillole che vanno a capo nell'ordine di aggiunta con una × dal nome accessibile "Rimuovi <Termine>", riga di spiegazione senza Termini, nota fissa su Whisper che considera solo gli ultimi Termini di un elenco lungo. Salva con `set_settings` tramite `SettingsProvider`, come gli altri controlli; un errore del salvataggio si mostra.
- [ ] Solo token del tema e componenti esistenti (`components/ui` solo via `shadcn add`); contratto visivo in `DESIGN.md`.
- [ ] Testi nelle sei lingue (`src/locales/*.json`, `it.json` riferimento).
- [ ] I sei controlli passano.
- [ ] Prova manuale con `bun tauri dev` (via CDP, vedi AGENTS.md "Pilotare l'app"): aggiungere, incollare più righe, doppione, rimuovere, riavvio con la lista conservata; ripristinare le impostazioni dell'utente alla fine.

## Comments
