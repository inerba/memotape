use df::tract::{DfParams, DfTract, RuntimeParams};
use ndarray::Array2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .ok_or("pass the official ONNX model archive")?
        .clone();
    if args.len() == 4 {
        return enhance_file(&path, &args[2], &args[3]);
    }
    let params = DfParams::new(path.into())?;
    let mut model = DfTract::new(params, &RuntimeParams::default())?;
    println!(
        "loaded: sr={} hop={} fft={} lookahead={}",
        model.sr, model.hop_size, model.fft_size, model.lookahead
    );
    let mut input = Array2::<f32>::zeros((1, model.hop_size));
    let mut output = input.clone();
    for frame in 0..120 {
        for (sample, value) in input.iter_mut().enumerate() {
            let t = (frame * model.hop_size + sample) as f32 / model.sr as f32;
            *value = if frame < 100 {
                0.1 * (t * 220.0 * std::f32::consts::TAU).sin()
            } else {
                0.0
            };
        }
        let snr = model.process(input.view(), output.view_mut())?;
        assert!(snr.is_finite());
        assert!(output.iter().all(|value| value.is_finite()));
    }
    model.set_atten_lim(12.0);
    model.process(input.view(), output.view_mut())?;
    assert!(output.iter().all(|value| value.is_finite()));
    println!("processed 120 hops plus attenuation change; finite output. This is a compatibility smoke test, not a quality or realtime benchmark.");
    Ok(())
}

fn enhance_file(
    model_path: &str,
    wav_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = (1_u64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let mono: Vec<f32> = samples
        .chunks_exact(spec.channels as usize)
        .map(|ch| ch.iter().sum::<f32>() / ch.len() as f32)
        .collect();
    let input = Array2::from_shape_vec((1, mono.len()), mono)?;
    let mut model = DfTract::new(DfParams::new(model_path.into())?, &RuntimeParams::default())?;
    let resampled =
        df::transforms::resample(input.view(), spec.sample_rate as usize, model.sr, None)?;
    let samples = resampled.row(0).to_vec();
    let delay = model.fft_size - model.hop_size + model.lookahead * model.hop_size;
    let frames = (samples.len() + delay).div_ceil(model.hop_size);
    let mut input_hop = Array2::<f32>::zeros((1, model.hop_size));
    let mut output_hop = input_hop.clone();
    let mut output = Vec::with_capacity(frames * model.hop_size);
    let start = std::time::Instant::now();
    for frame in 0..frames {
        for (index, value) in input_hop.iter_mut().enumerate() {
            *value = samples
                .get(frame * model.hop_size + index)
                .copied()
                .unwrap_or(0.0);
        }
        model.process(input_hop.view(), output_hop.view_mut())?;
        assert!(output_hop.iter().all(|s| s.is_finite()));
        output.extend(output_hop.iter().copied());
    }
    let process_seconds = start.elapsed().as_secs_f64();
    let audio_seconds = samples.len() as f64 / model.sr as f64;
    let output = &output[delay..delay + samples.len()];
    assert!(output.iter().any(|s| s.abs() > 1e-4));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)?;
    let mut writer = hound::WavWriter::new(
        file,
        hound::WavSpec {
            channels: 1,
            sample_rate: model.sr as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;
    for &sample in output {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    let result = hound::WavReader::open(output_path)?;
    assert_eq!(result.duration() as usize, samples.len());
    println!("file smoke: audio={audio_seconds:.3}s samples={} rate={} process={process_seconds:.3}s RTF={:.3} (debug build, single input, excludes load/resampling/ASR); finite nonzero output and exact duration at 48 kHz", samples.len(), model.sr, process_seconds / audio_seconds);
    Ok(())
}
