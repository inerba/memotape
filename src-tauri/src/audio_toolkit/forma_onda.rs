//! La Forma d'onda: i picchi dell'audio ogni 20 ms, raggruppati in un numero fisso di valori.

/// Quanti valori della Forma d'onda si salvano nel Tape, qualunque sia la durata (ADR-0010).
pub const VALORI: usize = 1000;
/// Quanto audio riassume un picco prima del raggruppamento finale.
const WINDOW_MS: u32 = 20;

/// Raccoglie i picchi di audio interleaved, un blocco alla volta.
pub struct Picchi {
    /// Campioni interleaved di 20 ms.
    window: usize,
    windows: Vec<f32>,
    peak: f32,
    filled: usize,
}

impl Picchi {
    pub fn new(rate: u32, channels: usize) -> Self {
        Self {
            window: (rate * WINDOW_MS / 1000).max(1) as usize * channels.max(1),
            windows: Vec::new(),
            peak: 0.0,
            filled: 0,
        }
    }

    pub fn push(&mut self, samples: &[f32]) {
        for sample in samples {
            self.peak = self.peak.max(sample.abs());
            self.filled += 1;
            if self.filled == self.window {
                self.windows.push(self.peak.min(1.0));
                (self.peak, self.filled) = (0.0, 0);
            }
        }
    }

    /// `count` valori (0–1) dall'inizio alla fine, meno se l'audio dura meno di `count` × 20 ms.
    pub fn values(&self, count: usize) -> Vec<f32> {
        let tail = (self.filled > 0).then_some(self.peak.min(1.0));
        let all: Vec<f32> = self.windows.iter().copied().chain(tail).collect();
        regroup(&all, count)
    }
}

/// `values` in `count` gruppi consecutivi, ciascuno con il suo massimo; com'è se sono già al più
/// `count`.
pub fn regroup(values: &[f32], count: usize) -> Vec<f32> {
    if values.len() <= count {
        return values.to_vec();
    }
    (0..count)
        .map(|i| {
            let range = i * values.len() / count..(i + 1) * values.len() / count;
            values[range].iter().copied().fold(0.0, f32::max)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_picco_ogni_20_ms_poi_raggruppati_nei_valori_chiesti() {
        // 1 s a 1 kHz stereo: 50 finestre da 40 campioni; la quinta ha un picco negativo.
        let mut samples = vec![0.1f32; 2000];
        samples[4 * 40 + 3] = -0.9;
        let mut picchi = Picchi::new(1000, 2);
        // A blocchi che non coincidono con le finestre.
        for block in samples.chunks(37) {
            picchi.push(block);
        }
        let fine = picchi.values(1000);
        assert_eq!(fine.len(), 50);
        assert!((fine[4] - 0.9).abs() < 1e-6 && (fine[5] - 0.1).abs() < 1e-6);
        assert_eq!(picchi.values(10), {
            let mut grouped = vec![0.1f32; 10];
            grouped[0] = 0.9;
            grouped
        });
        // Un resto più corto di 20 ms è un valore anche lui; oltre 1 si taglia.
        picchi.push(&[1.5]);
        assert_eq!(picchi.values(1000).len(), 51);
        assert_eq!(picchi.values(1000)[50], 1.0);
    }

    #[test]
    fn raggruppare_tiene_il_massimo_di_ogni_gruppo() {
        assert_eq!(regroup(&[0.1, 0.5, 0.2, 0.3], 2), [0.5, 0.3]);
        assert_eq!(regroup(&[0.1, 0.5], 3), [0.1, 0.5]);
    }
}
