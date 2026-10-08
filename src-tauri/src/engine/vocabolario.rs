//! Correzione approssimativa con i Termini del Vocabolario, senza Tauri: l'algoritmo
//! `apply_custom_words` di Handy (`cjpais/Handy`, `src-tauri/src/audio_toolkit/text.rs`), con gli
//! adattamenti della spec del Vocabolario.

use super::asr::AsrResult;
use crate::transcript::TempoTesto;

/// Punteggio massimo (escluso) per sostituire: distanza normalizzata, ×0,3 con lo stesso Soundex.
const SOGLIA: f64 = 0.18;

/// Il testo con i tratti simili a un Termine sostituiti dal Termine.
pub fn correct(mut result: AsrResult, termini: &[String]) -> AsrResult {
    // I Termini in altre scritture agiscono solo nel prompt di Whisper.
    let keys: Vec<(&str, String)> = termini
        .iter()
        .map(|t| (t.as_str(), key(t)))
        .filter(|(_, k)| latin(k))
        .collect();
    let words = words(&result.text);
    // Tratti da sostituire: byte di inizio e fine, sostituto.
    let mut swaps: Vec<(usize, usize, String)> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let mut best: Option<(usize, Match)> = None;
        for n in (1..=3).filter(|n| i + n <= words.len()) {
            let ngram = &words[i..i + n];
            // Non si attraversa la punteggiatura: chiude l'n-gram in coda a una parola interna,
            // lo apre in testa a una parola dopo la prima.
            if ngram.iter().any(|w| w.key.is_empty())
                || ngram[..n - 1].iter().any(|w| w.punct_after)
                || ngram[1..].iter().any(|w| w.punct_before)
            {
                break;
            }
            if !fits_tempi(&result, ngram[0].start, ngram[n - 1].end) {
                break;
            }
            let candidate: String = ngram.iter().map(|w| w.key.as_str()).collect();
            // Una parola in più vince solo se avvicina davvero al Termine: con la distanza divisa
            // per la lunghezza maggiore, "charge B per" (2 su 10) batterebbe "charge B" (2 su 9)
            // e la sostituzione mangerebbe "per".
            if let Some(found) = best_match(&candidate, &keys)
                && best.is_none_or(|(_, b)| found.score < b.score && found.distance < b.distance)
            {
                best = Some((n, found));
            }
        }
        let Some((n, found)) = best else {
            i += 1;
            continue;
        };
        // Lo stesso vale per la prima parola: in "a chargebee" la "a" resta.
        let rest: String = words[i + 1..i + n].iter().map(|w| w.key.as_str()).collect();
        if n > 1 && strsim::levenshtein(&rest, found.key) <= found.distance {
            i += 1;
            continue;
        }
        let first = &words[i];
        let original = &result.text[first.start..first.end];
        swaps.push((
            first.start,
            words[i + n - 1].end,
            cased(original, found.termine),
        ));
        i += n;
    }
    // Dall'ultimo, così gli offset dei tratti precedenti valgono ancora.
    for (a, b, sostituto) in swaps.into_iter().rev() {
        result.text.replace_range(a..b, &sostituto);
        let shift = |byte: usize| byte - b + a + sostituto.len();
        // I tempi che toccano il tratto diventano uno, indivisibile; un segmento che lo contiene
        // cambia solo la fine. I successivi si spostano.
        let touched: Vec<usize> = (0..result.tempi.len())
            .filter(|&t| result.tempi[t].inizio_byte < b && result.tempi[t].fine_byte > a)
            .collect();
        for tempo in result.tempi.iter_mut().filter(|t| t.inizio_byte >= b) {
            tempo.inizio_byte = shift(tempo.inizio_byte);
            tempo.fine_byte = shift(tempo.fine_byte);
        }
        if let (Some(&first), Some(&last)) = (touched.first(), touched.last()) {
            let merged = TempoTesto {
                fine_byte: shift(result.tempi[last].fine_byte),
                fine_ms: result.tempi[last].fine_ms,
                ..result.tempi[first].clone()
            };
            result.tempi.splice(first..=last, [merged]);
        }
    }
    result
}

/// Se il tratto `a..b` si può sostituire senza fondere tempi: dentro un solo tempo, oppure ogni
/// tempo che tocca sta tutto nel tratto, salvo spazi e punteggiatura (le parole di Nemotron e
/// Parakeet, non due segmenti di Whisper attraversati).
fn fits_tempi(result: &AsrResult, a: usize, b: usize) -> bool {
    let touched: Vec<&TempoTesto> = result
        .tempi
        .iter()
        .filter(|t| t.inizio_byte < b && t.fine_byte > a)
        .collect();
    let outside = |from: usize, to: usize| {
        !result.text[from..to.max(from)]
            .chars()
            .any(char::is_alphanumeric)
    };
    touched.len() < 2
        || touched
            .iter()
            .all(|t| outside(t.inizio_byte, a) && outside(b, t.fine_byte))
}

/// Una parola del testo: byte di lettere e cifre tra la punteggiatura ai bordi, e la sua chiave.
struct Word {
    start: usize,
    end: usize,
    punct_before: bool,
    punct_after: bool,
    key: String,
}

fn words(text: &str) -> Vec<Word> {
    text.split_whitespace()
        .map(|w| {
            let at = w.as_ptr() as usize - text.as_ptr() as usize;
            let mut core = w.trim_matches(|c: char| !c.is_alphanumeric());
            // L'elisione ("l'", "dell'", "un’") resta fuori dalla parola, come la punteggiatura.
            if let Some((prefix, rest)) = core.split_once(['\'', '’'])
                && !prefix.is_empty()
                && !rest.is_empty()
                && prefix.chars().all(char::is_alphabetic)
            {
                core = rest;
            }
            let start = at + (core.as_ptr() as usize - w.as_ptr() as usize);
            Word {
                start,
                end: start + core.len(),
                punct_before: start > at,
                punct_after: start + core.len() < at + w.len(),
                key: key(core),
            }
        })
        .collect()
}

/// Il Termine con le maiuscole di `original`: tutto maiuscolo se lo è la parola (di più lettere),
/// con l'iniziale maiuscola se lo è la sua, altrimenti com'è.
fn cased(original: &str, termine: &str) -> String {
    let letters: Vec<char> = original.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() > 1 && letters.iter().all(|c| c.is_uppercase()) {
        termine.to_uppercase()
    } else if original.starts_with(char::is_uppercase) {
        let mut chars = termine.chars();
        chars
            .next()
            .map(|c| c.to_uppercase().chain(chars).collect())
            .unwrap_or_default()
    } else {
        termine.to_string()
    }
}

#[derive(Clone, Copy)]
struct Match<'a> {
    termine: &'a str,
    key: &'a str,
    score: f64,
    distance: usize,
}

fn best_match<'a>(candidate: &str, keys: &'a [(&'a str, String)]) -> Option<Match<'a>> {
    if !latin(candidate) {
        return None;
    }
    let mut best: Option<Match> = None;
    for (termine, key) in keys {
        let (lc, lk) = (candidate.chars().count(), key.chars().count());
        let max = lc.max(lk) as f64;
        // Le parole italiane brevi darebbero falsi positivi: un Termine breve vale solo identico.
        if lc.abs_diff(lk) as f64 > (max * 0.25).max(2.0) || (lk < 4 && candidate != key) {
            continue;
        }
        let distance = strsim::levenshtein(candidate, key);
        let mut score = distance as f64 / max;
        if soundex(candidate).is_some_and(|s| soundex(key) == Some(s)) {
            score *= 0.3;
        }
        if score < SOGLIA && best.is_none_or(|b| score < b.score) {
            best = Some(Match {
                termine,
                key,
                score,
                distance,
            });
        }
    }
    best
}

/// La chiave di confronto: lettere e cifre, minuscole, senza accenti.
fn key(text: &str) -> String {
    use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
    text.nfd()
        .filter(|&c| c.is_alphanumeric() && !is_combining_mark(c))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Se la chiave ha solo lettere latine (senza accenti) e cifre ASCII.
fn latin(key: &str) -> bool {
    !key.is_empty()
        && key.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '\u{00C0}'..='\u{024F}' | '\u{1E00}'..='\u{1EFF}')
        })
}

/// Soundex americano di una chiave di sole lettere ASCII.
fn soundex(key: &str) -> Option<[u8; 4]> {
    fn digit(c: u8) -> u8 {
        match c {
            b'b' | b'f' | b'p' | b'v' => b'1',
            b'c' | b'g' | b'j' | b'k' | b'q' | b's' | b'x' | b'z' => b'2',
            b'd' | b't' => b'3',
            b'l' => b'4',
            b'm' | b'n' => b'5',
            b'r' => b'6',
            _ => 0,
        }
    }
    let bytes = key.as_bytes();
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_lowercase) {
        return None;
    }
    let mut code = [bytes[0].to_ascii_uppercase(), b'0', b'0', b'0'];
    let (mut len, mut previous) = (1, digit(bytes[0]));
    for &c in &bytes[1..] {
        let d = digit(c);
        if d != 0 && d != previous {
            code[len] = d;
            len += 1;
            if len == 4 {
                break;
            }
        }
        // h e w non separano due consonanti con la stessa cifra; le vocali sì.
        if c != b'h' && c != b'w' {
            previous = d;
        }
    }
    Some(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(text: &str, parole: &[&str]) -> String {
        correct(AsrResult::from(text.to_string()), &termini(parole)).text
    }

    #[test]
    fn piu_parole_simili_diventano_un_termine() {
        assert_eq!(
            text("Usiamo charge B per gli abbonamenti", &["ChargeBee"]),
            "Usiamo ChargeBee per gli abbonamenti"
        );
    }

    #[test]
    fn gli_accenti_non_contano_nel_confronto_e_restano_nel_termine() {
        assert_eq!(
            text("Ha parlato Nicolo.", &["Niccolò"]),
            "Ha parlato Niccolò."
        );
        assert_eq!(
            text("Ha parlato Niccolò.", &["Nicolo"]),
            "Ha parlato Nicolo."
        );
    }

    #[test]
    fn il_termine_prende_le_maiuscole_della_prima_parola() {
        let termini = &["iPhone"];
        assert_eq!(text("un iphone nuovo", termini), "un iPhone nuovo");
        assert_eq!(text("Iphone nuovo", termini), "IPhone nuovo");
        assert_eq!(text("un IPHONE nuovo", termini), "un IPHONE nuovo");
        assert_eq!(text("charge B", &["chargebee"]), "chargebee");
        assert_eq!(text("Charge B", &["chargebee"]), "Chargebee");
        // Una lettera sola maiuscola non è una parola tutta maiuscola.
        assert_eq!(text("Ascolto K pop", &["kpop"]), "Ascolto Kpop");
    }

    #[test]
    fn le_parole_vicine_che_non_avvicinano_al_termine_restano() {
        assert_eq!(text("a chargebee e", &["ChargeBee"]), "a ChargeBee e");
        assert_eq!(text("e Niccolò è qui", &["Niccolò"]), "e Niccolò è qui");
    }

    #[test]
    fn la_punteggiatura_ai_bordi_resta_e_chiude_il_tratto() {
        let termini = &["ChargeBee"];
        assert_eq!(text("«Charge B», disse.", termini), "«ChargeBee», disse.");
        assert_eq!(text("Charge B, che dici?", termini), "ChargeBee, che dici?");
        assert_eq!(text("Charge, B e altro", termini), "Charge, B e altro");
        assert_eq!(text("Charge (B) e altro", termini), "Charge (B) e altro");
    }

    #[test]
    fn l_elisione_davanti_al_termine_resta() {
        let termini = &["iPhone"];
        assert_eq!(text("compro l'iphone", termini), "compro l'iPhone");
        assert_eq!(text("dell’iphone nuovo", termini), "dell’iPhone nuovo");
        assert_eq!(text("Un'iphone", termini), "Un'iPhone");
    }

    #[test]
    fn un_termine_breve_si_sostituisce_solo_se_identico() {
        let termini = &["Ada"];
        assert_eq!(text("chiedi ad ada", termini), "chiedi ad Ada");
        assert_eq!(text("lungo l'Adda", termini), "lungo l'Adda");
        assert_eq!(text("e poi dai", termini), "e poi dai");
    }

    #[test]
    fn le_scritture_non_latine_non_si_confrontano() {
        assert_eq!(text("a москва ieri", &["Москва"]), "a москва ieri");
        assert_eq!(text("a Moskva ieri", &["Москва"]), "a Moskva ieri");
    }

    fn timed(text: &str, rows: &[(i64, i64, &str)]) -> AsrResult {
        AsrResult::timed(
            text.into(),
            rows.iter().map(|&(s, e, row)| (s, e, row.into())),
        )
    }

    fn termini(termini: &[&str]) -> Vec<String> {
        termini.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn le_parole_sostituite_diventano_un_solo_tempo_e_i_successivi_si_spostano() {
        let asr = timed(
            "Con charge B, ha detto Nicolo, va.",
            &[
                (0, 200, "Con"),
                (300, 600, "charge"),
                (650, 800, "B,"),
                (900, 1000, "ha"),
                (1100, 1300, "detto"),
                (1400, 1900, "Nicolo,"),
                (2000, 2100, "va."),
            ],
        );
        let atteso = timed(
            "Con ChargeBee, ha detto Niccolò, va.",
            &[
                (0, 200, "Con"),
                (300, 800, "ChargeBee,"),
                (900, 1000, "ha"),
                (1100, 1300, "detto"),
                (1400, 1900, "Niccolò,"),
                (2000, 2100, "va."),
            ],
        );
        assert_eq!(atteso.tempi.len(), 6);
        assert_eq!(correct(asr, &termini(&["ChargeBee", "Niccolò"])), atteso);
    }

    #[test]
    fn nei_segmenti_di_whisper_la_sostituzione_resta_nel_segmento() {
        let asr = timed(
            "Ha parlato Nicolo. Poi charge B.",
            &[
                (0, 1500, "Ha parlato Nicolo."),
                (1600, 3000, "Poi charge B."),
            ],
        );
        let atteso = timed(
            "Ha parlato Niccolò. Poi ChargeBee.",
            &[
                (0, 1500, "Ha parlato Niccolò."),
                (1600, 3000, "Poi ChargeBee."),
            ],
        );
        assert_eq!(atteso.tempi.len(), 2);
        assert_eq!(correct(asr, &termini(&["ChargeBee", "Niccolò"])), atteso);
    }

    #[test]
    fn un_tratto_a_cavallo_di_due_segmenti_resta_com_e() {
        let asr = timed(
            "Poi charge B dopo.",
            &[(0, 1500, "Poi charge"), (1600, 3000, "B dopo.")],
        );
        assert_eq!(correct(asr.clone(), &termini(&["ChargeBee"])), asr);
    }

    #[test]
    fn senza_termini_il_risultato_resta_identico() {
        let asr = timed(
            "Charge B, Nicolo.",
            &[(0, 500, "Charge"), (600, 700, "B,"), (800, 900, "Nicolo.")],
        );
        assert_eq!(correct(asr.clone(), &[]), asr);
    }

    #[test]
    fn le_parole_italiane_comuni_restano() {
        let frase = "Ho caricato il nastro e la memoria del telefono, poi niente: \
                     colla, chiave, cartella e note sul tavolo.";
        assert_eq!(
            text(frase, &["ChargeBee", "Niccolò", "Memotape", "Nemotron"]),
            frase
        );
    }
}
