//! Mixer della Registrazione: posiziona i blocchi di ogni ingresso (microfono, audio di sistema)
//! per timestamp di cattura rispetto all'inizio della sessione, esclude le pause, riempie di
//! silenzio i buchi, porta l'audio a canali e frequenza delle impostazioni e somma gli ingressi.
//!
//! I timestamp vengono tutti da QPC (anche `now`), quindi gli ingressi restano allineati tra loro.
//! Ogni ingresso resta entro `HOLE_NS` dai suoi timestamp: un vuoto più lungo diventa silenzio, un
//! anticipo più lungo (blocchi sovrapposti a quanto già scritto) si scarta. Così si corregge anche la
//! deriva tra il clock del dispositivo e QPC. ponytail: a scatti da `HOLE_NS`; se si sentono,
//! correggere con continuità con un resampler asincrono di rubato.

use crate::audio_toolkit::resample::Resampler;
use crate::error::AppError;

const NS_PER_S: u64 = 1_000_000_000;
/// Uno scarto da qui in su tra dove un blocco dovrebbe stare e dove arriva l'ingresso è audio perso
/// (o in più): diventa silenzio (o si scarta). Sotto è il tremolio dei timestamp.
const HOLE_NS: u64 = 20_000_000;
/// Un salto dell'orologio da qui in su non è silenzio ma una sospensione del PC: vale come una
/// pausa, senza ore di silenzio in memoria e nel file. Il worker chiama `advance` ogni 100 ms.
const MAX_HOLE_NS: u64 = 10 * NS_PER_S;
/// Quanto un ingresso può restare indietro rispetto all'orologio prima che il buco diventi
/// silenzio: il loopback non consegna nulla a riproduzione ferma. Deve superare il ritardo con cui
/// i blocchi arrivano al worker, anche dai dispositivi lenti (Bluetooth): un blocco che arriva dopo
/// il silenzio messo al suo posto si scarta.
const LAG_NS: u64 = 500_000_000;

pub struct Mixer {
    out_channels: usize,
    inputs: Vec<Input>,
    /// Inizio della sessione (QPC, ns).
    origin: u64,
    /// Il tempo escluso: pause e sospensioni.
    paused_ns: u64,
    /// Da quando è in pausa; `None` se non lo è.
    pause_start: Option<u64>,
    /// L'ultimo `now` di `advance`.
    last_now: u64,
    /// Campioni sommati non ancora usciti, interleaved, dal frame `emitted` in poi.
    mix: Vec<f32>,
    emitted: usize,
}

struct Input {
    rate: u64,
    channels: usize,
    resampler: Resampler,
    /// Frame alla frequenza del dispositivo già sulla linea del tempo, silenzio compreso.
    frames: u64,
    /// Frame alla frequenza della Registrazione già sommati in `mix`.
    mixed: usize,
    peak: f32,
    /// Per il log a fine Registrazione: frame di silenzio inseriti e scartati, e per misurare la
    /// deriva del clock del dispositivo rispetto a QPC i frame ricevuti (pause comprese) tra il
    /// timestamp del primo blocco e la fine dell'ultimo.
    silence: u64,
    dropped: u64,
    received: u64,
    first_ns: Option<u64>,
    last_end_ns: u64,
    converted: Vec<f32>,
    resampled: Vec<f32>,
}

impl Input {
    /// Fine dell'audio già scritto, sulla linea del tempo (ns).
    fn end_ns(&self) -> u64 {
        self.frames * NS_PER_S / self.rate
    }

    /// Il frame alla frequenza del dispositivo che corrisponde a `t_ns` sulla linea del tempo.
    fn frame_at(&self, t_ns: i64) -> i64 {
        (i128::from(t_ns) * i128::from(self.rate) / i128::from(NS_PER_S)) as i64
    }
}

impl Mixer {
    /// `origin_ns`: l'inizio della sessione (QPC); `inputs`: frequenza e canali di ogni
    /// dispositivo; `output`: quelli della Registrazione.
    pub fn new(
        origin_ns: u64,
        inputs: &[(u32, usize)],
        (out_rate, out_channels): (u32, usize),
    ) -> Result<Self, AppError> {
        let inputs = inputs
            .iter()
            .map(|&(rate, channels)| {
                Ok(Input {
                    rate: u64::from(rate),
                    channels: channels.max(1),
                    resampler: Resampler::new(rate, out_rate, out_channels)?,
                    frames: 0,
                    mixed: 0,
                    peak: 0.0,
                    silence: 0,
                    dropped: 0,
                    received: 0,
                    first_ns: None,
                    last_end_ns: 0,
                    converted: Vec::new(),
                    resampled: Vec::new(),
                })
            })
            .collect::<Result<_, AppError>>()?;
        Ok(Self {
            out_channels,
            inputs,
            origin: origin_ns,
            paused_ns: 0,
            pause_start: None,
            last_now: origin_ns,
            mix: Vec::new(),
            emitted: 0,
        })
    }

    /// Un blocco dell'ingresso `input`: `capture_ns` è il suo timestamp di cattura, `samples` i
    /// campioni interleaved del dispositivo. In pausa si scarta. Accoda in `out` l'audio pronto.
    pub fn push(&mut self, input: usize, capture_ns: u64, samples: &[f32], out: &mut Vec<f32>) {
        let source = &mut self.inputs[input];
        let frames = (samples.len() / source.channels) as u64;
        source.received += frames;
        source.first_ns.get_or_insert(capture_ns);
        source.last_end_ns = capture_ns + frames * NS_PER_S / source.rate;
        if self.pause_start.is_some() {
            return;
        }
        let t = self.timeline(capture_ns);
        let out_channels = self.out_channels;
        let Self {
            inputs,
            mix,
            emitted,
            ..
        } = self;
        let input = &mut inputs[input];
        input.peak = samples
            .iter()
            .fold(input.peak, |peak, s| peak.max(s.abs()))
            .min(1.0);
        let start = input.frame_at(t);
        let position = input.frames as i64;
        let tolerance = input.frame_at(HOLE_NS as i64);
        let mut skip = 0;
        if start - position >= tolerance {
            feed_silence(
                input,
                (start - position) as u64,
                out_channels,
                mix,
                *emitted,
            );
        } else if position - start >= tolerance {
            skip = (position - start) as usize;
        }
        let frames = samples.len() / input.channels;
        let skip = skip.min(frames);
        input.dropped += skip as u64;
        input.converted.clear();
        for frame in samples[skip * input.channels..].chunks_exact(input.channels) {
            match (out_channels, frame) {
                // In mono le sorgenti stereo (o con più canali) si mediano.
                (1, _) => input
                    .converted
                    .push(frame.iter().sum::<f32>() / frame.len() as f32),
                // In stereo il microfono mono si duplica; oltre due canali si tengono i primi.
                (_, [mono]) => input.converted.extend([*mono, *mono]),
                (_, [left, right, ..]) => input.converted.extend([*left, *right]),
                (_, []) => {}
            }
        }
        let converted = std::mem::take(&mut input.converted);
        feed(input, &converted, out_channels, mix, *emitted);
        input.converted = converted;
        self.emit(out);
    }

    /// L'orologio (QPC, ns) e lo stato della pausa, da chiamare di continuo (almeno ogni 100 ms).
    /// Conta pause e sospensioni e riempie di silenzio gli ingressi rimasti indietro, come il
    /// loopback a riproduzione ferma. Accoda in `out` l'audio pronto.
    pub fn advance(&mut self, now_ns: u64, paused: bool, out: &mut Vec<f32>) {
        if paused {
            self.pause_start.get_or_insert(now_ns);
            return;
        }
        let excluded = match self.pause_start.take() {
            Some(start) => now_ns.saturating_sub(start),
            None if now_ns >= self.last_now + MAX_HOLE_NS => now_ns - self.last_now,
            None => 0,
        };
        self.paused_ns += excluded;
        self.last_now = now_ns;
        self.fill_to(self.timeline(now_ns) - LAG_NS as i64, HOLE_NS);
        self.emit(out);
    }

    /// Chiude la sessione all'istante `now_ns` (Stop): gli ingressi si completano di silenzio fino
    /// a lì (in pausa fino al più lungo) e in `out` arriva tutto l'audio rimasto.
    pub fn finish(&mut self, now_ns: u64, out: &mut Vec<f32>) {
        if self.pause_start.is_none() {
            self.fill_to(self.timeline(now_ns), HOLE_NS);
        }
        let end = self.inputs.iter().map(Input::end_ns).max().unwrap_or(0);
        self.fill_to(end as i64, 0);
        let out_channels = self.out_channels;
        for input in &mut self.inputs {
            input.resampled.clear();
            input.resampler.finish(&mut input.resampled);
            let resampled = std::mem::take(&mut input.resampled);
            add(input, &resampled, out_channels, &mut self.mix, self.emitted);
            input.resampled = resampled;
            // Con una consegna continua (il microfono, il loopback durante una riproduzione) la
            // differenza tra le due durate è la deriva del clock del dispositivo rispetto a QPC.
            log::info!(
                "ingresso {} Hz: ricevuti {} ms in {} ms di timestamp; {} ms di silenzio inseriti, {} ms scartati",
                input.rate,
                input.received * 1_000 / input.rate,
                input
                    .first_ns
                    .map_or(0, |first| (input.last_end_ns - first) / 1_000_000),
                input.silence * 1_000 / input.rate,
                input.dropped * 1_000 / input.rate,
            );
        }
        let last = self.inputs.iter().map(|i| i.mixed).max().unwrap_or(0);
        self.mix
            .resize((last - self.emitted) * self.out_channels, 0.0);
        for input in &mut self.inputs {
            input.mixed = last;
        }
        self.emit(out);
    }

    /// La durata registrata, pause escluse, dai timestamp.
    pub fn elapsed_ms(&self) -> u32 {
        let end = self.inputs.iter().map(Input::end_ns).max().unwrap_or(0);
        u32::try_from(end / 1_000_000).unwrap_or(u32::MAX)
    }

    /// Il picco (0–1) di ogni ingresso dalla chiamata precedente.
    pub fn take_peaks(&mut self) -> Vec<f32> {
        self.inputs
            .iter_mut()
            .map(|i| std::mem::take(&mut i.peak))
            .collect()
    }

    /// La posizione di un istante QPC sulla linea del tempo della Registrazione (ns).
    fn timeline(&self, ns: u64) -> i64 {
        ns as i64 - (self.origin + self.paused_ns) as i64
    }

    /// Completa di silenzio fino a `t_ns` gli ingressi indietro di almeno `tolerance_ns`.
    fn fill_to(&mut self, t_ns: i64, tolerance_ns: u64) {
        let out_channels = self.out_channels;
        for input in &mut self.inputs {
            let missing = input.frame_at(t_ns) - input.frames as i64;
            if missing > 0 && missing >= input.frame_at(tolerance_ns as i64) {
                feed_silence(
                    input,
                    missing as u64,
                    out_channels,
                    &mut self.mix,
                    self.emitted,
                );
            }
        }
    }

    /// Accoda in `out`, con il clamp a [-1, 1], i frame che tutti gli ingressi hanno già sommato.
    fn emit(&mut self, out: &mut Vec<f32>) {
        let ready = self.inputs.iter().map(|i| i.mixed).min().unwrap_or(0);
        let n = (ready - self.emitted) * self.out_channels;
        out.extend(self.mix.drain(..n).map(|s| s.clamp(-1.0, 1.0)));
        self.emitted = ready;
    }
}

fn feed_silence(
    input: &mut Input,
    frames: u64,
    out_channels: usize,
    mix: &mut Vec<f32>,
    emitted: usize,
) {
    input.silence += frames;
    let silence = vec![0.0; frames as usize * out_channels];
    feed(input, &silence, out_channels, mix, emitted);
}

/// Ricampiona `samples` (già nei canali della Registrazione) e li somma in `mix`.
fn feed(
    input: &mut Input,
    samples: &[f32],
    out_channels: usize,
    mix: &mut Vec<f32>,
    emitted: usize,
) {
    input.frames += (samples.len() / out_channels) as u64;
    input.resampled.clear();
    input.resampler.push(samples, &mut input.resampled);
    let resampled = std::mem::take(&mut input.resampled);
    add(input, &resampled, out_channels, mix, emitted);
    input.resampled = resampled;
}

/// Somma in `mix` l'audio ricampionato dell'ingresso, dopo quanto ha già sommato.
fn add(
    input: &mut Input,
    resampled: &[f32],
    out_channels: usize,
    mix: &mut Vec<f32>,
    emitted: usize,
) {
    let start = (input.mixed - emitted) * out_channels;
    let end = start + resampled.len();
    if mix.len() < end {
        mix.resize(end, 0.0);
    }
    for (m, s) in mix[start..end].iter_mut().zip(resampled) {
        *m += s;
    }
    input.mixed += resampled.len() / out_channels;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{decoded_seconds, sine, temp_dir};

    const MS: u64 = 1_000_000;

    /// Spinge `seconds` di sinusoide nell'ingresso `input` in blocchi da 10 ms a partire da
    /// `start_ns`, come un dispositivo, con l'orologio che avanza a ogni blocco; restituisce il
    /// timestamp dopo l'ultimo blocco.
    fn capture(
        mixer: &mut Mixer,
        input: usize,
        (rate, channels): (u32, usize),
        start_ns: u64,
        seconds: f64,
        out: &mut Vec<f32>,
    ) -> u64 {
        let block = rate as usize / 100 * channels;
        let mut ts = start_ns;
        for chunk in sine(rate, channels, seconds).chunks(block) {
            // Come il worker: prima l'orologio, poi il blocco.
            mixer.advance(ts, false, out);
            mixer.push(input, ts, chunk, out);
            ts += 10 * MS;
        }
        ts
    }

    #[test]
    fn i_blocchi_continui_diventano_audio_continuo_alla_frequenza_scelta() {
        let device = (44_100, 2);
        let mut mixer = Mixer::new(5_000 * MS, &[device], (16_000, 1)).unwrap();
        let mut out = Vec::new();
        let end = capture(&mut mixer, 0, device, 5_000 * MS, 2.0, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(out.len(), 32_000);
        assert_eq!(mixer.elapsed_ms(), 2_000);
    }

    /// Come `capture`, ma con la Registrazione in pausa: i blocchi arrivano e si scartano.
    fn capture_paused(
        mixer: &mut Mixer,
        input: usize,
        device: (u32, usize),
        start_ns: u64,
        seconds: f64,
        out: &mut Vec<f32>,
    ) -> u64 {
        let block = device.0 as usize / 100 * device.1;
        let mut ts = start_ns;
        for chunk in sine(device.0, device.1, seconds).chunks(block) {
            mixer.advance(ts, true, out);
            mixer.push(input, ts, chunk, out);
            ts += 10 * MS;
        }
        ts
    }

    #[test]
    fn le_pause_non_contano_ne_nel_timer_ne_nell_audio() {
        let device = (48_000, 1);
        let mut mixer = Mixer::new(0, &[device], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, 0, device, 0, 1.0, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_000);
        // 3 s di pausa: i blocchi arrivano ma si scartano e il timer resta fermo.
        let ts = capture_paused(&mut mixer, 0, device, ts, 3.0, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_000);
        // Dopo Riprendi l'audio continua senza vuoti.
        let end = capture(&mut mixer, 0, device, ts, 0.5, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_500);
        assert_eq!(out.len(), 72_000);
    }

    #[test]
    fn un_buco_tra_i_blocchi_diventa_silenzio() {
        let device = (16_000, 1);
        let mut mixer = Mixer::new(0, &[device], (16_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, 0, device, 0, 1.0, &mut out);
        // 250 ms senza pacchetti (discontinuità), poi l'audio riprende.
        let end = capture(&mut mixer, 0, device, ts + 250 * MS, 1.0, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(mixer.elapsed_ms(), 2_250);
        assert_eq!(out.len(), 36_000);
        assert!(out[16_000..20_000].iter().all(|&s| s == 0.0));
        assert!(out[20_000..20_100].iter().any(|&s| s != 0.0));
    }

    #[test]
    fn a_riproduzione_ferma_il_timer_avanza_e_il_file_si_riempie_di_silenzio() {
        let device = (48_000, 2);
        let mut mixer = Mixer::new(0, &[device], (16_000, 1)).unwrap();
        let mut out = Vec::new();
        let mut ts = capture(&mut mixer, 0, device, 0, 1.0, &mut out);
        // Il loopback non consegna nulla per 3 s: l'orologio avanza ogni 100 ms.
        for _ in 0..30 {
            ts += 100 * MS;
            mixer.advance(ts, false, &mut out);
        }
        // Il timer resta indietro di poco rispetto all'orologio, non è fermo a 1 s.
        assert!(
            (3_400..=4_000).contains(&mixer.elapsed_ms()),
            "{}",
            mixer.elapsed_ms()
        );
        let end = capture(&mut mixer, 0, device, ts, 1.0, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(mixer.elapsed_ms(), 5_000);
        assert_eq!(out.len(), 80_000);
        // Silenzio, a parte lo smorzamento del filtro del resampler ai bordi.
        assert!(out[16_100..63_900].iter().all(|&s| s.abs() < 1e-4));
        assert!(out[64_000..64_100].iter().any(|&s| s != 0.0));
    }

    #[test]
    fn la_registrazione_finisce_all_istante_dello_stop_anche_in_silenzio() {
        let device = (48_000, 2);
        let mut mixer = Mixer::new(0, &[device], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        // Il loopback non consegna mai nulla: Stop dopo 2 s.
        for ms in (100..=2_000).step_by(100) {
            mixer.advance(ms * MS, false, &mut out);
        }
        mixer.finish(2_000 * MS, &mut out);
        assert_eq!(mixer.elapsed_ms(), 2_000);
        assert_eq!(out.len(), 96_000);
    }

    #[test]
    fn una_sospensione_del_pc_conta_come_una_pausa_e_non_diventa_silenzio() {
        let device = (16_000, 1);
        let mut mixer = Mixer::new(0, &[device], (16_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, 0, device, 0, 1.0, &mut out);
        // Il PC sospeso per un'ora: niente ore di silenzio in memoria né nel file.
        let end = capture(&mut mixer, 0, device, ts + 3_600_000 * MS, 1.0, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(mixer.elapsed_ms(), 2_000);
        assert_eq!(out.len(), 32_000);
    }

    #[test]
    fn mono_e_stereo_si_convertono_nei_canali_della_registrazione() {
        // Stereo con un canale muto → mono: la media dei due.
        let mut mixer = Mixer::new(0, &[(48_000, 2)], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &[0.8, 0.0, -0.4, 0.0], &mut out);
        assert_eq!(out, [0.4, -0.2]);
        assert_eq!(mixer.take_peaks(), [0.8]);
        assert_eq!(mixer.take_peaks(), [0.0]);
        // Mono → stereo: il campione si duplica.
        let mut mixer = Mixer::new(0, &[(48_000, 1)], (48_000, 2)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &[0.5, -0.25], &mut out);
        assert_eq!(out, [0.5, 0.5, -0.25, -0.25]);
    }

    #[test]
    fn con_entrambi_il_sistema_si_somma_al_microfono_dove_dicono_i_timestamp() {
        let (mic, system) = ((44_100, 1), (48_000, 2));
        let mut mixer = Mixer::new(0, &[mic, system], (48_000, 2)).unwrap();
        let mut out = Vec::new();
        // Microfono mono continuo per 2 s; il loopback consegna solo tra 1 s e 1,5 s.
        for k in 0..200u64 {
            let ts = k * 10 * MS;
            mixer.advance(ts, false, &mut out);
            mixer.push(0, ts, &[0.25; 441], &mut out);
            if (100..150).contains(&k) {
                mixer.push(1, ts, &[0.5, -0.5].repeat(480), &mut out);
            }
        }
        mixer.finish(2_000 * MS, &mut out);
        assert_eq!(mixer.elapsed_ms(), 2_000);
        assert_eq!(out.len(), 96_000 * 2);
        let frame = |seconds: f64| {
            let i = (seconds * 48_000.0) as usize * 2;
            (out[i], out[i + 1])
        };
        let near = |(l, r): (f32, f32), (el, er): (f32, f32)| {
            (l - el).abs() < 0.01 && (r - er).abs() < 0.01
        };
        // Il microfono mono va su entrambi i canali; il sistema si somma solo dove c'è.
        assert!(near(frame(0.5), (0.25, 0.25)), "{:?}", frame(0.5));
        assert!(near(frame(1.25), (0.75, -0.25)), "{:?}", frame(1.25));
        assert!(near(frame(1.75), (0.25, 0.25)), "{:?}", frame(1.75));
    }

    #[test]
    fn la_somma_si_ferma_a_fondo_scala() {
        let mut mixer = Mixer::new(0, &[(48_000, 1), (48_000, 2)], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &[0.8, -0.8], &mut out);
        mixer.push(1, 0, &[0.8, 0.8, -0.8, -0.8], &mut out);
        assert_eq!(out, [1.0, -1.0]);
        assert_eq!(mixer.take_peaks(), [0.8, 0.8]);
    }

    #[test]
    fn la_deriva_del_clock_del_dispositivo_non_sposta_il_file_rispetto_al_timer() {
        // Ogni blocco porta 10 ms di campioni, ma secondo QPC ne passano 9,9 (dispositivo veloce)
        // o 10,1 (lento): in 10 s sarebbero 100 ms di troppo o di meno.
        for step_us in [9_900, 10_100] {
            let device = (48_000, 1);
            let mut mixer = Mixer::new(0, &[device], (48_000, 1)).unwrap();
            let mut out = Vec::new();
            let mut ts = 0;
            for _ in 0..1_000 {
                mixer.advance(ts, false, &mut out);
                mixer.push(0, ts, &[0.25; 480], &mut out);
                ts += step_us * 1_000;
            }
            mixer.finish(ts, &mut out);
            let file_ms = out.len() as i64 * 1_000 / 48_000;
            let timeline_ms = i64::try_from(ts / MS).unwrap();
            assert!(
                (file_ms - timeline_ms).abs() <= 20,
                "{step_us} µs: file {file_ms} ms, QPC {timeline_ms} ms"
            );
            assert_eq!(i64::from(mixer.elapsed_ms()), file_ms, "{step_us} µs");
        }
    }

    #[test]
    fn una_registrazione_di_entrambi_con_pausa_rilegge_con_la_durata_senza_la_pausa() {
        let dir = temp_dir("mixer-registrazione");
        let (mic, system) = ((44_100, 1), (48_000, 2));
        let combinations = [8_000, 16_000, 24_000, 48_000]
            .into_iter()
            .flat_map(|rate| [(rate, 1), (rate, 2)]);
        for (rate, channels) in combinations {
            let path = dir.join(format!("{rate}-{channels}.ogg"));
            let mut writer =
                OggOpusWriter::new(std::fs::File::create(&path).unwrap(), rate, channels, 32)
                    .unwrap();
            let mut mixer = Mixer::new(0, &[mic, system], (rate, channels)).unwrap();
            let mut out = Vec::new();
            // Il sistema suona per 0,5 s, poi tace; il microfono continua.
            let sound = sine(system.0, system.1, 0.5);
            let mut ts = 0;
            for k in 0..120 {
                mixer.advance(ts, false, &mut out);
                mixer.push(0, ts, &sine(mic.0, mic.1, 0.01), &mut out);
                if let Some(block) = sound.chunks(960).nth(k) {
                    mixer.push(1, ts, block, &mut out);
                }
                ts += 10 * MS;
            }
            let ts = capture_paused(&mut mixer, 0, mic, ts, 2.0, &mut out);
            let end = capture(&mut mixer, 0, mic, ts, 0.8, &mut out);
            mixer.finish(end, &mut out);
            writer.write(&out).unwrap();
            writer.finish().unwrap();
            let seconds = decoded_seconds(&path);
            assert!(
                (seconds - 2.0).abs() < 0.001,
                "{rate} Hz, {channels} canali: {seconds} s"
            );
        }
    }
}
