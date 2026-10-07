//! Misura manuale del percorso effettivo: nessuna cattura, ASR o modifica alle Impostazioni.

use super::*;
use crate::audio_toolkit::decode::Decoder;
use std::time::Instant;

#[test]
#[ignore = "benchmark DFN3 reale: eseguire da solo, senza compilazioni o ASR concorrenti"]
fn dfn3_costo_reale_per_ingresso() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut decoder = Decoder::open(&root.join("tests/fixtures/parlato-it.wav")).unwrap();
    let mut voice = Vec::new();
    while let Some(block) = decoder.next_block().unwrap() {
        assert_eq!(block.rate, 16_000);
        voice.extend(block.mono());
    }
    let path = root.join(MODEL_FILE);
    for (rate, channels, ingressi, worker) in [
        (48_000, 1, 1, false),
        (48_000, 1, 1, true),
        (16_000, 1, 2, true),
        (48_000, 2, 2, true),
    ] {
        let format = Format { rate, channels };
        let mut mono = Vec::new();
        let mut resampler = Resampler::new(16_000, rate, 1).unwrap();
        resampler.push(&voice, &mut mono);
        resampler.finish(&mut mono);
        let input: Vec<f32> = mono
            .iter()
            .flat_map(|&sample| std::iter::repeat_n(sample, channels))
            .collect();
        if rate == 48_000 && channels == 1 && !worker {
            let folder = root
                .parent()
                .unwrap()
                .join(".scratch/diagnosi-registrazione");
            std::fs::create_dir_all(&folder).unwrap();
            let bytes: Vec<u8> = input
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .collect();
            std::fs::write(folder.join("benchmark-voce-48k.f32"), bytes).unwrap();
        }
        let started = Instant::now();
        let mut direct = if worker {
            None
        } else {
            Some(State::new(&path, format).unwrap())
        };
        let mut filters: Vec<_> = if worker {
            (0..ingressi)
                .map(|_| DeepFilter::new(&path, format).unwrap())
                .collect()
        } else {
            Vec::new()
        };
        let init = started.elapsed().as_secs_f64();
        let mut out = Vec::new();
        let mut emitted = 0;
        let size = rate as usize / 100 * channels;
        // Un secondo di riscaldamento, fuori dalla misura del flusso continuo.
        for samples in input[..rate as usize * channels].chunks(size) {
            let block = PcmBlock {
                format,
                start_frame: 0,
                samples,
            };
            if let Some(state) = &mut direct {
                state.process(block, &mut out).unwrap();
            }
            for filter in &mut filters {
                filter.process(block, &mut out).unwrap();
            }
            emitted += out.len();
            out.clear();
        }
        let mut timings = Vec::new();
        let started = Instant::now();
        for samples in input.chunks(size) {
            let tick = Instant::now();
            let block = PcmBlock {
                format,
                start_frame: 0,
                samples,
            };
            if let Some(state) = &mut direct {
                state.process(block, &mut out).unwrap();
            }
            for filter in &mut filters {
                filter.process(block, &mut out).unwrap();
            }
            timings.push(tick.elapsed().as_secs_f64());
            assert!(out.iter().all(|sample| sample.is_finite()));
            emitted += out.len();
            out.clear();
        }
        let seconds = started.elapsed().as_secs_f64();
        timings.sort_by(f64::total_cmp);
        let finish = Instant::now();
        if let Some(state) = &mut direct {
            state.flush(Boundary::Finish, &mut out).unwrap();
        }
        for filter in &mut filters {
            filter.flush(Boundary::Finish, &mut out).unwrap();
        }
        let flush = finish.elapsed().as_secs_f64();
        let audio = input.len() as f64 / f64::from(rate) / channels as f64;
        let expected = (input.len() + rate as usize * channels) * ingressi;
        assert_eq!(emitted + out.len(), expected);
        println!(
            "DFN3_PERF rate={rate} channels={channels} ingressi={ingressi} worker={worker} init_s={init:.4} audio_s={audio:.4} process_s={seconds:.4} rtf={:.4} block_p95_ms={:.3} block_p99_ms={:.3} block_max_ms={:.3} flush_s={flush:.4}",
            seconds / audio,
            timings[timings.len() * 95 / 100] * 1000.0,
            timings[timings.len() * 99 / 100] * 1000.0,
            timings[timings.len() - 1] * 1000.0,
        );
    }
}
