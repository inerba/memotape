//! Diarizzazione: dai turni di Sortformer (chi parla quando) al Parlante di ogni Frase. Senza Tauri.

use crate::transcript::{Ingresso, Phrase};

/// Un tratto in cui parla `parlante` (l'id di Sortformer), `fine_ms` esclusa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turn {
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub parlante: u32,
}

/// Il Parlante di ogni Frase (`(inizio_ms, fine_ms)`, in ordine di inizio): quello che si sovrappone
/// di più alla Frase, nessuno se nessun turno la tocca. I Parlanti si rinumerano da 1 per ordine di
/// comparsa nelle Frasi, così "Parlante 1" è chi parla per primo. I turni possono arrivare in
/// qualunque ordine.
pub fn assign(phrases: &[(u32, u32)], turns: &[Turn]) -> Vec<Option<u32>> {
    let mut numbers: Vec<u32> = Vec::new();
    phrases
        .iter()
        .map(|&(inizio, fine)| {
            let mut overlaps: Vec<(u32, u32)> = Vec::new();
            for turn in turns {
                let overlap = fine
                    .min(turn.fine_ms)
                    .saturating_sub(inizio.max(turn.inizio_ms));
                if overlap == 0 {
                    continue;
                }
                match overlaps.iter_mut().find(|(p, _)| *p == turn.parlante) {
                    Some((_, total)) => *total += overlap,
                    None => overlaps.push((turn.parlante, overlap)),
                }
            }
            // A pari sovrapposizione vince il Parlante incontrato prima.
            let (parlante, _) = overlaps
                .into_iter()
                .reduce(|best, other| if other.1 > best.1 { other } else { best })?;
            let at = numbers
                .iter()
                .position(|&p| p == parlante)
                .unwrap_or_else(|| {
                    numbers.push(parlante);
                    numbers.len() - 1
                });
            Some(u32::try_from(at).unwrap_or(u32::MAX - 1) + 1)
        })
        .collect()
}

/// Attribuisce ai Parlanti, con i turni di Sortformer sull'audio di `ingresso`, le Frasi di
/// quell'Ingresso tra `phrases` (in ordine di inizio): i Parlanti si numerano per Ingresso. Le Frasi
/// degli altri Ingressi non cambiano.
pub fn assign_ingresso(phrases: &mut [Phrase], ingresso: Ingresso, turns: &[Turn]) {
    let mut own: Vec<&mut Phrase> = phrases
        .iter_mut()
        .filter(|p| p.ingresso == ingresso)
        .collect();
    let times: Vec<_> = own.iter().map(|p| (p.inizio_ms, p.fine_ms)).collect();
    for (phrase, parlante) in own.iter_mut().zip(assign(&times, turns)) {
        phrase.parlante = parlante;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn con_gli_ingressi_separati_si_attribuiscono_solo_le_frasi_dell_ingresso_diarizzato() {
        let phrase = |inizio_ms, ingresso| Phrase {
            inizio_ms,
            fine_ms: inizio_ms + 1000,
            text: String::new(),
            ingresso,
            parlante: None,
        };
        let mut phrases = [
            phrase(0, Ingresso::Microfono),
            phrase(1000, Ingresso::Sistema),
            phrase(2000, Ingresso::Microfono),
            phrase(3000, Ingresso::Sistema),
            phrase(4000, Ingresso::Sistema),
        ];
        // I turni dell'audio di sistema; quelli che toccano le Frasi del microfono non contano.
        let turns = [turn(0, 2000, 5), turn(2000, 4000, 9), turn(4000, 5000, 5)];
        assign_ingresso(&mut phrases, Ingresso::Sistema, &turns);
        let parlanti: Vec<_> = phrases.iter().map(|p| p.parlante).collect();
        // Il 5 compare per primo tra le Frasi dell'audio di sistema: è il Parlante 1.
        assert_eq!(parlanti, [None, Some(1), None, Some(2), Some(1)]);
    }

    fn turn(inizio_ms: u32, fine_ms: u32, parlante: u32) -> Turn {
        Turn {
            inizio_ms,
            fine_ms,
            parlante,
        }
    }

    #[test]
    fn ogni_frase_va_al_parlante_che_la_copre_di_piu() {
        // Sortformer dà i turni per parlante, non per tempo.
        let turns = [turn(0, 1200, 7), turn(3000, 4000, 7), turn(1000, 3100, 3)];
        let phrases = [(0, 1000), (900, 3000), (2900, 4000)];
        // La seconda Frase tocca il 7 per 300 ms e il 3 per 2000; la terza il 3 per 200 e il 7 per
        // 1000.
        assert_eq!(assign(&phrases, &turns), [Some(1), Some(2), Some(1)]);
    }

    #[test]
    fn i_parlanti_si_numerano_per_ordine_di_comparsa_nelle_frasi() {
        let turns = [turn(0, 1000, 2), turn(1000, 2000, 0), turn(2000, 3000, 1)];
        let phrases = [(2000, 3000), (0, 1000), (1000, 2000)];
        assert_eq!(assign(&phrases, &turns), [Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn una_frase_senza_turni_non_ha_parlante() {
        let turns = [turn(0, 1000, 1), turn(5000, 6000, 2)];
        // La seconda Frase tocca il primo turno solo al bordo (fine esclusa).
        let phrases = [(2000, 3000), (1000, 2000), (5500, 5600)];
        assert_eq!(assign(&phrases, &turns), [None, None, Some(1)]);
        assert_eq!(assign(&phrases, &[]), [None, None, None]);
    }

    /// Smoke test con Silero, Nemotron e Sortformer veri, in `%APPDATA%\it.sbobino.desktop\models`,
    /// su due voci di sintesi che si alternano (Elsa, Cosimo, Elsa, Cosimo).
    #[test]
    #[ignore = "richiede Nemotron e Sortformer scaricati (Impostazioni → Trascrizione)"]
    fn sortformer_trova_due_parlanti_che_si_alternano() {
        use std::path::{Path, PathBuf};

        use transcribe_cpp::CancelToken;

        use crate::engine::pipeline::{PipelineEvent, transcribe_file};
        use crate::engine::transcribe_cpp::{Sortformer, TranscribeCpp};
        use crate::managers::models;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir =
            PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.sbobino.desktop/models");
        let mut detector =
            crate::audio_toolkit::vad::Silero::new(&root.join("resources/silero_vad.onnx"))
                .unwrap();
        let mut engine = TranscribeCpp::load(&models::default_model().path(&dir)).unwrap();
        let cancel = CancelToken::new();
        let (mut audio, mut phrases) = (Vec::new(), Vec::new());
        transcribe_file(
            &root.join("tests/fixtures/parlato-due-voci.wav"),
            &mut engine,
            &mut detector,
            Some("it"),
            Some(&mut audio),
            None,
            &cancel,
            &mut |event| {
                if let PipelineEvent::Phrase {
                    inizio_ms,
                    fine_ms,
                    text,
                    ..
                } = event
                {
                    phrases.push((inizio_ms, fine_ms, text));
                }
            },
        )
        .unwrap();
        let turns = Sortformer::load(&models::diarizer().path(&dir))
            .unwrap()
            .diarize(&audio, &cancel)
            .unwrap();
        println!("turni: {turns:?}");
        assert!(turns.is_sorted_by_key(|t| t.inizio_ms));
        let times: Vec<_> = phrases.iter().map(|(i, f, _)| (*i, *f)).collect();
        let speakers = assign(&times, &turns);
        for ((inizio, fine, text), parlante) in phrases.iter().zip(&speakers) {
            println!("{inizio}–{fine} {parlante:?}: {text}");
        }
        let mut found: Vec<_> = speakers.iter().flatten().collect();
        found.sort_unstable();
        found.dedup();
        assert_eq!(found, [&1, &2], "{speakers:?}");
        // La prima Frase è di Elsa, quindi Parlante 1; l'ultima di Cosimo.
        assert_eq!(speakers.first(), Some(&Some(1)));
        assert_eq!(speakers.last(), Some(&Some(2)));
    }

    #[test]
    fn i_tratti_dello_stesso_parlante_si_sommano() {
        // Il 2 ha due tratti da 400 ms dentro la Frase, l'1 uno da 600.
        let turns = [turn(0, 400, 2), turn(400, 1000, 1), turn(1000, 1400, 2)];
        assert_eq!(assign(&[(0, 1400)], &turns), [Some(1)]);
        assert_eq!(
            assign(&[(400, 1000), (0, 1400)], &turns),
            [Some(1), Some(2)]
        );
    }
}
