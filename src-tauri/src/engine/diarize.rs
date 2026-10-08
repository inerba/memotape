//! Diarizzazione: dai turni (chi parla quando) al Parlante di ogni Frase. Senza Tauri.

use crate::transcript::{Ingresso, Phrase};

/// Un tratto in cui parla `parlante` (l'id del modello), `fine_ms` esclusa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turn {
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub parlante: u32,
}

/// Consolida solo turni già calcolati, senza ASR o esecuzione del diarizer.
/// Nessuna rettifica si applica dopo Annulla o un guasto. Senza analisi non c'è un esito.
pub fn finalize(
    phrases: &mut Vec<Phrase>,
    modello: crate::managers::settings::Diarizer,
    cancel: &transcribe_cpp::CancelToken,
    result: Result<Vec<(Ingresso, Vec<Turn>)>, crate::error::AppError>,
) -> Option<crate::transcript::Diarizzazione> {
    use crate::error::AppError;
    use crate::transcript::{Diarizzazione, EsitoDiarizzazione};

    if result.as_ref().is_ok_and(Vec::is_empty) {
        return None;
    }
    let result = if cancel.is_cancelled() {
        Err(AppError::Cancelled)
    } else {
        result
    };
    let esito = match result {
        Ok(turns) => {
            for (ingresso, found) in turns {
                assign_configured(phrases, ingresso, &found, modello);
                for phrase in phrases.iter_mut().filter(|p| p.ingresso == ingresso) {
                    phrase.parlante_provvisorio = false;
                }
            }
            EsitoDiarizzazione::Completata
        }
        Err(AppError::Cancelled) => EsitoDiarizzazione::Annullata,
        Err(error) => {
            log::warn!("Diarizzazione finale non completata: {error}");
            EsitoDiarizzazione::Fallita
        }
    };
    Some(Diarizzazione {
        modello,
        esito,
        ingressi: Vec::new(),
    })
}

/// Ogni Ingresso si consolida per conto suo: un errore non scarta il risultato dell'altro.
/// L'analisi nativa è già conclusa; ogni risultato include l'eventuale Annulla di quell'Ingresso.
/// Un Annulla successivo non invalida un altro Ingresso già analizzato. Nessuna nuova ASR.
pub fn finalize_live_ingressi(
    transcript: &mut crate::transcript::Transcript,
    modello: crate::managers::settings::Diarizer,
    results: Vec<(Ingresso, Result<Vec<Turn>, crate::error::AppError>)>,
) {
    use crate::transcript::{Diarizzazione, DiarizzazioneIngresso, EsitoDiarizzazione};
    let mut states = Vec::new();
    for (ingresso, result) in results {
        let mut phrases: Vec<_> = transcript
            .phrases
            .iter()
            .filter(|p| p.ingresso == ingresso)
            .cloned()
            .collect();
        let state = finalize(
            &mut phrases,
            modello,
            &transcribe_cpp::CancelToken::new(),
            result.map(|turns| vec![(ingresso, turns)]),
        )
        .expect("un Ingresso analizzato ha sempre un esito");
        if state.esito == EsitoDiarizzazione::Completata {
            transcript.phrases.retain(|p| p.ingresso != ingresso);
            for phrase in phrases {
                transcript.insert(phrase);
            }
        }
        states.push(DiarizzazioneIngresso {
            ingresso,
            esito: state.esito,
        });
    }
    transcript.diarizzazione = (!states.is_empty()).then(|| {
        let esito = if states
            .iter()
            .any(|s| s.esito == EsitoDiarizzazione::Fallita)
        {
            EsitoDiarizzazione::Fallita
        } else if states
            .iter()
            .any(|s| s.esito == EsitoDiarizzazione::Annullata)
        {
            EsitoDiarizzazione::Annullata
        } else {
            EsitoDiarizzazione::Completata
        };
        Diarizzazione {
            modello,
            esito,
            ingressi: states,
        }
    });
}

/// Applica le attribuzioni del modello scelto all'Ingresso. Nemotron può dividere le Frasi
/// ai confini ASR e riordinare il risultato per inizio; gli altri Ingressi conservano il testo.
pub fn assign_configured(
    phrases: &mut Vec<Phrase>,
    ingresso: Ingresso,
    turns: &[Turn],
    modello: crate::managers::settings::Diarizer,
) {
    if modello == crate::managers::settings::Diarizer::Nemotron3 {
        assign_ingresso_unambiguous(phrases, ingresso, turns);
        *phrases = std::mem::take(phrases)
            .into_iter()
            .flat_map(|phrase| {
                if phrase.ingresso == ingresso {
                    divide(phrase, turns, None)
                } else {
                    vec![phrase]
                }
            })
            .collect();
        phrases.sort_by_key(|p| p.inizio_ms);
    } else {
        assign_ingresso(phrases, ingresso, turns);
    }
}

/// Ogni parola o segmento resta indivisibile. Un intervallo ambiguo conserva il proprio testo.
pub fn divide(mut phrase: Phrase, turns: &[Turn], horizon: Option<u32>) -> Vec<Phrase> {
    phrase.parlante = if horizon.is_some_and(|h| phrase.fine_ms > h) {
        None
    } else {
        assign_unambiguous(&[(phrase.inizio_ms, phrase.fine_ms)], turns)[0]
    };
    phrase.parlante_non_determinato = phrase.parlante.is_none();
    let mut cursor = 0;
    let mut end = phrase.inizio_ms;
    for span in &phrase.tempi {
        if span.inizio_byte != cursor
            || span.fine_byte <= cursor
            || !phrase.text.is_char_boundary(span.inizio_byte)
            || !phrase.text.is_char_boundary(span.fine_byte)
            || span.inizio_ms < end
            || span.fine_ms < span.inizio_ms
            || span.fine_ms > phrase.fine_ms
        {
            return vec![phrase];
        }
        cursor = span.fine_byte;
        end = span.fine_ms;
    }
    if phrase.tempi.is_empty() || cursor != phrase.text.len() {
        return vec![phrase];
    }
    let times: Vec<_> = phrase
        .tempi
        .iter()
        .map(|s| (s.inizio_ms, s.fine_ms))
        .collect();
    let assignments = assign_unambiguous(&times, turns)
        .into_iter()
        .zip(times)
        .map(|(speaker, (_, end))| {
            if horizon.is_some_and(|h| end > h) {
                None
            } else {
                speaker
            }
        });
    let mut groups: Vec<(usize, usize, Option<u32>)> = Vec::new();
    for (at, speaker) in assignments.into_iter().enumerate() {
        if let Some((_, last, previous)) = groups.last_mut()
            && *previous == speaker
        {
            *last = at;
        } else {
            groups.push((at, at, speaker));
        }
    }
    if groups.len() == 1 {
        phrase.parlante = groups[0].2;
        phrase.parlante_non_determinato = phrase.parlante.is_none();
        return vec![phrase];
    }
    if groups
        .iter()
        .any(|&(first, last, _)| phrase.tempi[first].inizio_ms >= phrase.tempi[last].fine_ms)
    {
        return vec![phrase];
    }
    groups
        .into_iter()
        .map(|(first, last, speaker)| {
            let from = phrase.tempi[first].inizio_byte;
            let to = phrase.tempi[last].fine_byte;
            let mut part = phrase.clone();
            part.text = phrase.text[from..to].to_string();
            part.inizio_ms = phrase.tempi[first].inizio_ms;
            part.fine_ms = phrase.tempi[last].fine_ms;
            part.parlante = speaker;
            part.parlante_non_determinato = speaker.is_none();
            part.tempi = phrase.tempi[first..=last]
                .iter()
                .cloned()
                .map(|mut span| {
                    span.inizio_byte -= from;
                    span.fine_byte -= from;
                    span
                })
                .collect();
            part
        })
        .collect()
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
    assign_to_ingresso(phrases, ingresso, turns, false);
}

/// Senza tempi delle parole non si sceglie la voce maggioritaria: una Frase toccata da più
/// Parlanti resta non determinata. I numeri seguono la comparsa nell'audio, anche nei tratti ambigui.
pub fn assign_unambiguous(phrases: &[(u32, u32)], turns: &[Turn]) -> Vec<Option<u32>> {
    let mut ordered: Vec<_> = turns.iter().filter(|t| t.fine_ms > t.inizio_ms).collect();
    ordered.sort_by_key(|t| (t.inizio_ms, t.parlante));
    let mut numbers = Vec::new();
    for turn in &ordered {
        if !numbers.contains(&turn.parlante) {
            numbers.push(turn.parlante);
        }
    }
    phrases
        .iter()
        .map(|&(inizio, fine)| {
            let mut found = None;
            for turn in &ordered {
                if fine.min(turn.fine_ms) <= inizio.max(turn.inizio_ms) {
                    continue;
                }
                if found.is_some_and(|speaker| speaker != turn.parlante) {
                    return None;
                }
                found = Some(turn.parlante);
            }
            found
                .and_then(|speaker| numbers.iter().position(|&n| n == speaker))
                .and_then(|at| u32::try_from(at + 1).ok())
        })
        .collect()
}

pub fn assign_ingresso_unambiguous(phrases: &mut [Phrase], ingresso: Ingresso, turns: &[Turn]) {
    assign_to_ingresso(phrases, ingresso, turns, true);
}

fn assign_to_ingresso(
    phrases: &mut [Phrase],
    ingresso: Ingresso,
    turns: &[Turn],
    unambiguous: bool,
) {
    let mut own: Vec<_> = phrases
        .iter_mut()
        .filter(|p| p.ingresso == ingresso)
        .collect();
    let times: Vec<_> = own.iter().map(|p| (p.inizio_ms, p.fine_ms)).collect();
    let assignments = if unambiguous {
        assign_unambiguous(&times, turns)
    } else {
        assign(&times, turns)
    };
    for (phrase, parlante) in own.iter_mut().zip(assignments) {
        phrase.parlante = parlante;
        phrase.parlante_non_determinato = unambiguous && parlante.is_none();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timed_phrase(text: &str, rows: &[(i64, i64, &str)]) -> Phrase {
        let result = crate::engine::asr::AsrResult::timed(
            text.into(),
            rows.iter().map(|&(s, e, text)| (s, e, text.into())),
        );
        Phrase {
            inizio_ms: 0,
            fine_ms: 3000,
            text: result.text,
            ingresso: Ingresso::Mix,
            parlante: None,
            parlante_non_determinato: false,
            parlante_provvisorio: false,
            tempi: result.tempi,
        }
    }

    #[test]
    fn le_voci_sovrapposte_restano_non_determinate_solo_nel_tratto_ambiguo() {
        let mut phrases = vec![timed_phrase(
            "Uno. Insieme! Due.",
            &[
                (100, 500, "Uno."),
                (1100, 1400, "Insieme!"),
                (2100, 2500, "Due."),
            ],
        )];
        assign_configured(
            &mut phrases,
            Ingresso::Mix,
            &[turn(0, 1500, 7), turn(1000, 3000, 3)],
            crate::managers::settings::Diarizer::Nemotron3,
        );
        assert_eq!(
            phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
            "Uno. Insieme! Due."
        );
        assert_eq!(
            phrases.iter().map(|p| p.parlante).collect::<Vec<_>>(),
            [Some(1), None, Some(2)]
        );
        assert!(phrases[1].parlante_non_determinato);
        assert!(!phrases[0].parlante_non_determinato);
        assert!(!phrases[2].parlante_non_determinato);
    }

    #[test]
    fn un_segmento_whisper_con_due_voci_non_si_divide_in_parole_inventate() {
        let mut phrases = vec![timed_phrase(
            "Uno, due. Tre!",
            &[(0, 1900, "Uno, due."), (2100, 2600, "Tre!")],
        )];
        assign_configured(
            &mut phrases,
            Ingresso::Mix,
            &[turn(0, 1000, 7), turn(1000, 3000, 3)],
            crate::managers::settings::Diarizer::Nemotron3,
        );
        assert_eq!(phrases.len(), 2);
        assert_eq!(phrases[0].text, "Uno, due. ");
        assert!(phrases[0].parlante_non_determinato);
        assert_eq!(phrases[1].parlante, Some(2));
    }

    #[test]
    fn senza_tempi_affidabili_o_con_offset_utf8_invalidi_il_testo_resta_integro() {
        let original = timed_phrase("Perché sì!", &[]);
        for invalid in [
            Vec::new(),
            vec![crate::transcript::TempoTesto {
                inizio_byte: 0,
                fine_byte: 6,
                inizio_ms: 100,
                fine_ms: 500,
            }],
            vec![crate::transcript::TempoTesto {
                inizio_byte: 0,
                fine_byte: 12,
                inizio_ms: 100,
                fine_ms: 0,
            }],
        ] {
            let mut phrase = original.clone();
            phrase.tempi = invalid;
            let mut phrases = vec![phrase];
            assign_configured(
                &mut phrases,
                Ingresso::Mix,
                &[turn(0, 1000, 7), turn(1000, 3000, 3)],
                crate::managers::settings::Diarizer::Nemotron3,
            );
            assert_eq!(phrases.len(), 1);
            assert_eq!(phrases[0].text, "Perché sì!");
            assert!(phrases[0].parlante_non_determinato);
        }
    }

    #[test]
    fn sortformer_conserva_la_regola_esistente_anche_con_tempi_asr() {
        let mut phrases = vec![timed_phrase(
            "Uno. Due.",
            &[(100, 500, "Uno."), (2100, 2500, "Due.")],
        )];
        assign_configured(
            &mut phrases,
            Ingresso::Mix,
            &[turn(0, 1000, 7), turn(1000, 3000, 3)],
            crate::managers::settings::Diarizer::Sortformer,
        );
        assert_eq!(phrases.len(), 1);
        assert_eq!(phrases[0].parlante, Some(1));
        assert_eq!(phrases[0].text, "Uno. Due.");
    }

    #[test]
    fn divide_al_cambio_di_voce_conservando_testo_accenti_e_punteggiatura() {
        use crate::transcript::TempoTesto;
        let mut phrases = vec![Phrase {
            inizio_ms: 5000,
            fine_ms: 7000,
            text: "Perché sì! D'accordo?".into(),
            ingresso: Ingresso::Sistema,
            parlante: None,
            parlante_non_determinato: false,
            parlante_provvisorio: true,
            tempi: vec![
                TempoTesto {
                    inizio_byte: 0,
                    fine_byte: 8,
                    inizio_ms: 5100,
                    fine_ms: 5400,
                },
                TempoTesto {
                    inizio_byte: 8,
                    fine_byte: 13,
                    inizio_ms: 5500,
                    fine_ms: 5700,
                },
                TempoTesto {
                    inizio_byte: 13,
                    fine_byte: 23,
                    inizio_ms: 6100,
                    fine_ms: 6500,
                },
            ],
        }];
        assign_configured(
            &mut phrases,
            Ingresso::Sistema,
            &[turn(5000, 5900, 7), turn(5900, 7000, 3)],
            crate::managers::settings::Diarizer::Nemotron3,
        );
        assert_eq!(phrases.len(), 2);
        assert_eq!(
            phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
            "Perché sì! D'accordo?"
        );
        assert_eq!(
            phrases.iter().map(|p| p.parlante).collect::<Vec<_>>(),
            [Some(1), Some(2)]
        );
        assert_eq!((phrases[0].inizio_ms, phrases[0].fine_ms), (5100, 5700));
        assert_eq!((phrases[1].inizio_ms, phrases[1].fine_ms), (6100, 6500));
    }

    #[test]
    fn analisi_finale_annullata_o_guasta_conserva_il_testo_e_le_attribuzioni_disponibili() {
        use crate::error::AppError;
        use crate::managers::settings::Diarizer;
        use crate::transcript::EsitoDiarizzazione;
        use transcribe_cpp::CancelToken;
        let original = vec![Phrase {
            inizio_ms: 0,
            fine_ms: 1000,
            text: "Testo, senza modifiche!".into(),
            ingresso: Ingresso::Mix,
            parlante: Some(2),
            parlante_non_determinato: false,
            parlante_provvisorio: true,
            tempi: Vec::new(),
        }];
        for (error, expected) in [
            (AppError::Cancelled, EsitoDiarizzazione::Annullata),
            (
                AppError::Internal("guasto".into()),
                EsitoDiarizzazione::Fallita,
            ),
        ] {
            let mut phrases = original.to_vec();
            let state = finalize(
                &mut phrases,
                Diarizer::Nemotron3,
                &CancelToken::new(),
                Err(error.clone()),
            );
            assert_eq!(state.unwrap().esito, expected);
            assert_eq!(phrases, original);
        }
        let mut phrases = original.to_vec();
        let state = finalize(
            &mut phrases,
            Diarizer::Nemotron3,
            &CancelToken::new(),
            Ok(vec![(Ingresso::Mix, vec![turn(0, 1000, 7)])]),
        );
        assert_eq!(state.unwrap().esito, EsitoDiarizzazione::Completata);
        assert_eq!(phrases[0].text, original[0].text);
        assert_eq!(phrases[0].parlante, Some(1));
        assert!(!phrases[0].parlante_provvisorio);
    }

    #[test]
    fn annulla_o_guasto_non_trasformano_una_frase_senza_attribuzione() {
        let original = [Phrase {
            inizio_ms: 0,
            fine_ms: 1000,
            text: "Testo senza attribuzione.".into(),
            ingresso: Ingresso::Mix,
            parlante: None,
            parlante_non_determinato: false,
            parlante_provvisorio: false,
            tempi: Vec::new(),
        }];
        for error in [
            crate::error::AppError::Cancelled,
            crate::error::AppError::Internal("modello assente".into()),
        ] {
            let mut phrases = original.to_vec();
            finalize(
                &mut phrases,
                crate::managers::settings::Diarizer::Nemotron3,
                &transcribe_cpp::CancelToken::new(),
                Err(error.clone()),
            );
            assert_eq!(phrases, original);
        }
    }

    #[test]
    fn annulla_dopo_l_ultimo_ingresso_non_consolida_attribuzioni_ne_perde_identita() {
        use crate::managers::settings::Diarizer;
        use crate::transcript::EsitoDiarizzazione;
        let cancel = transcribe_cpp::CancelToken::new();
        let mut phrases = vec![Phrase {
            inizio_ms: 0,
            fine_ms: 1000,
            text: "Voce incerta.".into(),
            ingresso: Ingresso::Sistema,
            parlante: None,
            parlante_non_determinato: true,
            parlante_provvisorio: false,
            tempi: Vec::new(),
        }];
        let original = phrases.clone();
        cancel.cancel();
        let state = finalize(
            &mut phrases,
            Diarizer::Nemotron3,
            &cancel,
            Ok(vec![(Ingresso::Sistema, vec![turn(0, 1000, 7)])]),
        );
        assert_eq!(state.unwrap().esito, EsitoDiarizzazione::Annullata);
        assert_eq!(phrases, original);
    }

    #[test]
    fn guasto_del_secondo_ingresso_non_applica_rettifiche_parziali() {
        use crate::managers::settings::Diarizer;
        use crate::transcript::EsitoDiarizzazione;
        let phrase = |ingresso| Phrase {
            inizio_ms: 0,
            fine_ms: 1000,
            text: "Testo.".into(),
            ingresso,
            parlante: Some(2),
            parlante_non_determinato: false,
            parlante_provvisorio: true,
            tempi: Vec::new(),
        };
        let mut phrases = vec![phrase(Ingresso::Microfono), phrase(Ingresso::Sistema)];
        let original = phrases.clone();
        let state = finalize(
            &mut phrases,
            Diarizer::Nemotron3,
            &transcribe_cpp::CancelToken::new(),
            Err(crate::error::AppError::Internal(
                "guasto del secondo Ingresso".into(),
            )),
        );
        assert_eq!(state.unwrap().esito, EsitoDiarizzazione::Fallita);
        assert_eq!(phrases, original);
    }

    #[test]
    fn nessuna_frase_non_dichiara_un_analisi_completata_e_conserva_il_guasto() {
        use crate::error::AppError;
        use crate::managers::settings::Diarizer;
        use crate::transcript::EsitoDiarizzazione;
        let cancel = transcribe_cpp::CancelToken::new();
        assert_eq!(
            finalize(&mut Vec::new(), Diarizer::Nemotron3, &cancel, Ok(vec![])),
            None
        );
        cancel.cancel();
        assert_eq!(
            finalize(&mut Vec::new(), Diarizer::Nemotron3, &cancel, Ok(vec![])),
            None
        );
        let state = finalize(
            &mut Vec::new(),
            Diarizer::Nemotron3,
            &transcribe_cpp::CancelToken::new(),
            Err(AppError::LocalDiarizerMissing),
        );
        assert_eq!(state.unwrap().esito, EsitoDiarizzazione::Fallita);
    }

    #[test]
    fn nemotron_non_inventa_un_parlante_per_una_frase_con_due_voci() {
        let turns = [turn(0, 900, 7), turn(900, 2000, 3), turn(2000, 3000, 7)];
        assert_eq!(
            assign_unambiguous(&[(0, 800), (800, 2100), (2200, 3000)], &turns),
            [Some(1), None, Some(1)]
        );
    }

    #[test]
    fn otto_voci_in_ordine_di_comparsa_e_identita_distinte_per_ingresso() {
        let turns: Vec<_> = (0..8)
            .rev()
            .map(|n| turn(n * 1000, n * 1000 + 900, 7 - n))
            .collect();
        let times: Vec<_> = (0..8).map(|n| (n * 1000, n * 1000 + 900)).collect();
        assert_eq!(
            assign_unambiguous(&times, &turns),
            (1..=8).map(Some).collect::<Vec<_>>()
        );
        let mut phrases = vec![
            Phrase {
                inizio_ms: 0,
                fine_ms: 900,
                text: "Microfono".into(),
                ingresso: Ingresso::Microfono,
                parlante: None,
                parlante_non_determinato: false,
                parlante_provvisorio: false,
                tempi: Vec::new(),
            },
            Phrase {
                inizio_ms: 0,
                fine_ms: 900,
                text: "Sistema".into(),
                ingresso: Ingresso::Sistema,
                parlante: None,
                parlante_non_determinato: false,
                parlante_provvisorio: false,
                tempi: Vec::new(),
            },
        ];
        assign_ingresso_unambiguous(&mut phrases, Ingresso::Sistema, &turns);
        assert_eq!(phrases[0].parlante, None);
        assert!(!phrases[0].parlante_non_determinato);
        assert_eq!(phrases[1].parlante, Some(1));
        assign_ingresso_unambiguous(&mut phrases, Ingresso::Microfono, &[turn(0, 900, 3)]);
        assert_eq!(phrases[0].parlante, Some(1));
    }

    #[test]
    fn sovrapposizioni_silenzio_e_bordi_non_inventano_identita() {
        let turns = [
            turn(0, 1000, 3),
            turn(500, 1500, 8),
            turn(2000, 3000, 3),
            turn(2100, 2900, 3),
        ];
        assert_eq!(
            assign_unambiguous(&[(0, 400), (500, 1000), (1500, 2000), (2000, 3000)], &turns),
            [Some(1), None, None, Some(1)]
        );
    }

    #[test]
    fn con_gli_ingressi_separati_si_attribuiscono_solo_le_frasi_dell_ingresso_diarizzato() {
        let phrase = |inizio_ms, ingresso| Phrase {
            inizio_ms,
            fine_ms: inizio_ms + 1000,
            text: String::new(),
            ingresso,
            parlante: None,
            parlante_non_determinato: false,
            parlante_provvisorio: false,
            tempi: Vec::new(),
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

    /// Smoke test con Silero, Nemotron e Sortformer veri, in `%APPDATA%\it.memotape.desktop\models`,
    /// su due voci di sintesi che si alternano (Elsa, Cosimo, Elsa, Cosimo).
    #[test]
    #[ignore = "richiede Nemotron e Sortformer scaricati (Impostazioni → Trascrizione)"]
    fn sortformer_trova_due_parlanti_che_si_alternano() {
        use std::path::{Path, PathBuf};

        use transcribe_cpp::CancelToken;

        use crate::engine::pipeline::{PipelineEvent, transcribe_file};
        use crate::engine::transcribe_cpp::{OfflineDiarizer, TranscribeCpp};
        use crate::managers::models;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir =
            PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
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
        let turns = OfflineDiarizer::load_sortformer(&models::diarizer().path(&dir))
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
