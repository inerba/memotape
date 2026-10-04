//! La Libreria (ADR-0008): la cartella dei Bini, con le Raccolte come cartelle di primo livello, e
//! un indice SQLite che si allinea alla cartella e si ricostruisce dai Bini. Senza Tauri.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use chrono::{DateTime, Local};
use rusqlite::{Connection, OptionalExtension, params};

use crate::bino;
use crate::error::AppError;
use crate::transcript::Ingresso;

/// La `user_version` dell'indice: con un numero diverso si ricostruisce.
const SCHEMA: i32 = 2;

const SCHEMA_SQL: &str = "
CREATE TABLE bini (
    percorso TEXT PRIMARY KEY,
    raccolta TEXT,
    titolo TEXT NOT NULL,
    creato TEXT NOT NULL,
    durata_ms INTEGER,
    modificato INTEGER NOT NULL,
    dimensione INTEGER NOT NULL
);
-- Una riga per Frase e una per il titolo (`frase` NULL). `parlante` è il nome dato al Parlante.
CREATE VIRTUAL TABLE ricerca USING fts5(
    percorso UNINDEXED,
    frase UNINDEXED,
    ingresso UNINDEXED,
    inizio_ms UNINDEXED,
    testo,
    parlante,
    tokenize = 'unicode61 remove_diacritics 2'
);
PRAGMA user_version = 2;
";

/// Dove cominciano e finiscono le parole trovate negli estratti della ricerca.
pub const MARK_START: char = '\u{1}';
pub const MARK_END: char = '\u{2}';
/// Le Frasi trovate per ogni Bino e i Bini di una ricerca.
const FRASI_PER_BINO: u32 = 5;
const BINI_PER_RICERCA: usize = 50;

/// Le Raccolte e tutti i Bini della Libreria.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
pub struct LibraryList {
    /// I nomi delle Raccolte, in ordine alfabetico.
    pub raccolte: Vec<String>,
    /// Dal più recente.
    pub bini: Vec<BinoEntry>,
}

/// Un Bino trovato dalla ricerca, con le Frasi trovate in ordine di inizio (nessuna se ha trovato
/// solo il titolo).
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
pub struct SearchResult {
    pub bino: BinoEntry,
    pub frasi: Vec<SearchHit>,
}

/// Una Frase trovata, con l'estratto in cui le parole trovate stanno tra `MARK_START` e `MARK_END`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub phrase_id: u32,
    pub ingresso: Ingresso,
    pub inizio_ms: u32,
    pub estratto: String,
}

/// Un Bino della Libreria, come lo mostra la barra laterale.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BinoEntry {
    pub path: String,
    /// `null`: Senza raccolta.
    pub raccolta: Option<String>,
    /// Il nome del file senza estensione.
    pub titolo: String,
    /// `creato` del Bino; per un Bino illeggibile la data di modifica del file.
    pub creato: String,
    /// `null` per un Bino illeggibile o di una versione futura.
    pub durata_ms: Option<u32>,
}

pub struct Library {
    root: PathBuf,
    db: Connection,
}

impl Library {
    /// Apre la Libreria `root` con l'indice `db`, che si ricostruisce se manca, è corrotto o ha
    /// un'altra versione. Non si allinea: lo fa `sync`.
    pub fn open(root: &Path, db: &Path) -> Result<Self, AppError> {
        if let Some(dir) = db.parent() {
            std::fs::create_dir_all(dir).map_err(|e| unwritable(dir, &e))?;
        }
        let db = connect(db).or_else(|e| {
            log::warn!(
                "indice della Libreria da ricostruire ({}): {e}",
                db.display()
            );
            for suffix in ["", "-journal", "-wal", "-shm"] {
                let mut file = db.as_os_str().to_owned();
                file.push(suffix);
                let _ = std::fs::remove_file(file);
            }
            connect(db).map_err(|e| AppError::Internal(format!("{}: {e}", db.display())))
        })?;
        Ok(Self {
            root: root.to_path_buf(),
            db,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Allinea l'indice alla cartella: rilegge i Bini nuovi o con data di modifica o dimensione
    /// cambiate e toglie quelli spariti. Restituisce quanti Bini ha riletto.
    pub fn sync(&mut self) -> Result<usize, AppError> {
        let mut found = Vec::new();
        walk(&self.root, &mut found)?;
        let tx = self.db.transaction().map_err(internal)?;
        let indexed: HashMap<String, (i64, i64)> = {
            let mut query = tx
                .prepare("SELECT percorso, modificato, dimensione FROM bini")
                .map_err(internal)?;
            query
                .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
                .and_then(Iterator::collect)
                .map_err(internal)?
        };
        let present: std::collections::HashSet<&str> =
            found.iter().map(|f| f.relative.as_str()).collect();
        for gone in indexed.keys().filter(|p| !present.contains(p.as_str())) {
            tx.execute("DELETE FROM bini WHERE percorso = ?1", [gone])
                .map_err(internal)?;
            tx.execute("DELETE FROM ricerca WHERE percorso = ?1", [gone])
                .map_err(internal)?;
        }
        let mut reread = 0;
        for file in &found {
            if indexed.get(&file.relative) == Some(&(file.modified, file.size)) {
                continue;
            }
            reread += 1;
            let path = self.root.join(&file.relative);
            let document = bino::read(&path)
                .inspect_err(|e| log::warn!("Bino illeggibile nella Libreria: {e}"))
                .ok();
            // ponytail: il DELETE su `percorso` scorre tutta la tabella FTS, quindi si fa solo per i
            // Bini già nell'indice e una ricostruzione non lo paga. Se diventa lento: una tabella
            // delle Frasi con un indice su `percorso` e la FTS a contenuto esterno.
            if indexed.contains_key(&file.relative) {
                tx.execute("DELETE FROM ricerca WHERE percorso = ?1", [&file.relative])
                    .map_err(internal)?;
            }
            let titolo = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            tx.execute(
                "INSERT INTO ricerca (percorso, testo) VALUES (?1, ?2)",
                params![file.relative, titolo],
            )
            .map_err(internal)?;
            if let Some(document) = &document {
                for frase in &document.frasi {
                    let parlante = frase
                        .parlante
                        .and_then(|n| document.parlanti.get(&frase.ingresso.parlante_key(n)));
                    tx.execute(
                        "INSERT INTO ricerca VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            file.relative,
                            frase.id,
                            frase.ingresso.key(),
                            frase.inizio_ms,
                            frase.testo,
                            parlante,
                        ],
                    )
                    .map_err(internal)?;
                }
            }
            let creato = document.as_ref().map_or_else(
                || {
                    DateTime::<Local>::from(UNIX_EPOCH + nanos(file.modified))
                        .format("%Y-%m-%dT%H:%M:%S%:z")
                        .to_string()
                },
                |d| d.creato.clone(),
            );
            tx.execute(
                "INSERT OR REPLACE INTO bini VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    file.relative,
                    raccolta_of(&file.relative),
                    titolo,
                    creato,
                    document.map(|d| d.durata_ms),
                    file.modified,
                    file.size,
                ],
            )
            .map_err(internal)?;
        }
        tx.commit().map_err(internal)?;
        Ok(reread)
    }

    /// Le Raccolte (le cartelle di primo livello, escluse quelle che iniziano con `.`) e i Bini
    /// dell'indice.
    pub fn list(&self) -> Result<LibraryList, AppError> {
        let mut raccolte: Vec<String> = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|name| !name.starts_with('.'))
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(unreadable(&self.root, &e)),
        };
        raccolte.sort_by_key(|name| name.to_lowercase());
        let mut query = self
            .db
            .prepare(
                "SELECT percorso, raccolta, titolo, creato, durata_ms FROM bini
                 ORDER BY creato DESC",
            )
            .map_err(internal)?;
        let bini = query
            .query_map([], |r| self.entry(r))
            .and_then(Iterator::collect)
            .map_err(internal)?;
        Ok(LibraryList { raccolte, bini })
    }

    /// Il Bino dalle prime cinque colonne di `r`: percorso, Raccolta, titolo, `creato`, durata.
    fn entry(&self, r: &rusqlite::Row) -> rusqlite::Result<BinoEntry> {
        Ok(BinoEntry {
            path: self.root.join(r.get::<_, String>(0)?).display().to_string(),
            raccolta: r.get(1)?,
            titolo: r.get(2)?,
            creato: r.get(3)?,
            durata_ms: r.get(4)?,
        })
    }

    /// Cerca `query` nei titoli, nelle Frasi e nei nomi dei Parlanti dei Bini della Raccolta
    /// `raccolta` (`None` tutta la Libreria, `""` Senza raccolta). Ogni parola vale come inizio di
    /// parola, senza maiuscole né accenti, e tutte devono stare nella stessa Frase (o nel titolo).
    /// Prima i Bini con più testo pertinente: la somma dei `bm25` delle loro righe.
    pub fn search(
        &self,
        query: &str,
        raccolta: Option<&str>,
    ) -> Result<Vec<SearchResult>, AppError> {
        let Some(query) = match_query(query) else {
            return Ok(Vec::new());
        };
        // ponytail: `snippet()` si calcola per tutte le righe trovate, anche oltre le
        // `FRASI_PER_BINO`; da spostare in una seconda query se una parola comune diventa lenta.
        let mut statement = self
            .db
            .prepare(
                "WITH trovate AS (
                     SELECT percorso, frase, ingresso, inizio_ms, bm25(ricerca) AS peso,
                            snippet(ricerca, 4, ?3, ?4, '…', 16) AS estratto,
                            highlight(ricerca, 5, ?3, ?4) AS parlante
                     FROM ricerca WHERE ricerca MATCH ?1
                 ), pesate AS (
                     SELECT *, sum(peso) OVER (PARTITION BY percorso) AS totale,
                            row_number() OVER (
                                PARTITION BY percorso ORDER BY frase IS NULL, peso
                            ) AS n
                     FROM trovate
                 )
                 SELECT b.percorso, b.raccolta, b.titolo, b.creato, b.durata_ms,
                        p.frase, p.ingresso, p.inizio_ms, p.estratto, p.parlante
                 FROM pesate p JOIN bini b ON b.percorso = p.percorso
                 WHERE p.n <= ?5 AND (?2 IS NULL OR coalesce(b.raccolta, '') = ?2)
                 ORDER BY p.totale, b.percorso, p.inizio_ms",
            )
            .map_err(internal)?;
        let rows = statement
            .query_map(
                params![
                    query,
                    raccolta,
                    MARK_START.to_string(),
                    MARK_END.to_string(),
                    FRASI_PER_BINO
                ],
                |r| {
                    let hit = match (r.get::<_, Option<u32>>(5)?, r.get::<_, Option<String>>(6)?) {
                        (Some(phrase_id), Some(ingresso)) => {
                            Ingresso::from_key(&ingresso).map(|ingresso| {
                                Ok::<_, rusqlite::Error>(SearchHit {
                                    phrase_id,
                                    ingresso,
                                    inizio_ms: r.get(7)?,
                                    estratto: estratto(r.get(8)?, r.get(9)?),
                                })
                            })
                        }
                        _ => None,
                    };
                    Ok((self.entry(r)?, hit.transpose()?))
                },
            )
            .map_err(internal)?;
        let mut results: Vec<SearchResult> = Vec::new();
        for row in rows {
            let (bino, hit) = row.map_err(internal)?;
            if results.last().is_none_or(|r| r.bino.path != bino.path) {
                if results.len() == BINI_PER_RICERCA {
                    break;
                }
                results.push(SearchResult {
                    bino,
                    frasi: Vec::new(),
                });
            }
            if let (Some(hit), Some(result)) = (hit, results.last_mut()) {
                result.frasi.push(hit);
            }
        }
        Ok(results)
    }

    /// La cartella in cui va un Bino nuovo della Raccolta `raccolta`: la sua, o la radice per
    /// `None` (Tutta la Libreria) e `""` (Senza raccolta).
    pub fn raccolta_dir(root: &Path, raccolta: Option<&str>) -> Result<PathBuf, AppError> {
        match raccolta {
            None | Some("") => Ok(root.to_path_buf()),
            Some(nome) => {
                validate_name(nome)?;
                Ok(root.join(nome))
            }
        }
    }

    pub fn create_raccolta(&mut self, nome: &str) -> Result<(), AppError> {
        validate_name(nome)?;
        let dir = self.root.join(nome);
        if dir.exists() {
            return Err(AppError::NameTaken(nome.into()));
        }
        std::fs::create_dir_all(&dir).map_err(|e| unwritable(&dir, &e))?;
        self.sync().map(drop)
    }

    /// Rinomina la Raccolta e la sua cartella; restituisce la cartella vecchia e quella nuova.
    pub fn rename_raccolta(
        &mut self,
        nome: &str,
        nuovo: &str,
    ) -> Result<(PathBuf, PathBuf), AppError> {
        let from = self.existing_raccolta(nome)?;
        validate_name(nuovo)?;
        let to = self.root.join(nuovo);
        // Cambiare solo maiuscole e minuscole è una rinomina: il file system non le distingue.
        if to.exists() && !nome.eq_ignore_ascii_case(nuovo) {
            return Err(AppError::NameTaken(nuovo.into()));
        }
        std::fs::rename(&from, &to).map_err(|e| unwritable(&from, &e))?;
        self.sync()?;
        Ok((from, to))
    }

    /// Elimina la cartella della Raccolta, solo se vuota.
    pub fn delete_raccolta(&mut self, nome: &str) -> Result<(), AppError> {
        let dir = self.existing_raccolta(nome)?;
        let mut entries = std::fs::read_dir(&dir).map_err(|e| unreadable(&dir, &e))?;
        if entries.next().is_some() {
            return Err(AppError::RaccoltaNotEmpty(nome.into()));
        }
        std::fs::remove_dir(&dir).map_err(|e| unwritable(&dir, &e))?;
        self.sync().map(drop)
    }

    /// Rinomina il file del Bino in `<titolo>.bino`, con la sua estensione. Restituisce il percorso
    /// nuovo.
    pub fn rename_bino(&mut self, path: &Path, titolo: &str) -> Result<PathBuf, AppError> {
        existing_bino(path)?;
        validate_name(titolo)?;
        let mut name = std::ffi::OsString::from(titolo);
        if let Some(extension) = path.extension() {
            name.push(".");
            name.push(extension);
        }
        let to = path.with_file_name(name);
        let same = path.file_stem().is_some_and(|stem| {
            stem.to_string_lossy()
                .to_lowercase()
                .eq(&titolo.to_lowercase())
        });
        if to.exists() && !same {
            return Err(AppError::NameTaken(titolo.into()));
        }
        std::fs::rename(path, &to).map_err(|e| unwritable(path, &e))?;
        self.sync()?;
        Ok(to)
    }

    /// Sposta il Bino nella Raccolta `raccolta` (la radice per `None` o `""`), anche da fuori della
    /// Libreria. Restituisce il percorso nuovo.
    pub fn move_bino(&mut self, path: &Path, raccolta: Option<&str>) -> Result<PathBuf, AppError> {
        existing_bino(path)?;
        let dir = Self::raccolta_dir(&self.root, raccolta)?;
        let to = dir.join(path.file_name().unwrap_or_default());
        if same_path(&to, path) {
            return Ok(path.to_path_buf());
        }
        if to.exists() {
            return Err(AppError::NameTaken(
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
            ));
        }
        std::fs::create_dir_all(&dir).map_err(|e| unwritable(&dir, &e))?;
        move_file(path, &to)?;
        self.sync()?;
        Ok(to)
    }

    /// Manda il Bino nel Cestino.
    pub fn trash_bino(&mut self, path: &Path) -> Result<(), AppError> {
        existing_bino(path)?;
        trash(path)?;
        self.sync().map(drop)
    }

    fn existing_raccolta(&self, nome: &str) -> Result<PathBuf, AppError> {
        validate_name(nome)?;
        let dir = self.root.join(nome);
        if dir.is_dir() {
            Ok(dir)
        } else {
            Err(AppError::UnreadableFile(dir.display().to_string()))
        }
    }
}

/// Il file dell'indice per la Libreria `root`, in `dir`: uno per cartella, con il nome ricavato dal
/// percorso, così tornare a una cartella già usata non ricostruisce l'indice.
pub fn db_path(dir: &Path, root: &Path) -> PathBuf {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(root.to_string_lossy().to_lowercase().as_bytes());
    let name: String = hash[..8].iter().map(|b| format!("{b:02x}")).collect();
    dir.join(format!("{name}.sqlite"))
}

/// Un nome valido per una Raccolta o il titolo di un Bino: non vuoto, senza i caratteri che Windows
/// non ammette, non un nome riservato (`CON`, `NUL`, `COM1`…), senza punto o spazio in fondo e senza
/// punto in testa (le cartelle con il punto non sono Raccolte).
pub fn validate_name(name: &str) -> Result<(), AppError> {
    let forbidden = |c: char| c.is_control() || r#"<>:"/\|?*"#.contains(c);
    let base = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_ascii_uppercase();
    let device = matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (base.len() == 4
            && (base.starts_with("COM") || base.starts_with("LPT"))
            && matches!(base.as_bytes()[3], b'1'..=b'9'));
    if name.is_empty()
        || name.contains(forbidden)
        || name.ends_with(['.', ' '])
        || name.starts_with('.')
        || device
    {
        return Err(AppError::InvalidName(name.into()));
    }
    Ok(())
}

/// Manda `path` nel Cestino di Windows. Su un volume senza Cestino (una condivisione di rete)
/// rifiuta: l'app non cancella mai per sempre. Se il Cestino è disattivato in Windows, è Windows a
/// chiedere conferma prima di cancellare (`FOF_WANTNUKEWARNING`).
pub fn trash(path: &Path) -> Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::{
        FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_SILENT, FOF_WANTNUKEWARNING,
        SHFILEOPSTRUCTW, SHFileOperationW, SHQUERYRBINFO, SHQueryRecycleBinW,
    };
    let absolute = std::path::absolute(path).map_err(|e| unwritable(path, &e))?;
    let volume: Vec<u16> = absolute
        .ancestors()
        .last()
        .unwrap_or(&absolute)
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect();
    let mut info = SHQUERYRBINFO {
        cbSize: u32::try_from(std::mem::size_of::<SHQUERYRBINFO>()).unwrap_or_default(),
        i64Size: 0,
        i64NumItems: 0,
    };
    // SAFETY: `volume` è un percorso terminato da zero e `info` ha `cbSize` impostato.
    if unsafe { SHQueryRecycleBinW(volume.as_ptr(), &raw mut info) } < 0 {
        return Err(AppError::UnwritableFolder(format!(
            "{}: il volume non ha un Cestino",
            path.display()
        )));
    }
    // Un elenco di percorsi terminato da due zeri.
    let from: Vec<u16> = absolute.as_os_str().encode_wide().chain([0, 0]).collect();
    let mut operation = SHFILEOPSTRUCTW {
        hwnd: std::ptr::null_mut(),
        wFunc: FO_DELETE,
        pFrom: from.as_ptr(),
        pTo: std::ptr::null(),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_WANTNUKEWARNING) as u16,
        fAnyOperationsAborted: 0,
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: std::ptr::null(),
    };
    // SAFETY: `from` è un elenco terminato da due zeri che vive per tutta la chiamata, e gli altri
    // puntatori sono nulli, come ammette `FO_DELETE`.
    let result = unsafe { SHFileOperationW(&raw mut operation) };
    if operation.fAnyOperationsAborted != 0 {
        return Err(AppError::Cancelled);
    }
    if result != 0 {
        return Err(AppError::UnwritableFolder(format!(
            "{}: SHFileOperationW {result:#x}",
            path.display()
        )));
    }
    Ok(())
}

/// L'estratto di una Frase; se le parole sono nel nome del Parlante, il nome prima del testo.
fn estratto(testo: String, parlante: Option<String>) -> String {
    match parlante {
        Some(nome) if nome.contains(MARK_START) => format!("{nome}: {testo}"),
        _ => testo,
    }
}

/// La query FTS5 per le parole di `query`: ognuna tra virgolette (così i caratteri speciali di FTS5
/// non contano) e come prefisso, unite in AND. `None` se non c'è nessuna parola.
fn match_query(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .map(|w| format!("\"{}\"*", w.replace('"', "\"\"")))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}

fn connect(path: &Path) -> rusqlite::Result<Connection> {
    let db = Connection::open(path)?;
    let version: i32 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version != SCHEMA {
        if version != 0 {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "user_version {version}"
            )));
        }
        db.execute_batch(SCHEMA_SQL)?;
    }
    db.query_row("SELECT count(*) FROM bini", [], |_| Ok(()))
        .optional()?;
    Ok(db)
}

/// Un `.bino` trovato nella cartella.
struct Found {
    /// Relativo alla Libreria.
    relative: String,
    /// Nanosecondi dal 1970.
    modified: i64,
    size: i64,
}

/// I `.bino` sotto `root`, saltando le cartelle che iniziano con `.` (come `.sbobino`).
fn walk(root: &Path, found: &mut Vec<Found>) -> Result<(), AppError> {
    fn visit(root: &Path, dir: &Path, found: &mut Vec<Found>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)?.filter_map(Result::ok) {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            if kind.is_dir() {
                if let Err(e) = visit(root, &path, found) {
                    log::warn!("{} non letta: {e}", path.display());
                }
            } else if kind.is_file() && bino::is_bino(&path) {
                let Ok(metadata) = entry.metadata() else {
                    continue;
                };
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX));
                found.push(Found {
                    relative: path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .display()
                        .to_string(),
                    modified,
                    size: i64::try_from(metadata.len()).unwrap_or(i64::MAX),
                });
            }
        }
        Ok(())
    }
    match visit(root, root, found) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result.map_err(|e| unreadable(root, &e)),
    }
}

/// La Raccolta di un Bino dal percorso relativo alla Libreria: la cartella di primo livello, se
/// non sta nella radice.
fn raccolta_of(relative: &str) -> Option<String> {
    let mut components = Path::new(relative).components();
    let first = components.next()?;
    components.next()?;
    match first {
        Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
        _ => None,
    }
}

fn nanos(n: i64) -> std::time::Duration {
    std::time::Duration::from_nanos(u64::try_from(n).unwrap_or_default())
}

/// Sposta un file, anche su un altro volume (copia e poi cancella l'originale).
fn move_file(from: &Path, to: &Path) -> Result<(), AppError> {
    const ERROR_NOT_SAME_DEVICE: i32 = 17;
    match std::fs::rename(from, to) {
        Err(e) if e.raw_os_error() == Some(ERROR_NOT_SAME_DEVICE) => {
            std::fs::copy(from, to).map_err(|e| {
                let _ = std::fs::remove_file(to);
                unwritable(to, &e)
            })?;
            std::fs::remove_file(from).map_err(|e| unwritable(from, &e))
        }
        result => result.map_err(|e| unwritable(from, &e)),
    }
}

/// Se `a` e `b` sono lo stesso percorso per Windows, che non distingue maiuscole e minuscole.
pub fn same_path(a: &Path, b: &Path) -> bool {
    a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
}

/// Se `path` è `dir` o sta dentro `dir`, senza distinguere maiuscole e minuscole.
pub fn inside(path: &Path, dir: &Path) -> bool {
    let lower = |p: &Path| PathBuf::from(p.to_string_lossy().to_lowercase());
    lower(path).starts_with(lower(dir))
}

fn existing_bino(path: &Path) -> Result<(), AppError> {
    if bino::is_bino(path) && path.is_file() {
        Ok(())
    } else {
        Err(AppError::BinoNotFound(path.display().to_string()))
    }
}

fn internal(e: rusqlite::Error) -> AppError {
    AppError::Internal(format!("indice della Libreria: {e}"))
}

fn unreadable(path: &Path, e: &std::io::Error) -> AppError {
    AppError::UnreadableFile(format!("{}: {e}", path.display()))
}

fn unwritable(path: &Path, e: &std::io::Error) -> AppError {
    AppError::UnwritableFolder(format!("{}: {e}", path.display()))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::fs::File;

    use super::*;
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{sine, temp_dir};
    use crate::managers::settings::SpeechLanguage;
    use crate::transcript::{Ingresso, Phrase};

    /// Un Bino vero in `path`, con `durata_ms` e una Frase.
    pub fn bino_at(path: &Path, durata_ms: u32) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let ogg = path.with_extension("ogg.tmp");
        let mut writer = OggOpusWriter::new(File::create(&ogg).unwrap(), 16_000, 1, 16).unwrap();
        writer.write(&sine(16_000, 1, 0.1)).unwrap();
        writer.finish().unwrap();
        let document = bino::Document::new(
            "2026-10-04T10:15:00+02:00".into(),
            durata_ms,
            bino::Modalita::Mix,
            None,
            SpeechLanguage::It,
            true,
            &[Phrase {
                inizio_ms: 0,
                fine_ms: 90,
                text: "Buongiorno.".into(),
                ingresso: Ingresso::Mix,
                parlante: None,
            }],
        );
        bino::write(path, &[(Ingresso::Mix, &ogg)], &document).unwrap();
        std::fs::remove_file(ogg).unwrap();
    }

    /// Una Libreria in una cartella temporanea, con l'indice accanto.
    fn library(name: &str) -> (PathBuf, Library) {
        let dir = temp_dir(&format!("libreria-{name}"));
        let root = dir.join("Sbobino");
        let library = Library::open(&root, &db_path(&dir.join("indice"), &root)).unwrap();
        (root, library)
    }

    fn titoli(library: &mut Library) -> Vec<(Option<String>, String)> {
        library.sync().unwrap();
        let mut bini: Vec<_> = library
            .list()
            .unwrap()
            .bini
            .into_iter()
            .map(|b| (b.raccolta, b.titolo))
            .collect();
        bini.sort();
        bini
    }

    fn entry(raccolta: Option<&str>, titolo: &str) -> (Option<String>, String) {
        (raccolta.map(Into::into), titolo.into())
    }

    #[test]
    fn le_raccolte_sono_le_cartelle_di_primo_livello_e_la_radice_e_senza_raccolta() {
        let (root, mut library) = library("raccolte");
        // Prima che la cartella esista la Libreria è vuota.
        assert_eq!(titoli(&mut library), []);
        bino_at(&root.join("Sciolto.bino"), 1000);
        bino_at(&root.join("Ferrara Quarzi").join("Preventivo.bino"), 2000);
        bino_at(
            &root
                .join("Ferrara Quarzi")
                .join("2025")
                .join("Vecchia.bino"),
            3000,
        );
        bino_at(&root.join(".sbobino").join("Nascosto.bino"), 1000);
        bino_at(
            &root.join("Acme").join(".bozze").join("Nascosto.bino"),
            1000,
        );
        std::fs::create_dir_all(root.join("Vuota")).unwrap();
        std::fs::write(root.join("Note.md"), "non è un Bino").unwrap();
        assert_eq!(
            titoli(&mut library),
            [
                entry(None, "Sciolto"),
                entry(Some("Ferrara Quarzi"), "Preventivo"),
                entry(Some("Ferrara Quarzi"), "Vecchia"),
            ]
        );
        let list = library.list().unwrap();
        assert_eq!(list.raccolte, ["Acme", "Ferrara Quarzi", "Vuota"]);
        let preventivo = list.bini.iter().find(|b| b.titolo == "Preventivo").unwrap();
        assert_eq!(
            preventivo.path,
            root.join("Ferrara Quarzi")
                .join("Preventivo.bino")
                .display()
                .to_string()
        );
        assert_eq!(preventivo.creato, "2026-10-04T10:15:00+02:00");
        assert_eq!(preventivo.durata_ms, Some(2000));
    }

    #[test]
    fn le_modifiche_fatte_da_esplora_file_si_vedono_dopo_l_allineamento() {
        let (root, mut library) = library("esplora-file");
        bino_at(&root.join("Uno.bino"), 1000);
        bino_at(&root.join("Due.bino"), 1000);
        bino_at(&root.join("Tre.bino"), 1000);
        std::fs::create_dir_all(root.join("Acme")).unwrap();
        assert_eq!(library.sync().unwrap(), 3);
        std::fs::rename(root.join("Uno.bino"), root.join("Acme").join("Uno.bino")).unwrap();
        std::fs::rename(root.join("Due.bino"), root.join("Secondo.bino")).unwrap();
        std::fs::remove_file(root.join("Tre.bino")).unwrap();
        bino_at(&root.join("Quattro.bino"), 1000);
        assert_eq!(
            titoli(&mut library),
            [
                entry(None, "Quattro"),
                entry(None, "Secondo"),
                entry(Some("Acme"), "Uno"),
            ]
        );
    }

    #[test]
    fn si_rileggono_solo_i_bini_con_data_o_dimensione_cambiate() {
        let (root, mut library) = library("rilettura");
        bino_at(&root.join("Uno.bino"), 1000);
        bino_at(&root.join("Due.bino"), 1000);
        assert_eq!(library.sync().unwrap(), 2);
        assert_eq!(library.sync().unwrap(), 0);
        let path = root.join("Uno.bino");
        let mut document = bino::read(&path).unwrap();
        document.durata_ms = 5000;
        bino::rewrite(&path, &document).unwrap();
        assert_eq!(library.sync().unwrap(), 1);
        let uno = library.list().unwrap().bini;
        assert!(
            uno.iter()
                .any(|b| b.titolo == "Uno" && b.durata_ms == Some(5000))
        );
        // Stessa dimensione e stessa data: non si rilegge.
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        document.durata_ms = 6000;
        bino::rewrite(&path, &document).unwrap();
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        assert_eq!(library.sync().unwrap(), 0);
    }

    #[test]
    fn un_bino_illeggibile_resta_nell_elenco_con_il_nome_del_file() {
        let (root, mut library) = library("illeggibile");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Rotto.bino"), "non è uno zip").unwrap();
        bino_at(&root.join("Buono.bino"), 1000);
        library.sync().unwrap();
        let bini = library.list().unwrap().bini;
        let rotto = bini.iter().find(|b| b.titolo == "Rotto").unwrap();
        assert_eq!(rotto.durata_ms, None);
        assert!(
            DateTime::parse_from_rfc3339(&rotto.creato).is_ok(),
            "{}",
            rotto.creato
        );
        assert!(bini.iter().any(|b| b.titolo == "Buono"));
    }

    #[test]
    fn un_indice_cancellato_corrotto_o_di_un_altra_versione_si_ricostruisce() {
        let dir = temp_dir("libreria-ricostruzione");
        let root = dir.join("Sbobino");
        let db = db_path(&dir.join("indice"), &root);
        bino_at(&root.join("Acme").join("Uno.bino"), 1000);
        bino_at(&root.join("Due.bino"), 2000);
        let listed = |db: &Path| {
            let mut library = Library::open(&root, db).unwrap();
            library.sync().unwrap();
            library.list().unwrap()
        };
        let before = listed(&db);
        assert_eq!(before.bini.len(), 2);
        std::fs::remove_file(&db).unwrap();
        assert_eq!(listed(&db), before);
        std::fs::write(&db, "spazzatura, non un database SQLite").unwrap();
        assert_eq!(listed(&db), before);
        Connection::open(&db)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        assert_eq!(listed(&db), before);
        // Un file per cartella.
        assert_ne!(db, db_path(&dir.join("indice"), &dir.join("Altra")));
        assert_eq!(
            db,
            db_path(
                &dir.join("indice"),
                Path::new(&root.display().to_string().to_uppercase())
            )
        );
    }

    #[test]
    fn le_raccolte_si_creano_si_rinominano_e_si_eliminano_solo_se_vuote() {
        let (root, mut library) = library("operazioni-raccolte");
        library.create_raccolta("Acme").unwrap();
        assert!(root.join("Acme").is_dir());
        assert_eq!(
            library.create_raccolta("acme").unwrap_err(),
            AppError::NameTaken("acme".into())
        );
        for invalid in [
            "",
            "a/b",
            "a:b",
            "CON",
            "nul.txt",
            "COM1",
            "fine.",
            "fine ",
            ".nascosta",
        ] {
            assert_eq!(
                library.create_raccolta(invalid).unwrap_err(),
                AppError::InvalidName(invalid.into()),
                "{invalid:?}"
            );
        }
        bino_at(&root.join("Acme").join("Call.bino"), 1000);
        library.create_raccolta("Beta").unwrap();
        assert_eq!(
            library.rename_raccolta("Acme", "Beta").unwrap_err(),
            AppError::NameTaken("Beta".into())
        );
        assert_eq!(
            library.rename_raccolta("Acme", "Acme Srl").unwrap(),
            (root.join("Acme"), root.join("Acme Srl"))
        );
        // Solo maiuscole e minuscole.
        library.rename_raccolta("Acme Srl", "ACME Srl").unwrap();
        assert_eq!(titoli(&mut library), [entry(Some("ACME Srl"), "Call")]);
        assert_eq!(library.list().unwrap().raccolte, ["ACME Srl", "Beta"]);
        assert_eq!(
            library.delete_raccolta("ACME Srl").unwrap_err(),
            AppError::RaccoltaNotEmpty("ACME Srl".into())
        );
        library.delete_raccolta("Beta").unwrap();
        assert!(!root.join("Beta").exists());
        assert!(matches!(
            library.delete_raccolta("Beta").unwrap_err(),
            AppError::UnreadableFile(_)
        ));
    }

    #[test]
    fn i_bini_si_rinominano_e_si_spostano_senza_sovrascrivere() {
        let (root, mut library) = library("operazioni-bini");
        let call = root.join("Call.bino");
        bino_at(&call, 1000);
        bino_at(&root.join("Altra.bino"), 1000);
        bino_at(&root.join("Acme").join("Altra.bino"), 1000);
        let renamed = library
            .rename_bino(&call, "Ferrara Quarzi, preventivo")
            .unwrap();
        assert_eq!(renamed, root.join("Ferrara Quarzi, preventivo.bino"));
        assert!(!call.exists() && renamed.is_file());
        assert_eq!(
            library.rename_bino(&renamed, "altra").unwrap_err(),
            AppError::NameTaken("altra".into())
        );
        assert_eq!(
            library.rename_bino(&renamed, "a?b").unwrap_err(),
            AppError::InvalidName("a?b".into())
        );
        // Solo maiuscole e minuscole; l'estensione resta quella del file.
        let upper = root.join("Maiuscolo.BINO");
        bino_at(&upper, 1000);
        assert_eq!(
            library.rename_bino(&upper, "MAIUSCOLO").unwrap(),
            root.join("MAIUSCOLO.BINO")
        );
        let moved = library.move_bino(&renamed, Some("Acme")).unwrap();
        assert_eq!(
            moved,
            root.join("Acme").join("Ferrara Quarzi, preventivo.bino")
        );
        assert_eq!(
            library
                .move_bino(&root.join("Acme").join("Altra.bino"), None)
                .unwrap_err(),
            AppError::NameTaken("Altra".into())
        );
        // Una Raccolta nuova si crea spostandoci un Bino.
        // Già lì, anche scritto con altre maiuscole.
        let upper_path = PathBuf::from(moved.to_string_lossy().to_uppercase());
        assert_eq!(
            library.move_bino(&upper_path, Some("Acme")).unwrap(),
            upper_path
        );
        let moved = library.move_bino(&moved, Some("Nuova")).unwrap();
        assert_eq!(
            moved,
            root.join("Nuova").join("Ferrara Quarzi, preventivo.bino")
        );
        assert_eq!(
            library.move_bino(&moved, Some("..")).unwrap_err(),
            AppError::InvalidName("..".into())
        );
        assert!(matches!(
            library
                .move_bino(&root.join("Assente.bino"), None)
                .unwrap_err(),
            AppError::BinoNotFound(_)
        ));
        assert_eq!(
            titoli(&mut library),
            [
                entry(None, "Altra"),
                entry(None, "MAIUSCOLO"),
                entry(Some("Acme"), "Altra"),
                entry(Some("Nuova"), "Ferrara Quarzi, preventivo"),
            ]
        );
    }

    #[test]
    fn un_bino_da_fuori_si_aggiunge_spostandolo_nella_libreria() {
        let (root, mut library) = library("aggiungi");
        let outside = root
            .parent()
            .unwrap()
            .join("Download")
            .join("Ricevuto.bino");
        bino_at(&outside, 1000);
        let added = library.move_bino(&outside, Some("Acme")).unwrap();
        assert_eq!(added, root.join("Acme").join("Ricevuto.bino"));
        assert!(!outside.exists());
        assert_eq!(titoli(&mut library), [entry(Some("Acme"), "Ricevuto")]);
    }

    #[test]
    fn eliminare_un_bino_lo_manda_nel_cestino() {
        let (root, mut library) = library("cestino");
        let path = root.join("Da buttare.bino");
        bino_at(&path, 1000);
        library.sync().unwrap();
        library.trash_bino(&path).unwrap();
        assert!(!path.exists());
        assert_eq!(library.list().unwrap().bini, []);
        assert!(matches!(
            library.trash_bino(&path).unwrap_err(),
            AppError::BinoNotFound(_)
        ));
    }

    /// Un Bino vero in `path` con le Frasi `frasi` (testo e Parlante) e i nomi `parlanti`.
    fn bino_with(path: &Path, frasi: &[(&str, Option<u32>)], parlanti: &[(&str, &str)]) {
        bino_at(path, 1000);
        let mut document = bino::read(path).unwrap();
        document.frasi = frasi
            .iter()
            .zip(0..)
            .map(|(&(testo, parlante), id)| bino::Frase {
                id,
                inizio_ms: id * 1000,
                fine_ms: id * 1000 + 900,
                testo: testo.into(),
                ingresso: Ingresso::Mix,
                parlante,
            })
            .collect();
        document.parlanti = parlanti
            .iter()
            .map(|&(k, v)| (k.into(), v.into()))
            .collect();
        bino::rewrite(path, &document).unwrap();
    }

    /// Una Libreria di prova per la ricerca, già allineata.
    fn searchable(name: &str) -> (PathBuf, Library) {
        let (root, mut library) = library(name);
        bino_with(
            &root.join("Ferrara Quarzi").join("Call di lunedì.bino"),
            &[
                ("Buongiorno a tutti.", Some(1)),
                ("Parliamo del preventivo per l'impianto.", Some(2)),
                ("Il PREVENTIVO arriva entro venerdì.", Some(1)),
                ("Perché la qualità è importante.", Some(2)),
            ],
            &[("mix:2", "Giulia Ferrara")],
        );
        bino_with(
            &root.join("Acme").join("Riunione Acme.bino"),
            &[("Il preventivo di Acme è pronto.", None)],
            &[],
        );
        bino_with(
            &root.join("Sciolto.bino"),
            &[("Nessun preventivo qui, solo saluti.", None)],
            &[],
        );
        library.sync().unwrap();
        (root, library)
    }

    /// I titoli dei Bini trovati, nell'ordine, ognuno con gli id delle Frasi trovate.
    fn found(library: &Library, query: &str, raccolta: Option<&str>) -> Vec<(String, Vec<u32>)> {
        library
            .search(query, raccolta)
            .unwrap()
            .into_iter()
            .map(|r| {
                (
                    r.bino.titolo,
                    r.frasi.into_iter().map(|f| f.phrase_id).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn la_ricerca_ignora_maiuscole_e_accenti_e_trova_i_prefissi() {
        let (_, library) = searchable("ricerca-parole");
        let ferrara = Some("Ferrara Quarzi");
        assert_eq!(
            found(&library, "preventivo", ferrara),
            [("Call di lunedì".to_string(), vec![1, 2])]
        );
        assert_eq!(
            found(&library, "PREV", ferrara),
            found(&library, "preventivo", ferrara)
        );
        assert_eq!(
            found(&library, "qualita", ferrara),
            [("Call di lunedì".to_string(), vec![3])]
        );
        assert_eq!(
            found(&library, "perche", ferrara),
            found(&library, "Perché", ferrara)
        );
        // Più parole: tutte nella stessa Frase.
        assert_eq!(
            found(&library, "preventivo venerdì", ferrara),
            [("Call di lunedì".to_string(), vec![2])]
        );
        assert_eq!(found(&library, "preventivo saluti", ferrara), []);
        assert_eq!(found(&library, "", None), []);
        assert_eq!(found(&library, "   ", None), []);
    }

    #[test]
    fn la_ricerca_trova_i_titoli_e_i_nomi_dei_parlanti() {
        let (root, library) = searchable("ricerca-titoli");
        let mut library = library;
        // Solo il titolo: il Bino senza Frasi.
        assert_eq!(
            found(&library, "lunedi", None),
            [("Call di lunedì".to_string(), vec![])]
        );
        // Il nome del Parlante 2 vale per le sue Frasi.
        assert_eq!(
            found(&library, "giulia", None),
            [("Call di lunedì".to_string(), vec![1, 3])]
        );
        // La rinomina riscrive il Bino: dopo l'allineamento il nome nuovo si trova.
        let path = root.join("Ferrara Quarzi").join("Call di lunedì.bino");
        let mut document = bino::read(&path).unwrap();
        document
            .parlanti
            .insert("mix:1".into(), "Marco Rossi".into());
        bino::rewrite(&path, &document).unwrap();
        library.sync().unwrap();
        assert_eq!(
            found(&library, "rossi", None),
            [("Call di lunedì".to_string(), vec![0, 2])]
        );
        // Un Bino rinominato si trova con il titolo nuovo e non con il vecchio.
        library.rename_bino(&path, "Call di martedì").unwrap();
        assert_eq!(found(&library, "lunedi", None), []);
        assert_eq!(
            found(&library, "martedi", None),
            [("Call di martedì".to_string(), vec![])]
        );
    }

    #[test]
    fn la_ricerca_vale_per_la_raccolta_scelta_o_per_tutta_la_libreria() {
        let (root, library) = searchable("ricerca-ambito");
        let mut all: Vec<_> = found(&library, "preventivo", None)
            .into_iter()
            .map(|(titolo, _)| titolo)
            .collect();
        all.sort();
        assert_eq!(all, ["Call di lunedì", "Riunione Acme", "Sciolto"]);
        assert_eq!(
            found(&library, "preventivo", Some("Acme")),
            [("Riunione Acme".to_string(), vec![0])]
        );
        assert_eq!(
            found(&library, "preventivo", Some("")),
            [("Sciolto".to_string(), vec![0])]
        );
        let results = library.search("preventivo", Some("Acme")).unwrap();
        let acme = &results[0];
        assert_eq!(
            acme.bino.path,
            root.join("Acme")
                .join("Riunione Acme.bino")
                .display()
                .to_string()
        );
        assert_eq!(acme.bino.raccolta.as_deref(), Some("Acme"));
        assert_eq!(acme.frasi[0].ingresso, Ingresso::Mix);
        assert_eq!(acme.frasi[0].inizio_ms, 0);
    }

    #[test]
    fn i_risultati_sono_per_pertinenza_con_l_estratto_segnato() {
        let (_, library) = searchable("ricerca-estratto");
        // Il Bino con più Frasi sulla parola viene prima.
        assert_eq!(found(&library, "preventivo", None)[0].0, "Call di lunedì");
        let results = library.search("prev", Some("Acme")).unwrap();
        assert_eq!(
            results[0].frasi[0].estratto,
            format!("Il {MARK_START}preventivo{MARK_END} di Acme è pronto.")
        );
        // Trovata per il nome del Parlante: il nome, poi il testo.
        let results = library.search("giulia", None).unwrap();
        assert_eq!(
            results[0].frasi[0].estratto,
            format!(
                "{MARK_START}Giulia{MARK_END} Ferrara: Parliamo del preventivo per l'impianto."
            )
        );
    }

    #[test]
    fn i_caratteri_speciali_della_query_non_danno_errori() {
        let (_, library) = searchable("ricerca-speciali");
        for query in [
            "\"",
            "\"preventivo",
            "preventivo*",
            "(preventivo",
            "AND",
            "preventivo OR",
            "NOT preventivo",
            "pre-ventivo",
            "col:preventivo",
            "^preventivo",
            "+ - * : ( ) { } ^ \"",
            "NEAR(preventivo acme)",
            "l'impianto",
        ] {
            assert!(library.search(query, None).is_ok(), "{query:?}");
        }
        assert_eq!(
            found(&library, "\"preventivo", Some("Acme")),
            [("Riunione Acme".to_string(), vec![0])]
        );
        assert_eq!(
            found(&library, "l'impianto", None),
            [("Call di lunedì".to_string(), vec![1])]
        );
    }

    #[test]
    fn un_indice_ricostruito_da_gli_stessi_risultati() {
        let dir = temp_dir("libreria-ricerca-ricostruita");
        let root = dir.join("Sbobino");
        let db = db_path(&dir.join("indice"), &root);
        bino_with(
            &root.join("Acme").join("Uno.bino"),
            &[("Il preventivo è pronto.", None)],
            &[],
        );
        bino_with(
            &root.join("Due.bino"),
            &[("Preventivo rifiutato.", None)],
            &[],
        );
        let searched = |db: &Path| {
            let mut library = Library::open(&root, db).unwrap();
            library.sync().unwrap();
            library.search("preventivo", None).unwrap()
        };
        let before = searched(&db);
        assert_eq!(before.len(), 2);
        std::fs::remove_file(&db).unwrap();
        assert_eq!(searched(&db), before);
        Connection::open(&db)
            .unwrap()
            .pragma_update(None, "user_version", 1)
            .unwrap();
        assert_eq!(searched(&db), before);
    }

    #[test]
    fn la_cartella_di_un_bino_nuovo_e_la_sua_raccolta_o_la_radice() {
        let root = Path::new(r"C:\Sbobino");
        assert_eq!(Library::raccolta_dir(root, None).unwrap(), root);
        assert_eq!(Library::raccolta_dir(root, Some("")).unwrap(), root);
        assert_eq!(
            Library::raccolta_dir(root, Some("Acme")).unwrap(),
            root.join("Acme")
        );
        assert!(Library::raccolta_dir(root, Some(r"..\fuori")).is_err());
    }
}
