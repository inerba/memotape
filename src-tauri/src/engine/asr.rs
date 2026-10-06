//! Testo ASR e intervalli realmente esposti dal motore, senza Tauri.

use crate::transcript::TempoTesto;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AsrResult {
    pub text: String,
    /// Tempi relativi all'audio dato al motore; ogni tratto è indivisibile.
    pub tempi: Vec<TempoTesto>,
}

impl AsrResult {
    /// Allinea righe di parola/token o segmento al testo finale, senza riformularlo.
    /// Se le righe non coprono esattamente il testo, conserva solo il testo.
    pub fn timed(text: String, rows: impl IntoIterator<Item = (i64, i64, String)>) -> Self {
        let mut result = Self::from(text);
        let mut cursor = 0;
        for (start, end, row) in rows {
            let row = row.trim();
            if row.is_empty() {
                continue;
            }
            let remaining = &result.text[cursor..];
            let gap = remaining.len() - remaining.trim_start().len();
            let at = cursor + gap;
            let times = u32::try_from(start).ok().zip(u32::try_from(end).ok());
            let Some((inizio_ms, fine_ms)) = times.filter(|(s, e)| e >= s) else {
                result.tempi.clear();
                return result;
            };
            if !result.text[at..].starts_with(row)
                || result.tempi.last().is_some_and(|p| inizio_ms < p.fine_ms)
            {
                result.tempi.clear();
                return result;
            }
            if let Some(previous) = result.tempi.last_mut() {
                previous.fine_byte = at;
            }
            result.tempi.push(TempoTesto {
                inizio_byte: if result.tempi.is_empty() { 0 } else { at },
                fine_byte: at + row.len(),
                inizio_ms,
                fine_ms,
            });
            cursor = at + row.len();
        }
        if !result.text[cursor..].trim().is_empty() {
            result.tempi.clear();
        } else if let Some(last) = result.tempi.last_mut() {
            last.fine_byte = result.text.len();
        }
        result
    }

    /// Interseca il solo padding encoder (un passo di 80 ms nei modelli fissati) con l'audio
    /// reale, poi trasla i tempi. Un intervallo esterno più lungo non è utilizzabile.
    pub fn on_source(mut self, start: u32, end: u32) -> Self {
        let duration = end.saturating_sub(start);
        if self
            .tempi
            .iter()
            .any(|p| p.inizio_ms >= duration || p.fine_ms.saturating_sub(duration) > 80)
        {
            self.tempi.clear();
        }
        for span in &mut self.tempi {
            span.inizio_ms += start;
            span.fine_ms = span.fine_ms.min(duration) + start;
        }
        self
    }
}

impl From<String> for AsrResult {
    fn from(text: String) -> Self {
        Self {
            text,
            tempi: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "richiede Nemotron, Parakeet e Whisper locali; eseguire in sequenza"]
    fn tempi_asr_reali_e_divisione_del_testo_sui_tre_modelli() {
        use crate::audio_toolkit::decode::Decoder;
        use crate::engine::pipeline::{Feed, FileFrames};
        use crate::engine::transcribe_cpp::TranscribeCpp;
        use crate::engine::{TranscriptionEngine, diarize};
        use crate::managers::{models, settings::Diarizer};
        use crate::transcript::{Ingresso, Phrase};
        use std::path::{Path, PathBuf};

        let directory =
            PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parlato-it.wav");
        let frames: Vec<_> = FileFrames::new(Decoder::open(&fixture).unwrap(), None)
            .filter_map(|feed| match feed.unwrap() {
                Feed::Frame(frame) => Some(frame),
                _ => None,
            })
            .collect();
        let duration = u32::try_from(frames.len() * 30).unwrap();
        println!("durata della fixture: {duration} ms");
        for id in [
            "nemotron-3.5-streaming-0.6b-q5km",
            "parakeet-tdt-0.6b-v3-q5km",
            "whisper-large-v3-turbo-q5km",
        ] {
            let mut engine =
                TranscribeCpp::load(&models::find(id).unwrap().path(&directory)).unwrap();
            for streaming in [false, true] {
                let mut partials = 0;
                let mut timed_partials = 0;
                let mut partial = |result: &AsrResult| {
                    partials += 1;
                    if !result.tempi.is_empty() {
                        timed_partials += 1;
                    }
                };
                let result = engine
                    .transcribe(
                        &mut frames.clone().into_iter(),
                        Some("it"),
                        streaming.then_some(&mut partial as &mut dyn FnMut(&AsrResult)),
                    )
                    .unwrap()
                    .on_source(5000, 5000 + duration);
                println!(
                    "{id}, Parziali richiesti={streaming}, {partials} Parziali: {} tratti, {:?}",
                    result.tempi.len(),
                    result.tempi
                );
                assert!(!result.text.is_empty(), "{id}");
                assert!(!result.tempi.is_empty(), "{id}: tempi assenti");
                let original = result.text.clone();
                assert_eq!(
                    result
                        .tempi
                        .iter()
                        .map(|s| &result.text[s.inizio_byte..s.fine_byte])
                        .collect::<String>(),
                    original
                );
                if id.starts_with("whisper") || id.starts_with("parakeet") {
                    assert_eq!(partials, 0);
                }
                if id.starts_with("whisper") {
                    continue;
                }
                assert!(result.tempi.len() >= 5, "{id}: attesi tempi di parola");
                if streaming {
                    assert!(partials > 0 || id.starts_with("parakeet"));
                    if id.starts_with("nemotron") {
                        assert!(timed_partials > 0, "Parziali senza tempi affidabili");
                    }
                }
                let at = result.tempi.len() / 2;
                let boundary = result.tempi[at].inizio_ms;
                let mut phrases = vec![Phrase {
                    inizio_ms: 5000,
                    fine_ms: 5000 + duration,
                    text: result.text,
                    tempi: result.tempi,
                    ingresso: Ingresso::Mix,
                    parlante: None,
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                }];
                diarize::assign_configured(
                    &mut phrases,
                    Ingresso::Mix,
                    &[
                        diarize::Turn {
                            inizio_ms: 5000,
                            fine_ms: boundary,
                            parlante: 7,
                        },
                        diarize::Turn {
                            inizio_ms: boundary,
                            fine_ms: 5000 + duration,
                            parlante: 3,
                        },
                    ],
                    Diarizer::Nemotron3,
                );
                assert!(phrases.len() >= 2, "{id}: divisione assente");
                assert_eq!(
                    phrases.iter().map(|p| p.text.as_str()).collect::<String>(),
                    original
                );
                assert!(phrases.iter().all(|p| p.fine_ms > p.inizio_ms));
            }
        }
    }

    #[test]
    fn i_tempi_asr_reali_diventano_assoluti_senza_cambiare_il_testo() {
        let result = AsrResult::timed(
            "Perché sì! D'accordo?".into(),
            [
                (100, 400, "Perché".into()),
                (500, 700, "sì!".into()),
                (1100, 1500, "D'accordo?".into()),
            ],
        )
        .on_source(5000, 7000);
        assert_eq!(result.text, "Perché sì! D'accordo?");
        assert_eq!(
            result.tempi,
            [
                TempoTesto {
                    inizio_byte: 0,
                    fine_byte: 8,
                    inizio_ms: 5100,
                    fine_ms: 5400
                },
                TempoTesto {
                    inizio_byte: 8,
                    fine_byte: 13,
                    inizio_ms: 5500,
                    fine_ms: 5700
                },
                TempoTesto {
                    inizio_byte: 13,
                    fine_byte: 23,
                    inizio_ms: 6100,
                    fine_ms: 6500
                },
            ]
        );
    }

    #[test]
    fn tempi_mancanti_invalidi_o_testo_diverso_non_creano_un_allineamento() {
        for rows in [
            vec![],
            vec![(-1, 100, "Uno due.".into())],
            vec![(500, 400, "Uno due.".into())],
            vec![(0, 200, "Uno".into()), (100, 300, "due.".into())],
            vec![(0, 100, "Uno".into())],
            vec![(0, 100, "Uno".into()), (200, 300, "tre.".into())],
        ] {
            let result = AsrResult::timed("Uno due.".into(), rows);
            assert_eq!(result.text, "Uno due.");
            assert!(result.tempi.is_empty());
        }
        let result = AsrResult::timed("Uno due.".into(), [(0, 1081, "Uno due.".into())])
            .on_source(5000, 6000);
        assert!(result.tempi.is_empty());
        assert_eq!(result.text, "Uno due.");
    }

    #[test]
    fn il_padding_encoder_si_interseca_con_l_audio_reale_senza_stimare_le_parole() {
        let result =
            AsrResult::timed("Ciao!".into(), [(100, 1040, "Ciao!".into())]).on_source(5000, 6000);
        assert_eq!(
            result.tempi,
            [TempoTesto {
                inizio_byte: 0,
                fine_byte: 5,
                inizio_ms: 5100,
                fine_ms: 6000
            }]
        );
    }
}
