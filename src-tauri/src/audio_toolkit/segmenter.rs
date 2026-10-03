//! Segmentatore: macchina a stati pura che trasforma `(frame, probabilità di parlato)` in Frasi.

use std::collections::VecDeque;

/// Durata di un frame del VAD: 480 campioni a 16 kHz.
pub const FRAME_MS: u32 = 30;

/// Parametri di partenza della spec, da calibrare sull'audio reale.
/// Handy usa soglia 0,3, prefill 450 ms e hangover 450 ms: se la calibrazione non converge si riparte da lì.
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub prefill_ms: u32,
    pub onset_ms: u32,
    pub hangover_ms: u32,
    pub max_phrase_ms: u32,
    pub threshold: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            prefill_ms: 300,
            onset_ms: 60,
            hangover_ms: 700,
            max_phrase_ms: 18_000,
            threshold: 0.4,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Event {
    /// Inizio di una Frase, con l'indice (da 0) del suo primo frame tra quelli ricevuti.
    PhraseStart(usize),
    Audio(Vec<f32>),
    PhraseEnd,
}

pub struct Segmenter {
    prefill: usize,
    onset: usize,
    hangover: usize,
    max_frames: usize,
    threshold: f32,
    /// In silenzio: gli ultimi frame (prefill + onset) da anteporre alla Frase.
    pending: VecDeque<Vec<f32>>,
    onset_count: usize,
    /// `Some` durante una Frase: frame di silenzio ancora tollerati prima della chiusura.
    hangover_left: Option<usize>,
    phrase_frames: usize,
    /// Frame ricevuti finora.
    position: usize,
}

/// Millisecondi in frame, per eccesso.
fn ms_to_frames(ms: u32) -> usize {
    ms.div_ceil(FRAME_MS) as usize
}

impl Segmenter {
    pub fn new(params: Params) -> Self {
        let (prefill, onset) = (
            ms_to_frames(params.prefill_ms),
            ms_to_frames(params.onset_ms).max(1),
        );
        assert!(
            prefill + onset < ms_to_frames(params.max_phrase_ms),
            "prefill e onset devono stare sotto la durata massima della Frase"
        );
        Self {
            prefill,
            onset,
            hangover: ms_to_frames(params.hangover_ms),
            max_frames: ms_to_frames(params.max_phrase_ms),
            threshold: params.threshold,
            pending: VecDeque::new(),
            onset_count: 0,
            hangover_left: None,
            phrase_frames: 0,
            position: 0,
        }
    }

    pub fn push(&mut self, frame: Vec<f32>, probability: f32) -> Vec<Event> {
        let speech = probability >= self.threshold;
        self.position += 1;
        let mut events = Vec::new();
        match self.hangover_left {
            None => {
                self.onset_count = if speech { self.onset_count + 1 } else { 0 };
                self.pending.push_back(frame);
                if self.pending.len() > self.prefill + self.onset {
                    self.pending.pop_front();
                }
                if self.onset_count >= self.onset {
                    events.push(Event::PhraseStart(self.position - self.pending.len()));
                    self.phrase_frames = self.pending.len();
                    events.extend(self.pending.drain(..).map(Event::Audio));
                    self.onset_count = 0;
                    self.hangover_left = Some(self.hangover);
                }
            }
            Some(left) => {
                events.push(Event::Audio(frame));
                self.phrase_frames += 1;
                let left = if speech {
                    self.hangover
                } else {
                    left.saturating_sub(1)
                };
                self.hangover_left = Some(left);
                if left == 0 {
                    self.end_phrase(&mut events);
                }
            }
        }
        // Taglio a durata massima: il parlato che segue riparte da onset e prefill.
        if self.hangover_left.is_some() && self.phrase_frames >= self.max_frames {
            self.end_phrase(&mut events);
        }
        events
    }

    /// Chiude la Frase in corso, se c'è, come a fine parlato: alla fine dell'audio o per la Pausa.
    /// Anche fuori da una Frase il parlato successivo deve superare di nuovo l'onset.
    pub fn close_phrase(&mut self) -> Vec<Event> {
        self.onset_count = 0;
        let mut events = Vec::new();
        if self.hangover_left.is_some() {
            self.end_phrase(&mut events);
        }
        events
    }

    fn end_phrase(&mut self, events: &mut Vec<Event>) {
        events.push(Event::PhraseEnd);
        self.hangover_left = None;
        self.phrase_frames = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Etichetta ogni frame con il suo indice, così dagli eventi si legge quali frame sono passati.
    fn run(probabilities: &[f32]) -> Vec<String> {
        let mut segmenter = Segmenter::new(Params::default());
        let mut out = Vec::new();
        let mut events = Vec::new();
        for (i, &p) in probabilities.iter().enumerate() {
            events.extend(segmenter.push(vec![i as f32], p));
        }
        events.extend(segmenter.close_phrase());
        for event in events {
            out.push(match event {
                Event::PhraseStart(_) => "[".to_string(),
                Event::Audio(frame) => format!("{}", frame[0]),
                Event::PhraseEnd => "]".to_string(),
            });
        }
        out
    }

    fn seq(parts: &[(f32, usize)]) -> Vec<f32> {
        parts
            .iter()
            .flat_map(|&(p, n)| std::iter::repeat_n(p, n))
            .collect()
    }

    fn phrases(labels: &[String]) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        for label in labels {
            match label.as_str() {
                "[" => out.push(Vec::new()),
                "]" => {}
                n => out.last_mut().unwrap().push(n.parse().unwrap()),
            }
        }
        out
    }

    #[test]
    fn il_silenzio_non_produce_frasi() {
        assert!(run(&seq(&[(0.1, 100)])).is_empty());
    }

    #[test]
    fn un_frame_di_parlato_isolato_non_supera_l_onset() {
        assert!(run(&seq(&[(0.0, 20), (0.9, 1), (0.0, 50)])).is_empty());
    }

    #[test]
    fn la_frase_include_300_ms_di_prefill_e_l_onset() {
        // Parlato dal frame 20: onset di 2 frame, prefill di 10 frame (12-19).
        let labels = run(&seq(&[(0.0, 20), (0.9, 5), (0.0, 40)]));
        let first = &phrases(&labels)[0];
        assert_eq!(first[0], 10);
        assert_eq!(&first[..12], &(10..22).collect::<Vec<_>>());
    }

    #[test]
    fn la_frase_si_chiude_dopo_700_ms_di_silenzio() {
        // Parlato 20-24, poi 24 frame (720 ms) di hangover: l'ultimo frame della Frase è il 48.
        let labels = run(&seq(&[(0.0, 20), (0.9, 5), (0.0, 40)]));
        let all = phrases(&labels);
        assert_eq!(all.len(), 1);
        assert_eq!(*all[0].last().unwrap(), 48);
    }

    #[test]
    fn una_pausa_piu_corta_dell_hangover_non_spezza_la_frase() {
        let labels = run(&seq(&[(0.9, 10), (0.0, 20), (0.9, 10), (0.0, 30)]));
        assert_eq!(phrases(&labels).len(), 1);
    }

    #[test]
    fn due_frasi_separate_da_una_pausa_lunga() {
        let labels = run(&seq(&[(0.9, 10), (0.0, 40), (0.9, 10), (0.0, 30)]));
        assert_eq!(phrases(&labels).len(), 2);
    }

    #[test]
    fn il_parlato_continuo_si_taglia_a_18_secondi_senza_perdere_frame() {
        // 40 s di parlato: 18 s + 18 s + il resto.
        let labels = run(&seq(&[(0.9, 1334)]));
        let all = phrases(&labels);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].len(), 600);
        assert_eq!(all[1].len(), 600);
        let flat: Vec<usize> = all.concat();
        assert_eq!(flat, (0..1334).collect::<Vec<_>>());
    }

    #[test]
    fn l_inizio_della_frase_e_l_indice_del_suo_primo_frame() {
        // Prefill, pausa lunga e taglio a 18 s: ogni inizio è il primo frame della sua Frase.
        let mut segmenter = Segmenter::new(Params::default());
        let mut events = Vec::new();
        let probabilities = seq(&[(0.0, 20), (0.9, 5), (0.0, 40), (0.9, 700), (0.0, 30)]);
        for (i, &p) in probabilities.iter().enumerate() {
            events.extend(segmenter.push(vec![i as f32], p));
        }
        let starts: Vec<(usize, f32)> = events
            .windows(2)
            .filter_map(|w| match w {
                [Event::PhraseStart(start), Event::Audio(frame)] => Some((*start, frame[0])),
                _ => None,
            })
            .collect();
        assert_eq!(starts.len(), 3, "{starts:?}");
        assert!(
            starts.iter().all(|&(start, first)| start as f32 == first),
            "{starts:?}"
        );
        assert_eq!(starts[0].0, 10);
    }

    #[test]
    fn chiudere_la_frase_a_meta_parlato_ne_apre_una_nuova_senza_prefill() {
        let mut segmenter = Segmenter::new(Params::default());
        let mut labels = Vec::new();
        for i in 0..20 {
            labels.extend(segmenter.push(vec![i as f32], 0.9));
        }
        labels.extend(segmenter.close_phrase());
        for i in 20..40 {
            labels.extend(segmenter.push(vec![i as f32], 0.9));
        }
        let ends = labels.iter().filter(|e| **e == Event::PhraseEnd).count();
        assert_eq!(ends, 1);
        assert!(labels.contains(&Event::PhraseStart(0)));
        // Dopo la chiusura serve di nuovo l'onset: la Frase nuova parte dal frame 20.
        assert!(labels.contains(&Event::PhraseStart(20)), "{labels:?}");
    }

    #[test]
    fn chiudere_fuori_da_una_frase_azzera_l_onset() {
        // Un frame di parlato prima della chiusura e uno dopo non fanno un onset da 2 frame.
        let mut segmenter = Segmenter::new(Params::default());
        let mut events = segmenter.push(vec![0.0], 0.9);
        events.extend(segmenter.close_phrase());
        events.extend(segmenter.push(vec![1.0], 0.9));
        assert!(events.is_empty(), "{events:?}");
    }

    #[test]
    fn la_fine_dell_audio_chiude_la_frase_in_corso() {
        let labels = run(&seq(&[(0.0, 5), (0.9, 10)]));
        assert_eq!(labels.last().unwrap(), "]");
        assert_eq!(phrases(&labels)[0], (0..15).collect::<Vec<_>>());
    }
}
