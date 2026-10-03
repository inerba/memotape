# 06: Impostazioni persistenti, scelta del modello e Lingua del parlato

**What to build:** l'utente sceglie quale modello usare e la Lingua del parlato, con un selettore accanto a Trascrivi, e ritrova le scelte al riavvio. Whisper e Parakeet trascrivono a fine Frase. Se il modello scelto non è scaricato, Trascrivi mostra un errore con il link alle Impostazioni.

**Blocked by:** 04 (Controllo della Trascrizione), 05 (Catalogo e download dei modelli)

**Status:** done

- [x] File impostazioni JSON in `app_data_dir`, letto e validato da Rust all'avvio, con tutti i campi della spec ("Dati")
  - se il file manca o è corrotto si usano i valori predefiniti, senza bloccare l'avvio;
  - il frontend legge le impostazioni prima del primo render.
- [x] Il form delle impostazioni usa react-hook-form + zod 4
- [x] Selezione del modello, Nemotron di default. Whisper e Parakeet funzionano tramite `TranscriptionEngine` in modalità `run`
- [x] Il selettore della Lingua del parlato offre "Automatica" e le lingue tra it, en, fr, es, de e pl supportate dal modello (capability a runtime; per Nemotron "it" diventa `it-IT`). La scelta resta salvata
- [x] Senza il modello scaricato, Trascrivi mostra un errore dedicato con il link a Impostazioni → Trascrizione
- [x] "Elimina" è disabilitato per il modello usato da una Trascrizione in corso
- [x] Il modello scelto resta caricato tra una Trascrizione e l'altra: si ricarica solo se cambia la selezione o se il modello viene eliminato (richiesta aggiunta: prima si ricaricava a ogni Trascrizione, e il secondo avvio superava i 30 s)
- [x] Test: predefiniti, file corrotto, salvataggio e rilettura (Rust); schema zod e filtro delle lingue (`bun test`)
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
