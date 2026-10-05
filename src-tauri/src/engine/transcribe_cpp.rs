//! `TranscriptionEngine` su transcribe-cpp: `Session::stream` per i modelli in streaming
//! (Nemotron) quando servono i Parziali (dal vivo), con un Parziale a ogni cambio del testo, e
//! `Session::run` sulla Frase intera per gli altri (Whisper, Parakeet) e per i file.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

use transcribe_cpp::{
    CancelToken, Diarize, Feature, Model, RunExtension, RunOptions, Session, StreamOptions,
    WhisperRunOptions,
};

use super::diarize::Turn;
use super::{EngineError, TranscriptionEngine};
use crate::error::AppError;

/// Un modello caricato, riusabile tra una Trascrizione e l'altra.
pub struct TranscribeCpp {
    session: Session,
    /// Le lingue di `capabilities().languages` (codici o locale, es. `it-IT`).
    languages: Vec<String>,
    /// `capabilities().supports_streaming`: fra i modelli del catalogo, solo Nemotron.
    streaming: bool,
    /// `Feature::TemperatureFallback`: fra i modelli del catalogo, solo Whisper.
    fallback: bool,
}

impl TranscribeCpp {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        init_backends()?;
        let (session, capabilities, fallback) = catch_native(|| {
            let model = Model::load(path)?;
            if !model.supports(Feature::Cancellation) {
                log::info!(
                    "il modello non supporta la cancellazione: Annulla aspetta la fine della Frase"
                );
            }
            Ok((
                model.session()?,
                model.capabilities(),
                model.supports(Feature::TemperatureFallback),
            ))
        })?;
        let mut engine = Self {
            session,
            languages: capabilities.languages,
            streaming: capabilities.supports_streaming,
            fallback,
        };
        engine.warm_up();
        Ok(engine)
    }

    /// La prima chiamata a un modello appena caricato paga il riscaldamento del backend (con Whisper
    /// su Vulkan 16 s contro 0,8 sulla fixture): si paga qui, su un secondo di silenzio, nel
    /// caricamento in background, e non alla prima Frase.
    fn warm_up(&mut self) {
        let started = Instant::now();
        let silence = vec![0.0; crate::audio_toolkit::resample::TARGET_RATE];
        let session = &mut self.session;
        match catch_native(|| session.run(&silence, &RunOptions::default())) {
            Ok(_) => log::info!("modello riscaldato in {:?}", started.elapsed()),
            Err(e) => log::warn!("riscaldamento del modello: {e}"),
        }
    }

    /// Le lingue che il modello accetta come indicazione, lette dal modello.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Installa il token di Annulla della prossima Trascrizione: interrompe anche la Frase in
    /// corso se il modello supporta la cancellazione, altrimenti la pipeline si ferma alla fine
    /// della Frase.
    pub fn set_cancel_token(&mut self, cancel: &CancelToken) {
        self.session.set_cancel_token(cancel);
    }
}

impl TranscriptionEngine for TranscribeCpp {
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
        language: Option<&str>,
        on_partial: Option<&mut dyn FnMut(&str)>,
    ) -> Result<String, EngineError> {
        let run = RunOptions {
            language: language.and_then(|l| super::resolve_language(l, &self.languages)),
            family: self
                .fallback
                .then(|| RunExtension::Whisper(confident_only())),
            ..RunOptions::default()
        };
        // Una Frase in una scrittura che la lingua scelta non usa è inventata: si scarta.
        let keep = |text: String| {
            let text = text.trim();
            if language.is_some_and(|l| super::foreign_script(text, l)) {
                log::info!("Frase scartata, scrittura estranea alla Lingua del parlato: {text}");
                return String::new();
            }
            text.to_string()
        };
        // Senza Parziali da mostrare `run` sulla Frase intera è più veloce dello stream anche per
        // Nemotron (93 s contro 40 su 10 minuti, docs/research/trascrizione-file-veloce.md).
        let Some(on_partial) = on_partial.filter(|_| self.streaming) else {
            let pcm: Vec<f32> = frames.flatten().collect();
            let session = &mut self.session;
            let transcript = catch_native(|| session.run(&pcm, &run))?;
            return Ok(keep(transcript.text));
        };
        // Uno stream per Frase. Senza estensione vale l'attenzione a destra predefinita del modello,
        // la prima del menu e la più accurata (`parakeet.h` di transcribe-cpp 0.2.4): per Nemotron
        // R=13, che la sua documentazione dà identico a `run`. Il lease del modello si libera a
        // `finalize`, o al drop dello stream se si esce prima.
        let session = &mut self.session;
        let mut stream = catch_native(|| session.stream(&run, &StreamOptions::default()))?;
        let mut shown = String::new();
        for frame in frames {
            let update = catch_native(|| stream.feed(&frame))?;
            if update.committed_changed || update.tentative_changed {
                let partial = stream.text().display().trim().to_string();
                if partial != shown {
                    on_partial(&partial);
                    shown = partial;
                }
            }
        }
        catch_native(|| stream.finalize())?;
        Ok(keep(stream.text().full))
    }
}

/// Whisper senza invenzioni: con la confidenza media sotto `logprob_thold` (−1, il predefinito) la
/// Frase si scarta subito, invece di ritentare a temperature più alte. Se nessun tentativo supera le
/// soglie transcribe-cpp tiene l'ultimo, campionato a temperatura 1: su audio incomprensibile erano
/// parole a caso in più lingue. La regola di salto è `no_speech_prob > no_speech_thold` e
/// confidenza sotto soglia, quindi con `no_speech_thold` 0 basta la confidenza. Su una
/// Registrazione di 7 minuti (76 Frasi) ha scartato le 7 inventate e lasciato identiche le altre.
fn confident_only() -> WhisperRunOptions {
    WhisperRunOptions {
        no_speech_thold: Some(0.0),
        ..WhisperRunOptions::default()
    }
}

/// Il modello di diarizzazione (Sortformer, al massimo 4 parlanti): solo `run`, sull'audio intero.
pub struct Sortformer {
    session: Session,
}

impl Sortformer {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        init_backends()?;
        let session = catch_native(|| Model::load(path)?.session())?;
        Ok(Self { session })
    }

    /// I turni di chi parla nell'audio `pcm` (mono a 16 kHz), in ordine di inizio. `cancel` lo
    /// interrompe tra un blocco e l'altro con `EngineError::Cancelled`.
    pub fn diarize(&mut self, pcm: &[f32], cancel: &CancelToken) -> Result<Vec<Turn>, EngineError> {
        self.session.set_cancel_token(cancel);
        let run = RunOptions {
            diarize: Diarize::On,
            ..RunOptions::default()
        };
        let session = &mut self.session;
        let transcript = catch_native(|| session.run(pcm, &run))?;
        // Sortformer dà i segmenti per parlante, non per tempo. Un id negativo o un tratto senza
        // tempi non dice nulla su chi parla quando: si scarta.
        let mut turns: Vec<Turn> = transcript
            .speaker_segments
            .iter()
            .filter_map(|s| {
                Some(Turn {
                    inizio_ms: u32::try_from(s.t0_ms).ok()?,
                    fine_ms: u32::try_from(s.t1_ms).ok()?,
                    parlante: u32::try_from(s.speaker_id).ok()?,
                })
            })
            .filter(|t| t.fine_ms > t.inizio_ms)
            .collect();
        turns.sort_by_key(|t| (t.inizio_ms, t.parlante));
        Ok(turns)
    }
}

/// Con `dynamic-backends` i backend (un modulo CPU per livello di ISA, Vulkan) sono DLL accanto a
/// `transcribe.dll`, da registrare una volta per processo prima del primo `Model::load`. Non si
/// può riprovare nello stesso processo, quindi anche l'errore resta memorizzato.
fn init_backends() -> Result<(), EngineError> {
    static BACKENDS: OnceLock<Result<(), EngineError>> = OnceLock::new();
    BACKENDS
        .get_or_init(|| {
            catch_native(transcribe_cpp::init_backends_default)?;
            let devices: Vec<_> = transcribe_cpp::devices()
                .into_iter()
                .map(|d| format!("{} ({})", d.name, d.description))
                .collect();
            log::info!("backend di transcribe-cpp: {}", devices.join(", "));
            Ok(())
        })
        .clone()
}

/// Chiamata nativa protetta: un panic non deve abbattere il thread della pipeline senza errore.
fn catch_native<T>(call: impl FnOnce() -> transcribe_cpp::Result<T>) -> Result<T, EngineError> {
    catch_unwind(AssertUnwindSafe(call))
        .map_err(|_| EngineError::Internal("panic in transcribe-cpp".into()))?
        .map_err(|e| match e {
            transcribe_cpp::Error::Aborted { .. } => EngineError::Cancelled,
            transcribe_cpp::Error::Busy(_) => EngineError::Busy,
            e => EngineError::Internal(format!("transcribe-cpp: {e}")),
        })
}
