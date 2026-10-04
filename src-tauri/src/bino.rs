//! Il Bino: lo zip di una Registrazione con l'audio del mix (`mix.ogg`) e, con gli Ingressi separati,
//! di ogni Ingresso (`microfono.ogg`, `sistema.ogg`), salvati senza ricompressione, e il testo con i
//! suoi metadati (`trascrizione.json`, schema v1). Senza Tauri.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use chrono::{DateTime, Local};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::AppError;
use crate::managers::settings::SpeechLanguage;
use crate::transcript::Ingresso;
use crate::transcript::Phrase;

/// La versione dello schema che questa app scrive e sa leggere.
pub const VERSION: u32 = 1;
const DOCUMENT: &str = "trascrizione.json";

/// La voce dello zip con l'audio dell'Ingresso; è anche la fine del nome del suo Ogg temporaneo.
pub fn audio_entry(ingresso: Ingresso) -> &'static str {
    match ingresso {
        Ingresso::Mix => "mix.ogg",
        Ingresso::Microfono => "microfono.ogg",
        Ingresso::Sistema => "sistema.ogg",
    }
}

/// `trascrizione.json`. In lettura i campi sconosciuti si ignorano.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub version: u32,
    /// Data e ora della Registrazione, ISO 8601 con il fuso.
    pub creato: String,
    pub durata_ms: u32,
    pub modalita: Modalita,
    /// L'id del modello nel catalogo; `null` se il testo non è stato trascritto.
    pub modello: Option<String>,
    pub lingua_parlato: SpeechLanguage,
    /// `false` se la Trascrizione non è arrivata alla fine (spenta, annullata o guasta).
    pub completa: bool,
    /// I nomi dati ai Parlanti, per chiave `<ingresso>:<n>`.
    pub parlanti: BTreeMap<String, String>,
    pub frasi: Vec<Frase>,
    /// Il nome del file audio o video da cui è stato trascritto il Bino; manca in una Registrazione.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origine: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modalita {
    Mix,
    IngressiSeparati,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Frase {
    pub id: u32,
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub testo: String,
    pub ingresso: Ingresso,
    pub parlante: Option<u32>,
}

/// `creato` di una Registrazione iniziata a `start`: `2026-10-03T17:05:42+02:00`.
pub fn creato(start: DateTime<Local>) -> String {
    start.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

impl Document {
    /// La data e l'ora locali di `creato`, ai minuti: `2026-10-03 17:05`.
    pub fn date(&self) -> String {
        self.creato
            .get(..16)
            .unwrap_or(&self.creato)
            .replace('T', " ")
    }

    /// Il documento di una Trascrizione: le Frasi, in ordine di inizio, hanno l'id della loro
    /// posizione.
    pub fn new(
        creato: String,
        durata_ms: u32,
        modalita: Modalita,
        modello: Option<String>,
        lingua_parlato: SpeechLanguage,
        completa: bool,
        phrases: &[Phrase],
    ) -> Self {
        Self {
            version: VERSION,
            creato,
            durata_ms,
            modalita,
            modello,
            lingua_parlato,
            completa,
            parlanti: BTreeMap::new(),
            frasi: phrases
                .iter()
                .zip(0..)
                .map(|(phrase, id)| Frase {
                    id,
                    inizio_ms: phrase.inizio_ms,
                    fine_ms: phrase.fine_ms,
                    testo: phrase.text.clone(),
                    ingresso: phrase.ingresso,
                    parlante: phrase.parlante,
                })
                .collect(),
            origine: None,
        }
    }
}

/// Crea il Bino `path` con gli Ogg di `audio` (il mix e, con gli Ingressi separati, ogni Ingresso) e
/// `document`. Non sovrascrive un file esistente; se non riesce a finirlo, lo cancella.
pub fn write(
    path: &Path,
    audio: &[(Ingresso, &Path)],
    document: &Document,
) -> Result<(), AppError> {
    let file = File::create_new(path).map_err(|e| unwritable(path, &e))?;
    let written = (|| {
        let mut zip = ZipWriter::new(file);
        for &(ingresso, ogg) in audio {
            let mut ogg_file = File::open(ogg).map_err(|e| unreadable(ogg, &e))?;
            zip.start_file(
                audio_entry(ingresso),
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .map_err(|e| unwritable(path, &e))?;
            std::io::copy(&mut ogg_file, &mut zip).map_err(|e| unwritable(path, &e))?;
        }
        write_document(&mut zip, path, document)?;
        zip.finish().map_err(|e| unwritable(path, &e))?;
        Ok(())
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(path);
    }
    written
}

/// Legge `trascrizione.json`. Una `version` più nuova di `VERSION` dà `unsupportedBino`.
pub fn read(path: &Path) -> Result<Document, AppError> {
    #[derive(serde::Deserialize)]
    struct Version {
        version: u32,
    }
    let mut zip = open(path)?;
    let mut json = Vec::new();
    zip.by_name(DOCUMENT)
        .map_err(|e| unreadable(path, &e))?
        .read_to_end(&mut json)
        .map_err(|e| unreadable(path, &e))?;
    let version: Version = serde_json::from_slice(&json).map_err(|e| unreadable(path, &e))?;
    if version.version > VERSION {
        return Err(AppError::UnsupportedBino);
    }
    serde_json::from_slice(&json).map_err(|e| unreadable(path, &e))
}

/// Sostituisce `trascrizione.json` scrivendo un Bino nuovo accanto e rinominandolo sopra il
/// vecchio: a metà, il Bino originale resta intatto. Le altre voci si copiano come sono.
pub fn rewrite(path: &Path, document: &Document) -> Result<(), AppError> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    let written = (|| {
        let mut old = open(path)?;
        let mut zip = ZipWriter::new(File::create(&temp).map_err(|e| unwritable(&temp, &e))?);
        for i in 0..old.len() {
            let entry = old.by_index_raw(i).map_err(|e| unreadable(path, &e))?;
            if entry.name() != DOCUMENT {
                zip.raw_copy_file(entry)
                    .map_err(|e| unwritable(&temp, &e))?;
            }
        }
        write_document(&mut zip, &temp, document)?;
        zip.finish().map_err(|e| unwritable(&temp, &e))?;
        // Il vecchio Bino si chiude prima del rename.
        drop(old);
        std::fs::rename(&temp, path).map_err(|e| unwritable(path, &e))
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

/// Corregge il testo della Frase `id` di `ingresso` e riscrive il Bino: tempi, Parlanti, nomi e audio
/// restano come sono.
pub fn edit_frase(path: &Path, ingresso: Ingresso, id: u32, testo: &str) -> Result<(), AppError> {
    update(path, |document| {
        let frase = document
            .frasi
            .iter_mut()
            .find(|f| f.ingresso == ingresso && f.id == id)
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "{}: Frase {ingresso:?} {id} assente",
                    path.display()
                ))
            })?;
        testo.clone_into(&mut frase.testo);
        Ok(())
    })
}

/// Dà il nome `nome` (senza spazi in testa e in coda, non vuoto) al Parlante `parlante` di `ingresso`,
/// per tutte le sue Frasi, e riscrive il Bino.
pub fn rename_parlante(
    path: &Path,
    ingresso: Ingresso,
    parlante: u32,
    nome: &str,
) -> Result<(), AppError> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Err(AppError::Internal("nome del Parlante vuoto".into()));
    }
    update(path, |document| {
        document
            .parlanti
            .insert(ingresso.parlante_key(parlante), nome.into());
        Ok(())
    })
}

/// Legge, cambia e riscrive il documento. Due modifiche dello stesso Bino non si sovrappongono: altrimenti
/// l'ultima cancellerebbe la prima e scriverebbero lo stesso `.tmp`.
// ponytail: un solo lock per tutti i Bini; una mappa per percorso se le scritture diventano lente.
fn update(
    path: &Path,
    change: impl FnOnce(&mut Document) -> Result<(), AppError>,
) -> Result<(), AppError> {
    static WRITING: Mutex<()> = Mutex::new(());
    let _writing = WRITING.lock().unwrap_or_else(PoisonError::into_inner);
    let mut document = read(path)?;
    change(&mut document)?;
    rewrite(path, &document)
}

/// L'audio di `mix.ogg`, letto direttamente dentro lo zip: la voce non è compressa.
pub struct Mix {
    file: File,
    start: u64,
    len: u64,
    pos: u64,
}

impl Mix {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut zip = open(path)?;
        let name = audio_entry(Ingresso::Mix);
        let entry = zip.by_name(name).map_err(|e| unreadable(path, &e))?;
        let (Some(start), CompressionMethod::Stored) = (entry.data_start(), entry.compression())
        else {
            return Err(AppError::UnreadableFile(format!(
                "{}: {name} compresso",
                path.display()
            )));
        };
        let len = entry.size();
        drop(entry);
        let file = File::open(path).map_err(|e| unreadable(path, &e))?;
        Ok(Self {
            file,
            start,
            len,
            pos: 0,
        })
    }

    pub fn len(&self) -> u64 {
        self.len
    }
}

impl Read for Mix {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let left = usize::try_from(self.len - self.pos).unwrap_or(usize::MAX);
        let max = buf.len().min(left);
        if max == 0 {
            return Ok(0);
        }
        self.file.seek(SeekFrom::Start(self.start + self.pos))?;
        let n = self.file.read(&mut buf[..max])?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for Mix {
    fn seek(&mut self, to: SeekFrom) -> std::io::Result<u64> {
        let pos = match to {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.pos) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.len) + i128::from(n),
        };
        self.pos = u64::try_from(pos.clamp(0, i128::from(self.len)))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        Ok(self.pos)
    }
}

/// Se `path` è un Bino, dall'estensione.
pub fn is_bino(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bino"))
}

/// Il Bino da aprire tra gli argomenti di un avvio (il doppio clic in Esplora file passa il
/// percorso): il primo `.bino` dopo l'eseguibile, rispetto alla cartella di lavoro `cwd`.
pub fn from_args(args: impl IntoIterator<Item = String>, cwd: &Path) -> Option<PathBuf> {
    args.into_iter()
        .skip(1)
        .map(PathBuf::from)
        .find(|path| is_bino(path))
        .map(|path| cwd.join(path))
}

fn open(path: &Path) -> Result<ZipArchive<File>, AppError> {
    let file = File::open(path).map_err(|e| unreadable(path, &e))?;
    ZipArchive::new(file).map_err(|e| unreadable(path, &e))
}

fn write_document(
    zip: &mut ZipWriter<File>,
    path: &Path,
    document: &Document,
) -> Result<(), AppError> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(DOCUMENT, options)
        .map_err(|e| unwritable(path, &e))?;
    serde_json::to_writer_pretty(zip, document).map_err(|e| unwritable(path, &e))
}

fn unreadable(path: &Path, e: &dyn std::fmt::Display) -> AppError {
    AppError::UnreadableFile(format!("{}: {e}", path.display()))
}

fn unwritable(path: &Path, e: &dyn std::fmt::Display) -> AppError {
    AppError::UnwritableFolder(format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{decoded_seconds, sine, temp_dir};

    fn document(frasi: &[&str]) -> Document {
        Document {
            version: VERSION,
            creato: "2026-10-03T17:05:00+02:00".into(),
            durata_ms: 1500,
            modalita: Modalita::Mix,
            modello: Some("nemotron".into()),
            lingua_parlato: SpeechLanguage::It,
            completa: true,
            parlanti: BTreeMap::from([("mix:1".into(), "Mario".into())]),
            frasi: frasi
                .iter()
                .zip(0..)
                .map(|(testo, id)| Frase {
                    id,
                    inizio_ms: id * 600,
                    fine_ms: id * 600 + 500,
                    testo: (*testo).into(),
                    ingresso: Ingresso::Mix,
                    parlante: Some(1),
                })
                .collect(),
            origine: None,
        }
    }

    /// Un Ogg/Opus di 1,5 s.
    fn ogg(dir: &Path) -> PathBuf {
        ogg_named(dir, "mix.ogg", 1.5)
    }

    fn ogg_named(dir: &Path, name: &str, seconds: f64) -> PathBuf {
        let path = dir.join(name);
        let mut writer = OggOpusWriter::new(File::create(&path).unwrap(), 48_000, 2, 64).unwrap();
        writer.write(&sine(48_000, 2, seconds)).unwrap();
        writer.finish().unwrap();
        path
    }

    /// Un Bino con `json` come `trascrizione.json`.
    fn bino_with(path: &Path, json: &str) {
        let mut zip = ZipWriter::new(File::create(path).unwrap());
        zip.start_file(DOCUMENT, SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut zip, json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn si_rilegge_il_documento_scritto_e_l_audio_e_intatto() {
        let dir = temp_dir("bino-scrittura");
        let mix = ogg(&dir);
        let path = dir.join("Registrazione.bino");
        let written = document(&["Buongiorno.", "Iniziamo."]);
        write(&path, &[(Ingresso::Mix, &mix)], &written).unwrap();
        assert_eq!(read(&path).unwrap(), written);
        let json = serde_json::to_value(&written).unwrap();
        assert_eq!(json["parlanti"]["mix:1"], "Mario");
        assert_eq!(json["frasi"][0]["parlante"], 1);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        // Non sovrascrive un Bino esistente.
        let error = write(&path, &[(Ingresso::Mix, &mix)], &written).unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
        assert_eq!(read(&path).unwrap(), written);
    }

    #[test]
    fn con_gli_ingressi_separati_il_bino_ha_l_audio_di_ogni_ingresso() {
        let dir = temp_dir("bino-ingressi-separati");
        let (mix, mic, system) = (
            ogg(&dir),
            ogg_named(&dir, "mic.ogg", 1.0),
            ogg_named(&dir, "sys.ogg", 0.5),
        );
        let path = dir.join("Call.bino");
        let mut written = document(&["Mi senti?", "Sì."]);
        written.modalita = Modalita::IngressiSeparati;
        written.frasi[0].ingresso = Ingresso::Microfono;
        written.frasi[1].ingresso = Ingresso::Sistema;
        write(
            &path,
            &[
                (Ingresso::Mix, &mix),
                (Ingresso::Microfono, &mic),
                (Ingresso::Sistema, &system),
            ],
            &written,
        )
        .unwrap();
        assert_eq!(read(&path).unwrap(), written);
        let json = serde_json::to_value(&written).unwrap();
        assert_eq!(json["modalita"], "ingressi_separati");
        assert_eq!(json["frasi"][0]["ingresso"], "microfono");
        let mut zip = open(&path).unwrap();
        for (name, ogg) in [
            ("mix.ogg", &mix),
            ("microfono.ogg", &mic),
            ("sistema.ogg", &system),
        ] {
            let mut entry = zip.by_name(name).unwrap();
            assert_eq!(entry.compression(), CompressionMethod::Stored, "{name}");
            let mut audio = Vec::new();
            entry.read_to_end(&mut audio).unwrap();
            assert_eq!(audio, std::fs::read(ogg).unwrap(), "{name}");
        }
        drop(zip);
        // Trascrivi legge sempre il mix.
        let seconds = decoded_seconds(&path);
        assert!((seconds - 1.5).abs() < 0.001, "{seconds} s");
    }

    #[test]
    fn creato_e_l_ora_locale_con_il_fuso_e_la_data_la_mostra_ai_minuti() {
        let start = DateTime::parse_from_rfc3339("2026-10-03T17:05:42+02:00")
            .unwrap()
            .with_timezone(&Local);
        let document = Document {
            creato: creato(start),
            ..document(&[])
        };
        assert_eq!(
            DateTime::parse_from_rfc3339(&document.creato).unwrap(),
            start
        );
        assert_eq!(document.date(), start.format("%Y-%m-%d %H:%M").to_string());
    }

    #[test]
    fn il_mix_si_decodifica_come_sorgente() {
        let dir = temp_dir("bino-decodifica");
        let path = dir.join("Registrazione.bino");
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &document(&[])).unwrap();
        let seconds = decoded_seconds(&path);
        assert!((seconds - 1.5).abs() < 0.001, "{seconds} s");
    }

    #[test]
    fn i_campi_sconosciuti_si_ignorano() {
        let dir = temp_dir("bino-campi-sconosciuti");
        let path = dir.join("Futuro.bino");
        let mut json = serde_json::to_value(document(&["Ciao."])).unwrap();
        json["etichette"] = serde_json::json!(["riunione"]);
        json["frasi"][0]["confidenza"] = serde_json::json!(0.9);
        bino_with(&path, &json.to_string());
        assert_eq!(read(&path).unwrap(), document(&["Ciao."]));
    }

    #[test]
    fn una_versione_futura_da_errore_dedicato() {
        let dir = temp_dir("bino-versione-futura");
        let path = dir.join("Futuro.bino");
        // Anche se il resto dello schema è cambiato.
        bino_with(&path, r#"{ "version": 2, "testo": {} }"#);
        assert_eq!(read(&path).unwrap_err(), AppError::UnsupportedBino);
        let error = read(&dir.join("Non esiste.bino")).unwrap_err();
        assert!(matches!(error, AppError::UnreadableFile(_)), "{error:?}");
    }

    #[test]
    fn la_riscrittura_sostituisce_solo_il_testo_senza_lasciare_temporanei() {
        let dir = temp_dir("bino-riscrittura");
        let mix = ogg(&dir);
        let path = dir.join("Registrazione.bino");
        let mut incompleta = document(&["Uno."]);
        incompleta.completa = false;
        write(&path, &[(Ingresso::Mix, &mix)], &incompleta).unwrap();
        assert!(!read(&path).unwrap().completa);
        let nuovo = document(&["Uno.", "Due."]);
        rewrite(&path, &nuovo).unwrap();
        assert_eq!(read(&path).unwrap(), nuovo);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        let mut names: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, ["Registrazione.bino", "mix.ogg"]);
    }

    #[test]
    fn una_riscrittura_fallita_lascia_il_bino_com_era() {
        let dir = temp_dir("bino-riscrittura-fallita");
        let path = dir.join("Registrazione.bino");
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &document(&["Uno."])).unwrap();
        let before = std::fs::read(&path).unwrap();
        // Il Bino aperto in esclusiva da un altro programma: il rename non riesce.
        let lock = {
            use std::os::windows::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(1) // FILE_SHARE_READ
                .open(&path)
                .unwrap()
        };
        assert!(rewrite(&path, &document(&["Due."])).is_err());
        drop(lock);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!dir.join("Registrazione.bino.tmp").exists());
    }

    #[test]
    fn origine_si_rilegge_e_un_bino_senza_origine_si_legge_come_prima() {
        let dir = temp_dir("bino-origine");
        let path = dir.join("Call.bino");
        let written = Document {
            origine: Some("Call Teams.mp4".into()),
            ..document(&["Ciao."])
        };
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &written).unwrap();
        assert_eq!(read(&path).unwrap(), written);
        // Senza origine il campo non si scrive, e un Bino di prima si legge con `None`.
        let json = serde_json::to_value(document(&["Ciao."])).unwrap();
        assert!(json.get("origine").is_none(), "{json}");
        let old = dir.join("Vecchio.bino");
        bino_with(&old, &json.to_string());
        assert_eq!(read(&old).unwrap().origine, None);
    }

    #[test]
    fn la_correzione_cambia_solo_il_testo_di_quella_frase() {
        let dir = temp_dir("bino-correzione");
        let mix = ogg(&dir);
        let path = dir.join("Call.bino");
        let mut before = document(&["Buongiorno.", "Inizziamo.", "Bene."]);
        before.origine = Some("Call.mp4".into());
        before.frasi[2].ingresso = Ingresso::Sistema;
        before.frasi[2].id = 1;
        write(&path, &[(Ingresso::Mix, &mix)], &before).unwrap();
        edit_frase(&path, Ingresso::Mix, 1, "Iniziamo.").unwrap();
        let mut expected = before.clone();
        expected.frasi[1].testo = "Iniziamo.".into();
        assert_eq!(read(&path).unwrap(), expected);
        // Lo stesso id in un altro Ingresso è un'altra Frase; una Frase svuotata resta.
        edit_frase(&path, Ingresso::Sistema, 1, "").unwrap();
        expected.frasi[2].testo = String::new();
        assert_eq!(read(&path).unwrap(), expected);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        // Una Frase che non c'è non cambia nulla.
        assert!(edit_frase(&path, Ingresso::Microfono, 0, "x").is_err());
        assert_eq!(read(&path).unwrap(), expected);
    }

    #[test]
    fn correzioni_contemporanee_dello_stesso_bino_restano_tutte() {
        let dir = temp_dir("bino-correzioni-contemporanee");
        let path = dir.join("Call.bino");
        let testi: Vec<String> = (0..8).map(|i| format!("Frase {i}.")).collect();
        let refs: Vec<&str> = testi.iter().map(String::as_str).collect();
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &document(&refs)).unwrap();
        std::thread::scope(|scope| {
            for id in 0..8 {
                let path = &path;
                scope.spawn(move || edit_frase(path, Ingresso::Mix, id, "corretta").unwrap());
            }
            scope.spawn(|| rename_parlante(&path, Ingresso::Mix, 2, "Lucia").unwrap());
        });
        let after = read(&path).unwrap();
        assert!(
            after.frasi.iter().all(|f| f.testo == "corretta"),
            "{after:?}"
        );
        assert_eq!(
            after.parlanti.get("mix:2").map(String::as_str),
            Some("Lucia")
        );
    }

    #[test]
    fn la_rinomina_di_un_parlante_si_salva_nel_bino() {
        let dir = temp_dir("bino-rinomina");
        let path = dir.join("Call.bino");
        let before = document(&["Ciao.", "Salve."]);
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &before).unwrap();
        rename_parlante(&path, Ingresso::Sistema, 2, "  Lucia ").unwrap();
        // Un nome vuoto si rifiuta e non cambia nulla.
        assert!(rename_parlante(&path, Ingresso::Mix, 1, "  ").is_err());
        let after = read(&path).unwrap();
        assert_eq!(
            after.parlanti,
            BTreeMap::from([
                ("mix:1".to_string(), "Mario".to_string()),
                ("sistema:2".to_string(), "Lucia".to_string()),
            ])
        );
        assert_eq!(after.frasi, before.frasi);
    }

    #[test]
    fn dagli_argomenti_di_avvio_il_primo_bino_rispetto_alla_cartella_di_lavoro() {
        let args = |a: &[&str]| a.iter().map(ToString::to_string).collect::<Vec<_>>();
        let cwd = Path::new(r"C:\Lavoro");
        assert_eq!(
            from_args(
                args(&[r"C:\Sbobino\sbobino.exe", r"D:\Note\Lezione.BINO"]),
                cwd
            ),
            Some(PathBuf::from(r"D:\Note\Lezione.BINO"))
        );
        assert_eq!(
            from_args(args(&["sbobino.exe", "--flag", "Lezione.bino"]), cwd),
            Some(PathBuf::from(r"C:\Lavoro\Lezione.bino"))
        );
        // L'eseguibile non conta, nemmeno se si chiamasse `.bino`.
        assert_eq!(from_args(args(&["x.bino"]), cwd), None);
        assert_eq!(from_args(args(&["sbobino.exe", "audio.mp3"]), cwd), None);
    }
}
