//! Catalogo dei modelli (`models.json`), download con ripresa e verifica, eliminazione.
//!
//! Il core (`download`, `disk_state`, `delete`) non dipende da Tauri; `Models` lo collega agli
//! eventi `model-download-progress` e `model-state-changed`.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::sync::Notify;

use crate::engine::transcribe_cpp::TranscribeCpp;
use crate::error::AppError;
use crate::managers::loaded_model::{Lease, LoadedModel};

/// Un modello del catalogo.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(rename = "nome")]
    pub name: String,
    /// Fissato a una revision di Hugging Face, quindi immutabile.
    pub url: String,
    pub sha256: String,
    /// Byte; tutti i modelli stanno sotto i 4 GiB.
    pub size: u32,
    #[serde(rename = "modalita")]
    pub mode: Mode,
    #[serde(rename = "licenza")]
    pub license: String,
}

/// Come compare il testo: con i Parziali mentre la Frase è in corso, o a fine Frase.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum Mode {
    #[serde(rename = "stream")]
    Stream,
    #[serde(rename = "frase")]
    Phrase,
}

impl Model {
    /// Il nome del file, l'ultimo segmento dell'URL: il catalogo è la sua unica fonte.
    pub fn file_name(&self) -> &str {
        self.url.rsplit('/').next().unwrap_or(&self.url)
    }

    pub fn path(&self, dir: &Path) -> PathBuf {
        dir.join(self.file_name())
    }

    fn partial_path(&self, dir: &Path) -> PathBuf {
        dir.join(format!("{}.partial", self.file_name()))
    }
}

#[derive(serde::Deserialize)]
struct Catalog {
    #[serde(rename = "predefinito")]
    default: String,
    #[serde(rename = "modelli")]
    models: Vec<Model>,
}

static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    serde_json::from_str(include_str!("models.json")).expect("models.json non valido")
});

/// I modelli del catalogo, nell'ordine di `models.json`.
pub fn catalog() -> &'static [Model] {
    &CATALOG.models
}

/// Il modello consigliato e predefinito (Nemotron).
pub fn default_model() -> &'static Model {
    find(&CATALOG.default).expect("il modello predefinito è nel catalogo")
}

pub fn find(id: &str) -> Option<&'static Model> {
    catalog().iter().find(|m| m.id == id)
}

/// Lo stato di un modello per l'interfaccia.
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ModelState {
    NotDownloaded,
    /// Un download interrotto: il `.partial` resta e il prossimo download riprende da lì.
    Interrupted {
        percent: u8,
    },
    Downloading {
        percent: u8,
    },
    /// Controllo di dimensione e SHA-256 prima che il modello diventi utilizzabile.
    Verifying,
    Downloaded,
}

/// Scarica `model` in `dir`: scrive in `<file>.partial`, riprende con `Range` da un `.partial`
/// esistente, verifica dimensione e SHA-256 e infine rinomina il file. `on_state` riceve
/// `Downloading` (al massimo 10 volte al secondo) e `Verifying`.
///
/// - `cancel` notificato prima del rename, anche durante la verifica: cancella il `.partial`
///   e restituisce `Cancelled`;
/// - rete interrotta: conserva il `.partial` e restituisce `DownloadFailed`;
/// - SHA-256 o dimensione errati: cancella il `.partial` e restituisce `VerificationFailed`.
pub async fn download(
    client: &reqwest::Client,
    model: &Model,
    dir: &Path,
    cancel: &Notify,
    on_state: &mut (dyn FnMut(ModelState) + Send),
) -> Result<(), AppError> {
    let partial = model.partial_path(dir);
    let size = u64::from(model.size);
    std::fs::create_dir_all(dir).map_err(|e| unwritable(dir, &e))?;
    let mut offset = std::fs::metadata(&partial).map_or(0, |m| m.len());
    if offset > size {
        remove(&partial)?;
        offset = 0;
    }
    if offset < size {
        // Il trasferimento si abbandona al primo Annulla, anche mentre aspetta dati. Uscito dal
        // `select!`, il future e il file aperto sono già chiusi: il `.partial` si può cancellare.
        let fetched = tokio::select! {
            fetched = fetch(client, model, &partial, offset, on_state) => Some(fetched),
            () = cancel.notified() => None,
        };
        let Some(fetched) = fetched else {
            remove(&partial)?;
            return Err(AppError::Cancelled);
        };
        fetched?;
    }
    on_state(ModelState::Verifying);
    let path = partial.clone();
    let actual = tokio::task::spawn_blocking(move || sha256_hex(&path))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::DownloadFailed(format!("{}: {e}", partial.display())))?;
    // L'hash gira in un thread bloccante che tiene aperto il file: Annulla si guarda qui, a
    // verifica finita, senza aspettare (`biased` prova prima il permesso già depositato).
    let cancelled = tokio::select! {
        biased;
        () = cancel.notified() => true,
        () = std::future::ready(()) => false,
    };
    if cancelled {
        remove(&partial)?;
        return Err(AppError::Cancelled);
    }
    if actual != model.sha256 {
        log::warn!(
            "{}: SHA-256 {actual}, atteso {}",
            model.file_name(),
            model.sha256
        );
        remove(&partial)?;
        return Err(AppError::VerificationFailed(model.file_name().into()));
    }
    let path = model.path(dir);
    std::fs::rename(&partial, &path).map_err(|e| unwritable(&path, &e))
}

/// Scarica il resto del file da `offset` e lo accoda al `.partial`.
async fn fetch(
    client: &reqwest::Client,
    model: &Model,
    partial: &Path,
    mut offset: u64,
    on_state: &mut (dyn FnMut(ModelState) + Send),
) -> Result<(), AppError> {
    let failed = |e: reqwest::Error| AppError::DownloadFailed(e.to_string());
    let size = u64::from(model.size);
    let mut request = client.get(&model.url);
    if offset > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={offset}-"));
    }
    let mut response = request.send().await.map_err(failed)?;
    let status = response.status();
    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
        // Il server dice che il `.partial` è già oltre la fine: non è un punto da cui riprendere.
        remove(partial)?;
    }
    if !status.is_success() {
        return Err(AppError::DownloadFailed(format!("HTTP {status}")));
    }
    // Un 200 a una richiesta con `Range` è il file intero: si riparte da zero. Un 206 da un
    // offset sbagliato non si controlla qui: lo scarta la verifica dello SHA-256.
    if status != reqwest::StatusCode::PARTIAL_CONTENT {
        offset = 0;
    }
    let mut file = if offset > 0 {
        std::fs::OpenOptions::new().append(true).open(partial)
    } else {
        std::fs::File::create(partial)
    }
    .map_err(|e| unwritable(partial, &e))?;
    let mut reported: Option<(u8, Instant)> = None;
    loop {
        let now = percent(offset, size);
        if reported.is_none_or(|(p, at)| p != now && at.elapsed() >= PROGRESS_INTERVAL) {
            on_state(ModelState::Downloading { percent: now });
            reported = Some((now, Instant::now()));
        }
        let Some(chunk) = response.chunk().await.map_err(failed)? else {
            break;
        };
        offset += chunk.len() as u64;
        if offset > size {
            // Un server che manda più del previsto non riempie il disco.
            log::warn!("{}: più di {size} byte", model.file_name());
            drop(file);
            remove(partial)?;
            return Err(AppError::VerificationFailed(model.file_name().into()));
        }
        // ponytail: scrittura sincrona nel runtime async; i blocchi sono di pochi KB su disco locale.
        file.write_all(&chunk)
            .map_err(|e| unwritable(partial, &e))?;
    }
    if offset < size {
        return Err(AppError::DownloadFailed(format!(
            "connessione chiusa a {offset} byte su {size}"
        )));
    }
    Ok(())
}

/// Lo stato su disco: scaricato, interrotto (c'è il `.partial`) o assente.
pub fn disk_state(model: &Model, dir: &Path) -> ModelState {
    let size = u64::from(model.size);
    if std::fs::metadata(model.path(dir)).is_ok_and(|m| m.len() == size) {
        ModelState::Downloaded
    } else if let Ok(partial) = std::fs::metadata(model.partial_path(dir)) {
        ModelState::Interrupted {
            percent: percent(partial.len(), size),
        }
    } else {
        ModelState::NotDownloaded
    }
}

/// Elimina il modello scaricato e un eventuale `.partial`.
pub fn delete(model: &Model, dir: &Path) -> Result<(), AppError> {
    remove(&model.path(dir))?;
    remove(&model.partial_path(dir))
}

const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

fn percent(done: u64, size: u64) -> u8 {
    u8::try_from(done * 100 / size.max(1)).unwrap_or(100)
}

fn sha256_hex(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 16];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Rimuove un file; se non c'è va bene lo stesso.
fn remove(path: &Path) -> Result<(), AppError> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(AppError::Internal(format!(
            "eliminazione di {}: {e}",
            path.display()
        ))),
        _ => Ok(()),
    }
}

fn unwritable(path: &Path, e: &std::io::Error) -> AppError {
    AppError::UnwritableFolder(format!("{}: {e}", path.display()))
}

/// Un modello del catalogo con il suo stato, per Impostazioni → Trascrizione.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    /// Byte da scaricare.
    pub size: u32,
    pub mode: Mode,
    pub license: String,
    /// Consigliato e predefinito.
    pub recommended: bool,
    pub state: ModelState,
    /// L'errore dell'ultimo download, finché non se ne avvia un altro.
    pub error: Option<AppError>,
    /// Le lingue che il modello accetta come indicazione (`it`, `it-IT`…), lette dal modello
    /// caricato. `null` finché non è stato caricato almeno una volta.
    pub languages: Option<Vec<String>>,
    /// Il modello si sta caricando o lo usa una Trascrizione: non si elimina.
    pub in_use: bool,
}

/// Avanzamento del download di un modello, al massimo 10 volte al secondo.
#[derive(Debug, Clone, serde::Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub model_id: String,
    pub percent: u8,
}

/// Un modello ha cambiato stato: download avviato, in verifica, finito, fallito, eliminato.
#[derive(Debug, Clone, serde::Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct ModelStateChanged {
    pub model_id: String,
    pub state: ModelState,
    pub error: Option<AppError>,
}

enum Entry {
    Running {
        cancel: Arc<Notify>,
        state: ModelState,
    },
    Failed(AppError),
}

/// I download in corso e gli errori dell'ultimo download, per modello, e il modello tenuto
/// caricato tra una Trascrizione e l'altra. In `tauri::State`.
pub struct Models {
    dir: PathBuf,
    client: reqwest::Client,
    entries: Mutex<HashMap<&'static str, Entry>>,
    loaded: LoadedModel<TranscribeCpp>,
    /// Le lingue dei modelli caricati almeno una volta.
    languages: Mutex<HashMap<&'static str, Vec<String>>>,
}

impl Models {
    /// `dir` è `app_data_dir/models`.
    pub fn new(dir: PathBuf) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("sbobino/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(15))
            // Una connessione ferma diventa un'interruzione: il `.partial` resta.
            .read_timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Self {
            dir,
            client,
            entries: Mutex::default(),
            loaded: LoadedModel::default(),
            languages: Mutex::default(),
        })
    }

    pub fn list(&self) -> Vec<ModelInfo> {
        let in_use = self.loaded.in_use();
        let languages = lock(&self.languages);
        catalog()
            .iter()
            .map(|model| {
                let (state, error) = self.status(model);
                ModelInfo {
                    id: model.id.clone(),
                    name: model.name.clone(),
                    size: model.size,
                    mode: model.mode,
                    license: model.license.clone(),
                    recommended: model.id == CATALOG.default,
                    state,
                    error,
                    languages: languages.get(model.id.as_str()).cloned(),
                    in_use: in_use == Some(model.id.as_str()),
                }
            })
            .collect()
    }

    /// Prende in prestito il motore del modello `id()`, caricandolo se non è quello tenuto.
    /// Se il modello non è scaricato restituisce `AppError::ModelMissing` con il suo nome.
    /// Il modello risulta in uso fino a `release`.
    pub fn take(
        &self,
        app: &AppHandle,
        id: impl FnOnce() -> &'static str,
    ) -> Result<Lease<'_, TranscribeCpp>, AppError> {
        let lease = self.loaded.take(id, |id| self.load_engine(app, id));
        if let Ok(lease) = &lease {
            self.emit_state(app, known(lease.id())?);
        }
        lease
    }

    /// Un'altra istanza del modello di `lease`, fuori da quella tenuta: con gli Ingressi separati
    /// ogni Ingresso oltre il primo ne ha una sua per la durata della Registrazione, perché
    /// transcribe-cpp ammette un solo stream per modello. Si libera al drop; finché c'è il prestito
    /// il modello è in uso e non si elimina.
    pub fn load_instance(
        &self,
        app: &AppHandle,
        lease: &Lease<'_, TranscribeCpp>,
    ) -> Result<TranscribeCpp, AppError> {
        self.load_engine(app, lease.id())
    }

    fn load_engine(&self, app: &AppHandle, id: &'static str) -> Result<TranscribeCpp, AppError> {
        let model = known(id)?;
        if disk_state(model, &self.dir) != ModelState::Downloaded {
            return Err(AppError::ModelMissing(model.name.clone()));
        }
        // Durante il caricamento Elimina è già disabilitato.
        self.emit_state(app, model);
        let started = Instant::now();
        let loaded = TranscribeCpp::load(&model.path(&self.dir));
        log::info!("{id} caricato in {:?}", started.elapsed());
        let engine = loaded.inspect_err(|_| self.emit_state(app, model))?;
        lock(&self.languages).insert(id, engine.languages().to_vec());
        Ok(engine)
    }

    /// Restituisce il motore, che resta caricato, oppure lo scarta (`keep` falso) dopo un guasto.
    pub fn release(&self, app: &AppHandle, lease: Lease<'_, TranscribeCpp>, keep: bool) {
        let id = lease.id();
        if keep {
            drop(lease);
        } else {
            lease.discard();
        }
        if let Some(model) = find(id) {
            self.emit_state(app, model);
        }
    }

    /// Avvia il download in background; se è già in corso o il modello è scaricato non fa nulla.
    pub fn start_download(&self, app: &AppHandle, id: &str) -> Result<(), AppError> {
        let model = known(id)?;
        let cancel = Arc::new(Notify::new());
        {
            let mut entries = self.entries();
            if let Some(Entry::Running { .. }) = entries.get(id) {
                return Ok(());
            }
            let percent = match disk_state(model, &self.dir) {
                // Riscaricarlo vorrebbe dire sovrascrivere un file forse aperto da una Trascrizione.
                ModelState::Downloaded => return Ok(()),
                ModelState::Interrupted { percent } => percent,
                _ => 0,
            };
            entries.insert(
                &model.id,
                Entry::Running {
                    cancel: cancel.clone(),
                    state: ModelState::Downloading { percent },
                },
            );
        }
        self.emit_state(app, model);
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let models = app.state::<Self>();
            let result = download(&models.client, model, &models.dir, &cancel, &mut |state| {
                models.on_state(&app, model, state);
            })
            .await;
            let downloaded = result.is_ok();
            {
                let mut entries = models.entries();
                match result {
                    Ok(()) | Err(AppError::Cancelled) => entries.remove(model.id.as_str()),
                    Err(e) => {
                        log::warn!("download di {}: {e}", model.id);
                        entries.insert(&model.id, Entry::Failed(e))
                    }
                };
            }
            models.emit_state(&app, model);
            if downloaded {
                // Se è il modello scelto si carica subito, e le sue lingue diventano note.
                crate::managers::transcription::preload(&app);
            }
        });
        Ok(())
    }

    /// Annulla il download in corso. Restituisce `false` se non ce n'era uno.
    pub fn cancel(&self, id: &str) -> bool {
        match self.entries().get(id) {
            Some(Entry::Running { cancel, .. }) => {
                cancel.notify_one();
                true
            }
            _ => false,
        }
    }

    /// Elimina il modello scaricato o il `.partial`. Durante un download vale come Annulla.
    /// Un errore resta sulla riga del modello, come quelli del download.
    pub fn delete(&self, app: &AppHandle, id: &str) -> Result<(), AppError> {
        let model = known(id)?;
        let deleted = {
            // Sotto il lock: un download non parte mentre i file spariscono.
            let mut entries = self.entries();
            if let Some(Entry::Running { cancel, .. }) = entries.get(id) {
                cancel.notify_one();
                return Ok(());
            }
            // Il file di un modello caricato resta aperto: prima si scarica il motore.
            self.loaded.evict(id)?;
            let deleted = delete(model, &self.dir);
            match &deleted {
                Ok(()) => entries.remove(id),
                Err(e) => entries.insert(&model.id, Entry::Failed(e.clone())),
            };
            deleted
        };
        self.emit_state(app, model);
        deleted
    }

    fn on_state(&self, app: &AppHandle, model: &Model, state: ModelState) {
        if let Some(Entry::Running { state: current, .. }) =
            self.entries().get_mut(model.id.as_str())
        {
            *current = state.clone();
        }
        let emitted = match state {
            ModelState::Downloading { percent } => ModelDownloadProgress {
                model_id: model.id.clone(),
                percent,
            }
            .emit(app),
            _ => {
                self.emit_state(app, model);
                Ok(())
            }
        };
        if let Err(e) = emitted {
            log::warn!("avanzamento del download non emesso: {e}");
        }
    }

    fn emit_state(&self, app: &AppHandle, model: &Model) {
        let (state, error) = self.status(model);
        let changed = ModelStateChanged {
            model_id: model.id.clone(),
            state,
            error,
        };
        if let Err(e) = changed.emit(app) {
            log::warn!("stato del modello non emesso: {e}");
        }
    }

    fn status(&self, model: &Model) -> (ModelState, Option<AppError>) {
        match self.entries().get(model.id.as_str()) {
            Some(Entry::Running { state, .. }) => (state.clone(), None),
            Some(Entry::Failed(error)) => (disk_state(model, &self.dir), Some(error.clone())),
            None => (disk_state(model, &self.dir), None),
        }
    }

    fn entries(&self) -> MutexGuard<'_, HashMap<&'static str, Entry>> {
        lock(&self.entries)
    }
}

/// Le mappe restano coerenti anche se un panic altrove avvelena il lock.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn known(id: &str) -> Result<&'static Model, AppError> {
    find(id).ok_or_else(|| AppError::Internal(format!("modello sconosciuto: {id}")))
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    use super::*;

    #[test]
    fn il_catalogo_ha_i_tre_modelli_con_nemotron_predefinito() {
        let ids: Vec<_> = catalog().iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "nemotron-3.5-streaming-0.6b-q5km",
                "whisper-large-v3-turbo-q5km",
                "parakeet-tdt-0.6b-v3-q5km"
            ]
        );
        let nemotron = default_model();
        assert_eq!(nemotron.id, ids[0]);
        assert_eq!(nemotron.mode, Mode::Stream);
        assert_eq!(
            nemotron.file_name(),
            "nemotron-3.5-asr-streaming-0.6b-Q5_K_M.gguf"
        );
        for model in catalog() {
            // URL fissati a una revision (40 cifre esadecimali), non a `main`.
            let revision = model.url.split('/').nth_back(1).unwrap();
            assert_eq!(revision.len(), 40, "{}", model.url);
            assert_eq!(model.sha256.len(), 64, "{}", model.id);
        }
    }

    /// Cosa fa il server di prova con il corpo della risposta.
    #[derive(Clone, Copy)]
    enum Behavior {
        Full,
        /// A pezzi, con una pausa tra l'uno e l'altro.
        Slow,
        /// Chiude la connessione dopo `n` byte del corpo.
        CutAfter(usize),
        /// Manda `n` byte e poi resta fermo senza chiudere.
        StallAfter(usize),
        /// Risponde 200 con tutto il file anche a una richiesta con `Range`.
        IgnoreRange,
    }

    /// Un server HTTP/1.1 minimo sulla loopback. Registra l'header `Range` di ogni richiesta.
    struct Server {
        url: String,
        ranges: Arc<Mutex<Vec<Option<String>>>>,
    }

    fn serve(body: Vec<u8>, behavior: Behavior) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!(
            "http://{}/revision/modello.gguf",
            listener.local_addr().unwrap()
        );
        let ranges = Arc::new(Mutex::new(Vec::new()));
        let recorded = ranges.clone();
        let body = Arc::new(body);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (body, recorded) = (body.clone(), recorded.clone());
                std::thread::spawn(move || respond(stream, &body, behavior, &recorded));
            }
        });
        Server { url, ranges }
    }

    fn respond(
        mut stream: std::net::TcpStream,
        body: &[u8],
        behavior: Behavior,
        recorded: &Mutex<Vec<Option<String>>>,
    ) {
        let mut range = None;
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("range")
            {
                range = Some(value.trim().to_string());
            }
        }
        recorded.lock().unwrap().push(range.clone());
        let start = match (&range, behavior) {
            (_, Behavior::IgnoreRange) | (None, _) => None,
            (Some(r), _) => r
                .strip_prefix("bytes=")
                .and_then(|r| r.strip_suffix('-'))
                .and_then(|n| n.parse::<usize>().ok()),
        };
        let head = match start {
            Some(n) => format!(
                "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {n}-{}/{}\r\nConnection: close\r\n\r\n",
                body.len() - n,
                body.len() - 1,
                body.len()
            ),
            None => format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            ),
        };
        let body = &body[start.unwrap_or(0)..];
        let _ = stream.write_all(head.as_bytes());
        match behavior {
            Behavior::Full | Behavior::IgnoreRange => {
                let _ = stream.write_all(body);
            }
            Behavior::Slow => {
                for piece in body.chunks(body.len().div_ceil(10)) {
                    let _ = stream.write_all(piece);
                    let _ = stream.flush();
                    std::thread::sleep(Duration::from_millis(60));
                }
            }
            Behavior::CutAfter(n) => {
                let _ = stream.write_all(&body[..n]);
            }
            Behavior::StallAfter(n) => {
                let _ = stream.write_all(&body[..n]);
                let _ = stream.flush();
                std::thread::sleep(Duration::from_secs(60));
            }
        }
    }

    /// Un corpo non periodico, così un errore di offset cambia lo SHA-256.
    fn body(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i * 7 + i / 251) as u8).collect()
    }

    fn model_for(server: &Server, body: &[u8]) -> Model {
        Model {
            id: "prova".into(),
            name: "Prova".into(),
            url: server.url.clone(),
            sha256: hex(&Sha256::digest(body)),
            size: u32::try_from(body.len()).unwrap(),
            mode: Mode::Phrase,
            license: "MIT".into(),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sbobino-test-modelli-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Esegue `download` e raccoglie gli stati notificati.
    async fn run(
        model: &Model,
        dir: &Path,
        cancel: &Notify,
    ) -> (Result<(), AppError>, Vec<ModelState>) {
        let mut states = Vec::new();
        let result = download(&reqwest::Client::new(), model, dir, cancel, &mut |s| {
            states.push(s);
        })
        .await;
        (result, states)
    }

    fn percents(states: &[ModelState]) -> Vec<u8> {
        states
            .iter()
            .filter_map(|s| match s {
                ModelState::Downloading { percent } => Some(*percent),
                _ => None,
            })
            .collect()
    }

    #[tokio::test]
    async fn il_download_riporta_la_percentuale_poi_verifica_e_rende_il_modello_utilizzabile() {
        let bytes = body(300_000);
        let server = serve(bytes.clone(), Behavior::Slow);
        let model = model_for(&server, &bytes);
        let dir = temp_dir("completo");
        let started = std::time::Instant::now();
        let (result, states) = run(&model, &dir, &Notify::new()).await;
        result.unwrap();
        let percents = percents(&states);
        assert_eq!(percents.first(), Some(&0), "{states:?}");
        assert!(percents.len() > 2, "percentuali intermedie: {percents:?}");
        assert!(percents.is_sorted(), "{percents:?}");
        assert!(percents.iter().all(|p| *p <= 100));
        // Al massimo 10 notifiche al secondo, più la prima.
        let max = started.elapsed().as_millis() / 100 + 1;
        assert!(percents.len() as u128 <= max, "{percents:?} in {max}");
        assert_eq!(states.last(), Some(&ModelState::Verifying));
        assert_eq!(std::fs::read(model.path(&dir)).unwrap(), bytes);
        assert!(!model.partial_path(&dir).exists());
        assert_eq!(disk_state(&model, &dir), ModelState::Downloaded);
    }

    #[tokio::test]
    async fn un_download_interrotto_conserva_il_file_incompleto_e_riprende_con_range() {
        let bytes = body(100_000);
        let dir = temp_dir("ripresa");
        let cut = serve(bytes.clone(), Behavior::CutAfter(40_000));
        let model = model_for(&cut, &bytes);
        let (result, _) = run(&model, &dir, &Notify::new()).await;
        assert!(
            matches!(result, Err(AppError::DownloadFailed(_))),
            "{result:?}"
        );
        assert_eq!(
            std::fs::read(model.partial_path(&dir)).unwrap(),
            bytes[..40_000]
        );
        assert_eq!(
            disk_state(&model, &dir),
            ModelState::Interrupted { percent: 40 }
        );

        let resumed = serve(bytes.clone(), Behavior::Full);
        let model = Model {
            url: resumed.url.clone(),
            ..model
        };
        let (result, states) = run(&model, &dir, &Notify::new()).await;
        result.unwrap();
        assert_eq!(
            *resumed.ranges.lock().unwrap(),
            [Some("bytes=40000-".to_string())]
        );
        // La percentuale riparte da dove si era fermata.
        assert_eq!(
            states.first(),
            Some(&ModelState::Downloading { percent: 40 })
        );
        assert_eq!(std::fs::read(model.path(&dir)).unwrap(), bytes);
        assert_eq!(disk_state(&model, &dir), ModelState::Downloaded);
    }

    #[tokio::test]
    async fn se_il_server_ignora_range_si_riparte_da_zero() {
        let bytes = body(50_000);
        let server = serve(bytes.clone(), Behavior::IgnoreRange);
        let model = model_for(&server, &bytes);
        let dir = temp_dir("range-ignorato");
        std::fs::write(model.partial_path(&dir), &bytes[..10_000]).unwrap();
        run(&model, &dir, &Notify::new()).await.0.unwrap();
        assert_eq!(
            *server.ranges.lock().unwrap(),
            [Some("bytes=10000-".to_string())]
        );
        assert_eq!(std::fs::read(model.path(&dir)).unwrap(), bytes);
    }

    #[tokio::test]
    async fn uno_sha_errato_cancella_il_file_incompleto_e_il_modello_non_e_utilizzabile() {
        let bytes = body(50_000);
        let server = serve(bytes.clone(), Behavior::Full);
        let model = Model {
            sha256: "0".repeat(64),
            ..model_for(&server, &bytes)
        };
        let dir = temp_dir("sha-errato");
        let (result, _) = run(&model, &dir, &Notify::new()).await;
        assert!(
            matches!(result, Err(AppError::VerificationFailed(_))),
            "{result:?}"
        );
        assert!(!model.partial_path(&dir).exists());
        assert!(!model.path(&dir).exists());
        assert_eq!(disk_state(&model, &dir), ModelState::NotDownloaded);
    }

    #[tokio::test]
    async fn annulla_ferma_anche_un_trasferimento_bloccato_e_cancella_il_file_incompleto() {
        let bytes = body(100_000);
        let server = serve(bytes.clone(), Behavior::StallAfter(30_000));
        let model = model_for(&server, &bytes);
        let dir = temp_dir("annulla");
        let cancel = Notify::new();
        let partial = model.partial_path(&dir);
        let press = async {
            // Annulla dopo che i primi byte sono arrivati sul disco.
            while std::fs::metadata(&partial).map_or(0, |m| m.len()) < 30_000 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            cancel.notify_one();
        };
        let ((result, _), ()) = tokio::join!(run(&model, &dir, &cancel), press);
        assert_eq!(result, Err(AppError::Cancelled));
        assert!(!partial.exists());
        assert_eq!(disk_state(&model, &dir), ModelState::NotDownloaded);
    }

    #[tokio::test]
    async fn annulla_vale_anche_durante_la_verifica() {
        let bytes = body(20_000);
        let server = serve(bytes.clone(), Behavior::Full);
        let model = model_for(&server, &bytes);
        let dir = temp_dir("annulla-verifica");
        // Un `.partial` già completo passa subito alla verifica, senza richieste.
        std::fs::write(model.partial_path(&dir), &bytes).unwrap();
        let cancel = Notify::new();
        cancel.notify_one();
        let (result, states) = run(&model, &dir, &cancel).await;
        assert_eq!(result, Err(AppError::Cancelled));
        assert_eq!(states, [ModelState::Verifying]);
        assert!(server.ranges.lock().unwrap().is_empty());
        assert_eq!(disk_state(&model, &dir), ModelState::NotDownloaded);
    }

    #[tokio::test]
    async fn elimina_rimuove_il_modello_scaricato() {
        let bytes = body(20_000);
        let server = serve(bytes.clone(), Behavior::Full);
        let model = model_for(&server, &bytes);
        let dir = temp_dir("elimina");
        run(&model, &dir, &Notify::new()).await.0.unwrap();
        assert_eq!(disk_state(&model, &dir), ModelState::Downloaded);
        delete(&model, &dir).unwrap();
        assert!(!model.path(&dir).exists());
        assert_eq!(disk_state(&model, &dir), ModelState::NotDownloaded);
        // Eliminare un modello assente non è un errore.
        delete(&model, &dir).unwrap();
    }
}
