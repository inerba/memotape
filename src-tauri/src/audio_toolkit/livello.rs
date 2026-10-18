//! Il livello di una Frase prima del modello: se è rumore, e dove dividerla.

/// Sotto questo livello (`livello`) una Frase è rumore. Sui Tape reali delle misure
/// (docs/research/eco-e-loop-sui-tape-reali.md) le Frasi del Microfono senza segnale restavano
/// sotto −52,7 dBFS e quelle dell'Audio di sistema sopra −39,5; la voce a −30 dB del corpus della
/// pulizia sta a −47,1 e −47,7 (ADR-0034).
/// ponytail: soglia fissa con 2,3 dB di margine sulla voce e 2,7 sul rumore misurato; un tratto
/// di 1 s del Microfono senza segnale, nel suo punto più forte, arriva a −49,1 e passerebbe.
/// Diventa regolabile, o si affianca a un altro indizio, se capita.
const SOGLIA_DBFS: f32 = -50.0;

/// Se la Frase `pcm` è rumore, da non dare al modello: Whisper ci ricopiava il prompt, Cohere
/// inventava frasi intere.
pub fn rumore(pcm: &[f32]) -> bool {
    livello(pcm) < 10f32.powf(SOGLIA_DBFS / 20.0)
}

/// L'RMS del quarto più forte dei frame: le pause di una Frase lunga non abbassano il livello della
/// voce che contiene, e qualche frame forte nel rumore non basta ad alzarlo.
fn livello(pcm: &[f32]) -> f32 {
    let mut frames: Vec<f32> = pcm
        .chunks(super::resample::FRAME_SAMPLES)
        .map(rms)
        .collect();
    frames.sort_by(|a, b| b.total_cmp(a));
    rms(&frames[..frames.len().div_ceil(4)])
}

/// Il campione in cui dividere una Frase che il modello non riesce a trascrivere intera: il centro
/// del frame più silenzioso nella metà centrale, così nessun pezzo è solo prefill o coda.
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

    /// Il picco di una sinusoide con RMS di `dbfs`: il picco è RMS per √2.
    fn picco(dbfs: f32) -> f32 {
        10f32.powf(dbfs / 20.0) * std::f32::consts::SQRT_2
    }

    #[test]
    fn sotto_meno_50_dbfs_una_frase_e_rumore() {
        // −62 dBFS come il Microfono senza segnale dei Tape reali, −25 dBFS come una voce normale,
        // −47 dBFS appena sopra la soglia.
        assert!(rumore(&tono(picco(-62.0), 1.0)));
        assert!(rumore(&vec![0.0; 16_000]));
        assert!(rumore(&[]));
        assert!(!rumore(&tono(picco(-25.0), 1.0)));
        assert!(!rumore(&tono(picco(-47.0), 1.0)));
    }

    #[test]
    fn una_voce_debole_con_lunghe_pause_non_e_rumore() {
        // 0,6 s di voce a −46 dBFS e 2,4 s di silenzio: sull'intera Frase −53 dBFS, ma la voce c'è
        // (il campione a −30 dB del corpus della pulizia, unito in una Frase da Silero v6.2).
        let mut pcm = tono(picco(-46.0), 0.6);
        pcm.extend(vec![0.0; 38_400]);
        assert!(!rumore(&pcm));
    }

    #[test]
    fn il_microfono_senza_segnale_con_un_frame_forte_resta_rumore() {
        // 2,2 s a −62 dBFS, come la Frase più forte del Microfono senza segnale dei Tape reali, con
        // un frame a −42 come il più forte della traccia: il quarto più forte sta a −53,9 dBFS.
        let frame = super::super::resample::FRAME_SAMPLES;
        let mut pcm = tono(picco(-62.0), 2.2);
        pcm.splice(frame * 30..frame * 31, tono(picco(-42.0), 0.032));
        assert!(rumore(&pcm));
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
