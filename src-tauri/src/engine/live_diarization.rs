//! Parlanti dal vivo: audio prima del VAD, coda limitata e attribuzioni rettificabili. Senza Tauri.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::time::{Duration, Instant};

use transcribe_cpp::CancelToken;

use super::diarize::Turn;
use super::pipeline::PipelineEvent;
use crate::audio_toolkit::segmenter::FRAME_MS;
use crate::error::AppError;
use crate::transcript::{Ingresso, Phrase};

/// Tre secondi di frame da 30 ms (circa 192 KB). Non è l'obiettivo di latenza del modello.
#[cfg(test)]
pub const QUEUE_FRAMES: usize = 100;
pub const MAX_DELAY: Duration = Duration::from_secs(3);

/// La scelta del modello e il suo canale appartengono alla stessa sessione.
pub struct LiveDiarizer {
    path: std::path::PathBuf,
    frames: DiarizationFrames,
}

impl LiveDiarizer {
    pub fn cancel_token(&self) -> CancelToken {
        self.frames.cancel_token()
    }

    pub fn run(
        &mut self,
        cancel: &CancelToken,
        on_turns: &mut dyn FnMut(Vec<Turn>),
    ) -> Result<(), AppError> {
        let result = super::transcribe_cpp::OfflineDiarizer::load_nemotron3(&self.path)
            .and_then(|mut model| model.diarize_live(&mut self.frames, cancel, on_turns));
        self.frames.check(cancel)?;
        result
    }
}

#[derive(Debug)]
pub struct AudioFrame {
    pub samples: Vec<f32>,
    pub sent: Instant,
    pub fine_ms: u32,
}

struct Shared {
    lagging: AtomicBool,
    cancel: CancelToken,
}

pub struct DiarizationFeed {
    tx: SyncSender<AudioFrame>,
    shared: Arc<Shared>,
    frames: u32,
}

pub struct DiarizationFrames {
    rx: Receiver<AudioFrame>,
    shared: Arc<Shared>,
}

#[cfg(test)]
pub fn channel() -> (DiarizationFeed, DiarizationFrames) {
    let (tx, rx) = std::sync::mpsc::sync_channel(QUEUE_FRAMES);
    let shared = Arc::new(Shared {
        lagging: AtomicBool::new(false),
        cancel: CancelToken::new(),
    });
    (
        DiarizationFeed {
            tx,
            shared: Arc::clone(&shared),
            frames: 0,
        },
        DiarizationFrames { rx, shared },
    )
}

impl DiarizationFeed {
    /// Non aspetta mai il diarizer. Se un frame non entra, l'intera sessione termina: niente buchi.
    pub fn push(&mut self, samples: &[f32]) {
        if self.shared.cancel.is_cancelled() {
            return;
        }
        self.frames = self.frames.saturating_add(1);
        let frame = AudioFrame {
            samples: samples.to_vec(),
            sent: Instant::now(),
            fine_ms: self.frames.saturating_mul(FRAME_MS),
        };
        match self.tx.try_send(frame) {
            Err(TrySendError::Full(_)) => {
                self.shared.lagging.store(true, Ordering::Release);
                self.shared.cancel.cancel();
            }
            Err(TrySendError::Disconnected(_)) => self.shared.cancel.cancel(),
            Ok(()) => {}
        }
    }
}

impl DiarizationFrames {
    pub fn cancel_token(&self) -> CancelToken {
        self.shared.cancel.clone()
    }

    pub fn check(&self, cancel: &CancelToken) -> Result<(), AppError> {
        if self.shared.lagging.load(Ordering::Acquire) {
            Err(AppError::LiveDiarizationLagging)
        } else if cancel.is_cancelled() || self.shared.cancel.is_cancelled() {
            Err(AppError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn next_frame(&mut self, cancel: &CancelToken) -> Result<Option<AudioFrame>, AppError> {
        loop {
            self.check(cancel)?;
            match self.rx.recv_timeout(Duration::from_millis(100)) {
                Ok(frame) => {
                    self.check_delay(frame.sent, cancel)?;
                    return Ok(Some(frame));
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    self.check(cancel)?;
                    return Ok(None);
                }
            }
        }
    }

    pub fn check_delay(&self, sent: Instant, cancel: &CancelToken) -> Result<(), AppError> {
        if sent.elapsed() > MAX_DELAY {
            self.shared.lagging.store(true, Ordering::Release);
            self.shared.cancel.cancel();
        }
        self.check(cancel)
    }
}

/// Uno snapshot di un solo Ingresso. Ogni parte mantiene un id per il proprio confine testuale,
/// anche se l'ASR cresce o il diarizer divide e poi riunisce il testo già comparso.
#[derive(Default)]
pub struct LiveTranscript {
    pub revision: u32,
    pub phrases: Vec<(u32, Phrase)>,
    pub partials: Vec<(u32, Phrase)>,
    originals: Vec<(u32, Phrase)>,
    partial: Option<(u32, Phrase)>,
    identities: std::collections::BTreeMap<(u32, usize), u32>,
    next_id: u32,
    turns: Vec<Turn>,
}

impl LiveTranscript {
    /// Le Frasi ASR originali restano separate dalla vista rettificata e dai Parziali.
    pub fn originals(&self) -> impl Iterator<Item = &Phrase> {
        self.originals.iter().map(|(_, phrase)| phrase)
    }

    /// L'ASR terminata o guasta non lascia un Parziale che una rettifica riaprirebbe.
    pub fn clear_partial(&mut self) {
        self.partial = None;
        self.partials.clear();
        self.revision = self.revision.saturating_add(1);
    }

    pub fn on_asr(&mut self, ingresso: Ingresso, event: PipelineEvent) {
        let (id, inizio_ms, fine_ms, text, tempi, partial) = match event {
            PipelineEvent::Partial {
                id,
                inizio_ms,
                fine_ms,
                text,
                tempi,
            } => (id, inizio_ms, fine_ms, text, tempi, true),
            PipelineEvent::Phrase {
                id,
                inizio_ms,
                fine_ms,
                text,
                tempi,
            } => (id, inizio_ms, fine_ms, text, tempi, false),
            PipelineEvent::Progress(_) => return,
        };
        if partial && self.originals.iter().any(|(closed, _)| *closed == id) {
            return;
        }
        let phrase = Phrase {
            inizio_ms,
            fine_ms,
            text,
            tempi,
            ingresso,
            parlante: None,
            parlante_non_determinato: true,
            parlante_provvisorio: true,
        };
        if partial {
            self.partial = (!phrase.text.is_empty()).then_some((id, phrase));
        } else {
            self.partial = None;
            if let Some((_, previous)) = self.originals.iter_mut().find(|(old, _)| *old == id) {
                *previous = phrase;
            } else {
                self.originals.push((id, phrase));
            }
        }
        self.assign();
    }

    pub fn on_turns(&mut self, turns: Vec<Turn>) {
        self.turns = turns;
        self.assign();
    }

    fn assign(&mut self) {
        let horizon = self.turns.iter().map(|t| t.fine_ms).max().unwrap_or(0);
        let mut project = |source: &(u32, Phrase)| {
            let mut offset = 0;
            super::diarize::divide(source.1.clone(), &self.turns, Some(horizon))
                .into_iter()
                .map(|phrase| {
                    let id = *self
                        .identities
                        .entry((source.0, offset))
                        .or_insert_with(|| {
                            let id = self.next_id;
                            self.next_id = self.next_id.saturating_add(1);
                            id
                        });
                    offset += phrase.text.len();
                    (id, phrase)
                })
                .collect::<Vec<_>>()
        };
        self.phrases = self.originals.iter().flat_map(&mut project).collect();
        self.partials = self.partial.iter().flat_map(project).collect();
        self.revision = self.revision.saturating_add(1);
    }
}

#[cfg(test)]
mod native_tests;

#[cfg(test)]
mod dual_native_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn il_testo_in_ritardo_riceve_i_turni_e_la_frase_conclusa_non_riapre_il_parziale() {
        let mut session = LiveTranscript::default();
        session.on_turns(vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 500,
                parlante: 8,
            },
            Turn {
                inizio_ms: 1000,
                fine_ms: 4000,
                parlante: 3,
            },
        ]);
        session.on_asr(
            Ingresso::Mix,
            PipelineEvent::Partial {
                id: 0,
                inizio_ms: 1000,
                fine_ms: 2000,
                text: "Buongiorno".into(),
                tempi: vec![],
            },
        );
        assert_eq!(session.partials[0].1.parlante, Some(2));
        session.on_asr(
            Ingresso::Mix,
            PipelineEvent::Phrase {
                id: 0,
                inizio_ms: 1000,
                fine_ms: 2500,
                text: "Buongiorno!".into(),
                tempi: vec![],
            },
        );
        let revision = session.revision;
        session.on_turns(vec![Turn {
            inizio_ms: 0,
            fine_ms: 4000,
            parlante: 8,
        }]);
        assert!(session.partial.is_none());
        assert!(session.revision > revision);
        assert_eq!(session.phrases[0].0, 0);
        assert_eq!(session.phrases[0].1.text, "Buongiorno!");
        assert_eq!(session.phrases[0].1.parlante, Some(1));
        assert!(session.phrases[0].1.parlante_provvisorio);
        // Una Registrazione nuova non eredita identità o cache.
        assert!(LiveTranscript::default().phrases.is_empty());
    }

    #[test]
    fn con_diarizer_in_ritardo_o_voci_sovrapposte_il_testo_resta_non_determinato() {
        let mut session = LiveTranscript::default();
        session.on_asr(
            Ingresso::Mix,
            PipelineEvent::Partial {
                id: 0,
                inizio_ms: 0,
                fine_ms: 2000,
                text: "Testo da conservare".into(),
                tempi: vec![],
            },
        );
        session.on_turns(vec![Turn {
            inizio_ms: 0,
            fine_ms: 1000,
            parlante: 0,
        }]);
        assert!(session.partials[0].1.parlante_non_determinato);
        session.on_turns(vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 2000,
                parlante: 0,
            },
            Turn {
                inizio_ms: 1500,
                fine_ms: 2000,
                parlante: 1,
            },
        ]);
        let partial = session.partials.remove(0).1;
        assert!(partial.parlante_non_determinato);
        assert_eq!(partial.parlante, None);
        assert_eq!(partial.text, "Testo da conservare");
    }

    #[test]
    fn un_parziale_interrotto_non_ricompare_con_una_rettifica_del_diarizer() {
        let mut session = LiveTranscript::default();
        session.on_asr(
            Ingresso::Mix,
            PipelineEvent::Partial {
                id: 0,
                inizio_ms: 0,
                fine_ms: 1000,
                text: "Testo interrotto".into(),
                tempi: vec![],
            },
        );
        session.clear_partial();
        session.on_turns(vec![Turn {
            inizio_ms: 0,
            fine_ms: 1000,
            parlante: 0,
        }]);
        assert!(session.partial.is_none());
        assert!(session.phrases.is_empty());
    }

    fn timed_event(partial: bool) -> PipelineEvent {
        let result = super::super::asr::AsrResult::timed(
            "Perché sì! D'accordo?".into(),
            [
                (100, 600, "Perché".into()),
                (800, 1100, "sì!".into()),
                (1300, 2300, "D'accordo?".into()),
            ],
        )
        .on_source(1000, 4000);
        if partial {
            PipelineEvent::Partial {
                id: 0,
                inizio_ms: 1000,
                fine_ms: 4000,
                text: result.text,
                tempi: result.tempi,
            }
        } else {
            PipelineEvent::Phrase {
                id: 0,
                inizio_ms: 1000,
                fine_ms: 4000,
                text: result.text,
                tempi: result.tempi,
            }
        }
    }

    #[test]
    fn rettifica_divide_e_riunisce_parziali_e_frasi_dalle_parole_originali() {
        let mut state = LiveTranscript::default();
        state.on_asr(Ingresso::Sistema, timed_event(true));
        let root_id = state.partials[0].0;
        state.on_turns(vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 2100,
                parlante: 7,
            },
            Turn {
                inizio_ms: 2100,
                fine_ms: 4000,
                parlante: 3,
            },
        ]);
        assert_eq!(state.partials.len(), 2);
        let parts = state.partials.clone();
        assert_eq!(parts[0].0, root_id);
        assert_eq!(parts[0].1.text, "Perché sì! ");
        assert_eq!(parts[1].1.text, "D'accordo?");
        assert_eq!(parts[0].1.parlante, Some(1));
        assert_eq!(parts[1].1.parlante, Some(2));
        assert_eq!(parts[0].1.fine_ms, 2100);
        assert_eq!(parts[1].1.inizio_ms, 2300);
        // Una rettifica può togliere un confine già mostrato, poi ripristinare le stesse identità.
        state.on_turns(vec![Turn {
            inizio_ms: 0,
            fine_ms: 4000,
            parlante: 3,
        }]);
        assert_eq!(state.partials.len(), 1);
        assert_eq!(state.partials[0].0, root_id);
        assert_eq!(state.partials[0].1.text, "Perché sì! D'accordo?");
        state.on_turns(vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 2100,
                parlante: 7,
            },
            Turn {
                inizio_ms: 2100,
                fine_ms: 4000,
                parlante: 3,
            },
        ]);
        assert_eq!(state.partials, parts);
        state.on_asr(Ingresso::Sistema, timed_event(false));
        assert!(state.partials.is_empty());
        assert_eq!(state.phrases, parts);
        assert_eq!(state.originals().count(), 1);
        assert_eq!(
            state.originals().next().unwrap().text,
            "Perché sì! D'accordo?"
        );
        let revision = state.revision;
        state.on_asr(Ingresso::Sistema, timed_event(true));
        assert_eq!(state.revision, revision);
        assert!(state.partials.is_empty());
    }

    #[test]
    fn un_orizzonte_in_ritardo_divide_solo_le_parole_coperte_e_non_separa_voci_simultanee() {
        let mut state = LiveTranscript::default();
        state.on_asr(Ingresso::Microfono, timed_event(true));
        state.on_turns(vec![Turn {
            inizio_ms: 0,
            fine_ms: 2000,
            parlante: 0,
        }]);
        assert_eq!(state.partials.len(), 2);
        assert_eq!(state.partials[0].1.parlante, Some(1));
        assert_eq!(state.partials[1].1.text, "sì! D'accordo?");
        assert!(state.partials[1].1.parlante_non_determinato);
        state.on_turns(vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 4000,
                parlante: 0,
            },
            Turn {
                inizio_ms: 1800,
                fine_ms: 2100,
                parlante: 1,
            },
        ]);
        assert_eq!(state.partials.len(), 3);
        assert_eq!(state.partials[1].1.text, "sì! ");
        assert!(state.partials[1].1.parlante_non_determinato);
        assert_eq!(
            state
                .partials
                .iter()
                .map(|(_, p)| p.text.as_str())
                .collect::<String>(),
            "Perché sì! D'accordo?"
        );
    }

    #[test]
    fn un_segmento_whisper_resta_indivisibile_e_senza_tempi_non_si_inventano_parole() {
        for tempi in [
            Vec::new(),
            vec![crate::transcript::TempoTesto {
                inizio_byte: 0,
                fine_byte: 13,
                inizio_ms: 0,
                fine_ms: 2000,
            }],
        ] {
            let mut state = LiveTranscript::default();
            state.on_asr(
                Ingresso::Mix,
                PipelineEvent::Phrase {
                    id: 0,
                    inizio_ms: 0,
                    fine_ms: 2000,
                    text: "Uno due. Tre!".into(),
                    tempi,
                },
            );
            state.on_turns(vec![
                Turn {
                    inizio_ms: 0,
                    fine_ms: 1000,
                    parlante: 0,
                },
                Turn {
                    inizio_ms: 1000,
                    fine_ms: 2000,
                    parlante: 1,
                },
            ]);
            assert!(state.partials.is_empty());
            assert_eq!(state.phrases.len(), 1);
            assert_eq!(state.phrases[0].1.text, "Uno due. Tre!");
            assert!(state.phrases[0].1.parlante_non_determinato);
        }
    }

    #[test]
    fn l_analisi_finale_riunisce_le_divisioni_senza_asr_e_annulla_conserva_la_vista() {
        use crate::managers::settings::{Diarizer, SpeechLanguage};
        use crate::transcript::{EsitoDiarizzazione, Transcript};
        let mut state = LiveTranscript::default();
        state.on_asr(Ingresso::Mix, timed_event(false));
        state.on_turns(vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 2100,
                parlante: 0,
            },
            Turn {
                inizio_ms: 2100,
                fine_ms: 4000,
                parlante: 1,
            },
        ]);
        let original = state.originals().cloned().collect::<Vec<_>>();
        let provisional = state
            .phrases
            .iter()
            .map(|(_, p)| p.clone())
            .collect::<Vec<_>>();
        let make_document = || Transcript {
            title: "Prova".into(),
            date: "2026-10-06 00:00".into(),
            durata_ms: Some(4000),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::from("it"),
            phrases: provisional.clone(),
            live_asr: original.clone(),
            parlanti: Default::default(),
            diarizzazione: None,
        };
        let turns = vec![(
            Ingresso::Mix,
            vec![Turn {
                inizio_ms: 0,
                fine_ms: 4000,
                parlante: 5,
            }],
        )];
        let mut document = make_document();
        super::super::diarize::finalize_live_ingressi(
            &mut document,
            Diarizer::Nemotron3,
            turns.clone().into_iter().map(|(i, t)| (i, Ok(t))).collect(),
        );
        assert_eq!(document.phrases.len(), 1);
        assert_eq!(document.phrases[0].text, original[0].text);
        assert_eq!(document.phrases[0].tempi, original[0].tempi);
        assert_eq!(
            (document.phrases[0].inizio_ms, document.phrases[0].fine_ms),
            (1000, 4000)
        );
        assert!(!document.phrases[0].parlante_provvisorio);
        assert!(document.live_asr.is_empty());
        let mut document = make_document();
        super::super::diarize::finalize_live_ingressi(
            &mut document,
            Diarizer::Nemotron3,
            vec![(Ingresso::Mix, Err(AppError::Cancelled))],
        );
        assert_eq!(document.phrases, provisional);
        assert_eq!(
            document.diarizzazione.unwrap().esito,
            EsitoDiarizzazione::Annullata
        );
        let mut document = make_document();
        super::super::diarize::finalize_live_ingressi(
            &mut document,
            Diarizer::Nemotron3,
            vec![(
                Ingresso::Mix,
                Err(AppError::Internal("errore finale".into())),
            )],
        );
        assert_eq!(document.phrases, provisional);
        assert_eq!(
            document.diarizzazione.unwrap().esito,
            EsitoDiarizzazione::Fallita
        );
    }

    #[test]
    fn due_ingressi_hanno_identita_revisioni_e_guasti_indipendenti() {
        let mut mic = LiveTranscript::default();
        let mut system = LiveTranscript::default();
        mic.on_asr(Ingresso::Microfono, timed_event(false));
        system.on_asr(Ingresso::Sistema, timed_event(false));
        let turns = vec![Turn {
            inizio_ms: 0,
            fine_ms: 4000,
            parlante: 7,
        }];
        mic.on_turns(turns.clone());
        system.on_turns(turns);
        assert_eq!(mic.phrases[0].1.parlante, Some(1));
        assert_eq!(system.phrases[0].1.parlante, Some(1));
        assert_ne!(
            mic.phrases[0].1.ingresso.parlante_key(Some(1)),
            system.phrases[0].1.ingresso.parlante_key(Some(1))
        );
        let revision = system.revision;
        let (mut mic_feed, mut mic_audio) = channel();
        let (mut system_feed, mut system_audio) = channel();
        for _ in 0..=QUEUE_FRAMES {
            mic_feed.push(&[0.0; 480]);
        }
        let cancel = CancelToken::new();
        assert_eq!(
            mic_audio.next_frame(&cancel).unwrap_err(),
            AppError::LiveDiarizationLagging
        );
        system_feed.push(&[1.0; 480]);
        assert_eq!(
            system_audio.next_frame(&cancel).unwrap().unwrap().fine_ms,
            30
        );
        system.on_turns(vec![Turn {
            inizio_ms: 0,
            fine_ms: 4000,
            parlante: 2,
        }]);
        assert!(system.revision > revision);
        assert_eq!(mic.phrases[0].1.ingresso, Ingresso::Microfono);
        assert!(
            system
                .phrases
                .iter()
                .all(|(_, p)| p.ingresso == Ingresso::Sistema)
        );
    }

    #[test]
    fn finale_per_ingresso_con_guasto_riapre_il_tape_con_audio_e_testo_separati() {
        use crate::managers::settings::{CopiaCome, Diarizer, Language, SpeechLanguage};
        use crate::transcript::{EsitoDiarizzazione, Transcript};
        let mut mic = LiveTranscript::default();
        let mut system = LiveTranscript::default();
        mic.on_asr(Ingresso::Microfono, timed_event(false));
        system.on_asr(Ingresso::Sistema, timed_event(false));
        let mut transcript = Transcript {
            title: "Entrambi".into(),
            date: "2026-10-06 00:00".into(),
            durata_ms: Some(4000),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::from("it"),
            phrases: mic
                .phrases
                .iter()
                .chain(&system.phrases)
                .map(|(_, p)| p.clone())
                .collect(),
            live_asr: mic.originals().chain(system.originals()).cloned().collect(),
            parlanti: Default::default(),
            diarizzazione: None,
        };
        let available = transcript.phrases[0].clone();
        super::super::diarize::finalize_live_ingressi(
            &mut transcript,
            Diarizer::Nemotron3,
            vec![
                (
                    Ingresso::Microfono,
                    Err(AppError::Internal("solo Microfono".into())),
                ),
                (
                    Ingresso::Sistema,
                    Ok(vec![Turn {
                        inizio_ms: 0,
                        fine_ms: 4000,
                        parlante: 7,
                    }]),
                ),
            ],
        );
        assert_eq!(
            transcript
                .phrases
                .iter()
                .find(|p| p.ingresso == Ingresso::Microfono)
                .unwrap(),
            &available
        );
        let system = transcript
            .phrases
            .iter()
            .find(|p| p.ingresso == Ingresso::Sistema)
            .unwrap();
        assert_eq!(system.parlante, Some(1));
        assert!(!system.parlante_provvisorio);
        let state = transcript.diarizzazione.as_ref().unwrap();
        assert_eq!(state.esito, EsitoDiarizzazione::Fallita);
        assert_eq!(state.ingressi[1].esito, EsitoDiarizzazione::Completata);
        let copied = crate::transcript::render(
            &transcript,
            &crate::transcript::Labels::of(Language::It),
            CopiaCome::Markdown,
        );
        assert!(copied.contains("Microfono: Diarizzazione non completata"));
        assert!(copied.contains("Audio di sistema: Diarizzazione completata"));
        let folder = crate::audio_toolkit::ogg_opus::tests::temp_dir("due-ingressi-tape");
        let audio = folder.join("audio.ogg");
        std::fs::write(&audio, b"audio conservato").unwrap();
        let mut document = crate::tape::Document::new(
            crate::tape::creato(chrono::Local::now()),
            4000,
            crate::tape::Modalita::IngressiSeparati,
            Some("nemotron".into()),
            SpeechLanguage::from("it"),
            true,
            &transcript.phrases,
        );
        document.diarizzazione = transcript.diarizzazione;
        let path = folder.join("Entrambi.tape");
        crate::tape::write(
            &path,
            &[
                (Ingresso::Mix, &audio),
                (Ingresso::Microfono, &audio),
                (Ingresso::Sistema, &audio),
            ],
            &document,
            None,
        )
        .unwrap();
        assert_eq!(crate::tape::read(&path).unwrap(), document);
        let reopened = crate::managers::transcription::open_tape(&path).unwrap();
        assert_eq!(reopened.info.diarizzazione, document.diarizzazione);
        assert!(
            reopened
                .phrases
                .iter()
                .any(|p| p.ingresso == Ingresso::Sistema && !p.parlante_provvisorio)
        );
        assert!(
            reopened
                .phrases
                .iter()
                .any(|p| p.ingresso == Ingresso::Microfono && p.parlante_provvisorio)
        );
        assert!(crate::tape::has_ingressi(&path).unwrap());
    }

    #[test]
    fn annulla_nell_altro_ingresso_non_invalida_un_analisi_gia_riuscita() {
        use crate::managers::settings::{Diarizer, SpeechLanguage};
        use crate::transcript::{EsitoDiarizzazione, Transcript};
        let mut mic = LiveTranscript::default();
        let mut system = LiveTranscript::default();
        mic.on_asr(Ingresso::Microfono, timed_event(false));
        system.on_asr(Ingresso::Sistema, timed_event(false));
        let available = system.phrases[0].1.clone();
        let mut transcript = Transcript {
            title: "Entrambi".into(),
            date: "2026-10-06 00:00".into(),
            durata_ms: Some(4000),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::from("it"),
            phrases: mic
                .phrases
                .iter()
                .chain(&system.phrases)
                .map(|(_, p)| p.clone())
                .collect(),
            live_asr: mic.originals().chain(system.originals()).cloned().collect(),
            parlanti: Default::default(),
            diarizzazione: None,
        };
        super::super::diarize::finalize_live_ingressi(
            &mut transcript,
            Diarizer::Nemotron3,
            vec![
                (
                    Ingresso::Microfono,
                    Ok(vec![Turn {
                        inizio_ms: 0,
                        fine_ms: 4000,
                        parlante: 7,
                    }]),
                ),
                (Ingresso::Sistema, Err(AppError::Cancelled)),
            ],
        );
        let mic = transcript
            .phrases
            .iter()
            .find(|p| p.ingresso == Ingresso::Microfono)
            .unwrap();
        assert_eq!(mic.parlante, Some(1));
        assert!(!mic.parlante_provvisorio);
        assert_eq!(
            transcript
                .phrases
                .iter()
                .find(|p| p.ingresso == Ingresso::Sistema)
                .unwrap(),
            &available
        );
        let state = transcript.diarizzazione.unwrap();
        assert_eq!(state.ingressi[0].esito, EsitoDiarizzazione::Completata);
        assert_eq!(state.ingressi[1].esito, EsitoDiarizzazione::Annullata);
        assert_eq!(state.esito, EsitoDiarizzazione::Annullata);
    }

    #[test]
    fn stop_svuota_il_canale_e_annulla_non_aspetta_nuovo_audio() {
        let (mut feed, mut frames) = channel();
        feed.push(&[0.0; 480]);
        drop(feed);
        let cancel = CancelToken::new();
        assert_eq!(frames.next_frame(&cancel).unwrap().unwrap().fine_ms, 30);
        assert!(frames.next_frame(&cancel).unwrap().is_none());
        let (_feed, mut frames) = channel();
        cancel.cancel();
        assert_eq!(frames.next_frame(&cancel).unwrap_err(), AppError::Cancelled);
    }

    #[test]
    fn una_coda_vecchia_si_interrompe_anche_se_non_e_piena() {
        let (mut feed, mut frames) = channel();
        feed.push(&[0.0; 480]);
        std::thread::sleep(MAX_DELAY + Duration::from_millis(20));
        assert_eq!(
            frames.next_frame(&CancelToken::new()).unwrap_err(),
            AppError::LiveDiarizationLagging
        );
        assert!(frames.cancel_token().is_cancelled());
    }
}
