use std::path::{Path, PathBuf};

use super::*;
use crate::audio_toolkit::decode::Decoder;
use crate::audio_toolkit::processing::PcmStream;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn weights() -> PathBuf {
    root().join(WEIGHTS_FILE)
}

/// Il parlato della fixture, mono a 16 kHz.
fn voice() -> Vec<f32> {
    let mut decoder = Decoder::open(&root().join("tests/fixtures/parlato-it.wav")).unwrap();
    let mut voice = Vec::new();
    while let Some(block) = decoder.next_block().unwrap() {
        assert_eq!(block.rate, 16_000);
        voice.extend(block.mono());
    }
    voice
}

/// Spinge `input` a blocchi irregolari e chiude con `boundary`.
fn clean(stream: &mut PcmStream, input: &[f32], channels: usize, boundary: Boundary) -> Vec<f32> {
    let mut output = Vec::new();
    for block in input.chunks(channels * 137) {
        output.extend(stream.push(block).unwrap().samples);
    }
    output.extend(stream.boundary(boundary).unwrap().samples);
    output
}

fn stream(format: Format) -> PcmStream {
    PcmStream::new(format, Box::new(RNNoise::new(&weights(), format).unwrap())).unwrap()
}

#[test]
fn durata_e_coda_completa_a_ogni_frequenza_e_a_ogni_confine() {
    for (rate, channels) in [
        (16_000, 1),
        (44_100, 1),
        (48_000, 1),
        (44_100, 2),
        (48_000, 2),
    ] {
        let format = Format { rate, channels };
        let input: Vec<f32> = (0..(rate / 5 + 13) as usize)
            .flat_map(|i| (0..channels).map(move |ch| 0.2 * (i as f32 * 0.03 + ch as f32).sin()))
            .collect();
        let mut stream = stream(format);
        // Pausa e Configurazione scaricano e lasciano ripartire lo stesso processore.
        for boundary in [Boundary::Pause, Boundary::Configuration, Boundary::Finish] {
            let output = clean(&mut stream, &input, channels, boundary);
            assert_eq!(
                output.len(),
                input.len(),
                "{rate} Hz, {channels} canali, {boundary:?}"
            );
            assert!(output.iter().all(|x| x.is_finite()));
        }
    }
}

#[test]
fn silenzio_digitale_invariato_e_canali_indipendenti() {
    let format = Format {
        rate: 16_000,
        channels: 2,
    };
    // Il parlato a sinistra, silenzio digitale a destra: lo stato di un canale non tocca l'altro.
    let input: Vec<f32> = voice().into_iter().flat_map(|x| [x, 0.0]).collect();
    let output = clean(&mut stream(format), &input, 2, Boundary::Finish);
    assert_eq!(output.len(), input.len());
    assert!(output.iter().skip(1).step_by(2).all(|&x| x == 0.0));
    assert!(output.iter().step_by(2).any(|x| x.abs() > 0.01));
}

#[test]
fn il_ritardo_del_modello_e_compensato() {
    let format = Format {
        rate: RATE,
        channels: 1,
    };
    // Un impulso nel parlato: esce nello stesso campione in cui è entrato.
    let mut input = Vec::new();
    let mut up = Resampler::new(16_000, RATE, 1).unwrap();
    let voice = voice();
    up.push(&voice, &mut input);
    up.finish(&mut input);
    let at = input.len() / 2;
    input[at] += 0.9;
    let output = clean(&mut stream(format), &input, 1, Boundary::Finish);
    let peak = (at - 2000..at + 2000)
        .max_by(|&a, &b| output[a].abs().total_cmp(&output[b].abs()))
        .unwrap();
    assert_eq!(
        peak,
        at,
        "impulso spostato di {} campioni",
        peak as isize - at as isize
    );
}

#[test]
fn conserva_la_voce_della_fixture() {
    let format = Format {
        rate: 16_000,
        channels: 1,
    };
    let input = voice();
    let output = clean(&mut stream(format), &input, 1, Boundary::Finish);
    let energy = |x: &[f32]| x.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>();
    let ratio = 10.0 * (energy(&output) / energy(&input)).log10();
    let cross: f64 = input
        .iter()
        .zip(&output)
        .map(|(&a, &b)| f64::from(a) * f64::from(b))
        .sum();
    let correlation = cross / (energy(&input) * energy(&output)).sqrt();
    assert!(ratio > -3.0, "voce attenuata di {ratio:.2} dB");
    assert!(
        correlation > 0.9,
        "voce alterata: correlazione {correlation:.3}"
    );
}

#[test]
fn pesi_assenti_o_diversi_sono_errori_distinti() {
    let format = Format {
        rate: 16_000,
        channels: 1,
    };
    let missing = std::env::temp_dir().join("memotape-rnnoise-mancante.bin");
    assert!(matches!(
        RNNoise::new(&missing, format),
        Err(AppError::AudioCleaningMissing)
    ));
    // Stessa dimensione dei pesi veri, contenuto diverso: lo SHA-256 lo rifiuta prima del C.
    let mut bytes = std::fs::read(weights()).unwrap();
    bytes[WEIGHTS_SIZE / 2] ^= 1;
    let altered = std::env::temp_dir().join("memotape-rnnoise-alterato.bin");
    std::fs::write(&altered, bytes).unwrap();
    let result = RNNoise::new(&altered, format);
    std::fs::remove_file(&altered).unwrap();
    assert!(matches!(
        result,
        Err(AppError::AudioCleaningIncompatible(_))
    ));
}

/// La prova d'ascolto: DFN3 restituiva silenzio digitale sul campione del corpus con la voce
/// attenuata di 30 dB (la fixture per 0,031622776) e Silero non trovava più nessuna Frase.
#[test]
fn la_voce_attenuata_di_30_db_resta_una_frase_per_silero() {
    use crate::audio_toolkit::resample::FrameResampler;
    use crate::audio_toolkit::segmenter::{Event, Params, Segmenter};
    use crate::audio_toolkit::vad::{Silero, VoiceDetector};
    let format = Format {
        rate: 16_000,
        channels: 1,
    };
    let weak: Vec<f32> = voice().iter().map(|x| x * 0.031_622_776).collect();
    let output = clean(&mut stream(format), &weak, 1, Boundary::Finish);
    let mut resampler = FrameResampler::new(16_000).unwrap();
    let mut frames = resampler.push(&output);
    frames.extend(resampler.finish());
    let mut silero = Silero::new(&root().join("resources/silero_vad.onnx")).unwrap();
    let probabilities: Vec<_> = frames
        .iter()
        .map(|frame| silero.probability(frame).unwrap())
        .collect();
    let mut segmenter = Segmenter::new(Params::default());
    assert!(
        frames.into_iter().zip(probabilities).any(|(frame, p)| {
            segmenter
                .push(frame, p)
                .iter()
                .any(|e| matches!(e, Event::PhraseStart(_)))
        }),
        "nessuna Frase dopo la pulizia"
    );
}
