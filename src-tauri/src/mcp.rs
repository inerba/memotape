//! Il server MCP degli Assistenti (ADR-0012, ADR-0031): `memotape.exe --mcp`, lanciato da Claude o
//! Codex, parla in stdio, legge la Libreria e, con il secondo permesso, scrive i Tape (mai l'indice).
//! Non usa Tauri: le cartelle dell'app vengono da `dirs`, come le risolve Tauri, e le impostazioni
//! si rileggono a ogni chiamata, così gli interruttori di Impostazioni valgono subito.

use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Mutex, PoisonError};

use chrono::NaiveDate;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{PromptMessage, Role};
use rmcp::{
    ServerHandler, ServiceExt, prompt, prompt_handler, prompt_router, schemars, tool, tool_handler,
    tool_router,
};

use crate::cartelle::{self, Cartelle};
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
/// I Turni di una pagina di read_turns, se l'Assistente non ne chiede un altro numero, e al massimo;
/// la pagina si ferma prima anche quando il testo supera `PAGE_BYTES`.
const TURNS_LIMIT: u32 = 50;
const TURNS_MAX: u32 = 200;

const DISABLED: &str = "Memotape does not allow assistants to read its Library. Ask the user to turn on \
\"Allow assistants to read the Library\" in Memotape → Settings → Assistants.";
const READ_ONLY: &str = "Memotape does not allow assistants to modify its Library. Ask the user to \
turn on \"Allow assistants to modify the Library\" in Memotape → Settings → Assistants.";
const BUSY: &str = "Memotape is recording, transcribing or recognizing speakers on this Tape: try \
again when it has finished, or tell the user.";
const NOT_INDEXED: &str =
    "Memotape has not indexed this Library yet: ask the user to open Memotape once.";

/// Quelle che usa l'app: `%APPDATA%`, `%LOCALAPPDATA%` e Documenti (le cartelle note, come Tauri),
/// o in debug la cartella dati di prova.
fn cartelle_of_app() -> Option<Cartelle> {
    Some(Cartelle::risolvi(
        cartelle::prova(),
        &dirs::data_dir()?.join(IDENTIFIER),
        &dirs::data_local_dir()?.join(IDENTIFIER),
        dirs::document_dir().as_deref(),
    ))
}

/// Avvia il server su stdin e stdout e risponde finché l'Assistente non chiude stdin.
pub fn serve() -> ExitCode {
    let Some(places) = cartelle_of_app() else {
        eprintln!("cartelle dell'app non trovate");
        return ExitCode::FAILURE;
    };
    start_log();
    if let Some(dir) = &places.prova {
        log::info!("cartella dati di prova attiva: {}", dir.display());
    }
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TurniParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The index of the first turn to read: the `next` of the previous page. Omit to start.
    from: Option<u32>,
    /// How many turns to read at most (default 50, at most 200); a page also stops earlier when its
    /// text gets long.
    limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EditTurnParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The Ingresso of the turn, as returned by read_turns.
    ingresso: String,
    /// All the phrase ids of the turn (`phraseIds`), as returned by read_turns.
    phrase_ids: Vec<u32>,
    /// The current text of the turn, exactly as returned by read_turns.
    expected: String,
    /// The corrected text of the whole turn; it may contain line breaks, and may be empty.
    text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MergeTurnParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The Ingresso of the turn, as returned by read_turns.
    ingresso: String,
    /// All the phrase ids of the turn to reattribute (`phraseIds`), as returned by read_turns.
    phrase_ids: Vec<u32>,
    /// A phrase id of the turn just before or just after it, of the same Ingresso: the turn takes
    /// that turn's speaker.
    into_phrase_id: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RenameSpeakerParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The Ingresso of the phrase, as returned by read_turns.
    ingresso: String,
    /// The id of any phrase of the speaker to name, as returned by read_turns.
    phrase_id: u32,
    /// The speaker's name, e.g. "Giulia Ferrara".
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetDateParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The new date and time, in the user's local time, as YYYY-MM-DDTHH:MM (e.g. 2026-10-03T17:05).
    date: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RenameTapeParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The new title: the file name without `.tape`, unique in its folder.
    titolo: String,
}

/// I parametri degli strumenti e dei prompt che chiedono solo un Tape.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TapeParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaglioParams {
    /// The Tape, as its path relative to the Library returned by search or list_tapes.
    tape: String,
    /// The stretches to remove, in ms on the timeline of the Tape as it is now (all of them in
    /// one call): e.g. the `taglio` of each silence returned by find_silences.
    tagli: Vec<TaglioIn>,
}

/// Un Taglio chiesto dall'Assistente, in ms sulla linea del tempo del Tape.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaglioIn {
    inizio_ms: u32,
    fine_ms: u32,
}

/// Il Taglio che l'app propone per un Silenzio, in ms sulla linea del tempo del Tape.
#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaglioOut {
    inizio_ms: u32,
    fine_ms: u32,
}

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SilenziOut {
    #[serde(flatten)]
    tape: TapeOut,
    /// In order of time; empty if the Tape has no long silences or no phrases.
    silenzi: Vec<SilenzioOut>,
}

/// Dove cade un Silenzio.
#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DoveOut {
    /// Before the first phrase.
    Inizio,
    /// Between two turns.
    Mezzo,
    /// After the last phrase, up to the end of the Tape.
    Fine,
}

/// Un Silenzio, con il Taglio che l'app propone.
#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SilenzioOut {
    /// "inizio" (before the first phrase), "mezzo" (between two turns) or "fine" (after the last
    /// phrase, up to the end of the Tape).
    dove: DoveOut,
    /// Where the speech before it ends (0 at the start of the Tape).
    inizio_ms: u32,
    /// Where the speech after it starts (the duration of the Tape at its end).
    fine_ms: u32,
    /// The estimated silence, at least 5 s.
    silenzio_ms: u32,
    /// The cut Memotape proposes: the whole silence at the start and at the end, in between 1 s
    /// left on each side. Pass it to cut_tape.
    taglio: TaglioOut,
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

#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurniOut {
    #[serde(flatten)]
    tape: TapeOut,
    /// In order of time.
    turni: Vec<TurnoOut>,
    /// How many turns the Tape has.
    total: u32,
    /// The `from` of the next page; null if this is the last one.
    next: Option<u32>,
}

/// Un Turno come lo mostra l'editor del Turno.
#[derive(Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnoOut {
    ingresso: String,
    /// The phrases of the turn: pass them all to edit_turn.
    phrase_ids: Vec<u32>,
    /// The speaker's name, or the numbered label; null if not attributed.
    parlante: Option<String>,
    inizio_ms: u32,
    fine_ms: u32,
    /// The text of the turn, its phrases separated by line breaks: pass it to edit_turn as
    /// `expected`.
    testo: String,
}

#[derive(Clone)]
pub struct Server {
    places: Cartelle,
    /// Dove va l'originale di un Taglio: il Cestino di Windows; nei test, un volume senza.
    cestino: fn(&Path) -> Result<(), AppError>,
}

#[tool_router]
impl Server {
    pub fn new(places: Cartelle) -> Self {
        Self {
            places,
            cestino: library::trash,
        }
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
        let ingresso = ingresso(&p.ingresso)?;
        let document = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        let mut frasi: Vec<FraseOut> = document
            .frasi_visibili()
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
            tape: tape_of(&p.tape, &path, &document),
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

    #[tool(
        description = "Read the turns of a Tape, one page at a time: each turn is the consecutive \
phrases of one speaker, with its Ingresso, phrase ids, speaker, start and end, and its text exactly \
as Memotape's turn editor shows it. Use it before edit_turn.",
        annotations(title = "Read the turns", read_only_hint = true)
    )]
    fn read_turns(&self, Parameters(p): Parameters<TurniParams>) -> Result<Json<TurniOut>, String> {
        log::info!("Assistente: read_turns {} da {:?}", p.tape, p.from);
        let (settings, library) = self.allowed_library()?;
        let path = resolve(library.root(), &p.tape)?;
        let labels = transcription::labels(&settings);
        let document = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        let visibili = document.frasi_visibili();
        let turni = document.turni();
        let from = p.from.unwrap_or(0) as usize;
        let limit = bounded(p.limit, TURNS_LIMIT, TURNS_MAX);
        let mut page = Vec::new();
        let mut bytes = 0;
        for indices in turni.iter().skip(from).take(limit) {
            let testo = document.testo_del_turno(indices, "\n");
            bytes += testo.len();
            if !page.is_empty() && bytes > PAGE_BYTES {
                break;
            }
            let first = &visibili[indices[0]];
            page.push(TurnoOut {
                ingresso: first.ingresso.key().into(),
                phrase_ids: indices.iter().map(|&i| visibili[i].id).collect(),
                parlante: labels.assigned_name(
                    &document.parlanti,
                    first.ingresso,
                    first.parlante,
                    first.parlante_non_determinato,
                    first.parlante_provvisorio,
                ),
                inizio_ms: first.inizio_ms,
                fine_ms: indices
                    .iter()
                    .map(|&i| visibili[i].fine_ms)
                    .max()
                    .unwrap_or(0),
                testo,
            });
        }
        let end = from + page.len();
        Ok(Json(TurniOut {
            tape: tape_of(&p.tape, &path, &document),
            turni: page,
            total: u32::try_from(turni.len()).unwrap_or(u32::MAX),
            next: (end < turni.len()).then(|| u32::try_from(end).unwrap_or(u32::MAX)),
        }))
    }

    #[tool(
        description = "Correct the text of a whole turn of a Tape, as the user does in Memotape's \
turn editor: the correction is marked as made by hand. Pass the turn as read_turns returns it \
(Ingresso, all its phrase ids, and its current text as `expected`); if the turn changed since, \
nothing is written and you must read it again.",
        annotations(
            title = "Correct a turn",
            read_only_hint = false,
            destructive_hint = false
        )
    )]
    fn edit_turn(&self, Parameters(p): Parameters<EditTurnParams>) -> Result<String, String> {
        log::info!(
            "Assistente: edit_turn {} {}:{:?}",
            p.tape,
            p.ingresso,
            p.phrase_ids
        );
        let path = self.writable_tape(&p.tape)?;
        let ingresso = ingresso(&p.ingresso)?;
        let stale = || {
            format!(
                "The turn {} {:?} of {} changed since you read it: call read_turns again and \
correct its current text.",
                p.ingresso, p.phrase_ids, p.tape
            )
        };
        // Lo stesso controllo lo rifà `edit_turno` sotto il lock: qui dà all'Assistente l'errore
        // giusto, senza scrivere.
        let document = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        let indices = document.turno(ingresso, &p.phrase_ids).ok_or_else(stale)?;
        if document.testo_del_turno(&indices, "\n") != p.expected {
            return Err(stale());
        }
        tape::edit_turno(&path, ingresso, &p.phrase_ids, &p.expected, &p.text)
            .map_err(|e| write_error(&p.tape, &e))?;
        Ok(if p.expected == p.text {
            "Nothing to change: the text is the same.".into()
        } else {
            "Corrected.".into()
        })
    }

    #[tool(
        description = "Find the long silences of a Tape, as Memotape's cut mode shows them: before \
the first phrase, between two turns and after the last phrase, when at least 5 s long (estimated from \
the phrases of all the Ingressi, not from the audio volume, so a Tape without phrases has none). \
Each comes with the cut Memotape proposes: pass them to cut_tape in a single call.",
        annotations(title = "Find the silences", read_only_hint = true)
    )]
    fn find_silences(
        &self,
        Parameters(p): Parameters<TapeParams>,
    ) -> Result<Json<SilenziOut>, String> {
        log::info!("Assistente: find_silences {}", p.tape);
        let (_, library) = self.allowed_library()?;
        let path = resolve(library.root(), &p.tape)?;
        let document = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        let silenzi = document
            .silenzi()
            .into_iter()
            .map(|s| SilenzioOut {
                dove: match s.dove {
                    tape::DoveSilenzio::Inizio => DoveOut::Inizio,
                    tape::DoveSilenzio::Mezzo => DoveOut::Mezzo,
                    tape::DoveSilenzio::Fine => DoveOut::Fine,
                },
                inizio_ms: s.tratto.start,
                fine_ms: s.tratto.end,
                silenzio_ms: s.ms,
                taglio: TaglioOut {
                    inizio_ms: s.taglio.start,
                    fine_ms: s.taglio.end,
                },
            })
            .collect();
        Ok(Json(SilenziOut {
            tape: tape_of(&p.tape, &path, &document),
            silenzi,
        }))
    }

    #[tool(
        description = "Cut stretches out of a Tape, with their audio and their phrases, as Memotape's \
cut mode does: the Tape is rewritten once, shorter, and the phrases after each cut move earlier. The \
original Tape goes to the Windows Recycle Bin, so the user can restore it; on a drive without a \
Recycle Bin nothing is cut. Pass all the cuts in one call, in ms on the Tape as it is now (e.g. the \
cuts proposed by find_silences); call find_silences or read_turns again afterwards for the new times.",
        annotations(title = "Cut a Tape", read_only_hint = false, destructive_hint = true)
    )]
    fn cut_tape(&self, Parameters(p): Parameters<TaglioParams>) -> Result<String, String> {
        log::info!("Assistente: cut_tape {} {:?}", p.tape, p.tagli);
        let path = self.writable_tape(&p.tape)?;
        let tagli: Vec<_> = p.tagli.iter().map(|t| t.inizio_ms..t.fine_ms).collect();
        let prima = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        tape::taglia(&path, &tagli, self.cestino).map_err(|e| match e {
            AppError::NoRecycleBin(_) => format!(
                "{} is on a drive without a Recycle Bin: Memotape cuts a Tape only if the original \
can go to the Recycle Bin, so nothing was changed.",
                p.tape
            ),
            _ => write_error(&p.tape, &e),
        })?;
        let durata = tape::read(&path)
            .map_err(|e| read_error(&p.tape, &e))?
            .durata_ms;
        Ok(if durata == prima.durata_ms {
            "Nothing to cut: no stretch falls inside the Tape.".into()
        } else {
            format!(
                "Cut: the Tape now lasts {durata} ms instead of {} ms; the original is in the \
Recycle Bin.",
                prima.durata_ms
            )
        })
    }

    #[tool(
        description = "Merge a turn of a Tape into the turn just before or just after it, as the user \
does in Memotape to fix a wrong speaker attribution: the turn takes that turn's speaker, its text and \
times stay. Both turns must be of the same Ingresso and the other turn's speaker must be known. The \
attribution is marked as made by hand; turns change after it, so call read_turns again.",
        annotations(
            title = "Merge a turn",
            read_only_hint = false,
            destructive_hint = false
        )
    )]
    fn merge_turn(&self, Parameters(p): Parameters<MergeTurnParams>) -> Result<String, String> {
        log::info!(
            "Assistente: merge_turn {} {}:{:?} in {}",
            p.tape,
            p.ingresso,
            p.phrase_ids,
            p.into_phrase_id
        );
        let path = self.writable_tape(&p.tape)?;
        let ingresso = ingresso(&p.ingresso)?;
        // `unisci_turno` rifiuta senza scrivere le richieste non valide o obsolete.
        tape::unisci_turno(&path, ingresso, &p.phrase_ids, p.into_phrase_id).map_err(|e| {
            format!(
                "The turn {} {:?} of {} cannot be merged into phrase {} ({e}): pass a whole turn and \
a phrase of the turn just before or after it, of the same Ingresso and with a known speaker. Call \
read_turns again if the turns changed.",
                p.ingresso, p.phrase_ids, p.tape, p.into_phrase_id
            )
        })?;
        Ok("Merged.".into())
    }

    #[tool(
        description = "Name a speaker of a Tape, as the user does in Memotape: every phrase of that \
speaker, in that Ingresso, shows the new name. Identify the speaker by any of its phrases, as \
read_turns returns them. The name must differ from the other speakers' names of the same Ingresso; a \
phrase of an unknown speaker cannot be named (merge its turn instead).",
        annotations(
            title = "Name a speaker",
            read_only_hint = false,
            destructive_hint = false
        )
    )]
    fn rename_speaker(
        &self,
        Parameters(p): Parameters<RenameSpeakerParams>,
    ) -> Result<String, String> {
        log::info!(
            "Assistente: rename_speaker {} {}:{}",
            p.tape,
            p.ingresso,
            p.phrase_id
        );
        let path = self.writable_tape(&p.tape)?;
        let ingresso = ingresso(&p.ingresso)?;
        let nome = p.name.trim();
        if nome.is_empty() {
            return Err("The name is empty.".into());
        }
        let labels = transcription::labels(&path.settings);
        let document = tape::read(&path).map_err(|e| read_error(&p.tape, &e))?;
        let visibili = document.frasi_visibili();
        let frase = visibili
            .iter()
            .find(|f| f.ingresso == ingresso && f.id == p.phrase_id)
            .ok_or_else(|| {
                format!(
                    "{} has no phrase {} {}: call read_turns again.",
                    p.tape, p.ingresso, p.phrase_id
                )
            })?;
        // Come nell'app: si nominano i Parlanti noti e gli Ingressi senza Parlanti, non il mix.
        if frase.parlante_non_determinato || (frase.parlante.is_none() && ingresso == Ingresso::Mix)
        {
            return Err(format!(
                "The phrase {} {} has no known speaker to name: merge its turn into an adjacent one \
instead.",
                p.ingresso, p.phrase_id
            ));
        }
        // Come `nomeTaken` dell'app: due Parlanti con lo stesso nome non si distinguerebbero più.
        let taken = visibili.iter().any(|f| {
            f.ingresso == ingresso
                && !f.parlante_non_determinato
                && f.parlante != frase.parlante
                && labels
                    .nome(&document.parlanti, ingresso, f.parlante)
                    .as_deref()
                    == Some(nome)
        });
        if taken {
            return Err(format!(
                "Another speaker of {} is already named {nome:?}: choose another name.",
                p.ingresso
            ));
        }
        tape::rename_parlante(&path, ingresso, frase.parlante, nome)
            .map_err(|e| write_error(&p.tape, &e))?;
        Ok("Renamed.".into())
    }

    #[tool(
        description = "Change the date and time of a Tape (when it was recorded), as the user does in \
Memotape. Pass the user's local time as YYYY-MM-DDTHH:MM; returns the date written, with its UTC \
offset.",
        annotations(
            title = "Change the date",
            read_only_hint = false,
            destructive_hint = false
        )
    )]
    fn set_date(&self, Parameters(p): Parameters<SetDateParams>) -> Result<String, String> {
        log::info!("Assistente: set_date {} {}", p.tape, p.date);
        let path = self.writable_tape(&p.tape)?;
        // `set_creato` rifiuta senza scrivere una data non valida o un'ora che non esiste.
        let creato = tape::set_creato(&path, &p.date).map_err(|e| {
            format!(
                "The date of {} cannot be set to {:?} ({e}): pass an existing local date and time \
as YYYY-MM-DDTHH:MM, e.g. 2026-10-03T17:05.",
                p.tape, p.date
            )
        })?;
        Ok(format!("The Tape is now dated {creato}."))
    }

    #[tool(
        description = "Rename a Tape, that is change its title (its file name). Returns the new path \
of the Tape relative to the Library: use it from now on.",
        annotations(
            title = "Rename a Tape",
            read_only_hint = false,
            destructive_hint = false
        )
    )]
    fn rename_tape(&self, Parameters(p): Parameters<RenameTapeParams>) -> Result<String, String> {
        log::info!("Assistente: rename_tape {} → {:?}", p.tape, p.titolo);
        let path = self.writable_tape(&p.tape)?;
        let to = library::rename_tape(&path, &p.titolo).map_err(|e| write_error(&p.tape, &e))?;
        let name = to.file_name().unwrap_or_default();
        Ok(Path::new(&p.tape)
            .with_file_name(name)
            .display()
            .to_string())
    }

    #[tool(
        description = "Move a Tape to the Windows Recycle Bin, where the user can restore it.",
        annotations(
            title = "Move a Tape to the Recycle Bin",
            read_only_hint = false,
            destructive_hint = true
        )
    )]
    fn trash_tape(&self, Parameters(p): Parameters<TapeParams>) -> Result<String, String> {
        log::info!("Assistente: trash_tape {}", p.tape);
        let path = self.writable_tape(&p.tape)?;
        library::trash_tape(&path).map_err(|e| write_error(&p.tape, &e))?;
        Ok("Moved to the Recycle Bin.".into())
    }
}

#[prompt_router]
impl Server {
    #[prompt(
        name = "fix_tape",
        title = "Sistema il Tape",
        description = "Tidy up a Tape the way Memotape suggests: a light correction of its turns, \
cut its long silences, then a title if the current one is generic."
    )]
    fn fix_tape(&self, Parameters(p): Parameters<TapeParams>) -> Vec<PromptMessage> {
        log::info!("Assistente: prompt fix_tape {}", p.tape);
        vec![PromptMessage::new_text(
            Role::User,
            fix_tape_recipe(&p.tape),
        )]
    }
}

/// La ricetta di «Sistema il Tape» (ADR-0031) per il Tape `tape`: il titolo per ultimo, perché
/// cambia il percorso.
fn fix_tape_recipe(tape: &str) -> String {
    format!(
        "Fix the Memotape Tape `{tape}` with the Memotape tools, in this order.\n\n\
1. Light correction. Read its turns with read_turns (every page, following `next`) and correct a \
turn with edit_turn only where there are evident errors: misrecognized words, proper names spelled \
inconsistently, punctuation, capitalization, involuntary repetitions and filler words. Keep the \
speaker's style, word order and meaning; do not rewrite, summarize or translate. Leave a turn \
without evident errors untouched: it keeps its original text and word timings. Pass the turn's \
text from read_turns as `expected`.\n\n\
2. Silences. Call find_silences, then remove the silences with a single cut_tape call with all the \
cuts it proposes; the original Tape goes to the Windows Recycle Bin, where the user can restore \
it. If there are none, skip this step.\n\n\
3. Title, last, because it changes the Tape's path. Only if the current title is generic (the \
default of a Recording, a word followed by the date and time, or the name of the imported file), \
give the Tape a new one with rename_tape: short, at most 100 characters, in the language spoken \
in the Tape, about its subject, without a date (the Tape already has one). If the title is \
already taken, choose another one. Otherwise keep the current title.\n\n\
At the end, tell the user in a few lines what you changed."
    )
}

#[tool_handler(
    name = "memotape",
    instructions = "Memotape is the user's local transcription app. Its Library is a folder of Tape \
(.tape files): each Tape is a recording or a transcribed audio/video file, with its transcript split \
into phrases (start and end in ms), optionally attributed to speakers (Parlanti) and to an Ingresso: \
mix, microfono (the user's microphone) or sistema (system audio, e.g. the other people in a call). \
Raccolte are folders that group Tape. Use search to find phrases by words, then read_around for the \
context of a hit or read_transcript for a whole Tape; list_tapes lists Tapes by date. Tapes are \
identified by their path relative to the Library, as these tools return it. If the user allows it, \
you can also correct a Tape: read its turns with read_turns, then correct a turn with edit_turn, \
merge a turn into an adjacent one with merge_turn, name a speaker with rename_speaker and change the \
date with set_date; rename_tape changes its title, trash_tape moves it to the Recycle Bin; find its \
long silences with find_silences and remove them with one cut_tape. The fix_tape prompt gives the \
recipe to tidy up a Tape."
)]
#[prompt_handler]
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
            None => self
                .places
                .libreria
                .clone()
                .ok_or("Documents folder not found")?,
        };
        let db = library::db_path(&self.places.indice, &root);
        if !db.is_file() {
            return Err(NOT_INDEXED.into());
        }
        let library = Library::open_read_only(&root, &db)
            .map_err(|e| format!("Ask the user to open Memotape to update its index ({e})."))?;
        Ok((settings, library))
    }

    /// Il Tape `tape` da scrivere, se l'utente consente agli Assistenti di leggere e di modificare
    /// la Libreria e l'app non ci sta lavorando: ogni strumento che scrive passa da qui. Aprire la
    /// Libreria registra la sua cartella, così le riscritture prendono il lock su file condiviso
    /// con l'app (ADR-0031).
    fn writable_tape(&self, tape: &str) -> Result<Writable, String> {
        let (settings, library) = self.allowed_library()?;
        if !settings.assistenti_modifica {
            return Err(READ_ONLY.into());
        }
        let path = resolve(library.root(), tape)?;
        let lock = tape::prova_attivita(&path).map_err(|e| write_error(tape, &e))?;
        Ok(Writable {
            path,
            settings,
            _lock: lock,
        })
    }
}

/// Un Tape da scrivere: finché vive tiene il lock dell'Attività, così l'app non ne avvia una sul
/// Tape a metà della scrittura.
struct Writable {
    path: PathBuf,
    /// Le impostazioni lette per il permesso: i nomi dei Parlanti.
    settings: Settings,
    _lock: Option<std::fs::File>,
}

impl std::ops::Deref for Writable {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.path
    }
}

/// L'Ingresso di nome `key`, come lo restituiscono gli strumenti.
fn ingresso(key: &str) -> Result<Ingresso, String> {
    Ingresso::from_key(key)
        .ok_or_else(|| format!("Unknown ingresso {key:?}: use mix, microfono or sistema."))
}

/// Il Tape `tape`, in `path`, con il suo documento.
fn tape_of(tape: &str, path: &Path, document: &tape::Document) -> TapeOut {
    TapeOut {
        tape: tape.to_owned(),
        titolo: transcription::title_of(path),
        raccolta: library::raccolta_of(tape),
        creato: document.creato.clone(),
        durata_ms: Some(document.durata_ms),
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
        Err(not_found(tape))
    }
}

/// Il Tape `tape` non c'è più, anche se l'indice lo elenca ancora: l'indice lo riallinea l'app.
fn not_found(tape: &str) -> String {
    format!(
        "{tape} is not in the Library any more: ask the user to open Memotape, which updates its \
index, then search again."
    )
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

/// L'errore di una scrittura, da leggere per l'Assistente.
fn write_error(tape: &str, e: &AppError) -> String {
    match e {
        AppError::ActivityInProgress => BUSY.into(),
        AppError::NameTaken(titolo) => format!(
            "A Tape titled {titolo:?} already exists in the same folder: choose another title."
        ),
        AppError::InvalidName(titolo) => format!(
            "{titolo:?} is not a valid title: it must not be empty, start with a dot, end with a dot \
or a space, or contain any of < > : \" / \\ | ? *."
        ),
        AppError::TapeNotFound(_) => not_found(tape),
        AppError::NoRecycleBin(_) => format!(
            "{tape} is on a drive without a Recycle Bin: ask the user to delete it in Memotape."
        ),
        _ => format!("{tape} cannot be changed: {e}"),
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
        server_with(name, assistenti, false)
    }

    /// Come `server`, con il permesso a modificare scritto così nel file delle impostazioni.
    fn server_with(name: &str, assistenti: bool, assistenti_modifica: bool) -> (PathBuf, Server) {
        let dir = temp_dir(&format!("mcp-{name}"));
        let root = dir.join("Memotape");
        let places = Cartelle {
            settings: dir.join("settings.json"),
            indice: dir.join("indice"),
            libreria: None,
            prova: None,
        };
        Settings {
            recordings_folder: Some(root.display().to_string()),
            assistenti,
            assistenti_modifica,
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
        Library::open(&root, &library::db_path(&places.indice, &root))
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
        std::fs::remove_dir_all(&server.places.indice).unwrap();
        let list = server.list_tapes(Parameters(ListParams {
            raccolta: None,
            from: None,
            to: None,
            limit: None,
            offset: None,
        }));
        assert_eq!(list.err().as_deref(), Some(NOT_INDEXED));
        assert!(!server.places.indice.exists());
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
        Library::open(&root, &library::db_path(&server.places.indice, &root))
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
    fn editor_turno_ricerca_e_assistenti_leggono_solo_la_correzione() {
        let (root, server) = server("editor-turno", true);
        let path = root.join("Editor.tape");
        tape_with(
            &path,
            &[
                ("Prima originale.", Some(1)),
                ("Seconda originale.", Some(1)),
                ("Altro turno.", Some(2)),
            ],
            &[],
        );
        let originale = "Prima originale.\nSeconda originale.";
        let corretto = "Testo riunito.\n\nNuovo paragrafo.";
        tape::edit_turno(&path, Ingresso::Mix, &[0, 1], originale, corretto).unwrap();
        let mut library =
            Library::open(&root, &library::db_path(&server.places.indice, &root)).unwrap();
        library.sync().unwrap();
        let Json(around) = server
            .read_around(Parameters(AroundParams {
                tape: "Editor.tape".into(),
                ingresso: "mix".into(),
                phrase_id: 0,
                before: None,
                after: None,
            }))
            .unwrap();
        assert_eq!(around.frasi[0].testo, corretto);
        assert_eq!(around.frasi[1].testo, "");
        let markdown = server
            .read_transcript(Parameters(TranscriptParams {
                tape: "Editor.tape".into(),
                from: None,
            }))
            .unwrap();
        assert!(markdown.contains(corretto));
        assert!(!markdown.contains("originale"));
        let found = library.search("paragrafo", None).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].frasi[0].phrase_id, 0);
        assert!(library.search("originale", None).unwrap().is_empty());
        tape::edit_turno(&path, Ingresso::Mix, &[0, 1], corretto, "").unwrap();
        library.sync().unwrap();
        assert!(library.search("paragrafo", None).unwrap().is_empty());
        let markdown = server
            .read_transcript(Parameters(TranscriptParams {
                tape: "Editor.tape".into(),
                from: None,
            }))
            .unwrap();
        assert!(!markdown.contains("Testo riunito"));
        assert!(!markdown.contains("Blocco senza testo"));
        assert!(markdown.contains("Altro turno."));
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

    const CALL: &str = r"Ferrara Quarzi\Call di lunedì.tape";

    fn read_turns(server: &Server, tape: &str, from: Option<u32>, limit: Option<u32>) -> TurniOut {
        server
            .read_turns(Parameters(TurniParams {
                tape: tape.into(),
                from,
                limit,
            }))
            .unwrap()
            .0
    }

    fn edit_turn(
        server: &Server,
        tape: &str,
        ids: &[u32],
        expected: &str,
        text: &str,
    ) -> Result<String, String> {
        server.edit_turn(Parameters(EditTurnParams {
            tape: tape.into(),
            ingresso: "mix".into(),
            phrase_ids: ids.to_vec(),
            expected: expected.into(),
            text: text.into(),
        }))
    }

    /// I byte del Tape e dell'indice: il server non tocca l'uno quando rifiuta, l'altro mai.
    fn bytes(root: &Path, server: &Server, tape: &str) -> (Vec<u8>, Vec<u8>) {
        (
            std::fs::read(root.join(tape)).unwrap(),
            std::fs::read(library::db_path(&server.places.indice, root)).unwrap(),
        )
    }

    #[test]
    fn senza_il_permesso_a_modificare_non_si_corregge_nulla() {
        for (lettura, modifica, errore) in [(true, false, READ_ONLY), (false, true, DISABLED)] {
            let (root, server) = server_with(&format!("sola-lettura-{lettura}"), lettura, modifica);
            let prima = bytes(&root, &server, CALL);
            let corretto = edit_turn(
                &server,
                CALL,
                &[0],
                "Buongiorno a tutti.",
                "Buonasera a tutti.",
            );
            assert_eq!(corretto.err().as_deref(), Some(errore));
            let unito = merge_turn(&server, CALL, "mix", &[1], 0);
            assert_eq!(unito.err().as_deref(), Some(errore));
            let nominato = rename_speaker(&server, "mix", 0, "Marco");
            assert_eq!(nominato.err().as_deref(), Some(errore));
            let datato = set_date(&server, "2025-12-31T23:30");
            assert_eq!(datato.err().as_deref(), Some(errore));
            assert_eq!(bytes(&root, &server, CALL), prima);
        }
    }

    #[test]
    fn un_attivita_sul_tape_blocca_le_scritture_solo_su_quel_tape() {
        use crate::managers::activity::Activity;
        let (root, server) = server_with("attivita", true, true);
        let activity = Activity::default();
        let correggi = || {
            edit_turn(
                &server,
                CALL,
                &[0],
                "Buongiorno a tutti.",
                "Buonasera a tutti.",
            )
        };
        // L'Attività prende il lock come nell'app, con il percorso che usa l'app.
        let call = root.join("Ferrara Quarzi").join("Call di lunedì.tape");
        let attivita = activity.begin(Some(call), || {}).unwrap();
        let prima = bytes(&root, &server, CALL);
        assert_eq!(correggi().err().as_deref(), Some(BUSY));
        assert_eq!(bytes(&root, &server, CALL), prima);
        // Lo stesso per il taglio e la rinomina: il Tape resta dov'era, intero.
        let tagliato = cut_tape(&server, CALL, &[(0, 1000)]);
        assert_eq!(tagliato.err().as_deref(), Some(BUSY));
        let rinominato = rename_tape(&server, CALL, "Altro titolo");
        assert_eq!(rinominato.err().as_deref(), Some(BUSY));
        // E per unione, nomi, data e Cestino.
        let unito = merge_turn(&server, CALL, "mix", &[1], 0);
        assert_eq!(unito.err().as_deref(), Some(BUSY));
        let nominato = rename_speaker(&server, "mix", 0, "Marco");
        assert_eq!(nominato.err().as_deref(), Some(BUSY));
        let datato = set_date(&server, "2025-12-31T23:30");
        assert_eq!(datato.err().as_deref(), Some(BUSY));
        let buttato = server.trash_tape(Parameters(TapeParams { tape: CALL.into() }));
        assert_eq!(buttato.err().as_deref(), Some(BUSY));
        assert_eq!(bytes(&root, &server, CALL), prima);
        // Finita l'Attività, come dopo un errore o Annulla, il lock è libero; un'Attività su un
        // altro Tape non blocca.
        drop(attivita);
        let _altra = activity
            .begin(Some(root.join("Sciolto.tape")), || {})
            .unwrap();
        assert!(correggi().is_ok());
    }

    #[test]
    fn si_leggono_i_turni_e_se_ne_corregge_uno_a_mano() {
        let (root, server) = server_with("corregge", true, true);
        let indice = bytes(&root, &server, CALL).1;
        let turni = read_turns(&server, CALL, None, None);
        assert_eq!((turni.total, turni.next), (4, None));
        assert_eq!(turni.tape.titolo, "Call di lunedì");
        assert_eq!(
            turni.turni[1],
            TurnoOut {
                ingresso: "mix".into(),
                phrase_ids: vec![1],
                parlante: Some("Giulia Ferrara".into()),
                inizio_ms: 1000,
                fine_ms: 1900,
                testo: "Parliamo del preventivo per l'impianto.".into(),
            }
        );
        let vecchio = turni.turni[1].testo.clone();
        let nuovo = "Parliamo del preventivo per l'impianto elettrico.";
        assert!(edit_turn(&server, CALL, &[1], &vecchio, nuovo).is_ok());
        assert_eq!(read_turns(&server, CALL, None, None).turni[1].testo, nuovo);
        // Una seconda correzione sul testo di prima è obsoleta: si rifiuta senza scrivere.
        let prima = bytes(&root, &server, CALL);
        let obsoleta = edit_turn(&server, CALL, &[1], &vecchio, "Altro.");
        assert!(obsoleta.unwrap_err().contains("read_turns"));
        assert_eq!(bytes(&root, &server, CALL), prima);
        let document = tape::read(&root.join(CALL)).unwrap();
        assert!(document.corretto_a_mano());
        assert!(document.frasi[1].testo_corretto);
        // Il server non scrive mai l'indice.
        assert_eq!(bytes(&root, &server, CALL).1, indice);
    }

    #[test]
    fn i_turni_si_leggono_a_pagine_e_un_turno_di_piu_frasi_si_corregge_intero() {
        let (root, server) = server_with("pagine", true, true);
        tape_with(
            &root.join("Lungo.tape"),
            &[
                ("Prima.", Some(1)),
                ("Seconda.", Some(1)),
                ("Terza.", Some(2)),
                ("Quarta.", Some(1)),
            ],
            &[],
        );
        let pagina = read_turns(&server, "Lungo.tape", None, Some(2));
        assert_eq!((pagina.total, pagina.next), (3, Some(2)));
        let primo = &pagina.turni[0];
        assert_eq!(primo.phrase_ids, [0, 1]);
        assert_eq!(primo.testo, "Prima.\nSeconda.");
        assert_eq!(primo.parlante.as_deref(), Some("Parlante 1"));
        assert_eq!((primo.inizio_ms, primo.fine_ms), (0, 1900));
        let resto = read_turns(&server, "Lungo.tape", Some(2), Some(2));
        assert_eq!(resto.next, None);
        assert_eq!(resto.turni.len(), 1);
        assert_eq!(resto.turni[0].testo, "Quarta.");
        let unito = "Prima e seconda.";
        assert!(edit_turn(&server, "Lungo.tape", &[0, 1], "Prima.\nSeconda.", unito).is_ok());
        let turni = read_turns(&server, "Lungo.tape", None, None);
        assert_eq!(turni.turni[0].phrase_ids, [0, 1]);
        assert_eq!(turni.turni[0].testo, unito);
        // Una parte di un Turno, o Frasi di due Turni, non sono un Turno.
        assert!(edit_turn(&server, "Lungo.tape", &[0], unito, "x").is_err());
        assert!(edit_turn(&server, "Lungo.tape", &[2, 3], "Terza.", "x").is_err());
    }

    fn merge_turn(
        server: &Server,
        tape: &str,
        ingresso: &str,
        ids: &[u32],
        into: u32,
    ) -> Result<String, String> {
        server.merge_turn(Parameters(MergeTurnParams {
            tape: tape.into(),
            ingresso: ingresso.into(),
            phrase_ids: ids.to_vec(),
            into_phrase_id: into,
        }))
    }

    #[test]
    fn si_unisce_un_turno_al_turno_adiacente() {
        let (root, server) = server_with("unisce", true, true);
        let indice = bytes(&root, &server, CALL).1;
        // Non adiacente, o solo una parte di un Turno: rifiutato senza scrivere.
        let prima = bytes(&root, &server, CALL);
        assert!(
            merge_turn(&server, CALL, "mix", &[3], 0)
                .unwrap_err()
                .contains("read_turns")
        );
        assert!(merge_turn(&server, CALL, "mix", &[1], 1).is_err());
        assert_eq!(bytes(&root, &server, CALL), prima);
        assert!(merge_turn(&server, CALL, "mix", &[1], 0).is_ok());
        let turni = read_turns(&server, CALL, None, None);
        assert_eq!(turni.total, 2);
        assert_eq!(turni.turni[0].phrase_ids, [0, 1, 2]);
        assert_eq!(turni.turni[0].parlante.as_deref(), Some("Parlante 1"));
        assert!(tape::read(&root.join(CALL)).unwrap().frasi[1].parlante_corretto);
        assert_eq!(bytes(&root, &server, CALL).1, indice);
    }

    #[test]
    fn non_si_unisce_un_turno_a_un_altro_ingresso() {
        let (root, server) = server_with("ingressi", true, true);
        let path = root.join("Ingressi.tape");
        tape_with(
            &path,
            &[("Io.", None), ("Lei.", Some(1)), ("Lui.", Some(2))],
            &[],
        );
        let mut document = tape::read(&path).unwrap();
        document.frasi[0].ingresso = Ingresso::Microfono;
        for frase in &mut document.frasi[1..] {
            frase.ingresso = Ingresso::Sistema;
        }
        tape::rewrite(&path, &document).unwrap();
        let prima = std::fs::read(&path).unwrap();
        assert!(merge_turn(&server, "Ingressi.tape", "sistema", &[1], 0).is_err());
        assert!(merge_turn(&server, "Ingressi.tape", "microfono", &[0], 1).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), prima);
        assert!(merge_turn(&server, "Ingressi.tape", "sistema", &[2], 1).is_ok());
        let turni = read_turns(&server, "Ingressi.tape", None, None);
        assert_eq!(turni.turni[1].phrase_ids, [1, 2]);
    }

    fn rename_speaker(
        server: &Server,
        ingresso: &str,
        id: u32,
        name: &str,
    ) -> Result<String, String> {
        server.rename_speaker(Parameters(RenameSpeakerParams {
            tape: CALL.into(),
            ingresso: ingresso.into(),
            phrase_id: id,
            name: name.into(),
        }))
    }

    #[test]
    fn si_da_un_nome_al_parlante_di_una_frase() {
        let (root, server) = server_with("nomi", true, true);
        let indice = bytes(&root, &server, CALL).1;
        let prima = bytes(&root, &server, CALL);
        // Il nome di un altro Parlante dello stesso Ingresso, un nome vuoto, una Frase che non c'è.
        assert!(rename_speaker(&server, "mix", 0, " Giulia Ferrara ").is_err());
        assert!(rename_speaker(&server, "mix", 0, "  ").is_err());
        assert!(rename_speaker(&server, "mix", 9, "Marco").is_err());
        assert!(rename_speaker(&server, "sistema", 0, "Marco").is_err());
        assert_eq!(bytes(&root, &server, CALL), prima);
        assert!(rename_speaker(&server, "mix", 2, " Marco Quarzi ").is_ok());
        let turni = read_turns(&server, CALL, None, None);
        assert_eq!(turni.turni[0].parlante.as_deref(), Some("Marco Quarzi"));
        assert_eq!(turni.turni[2].parlante.as_deref(), Some("Marco Quarzi"));
        assert_eq!(turni.turni[1].parlante.as_deref(), Some("Giulia Ferrara"));
        assert_eq!(bytes(&root, &server, CALL).1, indice);
    }

    fn set_date(server: &Server, date: &str) -> Result<String, String> {
        server.set_date(Parameters(SetDateParams {
            tape: CALL.into(),
            date: date.into(),
        }))
    }

    #[test]
    fn si_cambiano_data_e_ora_del_tape() {
        let (root, server) = server_with("data", true, true);
        let indice = bytes(&root, &server, CALL).1;
        let prima = bytes(&root, &server, CALL);
        for sbagliata in ["31/12/2025", "2025-13-01T10:00", "2025-12-31"] {
            assert!(
                set_date(&server, sbagliata)
                    .unwrap_err()
                    .contains("YYYY-MM-DDTHH:MM")
            );
        }
        assert_eq!(bytes(&root, &server, CALL), prima);
        assert!(
            set_date(&server, "2025-12-31T23:30")
                .unwrap()
                .contains("2025-12-31T23:30")
        );
        let creato = read_turns(&server, CALL, None, None).tape.creato;
        assert!(creato.starts_with("2025-12-31T23:30:00"), "{creato}");
        assert_eq!(bytes(&root, &server, CALL).1, indice);
    }

    #[test]
    fn non_si_scrive_fuori_dalla_libreria() {
        let (root, server) = server_with("fuori", true, true);
        let fuori = root.parent().unwrap().join("Fuori.tape");
        tape_with(&fuori, &[("Fuori.", Some(1))], &[]);
        let prima = std::fs::read(&fuori).unwrap();
        for tape in [
            fuori.display().to_string(),
            r"..\Fuori.tape".into(),
            r"Acme\..\..\Fuori.tape".into(),
        ] {
            let scritto = edit_turn(&server, &tape, &[0], "Fuori.", "Dentro.");
            assert!(scritto.is_err(), "{tape}");
        }
        assert_eq!(std::fs::read(&fuori).unwrap(), prima);
    }

    /// Il Tape `nome` nella Libreria `root`, lungo `durata_ms` con `audio_s` secondi di audio, con le
    /// Frasi `frasi` (Ingresso, inizio, fine) in ordine di inizio.
    fn tape_di(
        root: &Path,
        nome: &str,
        durata_ms: u32,
        audio_s: f64,
        frasi: &[(Ingresso, u32, u32)],
    ) -> String {
        use crate::audio_toolkit::ogg_opus::{OggOpusWriter, tests::sine};
        let ogg = root.parent().unwrap().join(format!("{nome}.ogg"));
        let mut writer =
            OggOpusWriter::new(std::fs::File::create(&ogg).unwrap(), 48_000, 2, 64).unwrap();
        writer.write(&sine(48_000, 2, audio_s)).unwrap();
        writer.finish().unwrap();
        let phrases: Vec<_> = frasi
            .iter()
            .map(
                |&(ingresso, inizio_ms, fine_ms)| crate::transcript::Phrase {
                    inizio_ms,
                    fine_ms,
                    text: format!("Frase delle {inizio_ms}."),
                    ingresso,
                    parlante: None,
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                    tempi: Vec::new(),
                },
            )
            .collect();
        let document = tape::Document::new(
            "2026-10-04T10:15:00+02:00".into(),
            durata_ms,
            tape::Modalita::Mix,
            None,
            crate::managers::settings::SpeechLanguage::from("it"),
            true,
            &phrases,
        );
        std::fs::create_dir_all(root).unwrap();
        tape::write(&root.join(nome), &[(Ingresso::Mix, &ogg)], &document, None).unwrap();
        nome.into()
    }

    fn find_silences(server: &Server, tape: &str) -> Result<Vec<SilenzioOut>, String> {
        server
            .find_silences(Parameters(TapeParams { tape: tape.into() }))
            .map(|out| out.0.silenzi)
    }

    /// Un Silenzio come lo restituisce `find_silences`, con il suo Taglio proposto.
    fn silenzio(dove: DoveOut, a: u32, b: u32, ms: u32, taglio: (u32, u32)) -> SilenzioOut {
        SilenzioOut {
            dove,
            inizio_ms: a,
            fine_ms: b,
            silenzio_ms: ms,
            taglio: TaglioOut {
                inizio_ms: taglio.0,
                fine_ms: taglio.1,
            },
        }
    }

    #[test]
    fn si_trovano_i_silenzi_della_modalita_taglio_con_il_taglio_proposto() {
        // Basta il permesso a leggere.
        let (root, server) = server_with("silenzi", true, false);
        let mix = Ingresso::Mix;
        // Saluti, la riunione, una pausa caffè di 7 minuti, la chiusura e 2 minuti di stanza vuota.
        let riunione = tape_di(
            &root,
            "Riunione.tape",
            800_000,
            1.0,
            &[
                (mix, 4000, 9000),
                (mix, 12_000, 19_000),
                (mix, 164_000, 170_000),
                (mix, 171_000, 180_000),
                (mix, 645_000, 650_000),
                (mix, 650_000, 655_000),
            ],
        );
        assert_eq!(
            find_silences(&server, &riunione).unwrap(),
            [
                silenzio(DoveOut::Mezzo, 19_000, 164_000, 146_000, (20_000, 163_000)),
                silenzio(
                    DoveOut::Mezzo,
                    180_000,
                    645_000,
                    466_000,
                    (181_000, 644_000)
                ),
                silenzio(DoveOut::Fine, 655_000, 800_000, 145_000, (656_000, 800_000)),
            ]
        );
        // All'inizio dai 5 s in su, tolto tutto; in mezzo il silenzio stimato aggiunge hangover e
        // prefill: 3,999 s fra le Frasi sono 4,999 s, 4 s sono 5 s.
        let soglie = tape_di(
            &root,
            "Soglie.tape",
            30_000,
            1.0,
            &[
                (mix, 5000, 6000),
                (mix, 9999, 11_000),
                (mix, 15_000, 26_000),
            ],
        );
        assert_eq!(
            find_silences(&server, &soglie).unwrap(),
            [
                silenzio(DoveOut::Inizio, 0, 5000, 5000, (0, 4000)),
                silenzio(DoveOut::Mezzo, 11_000, 15_000, 5000, (12_000, 14_000)),
            ]
        );
        // Con gli Ingressi separati il silenzio si misura dalla fine più tarda: il Microfono parla
        // sopra il Sistema e oltre.
        let accavallati = tape_di(
            &root,
            "Accavallati.tape",
            42_000,
            1.0,
            &[
                (Ingresso::Microfono, 6000, 20_000),
                (Ingresso::Sistema, 8000, 12_000),
                (Ingresso::Microfono, 30_000, 40_000),
            ],
        );
        assert_eq!(
            find_silences(&server, &accavallati).unwrap(),
            [
                silenzio(DoveOut::Inizio, 0, 6000, 6000, (0, 5000)),
                silenzio(DoveOut::Mezzo, 20_000, 30_000, 11_000, (21_000, 29_000)),
            ]
        );
        // Un Tape senza Frasi non ha Silenzi: non si taglia tutto l'audio di un Tape non trascritto.
        let muto = tape_di(&root, "Muto.tape", 60_000, 1.0, &[]);
        assert_eq!(find_silences(&server, &muto).unwrap(), []);
    }

    #[test]
    fn un_turno_corretto_non_si_divide_nei_silenzi() {
        let (root, server) = server_with("silenzi-correzione", true, false);
        let mix = Ingresso::Mix;
        let tape = tape_di(
            &root,
            "Corretto.tape",
            30_000,
            1.0,
            &[(mix, 0, 2000), (mix, 10_000, 12_000), (mix, 20_000, 30_000)],
        );
        let path = root.join(&tape);
        let mut document = tape::read(&path).unwrap();
        document.correzioni_testo.push(tape::TestoTurno {
            ingresso: mix,
            frasi: vec![0, 1],
            testo: "Le prime due, corrette.".into(),
        });
        tape::rewrite(&path, &document).unwrap();
        assert_eq!(
            find_silences(&server, &tape).unwrap(),
            [silenzio(
                DoveOut::Mezzo,
                12_000,
                20_000,
                9000,
                (13_000, 19_000)
            )]
        );
    }

    #[test]
    fn senza_il_permesso_a_leggere_non_si_trovano_silenzi() {
        let (_, server) = server_with("silenzi-spento", false, true);
        assert_eq!(
            find_silences(&server, CALL).err().as_deref(),
            Some(DISABLED)
        );
    }

    fn cut_tape(server: &Server, tape: &str, tagli: &[(u32, u32)]) -> Result<String, String> {
        server.cut_tape(Parameters(TaglioParams {
            tape: tape.into(),
            tagli: tagli
                .iter()
                .map(|&(inizio_ms, fine_ms)| TaglioIn { inizio_ms, fine_ms })
                .collect(),
        }))
    }

    /// Il Cestino dei test: l'originale finisce accanto, con l'estensione `.nel-cestino`.
    fn nel_cestino(path: &Path) -> Result<(), AppError> {
        std::fs::rename(path, path.with_extension("nel-cestino"))
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Una chiamata di 30 s: 8 s di attesa, due battute fra 10 s di silenzio e 8 s di coda.
    fn chiamata(root: &Path) -> String {
        let mix = Ingresso::Mix;
        tape_di(
            root,
            "Chiamata.tape",
            30_000,
            30.0,
            &[(mix, 8000, 10_000), (mix, 20_000, 22_000)],
        )
    }

    #[test]
    fn i_tagli_proposti_accorciano_il_tape_e_l_originale_va_nel_cestino() {
        let (root, mut server) = server_with("taglio", true, true);
        server.cestino = nel_cestino;
        let tape = chiamata(&root);
        let indice = bytes(&root, &server, CALL).1;
        let originale = tape::read(&root.join(&tape)).unwrap();
        let tagli: Vec<_> = find_silences(&server, &tape)
            .unwrap()
            .into_iter()
            .map(|s| (s.taglio.inizio_ms, s.taglio.fine_ms))
            .collect();
        assert_eq!(tagli.len(), 3);
        assert!(cut_tape(&server, &tape, &tagli).is_ok());
        let tagliato = tape::read(&root.join(&tape)).unwrap();
        // Resta 1 s di silenzio accanto a ogni Frase: 1 + 2 + 1 + 1 + 2 + 1 s, meno i 6,5 ms del
        // pre-skip dell'Opus: con la coda tagliata restano 400 pacchetti interi (ADR-0030).
        assert_eq!(tagliato.durata_ms, 7993);
        let tempi: Vec<_> = tagliato
            .frasi
            .iter()
            .map(|f| (f.inizio_ms, f.fine_ms))
            .collect();
        assert_eq!(tempi, [(1000, 3000), (5000, 7000)]);
        assert_eq!(find_silences(&server, &tape).unwrap(), []);
        assert_eq!(
            tape::read(&root.join(&tape).with_extension("nel-cestino")).unwrap(),
            originale
        );
        // Il server non scrive mai l'indice.
        assert_eq!(bytes(&root, &server, CALL).1, indice);
    }

    #[test]
    fn senza_cestino_il_taglio_si_rifiuta_e_il_tape_resta_com_e() {
        let (root, mut server) = server_with("taglio-senza-cestino", true, true);
        server.cestino = |path| Err(AppError::NoRecycleBin(path.display().to_string()));
        let tape = chiamata(&root);
        let prima = std::fs::read(root.join(&tape)).unwrap();
        let rifiuto = cut_tape(&server, &tape, &[(0, 7000)]).unwrap_err();
        assert!(rifiuto.contains("Recycle Bin"), "{rifiuto}");
        assert_eq!(std::fs::read(root.join(&tape)).unwrap(), prima);
    }

    #[test]
    fn senza_il_permesso_a_modificare_non_si_taglia() {
        for (lettura, modifica, errore) in [(true, false, READ_ONLY), (false, true, DISABLED)] {
            let (root, server) =
                server_with(&format!("taglio-spento-{lettura}"), lettura, modifica);
            let prima = bytes(&root, &server, CALL);
            let tagliato = cut_tape(&server, CALL, &[(0, 1000)]);
            assert_eq!(tagliato.err().as_deref(), Some(errore));
            assert_eq!(bytes(&root, &server, CALL), prima);
        }
    }

    fn rename_tape(server: &Server, tape: &str, titolo: &str) -> Result<String, String> {
        server.rename_tape(Parameters(RenameTapeParams {
            tape: tape.into(),
            titolo: titolo.into(),
        }))
    }

    #[test]
    fn si_rinomina_un_tape_e_si_continua_dal_percorso_nuovo() {
        let (root, server) = server_with("rinomina", true, true);
        let indice = bytes(&root, &server, CALL).1;
        let nuovo = rename_tape(&server, CALL, "Preventivo dell'impianto").unwrap();
        assert_eq!(nuovo, r"Ferrara Quarzi\Preventivo dell'impianto.tape");
        assert!(!root.join(CALL).exists());
        assert_eq!(read_turns(&server, &nuovo, None, None).total, 4);
        // Il server non scrive l'indice, che elenca ancora il percorso vecchio: chi lo legge è
        // mandato ad aprire Memotape.
        assert_eq!(bytes(&root, &server, &nuovo).1, indice);
        let vecchio = server.read_transcript(Parameters(TranscriptParams {
            tape: CALL.into(),
            from: None,
        }));
        assert!(vecchio.unwrap_err().contains("open Memotape"));
        let gia_usato = rename_tape(&server, &nuovo, "Sciolto");
        assert!(gia_usato.is_ok(), "un'altra cartella: {gia_usato:?}");
        tape_at(&root.join("Ferrara Quarzi").join("Altra call.tape"), 1000);
        let preso = rename_tape(&server, r"Ferrara Quarzi\Sciolto.tape", "Altra call");
        assert!(preso.unwrap_err().contains("already"));
        let non_valido = rename_tape(&server, r"Ferrara Quarzi\Sciolto.tape", "Call: lunedì?");
        assert!(non_valido.unwrap_err().contains("not a valid"));
        let sparito = rename_tape(&server, CALL, "Altro titolo");
        assert!(sparito.unwrap_err().contains("open Memotape"));
    }

    #[test]
    fn senza_il_permesso_a_modificare_non_si_rinomina_ne_si_butta() {
        for (lettura, modifica, errore) in [(true, false, READ_ONLY), (false, true, DISABLED)] {
            let (root, server) = server_with(
                &format!("rinomina-sola-lettura-{lettura}"),
                lettura,
                modifica,
            );
            let prima = bytes(&root, &server, CALL);
            assert_eq!(
                rename_tape(&server, CALL, "Altro").err().as_deref(),
                Some(errore)
            );
            let buttato = server.trash_tape(Parameters(TapeParams { tape: CALL.into() }));
            assert_eq!(buttato.err().as_deref(), Some(errore));
            assert_eq!(bytes(&root, &server, CALL), prima);
        }
    }

    #[test]
    fn un_tape_buttato_va_nel_cestino_e_l_indice_non_cambia() {
        let (root, server) = server_with("cestino", true, true);
        let indice = bytes(&root, &server, CALL).1;
        let buttato = server.trash_tape(Parameters(TapeParams { tape: CALL.into() }));
        assert!(buttato.is_ok(), "{buttato:?}");
        assert!(!root.join(CALL).exists());
        assert_eq!(
            std::fs::read(library::db_path(&server.places.indice, &root)).unwrap(),
            indice
        );
        let di_nuovo = server.trash_tape(Parameters(TapeParams { tape: CALL.into() }));
        assert!(di_nuovo.unwrap_err().contains("open Memotape"));
        let letto = server.read_turns(Parameters(TurniParams {
            tape: CALL.into(),
            from: None,
            limit: None,
        }));
        assert!(letto.err().unwrap().contains("open Memotape"));
    }

    #[test]
    fn il_server_offre_la_ricetta_sistema_il_tape_per_il_tape_indicato() {
        let (_, server) = server("prompt", true);
        assert!(server.get_info().capabilities.prompts.is_some());
        let prompts = Server::prompt_router().list_all();
        let names: Vec<_> = prompts.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["fix_tape"]);
        let argomenti = prompts[0].arguments.as_deref().unwrap_or_default();
        assert!(
            argomenti
                .iter()
                .any(|a| a.name == "tape" && a.required == Some(true)),
            "{argomenti:?}"
        );
        let messaggi = server.fix_tape(Parameters(TapeParams { tape: CALL.into() }));
        let testo: String = messaggi
            .iter()
            .filter_map(|m| m.content.as_text().map(|t| t.text.clone()))
            .collect();
        assert!(testo.contains(CALL), "{testo}");
        // I tre passi nel loro ordine: correzione, Silenzi, titolo per ultimo.
        let at = |strumento: &str| {
            testo
                .find(strumento)
                .unwrap_or_else(|| panic!("{strumento}"))
        };
        assert!(at("read_turns") < at("edit_turn"));
        assert!(at("edit_turn") < at("find_silences"));
        assert!(at("find_silences") < at("cut_tape"));
        assert!(at("cut_tape") < at("rename_tape"));
        assert!(testo.contains("100 characters"), "{testo}");
    }
}
