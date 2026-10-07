use super::*;
use crate::audio_toolkit::ogg_opus::tests::temp_dir;
use crate::managers::settings::ProfiloAudio;

#[test]
fn i_profili_persistenti_valgono_all_avvio_al_volo_e_non_dopo_stop() {
    let folder = temp_dir("controlli-pulizia");
    let store = SettingsStore::load(folder.join("settings.json"));
    let recorder = Recorder::default();
    let controls = Arc::new(Controls::default());
    let mut settings = store.get();
    settings.audio_microfono = ProfiloAudio {
        pulizia: true,
        sensibilita: Sensibilita::Sensibile,
    };
    store.set(settings).unwrap();
    recorder.set_audio(&store); // Ancora nessuna Registrazione.
    {
        let mut current = recorder.current();
        controls.set_audio(&store.get());
        *current = Some(controls.clone());
    }
    assert!(controls.pulizia(Kind::Microphone));
    assert!(!controls.pulizia(Kind::System));
    assert_eq!(
        controls.sensibilita(Kind::Microphone),
        Sensibilita::Sensibile
    );
    assert_eq!(controls.sensibilita(Kind::System), Sensibilita::Bilanciato);
    recorder.set_paused(true);
    let mut settings = store.get();
    settings.audio_sistema.pulizia = true;
    settings.audio_sistema.sensibilita = Sensibilita::Selettivo;
    settings.audio_file_misto.pulizia = true;
    store.set(settings).unwrap();
    recorder.set_audio(&store);
    assert!(controls.pulizia(Kind::System));
    assert_eq!(controls.sensibilita(Kind::System), Sensibilita::Selettivo);
    recorder.stop();
    let mut settings = store.get();
    settings.audio_microfono.pulizia = false;
    settings.audio_microfono.sensibilita = Sensibilita::Spento;
    store.set(settings).unwrap();
    recorder.set_audio(&store);
    assert!(controls.pulizia(Kind::Microphone));
    assert_eq!(
        controls.sensibilita(Kind::Microphone),
        Sensibilita::Sensibile
    );
    *recorder.current() = None;
    let next = Controls::default();
    next.set_audio(&store.get());
    assert!(!next.pulizia(Kind::Microphone));
    assert_eq!(next.sensibilita(Kind::Microphone), Sensibilita::Spento);
    assert_eq!(next.sensibilita(Kind::System), Sensibilita::Selettivo);
    assert!(next.pulizia(Kind::System));
    assert!(
        SettingsStore::load(folder.join("settings.json"))
            .get()
            .audio_file_misto
            .pulizia
    );
}

#[test]
#[ignore = "DFN3 reale: verifica il runtime riutilizzato alle transizioni del ticket 03"]
fn dfn3_registrazione_coda_pause_e_stato_indipendente() {
    use crate::audio_toolkit::processing::{Boundary, PcmStream};
    for (rate, channels) in [(48_000, 2), (44_100, 2), (24_000, 1)] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(deepfilter::MODEL_FILE);
        let format = Format { rate, channels };
        let enabled = Arc::new(AtomicBool::new(true));
        let control = enabled.clone();
        let log = CleaningLog::default();
        let mut processor = ConfiguredCleaning::new(
            path.clone(),
            move || control.load(Ordering::Relaxed),
            Ingresso::Sistema,
            log.clone(),
        )
        .recovering(|error| panic!("{error}"));
        processor.prepare(format).unwrap();
        let mut stream = PcmStream::new(format, Box::new(processor)).unwrap();
        let mut signal = Vec::new();
        for i in 0..19_213 {
            signal.push(0.03 * (i as f32 * 0.071).sin());
            if channels == 2 {
                signal.push(0.0);
            }
        }
        // Impulso nell'ultimo campione utile: il silenzio dopo il confine non può sostituirlo.
        let last_impulse = 19_212 * channels;
        signal[last_impulse] = 0.2;
        let mut segments = Vec::new();
        for boundary in [Boundary::Pause, Boundary::Configuration, Boundary::Finish] {
            let mut output = Vec::new();
            for block in signal.chunks(137 * channels) {
                output.extend(stream.push(block).unwrap().samples);
            }
            output.extend(stream.boundary(boundary).unwrap().samples);
            assert_eq!(output.len(), signal.len());
            if channels == 2 {
                assert!(output.chunks_exact(2).all(|frame| frame[1] == 0.0));
            }
            assert_eq!(output[last_impulse], 0.2); // Il fade conserva l'ultimo frame utile.
            segments.push(output);
        }
        assert_eq!(segments[0], segments[1]);
        assert_eq!(segments[1], segments[2]); // Clone del piano vergine, nessuna voce precedente.
        let intervals = log.intervals();
        assert_eq!(intervals.len(), 3);
        assert_eq!(
            (intervals[2].inizio_frame, intervals[2].fine_frame),
            (38_426, 57_639)
        );
        println!(
            "DFN3 ticket03: {rate}Hz {channels}ch, tre segmenti da 19.213 frame, coda/impulso finale e reset bit-identici"
        );
    }
}

#[cfg(windows)]
#[test]
#[ignore = "WASAPI release, due Ingressi + DFN3 + Nemotron: riproduce la fixture e registra 28 s"]
fn dfn3_due_ingressi_nativi_release() {
    due_ingressi_nativi_release(false);
}

#[cfg(windows)]
#[test]
#[ignore = "WASAPI release ticket 06: due Ingressi, DFN3, protezione streaming, Tape"]
fn protezione_due_ingressi_nativi_release() {
    due_ingressi_nativi_release(true);
}

#[cfg(windows)]
fn due_ingressi_nativi_release(protected: bool) {
    use crate::audio_toolkit::{decode::Decoder, vad::Silero};
    use crate::engine::{
        pipeline::{self, PipelineEvent},
        transcribe_cpp::TranscribeCpp,
    };
    use crate::managers::{models, settings::SpeechLanguage};
    use crate::transcript::Phrase;
    use std::os::windows::process::CommandExt;
    if cfg!(debug_assertions) {
        panic!("questa misura richiede --release");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let models_dir =
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("it.memotape.desktop/models");
    let log = CleaningLog::default();
    let pairs = live::channels(48_000, 2, 2).unwrap();
    let timelines: Vec<_> = pairs.iter().map(|(feed, _)| feed.protection()).collect();
    let folder = temp_dir(if protected {
        "protezione-nativo-ticket06"
    } else {
        "dfn3-nativo-ticket03"
    });
    let store = SettingsStore::load(folder.join("settings.json"));
    let recorder = Recorder::default();
    let controls = Arc::new(Controls::default());
    let mut settings = store.get();
    for profile in [&mut settings.audio_microfono, &mut settings.audio_sistema] {
        profile.pulizia = true;
        profile.sensibilita = if protected {
            Sensibilita::Bilanciato
        } else {
            Sensibilita::Spento
        };
    }
    store.set(settings).unwrap();
    controls.set_audio(&store.get());
    *recorder.current() = Some(controls.clone());
    let mut applied_cleaning = [true; 2];
    let mut processors = Vec::new();
    for (ingresso, kind) in [
        (Ingresso::Microfono, Kind::Microphone),
        (Ingresso::Sistema, Kind::System),
    ] {
        let control = controls.clone();
        let mut processor = ConfiguredCleaning::new(
            root.join(deepfilter::MODEL_FILE),
            move || control.pulizia(kind),
            ingresso,
            log.clone(),
        )
        .recovering(|error| panic!("pulizia nativa guasta: {error}"));
        processor
            .prepare(Format {
                rate: 48_000,
                channels: 2,
            })
            .unwrap();
        processors.push(Box::new(processor) as Box<dyn AudioProcessor>);
    }
    // Caricamenti e riscaldamento fuori dalla cattura; le istanze lavorano sugli Ingressi separati.
    let mut engines = Vec::new();
    for _ in 0..2 {
        engines.push(TranscribeCpp::load(&models::default_model().path(&models_dir)).unwrap());
    }
    let backlogs: Vec<_> = pairs.iter().map(|(_, frames)| frames.backlog()).collect();
    let mut feeds = Vec::new();
    let mut workers = Vec::new();
    for ((feed, mut frames), (ingresso, mut engine)) in pairs.into_iter().zip(
        [Ingresso::Microfono, Ingresso::Sistema]
            .into_iter()
            .zip(engines),
    ) {
        feeds.push(feed);
        let mut detector = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
        workers.push(std::thread::spawn(move || {
            let mut phrases = Vec::new();
            let protection = frames.protection();
            let mut partials = 0;
            pipeline::transcribe_protected(
                &mut frames,
                &mut engine,
                &mut detector,
                Some("it"),
                &CancelToken::new(),
                &mut |event| {
                    if let PipelineEvent::Partial { text, .. } = &event
                        && !text.is_empty()
                    {
                        partials += 1;
                    }
                    if let PipelineEvent::Phrase {
                        inizio_ms,
                        fine_ms,
                        text,
                        tempi,
                        ..
                    } = event
                    {
                        phrases.push(Phrase {
                            inizio_ms,
                            fine_ms,
                            text,
                            tempi,
                            ingresso,
                            parlante: None,
                            parlante_provvisorio: false,
                            parlante_non_determinato: false,
                        });
                    }
                },
                &protection,
            )
            .unwrap();
            println!(
                "STREAM {ingresso:?} partials={partials} phrases={:?}",
                phrases.iter().map(|p| &p.text).collect::<Vec<_>>()
            );
            if protected && ingresso == Ingresso::Sistema {
                assert!(partials > 0);
            }
            phrases
        }));
    }
    let (sender, blocks) = capture::channel(2);
    let captures = [
        Capture::open(Kind::Microphone, None, 0, sender.clone()).unwrap(),
        Capture::open(Kind::System, None, 1, sender).unwrap(),
    ];
    let origin = captures[0].now();
    let mut mixer = Mixer::with_processors(
        origin,
        &captures
            .iter()
            .map(|capture| (capture.rate, capture.channels))
            .collect::<Vec<_>>(),
        (48_000, 2),
        processors,
    )
    .unwrap()
    .with_tracks();
    if protected {
        for (i, timeline) in timelines.iter().enumerate() {
            mixer.protect(
                i,
                timeline,
                controls.sensibilita([Kind::Microphone, Kind::System][i]),
            );
        }
    }
    let mut outputs = [Ingresso::Mix, Ingresso::Microfono, Ingresso::Sistema]
        .into_iter()
        .map(|ingresso| {
            let path = folder.join(tape::audio_entry(ingresso));
            Output {
                ingresso,
                writer: OggOpusWriter::new(File::create(&path).unwrap(), 48_000, 2, 64).unwrap(),
                path,
                feed: if ingresso == Ingresso::Mix {
                    None
                } else {
                    Some(feeds.remove(0))
                },
            }
        })
        .collect::<Vec<_>>();
    for capture in &captures {
        println!(
            "DEVICE {} {}Hz {}ch",
            capture.name, capture.rate, capture.channels
        );
    }
    let mut player = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", "Start-Sleep -Seconds 2; $taskPlayer = New-Object System.Media.SoundPlayer $env:MEMOTAPE_SMOKE_WAV; $taskPlayer.PlaySync(); Start-Sleep -Seconds 3; $taskPlayer.PlaySync()"])
        .env("MEMOTAPE_SMOKE_WAV", root.join("tests/fixtures/parlato-it.wav"))
        .creation_flags(0x08000000).spawn().unwrap();
    let start = Instant::now();
    let mut out = Vec::new();
    let mut processing_time = Duration::ZERO;
    let mut last_second = 0;
    let mut max_pool = [0; 2];
    let mut changes = [false; 2];
    while start.elapsed().as_secs_f64() < 28.0 {
        let block = blocks.recv_timeout(Duration::from_millis(10));
        let tick = Instant::now();
        let seconds = start.elapsed().as_secs_f64();
        // Store temporaneo: stesso contratto Settings → Recorder, nessuna Impostazione dell'utente.
        if seconds >= 7.0 && !changes[0] {
            let mut settings = store.get();
            settings.audio_microfono.pulizia = false;
            if protected {
                settings.audio_microfono.sensibilita = Sensibilita::Selettivo;
                settings.audio_sistema.sensibilita = Sensibilita::Sensibile;
            }
            store.set(settings).unwrap();
            recorder.set_audio(&store);
            changes[0] = true;
        }
        let paused = (12.0..13.0).contains(&seconds);
        recorder.set_paused(paused);
        if !paused {
            for (i, kind) in [Kind::Microphone, Kind::System].into_iter().enumerate() {
                let cleaning = controls.pulizia(kind);
                if cleaning != applied_cleaning[i] {
                    mixer.configuration(i, &mut out).unwrap();
                    applied_cleaning[i] = cleaning;
                }
                if protected {
                    mixer.protect(i, &timelines[i], controls.sensibilita(kind));
                }
            }
        }
        mixer.advance(captures[0].now(), paused, &mut out).unwrap();
        if paused && !changes[1] {
            let mut settings = store.get();
            settings.audio_microfono.pulizia = true;
            if protected {
                settings.audio_microfono.sensibilita = Sensibilita::Sensibile;
                settings.audio_sistema.sensibilita = Sensibilita::Bilanciato;
            }
            store.set(settings).unwrap();
            recorder.set_audio(&store);
            changes[1] = true;
        }
        if let Ok(block) = block {
            mixer
                .push(block.input, block.capture_ns, &block.samples, &mut out)
                .unwrap();
            captures[block.input].recycle(block.samples);
        }
        processing_time += tick.elapsed();
        write_ready(&mut outputs, &mut out, mixer.tracks(), paused).unwrap();
        let second = start.elapsed().as_secs();
        if second > last_second {
            last_second = second;
            let pool = captures
                .each_ref()
                .map(|capture| capture.metrics.in_flight.load(Ordering::Relaxed));
            for i in 0..2 {
                max_pool[i] = max_pool[i].max(pool[i]);
            }
            println!(
                "LOAD t={second}s capture_pool={pool:?} asr_backlog_ms={:?} timeline_ms={}",
                backlogs.iter().map(|backlog| backlog()).collect::<Vec<_>>(),
                mixer.elapsed_ms()
            );
        }
        assert!(
            captures
                .iter()
                .all(|capture| device_error(capture).is_none())
        );
    }
    let stop = captures[0].now();
    recorder.stop();
    let frozen = controls.sensibilita(Kind::Microphone);
    let mut settings = store.get();
    settings.audio_microfono.sensibilita = Sensibilita::Spento;
    store.set(settings).unwrap();
    recorder.set_audio(&store);
    assert_eq!(controls.sensibilita(Kind::Microphone), frozen);
    for block in blocks.try_iter().filter(|block| block.capture_ns < stop) {
        mixer
            .push(block.input, block.capture_ns, &block.samples, &mut out)
            .unwrap();
        captures[block.input].recycle(block.samples);
    }
    let lost = captures
        .each_ref()
        .map(|capture| capture.metrics.lost_frames.load(Ordering::Relaxed));
    drop(captures);
    let tick = Instant::now();
    mixer.finish(stop, &mut out).unwrap();
    processing_time += tick.elapsed();
    let durata_ms = mixer.elapsed_ms();
    let mut forma_onda = Vec::new();
    let mut oggs = Vec::new();
    for (output, samples) in outputs
        .into_iter()
        .zip(std::iter::once(&mut out).chain(mixer.tracks()))
    {
        let Output {
            ingresso,
            path,
            mut writer,
            feed,
        } = output;
        writer.write(samples).unwrap();
        if ingresso == Ingresso::Mix {
            forma_onda = writer.forma_onda();
        }
        writer.finish().unwrap();
        if let Some(mut feed) = feed {
            feed.push(samples, false);
            feed.finish();
        }
        oggs.push((ingresso, path));
    }
    let mut phrases = Vec::new();
    for worker in workers {
        phrases.extend(worker.join().unwrap());
    }
    assert!(player.wait().unwrap().success());
    let recorded = Recorded {
        oggs,
        start: Local::now(),
        durata_ms,
        error: None,
        forma_onda,
        pulizia_audio: log.intervals(),
    };
    let mut document = tape::Document::new(
        tape::creato(recorded.start),
        durata_ms,
        tape::Modalita::IngressiSeparati,
        Some(models::default_model().id.clone()),
        SpeechLanguage::from("it"),
        true,
        &phrases,
    );
    document.pulizia_audio = recorded.pulizia_audio.clone();
    let path = save_tape(&folder, &folder, "Nativo", &recorded, &document).unwrap();
    assert_eq!(tape::read(&path).unwrap(), document);
    assert_eq!(tape::forma_onda(&path), Some(recorded.forma_onda));
    for ingresso in [Ingresso::Microfono, Ingresso::Sistema, Ingresso::Mix] {
        let mut decoder = Decoder::open_ingresso(&path, ingresso).unwrap();
        let mut frames = 0;
        while let Some(block) = decoder.next_block().unwrap() {
            assert_eq!(block.channels, 2);
            assert_eq!(block.rate, 48_000);
            frames += block.samples.len() / 2;
        }
        assert!((frames as f64 / 48.0 - f64::from(durata_ms)).abs() < 1.0);
    }
    println!(
        "RESULT protected={protected} durata_ms={durata_ms} processing_s={} time_per_audio_s={} max_capture_pool={max_pool:?} lost_frames={lost:?} asr_backlog_ms={:?} phrases={} tape={}",
        processing_time.as_secs_f64(),
        processing_time.as_secs_f64() / (f64::from(durata_ms) / 1000.0),
        backlogs.iter().map(|backlog| backlog()).collect::<Vec<_>>(),
        phrases.len(),
        path.display()
    );
    let task_script = format!(
        "$taskProcess = Get-Process -Id {}; Write-Output ($taskProcess.WorkingSet64.ToString() + ',' + $taskProcess.PrivateMemorySize64.ToString() + ',' + $taskProcess.PeakWorkingSet64.ToString())",
        std::process::id()
    );
    let memory = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &task_script])
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    println!(
        "MEMORY working,private,peak_working bytes {}",
        String::from_utf8_lossy(&memory.stdout)
    );
    assert_eq!(
        lost,
        [0, 0],
        "la misura non supera il criterio se perde buffer"
    );
    assert!(
        phrases
            .iter()
            .any(|phrase| phrase.ingresso == Ingresso::Sistema)
    );
}
