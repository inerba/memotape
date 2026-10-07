//! Banco diagnostico su un file reale esterno, senza modificare Impostazioni o Libreria.

use super::*;
use std::io::Write;

fn write_wave(path: &Path, format: Format, samples: &[f32]) {
    let mut file = std::fs::File::create(path).unwrap();
    let bytes = u32::try_from(samples.len() * 2).unwrap();
    let channels = format.channels as u16;
    file.write_all(b"RIFF").unwrap();
    file.write_all(&(36 + bytes).to_le_bytes()).unwrap();
    file.write_all(b"WAVEfmt ").unwrap();
    file.write_all(&16_u32.to_le_bytes()).unwrap();
    file.write_all(&1_u16.to_le_bytes()).unwrap();
    file.write_all(&channels.to_le_bytes()).unwrap();
    file.write_all(&format.rate.to_le_bytes()).unwrap();
    file.write_all(&(format.rate * u32::from(channels) * 2).to_le_bytes())
        .unwrap();
    file.write_all(&(channels * 2).to_le_bytes()).unwrap();
    file.write_all(&16_u16.to_le_bytes()).unwrap();
    file.write_all(b"data").unwrap();
    file.write_all(&bytes.to_le_bytes()).unwrap();
    for &sample in samples {
        file.write_all(&((sample.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes())
            .unwrap();
    }
}

#[test]
#[ignore = "DFN3 reale: MEMOTAPE_DFN3_SAMPLE, eseguire separatamente"]
fn dfn3_campione_reale_confronta_pcm_e_modello() {
    use crate::audio_toolkit::decode::Decoder;
    let sample = std::env::var("MEMOTAPE_DFN3_SAMPLE").unwrap();
    let mut decoder = Decoder::open(Path::new(&sample)).unwrap();
    let mut input = Vec::new();
    let mut format = None;
    while let Some(block) = decoder.next_block().unwrap() {
        let current = Format {
            rate: block.rate,
            channels: block.channels,
        };
        if let Some(format) = format {
            assert_eq!(format, current);
        }
        format = Some(current);
        input.extend(block.samples);
    }
    let format = format.unwrap();
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join(MODEL_FILE);
    use crate::audio_toolkit::processing::PcmStream;
    let mut app =
        PcmStream::new(format, Box::new(DeepFilter::new(&model, format).unwrap())).unwrap();
    let mut actual = Vec::new();
    for block in input.chunks(960 * format.channels) {
        actual.extend(app.push(block).unwrap().samples);
    }
    actual.extend(app.boundary(Boundary::Finish).unwrap().samples);
    let mut native = State::new(&model, format).unwrap();
    let mut at_48 = Vec::new();
    native.to_48.push(&input, &mut at_48);
    native.to_48.finish(&mut at_48);
    native.infer(&at_48, true).unwrap();
    let mut tail = Vec::new();
    native.from_48.finish(&mut tail);
    native.wet.extend(tail);
    let wet: Vec<f32> = native.wet.iter().copied().collect();
    assert_eq!(actual.len(), input.len());
    assert_eq!(wet.len(), input.len());
    assert!(actual.iter().chain(&wet).all(|v| v.is_finite()));
    let difference = actual
        .iter()
        .zip(&wet)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(difference <= 1e-6, "differs from direct DFN3: {difference}");
    // Riferimento registrato prima della modifica e scelto dall'utente, indipendente
    // dall'implementazione corrente. La tolleranza copre soltanto l'esportazione PCM16.
    let reference_difference = std::env::var("MEMOTAPE_DFN3_REFERENCE").ok().map(|path| {
        let mut reference = Decoder::open(Path::new(&path)).unwrap();
        let mut samples = Vec::new();
        while let Some(block) = reference.next_block().unwrap() {
            assert_eq!(block.rate, format.rate);
            assert_eq!(block.channels, format.channels);
            samples.extend(block.samples);
        }
        assert_eq!(samples.len(), actual.len());
        let error = samples
            .iter()
            .zip(&actual)
            .map(|(a, b)| (a - b.clamp(-1.0, 1.0)).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            error <= 1.5 / 32768.0,
            "differs from chosen PCM16 reference: {error}"
        );
        error
    });
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".scratch/pulizia-audio/dfn3-diretto");
    std::fs::create_dir_all(&folder).unwrap();
    write_wave(&folder.join("01-originale.wav"), format, &input);
    write_wave(&folder.join("02-app-dfn3-diretto.wav"), format, &actual);
    write_wave(&folder.join("03-dfn3-diretto.wav"), format, &wet);
    let energy = |values: &[f32]| values.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>();
    let mut rows = Vec::new();
    let window = format.rate as usize * format.channels;
    for (index, original) in input.chunks(window).enumerate() {
        let start = index * window;
        let end = start + original.len();
        let original_energy = energy(original);
        rows.push(serde_json::json!({
            "secondo": index,
            "rms_originale": (original_energy / original.len() as f64).sqrt(),
            "attenuazione_app_db": 10.0 * (original_energy / energy(&actual[start..end])).log10(),
            "attenuazione_dfn3_db": 10.0 * (original_energy / energy(&wet[start..end])).log10(),
        }));
    }
    let report = serde_json::json!({
        "source": sample, "rate": format.rate, "channels": format.channels,
        "durata_s": input.len() as f64 / f64::from(format.rate) / format.channels as f64,
        "attenuazione_energia_app_db": 10.0 * (energy(&input) / energy(&actual)).log10(),
        "attenuazione_energia_dfn3_db": 10.0 * (energy(&input) / energy(&wet)).log10(),
        "scarto_massimo_app_dfn3": difference,
        "scarto_massimo_riferimento_pcm16": reference_difference,
        "per_secondo": rows,
        "nota": "Confronto di energia del segnale misto; senza voce pulita di riferimento non misura SNR o preservazione del parlato. WAV a 16 bit solo per ascolto; metriche su PCM f32 prima dell'esportazione."
    });
    std::fs::write(
        folder.join("confronto.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}
