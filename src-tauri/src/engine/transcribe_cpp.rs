//! `TranscriptionEngine` su transcribe-cpp: `Session::stream` per i modelli in streaming
//! (Nemotron) quando servono i Parziali (dal vivo), con un Parziale a ogni cambio del testo, e
//! `Session::run` sulla Frase intera per gli altri (Whisper, Parakeet) e per i file.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

use transcribe_cpp::{
    CancelToken, Diarize, Feature, Model, Nemotron3DiarOptions, Nemotron3DiarPreset, RunExtension,
    RunOptions, Session, StreamExtension, StreamOptions, TimestampKind, WhisperRunOptions,
};

use super::asr::AsrResult;
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
    timestamps: TimestampKind,
    /// Il Vocabolario della Trascrizione in corso (`set_termini`).
    termini: Vec<String>,
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
            timestamps: capabilities.max_timestamp_kind,
            termini: Vec::new(),
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

    /// I Termini del Vocabolario della prossima Trascrizione, com'erano al suo avvio: il prompt di
    /// Whisper e la correzione delle Frasi concluse.
    pub fn set_termini(&mut self, termini: &[String]) {
        termini.clone_into(&mut self.termini);
    }
}

impl TranscriptionEngine for TranscribeCpp {
    fn transcribe(
        &mut self,
        frames: &mut dyn Iterator<Item = Vec<f32>>,
        language: Option<&str>,
        on_partial: Option<&mut dyn FnMut(&AsrResult)>,
    ) -> Result<AsrResult, EngineError> {
        let run = RunOptions {
            timestamps: self.timestamps,
            language: language.and_then(|l| super::resolve_language(l, &self.languages)),
            family: self
                .fallback
                .then(|| RunExtension::Whisper(confident_only(&self.termini))),
            ..RunOptions::default()
        };
        let termini = &self.termini;
        let keep = |transcript| concluded(transcript, language, termini);
        // Senza Parziali da mostrare `run` sulla Frase intera è più veloce dello stream anche per
        // Nemotron (93 s contro 40 su 10 minuti, docs/research/trascrizione-file-veloce.md).
        let Some(on_partial) = on_partial.filter(|_| self.streaming) else {
            let pcm: Vec<f32> = frames.flatten().collect();
            let session = &mut self.session;
            let transcript = catch_native(|| session.run(&pcm, &run))?;
            return Ok(keep(transcript));
        };
        // Uno stream per Frase. Senza estensione vale l'attenzione a destra predefinita del modello,
        // la prima del menu e la più accurata (`parakeet.h` di transcribe-cpp 0.2.4): per Nemotron
        // R=13, che la sua documentazione dà identico a `run`. Il lease del modello si libera a
        // `finalize`, o al drop dello stream se si esce prima.
        let session = &mut self.session;
        let mut stream = catch_native(|| session.stream(&run, &StreamOptions::default()))?;
        let mut shown = AsrResult::default();
        for frame in frames {
            let update = catch_native(|| stream.feed(&frame))?;
            if update.result_changed {
                let mut snapshot = stream.snapshot();
                snapshot.text = stream.text().display();
                let partial = timed_result(snapshot);
                if partial != shown {
                    on_partial(&partial);
                    shown = partial;
                }
            }
        }
        catch_native(|| stream.finalize())?;
        Ok(keep(stream.snapshot()))
    }
}

/// Il risultato di una Frase conclusa, mai di un Parziale. Una Frase in una scrittura che la lingua
/// scelta non usa è inventata: si scarta. Poi i tratti simili a un Termine diventano il Termine.
fn concluded(
    transcript: transcribe_cpp::Transcript,
    language: Option<&str>,
    termini: &[String],
) -> AsrResult {
    let text = transcript.text.trim();
    if language.is_some_and(|l| super::foreign_script(text, l)) {
        log::info!("Frase scartata, scrittura estranea alla Lingua del parlato: {text}");
        return AsrResult::default();
    }
    super::vocabolario::correct(timed_result(transcript), termini)
}

/// Le parole del runtime includono i token della parola: non si taglia dentro un token/parola.
/// Whisper mantiene i segmenti; lo stream Nemotron espone soltanto token, raggruppati sui
/// confini testuali realmente decodificati. Un disallineamento conserva il testo senza tempi.
fn timed_result(transcript: transcribe_cpp::Transcript) -> AsrResult {
    let text = transcript.text.trim().to_string();
    let mut rows = Vec::new();
    match transcript.timestamp_kind {
        TimestampKind::Word | TimestampKind::Token if !transcript.words.is_empty() => {
            rows.extend(
                transcript
                    .words
                    .into_iter()
                    .map(|w| (w.t0_ms, w.t1_ms, w.text)),
            );
        }
        TimestampKind::Token if !transcript.tokens.is_empty() => {
            for token in transcript.tokens {
                if token.text.is_empty() {
                    continue;
                }
                if rows.is_empty() || token.text.starts_with(char::is_whitespace) {
                    rows.push((token.t0_ms, token.t1_ms, token.text));
                } else if let Some((_, end, text)) = rows.last_mut() {
                    *end = token.t1_ms;
                    text.push_str(&token.text);
                }
            }
        }
        TimestampKind::Segment => {
            rows.extend(
                transcript
                    .segments
                    .into_iter()
                    .map(|s| (s.t0_ms, s.t1_ms, s.text)),
            );
        }
        _ => {}
    }
    AsrResult::timed(text, rows)
}

/// Whisper senza invenzioni: con la confidenza media sotto `logprob_thold` (−1, il predefinito) la
/// Frase si scarta subito, invece di ritentare a temperature più alte. Se nessun tentativo supera le
/// soglie transcribe-cpp tiene l'ultimo, campionato a temperatura 1: su audio incomprensibile erano
/// parole a caso in più lingue. La regola di salto è `no_speech_prob > no_speech_thold` e
/// confidenza sotto soglia, quindi con `no_speech_thold` 0 basta la confidenza. Su una
/// Registrazione di 7 minuti (76 Frasi) ha scartato le 7 inventate e lasciato identiche le altre.
///
/// Il prompt iniziale sono i Termini del Vocabolario uniti da `", "`; oltre circa 223 token
/// transcribe-cpp ne toglie l'inizio. Un `<|…|>` farebbe fallire la Frase con `INVALID_ARG`:
/// `set_settings` lo rifiuta, ma un `settings.json` scritto a mano può averlo, quindi si salta.
fn confident_only(termini: &[String]) -> WhisperRunOptions {
    let prompt: Vec<&str> = termini
        .iter()
        .map(String::as_str)
        .filter(|t| !t.contains("<|") && !t.contains("|>"))
        .collect();
    WhisperRunOptions {
        no_speech_thold: Some(0.0),
        initial_prompt: (!prompt.is_empty()).then(|| prompt.join(", ")),
        ..WhisperRunOptions::default()
    }
}

/// Diarizzazione offline: Sortformer (4 Parlanti) o Nemotron 3 (8), con sessione per Ingresso.
pub struct OfflineDiarizer {
    session: Session,
    nemotron3: bool,
}

impl OfflineDiarizer {
    /// Nemotron analizza il file a blocchi con il preset finale. Il PCM non viene accumulato:
    /// il runtime conserva le cache del preset e i turni, non l'intero audio della Registrazione.
    pub fn diarize_saved(
        &mut self,
        decoder: crate::audio_toolkit::decode::Decoder,
        cancel: &CancelToken,
    ) -> Result<Vec<Turn>, AppError> {
        use crate::engine::pipeline::{Feed, FileFrames};
        let frames = FileFrames::new(decoder, None);
        if !self.nemotron3 {
            // Sortformer conserva il suo contratto offline, che richiede tutto il PCM.
            let mut pcm = Vec::new();
            for feed in frames {
                if cancel.is_cancelled() {
                    return Err(AppError::Cancelled);
                }
                if let Feed::Frame(frame) = feed? {
                    pcm.extend(frame);
                }
            }
            return Ok(self.diarize(&pcm, cancel)?);
        }
        self.session.set_cancel_token(cancel);
        let run = RunOptions {
            diarize: Diarize::On,
            ..Default::default()
        };
        let options = StreamOptions {
            family: Some(StreamExtension::Nemotron3Diar(Nemotron3DiarOptions {
                preset: Some(Nemotron3DiarPreset::VeryHighLatency),
            })),
            ..Default::default()
        };
        let mut stream = catch_native(|| self.session.stream(&run, &options))?;
        for feed in frames {
            if cancel.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            if let Feed::Frame(frame) = feed? {
                catch_native(|| stream.feed(&frame))?;
            }
        }
        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }
        catch_native(|| stream.finalize())?;
        Ok(speaker_turns(&stream.snapshot()))
    }
    pub fn load_sortformer(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::ModelMissing(path.display().to_string()));
        }
        init_backends()?;
        let session = catch_native(|| Model::load(path)?.session())?;
        Ok(Self {
            session,
            nemotron3: false,
        })
    }

    /// Stesso contratto offline, con l'artefatto fissato e il preset accurato della prova.
    pub fn load_nemotron3(path: &Path) -> Result<Self, AppError> {
        super::local_diarizer::validate(path)?;
        init_backends()?;
        let incompatible = || AppError::LocalDiarizerIncompatible(path.display().to_string());
        let model = catch_native(|| Model::load(path)).map_err(|_| incompatible())?;
        if model.arch() != "nemotron3_diar" {
            return Err(incompatible());
        }
        let session = catch_native(|| model.session()).map_err(|_| incompatible())?;
        Ok(Self {
            session,
            nemotron3: true,
        })
    }

    /// I turni di chi parla nell'audio `pcm` (mono a 16 kHz), in ordine di inizio. `cancel` lo
    /// interrompe tra un blocco e l'altro con `EngineError::Cancelled`.
    pub fn diarize(&mut self, pcm: &[f32], cancel: &CancelToken) -> Result<Vec<Turn>, EngineError> {
        self.session.set_cancel_token(cancel);
        let run = RunOptions {
            diarize: Diarize::On,
            family: self
                .nemotron3
                .then_some(RunExtension::Nemotron3Diar(Nemotron3DiarOptions {
                    preset: Some(Nemotron3DiarPreset::VeryHighLatency),
                })),
            ..RunOptions::default()
        };
        let session = &mut self.session;
        let transcript = catch_native(|| session.run(pcm, &run))?;
        // Sortformer dà i segmenti per parlante, non per tempo. Un id negativo o un tratto senza
        // tempi non dice nulla su chi parla quando: si scarta.
        Ok(speaker_turns(&transcript))
    }
}

fn speaker_turns(transcript: &transcribe_cpp::Transcript) -> Vec<Turn> {
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
    turns
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::TempoTesto;

    fn termini(t: &[&str]) -> Vec<String> {
        t.iter().map(|&t| t.to_owned()).collect()
    }

    #[test]
    fn whisper_riceve_i_termini_nel_prompt_iniziale() {
        let options = confident_only(&termini(&["ChargeBee", "Niccolò", "Kubernetes"]));
        assert_eq!(
            options.initial_prompt.as_deref(),
            Some("ChargeBee, Niccolò, Kubernetes")
        );
        assert_eq!(options.no_speech_thold, Some(0.0));
    }

    fn words(rows: &[(i64, i64, &str)]) -> transcribe_cpp::Transcript {
        transcribe_cpp::Transcript {
            text: rows.iter().map(|r| r.2).collect(),
            timestamp_kind: TimestampKind::Word,
            words: rows
                .iter()
                .map(|&(t0_ms, t1_ms, text)| transcribe_cpp::Word {
                    t0_ms,
                    t1_ms,
                    text: text.to_owned(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn tempo(inizio_byte: usize, fine_byte: usize, inizio_ms: u32, fine_ms: u32) -> TempoTesto {
        TempoTesto {
            inizio_byte,
            fine_byte,
            inizio_ms,
            fine_ms,
        }
    }

    #[test]
    fn la_frase_conclusa_prende_i_termini_con_i_tempi_uniti() {
        let transcript = words(&[(0, 300, " Uso"), (300, 700, " Charge"), (700, 900, " B.")]);
        let result = concluded(transcript, Some("it"), &termini(&["ChargeBee"]));
        assert_eq!(result.text, "Uso ChargeBee.");
        assert_eq!(
            result.tempi,
            vec![tempo(0, 4, 0, 300), tempo(4, 14, 300, 900)]
        );
    }

    #[test]
    fn la_frase_in_scrittura_estranea_si_scarta_anche_con_i_termini() {
        let transcript = words(&[(0, 500, " Привет"), (500, 900, " мир")]);
        let result = concluded(transcript, Some("it"), &termini(&["Привет"]));
        assert_eq!(result, AsrResult::default());
    }

    #[test]
    fn senza_termini_la_frase_conclusa_resta_com_e() {
        let transcript = words(&[(0, 300, " Uso"), (300, 700, " Charge"), (700, 900, " B.")]);
        let result = concluded(transcript, None, &[]);
        assert_eq!(result.text, "Uso Charge B.");
        assert_eq!(result.tempi.len(), 3);
    }

    #[test]
    fn senza_termini_whisper_non_ha_prompt() {
        assert_eq!(confident_only(&[]).initial_prompt, None);
    }

    #[test]
    fn un_termine_con_token_speciali_non_entra_nel_prompt() {
        let options = confident_only(&termini(&["<|it|>", "ChargeBee", "a|>b"]));
        assert_eq!(options.initial_prompt.as_deref(), Some("ChargeBee"));
        assert_eq!(confident_only(&termini(&["<|en|>"])).initial_prompt, None);
    }
}
