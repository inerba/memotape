//! Impostazioni persistenti: `settings.json` in `app_data_dir`, letto e validato all'avvio.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::error::AppError;
use crate::managers::models;
use crate::transcript::Ingresso;

/// I bitrate di una Registrazione, in kbps.
pub const BITRATES_KBPS: [u32; 9] = [16, 24, 32, 48, 64, 96, 128, 192, 320];
/// Le frequenze di una Registrazione, in Hz.
pub const SAMPLE_RATES: [u32; 4] = [8_000, 16_000, 24_000, 48_000];

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub recording_source: RecordingSource,
    /// `null`: il microfono predefinito di sistema.
    pub microphone: Option<String>,
    /// `null`: il dispositivo di uscita predefinito, per l'audio di sistema.
    pub output_device: Option<String>,
    /// L'id del modello nel catalogo.
    pub model: String,
    /// Scelta distinta dall'ASR, fissata all'avvio dell'Attività. I file precedenti usano Sortformer.
    #[serde(default)]
    pub diarizer: Diarizer,
    /// Artefatto locale verificato: non si scarica e non si sostituisce automaticamente.
    #[serde(default)]
    pub nemotron3_path: Option<String>,
    pub speech_language: SpeechLanguage,
    pub bitrate_kbps: u32,
    pub channels: Channels,
    /// Hz.
    pub sample_rate: u32,
    /// La Cartella della Libreria; `null`: `Documenti\Memotape`.
    pub recordings_folder: Option<String>,
    /// `null`: la lingua di sistema se è tra le sei, altrimenti l'inglese.
    pub interface_language: Option<Language>,
    /// Trascrivi dal vivo: la Registrazione trascrive mentre registra. Manca nei file salvati
    /// prima che esistesse: allora è spenta.
    #[serde(default)]
    pub trascrizione_dal_vivo: bool,
    /// Il formato di Copia testo. Manca nei file salvati prima che esistesse: allora è testo.
    #[serde(default)]
    pub copia_come: CopiaCome,
    /// Riconosci i parlanti: Trascrivi su un file o su un Tape diarizza dopo la Trascrizione. Manca
    /// nei file salvati prima che esistesse: allora è spenta.
    #[serde(default)]
    pub parlanti_file: bool,
    /// Riconosci i parlanti di una Registrazione, dopo Stop, sul mix (da un solo Ingresso) o, da
    /// Entrambi, sul microfono e sull'audio di sistema. Mancano nei file salvati prima che
    /// esistessero: allora sono spente.
    #[serde(default)]
    pub parlanti_mix: bool,
    /// Riconosci i parlanti sul microfono, con gli Ingressi separati. Spenta, il microfono è una
    /// persona sola (ADR-0015).
    #[serde(default)]
    pub parlanti_microfono: bool,
    /// Riconosci i parlanti sull'audio di sistema, con gli Ingressi separati.
    #[serde(default)]
    pub parlanti_sistema: bool,
    /// La Raccolta scelta nella barra laterale: `null` Tutta la Libreria, `""` Senza raccolta,
    /// altrimenti il nome. Manca nei file salvati prima che esistesse: allora è Tutta la Libreria.
    #[serde(default)]
    pub raccolta: Option<String>,
    /// Il tema dell'interfaccia. Manca nei file salvati prima che esistesse: allora segue Windows.
    #[serde(default)]
    pub tema: Tema,
    /// Consenti agli Assistenti di leggere la Libreria con il server MCP (ADR-0012). Manca nei file
    /// salvati prima che esistesse: allora è spenta.
    #[serde(default)]
    pub assistenti: bool,
    /// Il Guadagno del microfono e dell'audio di sistema in dB, da −12 a +24 a passi di 3: vale
    /// per la prossima Registrazione e, cambiato durante una Registrazione, anche per quella.
    /// Mancano nei file salvati prima che esistessero: allora sono 0 dB.
    #[serde(default)]
    pub guadagno_microfono: i8,
    #[serde(default)]
    pub guadagno_sistema: i8,
}

/// Il fattore per cui un Guadagno di `db` moltiplica i campioni.
pub fn guadagno_factor(db: i8) -> f32 {
    10_f32.powf(f32::from(db) / 20.0)
}

fn valid_guadagno(db: i8) -> bool {
    (-12..=24).contains(&db) && db % 3 == 0
}

/// Il tema dell'interfaccia: quello di Windows o uno fisso.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum Tema {
    #[default]
    Sistema,
    Chiaro,
    Scuro,
}

impl Tema {
    /// Il tema della finestra; `None` segue Windows.
    pub fn theme(self) -> Option<tauri::Theme> {
        match self {
            Self::Sistema => None,
            Self::Chiaro => Some(tauri::Theme::Light),
            Self::Scuro => Some(tauri::Theme::Dark),
        }
    }

    /// Lo applica alla finestra principale: barra del titolo e `prefers-color-scheme` della
    /// WebView2, quindi i token scuri di `global.css` e la variante `dark:` lo seguono.
    pub fn apply(self, app: &tauri::AppHandle) {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("main")
            && let Err(e) = window.set_theme(self.theme())
        {
            log::warn!("tema non applicato: {e}");
        }
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum CopiaCome {
    /// Testo semplice, senza sintassi Markdown.
    #[default]
    Testo,
    Markdown,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum Diarizer {
    #[default]
    Sortformer,
    Nemotron3,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RecordingSource {
    Mic,
    System,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Channels {
    Mono,
    Stereo,
}

/// Le sei lingue dell'interfaccia.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    It,
    En,
    Fr,
    Es,
    De,
    Pl,
}

impl Language {
    /// La lingua dell'app per un locale BCP 47 (`it-IT`, `es-419`): quella con lo stesso codice,
    /// altrimenti l'inglese.
    pub fn from_locale(locale: &str) -> Self {
        let code = locale.split(['-', '_']).next().unwrap_or_default();
        match code.to_ascii_lowercase().as_str() {
            "it" => Self::It,
            "fr" => Self::Fr,
            "es" => Self::Es,
            "de" => Self::De,
            "pl" => Self::Pl,
            _ => Self::En,
        }
    }

    /// Il codice ISO 639 (`it`).
    pub fn code(self) -> &'static str {
        match self {
            Self::It => "it",
            Self::En => "en",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::De => "de",
            Self::Pl => "pl",
        }
    }

    /// La lingua di visualizzazione di Windows, se è tra le sei, altrimenti l'inglese.
    pub fn system() -> Self {
        sys_locale::get_locale().map_or(Self::En, |locale| Self::from_locale(&locale))
    }
}

/// La Lingua del parlato: `auto` o il codice ISO 639 di una lingua senza regione (`it`, `ja`,
/// `yue`). Si offrono le lingue del modello scelto; `engine::resolve_language` lo traduce nel
/// codice del modello (`it-IT` per Nemotron).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(transparent)]
pub struct SpeechLanguage(String);

impl SpeechLanguage {
    pub fn auto() -> Self {
        Self("auto".into())
    }

    /// Il codice da indicare al modello; `None` per il riconoscimento automatico.
    pub fn code(&self) -> Option<&str> {
        Some(self.0.as_str()).filter(|c| *c != "auto")
    }

    /// `auto` o 2–3 lettere minuscole.
    fn is_valid(&self) -> bool {
        self.0 == "auto"
            || (2..=3).contains(&self.0.len()) && self.0.bytes().all(|b| b.is_ascii_lowercase())
    }

    /// Il nome della lingua nella lingua `interface`, con l'iniziale maiuscola (`Italiano`), dalle
    /// ICU di Windows. `None` per Automatica.
    pub fn name(&self, interface: Language) -> Option<String> {
        use windows_sys::Win32::Globalization::uloc_getDisplayLanguage;
        let code = std::ffi::CString::new(self.code()?).ok()?;
        let display = std::ffi::CString::new(interface.code()).ok()?;
        let mut buffer = [0u16; 64];
        let mut status = 0;
        // SAFETY: stringhe terminate da zero e un buffer della capacità dichiarata.
        let len = unsafe {
            uloc_getDisplayLanguage(
                code.as_ptr().cast(),
                display.as_ptr().cast(),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                &mut status,
            )
        };
        if status > 0 || len <= 0 {
            return Some(self.0.clone());
        }
        let name = String::from_utf16_lossy(&buffer[..len as usize]);
        let mut chars = name.chars();
        Some(chars.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(chars).collect()
        }))
    }
}

impl From<&str> for SpeechLanguage {
    fn from(code: &str) -> Self {
        Self(code.into())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recording_source: RecordingSource::Mic,
            microphone: None,
            output_device: None,
            model: models::default_model().id.clone(),
            diarizer: Diarizer::Sortformer,
            nemotron3_path: None,
            speech_language: SpeechLanguage::auto(),
            bitrate_kbps: 16,
            channels: Channels::Mono,
            sample_rate: 16_000,
            recordings_folder: None,
            interface_language: None,
            trascrizione_dal_vivo: false,
            copia_come: CopiaCome::Testo,
            parlanti_file: false,
            parlanti_mix: false,
            parlanti_microfono: false,
            parlanti_sistema: false,
            raccolta: None,
            tema: Tema::Sistema,
            assistenti: false,
            guadagno_microfono: 0,
            guadagno_sistema: 0,
        }
    }
}

impl Settings {
    /// Gli audio da diarizzare dopo Stop: quelli trascritti dal vivo (il mix, o da Entrambi ogni
    /// Ingresso) con la loro casella attiva. Senza Trascrizione dal vivo nessuno: non ci sono Frasi
    /// da attribuire.
    pub fn parlanti_registrazione(&self) -> Vec<Ingresso> {
        if !self.trascrizione_dal_vivo {
            return Vec::new();
        }
        let chosen: &[(bool, Ingresso)] = if self.ingressi_separati() {
            &[
                (self.parlanti_microfono, Ingresso::Microfono),
                (self.parlanti_sistema, Ingresso::Sistema),
            ]
        } else {
            &[(self.parlanti_mix, Ingresso::Mix)]
        };
        chosen
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, ingresso)| *ingresso)
            .collect()
    }

    /// Gli audio da diarizzare con Riconosci i parlanti di Trascrivi: il mix di un file o di un Tape
    /// con solo il mix; di un Tape con l'audio di ogni Ingresso (`separate`) l'Audio di sistema e,
    /// solo con la casella del Microfono della Registrazione, il Microfono (ADR-0015).
    pub fn parlanti_trascrivi(&self, separate: bool) -> Vec<Ingresso> {
        if !self.parlanti_file {
            return Vec::new();
        }
        if !separate {
            return vec![Ingresso::Mix];
        }
        let mut chosen = Vec::new();
        if self.parlanti_microfono {
            chosen.push(Ingresso::Microfono);
        }
        chosen.push(Ingresso::Sistema);
        chosen
    }

    /// Se la Registrazione è a Ingressi separati: da Entrambi sempre (ADR-0015). Salva l'audio di
    /// ogni Ingresso e, dal vivo, li trascrive ognuno per conto suo.
    pub fn ingressi_separati(&self) -> bool {
        self.recording_source == RecordingSource::Both
    }

    /// Legge `path`. Se manca, è corrotto o non è valido restituisce i predefiniti: l'avvio non si
    /// blocca. Un modello non più nel catalogo torna al predefinito senza toccare il resto. Un file
    /// che esiste ma non si legge (permessi, una cartella al suo posto) dà `unreadableSettings`.
    pub fn load(path: &Path) -> Result<Self, AppError> {
        // Byte e non stringa: un file non UTF-8 è corrotto, non illeggibile.
        let read = match read_retrying(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => {
                return Err(AppError::UnreadableSettings(format!(
                    "{}: {e}",
                    path.display()
                )));
            }
        };
        Ok(match serde_json::from_slice::<Self>(&read) {
            Ok(mut settings) if settings.is_valid() => {
                if models::find_transcriber(&settings.model).is_none() {
                    log::warn!("modello {} non più nel catalogo", settings.model);
                    settings.model = Self::default().model;
                }
                settings
            }
            Ok(_) => {
                log::warn!("impostazioni non valide in {}", path.display());
                Self::default()
            }
            Err(e) => {
                log::warn!("impostazioni corrotte in {}: {e}", path.display());
                Self::default()
            }
        })
    }

    /// Scrive `path` passando da un file temporaneo, così un'interruzione non lo lascia a metà.
    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        if !self.is_valid() || models::find_transcriber(&self.model).is_none() {
            return Err(AppError::Internal(format!(
                "impostazioni non valide: {self:?}"
            )));
        }
        let unwritable =
            |e: std::io::Error| AppError::UnwritableFolder(format!("{}: {e}", path.display()));
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(unwritable)?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| AppError::Internal(e.to_string()))?;
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, json).map_err(unwritable)?;
        std::fs::rename(&temp, path).map_err(unwritable)
    }

    fn is_valid(&self) -> bool {
        BITRATES_KBPS.contains(&self.bitrate_kbps)
            && SAMPLE_RATES.contains(&self.sample_rate)
            && self.speech_language.is_valid()
            && valid_guadagno(self.guadagno_microfono)
            && valid_guadagno(self.guadagno_sistema)
    }
}

/// Legge `path`, riprovando per circa un secondo se un altro processo lo tiene aperto in esclusiva
/// (antivirus, OneDrive, un editor): `ERROR_SHARING_VIOLATION` e `ERROR_LOCK_VIOLATION`.
fn read_retrying(path: &Path) -> std::io::Result<Vec<u8>> {
    for _ in 0..10 {
        match std::fs::read(path) {
            Err(e) if matches!(e.raw_os_error(), Some(32 | 33)) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            read => return read,
        }
    }
    std::fs::read(path)
}

/// Le impostazioni correnti e il file in cui si salvano. In `tauri::State`.
pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
    /// Perché il file non si è letto all'avvio: allora valgono i predefiniti, finché un
    /// salvataggio non riesce.
    load_error: Mutex<Option<AppError>>,
}

impl SettingsStore {
    /// Legge `path` all'avvio. Se non si legge valgono i predefiniti, e l'errore resta per `loaded`.
    pub fn load(path: PathBuf) -> Self {
        let (current, load_error) = match Settings::load(&path) {
            Ok(settings) => (settings, None),
            Err(e) => {
                log::warn!("{e}");
                (Settings::default(), Some(e))
            }
        };
        Self {
            path,
            current: Mutex::new(current),
            load_error: Mutex::new(load_error),
        }
    }

    /// Le impostazioni correnti, o l'errore se all'avvio il file non si è letto.
    pub fn loaded(&self) -> Result<Settings, AppError> {
        let error = self
            .load_error
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        error.map_or_else(|| Ok(self.get()), Err)
    }

    /// Il modello scelto.
    pub fn model(&self) -> &'static models::Model {
        Self::model_of(&self.get())
    }

    /// Il modello di `settings`, che sono validate: è nel catalogo.
    pub fn model_of(settings: &Settings) -> &'static models::Model {
        models::find_transcriber(&settings.model).unwrap_or_else(models::default_model)
    }

    pub fn get(&self) -> Settings {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Salva `settings` e le rende correnti; restituisce le precedenti. Se all'avvio il file non si è
    /// letto, non lo sovrascrive con i predefiniti: lo rilegge e applica sopra solo i campi che
    /// `settings` cambia rispetto alle correnti, o risponde con l'errore se ancora non si legge.
    pub fn set(&self, settings: Settings) -> Result<Settings, AppError> {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        let mut load_error = self
            .load_error
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let settings = if load_error.is_some() {
            let file = Settings::load(&self.path).inspect_err(|e| *load_error = Some(e.clone()))?;
            changes_onto(&current, &settings, file)?
        } else {
            settings
        };
        settings.save(&self.path)?;
        *load_error = None;
        Ok(std::mem::replace(&mut current, settings))
    }
}

/// `onto` con i campi che `next` cambia rispetto a `base`.
fn changes_onto(base: &Settings, next: &Settings, onto: Settings) -> Result<Settings, AppError> {
    let internal = |e: serde_json::Error| AppError::Internal(e.to_string());
    let base = serde_json::to_value(base).map_err(internal)?;
    let next = serde_json::to_value(next).map_err(internal)?;
    let mut merged = serde_json::to_value(onto).map_err(internal)?;
    if let (Some(next), Some(merged)) = (next.as_object(), merged.as_object_mut()) {
        for (key, value) in next {
            if base.get(key) != Some(value) {
                merged.insert(key.clone(), value.clone());
            }
        }
    }
    serde_json::from_value(merged).map_err(internal)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn senza_scelta_del_diarizer_si_conservano_sortformer_e_le_impostazioni() {
        let mut old = serde_json::to_value(Settings {
            parlanti_file: true,
            ..Settings::default()
        })
        .unwrap();
        old.as_object_mut().unwrap().remove("diarizer");
        old.as_object_mut().unwrap().remove("nemotron3Path");
        let path = temp_file("diarizer-precedente");
        std::fs::write(&path, old.to_string()).unwrap();
        let read = Settings::load(&path).unwrap();
        assert_eq!(read.diarizer, Diarizer::Sortformer);
        assert_eq!(read.nemotron3_path, None);
        assert!(read.parlanti_file);
    }

    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("memotape-test-impostazioni-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("settings.json")
    }

    #[test]
    fn senza_file_valgono_i_predefiniti() {
        let settings = Settings::load(&temp_file("assente")).unwrap();
        assert_eq!(settings, Settings::default());
        assert_eq!(settings.model, "nemotron-3.5-streaming-0.6b-q5km");
        assert_eq!(settings.speech_language, SpeechLanguage::auto());
        assert_eq!(settings.recording_source, RecordingSource::Mic);
        assert_eq!(
            (
                settings.bitrate_kbps,
                settings.channels,
                settings.sample_rate
            ),
            (16, Channels::Mono, 16_000)
        );
        assert_eq!(settings.recordings_folder, None);
        assert_eq!(settings.interface_language, None);
        assert!(!settings.trascrizione_dal_vivo);
        assert_eq!(settings.copia_come, CopiaCome::Testo);
        assert!(!settings.parlanti_file);
        assert!(
            !settings.parlanti_mix && !settings.parlanti_microfono && !settings.parlanti_sistema
        );
        assert_eq!(settings.raccolta, None);
        assert_eq!(settings.tema, Tema::Sistema);
        assert!(!settings.assistenti);
    }

    #[test]
    fn dopo_stop_si_diarizzano_gli_audio_trascritti_dal_vivo_con_la_casella_attiva() {
        let tutte = Settings {
            trascrizione_dal_vivo: true,
            recording_source: RecordingSource::Both,
            parlanti_mix: true,
            parlanti_microfono: true,
            parlanti_sistema: true,
            ..Settings::default()
        };
        // Da Entrambi mai il mix: i Parlanti di un Ingresso non si mescolano con quelli dell'altro.
        assert_eq!(
            tutte.parlanti_registrazione(),
            [Ingresso::Microfono, Ingresso::Sistema]
        );
        let solo_sistema = Settings {
            parlanti_microfono: false,
            ..tutte.clone()
        };
        assert_eq!(solo_sistema.parlanti_registrazione(), [Ingresso::Sistema]);
        // Da un solo Ingresso vale il mix.
        for recording_source in [RecordingSource::Mic, RecordingSource::System] {
            let one = Settings {
                recording_source,
                ..tutte.clone()
            };
            assert_eq!(one.parlanti_registrazione(), [Ingresso::Mix]);
        }
        // Senza Trascrizione dal vivo non ci sono Frasi.
        let spenta = Settings {
            trascrizione_dal_vivo: false,
            ..tutte
        };
        assert_eq!(spenta.parlanti_registrazione(), []);
    }

    #[test]
    fn da_entrambi_la_registrazione_e_sempre_a_ingressi_separati() {
        for trascrizione_dal_vivo in [false, true] {
            let both = Settings {
                trascrizione_dal_vivo,
                recording_source: RecordingSource::Both,
                ..Settings::default()
            };
            assert!(both.ingressi_separati());
        }
        for recording_source in [RecordingSource::Mic, RecordingSource::System] {
            let one = Settings {
                trascrizione_dal_vivo: true,
                recording_source,
                ..Settings::default()
            };
            assert!(!one.ingressi_separati());
        }
    }

    #[test]
    fn un_tape_con_l_audio_degli_ingressi_diarizza_il_sistema_e_il_microfono_solo_a_richiesta() {
        let file = Settings {
            parlanti_file: true,
            ..Settings::default()
        };
        assert_eq!(file.parlanti_trascrivi(true), [Ingresso::Sistema]);
        let microfono = Settings {
            parlanti_microfono: true,
            ..file.clone()
        };
        assert_eq!(
            microfono.parlanti_trascrivi(true),
            [Ingresso::Microfono, Ingresso::Sistema]
        );
        // Un file, o un Tape con solo il mix.
        assert_eq!(microfono.parlanti_trascrivi(false), [Ingresso::Mix]);
        let spenta = Settings {
            parlanti_file: false,
            ..microfono
        };
        assert_eq!(spenta.parlanti_trascrivi(true), []);
        assert_eq!(spenta.parlanti_trascrivi(false), []);
    }

    #[test]
    fn un_file_di_prima_delle_impostazioni_v2_le_tiene_ai_predefiniti_senza_perdere_il_resto() {
        let path = temp_file("senza-dal-vivo");
        let mut value = serde_json::to_value(Settings {
            bitrate_kbps: 64,
            ..Settings::default()
        })
        .unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("trascrizioneDalVivo");
        // La modalità della Trascrizione dal vivo di prima dell'ADR-0015 si ignora.
        object.insert("modalitaDalVivo".into(), "mix".into());
        object.remove("copiaCome");
        object.remove("parlantiFile");
        object.remove("parlantiMix");
        object.remove("parlantiMicrofono");
        object.remove("parlantiSistema");
        object.remove("tema");
        object.remove("assistenti");
        std::fs::write(&path, value.to_string()).unwrap();
        let settings = Settings::load(&path).unwrap();
        assert_eq!(settings.bitrate_kbps, 64);
        assert!(!settings.trascrizione_dal_vivo);
        assert_eq!(settings.copia_come, CopiaCome::Testo);
        assert!(!settings.parlanti_file);
        assert!(
            !settings.parlanti_mix && !settings.parlanti_microfono && !settings.parlanti_sistema
        );
        assert_eq!(settings.raccolta, None);
        assert_eq!(settings.tema, Tema::Sistema);
        assert!(!settings.assistenti);
    }

    #[test]
    fn sistema_lascia_il_tema_a_windows_chiaro_e_scuro_lo_fissano() {
        assert_eq!(Tema::Sistema.theme(), None);
        assert_eq!(Tema::Chiaro.theme(), Some(tauri::Theme::Light));
        assert_eq!(Tema::Scuro.theme(), Some(tauri::Theme::Dark));
    }

    #[test]
    fn un_file_corrotto_o_non_valido_riporta_ai_predefiniti() {
        let path = temp_file("corrotto");
        let valid = serde_json::to_value(Settings::default()).unwrap();
        let invalid = [
            "{ non è json".to_string(),
            "[]".to_string(),
            // Un campo mancante, un valore fuori dall'elenco, un modello sconosciuto.
            r#"{"model": "nemotron-3.5-streaming-0.6b-q5km"}"#.to_string(),
            with(&valid, "bitrateKbps", 33.into()),
            with(&valid, "sampleRate", 44_100.into()),
            with(&valid, "speechLanguage", "JA".into()),
            with(&valid, "speechLanguage", "italiano".into()),
            with(&valid, "speechLanguage", serde_json::Value::Null),
        ];
        for content in invalid {
            std::fs::write(&path, &content).unwrap();
            assert_eq!(Settings::load(&path), Ok(Settings::default()), "{content}");
        }
        // Non UTF-8: corrotto, non illeggibile.
        std::fs::write(&path, b"\xff\xfe{").unwrap();
        assert_eq!(Settings::load(&path), Ok(Settings::default()));
    }

    #[test]
    fn un_modello_tolto_dal_catalogo_torna_al_predefinito_senza_perdere_il_resto() {
        let path = temp_file("modello-sconosciuto");
        let saved = Settings {
            model: "whisper-large-v3-turbo-q5km".into(),
            diarizer: Diarizer::Nemotron3,
            nemotron3_path: Some(r"D:\modelli\Nemotron-3-Diarization-BF16.gguf".into()),
            speech_language: SpeechLanguage::from("de"),
            bitrate_kbps: 64,
            ..Settings::default()
        };
        saved.save(&path).unwrap();
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace("whisper-large-v3-turbo-q5km", "modello-ritirato");
        std::fs::write(&path, content).unwrap();
        assert_eq!(
            Settings::load(&path),
            Ok(Settings {
                model: Settings::default().model,
                ..saved
            })
        );
    }

    fn with(value: &serde_json::Value, key: &str, field: serde_json::Value) -> String {
        let mut value = value.clone();
        value[key] = field;
        value.to_string()
    }

    #[test]
    fn le_impostazioni_salvate_si_rileggono() {
        let path = temp_file("salvate");
        let settings = Settings {
            recording_source: RecordingSource::Both,
            microphone: Some("Microfono USB".into()),
            output_device: Some("Cuffie".into()),
            model: "whisper-large-v3-turbo-q5km".into(),
            diarizer: Diarizer::Nemotron3,
            nemotron3_path: Some(r"D:\modelli\Nemotron-3-Diarization-BF16.gguf".into()),
            speech_language: SpeechLanguage::from("it"),
            bitrate_kbps: 128,
            channels: Channels::Stereo,
            sample_rate: 24_000,
            recordings_folder: Some(r"D:\Registrazioni".into()),
            interface_language: Some(Language::Pl),
            trascrizione_dal_vivo: true,
            copia_come: CopiaCome::Markdown,
            parlanti_file: true,
            parlanti_mix: true,
            parlanti_microfono: false,
            parlanti_sistema: true,
            raccolta: Some("Ferrara Quarzi".into()),
            tema: Tema::Scuro,
            assistenti: true,
            guadagno_microfono: 6,
            guadagno_sistema: -3,
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), Ok(settings.clone()));
        // Un secondo salvataggio sostituisce il primo.
        let again = Settings {
            speech_language: SpeechLanguage::auto(),
            // Senza raccolta.
            raccolta: Some(String::new()),
            ..settings
        };
        again.save(&path).unwrap();
        assert_eq!(Settings::load(&path), Ok(again));
    }

    #[test]
    fn un_file_illeggibile_da_i_predefiniti_e_l_errore() {
        // Una cartella al posto del file: esiste ma non si legge.
        let path = temp_file("illeggibile");
        std::fs::create_dir(&path).unwrap();
        let store = SettingsStore::load(path);
        assert_eq!(store.get(), Settings::default());
        assert!(matches!(
            store.loaded(),
            Err(AppError::UnreadableSettings(detail)) if detail.contains("settings.json")
        ));
        // Mancante o corrotto non è un errore: valgono i predefiniti e basta.
        let path = temp_file("corrotto-nello-store");
        std::fs::write(&path, "{ non è json").unwrap();
        assert_eq!(SettingsStore::load(path).loaded(), Ok(Settings::default()));
        assert_eq!(
            SettingsStore::load(temp_file("mancante-nello-store")).loaded(),
            Ok(Settings::default())
        );
    }

    /// Apre `path` in esclusiva, come un antivirus o OneDrive: finché il file resta aperto nessuno
    /// lo legge.
    fn lock(path: &Path) -> std::fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        std::fs::File::options()
            .read(true)
            .share_mode(0)
            .open(path)
            .unwrap()
    }

    #[test]
    fn un_file_bloccato_per_un_attimo_si_legge_lo_stesso() {
        let path = temp_file("bloccato-un-attimo");
        let saved = Settings {
            bitrate_kbps: 64,
            ..Settings::default()
        };
        saved.save(&path).unwrap();
        let locked = lock(&path);
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            drop(locked);
        });
        assert_eq!(Settings::load(&path), Ok(saved));
        release.join().unwrap();
    }

    #[test]
    fn un_file_non_letto_all_avvio_non_si_sovrascrive_con_i_predefiniti() {
        let path = temp_file("bloccato-all-avvio");
        let saved = Settings {
            bitrate_kbps: 64,
            interface_language: Some(Language::De),
            ..Settings::default()
        };
        saved.save(&path).unwrap();
        let locked = lock(&path);
        let store = SettingsStore::load(path.clone());
        assert_eq!(store.get(), Settings::default());
        // L'utente cambia un'impostazione partendo dai predefiniti.
        let changed = Settings {
            copia_come: CopiaCome::Markdown,
            ..Settings::default()
        };
        // Finché il file non si legge, non si salva.
        assert!(matches!(
            store.set(changed.clone()),
            Err(AppError::UnreadableSettings(_))
        ));
        drop(locked);
        assert_eq!(Settings::load(&path), Ok(saved.clone()));
        // Quando si legge, la modifica va sopra il file, non sopra i predefiniti.
        let merged = Settings {
            copia_come: CopiaCome::Markdown,
            ..saved
        };
        store.set(changed).unwrap();
        assert_eq!(store.get(), merged);
        assert_eq!(store.loaded(), Ok(merged.clone()));
        assert_eq!(Settings::load(&path), Ok(merged));
    }

    #[test]
    fn dopo_un_salvataggio_riuscito_l_errore_di_lettura_non_vale_piu() {
        let path = temp_file("illeggibile-poi-salvato");
        std::fs::create_dir(&path).unwrap();
        let store = SettingsStore::load(path.clone());
        std::fs::remove_dir(&path).unwrap();
        let saved = Settings {
            interface_language: Some(Language::De),
            ..Settings::default()
        };
        store.set(saved.clone()).unwrap();
        // Un nuovo `get_settings` (la webview ricaricata) riceve le impostazioni salvate.
        assert_eq!(store.loaded(), Ok(saved));
    }

    #[test]
    fn la_lingua_dell_interfaccia_di_default_e_quella_di_sistema_se_supportata_altrimenti_l_inglese()
     {
        let cases = [
            ("it-IT", Language::It),
            ("it-CH", Language::It),
            ("en-US", Language::En),
            ("fr-CA", Language::Fr),
            ("es-419", Language::Es),
            ("de_AT", Language::De),
            ("pl", Language::Pl),
            ("PL-pl", Language::Pl),
            ("pt-BR", Language::En),
            ("ja-JP", Language::En),
            // Un prefisso che non è il codice della lingua non vale.
            ("ita", Language::En),
            ("", Language::En),
        ];
        for (locale, expected) in cases {
            assert_eq!(Language::from_locale(locale), expected, "{locale}");
        }
    }

    #[test]
    fn il_guadagno_va_da_meno_12_a_piu_24_db_a_passi_di_3_e_manca_nei_file_di_prima() {
        let path = temp_file("guadagno");
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("guadagnoMicrofono");
        object.remove("guadagnoSistema");
        std::fs::write(&path, value.to_string()).unwrap();
        let settings = Settings::load(&path).unwrap();
        assert_eq!(
            (settings.guadagno_microfono, settings.guadagno_sistema),
            (0, 0)
        );
        for (microfono, sistema) in [(-12, 24), (6, -3)] {
            let valid = Settings {
                guadagno_microfono: microfono,
                guadagno_sistema: sistema,
                ..Settings::default()
            };
            assert!(valid.save(&path).is_ok(), "{microfono} {sistema}");
        }
        for (microfono, sistema) in [(-15, 0), (0, 27), (5, 0), (0, -1)] {
            let invalid = Settings {
                guadagno_microfono: microfono,
                guadagno_sistema: sistema,
                ..Settings::default()
            };
            assert!(invalid.save(&path).is_err(), "{microfono} {sistema}");
        }
    }

    #[test]
    fn non_si_salvano_impostazioni_non_valide() {
        let path = temp_file("rifiutate");
        let invalid = Settings {
            bitrate_kbps: 33,
            ..Settings::default()
        };
        assert!(invalid.save(&path).is_err());
        assert!(!path.exists());
    }
}
