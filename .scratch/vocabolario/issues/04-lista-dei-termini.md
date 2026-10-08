# 04: La lista dei Termini in Impostazioni

**What to build:** In Impostazioni → Trascrizione la lista del Vocabolario: campo con Aggiungi, pillole con ×, salvataggio immediato. Regole in `../spec.md`, sezione "Impostazioni e interfaccia"; testi in `PRODUCT.md`, storie 81–83.

**Blocked by:** 02.

**Status:** done

- [x] Logica pura in `src/features/settings/` (con `*.test.ts` accanto): aggiungere uno o più Termini da un testo (una riga per Termine), con trim, righe vuote ignorate, doppioni senza distinguere maiuscole e accenti (`normalize("NFD")`), rifiuto di `<|`/`|>`, conteggio degli scartati; rimuovere un Termine.
- [x] Componente: campo + **Aggiungi** (Invio aggiunge; incollare più righe ne aggiunge più), messaggio accanto al campo per scarti e doppioni con il testo che resta, pillole che vanno a capo nell'ordine di aggiunta con una × dal nome accessibile "Rimuovi <Termine>", riga di spiegazione senza Termini, nota fissa su Whisper che considera solo gli ultimi Termini di un elenco lungo. Salva con `set_settings` tramite `SettingsProvider`, come gli altri controlli; un errore del salvataggio si mostra.
- [x] Solo token del tema e componenti esistenti (`components/ui` solo via `shadcn add`); contratto visivo in `DESIGN.md`.
- [x] Testi nelle sei lingue (`src/locales/*.json`, `it.json` riferimento).
- [x] I sei controlli passano.
- [x] Prova manuale con `bun tauri dev` (via CDP, vedi AGENTS.md "Pilotare l'app"): aggiungere, incollare più righe, doppione, rimuovere, riavvio con la lista conservata; ripristinare le impostazioni dell'utente alla fine. Fatta con Vite e comandi finti, non con `tauri dev`: vedi il commento.

## Comments

**2026-10-08 — implementazione (branch `feat/vocabolario-04`).**

- Logica pura in `src/features/settings/vocabolario.ts` (test in `vocabolario.test.ts`): `addTermini(termini, testo)` restituisce la lista nuova, quanti Termini sono entrati e le righe scartate come doppione o non ammesse (anche i doppioni dentro lo stesso incollato); `addMessage(esito)` sceglie il messaggio: con un solo scarto e nulla aggiunto nomina il Termine (doppione o `<|`/`|>`), altrimenti conta gli scartati (chiave plurale `settings.vocabolario.scartati`); `keep` dice se il testo resta nel campo (nulla è entrato). `removeTermine` toglie il Termine esatto.
- Componente `src/features/settings/vocabolario-list.tsx`, in Impostazioni → Trascrizione sotto la scelta del modello. Campo in un `<form>` nativo (Invio = submit = Aggiungi, disabilitato col campo vuoto). Incollare testo con più righe non entra nel campo: aggiunge subito un Termine per riga, e il testo già scritto nel campo resta. Il salvataggio passa da `save(change, "vocabolario")` di `SettingsProvider` con un'intenzione riapplicata alla lista corrente (non uno snapshot), quindi `SaveTick` accanto al titolo e `SettingFeedback` (errore con Riprova) come gli altri campi; dopo un errore la lista torna quella salvata.
- Pillole come le `Chip` di `DESIGN.md` (foglio, filo, 10 px), con la × da 24 px e l'anello salvia di focus; i Termini lunghi vanno a capo dentro la pillola. Dopo una rimozione il focus torna al campo (il pulsante sparisce con la pillola).
- Deviazioni: la spiegazione senza Termini dice anche che il Vocabolario vale dalla prossima Trascrizione e non cambia i Tape già trascritti. Il messaggio di scarto per più righe non distingue doppioni e `<|`/`|>` (un solo conteggio, come chiede la spec).
- Prova manuale: un altro implementatore usava la target dir condivisa, quindi niente `bun tauri dev`. Vite su una porta libera con una pagina di prova temporanea (non committata) che sostituiva `__TAURI_INTERNALS__` (`get_settings`/`set_settings` in `sessionStorage`, errore di scrittura a comando). Provati nei due temi: Invio e Aggiungi con trim, incollato di 5 righe con righe vuote, un doppione e un `<|it|>` (entrano le valide, "2 righe scartate"), doppione `NICCOLO` di `Niccolò` (messaggio, testo resta, campo in errore), `fine <|endoftext|>` (messaggio, testo resta), rimozione con la ×, salvataggio fallito (errore con Riprova, lista ripristinata; Riprova riuscito), ricaricamento con la lista conservata. Le impostazioni reali dell'utente non sono state toccate.
- Resta da provare nell'app reale: il riavvio con `settings.json` vero, l'incolla dagli appunti di Windows (provato con un `ClipboardEvent` sintetico) e la lettura con un lettore di schermo.

**2026-10-08 — code review (branch `feat/vocabolario-review`).** In en/fr/es/pl il Termine ha la maiuscola come gli altri termini del glossario ("Term", "Terme", "Término", "Termin"; in tedesco "Begriff" lo era già) in `invalidTermine`, `label` e `whisper`; "termini tecnici" del segnaposto e della descrizione restano nomi comuni. `settings.vocabolario.nonAmmesso` spiega ora la regola con le parole di `errors.invalidTermine` ("…che il prompt di Whisper non ammette") nelle sei lingue. La × delle pillole usa `Button` di `components/ui` (`variant="ghost"`, `size="icon-xs"`), con lo stesso `aria-label`.
