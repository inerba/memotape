//! Il Bino: lo zip di una Registrazione con l'audio del mix (`mix.ogg`, salvato senza
//! ricompressione) e il testo con i suoi metadati (`trascrizione.json`, schema v1). Senza Tauri.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::AppError;
use crate::managers::settings::SpeechLanguage;
use crate::transcript::Phrase;

/// La versione dello schema che questa app scrive e sa leggere.
pub const VERSION: u32 = 1;
const MIX: &str = "mix.ogg";
const DOCUMENT: &str = "trascrizione.json";

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
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modalita {
    Mix,
    IngressiSeparati,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ingresso {
    Mix,
    Microfono,
    Sistema,
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

    /// Il documento di una Trascrizione del mix: le Frasi hanno l'id della loro posizione.
    pub fn of_mix(
        creato: String,
        durata_ms: u32,
        modello: Option<String>,
        lingua_parlato: SpeechLanguage,
        completa: bool,
        phrases: &[Phrase],
    ) -> Self {
        Self {
            version: VERSION,
            creato,
            durata_ms,
            modalita: Modalita::Mix,
            modello,
            lingua_parlato,
            completa,
            parlanti: BTreeMap::new(),
            // ponytail: Ingresso e Parlante arrivano con gli Ingressi separati e la Diarizzazione.
            frasi: phrases
                .iter()
                .zip(0..)
                .map(|(phrase, id)| Frase {
                    id,
                    inizio_ms: phrase.inizio_ms,
                    fine_ms: phrase.fine_ms,
                    testo: phrase.text.clone(),
                    ingresso: Ingresso::Mix,
                    parlante: None,
                })
                .collect(),
        }
    }
}

/// Crea il Bino `path` con l'Ogg `mix` e `document`. Non sovrascrive un file esistente; se non
/// riesce a finirlo, lo cancella.
pub fn write(path: &Path, mix: &Path, document: &Document) -> Result<(), AppError> {
    let file = File::create_new(path).map_err(|e| unwritable(path, &e))?;
    let written = (|| {
        let mut zip = ZipWriter::new(file);
        let mut audio = File::open(mix).map_err(|e| unreadable(mix, &e))?;
        zip.start_file(
            MIX,
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .map_err(|e| unwritable(path, &e))?;
        std::io::copy(&mut audio, &mut zip).map_err(|e| unwritable(path, &e))?;
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
        let entry = zip.by_name(MIX).map_err(|e| unreadable(path, &e))?;
        let (Some(start), CompressionMethod::Stored) = (entry.data_start(), entry.compression())
        else {
            return Err(AppError::UnreadableFile(format!(
                "{}: {MIX} compresso",
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
            parlanti: BTreeMap::new(),
            frasi: frasi
                .iter()
                .zip(0..)
                .map(|(testo, id)| Frase {
                    id,
                    inizio_ms: id * 600,
                    fine_ms: id * 600 + 500,
                    testo: (*testo).into(),
                    ingresso: Ingresso::Mix,
                    parlante: None,
                })
                .collect(),
        }
    }

    /// Un Ogg/Opus di 1,5 s.
    fn ogg(dir: &Path) -> PathBuf {
        let path = dir.join("mix.ogg");
        let mut writer = OggOpusWriter::new(File::create(&path).unwrap(), 48_000, 2, 64).unwrap();
        writer.write(&sine(48_000, 2, 1.5)).unwrap();
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
        write(&path, &mix, &written).unwrap();
        assert_eq!(read(&path).unwrap(), written);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        // Non sovrascrive un Bino esistente.
        let error = write(&path, &mix, &written).unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
        assert_eq!(read(&path).unwrap(), written);
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
        write(&path, &ogg(&dir), &document(&[])).unwrap();
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
        write(&path, &mix, &incompleta).unwrap();
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
        write(&path, &ogg(&dir), &document(&["Uno."])).unwrap();
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
}
