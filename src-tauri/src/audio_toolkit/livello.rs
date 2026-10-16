//! Il livello di una Frase prima del modello: se è rumore, e dove dividerla.

/// Sotto questo RMS una Frase è rumore: sui Tape reali il Microfono muto stava tra −62 e −76 dBFS
/// e il parlato più debole 23 dB sopra (docs/research/eco-e-loop-sui-tape-reali.md).
/// ponytail: soglia fissa, non provata su voci molto deboli o lontane; diventa regolabile se serve.
const SOGLIA_DBFS: f32 = -50.0;

/// Se la Frase `pcm` è rumore, da non dare al modello: Whisper ci ricopiava il prompt, Cohere
/// inventava frasi intere.
pub fn rumore(pcm: &[f32]) -> bool {
    rms(pcm) < 10f32.powf(SOGLIA_DBFS / 20.0)
}

/// Il campione in cui dividere una Frase che il modello non riesce a trascrivere intera: il centro
/// del frame da 30 ms più silenzioso nella metà centrale, così nessun pezzo è solo prefill o coda.
pub fn punto_piu_silenzioso(pcm: &[f32]) -> usize {
    let frame = super::resample::FRAME_SAMPLES;
    let (da, a) = (pcm.len() / 4, pcm.len() * 3 / 4);
    (da..a.saturating_sub(frame))
        .step_by(frame)
        .min_by(|&x, &y| rms(&pcm[x..x + frame]).total_cmp(&rms(&pcm[y..y + frame])))
        .map_or(pcm.len() / 2, |inizio| inizio + frame / 2)
}

fn rms(pcm: &[f32]) -> f32 {
    if pcm.is_empty() {
        return 0.0;
    }
    (pcm.iter().map(|s| s * s).sum::<f32>() / pcm.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `secondi` di sinusoide a 300 Hz con ampiezza di picco `picco`, a 16 kHz.
    fn tono(picco: f32, secondi: f32) -> Vec<f32> {
        (0..(16_000.0 * secondi) as usize)
            .map(|i| (i as f32 / 16_000.0 * 300.0 * std::f32::consts::TAU).sin() * picco)
            .collect()
    }

    #[test]
    fn sotto_meno_50_dbfs_una_frase_e_rumore() {
        // Una sinusoide di picco p ha RMS p/√2: −62 dBFS come il Microfono muto dei Tape reali,
        // −25 dBFS come una voce normale, −47 dBFS appena sopra la soglia.
        let rms = |dbfs: f32| 10f32.powf(dbfs / 20.0) * std::f32::consts::SQRT_2;
        assert!(rumore(&tono(rms(-62.0), 1.0)));
        assert!(rumore(&vec![0.0; 16_000]));
        assert!(rumore(&[]));
        assert!(!rumore(&tono(rms(-25.0), 1.0)));
        assert!(!rumore(&tono(rms(-47.0), 1.0)));
    }

    #[test]
    fn la_frase_si_divide_nella_pausa_al_centro_non_nel_silenzio_ai_bordi() {
        // 0,3 s di prefill muto, parlato, una pausa di 0,4 s a 3,5 s, parlato, coda muta.
        let mut pcm = vec![0.0; 4800];
        pcm.extend(tono(0.3, 3.2));
        pcm.extend(vec![0.001; 6400]);
        pcm.extend(tono(0.3, 3.0));
        pcm.extend(vec![0.0; 8000]);
        let taglio = punto_piu_silenzioso(&pcm);
        assert!((56_000..62_400).contains(&taglio), "{taglio}");
    }
}
