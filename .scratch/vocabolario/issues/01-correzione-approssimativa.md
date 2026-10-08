# 01: Correzione approssimativa del testo con i Termini

**What to build:** Il modulo puro `engine::vocabolario` (senza Tauri) che riceve un `AsrResult` (testo e `TempoTesto` con offset in byte e tempi relativi) e i Termini, e restituisce l'`AsrResult` con i tratti simili a un Termine sostituiti. Regole in `../spec.md`, sezioni "Correzione approssimativa" e "Tempi del testo"; algoritmo di riferimento `apply_custom_words` di Handy (`cjpais/Handy`, `src-tauri/src/audio_toolkit/text.rs`).

**Blocked by:** nessuno.

**Status:** ready-for-agent

- [ ] n-gram da 3 a 1 parole, migliore punteggio, niente attraversamento della punteggiatura; chiave senza accenti (`unicode-normalization`, già nel `Cargo.lock`), minuscola, solo lettere e cifre; scritture non latine escluse dal confronto.
- [ ] Levenshtein normalizzato (`strsim`, già nel `Cargo.lock`), Soundex scritto qui (poche righe, niente crate nuovo) con fattore 0,3, soglia fissa 0,18, filtro del 25% sulla lunghezza, chiave sotto i 4 caratteri solo se identica.
- [ ] Maiuscole come la prima parola originale, punteggiatura ai bordi conservata.
- [ ] `TempoTesto`: i tratti che toccano il tratto sostituito diventano uno solo (dal primo `inizio_ms` all'ultimo `fine_ms`, byte del sostituto), gli offset successivi si spostano. Un segmento di Whisper che contiene il tratto ne aggiusta solo il `fine_byte`. Il risultato deve restare allineato come lo darebbe `AsrResult::timed`.
- [ ] Senza Termini l'`AsrResult` resta identico.
- [ ] Test (con `tdd`): "Charge B" → "ChargeBee", "Nicolo" → "Niccolò", maiuscole, punteggiatura, Termine breve solo esatto, scrittura non latina ignorata, parole italiane comuni non sostituite da un Termine vicino, tempi uniti e offset spostati.
- [ ] I sei controlli passano.

## Comments
