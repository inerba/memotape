//! Il Tape: lo zip di una Registrazione con l'audio del mix (`mix.ogg`) e, con gli Ingressi separati,
//! di ogni Ingresso (`microfono.ogg`, `sistema.ogg`), salvati senza ricompressione, e il testo con i
//! suoi metadati (`trascrizione.json`, schema v1), e la Forma d'onda del mix (`forma-onda.json`,
//! ADR-0010). Senza Tauri.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

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
/// La Forma d'onda del mix: facoltativa, un Tape scritto prima di lei non ce l'ha.
const FORMA_ONDA: &str = "forma-onda.json";

pub(crate) fn temporary_id() -> u64 {
    use std::hash::{BuildHasher, RandomState};
    RandomState::new().hash_one(std::time::SystemTime::now())
}

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
    /// Il nome del file audio o video da cui è stato trascritto il Tape; manca in una Registrazione.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origine: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diarizzazione: Option<crate::transcript::Diarizzazione>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pulizia_audio: Vec<crate::audio_toolkit::cleaning::TrattoPulizia>,
    /// Testo corretto come unità, collegato alle Frasi senza inventare nuovi tempi delle parole.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub correzioni_testo: Vec<TestoTurno>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TestoTurno {
    pub ingresso: Ingresso,
    pub frasi: Vec<u32>,
    pub testo: String,
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
    /// Assente nei Tape precedenti: conserva la distinzione dall'Ingresso non diarizzato.
    #[serde(default, skip_serializing_if = "is_false")]
    pub parlante_non_determinato: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub parlante_provvisorio: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tempi: Vec<crate::transcript::TempoTesto>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub testo_corretto: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub parlante_corretto: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

/// `creato` di una Registrazione iniziata a `start`: `2026-10-03T17:05:42+02:00`.
pub fn creato(start: DateTime<Local>) -> String {
    start.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

impl Document {
    /// Il nome del Microfono persona sola, se c'è (ADR-0027).
    pub fn set_nome_microfono(&mut self, nome: Option<&str>) {
        if let Some(nome) = nome {
            self.parlanti
                .insert(Ingresso::Microfono.parlante_key(None), nome.to_owned());
        }
    }

    /// Se il testo, un'attribuzione o il nome di un Parlante sono stati corretti a mano. Il nome del
    /// Microfono persona sola non conta: non è un Parlante, può venire dal nome predefinito
    /// (ADR-0027) e nessuna nuova analisi lo cambia (ADR-0017).
    pub fn corretto_a_mano(&self) -> bool {
        self.parlanti
            .keys()
            .any(|key| *key != Ingresso::Microfono.parlante_key(None))
            || !self.correzioni_testo.is_empty()
            || self
                .frasi
                .iter()
                .any(|f| f.testo_corretto || f.parlante_corretto)
    }

    pub fn correzione_testo(&self, frase: &Frase) -> Option<&TestoTurno> {
        self.correzioni_testo
            .iter()
            .find(|c| c.ingresso == frase.ingresso && c.frasi.contains(&frase.id))
    }

    /// Proiezione comune a vista, ricerca e copia. Ogni riferimento audio resta presente;
    /// solo la prima Frase porta il testo della correzione. Voci discordanti non allineano parole.
    pub fn frasi_visibili(&self) -> Vec<Frase> {
        let mut frasi = self.frasi.clone();
        for correzione in &self.correzioni_testo {
            let members: Vec<_> = self
                .frasi
                .iter()
                .filter(|f| f.ingresso == correzione.ingresso && correzione.frasi.contains(&f.id))
                .collect();
            let Some(first) = members.first() else {
                continue;
            };
            let same_voice = members.iter().all(|f| same_voice(first, f));
            for frase in frasi
                .iter_mut()
                .filter(|f| f.ingresso == correzione.ingresso && correzione.frasi.contains(&f.id))
            {
                frase.testo = if correzione.frasi.first() == Some(&frase.id) {
                    correzione.testo.clone()
                } else {
                    String::new()
                };
                if !same_voice {
                    frase.parlante = None;
                    frase.parlante_non_determinato = true;
                    frase.parlante_provvisorio = false;
                }
            }
        }
        frasi
    }

    /// Turni della proiezione, con le stesse pause e identità della vista.
    pub fn turni(&self) -> Vec<Vec<usize>> {
        let frasi = self.frasi_visibili();
        let labeled = frasi.iter().any(|f| {
            f.ingresso != Ingresso::Mix
                || f.parlante.is_some()
                || f.parlante_non_determinato
                || self.parlanti.contains_key("mix")
        });
        let mut ordered: Vec<_> = (0..frasi.len()).collect();
        ordered.sort_by_key(|&i| frasi[i].inizio_ms);
        let mut turni: Vec<Vec<usize>> = Vec::new();
        for index in ordered {
            if let Some(last) = turni.last_mut() {
                let previous = &frasi[*last.last().expect("Turno non vuoto")];
                let current = &frasi[index];
                let gap = current.inizio_ms.saturating_sub(previous.fine_ms);
                let silence = if gap == 0 {
                    0
                } else {
                    gap.saturating_add(1000)
                };
                if same_voice(previous, current) && (labeled || silence <= 2000) {
                    last.push(index);
                    continue;
                }
            }
            turni.push(vec![index]);
        }
        turni
    }

    pub fn testo_del_turno(&self, indices: &[usize], separator: &str) -> String {
        indices
            .iter()
            .filter_map(|&i| {
                let frase = &self.frasi[i];
                match self.correzione_testo(frase) {
                    Some(c) if c.frasi.first() == Some(&frase.id) => Some(c.testo.as_str()),
                    Some(_) => None,
                    None => Some(frase.testo.as_str()),
                }
            })
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// Un'unità testuale per Turno, usata solo nelle uscite del documento, senza modificare gli id.
    pub fn frasi_per_lettura(&self) -> Vec<Frase> {
        let visible = self.frasi_visibili();
        self.turni()
            .into_iter()
            .flat_map(|indices| {
                if !indices
                    .iter()
                    .any(|&i| self.correzione_testo(&self.frasi[i]).is_some())
                {
                    return indices
                        .into_iter()
                        .map(|i| visible[i].clone())
                        .collect::<Vec<_>>();
                }
                let mut first = visible[indices[0]].clone();
                first.testo = self.testo_del_turno(&indices, "\n");
                first.fine_ms = indices
                    .iter()
                    .map(|&i| visible[i].fine_ms)
                    .max()
                    .unwrap_or(first.fine_ms);
                vec![first]
            })
            .filter(|f| !f.testo.is_empty())
            .collect()
    }

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
            diarizzazione: None,
            pulizia_audio: Vec::new(),
            correzioni_testo: Vec::new(),
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
                    parlante_non_determinato: phrase.parlante_non_determinato,
                    parlante_provvisorio: phrase.parlante_provvisorio,
                    tempi: phrase.tempi.clone(),
                    testo_corretto: false,
                    parlante_corretto: false,
                })
                .collect(),
            origine: None,
        }
    }
}

fn same_voice(a: &Frase, b: &Frase) -> bool {
    a.ingresso == b.ingresso
        && (a.parlante_non_determinato || a.parlante == b.parlante)
        && a.parlante_non_determinato == b.parlante_non_determinato
        && a.parlante_provvisorio == b.parlante_provvisorio
}

/// Crea il Tape `path` con gli Ogg di `audio` (il mix e, con gli Ingressi separati, ogni Ingresso),
/// `document` e, se c'è, la Forma d'onda del mix. Non sovrascrive un file esistente; se non riesce
/// a finirlo, lo cancella.
pub fn write(
    path: &Path,
    audio: &[(Ingresso, &Path)],
    document: &Document,
    forma_onda: Option<&[f32]>,
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
        if let Some(values) = forma_onda {
            write_forma_onda(&mut zip, path, values)?;
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

/// Legge `trascrizione.json`. Una `version` più nuova di `VERSION` dà `unsupportedTape`.
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
        return Err(AppError::UnsupportedTape);
    }
    serde_json::from_slice(&json).map_err(|e| unreadable(path, &e))
}

/// La Forma d'onda del mix salvata nel Tape; `None` se manca, non si legge o è vuota.
pub fn forma_onda(path: &Path) -> Option<Vec<f32>> {
    let mut json = Vec::new();
    open(path)
        .ok()?
        .by_name(FORMA_ONDA)
        .ok()?
        .read_to_end(&mut json)
        .ok()?;
    serde_json::from_slice::<Vec<f32>>(&json)
        .ok()
        .filter(|values| !values.is_empty())
}

/// Aggiunge al Tape la Forma d'onda `values`, se non ce l'ha già, riscrivendolo come `rewrite`.
#[cfg(test)]
pub fn save_forma_onda(path: &Path, values: &[f32]) -> Result<(), AppError> {
    let _writing = writing();
    if forma_onda(path).is_some() {
        return Ok(());
    }
    replace(path, FORMA_ONDA, |zip, temp| {
        write_forma_onda(zip, temp, values)
    })
}

/// Sostituisce `trascrizione.json` scrivendo un Tape nuovo accanto e rinominandolo sopra il
/// vecchio: a metà, il Tape originale resta intatto. Le altre voci si copiano come sono.
pub fn rewrite(path: &Path, document: &Document) -> Result<(), AppError> {
    replace(path, DOCUMENT, |zip, temp| {
        write_document(zip, temp, document)
    })
}

/// Come `rewrite`, ma Annulla durante la copia dello zip impedisce la sostituzione finale.
pub fn rewrite_cancellable(
    path: &Path,
    document: &Document,
    cancel: &transcribe_cpp::CancelToken,
) -> Result<(), AppError> {
    let _writing = writing();
    replace_checked(
        path,
        DOCUMENT,
        |zip, temp| write_document(zip, temp, document),
        || {
            if cancel.is_cancelled() {
                Err(AppError::Cancelled)
            } else {
                Ok(())
            }
        },
    )
}

/// Trascrivi su Tape (ADR-0029): sostituisce il documento conservando i campi JSON sconosciuti.
/// Le altre voci (audio, Forma d'onda, dati aggiuntivi) restano byte-identiche.
pub fn replace_transcription(
    path: &Path,
    document: &Document,
    cancel: &transcribe_cpp::CancelToken,
) -> Result<(), AppError> {
    let _writing = writing();
    let check = || {
        if cancel.is_cancelled() {
            Err(AppError::Cancelled)
        } else {
            Ok(())
        }
    };
    check()?;
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{}.tmp", temporary_id()));
    let temp = PathBuf::from(name);
    let mut created = false;
    let written = (|| {
        let mut old = open(path)?;
        // Conserva i campi JSON sconosciuti senza ripristinare dati testuali rimossi dalla nuova ASR.
        let mut json: serde_json::Value =
            serde_json::from_reader(old.by_name(DOCUMENT).map_err(|e| unreadable(path, &e))?)
                .map_err(|e| unreadable(path, &e))?;
        let fields = json
            .as_object_mut()
            .ok_or_else(|| unreadable(path, &"documento non valido"))?;
        for key in [
            "version",
            "creato",
            "durata_ms",
            "modalita",
            "modello",
            "lingua_parlato",
            "completa",
            "parlanti",
            "frasi",
            "origine",
            "diarizzazione",
            "pulizia_audio",
            "correzioni_testo",
        ] {
            fields.remove(key);
        }
        let new = serde_json::to_value(document).map_err(|e| unwritable(&temp, &e))?;
        fields.extend(new.as_object().unwrap().clone());
        let file = File::create_new(&temp).map_err(|e| unwritable(&temp, &e))?;
        created = true;
        let mut zip = ZipWriter::new(file);
        for i in 0..old.len() {
            check()?;
            let entry = old.by_index_raw(i).map_err(|e| unreadable(path, &e))?;
            if entry.name() != DOCUMENT {
                zip.raw_copy_file(entry)
                    .map_err(|e| unwritable(&temp, &e))?;
            }
        }
        zip.start_file(
            DOCUMENT,
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .map_err(|e| unwritable(&temp, &e))?;
        serde_json::to_writer_pretty(&mut zip, &json).map_err(|e| unwritable(&temp, &e))?;
        let file = zip.finish().map_err(|e| unwritable(&temp, &e))?;
        file.sync_all().map_err(|e| unwritable(&temp, &e))?;
        drop(file);
        drop(old);
        check()?;
        std::fs::rename(&temp, path).map_err(|e| unwritable(path, &e))
    })();
    if written.is_err() && created {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

/// Riscrive il Tape con la voce `name` scritta da `add` al posto di quella che c'era, se c'era.
fn replace(
    path: &Path,
    name: &str,
    add: impl FnOnce(&mut ZipWriter<File>, &Path) -> Result<(), AppError>,
) -> Result<(), AppError> {
    replace_checked(path, name, add, || Ok(()))
}

fn replace_checked(
    path: &Path,
    name: &str,
    add: impl FnOnce(&mut ZipWriter<File>, &Path) -> Result<(), AppError>,
    before_replace: impl FnOnce() -> Result<(), AppError>,
) -> Result<(), AppError> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    let written = (|| {
        let mut old = open(path)?;
        let mut zip = ZipWriter::new(File::create(&temp).map_err(|e| unwritable(&temp, &e))?);
        for i in 0..old.len() {
            let entry = old.by_index_raw(i).map_err(|e| unreadable(path, &e))?;
            if entry.name() != name {
                zip.raw_copy_file(entry)
                    .map_err(|e| unwritable(&temp, &e))?;
            }
        }
        add(&mut zip, &temp)?;
        zip.finish().map_err(|e| unwritable(&temp, &e))?;
        // Il vecchio Tape si chiude prima del rename.
        drop(old);
        before_replace()?;
        std::fs::rename(&temp, path).map_err(|e| unwritable(path, &e))
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

/// Corregge il testo della Frase `id` di `ingresso` e riscrive il Tape: tempi, Parlanti, nomi e audio
/// restano come sono.
pub fn edit_frase(path: &Path, ingresso: Ingresso, id: u32, testo: &str) -> Result<(), AppError> {
    update(path, |document| {
        if document
            .correzioni_testo
            .iter()
            .any(|c| c.ingresso == ingresso && c.frasi.contains(&id))
        {
            return Err(AppError::Internal(
                "questa Frase appartiene a una correzione del Turno".into(),
            ));
        }
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
        if frase.testo != testo {
            testo.clone_into(&mut frase.testo);
            // I tempi ASR si riferiscono al testo originale, non alla correzione dell'utente.
            frase.tempi.clear();
            frase.testo_corretto = true;
        }
        Ok(())
    })
}

/// Corregge un Turno completo; il testo originale atteso impedisce di sovrascrivere una bozza obsoleta.
pub fn edit_turno(
    path: &Path,
    ingresso: Ingresso,
    ids: &[u32],
    originale: &str,
    testo: &str,
) -> Result<(), AppError> {
    let _writing = writing();
    let mut document = read(path)?;
    let indices = document
        .turni()
        .into_iter()
        .find(|indices| {
            indices.len() == ids.len()
                && indices.iter().zip(ids).all(|(&i, &id)| {
                    document.frasi[i].ingresso == ingresso && document.frasi[i].id == id
                })
        })
        .ok_or_else(|| AppError::Internal("il Turno non è più disponibile".into()))?;
    if document.testo_del_turno(&indices, "\n") != originale {
        return Err(AppError::Internal(
            "il testo del Turno è cambiato: riaprire il Tape prima di correggerlo".into(),
        ));
    }
    if originale == testo {
        return Ok(());
    }
    document
        .correzioni_testo
        .retain(|c| !(c.ingresso == ingresso && c.frasi.iter().any(|id| ids.contains(id))));
    for &i in &indices {
        document.frasi[i].tempi.clear();
        document.frasi[i].testo_corretto = true;
    }
    document.correzioni_testo.push(TestoTurno {
        ingresso,
        frasi: ids.to_vec(),
        testo: testo.to_string(),
    });
    rewrite(path, &document)
}

/// Attribuisce un Turno intero al Parlante del Turno immediatamente adiacente.
/// I riferimenti, il testo e i tempi restano invariati; richieste obsolete non scrivono nulla.
pub fn unisci_turno(
    path: &Path,
    ingresso: Ingresso,
    ids: &[u32],
    destinazione: u32,
) -> Result<(), AppError> {
    update(path, |document| {
        let invalid =
            || AppError::Internal("il Turno o la destinazione non sono più disponibili".into());
        let visible = document.frasi_visibili();
        let mut ordered: Vec<_> = visible.iter().enumerate().collect();
        ordered.sort_by_key(|(_, f)| f.inizio_ms);
        let same = same_voice;
        let positions: Vec<_> = ordered
            .iter()
            .enumerate()
            .filter(|(_, (_, f))| f.ingresso == ingresso && ids.contains(&f.id))
            .map(|(position, _)| position)
            .collect();
        let (&start, &end) = positions
            .first()
            .zip(positions.last())
            .ok_or_else(invalid)?;
        let source = ordered[start].1;
        if positions.len() != ids.len()
            || end - start + 1 != ids.len()
            || ordered[start..=end].iter().any(|(_, f)| !same(source, f))
            || (start > 0 && same(source, ordered[start - 1].1))
            || (end + 1 < ordered.len() && same(source, ordered[end + 1].1))
        {
            return Err(invalid());
        }
        let target = ordered
            .iter()
            .position(|(_, f)| f.ingresso == ingresso && f.id == destinazione)
            .ok_or_else(invalid)?;
        let boundary = if target < start {
            start - 1
        } else if target > end && end + 1 < ordered.len() {
            end + 1
        } else {
            return Err(invalid());
        };
        let voice = ordered[target].1;
        if voice.parlante_non_determinato
            || voice.parlante_provvisorio
            || (voice.parlante.is_none()
                && ingresso == Ingresso::Mix
                && !document.parlanti.contains_key("mix"))
            || ordered[target.min(boundary)..=target.max(boundary)]
                .iter()
                .any(|(_, f)| !same(voice, f))
        {
            return Err(invalid());
        }
        let speaker = voice.parlante;
        let selected: Vec<_> = ordered[start..=end]
            .iter()
            .map(|(index, _)| *index)
            .collect();
        for index in selected {
            let frase = &mut document.frasi[index];
            frase.parlante = speaker;
            frase.parlante_non_determinato = false;
            frase.parlante_provvisorio = false;
            frase.parlante_corretto = true;
        }
        Ok(())
    })
}

/// Dà il nome `nome` (senza spazi in testa e in coda, non vuoto) al Parlante `parlante` di `ingresso`
/// (con `None` all'Ingresso, per le sue Frasi senza Parlante), per tutte le sue Frasi, e riscrive il
/// Tape.
pub fn rename_parlante(
    path: &Path,
    ingresso: Ingresso,
    parlante: Option<u32>,
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

/// Cambia `creato` con la data e l'ora locali `local` (`2026-10-03T17:05`, come
/// `<input type="datetime-local">`) e riscrive il Tape. Restituisce il `creato` scritto.
pub fn set_creato(path: &Path, local: &str) -> Result<String, AppError> {
    let scritto = chrono::NaiveDateTime::parse_from_str(local, "%Y-%m-%dT%H:%M")
        .ok()
        // Un'ora che non esiste (cambio dell'ora legale) si rifiuta.
        .and_then(|naive| naive.and_local_timezone(Local).earliest())
        .map(creato)
        .ok_or_else(|| AppError::Internal(format!("data e ora non valide: {local}")))?;
    update(path, |document| {
        document.creato.clone_from(&scritto);
        Ok(())
    })?;
    Ok(scritto)
}

/// Legge, cambia e riscrive il documento.
fn update(
    path: &Path,
    change: impl FnOnce(&mut Document) -> Result<(), AppError>,
) -> Result<(), AppError> {
    let _writing = writing();
    let mut document = read(path)?;
    change(&mut document)?;
    rewrite(path, &document)
}

/// Il lock delle riscritture di un Tape. Due riscritture dello stesso Tape non si sovrappongono:
/// altrimenti l'ultima cancellerebbe la prima e scriverebbero lo stesso `.tmp`.
// ponytail: un solo lock per tutti i Tape; una mappa per percorso se le scritture diventano lente.
fn writing() -> MutexGuard<'static, ()> {
    static WRITING: Mutex<()> = Mutex::new(());
    WRITING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Se il Tape `path` ha l'audio di ogni Ingresso: allora si trascrive per Ingresso (ADR-0015).
pub fn has_ingressi(path: &Path) -> Result<bool, AppError> {
    let zip = open(path)?;
    let names: Vec<_> = zip.file_names().collect();
    Ok([Ingresso::Microfono, Ingresso::Sistema]
        .into_iter()
        .all(|ingresso| names.contains(&audio_entry(ingresso))))
}

/// L'audio di `mix.ogg` (o di un Ingresso), letto direttamente dentro lo zip: la voce non è
/// compressa.
pub struct Mix {
    file: File,
    start: u64,
    len: u64,
    pos: u64,
}

impl Mix {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        Self::open_ingresso(path, Ingresso::Mix)
    }

    /// L'audio di `ingresso`: `microfono.ogg` e `sistema.ogg` ci sono con gli Ingressi separati.
    pub fn open_ingresso(path: &Path, ingresso: Ingresso) -> Result<Self, AppError> {
        let mut zip = open(path)?;
        let name = audio_entry(ingresso);
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

/// Se `path` è un Tape, dall'estensione.
pub fn is_tape(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("tape"))
}

/// Il Tape da aprire tra gli argomenti di un avvio (il doppio clic in Esplora file passa il
/// percorso): il primo `.tape` dopo l'eseguibile, rispetto alla cartella di lavoro `cwd`.
pub fn from_args(args: impl IntoIterator<Item = String>, cwd: &Path) -> Option<PathBuf> {
    args.into_iter()
        .skip(1)
        .map(PathBuf::from)
        .find(|path| is_tape(path))
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

fn write_forma_onda(
    zip: &mut ZipWriter<File>,
    path: &Path,
    values: &[f32],
) -> Result<(), AppError> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(FORMA_ONDA, options)
        .map_err(|e| unwritable(path, &e))?;
    serde_json::to_writer(zip, values).map_err(|e| unwritable(path, &e))
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

    #[test]
    fn il_nome_predefinito_del_microfono_non_e_una_correzione_manuale() {
        let mut doc = document(&["Ciao."]);
        doc.parlanti.clear();
        doc.set_nome_microfono(None);
        assert!(doc.parlanti.is_empty());
        doc.set_nome_microfono(Some("Francesco"));
        assert_eq!(doc.parlanti["microfono"], "Francesco");
        assert!(!doc.corretto_a_mano());
        // Il nome di un Parlante, invece, è una Correzione manuale.
        doc.parlanti.insert("sistema:1".into(), "Lucia".into());
        assert!(doc.corretto_a_mano());
    }
    #[test]
    fn un_tape_vecchio_con_solo_mix_e_metadati_senza_ingressi_resta_non_separabile() {
        let mut doc = document(&["Testo del mix."]);
        doc.diarizzazione =
            Some(serde_json::from_str(r#"{"modello":"nemotron3","esito":"completata"}"#).unwrap());
        assert!(doc.diarizzazione.as_ref().unwrap().ingressi.is_empty());
        let dir = crate::audio_toolkit::ogg_opus::tests::temp_dir("vecchio-mix");
        let path = dir.join("Mix.tape");
        let audio = ogg(&dir);
        write(&path, &[(Ingresso::Mix, &audio)], &doc, None).unwrap();
        assert!(!has_ingressi(&path).unwrap());
        let opened = crate::managers::transcription::open_tape(&path).unwrap();
        assert!(opened.phrases.iter().all(|p| p.ingresso == Ingresso::Mix));
        assert!(!opened.info.ingressi_separati);
    }

    #[test]
    fn diarizzazione_annullata_conserva_testo_completo_e_attribuzioni_provvisorie() {
        let mut doc = document(&["Testo già finito."]);
        doc.diarizzazione = Some(crate::transcript::Diarizzazione {
            modello: crate::managers::settings::Diarizer::Nemotron3,
            esito: crate::transcript::EsitoDiarizzazione::Annullata,
            ingressi: Vec::new(),
        });
        doc.frasi[0].parlante_provvisorio = true;
        let dir = crate::audio_toolkit::ogg_opus::tests::temp_dir("diarizzazione-annullata");
        let path = dir.join("Conservato.tape");
        let audio = ogg(&dir);
        write(&path, &[(Ingresso::Mix, &audio)], &doc, Some(&[0.5])).unwrap();
        let reopened = read(&path).unwrap();
        assert_eq!(reopened, doc);
        assert!(reopened.completa);
        assert!(reopened.frasi[0].parlante_provvisorio);
        assert_eq!(forma_onda(&path), Some(vec![0.5]));
        let opened = crate::managers::transcription::open_tape(&path).unwrap();
        assert_eq!(opened.info.diarizzazione, doc.diarizzazione);
        assert!(opened.phrases[0].parlante_provvisorio);
        let text = crate::managers::transcription::tape_text(
            &path,
            &crate::managers::settings::Settings::default(),
            crate::managers::settings::CopiaCome::Markdown,
        )
        .unwrap();
        assert!(text.contains("Diarizzazione non completata"));
        assert!(text.contains("provvisorio"));
        assert!(text.contains("Testo già finito."));
        assert_eq!(std::fs::read(&audio).unwrap(), {
            let mut mix = Mix::open(&path).unwrap();
            let mut bytes = Vec::new();
            mix.read_to_end(&mut bytes).unwrap();
            bytes
        });
    }
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{decoded_seconds, sine, temp_dir};

    fn document(frasi: &[&str]) -> Document {
        Document {
            version: VERSION,
            pulizia_audio: Vec::new(),
            correzioni_testo: Vec::new(),
            creato: "2026-10-03T17:05:00+02:00".into(),
            durata_ms: 1500,
            modalita: Modalita::Mix,
            modello: Some("nemotron".into()),
            lingua_parlato: SpeechLanguage::from("it"),
            completa: true,
            parlanti: BTreeMap::from([("mix:1".into(), "Mario".into())]),
            diarizzazione: None,
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
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                    tempi: Vec::new(),
                    testo_corretto: false,
                    parlante_corretto: false,
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

    /// Un Tape con `json` come `trascrizione.json`.
    fn tape_with(path: &Path, json: &str) {
        let mut zip = ZipWriter::new(File::create(path).unwrap());
        zip.start_file(DOCUMENT, SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut zip, json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn si_rilegge_il_documento_scritto_e_l_audio_e_intatto() {
        let dir = temp_dir("tape-scrittura");
        let mix = ogg(&dir);
        let path = dir.join("Registrazione.tape");
        let written = document(&["Buongiorno.", "Iniziamo."]);
        write(&path, &[(Ingresso::Mix, &mix)], &written, None).unwrap();
        assert_eq!(read(&path).unwrap(), written);
        let json = serde_json::to_value(&written).unwrap();
        assert_eq!(json["parlanti"]["mix:1"], "Mario");
        assert_eq!(json["frasi"][0]["parlante"], 1);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        // Non sovrascrive un Tape esistente.
        let error = write(&path, &[(Ingresso::Mix, &mix)], &written, None).unwrap_err();
        assert!(matches!(error, AppError::UnwritableFolder(_)), "{error:?}");
        assert_eq!(read(&path).unwrap(), written);
    }

    #[test]
    fn con_gli_ingressi_separati_il_tape_ha_l_audio_di_ogni_ingresso() {
        let dir = temp_dir("tape-ingressi-separati");
        let (mix, mic, system) = (
            ogg(&dir),
            ogg_named(&dir, "mic.ogg", 1.0),
            ogg_named(&dir, "sys.ogg", 0.5),
        );
        let path = dir.join("Call.tape");
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
            None,
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
        let dir = temp_dir("tape-decodifica");
        let path = dir.join("Registrazione.tape");
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &document(&[]), None).unwrap();
        let seconds = decoded_seconds(&path);
        assert!((seconds - 1.5).abs() < 0.001, "{seconds} s");
    }

    #[test]
    fn i_campi_sconosciuti_si_ignorano() {
        let dir = temp_dir("tape-campi-sconosciuti");
        let path = dir.join("Futuro.tape");
        let mut json = serde_json::to_value(document(&["Ciao."])).unwrap();
        json["etichette"] = serde_json::json!(["riunione"]);
        json["frasi"][0]["confidenza"] = serde_json::json!(0.9);
        tape_with(&path, &json.to_string());
        assert_eq!(read(&path).unwrap(), document(&["Ciao."]));
    }

    #[test]
    fn una_versione_futura_da_errore_dedicato() {
        let dir = temp_dir("tape-versione-futura");
        let path = dir.join("Futuro.tape");
        // Anche se il resto dello schema è cambiato.
        tape_with(&path, r#"{ "version": 2, "testo": {} }"#);
        assert_eq!(read(&path).unwrap_err(), AppError::UnsupportedTape);
        let error = read(&dir.join("Non esiste.tape")).unwrap_err();
        assert!(matches!(error, AppError::UnreadableFile(_)), "{error:?}");
    }

    #[test]
    fn la_riscrittura_sostituisce_solo_il_testo_senza_lasciare_temporanei() {
        let dir = temp_dir("tape-riscrittura");
        let mix = ogg(&dir);
        let path = dir.join("Registrazione.tape");
        let mut incompleta = document(&["Uno."]);
        incompleta.completa = false;
        write(&path, &[(Ingresso::Mix, &mix)], &incompleta, None).unwrap();
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
        assert_eq!(names, ["Registrazione.tape", "mix.ogg"]);
    }

    #[test]
    fn la_forma_d_onda_si_rilegge_e_resta_dopo_correzioni_e_riscritture() {
        let dir = temp_dir("tape-forma-onda");
        let path = dir.join("Call.tape");
        let written = document(&["Ciao."]);
        write(
            &path,
            &[(Ingresso::Mix, &ogg(&dir))],
            &written,
            Some(&[0.25, 1.0, 0.0]),
        )
        .unwrap();
        assert_eq!(forma_onda(&path), Some(vec![0.25, 1.0, 0.0]));
        edit_frase(&path, Ingresso::Mix, 0, "Salve.").unwrap();
        rewrite(&path, &written).unwrap();
        assert_eq!(forma_onda(&path), Some(vec![0.25, 1.0, 0.0]));
        assert_eq!(read(&path).unwrap(), written);
    }

    #[test]
    fn un_tape_senza_forma_d_onda_la_riceve_una_volta_sola() {
        let dir = temp_dir("tape-forma-onda-aggiunta");
        let mix = ogg(&dir);
        let path = dir.join("Vecchio.tape");
        let written = document(&["Ciao."]);
        write(&path, &[(Ingresso::Mix, &mix)], &written, None).unwrap();
        assert_eq!(forma_onda(&path), None);
        save_forma_onda(&path, &[0.5]).unwrap();
        assert_eq!(forma_onda(&path), Some(vec![0.5]));
        // Due aperture insieme la calcolano due volte: resta la prima.
        save_forma_onda(&path, &[0.9]).unwrap();
        assert_eq!(forma_onda(&path), Some(vec![0.5]));
        assert_eq!(read(&path).unwrap(), written);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        // Una Forma d'onda vuota vale come assente, e si sostituisce.
        let vuota = dir.join("Vuota.tape");
        write(&vuota, &[(Ingresso::Mix, &mix)], &written, Some(&[])).unwrap();
        assert_eq!(forma_onda(&vuota), None);
        save_forma_onda(&vuota, &[0.5]).unwrap();
        assert_eq!(forma_onda(&vuota), Some(vec![0.5]));
    }

    #[test]
    fn una_riscrittura_fallita_lascia_il_tape_com_era() {
        let dir = temp_dir("tape-riscrittura-fallita");
        let path = dir.join("Registrazione.tape");
        write(
            &path,
            &[(Ingresso::Mix, &ogg(&dir))],
            &document(&["Uno."]),
            None,
        )
        .unwrap();
        let before = std::fs::read(&path).unwrap();
        // Il Tape aperto in esclusiva da un altro programma: il rename non riesce.
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
        assert!(!dir.join("Registrazione.tape.tmp").exists());
    }

    #[test]
    fn origine_si_rilegge_e_un_tape_senza_origine_si_legge_come_prima() {
        let dir = temp_dir("tape-origine");
        let path = dir.join("Call.tape");
        let written = Document {
            origine: Some("Call Teams.mp4".into()),
            ..document(&["Ciao."])
        };
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &written, None).unwrap();
        assert_eq!(read(&path).unwrap(), written);
        // Senza origine il campo non si scrive, e un Tape di prima si legge con `None`.
        let json = serde_json::to_value(document(&["Ciao."])).unwrap();
        assert!(json.get("origine").is_none(), "{json}");
        let old = dir.join("Vecchio.tape");
        tape_with(&old, &json.to_string());
        assert_eq!(read(&old).unwrap().origine, None);
    }

    #[test]
    fn il_testo_del_turno_si_corregge_in_un_solo_salvataggio_e_si_riapre() {
        let dir = temp_dir("editor-turno");
        let path = dir.join("Call.tape");
        let mut original = document(&["Prima.", "Seconda.", "Altro turno."]);
        original.frasi[2].parlante = Some(2);
        let audio = ogg(&dir);
        write(
            &path,
            &[(Ingresso::Mix, &audio)],
            &original,
            Some(&[0.2, 0.7]),
        )
        .unwrap();
        let audio_before = std::fs::read(&audio).unwrap();
        edit_turno(
            &path,
            Ingresso::Mix,
            &[0, 1],
            "Prima.\nSeconda.",
            "Prima e seconda.\n\nNuovo paragrafo.",
        )
        .unwrap();
        let opened = crate::managers::transcription::open_tape(&path).unwrap();
        assert_eq!(
            opened.phrases[0].text,
            "Prima e seconda.\n\nNuovo paragrafo."
        );
        assert_eq!(opened.phrases[1].text, "");
        assert_eq!(opened.phrases[2].text, "Altro turno.");
        assert!(opened.info.corretto_a_mano);
        let after = read(&path).unwrap();
        assert_eq!(after.frasi.len(), original.frasi.len());
        for (saved, before) in after.frasi.iter().zip(&original.frasi) {
            assert_eq!(
                (
                    saved.id,
                    saved.inizio_ms,
                    saved.fine_ms,
                    saved.ingresso,
                    saved.parlante
                ),
                (
                    before.id,
                    before.inizio_ms,
                    before.fine_ms,
                    before.ingresso,
                    before.parlante
                )
            );
        }
        let mut saved_audio = Vec::new();
        Mix::open(&path)
            .unwrap()
            .read_to_end(&mut saved_audio)
            .unwrap();
        assert_eq!(saved_audio, audio_before);
        assert_eq!(forma_onda(&path), Some(vec![0.2, 0.7]));
        let rendered = crate::managers::transcription::tape_text(
            &path,
            &crate::managers::settings::Settings::default(),
            crate::managers::settings::CopiaCome::Testo,
        )
        .unwrap();
        assert!(rendered.contains("Prima e seconda.\n\nNuovo paragrafo."));
        assert_eq!(rendered.matches("Nuovo paragrafo.").count(), 1);
        assert!(!rendered.contains("Seconda."));
    }

    #[test]
    fn editor_turno_noop_richieste_obsolete_ed_errore_non_riscrivono_il_tape() {
        let dir = temp_dir("editor-turno-atomicita");
        let path = dir.join("Call.tape");
        let doc = document(&["Prima.", "Seconda."]);
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &doc, None).unwrap();
        let before = std::fs::read(&path).unwrap();
        edit_turno(
            &path,
            Ingresso::Mix,
            &[0, 1],
            "Prima.\nSeconda.",
            "Prima.\nSeconda.",
        )
        .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before);
        for ids in [vec![0], vec![0, 0], vec![1, 0], vec![0, 1, 2]] {
            assert!(edit_turno(&path, Ingresso::Mix, &ids, "Prima.\nSeconda.", "Bozza").is_err());
        }
        assert!(
            edit_turno(
                &path,
                Ingresso::Sistema,
                &[0, 1],
                "Prima.\nSeconda.",
                "Bozza"
            )
            .is_err()
        );
        assert!(edit_turno(&path, Ingresso::Mix, &[0, 1], "Obsoleto", "Bozza").is_err());
        std::fs::create_dir(path.with_extension("tape.tmp")).unwrap();
        assert!(edit_turno(&path, Ingresso::Mix, &[0, 1], "Prima.\nSeconda.", "Bozza").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(read(&path).unwrap().correzioni_testo.is_empty());
    }

    #[test]
    fn editor_turno_vuoto_unito_non_aggiunge_separatori_alle_uscite() {
        for (suffix, empty_id, target_id) in [("sopra", 1, 0), ("sotto", 0, 1)] {
            let dir = temp_dir(&format!("editor-turno-vuoto-join-{suffix}"));
            let path = dir.join("Call.tape");
            let mut doc = document(&["Prima.", "Seconda."]);
            doc.frasi[1].parlante = Some(2);
            let original = doc.frasi[empty_id].testo.clone();
            let expected = doc.frasi[target_id].testo.clone();
            let audio = ogg(&dir);
            write(&path, &[(Ingresso::Mix, &audio)], &doc, None).unwrap();
            edit_turno(&path, Ingresso::Mix, &[empty_id as u32], &original, "").unwrap();
            unisci_turno(&path, Ingresso::Mix, &[empty_id as u32], target_id as u32).unwrap();
            let after = read(&path).unwrap();
            assert_eq!(after.testo_del_turno(&[0, 1], "\n"), expected);
            let exported = crate::managers::transcription::tape_text(
                &path,
                &crate::managers::settings::Settings::default(),
                crate::managers::settings::CopiaCome::Testo,
            )
            .unwrap();
            assert!(exported.contains(&expected));
            assert_eq!(after.frasi_per_lettura()[0].testo, expected);
            assert_eq!(after.frasi.len(), 2);
            assert_eq!(after.correzioni_testo[0].testo, "");
        }
    }

    #[test]
    fn editor_turno_vuoto_e_unisci_conservano_testo_audio_e_localita() {
        let dir = temp_dir("editor-turno-vuoto-unisci");
        let path = dir.join("Call.tape");
        let mut doc = document(&["Prima.", "Seconda.", "Anna.", "Altro ingresso."]);
        doc.frasi[2].parlante = Some(2);
        doc.frasi[3].ingresso = Ingresso::Sistema;
        let other = doc.frasi[3].clone();
        let audio = ogg(&dir);
        write(&path, &[(Ingresso::Mix, &audio)], &doc, Some(&[0.2])).unwrap();
        edit_turno(
            &path,
            Ingresso::Mix,
            &[0, 1],
            "Prima.\nSeconda.",
            "Testo unito.\n\nParagrafo.",
        )
        .unwrap();
        unisci_turno(&path, Ingresso::Mix, &[2], 1).unwrap();
        let after = read(&path).unwrap();
        assert_eq!(after.frasi[3], other);
        assert_eq!(
            after.correzioni_testo[0].testo,
            "Testo unito.\n\nParagrafo."
        );
        assert_eq!(after.frasi[2].parlante, Some(1));
        edit_turno(
            &path,
            Ingresso::Mix,
            &[0, 1, 2],
            "Testo unito.\n\nParagrafo.\nAnna.",
            "",
        )
        .unwrap();
        let opened = crate::managers::transcription::open_tape(&path).unwrap();
        assert!(opened.phrases[..3].iter().all(|f| f.text.is_empty()));
        assert_eq!(opened.phrases[3].text, "Altro ingresso.");
        assert!(opened.info.corretto_a_mano);
        let mut saved_audio = Vec::new();
        Mix::open(&path)
            .unwrap()
            .read_to_end(&mut saved_audio)
            .unwrap();
        assert_eq!(saved_audio, std::fs::read(audio).unwrap());
        assert_eq!(forma_onda(&path), Some(vec![0.2]));
        assert_eq!(read(&path).unwrap().frasi[3], other);
    }

    #[test]
    fn una_correzione_testuale_si_ricorda_alla_riapertura_del_tape() {
        let dir = temp_dir("tape-testo-manuale");
        let path = dir.join("Call.tape");
        write(
            &path,
            &[(Ingresso::Mix, &ogg(&dir))],
            &document(&["Errore."]),
            None,
        )
        .unwrap();
        edit_frase(&path, Ingresso::Mix, 0, "Corretto.").unwrap();
        let json = serde_json::to_value(read(&path).unwrap()).unwrap();
        assert_eq!(json["frasi"][0]["testo_corretto"], true);
    }

    #[test]
    fn unione_corregge_solo_il_turno_scelto_e_conserva_i_riferimenti() {
        for target in [0, 3] {
            let dir = temp_dir(&format!("unisci-{target}"));
            let path = dir.join("Call.tape");
            let mut doc = document(&["Mario.", "Anna.", "Ancora Anna.", "Mario.", "Altra Anna."]);
            for (f, speaker) in doc.frasi.iter_mut().zip([1, 2, 2, 1, 2]) {
                f.parlante = Some(speaker);
            }
            write(&path, &[(Ingresso::Mix, ogg(&dir).as_path())], &doc, None).unwrap();
            let audio = std::fs::read(&path).unwrap();
            let mut library = crate::library::Library::open(&dir, &dir.join("indice.db")).unwrap();
            library.sync().unwrap();
            assert!(library.search("Mario Ancora", None).unwrap().is_empty());
            for (ids, destination) in [(vec![1], target), (vec![1, 2], 4), (vec![1, 1], target)] {
                assert!(unisci_turno(&path, Ingresso::Mix, &ids, destination).is_err());
                assert_eq!(std::fs::read(&path).unwrap(), audio);
            }
            unisci_turno(&path, Ingresso::Mix, &[1, 2], target).unwrap();
            let after = read(&path).unwrap();
            let mut expected = doc.clone();
            for f in &mut expected.frasi[1..=2] {
                f.parlante = Some(1);
                f.parlante_corretto = true;
            }
            assert_eq!(after, expected);
            assert!(after.corretto_a_mano());
            library.sync().unwrap();
            let found = library.search("Mario Ancora", None).unwrap();
            assert_eq!(found[0].frasi[0].phrase_id, 2);
        }
    }

    #[test]
    fn aprire_senza_modificare_non_segna_il_testo_e_una_copia_conserva_la_correzione() {
        let dir = temp_dir("correzione-noop-copia");
        let path = dir.join("Call.tape");
        let mut doc = document(&["Originale."]);
        doc.parlanti.clear();
        write(&path, &[(Ingresso::Mix, ogg(&dir).as_path())], &doc, None).unwrap();
        edit_frase(&path, Ingresso::Mix, 0, "Originale.").unwrap();
        assert_eq!(read(&path).unwrap(), doc);
        assert!(
            !crate::managers::transcription::open_tape(&path)
                .unwrap()
                .info
                .corretto_a_mano
        );
        edit_frase(&path, Ingresso::Mix, 0, "Corretto.").unwrap();
        let copy = dir.join("Copia.tape");
        std::fs::copy(&path, &copy).unwrap();
        assert!(
            crate::managers::transcription::open_tape(&copy)
                .unwrap()
                .info
                .corretto_a_mano
        );
        assert_eq!(read(&copy).unwrap().frasi[0].testo, "Corretto.");
    }

    #[test]
    fn unione_non_attraversa_ingressi_o_destinazioni_non_determinate() {
        let dir = temp_dir("unisci-invalidi");
        let path = dir.join("Call.tape");
        let mut doc = document(&["Uno.", "Due.", "Tre."]);
        doc.frasi[0].ingresso = Ingresso::Microfono;
        doc.frasi[1].parlante = Some(2);
        doc.frasi[2].parlante_non_determinato = true;
        write(&path, &[(Ingresso::Mix, ogg(&dir).as_path())], &doc, None).unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(unisci_turno(&path, Ingresso::Mix, &[1], 0).is_err());
        assert!(unisci_turno(&path, Ingresso::Mix, &[1], 2).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        // La sorgente non determinata, invece, può essere corretta verso una voce nota.
        unisci_turno(&path, Ingresso::Mix, &[2], 1).unwrap();
        assert!(!read(&path).unwrap().frasi[2].parlante_non_determinato);
    }

    #[test]
    fn la_correzione_cambia_solo_il_testo_di_quella_frase() {
        let dir = temp_dir("tape-correzione");
        let mix = ogg(&dir);
        let path = dir.join("Call.tape");
        let mut before = document(&["Buongiorno.", "Inizziamo.", "Bene."]);
        before.origine = Some("Call.mp4".into());
        before.frasi[2].ingresso = Ingresso::Sistema;
        before.frasi[2].id = 1;
        write(&path, &[(Ingresso::Mix, &mix)], &before, None).unwrap();
        edit_frase(&path, Ingresso::Mix, 1, "Iniziamo.").unwrap();
        let mut expected = before.clone();
        expected.frasi[1].testo = "Iniziamo.".into();
        expected.frasi[1].testo_corretto = true;
        assert_eq!(read(&path).unwrap(), expected);
        // Lo stesso id in un altro Ingresso è un'altra Frase; una Frase svuotata resta.
        edit_frase(&path, Ingresso::Sistema, 1, "").unwrap();
        expected.frasi[2].testo = String::new();
        expected.frasi[2].testo_corretto = true;
        assert_eq!(read(&path).unwrap(), expected);
        let mut audio = Vec::new();
        Mix::open(&path).unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, std::fs::read(&mix).unwrap());
        // Una Frase che non c'è non cambia nulla.
        assert!(edit_frase(&path, Ingresso::Microfono, 0, "x").is_err());
        assert_eq!(read(&path).unwrap(), expected);
    }

    #[test]
    fn correzioni_contemporanee_dello_stesso_tape_restano_tutte() {
        let dir = temp_dir("tape-correzioni-contemporanee");
        let path = dir.join("Call.tape");
        let testi: Vec<String> = (0..8).map(|i| format!("Frase {i}.")).collect();
        let refs: Vec<&str> = testi.iter().map(String::as_str).collect();
        write(
            &path,
            &[(Ingresso::Mix, &ogg(&dir))],
            &document(&refs),
            None,
        )
        .unwrap();
        std::thread::scope(|scope| {
            for id in 0..8 {
                let path = &path;
                scope.spawn(move || edit_frase(path, Ingresso::Mix, id, "corretta").unwrap());
            }
            scope.spawn(|| rename_parlante(&path, Ingresso::Mix, Some(2), "Lucia").unwrap());
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
    fn la_data_si_cambia_con_l_ora_locale_e_il_resto_resta() {
        let dir = temp_dir("tape-creato");
        let path = dir.join("Call.tape");
        let before = document(&["Ciao."]);
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &before, None).unwrap();
        let creato = set_creato(&path, "2025-12-31T23:30").unwrap();
        let after = read(&path).unwrap();
        assert_eq!(after.creato, creato);
        assert_eq!(after.date(), "2025-12-31 23:30");
        assert_eq!(after.frasi, before.frasi);
        assert_eq!(after.durata_ms, before.durata_ms);
        // Un valore che non è data e ora si rifiuta e non cambia nulla.
        assert!(set_creato(&path, "31/12/2025").is_err());
        assert_eq!(read(&path).unwrap().creato, creato);
    }

    #[test]
    fn annulla_dopo_la_copia_del_tape_impedisce_la_sostituzione_e_rimuove_il_temporaneo() {
        let dir = temp_dir("tape-annulla-sostituzione");
        let path = dir.join("Call.tape");
        let before = document(&["Il testo precedente."]);
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &before, None).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let changed = document(&["Il risultato nuovo."]);
        // Il controllo avviene dopo aver scritto e chiuso l'intero zip temporaneo.
        let result = replace_checked(
            &path,
            DOCUMENT,
            |zip, temp| write_document(zip, temp, &changed),
            || Err(AppError::Cancelled),
        );
        assert_eq!(result, Err(AppError::Cancelled));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(!path.with_extension("tape.tmp").exists());
        let cancel = transcribe_cpp::CancelToken::new();
        cancel.cancel();
        assert_eq!(
            rewrite_cancellable(&path, &changed, &cancel),
            Err(AppError::Cancelled)
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(!path.with_extension("tape.tmp").exists());
    }

    /// Le voci del Tape decompresse, per nome.
    fn entries(path: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut zip = open(path).unwrap();
        (0..zip.len())
            .map(|i| {
                let mut entry = zip.by_index(i).unwrap();
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).unwrap();
                (entry.name().to_string(), bytes)
            })
            .collect()
    }

    #[test]
    fn ritrascrivere_cambia_solo_il_documento_e_conserva_audio_e_dati_aggiuntivi() {
        let dir = temp_dir("tape-ritrascrizione");
        let path = dir.join("Call.tape");
        let mut before = document(&["Il testo precedente."]);
        before.pulizia_audio = vec![crate::audio_toolkit::cleaning::TrattoPulizia {
            ingresso: Ingresso::Microfono,
            algoritmo: "deepfilternet3".into(),
            versione: "v1".into(),
            frequenza: 16_000,
            inizio_frame: 0,
            fine_frame: 16_000,
        }];
        let audio = ogg(&dir);
        write(
            &path,
            &[(Ingresso::Mix, &audio), (Ingresso::Microfono, &audio)],
            &before,
            Some(&[0.3]),
        )
        .unwrap();
        // Un campo e una voce di una versione futura.
        let mut json: serde_json::Value =
            serde_json::from_reader(open(&path).unwrap().by_name(DOCUMENT).unwrap()).unwrap();
        json["dati_futuri"] = serde_json::json!("da conservare");
        replace(&path, DOCUMENT, |zip, temp| {
            zip.start_file("allegato.txt", SimpleFileOptions::default())
                .map_err(|e| unwritable(temp, &e))?;
            std::io::Write::write_all(zip, b"dato aggiuntivo").map_err(|e| unwritable(temp, &e))?;
            zip.start_file(DOCUMENT, SimpleFileOptions::default())
                .map_err(|e| unwritable(temp, &e))?;
            serde_json::to_writer(zip, &json).map_err(|e| unwritable(temp, &e))
        })
        .unwrap();
        let old = entries(&path);
        let mut new = document(&["Il risultato nuovo."]);
        new.pulizia_audio.clone_from(&before.pulizia_audio);
        let cancel = transcribe_cpp::CancelToken::new();
        cancel.cancel();
        assert_eq!(
            replace_transcription(&path, &new, &cancel),
            Err(AppError::Cancelled)
        );
        assert_eq!(entries(&path), old);
        replace_transcription(&path, &new, &transcribe_cpp::CancelToken::new()).unwrap();
        let mut after = entries(&path);
        let json: serde_json::Value = serde_json::from_slice(&after[DOCUMENT]).unwrap();
        assert_eq!(json["dati_futuri"], "da conservare");
        assert_eq!(read(&path).unwrap().frasi, new.frasi);
        assert_eq!(read(&path).unwrap().pulizia_audio, before.pulizia_audio);
        after.remove(DOCUMENT);
        let mut old = old;
        old.remove(DOCUMENT);
        assert_eq!(after, old);
    }

    #[test]
    fn la_rinomina_di_un_parlante_si_salva_nel_tape() {
        let dir = temp_dir("tape-rinomina");
        let path = dir.join("Call.tape");
        let before = document(&["Ciao.", "Salve."]);
        write(&path, &[(Ingresso::Mix, &ogg(&dir))], &before, None).unwrap();
        rename_parlante(&path, Ingresso::Sistema, Some(2), "  Lucia ").unwrap();
        // Il Microfono senza Parlanti si rinomina come una persona sola.
        rename_parlante(&path, Ingresso::Microfono, None, "Francesco").unwrap();
        // Un nome vuoto si rifiuta e non cambia nulla.
        assert!(rename_parlante(&path, Ingresso::Mix, Some(1), "  ").is_err());
        let after = read(&path).unwrap();
        assert_eq!(
            after.parlanti,
            BTreeMap::from([
                ("microfono".to_string(), "Francesco".to_string()),
                ("mix:1".to_string(), "Mario".to_string()),
                ("sistema:2".to_string(), "Lucia".to_string()),
            ])
        );
        assert_eq!(after.frasi, before.frasi);
    }

    #[test]
    fn dagli_argomenti_di_avvio_il_primo_tape_rispetto_alla_cartella_di_lavoro() {
        let args = |a: &[&str]| a.iter().map(ToString::to_string).collect::<Vec<_>>();
        let cwd = Path::new(r"C:\Lavoro");
        assert_eq!(
            from_args(
                args(&[r"C:\Memotape\memotape.exe", r"D:\Note\Lezione.TAPE"]),
                cwd
            ),
            Some(PathBuf::from(r"D:\Note\Lezione.TAPE"))
        );
        assert_eq!(
            from_args(args(&["memotape.exe", "--flag", "Lezione.tape"]), cwd),
            Some(PathBuf::from(r"C:\Lavoro\Lezione.tape"))
        );
        // L'eseguibile non conta, nemmeno se si chiamasse `.tape`.
        assert_eq!(from_args(args(&["x.tape"]), cwd), None);
        assert_eq!(from_args(args(&["memotape.exe", "audio.mp3"]), cwd), None);
    }
}
