//! Mixer della Registrazione: posiziona i blocchi di ogni ingresso (microfono, audio di sistema)
//! per timestamp di cattura rispetto all'inizio della sessione, esclude le pause, riempie di
//! silenzio i buchi, porta l'audio a canali e frequenza delle impostazioni e somma gli ingressi.
//! Con gli Ingressi separati dà anche l'audio di ogni ingresso, allineato al mix.
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
/// Quanto dura il passaggio a un nuovo Guadagno: abbastanza da non sentire uno scatto.
const GUADAGNO_RAMP_NS: u64 = 20_000_000;
/// -3 dB: il peso del canale centrale e dei surround nel downmix.
const MINUS_3_DB: f32 = std::f32::consts::FRAC_1_SQRT_2;

pub struct Mixer {
    inputs: Vec<Input>,
    mix: Mix,
    /// Inizio della sessione (QPC, ns).
    origin: u64,
    /// Il tempo escluso: pause e sospensioni.
    paused_ns: u64,
    /// Da quando è in pausa; `None` se non lo è.
    pause_start: Option<u64>,
    /// L'ultimo `now` di `advance`.
    last_now: u64,
    /// Un blocco nei canali della Registrazione, prima del ricampionamento.
    converted: Vec<f32>,
    /// Con gli Ingressi separati, l'audio pronto di ogni ingresso, come in `out` per il mix.
    tracks: Vec<Vec<f32>>,
}

struct Input {
    rate: u64,
    channels: usize,
    resampler: Resampler,
    /// Frame alla frequenza del dispositivo già sulla linea del tempo, silenzio compreso.
    frames: u64,
    /// Frame alla frequenza della Registrazione già sommati nel mix.
    mixed: usize,
    /// Il picco dopo il Guadagno.
    peak: f32,
    /// Il Guadagno (lineare) applicato adesso, quello verso cui va e di quanto si muove a frame.
    guadagno: f32,
    guadagno_target: f32,
    guadagno_step: f32,
    resampled: Vec<f32>,
    /// Per il log a fine Registrazione: frame di silenzio inseriti e scartati, e per misurare la
    /// deriva del clock del dispositivo rispetto a QPC i frame ricevuti (pause comprese) tra il
    /// timestamp del primo blocco e la fine dell'ultimo.
    silence: u64,
    dropped: u64,
    received: u64,
    first_ns: Option<u64>,
    last_end_ns: u64,
    /// Con gli Ingressi separati, l'audio ricampionato non ancora uscito, dal frame `Mix::emitted`.
    pending: Option<Vec<f32>>,
}

/// I campioni sommati non ancora usciti, interleaved nei canali della Registrazione, dal frame
/// `emitted` in poi.
struct Mix {
    samples: Vec<f32>,
    emitted: usize,
    channels: usize,
}

impl Mix {
    /// Somma `resampled` dopo i `mixed` frame che l'ingresso ha già sommato, e li conta.
    fn add(&mut self, mixed: &mut usize, resampled: &[f32]) {
        let start = (*mixed - self.emitted) * self.channels;
        let end = start + resampled.len();
        if self.samples.len() < end {
            self.samples.resize(end, 0.0);
        }
        for (m, s) in self.samples[start..end].iter_mut().zip(resampled) {
            *m += s;
        }
        *mixed += resampled.len() / self.channels;
    }

    /// Accoda in `out`, con il clamp a [-1, 1], i frame fino a `ready`.
    fn emit(&mut self, ready: usize, out: &mut Vec<f32>) {
        let n = (ready - self.emitted) * self.channels;
        out.extend(self.samples.drain(..n).map(|s| s.clamp(-1.0, 1.0)));
        self.emitted = ready;
    }
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

    /// Ricampiona `samples` (già nei canali della Registrazione) e li somma nel mix.
    fn feed(&mut self, samples: &[f32], mix: &mut Mix) {
        self.frames += (samples.len() / mix.channels) as u64;
        self.resampled.clear();
        self.resampler.push(samples, &mut self.resampled);
        self.add(mix);
    }

    /// Somma nel mix `resampled`, e lo tiene per l'audio dell'ingresso.
    fn add(&mut self, mix: &mut Mix) {
        mix.add(&mut self.mixed, &self.resampled);
        if let Some(pending) = &mut self.pending {
            pending.extend_from_slice(&self.resampled);
        }
    }

    /// Applica il Guadagno ai frame di `samples` (nei canali `channels`), andando verso quello
    /// scelto a passi di `guadagno_step`.
    fn apply_guadagno(&mut self, samples: &mut [f32], channels: usize) {
        for frame in samples.chunks_exact_mut(channels) {
            if self.guadagno != self.guadagno_target {
                self.guadagno += self.guadagno_step;
                if (self.guadagno_target - self.guadagno) * self.guadagno_step <= 0.0 {
                    self.guadagno = self.guadagno_target;
                }
            }
            for s in frame {
                *s *= self.guadagno;
            }
        }
    }

    fn feed_silence(&mut self, frames: u64, mix: &mut Mix) {
        self.silence += frames;
        self.feed(&vec![0.0; frames as usize * mix.channels], mix);
    }
}

/// Accoda in `out` un frame del dispositivo nei canali della Registrazione. Oltre due canali vale
/// l'ordine di WASAPI (FL, FR, FC, LFE, BL, BR, SL, SR): centrale e surround vanno a sinistra e a
/// destra a -3 dB, il subwoofer no; in mono si fa la media di sinistra e destra.
/// ponytail: l'ordine si assume, non si legge dalla maschera dei canali (un quad lo sbaglia).
pub(crate) fn downmix(frame: &[f32], out_channels: usize, out: &mut Vec<f32>) {
    let (left, right) = match frame {
        [] => return,
        [mono] => (*mono, *mono),
        [left, right] => (*left, *right),
        [left, right, center, rest @ ..] => {
            let center = center * MINUS_3_DB;
            // Dopo il subwoofer i surround alternano sinistra e destra.
            let (sl, sr) = rest
                .iter()
                .skip(1)
                .enumerate()
                .fold(
                    (0.0, 0.0),
                    |(l, r), (i, s)| {
                        if i % 2 == 0 { (l + s, r) } else { (l, r + s) }
                    },
                );
            (
                left + center + sl * MINUS_3_DB,
                right + center + sr * MINUS_3_DB,
            )
        }
    };
    if out_channels == 1 {
        out.push(f32::midpoint(left, right));
    } else {
        out.extend([left, right]);
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
                    guadagno: 1.0,
                    guadagno_target: 1.0,
                    guadagno_step: 0.0,
                    resampled: Vec::new(),
                    silence: 0,
                    dropped: 0,
                    received: 0,
                    first_ns: None,
                    last_end_ns: 0,
                    pending: None,
                })
            })
            .collect::<Result<_, AppError>>()?;
        Ok(Self {
            inputs,
            mix: Mix {
                samples: Vec::new(),
                emitted: 0,
                channels: out_channels,
            },
            origin: origin_ns,
            paused_ns: 0,
            pause_start: None,
            last_now: origin_ns,
            converted: Vec::new(),
            tracks: Vec::new(),
        })
    }

    /// Dà anche l'audio di ogni ingresso, in `tracks`.
    #[must_use]
    pub fn with_tracks(mut self) -> Self {
        for input in &mut self.inputs {
            input.pending = Some(Vec::new());
        }
        self.tracks = vec![Vec::new(); self.inputs.len()];
        self
    }

    /// Con gli Ingressi separati, l'audio pronto di ogni ingresso dall'ultimo svuotamento, nei canali
    /// e alla frequenza della Registrazione: lo stesso tratto della linea del tempo uscito in `out`
    /// per il mix, silenzio nei buchi e pause escluse. Lo svuota chi lo usa. Vuoto senza
    /// `with_tracks`.
    pub fn tracks(&mut self) -> &mut [Vec<f32>] {
        &mut self.tracks
    }

    /// Un blocco dell'ingresso `input`: `capture_ns` è il suo timestamp di cattura, `samples` i
    /// campioni interleaved del dispositivo. In pausa si scarta. Accoda in `out` l'audio pronto.
    pub fn push(&mut self, input: usize, capture_ns: u64, samples: &[f32], out: &mut Vec<f32>) {
        let t = self.timeline(capture_ns);
        let paused = self.pause_start.is_some();
        let Self {
            inputs,
            mix,
            converted,
            ..
        } = self;
        let input = &mut inputs[input];
        let frames = samples.len() / input.channels;
        input.received += frames as u64;
        input.first_ns.get_or_insert(capture_ns);
        input.last_end_ns = capture_ns + frames as u64 * NS_PER_S / input.rate;
        if paused {
            return;
        }
        let start = input.frame_at(t);
        let position = input.frames as i64;
        let tolerance = input.frame_at(HOLE_NS as i64);
        let mut skip = 0;
        if start - position >= tolerance {
            input.feed_silence((start - position) as u64, mix);
        } else if position - start >= tolerance {
            // Sovrapposto a quanto già scritto (o prima dell'inizio): la parte in più si scarta.
            skip = ((position - start) as usize).min(frames);
        }
        input.dropped += skip as u64;
        converted.clear();
        for frame in samples[skip * input.channels..].chunks_exact(input.channels) {
            downmix(frame, mix.channels, converted);
        }
        input.apply_guadagno(converted, mix.channels);
        // Il picco dei campioni del dispositivo, con il Guadagno a fine blocco.
        let raw = samples.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
        input.peak = input.peak.max(raw * input.guadagno).min(1.0);
        input.feed(converted, mix);
        self.emit(out);
    }

    /// L'orologio (QPC, ns) e lo stato della pausa, da chiamare di continuo (almeno ogni 100 ms).
    /// Conta pause e sospensioni e riempie di silenzio gli ingressi rimasti indietro, come il
    /// loopback a riproduzione ferma. All'inizio di una pausa li completa fino a lì, così in `out`
    /// esce subito tutto l'audio prima della pausa. Accoda in `out` l'audio pronto.
    pub fn advance(&mut self, now_ns: u64, paused: bool, out: &mut Vec<f32>) {
        if paused {
            if self.pause_start.is_none() {
                self.pause_start = Some(now_ns);
                // I blocchi arrivati in pausa si scartano: quello che manca fin qui è silenzio.
                self.fill_to(self.timeline(now_ns), HOLE_NS);
                self.emit(out);
            }
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
        for input in &mut self.inputs {
            input.resampled.clear();
            input.resampler.finish(&mut input.resampled);
            input.add(&mut self.mix);
            // Con una consegna continua (il microfono, il loopback durante una riproduzione) la
            // differenza tra le due durate è la deriva del clock del dispositivo rispetto a QPC.
            log::info!(
                "ingresso {} Hz: ricevuti {} ms in {} ms di timestamp; {} ms di silenzio inseriti, {} ms scartati",
                input.rate,
                input.received * 1_000 / input.rate,
                input
                    .first_ns
                    .map_or(0, |first| input.last_end_ns.saturating_sub(first)
                        / 1_000_000),
                input.silence * 1_000 / input.rate,
                input.dropped * 1_000 / input.rate,
            );
        }
        let last = self.inputs.iter().map(|i| i.mixed).max().unwrap_or(0);
        self.mix
            .samples
            .resize((last - self.mix.emitted) * self.mix.channels, 0.0);
        self.emit_to(last, out);
    }

    /// La durata registrata, pause escluse, dai timestamp.
    pub fn elapsed_ms(&self) -> u32 {
        let end = self.inputs.iter().map(Input::end_ns).max().unwrap_or(0);
        u32::try_from(end / 1_000_000).unwrap_or(u32::MAX)
    }

    /// Il Guadagno (lineare) dell'ingresso `input`. Prima del suo primo audio vale da subito,
    /// poi ci arriva in `GUADAGNO_RAMP_NS`, senza scatti.
    pub fn set_guadagno(&mut self, input: usize, guadagno: f32) {
        let input = &mut self.inputs[input];
        input.guadagno_target = guadagno;
        if input.received == 0 {
            input.guadagno = guadagno;
        }
        let ramp = (input.rate * GUADAGNO_RAMP_NS / NS_PER_S).max(1);
        input.guadagno_step = (guadagno - input.guadagno) / ramp as f32;
    }

    /// Il picco (0–1) di ogni ingresso, dopo il Guadagno, dalla chiamata precedente.
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
        for input in &mut self.inputs {
            let missing = input.frame_at(t_ns) - input.frames as i64;
            if missing > 0 && missing >= input.frame_at(tolerance_ns as i64) {
                input.feed_silence(missing as u64, &mut self.mix);
            }
        }
    }

    /// Accoda in `out` i frame che tutti gli ingressi hanno già sommato.
    fn emit(&mut self, out: &mut Vec<f32>) {
        let ready = self.inputs.iter().map(|i| i.mixed).min().unwrap_or(0);
        self.emit_to(ready, out);
    }

    /// Accoda in `out` il mix fino al frame `ready`, e lo stesso tratto di ogni ingresso in `tracks`
    /// (completato di silenzio a fine sessione).
    fn emit_to(&mut self, ready: usize, out: &mut Vec<f32>) {
        let n = (ready - self.mix.emitted) * self.mix.channels;
        for (input, track) in self.inputs.iter_mut().zip(&mut self.tracks) {
            if let Some(pending) = &mut input.pending {
                pending.resize(pending.len().max(n), 0.0);
                track.extend(pending.drain(..n).map(|s| s.clamp(-1.0, 1.0)));
            }
        }
        self.mix.emit(ready, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{decoded_seconds, sine, temp_dir};

    const MS: u64 = 1_000_000;

    /// Spinge `seconds` di sinusoide nell'ingresso `input` in blocchi da 10 ms a partire da
    /// `start_ns`, come un dispositivo, con l'orologio che avanza a ogni blocco (in pausa se
    /// `paused`: i blocchi arrivano e si scartano); restituisce il timestamp dopo l'ultimo blocco.
    fn capture(
        mixer: &mut Mixer,
        input: usize,
        (rate, channels): (u32, usize),
        start_ns: u64,
        seconds: f64,
        paused: bool,
        out: &mut Vec<f32>,
    ) -> u64 {
        let block = rate as usize / 100 * channels;
        let mut ts = start_ns;
        for chunk in sine(rate, channels, seconds).chunks(block) {
            // Come il worker: prima l'orologio, poi il blocco.
            mixer.advance(ts, paused, out);
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
        let end = capture(&mut mixer, 0, device, 5_000 * MS, 2.0, false, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(out.len(), 32_000);
        assert_eq!(mixer.elapsed_ms(), 2_000);
    }

    #[test]
    fn le_pause_non_contano_ne_nel_timer_ne_nell_audio() {
        let device = (48_000, 1);
        let mut mixer = Mixer::new(0, &[device], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, 0, device, 0, 1.0, false, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_000);
        // 3 s di pausa: i blocchi arrivano ma si scartano e il timer resta fermo.
        let ts = capture(&mut mixer, 0, device, ts, 3.0, true, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_000);
        // Dopo Riprendi l'audio continua senza vuoti.
        let end = capture(&mut mixer, 0, device, ts, 0.5, false, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(mixer.elapsed_ms(), 1_500);
        assert_eq!(out.len(), 72_000);
    }

    #[test]
    fn all_inizio_della_pausa_esce_tutto_l_audio_prima_della_pausa() {
        // Microfono e loopback a riproduzione ferma, che trattiene il mix fino a `LAG_NS`.
        let device = (48_000, 1);
        let mut mixer = Mixer::new(0, &[device, device], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, 0, device, 0, 1.0, false, &mut out);
        assert!(out.len() < 48_000);
        mixer.advance(ts, true, &mut out);
        assert_eq!(out.len(), 48_000);
        // Dopo Riprendi l'audio continua da lì.
        let end = capture(&mut mixer, 0, device, ts + 2_000 * MS, 0.5, false, &mut out);
        mixer.finish(end, &mut out);
        assert_eq!(out.len(), 72_000);
    }

    #[test]
    fn un_buco_tra_i_blocchi_diventa_silenzio() {
        let device = (16_000, 1);
        let mut mixer = Mixer::new(0, &[device], (16_000, 1)).unwrap();
        let mut out = Vec::new();
        let ts = capture(&mut mixer, 0, device, 0, 1.0, false, &mut out);
        // 250 ms senza pacchetti (discontinuità), poi l'audio riprende.
        let end = capture(&mut mixer, 0, device, ts + 250 * MS, 1.0, false, &mut out);
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
        let mut ts = capture(&mut mixer, 0, device, 0, 1.0, false, &mut out);
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
        let end = capture(&mut mixer, 0, device, ts, 1.0, false, &mut out);
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
        let ts = capture(&mut mixer, 0, device, 0, 1.0, false, &mut out);
        // Il PC sospeso per un'ora: niente ore di silenzio in memoria né nel file.
        let end = capture(
            &mut mixer,
            0,
            device,
            ts + 3_600_000 * MS,
            1.0,
            false,
            &mut out,
        );
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
            let ts = capture(&mut mixer, 0, mic, ts, 2.0, true, &mut out);
            let end = capture(&mut mixer, 0, mic, ts, 0.8, false, &mut out);
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

    #[test]
    fn con_gli_ingressi_separati_ogni_ingresso_esce_allineato_al_mix() {
        let (mic, system) = ((44_100, 1), (48_000, 2));
        let mut mixer = Mixer::new(0, &[mic, system], (48_000, 1))
            .unwrap()
            .with_tracks();
        let mut out = Vec::new();
        let mut tracks = [Vec::new(), Vec::new()];
        let mut collect = |mixer: &mut Mixer| {
            for (all, ready) in tracks.iter_mut().zip(mixer.tracks()) {
                all.append(ready);
            }
        };
        // Microfono continuo per 1 s; il loopback consegna solo tra 0,2 s e 0,4 s.
        for k in 0..100u64 {
            let ts = k * 10 * MS;
            mixer.advance(ts, false, &mut out);
            mixer.push(0, ts, &[0.25; 441], &mut out);
            if (20..40).contains(&k) {
                mixer.push(1, ts, &[0.5; 960], &mut out);
            }
            collect(&mut mixer);
        }
        // 2 s di pausa, poi altri 0,5 s di microfono.
        let ts = capture(&mut mixer, 0, mic, 1_000 * MS, 2.0, true, &mut out);
        collect(&mut mixer);
        let end = capture(&mut mixer, 0, mic, ts, 0.5, false, &mut out);
        mixer.finish(end, &mut out);
        collect(&mut mixer);
        // Pause escluse, ogni Ingresso lungo quanto il mix e il mix è la loro somma.
        assert_eq!(out.len(), 72_000);
        for track in &tracks {
            assert_eq!(track.len(), out.len());
        }
        assert!(
            out.iter()
                .zip(tracks[0].iter().zip(&tracks[1]))
                .all(|(m, (a, b))| (m - (a + b)).abs() < 1e-6)
        );
        // Il buco del loopback è silenzio, dove consegna c'è il suo audio.
        let at = |track: &[f32], seconds: f64| track[(seconds * 48_000.0) as usize];
        assert!(at(&tracks[1], 0.1).abs() < 1e-4);
        assert!((at(&tracks[1], 0.3) - 0.5).abs() < 0.01);
        assert!(at(&tracks[1], 0.7).abs() < 1e-4);
        assert!((at(&tracks[0], 0.3) - 0.25).abs() < 0.01);
    }

    #[test]
    fn il_guadagno_di_un_ingresso_cambia_il_mix_e_il_suo_livello() {
        let mut mixer = Mixer::new(0, &[(48_000, 1), (48_000, 1)], (48_000, 1)).unwrap();
        // +6 dB al microfono (× 2), 0 dB all'audio di sistema.
        mixer.set_guadagno(0, 2.0);
        let mut out = Vec::new();
        mixer.push(0, 0, &[0.2; 480], &mut out);
        mixer.push(1, 0, &[0.1; 480], &mut out);
        assert_eq!(out.len(), 480);
        assert!(
            out.iter().all(|s| (s - 0.5).abs() < 1e-6),
            "{:?}",
            &out[..4]
        );
        let peaks = mixer.take_peaks();
        assert!((peaks[0] - 0.4).abs() < 1e-6 && (peaks[1] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn un_guadagno_cambiato_durante_la_registrazione_arriva_senza_scatti() {
        let mut mixer = Mixer::new(0, &[(48_000, 1)], (48_000, 1))
            .unwrap()
            .with_tracks();
        let mut out = Vec::new();
        let mut track = Vec::new();
        for k in 0..100u64 {
            if k == 50 {
                // Da 0 a +12 dB (× 4) a metà.
                mixer.set_guadagno(0, 4.0);
            }
            mixer.push(0, k * 10 * MS, &[0.1; 480], &mut out);
            track.append(&mut mixer.tracks()[0]);
        }
        mixer.finish(1_000 * MS, &mut out);
        track.append(&mut mixer.tracks()[0]);
        assert_eq!(out.len(), 48_000);
        assert!((out[24_000 - 1] - 0.1).abs() < 1e-6);
        assert!((out[47_999] - 0.4).abs() < 1e-6);
        let jump = out
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f32::max);
        assert!(jump < 0.001, "salto di {jump}");
        // L'audio dell'Ingresso, con gli Ingressi separati, è quello con il Guadagno.
        assert_eq!(track, out);
    }

    #[test]
    fn senza_ingressi_separati_non_c_e_l_audio_di_ogni_ingresso() {
        let mut mixer = Mixer::new(0, &[(48_000, 1), (48_000, 1)], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &[0.5; 480], &mut out);
        mixer.push(1, 0, &[0.5; 480], &mut out);
        assert_eq!(out.len(), 480);
        assert!(mixer.tracks().is_empty());
    }

    #[test]
    fn un_blocco_sovrapposto_a_quanto_gia_scritto_si_scarta() {
        let device = (48_000, 1);
        let mut mixer = Mixer::new(0, &[device], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &[0.5; 4_800], &mut out);
        // Un blocco da 100 ms che parte a 50 ms: i primi 50 ms sono già scritti e si scartano.
        mixer.push(0, 50 * MS, &[-0.5; 4_800], &mut out);
        // Un blocco già scritto per intero (timestamp all'indietro) si scarta tutto.
        mixer.push(0, 60 * MS, &[0.9; 480], &mut out);
        mixer.finish(150 * MS, &mut out);
        assert_eq!(mixer.elapsed_ms(), 150);
        assert_eq!(out.len(), 7_200);
        assert!(out[..4_800].iter().all(|&s| s == 0.5));
        assert!(out[4_800..].iter().all(|&s| s == -0.5));
    }

    #[test]
    fn il_loopback_multicanale_tiene_centrale_e_surround() {
        // 5.1 nell'ordine di WASAPI: FL, FR, FC, LFE, BL, BR.
        let frame = [0.1, 0.2, 0.4, 0.9, 0.3, 0.0];
        let mut mixer = Mixer::new(0, &[(48_000, 6)], (48_000, 2)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &frame, &mut out);
        // Centrale e surround a -3 dB (× 0,7071), subwoofer escluso.
        let (left, right) = (0.1 + 0.282_84 + 0.212_13, 0.2 + 0.282_84);
        assert!(
            (out[0] - left).abs() < 1e-4 && (out[1] - right).abs() < 1e-4,
            "{out:?}"
        );
        let mut mixer = Mixer::new(0, &[(48_000, 6)], (48_000, 1)).unwrap();
        let mut out = Vec::new();
        mixer.push(0, 0, &frame, &mut out);
        assert!((out[0] - (left + right) / 2.0).abs() < 1e-4, "{out:?}");
    }
}
