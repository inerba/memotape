# 01: Correzione approssimativa del testo con i Termini

**What to build:** Il modulo puro `engine::vocabolario` (senza Tauri) che riceve un `AsrResult` (testo e `TempoTesto` con offset in byte e tempi relativi) e i Termini, e restituisce l'`AsrResult` con i tratti simili a un Termine sostituiti. Regole in `../spec.md`, sezioni "Correzione approssimativa" e "Tempi del testo"; algoritmo di riferimento `apply_custom_words` di Handy (`cjpais/Handy`, `src-tauri/src/audio_toolkit/text.rs`).

**Blocked by:** nessuno.

**Status:** done

- [x] n-gram da 3 a 1 parole, migliore punteggio, niente attraversamento della punteggiatura; chiave senza accenti (`unicode-normalization`, già nel `Cargo.lock`), minuscola, solo lettere e cifre; scritture non latine escluse dal confronto.
- [x] Levenshtein normalizzato (`strsim`, già nel `Cargo.lock`), Soundex scritto qui (poche righe, niente crate nuovo) con fattore 0,3, soglia fissa 0,18, filtro del 25% sulla lunghezza, chiave sotto i 4 caratteri solo se identica.
- [x] Maiuscole come la prima parola originale, punteggiatura ai bordi conservata.
- [x] `TempoTesto`: i tratti che toccano il tratto sostituito diventano uno solo (dal primo `inizio_ms` all'ultimo `fine_ms`, byte del sostituto), gli offset successivi si spostano. Un segmento di Whisper che contiene il tratto ne aggiusta solo il `fine_byte`. Il risultato deve restare allineato come lo darebbe `AsrResult::timed`.
- [x] Senza Termini l'`AsrResult` resta identico.
- [x] Test (con `tdd`): "Charge B" → "ChargeBee", "Nicolo" → "Niccolò", maiuscole, punteggiatura, Termine breve solo esatto, scrittura non latina ignorata, parole italiane comuni non sostituite da un Termine vicino, tempi uniti e offset spostati.
- [x] I sei controlli passano.

## Comments

### 2026-10-08

Fatto in `src-tauri/src/engine/vocabolario.rs`: `correct(AsrResult, &[String]) -> AsrResult`, 11 test. `strsim` e `unicode-normalization` aggiunti a `Cargo.toml` alle versioni già nel lock. Il modulo è dichiarato con `#[allow(dead_code)]` finché il ticket 03 non lo chiama.

Scelte e deviazioni da Handy:

- **Una parola in più deve avvicinare al Termine.** Con il punteggio diviso per la lunghezza maggiore, "charge B per" (distanza 2 su 10, stesso Soundex C621) batte "charge B" (2 su 9) e la sostituzione mangerebbe "per". Gli n-gram si provano da 1 a 3 e uno più lungo vince solo con punteggio **e** distanza di Levenshtein minori. Allo stesso modo la prima parola si tiene solo se senza di lei la distanza peggiora: in "a chargebee" la "a" resta.
- La punteggiatura chiude l'n-gram anche in testa a una parola dopo la prima (`Charge (B)`), non solo in coda a una interna; una parola di sola punteggiatura non entra in un n-gram.
- Si sostituisce solo il nucleo alfanumerico delle parole nel testo originale (punteggiatura e spazi intorno restano byte per byte), invece di ricomporre la Frase con spazi singoli come Handy: così gli offset dei tempi si spostano di una quantità nota.
- "Tutta maiuscola" vale solo con più di una lettera: "K pop" dà "Kpop", non "KPOP".
- Scrittura latina = cifre ASCII e lettere nei blocchi Latin-1/Latin Extended/Latin Extended Additional dopo aver tolto gli accenti (`ł`, `ß` restano latine).
- Soundex americano scritto qui, solo per chiavi di sole lettere ASCII.

Da tenere d'occhio nella prova a mano: con il Soundex ×0,3 parole italiane con lo stesso scheletro di consonanti e un Termine di almeno 4 lettere passano la soglia (per esempio "luce" → Luca, "marzo" → Marco, "vulcano" → Vulkan, "Nicola" → Niccolò). È l'algoritmo della spec, non un difetto di questa implementazione.

### 2026-10-08, code review (branch `feat/vocabolario-review`)

- Tratti a cavallo di più tempi: `fits_tempi` ammette un n-gram dentro un solo `TempoTesto`, oppure su più tempi solo se ognuno sta tutto nel tratto salvo spazi e punteggiatura (parole di Nemotron/Parakeet). Un n-gram che attraversa due segmenti di Whisper (`"Poi charge" | "B dopo"`) non si sostituisce più; il controllo è nel ciclo degli n-gram, quindi un n-gram più corto dentro un segmento resta possibile. Test `un_tratto_a_cavallo_di_due_segmenti_resta_com_e`.
- Elisione: in `words` un prefisso di lettere seguito da `'` o `’` dentro la parola resta fuori dalla parola e dalla chiave, come la punteggiatura in testa: "compro l'iphone" → "compro l'iPhone", anche "dell’", "Un'". Test `l_elisione_davanti_al_termine_resta`.
