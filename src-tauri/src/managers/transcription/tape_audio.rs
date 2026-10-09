//! Prepara l'audio di Trascrivi su Tape senza modificare la Sorgente. Il commit sostituisce
//! insieme audio, Forma d'onda e documento, soltanto dopo ASR e analisi dei Parlanti.
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use transcribe_cpp::CancelToken;

use super::{CleaningLog, Decoder, Ingresso, OggCopy, TEMP_FOLDER, temp_folder};
use crate::audio_toolkit::decode::Block;
use crate::audio_toolkit::mixer::downmix;
use crate::audio_toolkit::processing::{
    AudioProcessor, Boundary, Format, PcmStream, ProcessedBlock,
};
use crate::error::AppError;
use crate::tape::{self, Document};

struct WorkDir(PathBuf);

impl Drop for WorkDir {
    fn drop(&mut self) {
        if let Ok(files) = std::fs::read_dir(&self.0) {
            for file in files.flatten() {
                let _ = std::fs::remove_file(file.path());
            }
        }
        let _ = std::fs::remove_dir(&self.0);
        if let Some(parent) = self.0.parent().filter(|p| p.ends_with(TEMP_FOLDER)) {
            let _ = std::fs::remove_dir(parent);
        }
    }
}

struct Track {
    ingresso: Ingresso,
    pcm: PathBuf,
    format: Format,
    frames: u64,
}

pub(super) struct TapeAudio {
    work: Option<WorkDir>,
    original: PathBuf,
    replacements: Vec<(Ingresso, PathBuf)>,
    forma_onda: Option<Vec<f32>>,
    intervals: Vec<crate::audio_toolkit::cleaning::TrattoPulizia>,
}

impl TapeAudio {
    pub(super) fn prepare(
        source: &Path,
        old: &Document,
        bitrate: u32,
        log: &CleaningLog,
        cancel: &CancelToken,
        mut processor: impl FnMut(Ingresso) -> Box<dyn AudioProcessor>,
    ) -> Result<Self, AppError> {
        check(cancel)?;
        let folder = temp_folder(source.parent().unwrap_or_else(|| Path::new(".")))?;
        let path = folder.join(format!("ritrascrizione-{}", tape::temporary_id()));
        std::fs::create_dir(&path).map_err(|e| io(&path, e))?;
        let work = WorkDir(path);
        let jobs: Vec<_> = super::ingressi_of(source)?
            .into_iter()
            .map(|ingresso| (ingresso, processor(ingresso)))
            .collect();
        // Gli Ingressi sono indipendenti: la pulizia (DFN3, un core) di ciascuno gira in parallelo,
        // e si ricodifica in Opus solo la traccia che è stata davvero pulita.
        let folder = work.0.as_path();
        let prepared = std::thread::scope(|scope| {
            let handles: Vec<_> = jobs
                .into_iter()
                .map(|(ingresso, processor)| {
                    scope.spawn(move || {
                        let track = prepare_track(source, ingresso, folder, processor, cancel)?;
                        let cleaned = log.intervals().iter().any(|i| i.ingresso == ingresso);
                        let encoded = cleaned
                            .then(|| encode_track(&track, folder, bitrate, cancel))
                            .transpose()?;
                        Ok::<_, AppError>((track, encoded))
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("preparazione di un Ingresso"))
                .collect::<Result<Vec<_>, _>>()
        })?;
        let mut tracks = Vec::new();
        let mut replacements = Vec::new();
        let mut forma_onda = None;
        for (track, encoded) in prepared {
            if let Some((ogg, wave)) = encoded {
                replacements.push((track.ingresso, ogg));
                forma_onda = Some(wave);
            }
            tracks.push(track);
        }
        if !replacements.is_empty() && tracks.len() == 2 {
            let mut decoder = Decoder::open(source)?;
            let first = decoder
                .next_block()?
                .ok_or_else(|| AppError::UnreadableFile("mix vuoto".into()))?;
            let format = Format {
                rate: first.rate,
                channels: first.channels,
            };
            let (mix, wave) = rebuild_mix(&tracks, &work.0, format, bitrate, cancel)?;
            replacements.push((Ingresso::Mix, mix));
            forma_onda = Some(wave);
        }
        let mut all_intervals = old.pulizia_audio.clone();
        // In parallelo gli Ingressi scrivono nel registro alternandosi: si torna all'ordine delle tracce.
        let mut fresh = log.intervals();
        fresh.sort_by_key(|i| tracks.iter().position(|t| t.ingresso == i.ingresso));
        all_intervals.extend(fresh);
        Ok(Self {
            work: Some(work),
            original: source.to_path_buf(),
            replacements,
            forma_onda,
            intervals: all_intervals,
        })
    }

    /// Senza pulizia l'audio del Tape resta com'è: niente decodifica preliminare né PCM su disco,
    /// l'ASR legge direttamente il Tape e il commit riscrive solo il documento.
    pub(super) fn unchanged(source: &Path, old: &Document) -> Self {
        Self {
            work: None,
            original: source.to_path_buf(),
            replacements: Vec::new(),
            forma_onda: None,
            intervals: old.pulizia_audio.clone(),
        }
    }

    /// Se l'audio è stato preparato qui: altrimenti la pipeline deve catturare la protezione.
    pub(super) fn prepared(&self) -> bool {
        self.work.is_some()
    }

    pub(super) fn source(&self, ingresso: Ingresso) -> &Path {
        self.replacements
            .iter()
            .find(|(i, _)| *i == ingresso)
            .map_or(self.original.as_path(), |(_, path)| path.as_path())
    }

    pub(super) fn commit(
        self,
        path: &Path,
        document: &Document,
        cancel: &CancelToken,
    ) -> Result<(), AppError> {
        let mut document = document.clone();
        document.pulizia_audio.clone_from(&self.intervals);
        let audio: Vec<_> = self
            .replacements
            .iter()
            .map(|(i, p)| (*i, p.as_path()))
            .collect();
        tape::rewrite_audio_cancellable(path, &document, &audio, self.forma_onda.as_deref(), cancel)
    }
}

fn check(cancel: &CancelToken) -> Result<(), AppError> {
    if cancel.is_cancelled() {
        Err(AppError::Cancelled)
    } else {
        Ok(())
    }
}

fn io(path: &Path, error: std::io::Error) -> AppError {
    AppError::UnwritableFolder(format!("{}: {error}", path.display()))
}

fn prepare_track(
    source: &Path,
    ingresso: Ingresso,
    folder: &Path,
    processor: Box<dyn AudioProcessor>,
    cancel: &CancelToken,
) -> Result<Track, AppError> {
    let mut decoder = Decoder::open_ingresso(source, ingresso)?;
    let first = decoder
        .next_block()?
        .ok_or_else(|| AppError::UnreadableFile("traccia vuota".into()))?;
    let format = Format {
        rate: first.rate,
        channels: first.channels,
    };
    let pcm = folder
        .join(tape::audio_entry(ingresso))
        .with_extension("pcm");
    let mut raw = BufWriter::new(File::create_new(&pcm).map_err(|e| io(&pcm, e))?);
    let mut stream = PcmStream::new(format, processor)?;
    let mut frames = 0;
    let mut deliver = |block: ProcessedBlock| -> Result<(), AppError> {
        frames += (block.samples.len() / format.channels) as u64;
        for sample in block.samples {
            raw.write_all(&sample.to_le_bytes())
                .map_err(|e| io(&pcm, e))?;
        }
        Ok(())
    };
    let mut block = Some(first);
    while let Some(current) = block {
        check(cancel)?;
        if current.rate != format.rate || current.channels != format.channels {
            return Err(AppError::UnreadableFile(
                "la traccia cambia formato PCM".into(),
            ));
        }
        deliver(stream.push(&current.samples)?)?;
        block = decoder.next_block()?;
    }
    check(cancel)?;
    deliver(stream.boundary(Boundary::Finish)?)?;
    raw.flush().map_err(|e| io(&pcm, e))?;
    Ok(Track {
        ingresso,
        pcm,
        format,
        frames,
    })
}

/// L'Ogg di una traccia pulita, dal suo PCM.
fn encode_track(
    track: &Track,
    folder: &Path,
    bitrate: u32,
    cancel: &CancelToken,
) -> Result<(PathBuf, Vec<f32>), AppError> {
    let ogg = folder.join(tape::audio_entry(track.ingresso));
    let mut copy = OggCopy::new(
        File::create_new(&ogg).map_err(|e| io(&ogg, e))?,
        track.format.rate,
        track.format.channels,
        bitrate,
    )?;
    let wave = copy.forma_onda();
    let mut input = BufReader::new(File::open(&track.pcm).map_err(|e| io(&track.pcm, e))?);
    let mut remaining = track.frames;
    while remaining > 0 {
        check(cancel)?;
        let frames = remaining.min(4096) as usize;
        let mut bytes = vec![0; frames * track.format.channels * 4];
        input
            .read_exact(&mut bytes)
            .map_err(|e| io(&track.pcm, e))?;
        copy.push(&Block {
            rate: track.format.rate,
            channels: track.format.channels,
            samples: bytes
                .chunks_exact(4)
                .map(|s| f32::from_le_bytes(s.try_into().unwrap()))
                .collect(),
        })?;
        remaining -= frames as u64;
    }
    copy.finish()?;
    Ok((ogg, wave.get().cloned().unwrap_or_default()))
}

fn rebuild_mix(
    tracks: &[Track],
    folder: &Path,
    format: Format,
    bitrate: u32,
    cancel: &CancelToken,
) -> Result<(PathBuf, Vec<f32>), AppError> {
    if tracks
        .iter()
        .any(|t| t.format.rate != format.rate || t.frames != tracks[0].frames)
    {
        return Err(AppError::UnreadableFile(
            "tracce e mix del Tape non sono sincronizzati".into(),
        ));
    }
    let path = folder.join(tape::audio_entry(Ingresso::Mix));
    let mut copy = OggCopy::new(
        File::create_new(&path).map_err(|e| io(&path, e))?,
        format.rate,
        format.channels,
        bitrate,
    )?;
    let wave = copy.forma_onda();
    let mut inputs: Vec<_> = tracks
        .iter()
        .map(|t| File::open(&t.pcm).map_err(|e| io(&t.pcm, e)))
        .collect::<Result<_, _>>()?;
    let mut remaining = tracks[0].frames;
    while remaining > 0 {
        check(cancel)?;
        let frames = remaining.min(4096) as usize;
        let mut mixed = vec![0.0_f32; frames * format.channels];
        for (track, input) in tracks.iter().zip(&mut inputs) {
            let mut bytes = vec![0; frames * track.format.channels * 4];
            input
                .read_exact(&mut bytes)
                .map_err(|e| io(&track.pcm, e))?;
            let samples: Vec<_> = bytes
                .chunks_exact(4)
                .map(|s| f32::from_le_bytes(s.try_into().unwrap()))
                .collect();
            let mut converted = Vec::with_capacity(mixed.len());
            for frame in samples.chunks_exact(track.format.channels) {
                downmix(frame, format.channels, &mut converted);
            }
            for (sum, value) in mixed.iter_mut().zip(converted) {
                *sum += value;
            }
        }
        for sample in &mut mixed {
            *sample = sample.clamp(-1.0, 1.0);
        }
        copy.push(&Block {
            rate: format.rate,
            channels: format.channels,
            samples: mixed,
        })?;
        remaining -= frames as u64;
    }
    copy.finish()?;
    Ok((path, wave.get().cloned().unwrap_or_default()))
}
