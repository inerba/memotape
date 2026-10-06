//! Il server MCP degli Assistenti (ADR-0012): `memotape.exe --mcp`, lanciato da Claude o Codex, parla
//! in stdio e legge la Libreria senza scrivere nulla. Non usa Tauri: le cartelle dell'app vengono da
//! `dirs`, come le risolve Tauri, e le impostazioni si rileggono a ogni chiamata, così l'interruttore
//! di Impostazioni vale subito.

use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Mutex, PoisonError};

use chrono::NaiveDate;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::{ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};

use crate::library::{self, Library, MARK_END, MARK_START};
use crate::managers::settings::{CopiaCome, Settings};
use crate::managers::transcription;
use crate::transcript::Ingresso;
use crate::{error::AppError, tape};

/// L'identifier di `tauri.conf.json`: il nome delle cartelle dell'app.
const IDENTIFIER: &str = "it.memotape.desktop";
/// I Tape di una ricerca, se l'Assistente non ne chiede un altro numero, e al massimo.
const SEARCH_LIMIT: u32 = 10;
const SEARCH_MAX: u32 = 25;
/// I Tape di un elenco, se l'Assistente non ne chiede un altro numero, e al massimo.
const LIST_LIMIT: u32 = 50;
const LIST_MAX: u32 = 100;
/// Le Frasi prima e dopo quella chiesta, se l'Assistente non ne chiede un altro numero, e al massimo.
const AROUND: u32 = 10;
const AROUND_MAX: u32 = 25;
// I massimi tengono ogni risposta sotto i 10 000 token a cui Codex taglia.
/// I byte di una pagina del testo: circa 7 000 token, sotto i 10 000 a cui Codex taglia.
const PAGE_BYTES: usize = 24_000;

const DISABLED: &str = "Memotape does not allow assistants to read its Library. Ask the user to turn on \
\"Allow assistants to read the Library\" in Memotape → Settings → Assistants.";
const NOT_INDEXED: &str =
    "Memotape has not indexed this Library yet: ask the user to open Memotape once.";

/// Le cartelle dell'app: dove leggere impostazioni e indice.
#[derive(Clone)]
pub struct Places {
    /// `settings.json`, in `app_data_dir`.
    pub settings: PathBuf,
    /// La cartella degli indici della Libreria, in `app_local_data_dir`.
    pub index: PathBuf,
}

impl Places {
    /// Quelle che usa l'app: `%APPDATA%` e `%LOCALAPPDATA%` (le cartelle note, come Tauri).
    fn of_app() -> Option<Self> {
        Some(Self {
            settings: dirs::data_dir()?.join(IDENTIFIER).join("settings.json"),
            index: dirs::data_local_dir()?.join(IDENTIFIER).join("libreria"),
        })
    }
}

/// Avvia il server su stdin e stdout e risponde finché l'Assistente non chiude stdin.
pub fn serve() -> ExitCode {
    let Some(places) = Places::of_app() else {
        eprintln!("cartelle dell'app non trovate");
        return ExitCode::FAILURE;
    };
    start_log();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            log::error!("server MCP: {e}");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(async {
        Server::new(places)
            .serve(rmcp::transport::stdio())
            .await?
            .waiting()
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("server MCP: {e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// The words to find. Each word matches as a word prefix, ignoring case and accents; all words
    /// must appear in the same phrase (or in the title, or in a renamed speaker's name).
    query: String,
    /// Search only this Raccolta (collection), by name; "" searches only Tape outside any Raccolta.
    /// Omit to search the whole Library.
    raccolta: Option<String>,
    /// How many Tape to return, most relevant first (default 10, at most 25).
    limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListParams {
    /// List only this Raccolta (collection), by name; "" lists only Tape outside any Raccolta.
    /// Omit to list the whole Library.
    raccolta: Option<String>,
    /// Only Tape recorded on or after this local date, as YYYY-MM-DD.
    from: Option<String>,
    /// Only Tape recorded on or before this local date, as YYYY-MM-DD.
    to: Option<String>,
    /// How many Tape to return, most recent first (default 50, at most 100).
    limit: Option<u32>,
    /// How many of the matching Tape to skip, to read past `limit` (see `total`).
    offset: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AroundParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The Ingresso of the phrase: "mix", "microfono" or "sistema", as returned by search.
    ingresso: String,
    /// The id of the phrase, as returned by search (`phraseId`).
    phrase_id: u32,
    /// How many phrases to read before it, of any Ingresso (default 10, at most 25).
    before: Option<u32>,
    /// How many phrases to read after it, of any Ingresso (default 10, at most 25).
    after: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TranscriptParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// Where to continue reading: the `from` given at the end of the previous page. Omit to start.
    from: Option<u32>,
}

/// Un Tape della Libreria.
#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TapeOut {
    /// Path relative to the Library: pass it to read_around and read_transcript.
    tape: String,
    titolo: String,
    /// The Raccolta (collection); null outside any Raccolta.
    raccolta: Option<String>,
    /// When it was recorded, local time with UTC offset.
    creato: String,
    /// Null if the Tape cannot be read.
    durata_ms: Option<u32>,
}

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchOut {
    tapes: Vec<FoundTape>,
}

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FoundTape {
    #[serde(flatten)]
    tape: TapeOut,
    /// The matching phrases (up to 5), in order of time; empty if only the title matched.
    frasi: Vec<FoundFrase>,
}

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FoundFrase {
    ingresso: String,
    phrase_id: u32,
    inizio_ms: u32,
    /// An excerpt of the phrase with the matching words in **bold**.
    estratto: String,
}

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListOut {
    /// All the Raccolte (collections) of the Library.
    raccolte: Vec<String>,
    /// How many Tape match, before `offset` and `limit`.
    total: u32,
    tapes: Vec<TapeOut>,
}

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AroundOut {
    #[serde(flatten)]
    tape: TapeOut,
    /// In order of time.
    frasi: Vec<FraseOut>,
    /// Whether the Tape has phrases before the first one returned.
    more_before: bool,
    /// Whether the Tape has phrases after the last one returned.
    more_after: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FraseOut {
    ingresso: String,
    phrase_id: u32,
    inizio_ms: u32,
    fine_ms: u32,
    /// The speaker's name, or the numbered label; null if not attributed.
    parlante: Option<String>,
    testo: String,
}

#[derive(Clone)]
pub struct Server {
    places: Places,
}

#[tool_router]
impl Server {
    pub fn new(places: Places) -> Self {
        Self { places }
    }

    #[tool(
        description = "Search the Memotape Library for words in titles, phrases and speaker names. \
Returns the most relevant Tape, each with up to 5 matching phrases (excerpt, Ingresso, phraseId, \
start time). Rephrase and search again with synonyms if nothing is found; read a hit's context with \
read_around.",
        annotations(title = "Search the Library", read_only_hint = true)
    )]
    fn search(&self, Parameters(p): Parameters<SearchParams>) -> Result<Json<SearchOut>, String> {
        log::info!("Assistente: search {:?} in {:?}", p.query, p.raccolta);
        let (_, library) = self.allowed_library()?;
        let limit = bounded(p.limit, SEARCH_LIMIT, SEARCH_MAX);
        let results = library
            .search(&p.query, p.raccolta.as_deref())
            .map_err(|e| e.to_string())?;
        let tapes = results
            .into_iter()
            .take(limit)
            .map(|r| FoundTape {
                tape: tape_out(library.root(), r.tape),
                frasi: r
                    .frasi
                    .into_iter()
                    .map(|hit| FoundFrase {
                        ingresso: hit.ingresso.key().into(),
                        phrase_id: hit.phrase_id,
                        inizio_ms: hit.inizio_ms,
                        estratto: bold(&hit.estratto),
                    })
                    .collect(),
            })
            .collect();
        Ok(Json(SearchOut { tapes }))
    }

    #[tool(
        description = "List the Raccolte (collections) and the Tape of the Memotape Library, most \
recent first, optionally only one Raccolta or a range of dates.",
        annotations(title = "List the Library", read_only_hint = true)
    )]
    fn list_tapes(&self, Parameters(p): Parameters<ListParams>) -> Result<Json<ListOut>, String> {
        log::info!(
            "Assistente: list_tapes {:?} dal {:?} al {:?}",
            p.raccolta,
            p.from,
            p.to
        );
        let (from, to) = (date(p.from.as_deref())?, date(p.to.as_deref())?);
        let (_, library) = self.allowed_library()?;
        let list = library.list().map_err(|e| e.to_string())?;
        let limit = bounded(p.limit, LIST_LIMIT, LIST_MAX);
        let matching: Vec<_> = list
            .tapes
            .into_iter()
            .filter(|b| match p.raccolta.as_deref() {
                None => true,
                Some("") => b.raccolta.is_none(),
                Some(nome) => b.raccolta.as_deref() == Some(nome),
            })
            .filter(|b| in_period(&b.creato, from, to))
            .collect();
        Ok(Json(ListOut {
            raccolte: list.raccolte,
            total: u32::try_from(matching.len()).unwrap_or(u32::MAX),
            tapes: matching
                .into_iter()
                .skip(p.offset.unwrap_or(0) as usize)
                .take(limit)
                .map(|b| tape_out(library.root(), b))
                .collect(),
        }))
    }

    #[tool(
        description = "Read the phrases around a phrase of a Tape, with times, Ingresso and speaker: \
use it to read the context of a search hit. Phrases of all the Ingressi are in order of time, so \
`before` and `after` count phrases of any Ingresso.",
        annotations(title = "Read around a phrase", read_only_hint = true)
    )]
    fn read_around(
        &self,
        Parameters(p): Parameters<AroundParams>,
    ) -> Result<Json<AroundOut>, String> {
        log::info!(
            "Assistente: read_around {} {}:{}",
            p.tape,
            p.ingresso,
            p.phrase_id
        );
        let (settings, library) = self.allowed_library()?;
        let path = resolve(library.root(), &p.tape)?;
        let labels = transcription::labels(&settings);
        let ingresso = Ingresso::from_key(&p.ingresso).ok_or_else(|| {
            format!(
                "Unknown ingresso {:?}: use mix, microfono or sistema.",
                p.ingresso
            )
        })?;
        let document = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        let mut frasi: Vec<FraseOut> = document
            .frasi
            .iter()
            .map(|f| FraseOut {
                ingresso: f.ingresso.key().into(),
                phrase_id: f.id,
                inizio_ms: f.inizio_ms,
                fine_ms: f.fine_ms,
                parlante: labels.assigned_name(
                    &document.parlanti,
                    f.ingresso,
                    f.parlante,
                    f.parlante_non_determinato,
                    f.parlante_provvisorio,
                ),
                testo: f.testo.clone(),
            })
            .collect();
        frasi.sort_by_key(|f| f.inizio_ms);
        let at = frasi
            .iter()
            .position(|f| f.ingresso == ingresso.key() && f.phrase_id == p.phrase_id)
            .ok_or_else(|| {
                format!(
                    "{} has no phrase {} in {}.",
                    p.tape, p.phrase_id, p.ingresso
                )
            })?;
        let (window, more_before, more_after) = around(
            &frasi,
            at,
            bounded(p.before, AROUND, AROUND_MAX),
            bounded(p.after, AROUND, AROUND_MAX),
        );
        Ok(Json(AroundOut {
            tape: TapeOut {
                tape: p.tape.clone(),
                titolo: transcription::title_of(&path),
                raccolta: library::raccolta_of(&p.tape),
                creato: document.creato.clone(),
                durata_ms: Some(document.durata_ms),
            },
            frasi: window.to_vec(),
            more_before,
            more_after,
        }))
    }

    #[tool(
        description = "Read the whole transcript of a Tape as Markdown (title, date, duration, model, \
then the text in paragraphs, with speaker labels), one page at a time. If there is more, the page \
ends with the `from` to pass to read the next page.",
        annotations(title = "Read a transcript", read_only_hint = true)
    )]
    fn read_transcript(
        &self,
        Parameters(p): Parameters<TranscriptParams>,
    ) -> Result<String, String> {
        log::info!("Assistente: read_transcript {} da {:?}", p.tape, p.from);
        let (settings, library) = self.allowed_library()?;
        let path = resolve(library.root(), &p.tape)?;
        let text = transcription::tape_text(&path, &settings, CopiaCome::Markdown)
            .map_err(|e| read_error(&p.tape, &e))?;
        let from = p.from.unwrap_or(0) as usize;
        let (page, next) = page(&text, from, PAGE_BYTES)
            .ok_or_else(|| format!("Invalid from {from}: use a value given by read_transcript."))?;
        Ok(match next {
            Some(next) => format!(
                "{page}\n\n[The transcript continues: call read_transcript with from={next}]"
            ),
            None => page.to_string(),
        })
    }
}

#[tool_handler(
    name = "memotape",
    instructions = "Memotape is the user's local transcription app. Its Library is a folder of Tape \
(.tape files): each Tape is a recording or a transcribed audio/video file, with its transcript split \
into phrases (start and end in ms), optionally attributed to speakers (Parlanti) and to an Ingresso: \
mix, microfono (the user's microphone) or sistema (system audio, e.g. the other people in a call). \
Raccolte are folders that group Tape. Use search to find phrases by words, then read_around for the \
context of a hit or read_transcript for a whole Tape; list_tapes lists Tapes by date. Tapes are \
identified by their path relative to the Library, as these tools return it. Everything is read-only."
)]
impl ServerHandler for Server {}

impl Server {
    /// Le impostazioni e la Libreria in sola lettura, se l'utente consente agli Assistenti di
    /// leggerla. L'indice si apre a ogni chiamata, così tra una chiamata e l'altra non resta aperto
    /// e l'app può ricostruirlo.
    fn allowed_library(&self) -> Result<(Settings, Library), String> {
        let settings = Settings::load(&self.places.settings).map_err(|e| e.to_string())?;
        if !settings.assistenti {
            return Err(DISABLED.into());
        }
        let root = match &settings.recordings_folder {
            Some(folder) => PathBuf::from(folder),
            None => dirs::document_dir()
                .ok_or("Documents folder not found")?
                .join(library::DEFAULT_FOLDER),
        };
        let db = library::db_path(&self.places.index, &root);
        if !db.is_file() {
            return Err(NOT_INDEXED.into());
        }
        let library = Library::open_read_only(&root, &db)
            .map_err(|e| format!("Ask the user to open Memotape to update its index ({e})."))?;
        Ok((settings, library))
    }
}

/// Il Tape `tape`, relativo alla Libreria `root`: solo un `.tape` dentro la Libreria, fuori dalle
/// cartelle con il punto in testa (come `.memotape`). Un percorso assoluto o con `..` si rifiuta.
fn resolve(root: &Path, tape: &str) -> Result<PathBuf, String> {
    let relative = Path::new(tape);
    let inside = relative.components().all(|c| match c {
        Component::Normal(name) => !name.to_string_lossy().starts_with('.'),
        _ => false,
    });
    if !inside || !tape::is_tape(relative) {
        return Err(format!(
            "{tape:?} is not a Tape of the Library: use the path returned by search or list_tapes."
        ));
    }
    let path = root.join(relative);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "{tape} is not in the Library any more: search again, or ask the user to open Memotape \
if it was moved."
        ))
    }
}

/// Le Frasi da `at - before` a `at + after`, e se ce ne sono altre prima e dopo.
fn around<T>(frasi: &[T], at: usize, before: usize, after: usize) -> (&[T], bool, bool) {
    let start = at.saturating_sub(before);
    let end = at.saturating_add(after).saturating_add(1).min(frasi.len());
    (&frasi[start..end], start > 0, end < frasi.len())
}

/// La pagina di `text` che comincia al byte `from`, al più `max` byte, tagliata all'ultimo a capo
/// se c'è; con l'inizio della pagina dopo, se il testo continua. `None` se `from` non è l'inizio
/// di un carattere del testo.
fn page(text: &str, from: usize, max: usize) -> Option<(&str, Option<usize>)> {
    let rest = text.get(from..)?;
    if rest.len() <= max {
        return Some((rest, None));
    }
    let mut end = max;
    while !rest.is_char_boundary(end) {
        end -= 1;
    }
    if let Some(newline) = rest[..end].rfind('\n').filter(|&n| n > 0) {
        end = newline + 1;
    }
    Some((&rest[..end], Some(from + end)))
}

/// Il valore di `from` o `to`, una data `YYYY-MM-DD`.
fn date(value: Option<&str>) -> Result<Option<NaiveDate>, String> {
    value
        .map(|v| {
            NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .map_err(|_| format!("Invalid date {v:?}: use YYYY-MM-DD."))
        })
        .transpose()
}

/// Se il Tape creato a `creato` (`2026-10-04T10:15:00+02:00`, ora locale) cade tra i giorni `from` e
/// `to`, compresi. Un `creato` illeggibile resta fuori da ogni periodo.
fn in_period(creato: &str, from: Option<NaiveDate>, to: Option<NaiveDate>) -> bool {
    if from.is_none() && to.is_none() {
        return true;
    }
    creato
        .get(..10)
        .and_then(|day| NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
        .is_some_and(|day| from.is_none_or(|f| day >= f) && to.is_none_or(|t| day <= t))
}

/// Il numero chiesto dall'Assistente, `default` se non lo dice, al più `max`.
fn bounded(value: Option<u32>, default: u32, max: u32) -> usize {
    value.unwrap_or(default).min(max) as usize
}

/// L'estratto con le parole trovate in grassetto Markdown invece che tra `MARK_START` e `MARK_END`.
fn bold(estratto: &str) -> String {
    estratto.replace([MARK_START, MARK_END], "**")
}

fn tape_out(root: &Path, entry: library::TapeEntry) -> TapeOut {
    TapeOut {
        tape: Path::new(&entry.path)
            .strip_prefix(root)
            .map_or(entry.path.clone(), |p| p.display().to_string()),
        titolo: entry.titolo,
        raccolta: entry.raccolta,
        creato: entry.creato,
        durata_ms: entry.durata_ms,
    }
}

fn read_error(tape: &str, e: &AppError) -> String {
    match e {
        AppError::UnsupportedTape => {
            format!("{tape} comes from a newer Memotape: ask the user to update the app.")
        }
        _ => format!("{tape} cannot be read: {e}"),
    }
}

/// Il log del server: su stderr, che Claude Desktop salva in `mcp-server-memotape.log`, e nel log
/// dell'app. Sotto Claude Desktop (MSIX) anche il file finisce nel livello privato di Claude.
fn start_log() {
    struct ToFile(Option<Mutex<std::fs::File>>);
    impl log::Log for ToFile {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::Level::Info
        }
        fn log(&self, record: &log::Record) {
            if !self.enabled(record.metadata()) {
                return;
            }
            let line = format!(
                "[{}][mcp][{}] {}\n",
                chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                record.level(),
                record.args()
            );
            let _ = std::io::stderr().write_all(line.as_bytes());
            if let Some(file) = &self.0 {
                let _ = file
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .write_all(line.as_bytes());
            }
        }
        fn flush(&self) {}
    }
    let file = dirs::data_local_dir()
        .map(|dir| dir.join(IDENTIFIER).join("logs"))
        .and_then(|dir| {
            std::fs::create_dir_all(&dir).ok()?;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("Memotape.log"))
                .ok()
        });
    if log::set_boxed_logger(Box::new(ToFile(file.map(Mutex::new)))).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::ogg_opus::tests::temp_dir;
    use crate::library::tests::{tape_at, tape_with};

    #[test]
    fn l_identifier_e_quello_di_tauri() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(conf["identifier"], IDENTIFIER);
    }

    #[test]
    fn si_legge_solo_un_tape_dentro_la_libreria() {
        let dir = temp_dir("mcp-percorsi");
        tape_at(&dir.join("Acme").join("Call.tape"), 1000);
        tape_at(&dir.join(".memotape").join("Nascosto.tape"), 1000);
        std::fs::write(dir.join("Note.md"), "").unwrap();
        assert_eq!(
            resolve(&dir, r"Acme\Call.tape"),
            Ok(dir.join("Acme").join("Call.tape"))
        );
        assert_eq!(
            resolve(&dir, "Acme/Call.tape"),
            Ok(dir.join("Acme").join("Call.tape"))
        );
        for rifiutato in [
            "",
            "Note.md",
            r"..\fuori.tape",
            r"Acme\..\..\fuori.tape",
            r".memotape\Nascosto.tape",
            r"C:\Windows\x.tape",
            r"\x.tape",
            "Assente.tape",
        ] {
            assert!(resolve(&dir, rifiutato).is_err(), "{rifiutato}");
        }
        let assoluto = dir.join("Acme").join("Call.tape");
        assert!(resolve(&dir, &assoluto.display().to_string()).is_err());
    }

    #[test]
    fn intorno_a_una_frase_si_ferma_ai_bordi() {
        let frasi: Vec<u32> = (0..10).collect();
        assert_eq!(around(&frasi, 5, 2, 1), (&frasi[3..7], true, true));
        assert_eq!(around(&frasi, 1, 5, 0), (&frasi[0..2], false, true));
        assert_eq!(around(&frasi, 8, 0, 5), (&frasi[8..10], true, false));
        assert_eq!(around(&frasi, 0, 0, 0), (&frasi[0..1], false, true));
    }

    #[test]
    fn le_pagine_si_tagliano_a_capo_e_coprono_tutto_il_testo() {
        let text = "prima riga\nseconda è più lunga\nterza\n";
        assert_eq!(page(text, 0, 100), Some((text, None)));
        let (first, next) = page(text, 0, 25).unwrap();
        assert_eq!((first, next), ("prima riga\n", Some(11)));
        let (second, next) = page(text, 11, 25).unwrap();
        assert_eq!(second, "seconda è più lunga\n");
        let (third, next) = page(text, next.unwrap(), 25).unwrap();
        assert_eq!((third, next), ("terza\n", None));
        // Senza a capo si taglia al limite, senza spezzare un carattere.
        assert_eq!(page("àààà", 0, 3), Some(("à", Some(2))));
        // `from` fuori dal testo o a metà di un carattere.
        assert_eq!(page(text, 1000, 25), None);
        assert_eq!(page("à", 1, 25), None);
    }

    #[test]
    fn il_periodo_comprende_i_due_giorni() {
        let creato = "2026-10-04T23:30:00+02:00";
        let day = |d: &str| date(Some(d)).unwrap();
        assert!(in_period(creato, None, None));
        assert!(in_period(creato, day("2026-10-04"), day("2026-10-04")));
        assert!(in_period(creato, day("2026-10-01"), None));
        assert!(!in_period(creato, day("2026-10-05"), None));
        assert!(!in_period(creato, None, day("2026-10-03")));
        assert!(!in_period("illeggibile", day("2026-10-01"), None));
        assert_eq!(date(None), Ok(None));
        for wrong in ["2026-10", "4/10/2026", "2026-13-01", ""] {
            assert!(date(Some(wrong)).is_err(), "{wrong}");
        }
    }

    /// Una Libreria di prova già indicizzata, con le impostazioni che consentono o no di leggerla.
    fn server(name: &str, assistenti: bool) -> (PathBuf, Server) {
        let dir = temp_dir(&format!("mcp-{name}"));
        let root = dir.join("Memotape");
        let places = Places {
            settings: dir.join("settings.json"),
            index: dir.join("indice"),
        };
        Settings {
            recordings_folder: Some(root.display().to_string()),
            assistenti,
            ..Settings::default()
        }
        .save(&places.settings)
        .unwrap();
        tape_with(
            &root.join("Ferrara Quarzi").join("Call di lunedì.tape"),
            &[
                ("Buongiorno a tutti.", Some(1)),
                ("Parliamo del preventivo per l'impianto.", Some(2)),
                ("Il preventivo arriva entro venerdì.", Some(1)),
                ("Perfetto, grazie.", Some(2)),
            ],
            &[("mix:2", "Giulia Ferrara")],
        );
        tape_at(&root.join("Sciolto.tape"), 1000);
        Library::open(&root, &library::db_path(&places.index, &root))
            .unwrap()
            .sync()
            .unwrap();
        (root, Server::new(places))
    }

    #[test]
    fn senza_il_consenso_non_si_legge_nulla() {
        let (_, server) = server("spento", false);
        let search = server.search(Parameters(SearchParams {
            query: "preventivo".into(),
            raccolta: None,
            limit: None,
        }));
        assert_eq!(search.err().as_deref(), Some(DISABLED));
    }

    #[test]
    fn senza_indice_si_chiede_di_aprire_memotape() {
        let (_, server) = server("senza-indice", true);
        std::fs::remove_dir_all(&server.places.index).unwrap();
        let list = server.list_tapes(Parameters(ListParams {
            raccolta: None,
            from: None,
            to: None,
            limit: None,
            offset: None,
        }));
        assert_eq!(list.err().as_deref(), Some(NOT_INDEXED));
        assert!(!server.places.index.exists());
    }

    #[test]
    fn si_cerca_e_si_legge_intorno_a_una_frase_trovata() {
        let (_, server) = server("cerca", true);
        let Json(found) = server
            .search(Parameters(SearchParams {
                query: "preventivo".into(),
                raccolta: None,
                limit: None,
            }))
            .unwrap();
        let [tape] = found.tapes.as_slice() else {
            panic!("{found:?}");
        };
        assert_eq!(tape.tape.tape, r"Ferrara Quarzi\Call di lunedì.tape");
        assert_eq!(tape.tape.raccolta.as_deref(), Some("Ferrara Quarzi"));
        assert_eq!(
            tape.frasi
                .iter()
                .map(|f| (f.phrase_id, f.estratto.as_str()))
                .collect::<Vec<_>>(),
            [
                (1, "Parliamo del **preventivo** per l'impianto."),
                (2, "Il **preventivo** arriva entro venerdì.")
            ]
        );
        let Json(around) = server
            .read_around(Parameters(AroundParams {
                tape: tape.tape.tape.clone(),
                ingresso: "mix".into(),
                phrase_id: 1,
                before: Some(1),
                after: Some(1),
            }))
            .unwrap();
        assert_eq!(around.tape.titolo, "Call di lunedì");
        assert_eq!(
            around
                .frasi
                .iter()
                .map(|f| (f.phrase_id, f.parlante.as_deref()))
                .collect::<Vec<_>>(),
            [
                (0, Some("Parlante 1")),
                (1, Some("Giulia Ferrara")),
                (2, Some("Parlante 1"))
            ]
        );
        assert_eq!((around.more_before, around.more_after), (false, true));
        assert!(
            server
                .read_around(Parameters(AroundParams {
                    tape: tape.tape.tape.clone(),
                    ingresso: "sistema".into(),
                    phrase_id: 1,
                    before: None,
                    after: None,
                }))
                .is_err()
        );
    }

    #[test]
    fn una_frase_ambigua_non_si_legge_ne_si_cerca_col_nome_del_microfono() {
        let (root, server) = server("voce-non-determinata", true);
        let path = root.join("Ambigua.tape");
        tape_at(&path, 1000);
        let mut document = tape::read(&path).unwrap();
        document.frasi[0].ingresso = Ingresso::Microfono;
        document.frasi[0].parlante_non_determinato = true;
        document
            .parlanti
            .insert("microfono".into(), "Mario Ambiguo".into());
        tape::rewrite(&path, &document).unwrap();
        Library::open(&root, &library::db_path(&server.places.index, &root))
            .unwrap()
            .sync()
            .unwrap();
        let Json(around) = server
            .read_around(Parameters(AroundParams {
                tape: "Ambigua.tape".into(),
                ingresso: "microfono".into(),
                phrase_id: 0,
                before: None,
                after: None,
            }))
            .unwrap();
        assert_eq!(
            around.frasi[0].parlante.as_deref(),
            Some("Parlante non determinato")
        );
        let Json(found) = server
            .search(Parameters(SearchParams {
                query: "Mario".into(),
                raccolta: None,
                limit: None,
            }))
            .unwrap();
        assert!(found.tapes.is_empty());
    }

    #[test]
    fn una_voce_provvisoria_si_legge_come_tale_anche_dall_assistente() {
        let (root, server) = server("voce-provvisoria", true);
        let path = root.join("Provvisoria.tape");
        tape_at(&path, 1000);
        let mut document = tape::read(&path).unwrap();
        document.frasi[0].parlante = Some(1);
        document.frasi[0].parlante_provvisorio = true;
        document
            .parlanti
            .insert("mix:1".into(), "Mario Incerto".into());
        tape::rewrite(&path, &document).unwrap();
        let Json(around) = server
            .read_around(Parameters(AroundParams {
                tape: "Provvisoria.tape".into(),
                ingresso: "mix".into(),
                phrase_id: 0,
                before: None,
                after: None,
            }))
            .unwrap();
        assert_eq!(
            around.frasi[0].parlante.as_deref(),
            Some("Mario Incerto (provvisorio)")
        );
    }

    #[test]
    fn si_elencano_i_tape_per_raccolta() {
        let (_, server) = server("elenca", true);
        let list = |raccolta: Option<&str>| {
            server
                .list_tapes(Parameters(ListParams {
                    raccolta: raccolta.map(Into::into),
                    from: None,
                    to: None,
                    limit: None,
                    offset: None,
                }))
                .unwrap()
                .0
        };
        let all = list(None);
        assert_eq!(all.raccolte, ["Ferrara Quarzi"]);
        assert_eq!(all.total, 2);
        assert_eq!(
            list(Some(""))
                .tapes
                .iter()
                .map(|b| b.tape.as_str())
                .collect::<Vec<_>>(),
            ["Sciolto.tape"]
        );
        assert_eq!(list(Some("Ferrara Quarzi")).total, 1);
    }

    #[test]
    fn il_testo_intero_e_il_markdown_di_copia_testo() {
        let (_, server) = server("testo", true);
        let text = server
            .read_transcript(Parameters(TranscriptParams {
                tape: "Ferrara Quarzi/Call di lunedì.tape".into(),
                from: None,
            }))
            .unwrap();
        assert!(text.starts_with("# Call di lunedì"), "{text}");
        assert!(text.contains("**Giulia Ferrara:**"), "{text}");
        assert!(!text.contains("read_transcript with from="), "{text}");
    }
}
