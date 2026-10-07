//! Prepara l'audio di Trascrivi su Tape senza modificare la Sorgente. Il commit sostituisce
//! insieme audio, Forma d'onda e documento, soltanto dopo ASR e analisi dei Parlanti.
use std::fs::File;
use std::io::{Read, Write};
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
    ogg: PathBuf,
    pcm: PathBuf,
    format: Format,
    frames: u64,
}

pub(super) struct TapeAudio {
    _work: WorkDir,
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
        let ingressi = super::ingressi_of(source)?;
        let mut tracks = Vec::new();
        let mut forma_onda = None;
        for ingresso in ingressi {
            let (track, wave) = prepare_track(
                source,
                ingresso,
                &work.0,
                bitrate,
                processor(ingresso),
                cancel,
            )?;
            forma_onda = Some(wave);
            tracks.push(track);
        }
        let intervals = log.intervals();
        let mut replacements: Vec<_> = tracks
            .iter()
            .filter(|track| intervals.iter().any(|i| i.ingresso == track.ingresso))
            .map(|track| (track.ingresso, track.ogg.clone()))
            .collect();
        if replacements.is_empty() {
            forma_onda = None;
        } else if tracks.len() == 2 {
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
        all_intervals.extend(intervals);
        Ok(Self {
            _work: work,
            original: source.to_path_buf(),
            replacements,
            forma_onda,
            intervals: all_intervals,
        })
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
    bitrate: u32,
    processor: Box<dyn AudioProcessor>,
    cancel: &CancelToken,
) -> Result<(Track, Vec<f32>), AppError> {
    let mut decoder = Decoder::open_ingresso(source, ingresso)?;
    let first = decoder
        .next_block()?
        .ok_or_else(|| AppError::UnreadableFile("traccia vuota".into()))?;
    let format = Format {
        rate: first.rate,
        channels: first.channels,
    };
    let ogg = folder.join(tape::audio_entry(ingresso));
    let pcm = ogg.with_extension("pcm");
    let mut raw = File::create_new(&pcm).map_err(|e| io(&pcm, e))?;
    let mut copy = OggCopy::new(
        File::create_new(&ogg).map_err(|e| io(&ogg, e))?,
        format.rate,
        format.channels,
        bitrate,
    )?;
    let wave = copy.forma_onda();
    let mut stream = PcmStream::new(format, processor)?;
    let mut frames = 0;
    let mut deliver = |block: ProcessedBlock| -> Result<(), AppError> {
        frames += (block.samples.len() / format.channels) as u64;
        let bytes: Vec<_> = block.samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        raw.write_all(&bytes).map_err(|e| io(&pcm, e))?;
        copy.push(&Block {
            rate: format.rate,
            channels: format.channels,
            samples: block.samples,
        })
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
    copy.finish()?;
    Ok((
        Track {
            ingresso,
            ogg,
            pcm,
            format,
            frames,
        },
        wave.get().cloned().unwrap_or_default(),
    ))
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
