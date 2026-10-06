//! Prova nativa isolata. Un processo per backend/modello/modalità, nessun fallback implicito.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::Path;
use std::time::Instant;

use serde_json::json;
use transcribe_cpp::{
    Backend, Diarize, Model, ModelOptions, Nemotron3DiarOptions, Nemotron3DiarPreset, RunExtension,
    RunOptions, StreamExtension, StreamOptions, Transcript,
};

fn main() {
    if let Err(error) = probe() {
        eprintln!(
            "{}",
            json!({"status": "failed", "error": error.to_string()})
        );
        std::process::exit(1);
    }
}

fn probe() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err("uso: memotape-runtime-probe cpu|vulkan MODEL.gguf AUDIO.wav asr-offline|asr-stream|diar-offline|diar-stream auto|it".into());
    }
    let backend = match args[0].as_str() {
        "cpu" => Backend::Cpu,
        "vulkan" => Backend::Vulkan,
        _ => return Err("backend deve essere cpu o vulkan".into()),
    };
    let (diarization, streaming) = match args[3].as_str() {
        "asr-offline" => (false, false),
        "asr-stream" => (false, true),
        "diar-offline" => (true, false),
        "diar-stream" => (true, true),
        _ => return Err("modalità sconosciuta".into()),
    };
    let mut reader = hound::WavReader::open(&args[2])?;
    let spec = reader.spec();
    if spec.channels != 1 || spec.sample_rate != 16_000 || spec.bits_per_sample != 16 {
        return Err("la fixture deve essere WAV PCM16 mono a 16 kHz".into());
    }
    let audio: Vec<f32> = reader
        .samples::<i16>()
        .map(|sample| sample.map(|s| f32::from(s) / 32768.0))
        .collect::<Result<_, _>>()?;
    transcribe_cpp::init_backends_default()?;
    if transcribe_cpp::version() != transcribe_cpp::compiled_version()
        || transcribe_cpp::version_commit() != "e6672a8"
    {
        return Err("la DLL caricata non corrisponde al runtime fissato e6672a8".into());
    }
    println!(
        "{}",
        json!({"stage": "load", "binding": transcribe_cpp::compiled_version(),
            "runtime": transcribe_cpp::version(), "commit": transcribe_cpp::version_commit(),
            "abi": transcribe_cpp::header_hash(), "backend_requested": args[0],
            "model": args[1], "mode": args[3]})
    );
    let started = Instant::now();
    let model = Model::load_with(
        Path::new(&args[1]),
        &ModelOptions {
            backend,
            ..Default::default()
        },
    )?;
    let load_ms = started.elapsed().as_millis();
    let actual_backend = model.backend();
    // Il runtime restituisce il nome ggml del dispositivo: "CPU" oppure "Vulkan0".
    let backend_matches = match backend {
        Backend::Cpu => actual_backend == "CPU",
        Backend::Vulkan => actual_backend
            .strip_prefix("Vulkan")
            .is_some_and(|index| index.parse::<u32>().is_ok()),
        _ => false,
    };
    if !backend_matches {
        return Err(format!(
            "backend effettivo {actual_backend:?} diverso da quello richiesto {:?}",
            args[0]
        )
        .into());
    }
    let caps = model.capabilities();
    let language = match args[4].as_str() {
        "auto" => None,
        code => Some(
            caps.languages
                .iter()
                .find(|locale| {
                    locale.eq_ignore_ascii_case(code)
                        || locale
                            .split('-')
                            .next()
                            .is_some_and(|l| l.eq_ignore_ascii_case(code))
                })
                .ok_or("Lingua del parlato assente nelle capacità del modello")?
                .clone(),
        ),
    };
    println!(
        "{}",
        json!({"stage": "inference", "device": model.device()?.name,
            "arch": model.arch(), "max_timestamp_kind": format!("{:?}", caps.max_timestamp_kind),
            "supports_streaming": caps.supports_streaming, "language_hint": language,
            "languages": caps.languages, "load_ms": load_ms})
    );
    let mut session = model.session()?;
    let run = RunOptions {
        language,
        diarize: if diarization {
            Diarize::On
        } else {
            Diarize::Default
        },
        family: (!streaming && model.arch() == "nemotron3_diar").then_some({
            RunExtension::Nemotron3Diar(Nemotron3DiarOptions {
                preset: Some(Nemotron3DiarPreset::VeryHighLatency),
            })
        }),
        ..Default::default()
    };
    let started = Instant::now();
    let mut partial_changes = 0;
    let mut speaker_updates = 0;
    let transcript = if streaming {
        let options = StreamOptions {
            family: diarization.then_some({
                StreamExtension::Nemotron3Diar(Nemotron3DiarOptions {
                    preset: Some(Nemotron3DiarPreset::LowLatency),
                })
            }),
            ..Default::default()
        };
        let mut stream = session.stream(&run, &options)?;
        let mut previous = String::new();
        for frame in audio.chunks(480) {
            stream.feed(frame)?;
            let text = stream.text().display();
            if text != previous {
                partial_changes += 1;
                previous = text;
            }
            if diarization && !stream.snapshot().speaker_segments.is_empty() {
                speaker_updates += 1;
            }
        }
        stream.finalize()?;
        stream.snapshot()
    } else {
        session.run(&audio, &run)?
    };
    let inference_ms = started.elapsed().as_millis();
    if !diarization {
        validate_language(
            &args[4],
            run.language.as_deref(),
            &model.arch(),
            transcript.language.as_deref(),
        )?;
    }
    validate(
        &transcript,
        diarization,
        streaming,
        partial_changes,
        speaker_updates,
        (audio.len() / 16) as i64,
    )?;
    println!(
        "{}",
        json!({"status": "passed", "backend": args[0], "mode": args[3],
            "audio_ms": audio.len() / 16, "load_ms": load_ms, "inference_ms": inference_ms,
            "text": transcript.text, "detected_language": transcript.language,
            "timestamp_kind": format!("{:?}", transcript.timestamp_kind),
            "tokens": transcript.tokens.len(), "words": transcript.words.len(),
            "segments": transcript.segments.iter().map(|s| json!({"start_ms": s.t0_ms,
                "end_ms": s.t1_ms, "text": s.text})).collect::<Vec<_>>(),
            "token_samples": transcript.tokens.iter().take(12).map(|t| json!({"text": t.text,
                "start_ms": t.t0_ms, "end_ms": t.t1_ms})).collect::<Vec<_>>(),
            "speakers": transcript.speaker_segments.iter().map(|s| s.speaker_id).collect::<BTreeSet<_>>(),
            "speaker_segments": transcript.speaker_segments.iter().map(|s| json!({"speaker": s.speaker_id,
                "start_ms": s.t0_ms, "end_ms": s.t1_ms})).collect::<Vec<_>>(),
            "partial_changes_before_finalize": partial_changes, "speaker_updates_before_finalize": speaker_updates})
    );
    Ok(())
}

fn validate(
    result: &Transcript,
    diarization: bool,
    streaming: bool,
    partial_changes: usize,
    speaker_updates: usize,
    audio_ms: i64,
) -> Result<(), Box<dyn Error>> {
    if diarization {
        if result.speaker_segments.is_empty()
            || result.speaker_segments.iter().any(|s| {
                !(1..=8).contains(&s.speaker_id)
                        || s.t0_ms < 0
                        || s.t1_ms <= s.t0_ms
                        // Sortformer può arrotondare alla griglia di 80 ms.
                        || s.t1_ms > audio_ms + 80
            })
        {
            return Err("Diarizzazione senza segmenti validi".into());
        }
        validate_two_speakers(result)?;
        if streaming && speaker_updates == 0 {
            return Err("nessun Parlante disponibile prima di finalize".into());
        }
    } else {
        let words: Vec<_> = result
            .text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(str::to_lowercase)
            .collect();
        let expected: Vec<_> = "buongiorno a tutti oggi parliamo di trascrizione il computer trasforma la voce in testo"
            .split_whitespace().collect();
        if words != expected {
            return Err(format!(
                "Trascrizione inattesa sulla fixture italiana: {}",
                result.text
            )
            .into());
        }
        if streaming && partial_changes == 0 {
            return Err("nessun Parziale disponibile prima di finalize".into());
        }
    }
    Ok(())
}

fn validate_language(
    requested: &str,
    hint: Option<&str>,
    arch: &str,
    detected: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let italian = |language: &str| {
        language
            .split('-')
            .next()
            .is_some_and(|code| code.eq_ignore_ascii_case("it"))
    };
    let hint_matches = match requested {
        "auto" => hint.is_none(),
        "it" => hint.is_some_and(italian),
        _ => false,
    };
    if !hint_matches || detected.is_some_and(|language| !italian(language)) {
        return Err("Lingua del parlato diversa da quella richiesta/attesa nella fixture".into());
    }
    // Whisper espone il rilevamento automatico; Nemotron/Parakeet possono dare None.
    if arch == "whisper" && requested == "auto" && detected.is_none() {
        return Err("Whisper non ha rilevato la lingua italiana in modalità automatica".into());
    }
    Ok(())
}

fn validate_two_speakers(result: &Transcript) -> Result<(), Box<dyn Error>> {
    let speakers: BTreeSet<_> = result
        .speaker_segments
        .iter()
        .map(|s| s.speaker_id)
        .collect();
    if speakers.len() != 2 {
        return Err("la fixture deve restituire esattamente due Parlanti".into());
    }
    let mut segments: Vec<_> = result.speaker_segments.iter().collect();
    segments.sort_by_key(|s| s.t0_ms);
    let mut covered_ms = 0;
    let mut end_ms = 0;
    for segment in segments {
        // La fixture ha voci alternate, senza sovrapposizioni; tolleriamo 200 ms ai bordi.
        if end_ms - segment.t0_ms > 200 {
            return Err("sovrapposizione inattesa nella fixture a voci alternate".into());
        }
        covered_ms += (segment.t1_ms - segment.t0_ms.max(end_ms)).max(0);
        end_ms = end_ms.max(segment.t1_ms);
    }
    if covered_ms < 14_000 {
        return Err("copertura del parlato insufficiente nella fixture a due voci".into());
    }
    // Finestre centrali dei quattro turni noti: criteri grossolani, non allineamento esatto.
    let windows = [(2000, 4000), (8500, 10500), (14500, 16500), (20500, 22500)];
    let mut turns = Vec::new();
    for (start, end) in windows {
        let coverage: Vec<_> = speakers
            .iter()
            .map(|speaker| {
                let ms: i64 = result
                    .speaker_segments
                    .iter()
                    .filter(|s| s.speaker_id == *speaker)
                    .map(|s| (s.t1_ms.min(end) - s.t0_ms.max(start)).max(0))
                    .sum();
                (*speaker, ms)
            })
            .collect();
        let dominant = coverage.iter().max_by_key(|(_, ms)| *ms).unwrap();
        if dominant.1 < 1500
            || coverage
                .iter()
                .any(|(speaker, ms)| *speaker != dominant.0 && *ms > 250)
        {
            return Err(
                "Parlante o copertura inattesi in uno dei quattro turni della fixture".into(),
            );
        }
        turns.push(dominant.0);
    }
    if turns[0] == turns[1] || turns[0] != turns[2] || turns[1] != turns[3] {
        return Err("le due voci della fixture non si alternano A/B/A/B".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use transcribe_cpp::SpeakerSegment;

    #[test]
    fn due_parole_non_bastano_a_conservare_la_trascrizione() {
        let result = Transcript {
            text: "trascrizione testo".into(),
            ..Default::default()
        };
        assert!(validate(&result, false, false, 0, 0, 8960).is_err());
    }

    #[test]
    fn un_segmento_minimo_non_basta_per_la_fixture_a_due_voci() {
        let result = Transcript {
            speaker_segments: vec![SpeakerSegment {
                t0_ms: 2000,
                t1_ms: 2010,
                speaker_id: 1,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(validate(&result, true, false, 0, 0, 25_675).is_err());
    }

    fn two_speakers() -> Transcript {
        Transcript {
            speaker_segments: [
                (0, 4700, 1),
                (6200, 12000, 2),
                (13500, 18000, 1),
                (19400, 24300, 2),
            ]
            .into_iter()
            .map(|(t0_ms, t1_ms, speaker_id)| SpeakerSegment {
                t0_ms,
                t1_ms,
                speaker_id,
                ..Default::default()
            })
            .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn accetta_due_voci_alternate_e_rifiuta_tempi_o_turni_errati() {
        let result = two_speakers();
        assert!(validate(&result, true, false, 0, 0, 25_675).is_ok());
        assert!(validate(&result, true, true, 0, 0, 25_675).is_err());
        assert!(validate(&result, true, true, 0, 1, 25_675).is_ok());
        let mut outside = result.clone();
        outside.speaker_segments[3].t1_ms = 30_000;
        assert!(validate(&outside, true, false, 0, 0, 25_675).is_err());
        let mut overlap = result.clone();
        overlap.speaker_segments[1].t0_ms = 2000;
        assert!(validate(&overlap, true, false, 0, 0, 25_675).is_err());
        let mut wrong_turn = result.clone();
        wrong_turn.speaker_segments[2].speaker_id = 2;
        assert!(validate(&wrong_turn, true, false, 0, 0, 25_675).is_err());
        let mut sparse = result;
        for segment in &mut sparse.speaker_segments {
            segment.t1_ms = segment.t0_ms + 300;
        }
        assert!(validate(&sparse, true, false, 0, 0, 25_675).is_err());
    }

    #[test]
    fn richiede_testo_intero_e_parziali_prima_di_finalize() {
        let result = Transcript {
            text: "Buongiorno a tutti, oggi parliamo di trascrizione. Il computer trasforma la voce in testo.".into(),
            ..Default::default()
        };
        assert!(validate(&result, false, false, 0, 0, 8960).is_ok());
        assert!(validate(&result, false, true, 0, 0, 8960).is_err());
        assert!(validate(&result, false, true, 6, 0, 8960).is_ok());
    }

    #[test]
    fn rifiuta_hint_o_rilevamento_di_lingua_errati() {
        assert!(validate_language("it", Some("it-IT"), "parakeet", None).is_ok());
        assert!(validate_language("it", Some("en"), "parakeet", None).is_err());
        assert!(validate_language("it", None, "parakeet", None).is_err());
        assert!(validate_language("auto", Some("it"), "whisper", Some("it")).is_err());
        assert!(validate_language("auto", None, "whisper", Some("it")).is_ok());
        assert!(validate_language("auto", None, "whisper", Some("en")).is_err());
        assert!(validate_language("auto", None, "whisper", None).is_err());
        assert!(validate_language("auto", None, "parakeet", None).is_ok());
    }
}
