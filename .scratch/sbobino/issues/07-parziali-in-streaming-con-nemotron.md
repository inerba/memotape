# 07: Parziali in streaming con Nemotron

**What to build:** con Nemotron il testo della Frase compare mentre la Frase è in corso, come Parziale, e alla chiusura viene sostituito dal testo definitivo.

**Blocked by:** 06 (Impostazioni persistenti, scelta del modello e Lingua del parlato)

**Status:** done

- [x] L'implementazione Nemotron di `TranscriptionEngine` usa l'API stream (`feed` / `text` / `finalize`), con uno stream per Frase chiuso alla fine della Frase
- [x] `transcript-partial { phrase_id, text }` e `transcript-phrase { phrase_id, text }` condividono l'id. La UI aggiorna la riga in corso e la fissa all'arrivo della Frase
- [x] `Busy` diventa un errore riprovabile del trait (`EngineError::Busy`, che arriva al frontend come `modelInUse` senza scartare il motore)
- [x] La Lingua del parlato (opzionale) e la callback dei Parziali passano per `TranscriptionEngine`, non più da `TranscribeCpp::prepare`
- [x] Test: i Parziali arrivano prima della Frase con lo stesso id (motore finto). Lo smoke test `#[ignore]` copre lo stream reale
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
