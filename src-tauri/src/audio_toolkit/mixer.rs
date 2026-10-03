//! Mixer della Registrazione: posiziona i blocchi catturati per timestamp, esclude le pause,
//! riempie di silenzio i buchi e porta l'audio a canali e frequenza delle impostazioni.

use crate::audio_toolkit::resample::Resampler;
use crate::error::AppError;

const NS_PER_S: u64 = 1_000_000_000;
/// Un vuoto tra due blocchi da qui in su è audio perso (discontinuità): diventa silenzio. Sotto è
/// il tremolio dei timestamp, e i blocchi restano attaccati.
const HOLE_NS: u64 = 20_000_000;

/// Una sorgente della Registrazione. ponytail: una sola sorgente (il microfono); la somma di
/// microfono e audio di sistema arriva con "Entrambi" (ticket 09).
pub struct Mixer {
    in_rate: u64,
    in_channels: usize,
    out_channels: usize,
    resampler: Resampler,
    /// Timestamp del primo blocco tenuto.
    origin: u64,
    /// Fine (timestamp + durata) dell'ultimo blocco tenuto; `None` prima del primo.
    last_end: Option<u64>,
    /// Il tempo tra l'ultimo blocco prima di una pausa e il primo dopo.
    paused_ns: u64,
    /// Il prossimo blocco tenuto segue una pausa: il vuoto non è un buco.
    resuming: bool,
    converted: Vec<f32>,
}

impl Mixer {
    /// `input`: frequenza e canali del dispositivo; `output`: quelli della Registrazione.
    pub fn new(
        (in_rate, in_channels): (u32, usize),
        (out_rate, out_channels): (u32, usize),
    ) -> Result<Self, AppError> {
        Ok(Self {
            in_rate: u64::from(in_rate),
            in_channels: in_channels.max(1),
            out_channels,
            resampler: Resampler::new(in_rate, out_rate, out_channels)?,
            origin: 0,
            last_end: None,
            paused_ns: 0,
            resuming: false,
            converted: Vec::new(),
        })
    }

    /// Un blocco catturato: `capture_ns` è il suo timestamp di cattura, `samples` i campioni
    /// interleaved del dispositivo. In pausa il blocco si scarta. Accoda in `out` l'audio pronto.
    /// Restituisce il picco del blocco, tra 0 e 1 (0 in pausa).
    pub fn push(
        &mut self,
        capture_ns: u64,
        samples: &[f32],
        paused: bool,
        out: &mut Vec<f32>,
    ) -> f32 {
        if paused {
            self.resuming |= self.last_end.is_some();
            return 0.0;
        }
        let frames = samples.len() / self.in_channels;
        match self.last_end {
            None => self.origin = capture_ns,
            Some(end) => {
                let gap = capture_ns.saturating_sub(end);
                if self.resuming {
                    self.paused_ns += gap;
                } else if gap >= HOLE_NS {
                    let silence = (gap * self.in_rate / NS_PER_S) as usize * self.out_channels;
                    self.resampler.push(&vec![0.0; silence], out);
                }
            }
        }
        self.resuming = false;
        let end = capture_ns + frames as u64 * NS_PER_S / self.in_rate;
        self.last_end = Some(self.last_end.map_or(end, |last| last.max(end)));
        self.converted.clear();
        for frame in samples.chunks_exact(self.in_channels) {
            match (self.out_channels, frame) {
                // In mono le sorgenti stereo (o con più canali) si mediano.
                (1, _) => self
                    .converted
                    .push(frame.iter().sum::<f32>() / frame.len() as f32),
                // In stereo il microfono mono si duplica; oltre due canali si tengono i primi.
                (_, [mono]) => self.converted.extend([*mono, *mono]),
                (_, [left, right, ..]) => self.converted.extend([*left, *right]),
                (_, []) => {}
            }
        }
        self.resampler.push(&self.converted, out);
        samples
            .iter()
            .fold(0.0, |peak: f32, s| peak.max(s.abs()))
            .min(1.0)
    }

    /// Accoda in `out` l'audio ancora nel resampler.
    pub fn finish(&mut self, out: &mut Vec<f32>) {
        self.resampler.finish(out);
    }

    /// La durata registrata, pause escluse, dai timestamp dei blocchi.
    pub fn elapsed_ms(&self) -> u32 {
        self.last_end.map_or(0, |end| {
            u32::try_from((end - self.origin - self.paused_ns) / 1_000_000).unwrap_or(u32::MAX)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{decoded_seconds, sine, temp_dir};

    const MS: u64 = 1_000_000;

    /// Spinge `seconds` di sinusoide in blocchi da 10 ms a partire da `start_ns`, come un
    /// dispositivo; restituisce il timestamp dopo l'ultimo blocco.
    fn capture(
        mixer: &mut Mixer,
        (rate, channels): (u32, usize),
        start_ns: u64,
        seconds: f64,
        paused: bool,
        out: &mut Vec<f32>,
    ) -> u64 {
        let block = rate as usize / 100 * channels;
        let mut ts = start_ns;
        for chunk in sine(rate, channels, seconds).chunks(block) {
            mixer.push(ts, chunk, paused, out);
            ts += 10 * MS;
        }
        ts
    }

    #[test]
    fn i_blocchi_continui_diventano_audio_continuo_alla_frequenza_scelta() {
        let device = (44_100, 2);
        let mut mixer = Mixer::new(device, (16_000, 1)).unwrap();
        let mut out = Vec::new();
        capture(&mut mixer, device, 5_000 * MS, 2.0, false, &mut out);
        mixer.finish(&mut out);
        assert_eq!(out.len(), 32_000);
        assert_eq!(mixer.elapsed_ms(), 2_000);
    }

    #[test]
    fn le_pause_non_contano_ne_nel_timer_ne_nell_audio() {
        let device = (48_000, 1);
        let mut mixer = Mixer::new(device, (48_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, device, 0, 1.0, false, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_000);
        // 3 s di pausa: i blocchi arrivano ma si scartano e il timer resta fermo.
        let ts = capture(&mut mixer, device, ts, 3.0, true, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_000);
        // Dopo Riprendi il primo blocco arriva anche un po' più tardi: nessun vuoto.
        capture(&mut mixer, device, ts + 50 * MS, 0.5, false, &mut out);
        mixer.finish(&mut out);
        assert_eq!(mixer.elapsed_ms(), 1_500);
        assert_eq!(out.len(), 72_000);
    }

    #[test]
    fn un_buco_tra_i_blocchi_diventa_silenzio() {
        let device = (16_000, 1);
        let mut mixer = Mixer::new(device, (16_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, device, 0, 1.0, false, &mut out);
        // 250 ms senza pacchetti (discontinuità), poi l'audio riprende.
        capture(&mut mixer, device, ts + 250 * MS, 1.0, false, &mut out);
        mixer.finish(&mut out);
        assert_eq!(mixer.elapsed_ms(), 2_250);
        assert_eq!(out.len(), 36_000);
        assert!(out[16_000..20_000].iter().all(|&s| s == 0.0));
        assert!(out[20_000..20_100].iter().any(|&s| s != 0.0));
    }

    #[test]
    fn mono_e_stereo_si_convertono_nei_canali_della_registrazione() {
        // Stereo con un canale muto → mono: la media dei due.
        let mut mixer = Mixer::new((48_000, 2), (48_000, 1)).unwrap();
        let mut out = Vec::new();
        let peak = mixer.push(0, &[0.8, 0.0, -0.4, 0.0], false, &mut out);
        assert_eq!(out, [0.4, -0.2]);
        assert!((peak - 0.8).abs() < 1e-6);
        // Mono → stereo: il campione si duplica.
        let mut mixer = Mixer::new((48_000, 1), (48_000, 2)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, &[0.5, -0.25], false, &mut out);
        assert_eq!(out, [0.5, 0.5, -0.25, -0.25]);
    }

    #[test]
    fn una_registrazione_con_pausa_rilegge_con_la_durata_senza_la_pausa() {
        let dir = temp_dir("mixer-registrazione");
        let device = (44_100, 2);
        for (rate, channels) in [(16_000, 1), (48_000, 2)] {
            let path = dir.join(format!("{rate}-{channels}.ogg"));
            let mut writer =
                OggOpusWriter::new(std::fs::File::create(&path).unwrap(), rate, channels, 32)
                    .unwrap();
            let mut mixer = Mixer::new(device, (rate, channels)).unwrap();
            let mut out = Vec::new();
            let ts = capture(&mut mixer, device, 0, 1.2, false, &mut out);
            let ts = capture(&mut mixer, device, ts, 2.0, true, &mut out);
            capture(&mut mixer, device, ts, 0.8, false, &mut out);
            mixer.finish(&mut out);
            writer.write(&out).unwrap();
            writer.finish().unwrap();
            let seconds = decoded_seconds(&path);
            assert!((seconds - 2.0).abs() < 0.001, "{rate} Hz: {seconds} s");
        }
    }
}
