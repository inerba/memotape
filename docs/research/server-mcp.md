# Server MCP della Libreria: rmcp, registrazione nei client, stdio su Windows, SQLite condiviso

Data delle verifiche: 2026-10-04. Serve alla progettazione di un server MCP in sola lettura che permetta a Claude
(Claude Code, Claude Desktop) e a OpenAI Codex di cercare nella Libreria dell'utente. Nessun codice scritto: solo fonti.

Legenda: **[F]** = verificato sulla fonte (documentazione ufficiale, sorgente del crate o del client); **[I]** = inferenza,
da confermare con una prova.

Versioni verificate oggi: `rmcp` 3.5.0 (2026-09-28), protocollo MCP 2026-07-28, Claude Code 2.1.289 (npm, 2026-10-03),
Codex CLI 0.160.0 (npm, 2026-10-01), `@anthropic-ai/mcpb` 2.1.2, `rusqlite` 0.40.2 (già nel progetto, SQLite 3.53.2),
`tokio` 1.53.1 (già nel progetto).

## 1. SDK Rust ufficiale: `rmcp`

Fonti: [crates.io/crates/rmcp](https://crates.io/crates/rmcp), [README del repo](https://github.com/modelcontextprotocol/rust-sdk),
[docs.rs/rmcp](https://docs.rs/rmcp), sorgente del pacchetto 3.5.0 scaricato da crates.io.

- **[F]** Ultima versione `rmcp` 3.5.0, pubblicata il 2026-09-28 (tag `rmcp-v3.5.0`), con `rmcp-macros` 3.5.0.
  Edition 2024, `rust-version = "1.88"`: il progetto (Rust ≥ 1.90 per `tauri` 2.12) è sopra.
- **[F]** Feature predefinite: `base64`, `macros`, `server`. `server` porta `transport-async-rw`, `schemars` (schemars **1.0**,
  nel lock del progetto c'è già 1.2.2), `uuid`.
- **[F]** Server stdio: feature `transport-io` (= `transport-async-rw` + `tokio/io-std`). `rmcp::transport::stdio()` restituisce
  la coppia `(tokio::io::stdin(), tokio::io::stdout())`. Dipendenza minima:
  `rmcp = { version = "3.5", features = ["server", "transport-io"] }` (`macros` è già nei default).
- **[F]** Server Streamable HTTP: feature `transport-streamable-http-server` (porta `server-side-http`, `transport-worker`,
  le sessioni). Espone `StreamableHttpService`, un servizio Tower da montare su un router (gli esempi usano `axum`, che
  `rmcp` non include, e serve `tokio/net` per il listener). Configurazione con `StreamableHttpServerConfig`:
  `with_json_response(true)` risponde con un solo `application/json` invece di uno stream SSE,
  `with_legacy_session_mode(false)` è stateless anche per i client vecchi. Di default `allowed_hosts` è
  `["localhost", "127.0.0.1", "::1"]` (controllo dell'header `Host` contro il DNS rebinding); la validazione di `Origin` è
  facoltativa (`allowed_origins`, `enforce_origin_validation`).
- **[F]** Richiede tokio: `rmcp` dipende da `tokio` 1 con `sync`, `macros`, `rt`, `time`, e il trasporto stdio da `io-std`.
  Il progetto ha già `tokio` (`macros`, `rt`, `sync`); `rt-multi-thread` non serve se si usa
  `#[tokio::main(flavor = "current_thread")]` **[I]**.
- **[F]** Dichiarazione dei tool con le macro:
  - `#[tool_router]` sull'`impl` che contiene i metodi, `#[tool(description = "...")]` su ogni metodo, e
    `#[tool_handler(name = "...", version = "...", instructions = "...")] impl ServerHandler for X {}`;
  - per un server di soli tool basta `#[tool_router(server_handler)]`, senza l'`impl ServerHandler`;
  - i parametri arrivano con `Parameters<T>` (`T: Deserialize + schemars::JsonSchema`): `inputSchema` viene dai campi di `T`
    e dalla loro documentazione (nome e doc del tipo sono ignorati);
  - un metodo può restituire `String`, `CallToolResult`, `Json<T>` o `Result<_, McpError>`; può essere `async`;
  - attributi di `#[tool]`: `name`, `title`, `description` (altrimenti la doc del metodo), `input_schema`, `output_schema`,
    `annotations(...)`, `icons`, `meta`, e `local` per handler non `Send`.

  Esempio del README (stdio):

  ```rust
  use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router, ServiceExt, transport::stdio};

  #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
  struct AddParams { a: i32, b: i32 }

  #[derive(Clone)]
  struct Calculator;

  #[tool_router(server_handler)]
  impl Calculator {
      #[tool(description = "Add two numbers")]
      fn add(&self, Parameters(AddParams { a, b }): Parameters<AddParams>) -> String {
          (a + b).to_string()
      }
  }

  #[tokio::main]
  async fn main() -> anyhow::Result<()> {
      let service = Calculator.serve(stdio()).await?;
      service.waiting().await?;
      Ok(())
  }
  ```

- **[F]** Protocollo: `ProtocolVersion::LATEST` = **2026-07-28**; `KNOWN_VERSIONS` = 2024-11-05, 2025-03-26, 2025-06-18,
  2025-11-25, 2026-07-28. La 2026-07-28 toglie l'handshake `initialize` (metadati in `_meta` di ogni richiesta, nuovo
  `server/discover`); `LATEST_WITH_INITIALIZE` = 2025-11-25. Il server risponde sia all'`initialize` dei client vecchi sia
  alla negoziazione per richiesta dei nuovi (`supported_protocol_versions` del `ServerHandler`).
  Changelog della spec: [modelcontextprotocol.io/specification/2026-07-28/changelog](https://modelcontextprotocol.io/specification/2026-07-28/changelog).
- **[F]** Annotazioni: `#[tool(annotations(title = "...", read_only_hint = true))]`; ci sono anche `destructive_hint`,
  `idempotent_hint`, `open_world_hint` (`ToolAnnotationsAttribute` in `rmcp-macros/src/tool.rs`; `read_only_hint` di default
  `false`). La spec: i client **MUST** considerare le annotazioni non affidabili se il server non è fidato
  ([server/tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)).
- **[F]** Output strutturato: restituire `Json<T>` (`T: Serialize + JsonSchema`) mette il valore in `structuredContent` e
  **anche** il JSON serializzato in un blocco di testo (`CallToolResult::structured`), come la spec raccomanda per
  compatibilità; la macro ricava `outputSchema` dal tipo di ritorno. Dalla 2026-07-28 `outputSchema` può essere qualunque
  schema e `structuredContent` qualunque valore JSON (SEP-2106).
- **[F]** Resources: si implementano `list_resources`, `read_resource` e facoltativamente `list_resource_templates` sul
  `ServerHandler`, con `enable_resources()` nelle capabilities di `get_info()`. Un tool può restituire `resource_link` o una
  risorsa incorporata. Claude Code mostra le risorse nel menu `@` (`@server:protocol://resource/path`) e dà a Claude i tool
  per elencarle e leggerle ([docs Claude Code](https://code.claude.com/docs/en/mcp), "Use MCP resources").
- **[F]** Nella 2026-07-28 la paginazione con `cursor`/`nextCursor` esiste solo per `tools/list`, `resources/list`,
  `resources/templates/list` e `prompts/list`, non per `tools/call`
  ([pagination](https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/pagination)): la paginazione
  dei risultati di una ricerca va fatta con parametri del tool.

## 2. Registrazione nei client

### Claude Code

Fonte: [code.claude.com/docs/en/mcp](https://code.claude.com/docs/en/mcp) (versione Markdown `…/mcp.md`).

- **[F]** Sintassi stdio: `claude mcp add [opzioni] <nome> -- <comando> [argomenti...]`. Tutto quello che segue `--` va al
  server senza modifiche; `--env KEY=value` (ripetibile) va prima del nome, con un'altra opzione tra `--env` e il nome.
- **[F]** HTTP: `claude mcp add --transport http <nome> <url>` (anche `--header`).
- **[F]** Scope (`-s`/`--scope`): `local` (predefinito, solo il progetto corrente, in `~/.claude.json` sotto il percorso del
  progetto), `project` (`.mcp.json` alla radice del progetto, da committare, chiede approvazione), `user` (tutti i progetti,
  in `~/.claude.json`). Per la Libreria personale ha senso `user`.
- **[F]** Altri comandi: `claude mcp add-json <nome> '<json>'`, `claude mcp add-from-claude-desktop`, `claude mcp list`,
  `claude mcp get`, `claude mcp remove`, `/mcp` nella sessione.
- **[F]** La nota "usa `cmd /c npx …` su Windows" non c'è più nella pagina di oggi. Serviva perché `npx` è uno shim `.cmd`
  che non si avvia direttamente; un `.exe` nativo non ne ha bisogno **[I]**.
- **[F]** Claude Code imposta `CLAUDE_PROJECT_DIR` nell'ambiente del server stdio.
- **[F]** Runtime v2 (SDK TypeScript 2.0): negozia la 2026-07-28 con i server HTTP e, da 2.1.285, con quelli stdio man mano
  che Anthropic lo attiva (`MCP_PROTOCOL_NEGOTIATION=auto|legacy` per decidere). Un server `rmcp` 3.5 regge entrambe.
- **[F]** Permessi: una regola di allow `mcp__sbobino__*` approva tutti i tool del server. La pagina dei permessi non dice
  che `readOnlyHint` cambi le richieste di conferma.

Esempi (PowerShell; il percorso dell'installazione è `%LOCALAPPDATA%\Sbobino`, l'argomento `--mcp` è solo un esempio):

```powershell
claude mcp add --scope user sbobino -- "$env:LOCALAPPDATA\Sbobino\sbobino.exe" --mcp
claude mcp add --transport http --scope user sbobino http://127.0.0.1:47800/mcp
```

Equivalente in `~/.claude.json` (scope `user`) o in `.mcp.json`:

```json
{
  "mcpServers": {
    "sbobino": {
      "type": "stdio",
      "command": "C:\\Users\\<utente>\\AppData\\Local\\Sbobino\\sbobino.exe",
      "args": ["--mcp"]
    }
  }
}
```

### Claude Desktop

Fonti: [Connect to local MCP servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers),
[Getting started with local MCP servers on Claude Desktop](https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop),
[Custom connectors using remote MCP](https://support.claude.com/en/articles/11175166-getting-started-with-custom-connectors-using-remote-mcp),
[MCPB `MANIFEST.md`](https://github.com/modelcontextprotocol/mcpb/blob/main/MANIFEST.md).

- **[F]** File: `%APPDATA%\Claude\claude_desktop_config.json` (Settings → Developer → Edit Config), formato
  `{"mcpServers": {"<nome>": {"command", "args", "env"}}}`, percorsi assoluti con `\\`. Si riavvia Claude Desktop dopo la
  modifica. Log in `%APPDATA%\Claude\logs`: `mcp.log` e `mcp-server-<nome>.log` con lo stderr del server.
- **[F]** Con l'installazione MSIX, che è quella dell'installer ufficiale, il file letto davvero è nella cartella virtualizzata
  `%LOCALAPPDATA%\Packages\Claude_<hash>\LocalCache\Roaming\Claude\claude_desktop_config.json`, e "Edit Config" ha aperto in
  passato il file sbagliato (issue [#26073](https://github.com/anthropics/claude-code/issues/26073),
  [#25579](https://github.com/anthropics/claude-code/issues/25579)). È lo stesso livello privato descritto in AGENTS.md
  ("`%APPDATA%` nell'app desktop di Claude").
- **[F]** Il file di configurazione è solo per server **locali stdio**. I connettori remoti (Settings → Connectors) sono
  contattati **dall'infrastruttura cloud di Anthropic**, non dal PC: un URL `http://127.0.0.1/...` non funziona come
  connettore. Per Claude Desktop l'unica via per la Libreria locale è quindi stdio (config o estensione).
- **[F]** Estensioni Desktop: oggi `.mcpb` (MCP Bundle, prima `.dxt`), uno zip con `manifest.json`; si installano con
  Settings → Extensions → Advanced settings → "Install Extension…". `server.type` può essere `binary` (eseguibile
  autonomo), con `mcp_config.command` che usa `${__dirname}` e `platform_overrides.win32`. Manifest spec 0.3/0.4,
  CLI `@anthropic-ai/mcpb` 2.1.2 (`mcpb pack`).

```json
{
  "mcpServers": {
    "sbobino": {
      "command": "C:\\Users\\<utente>\\AppData\\Local\\Sbobino\\sbobino.exe",
      "args": ["--mcp"]
    }
  }
}
```

### OpenAI Codex (CLI, estensione IDE, app desktop)

Fonti: [Codex MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) (era developers.openai.com/codex/mcp),
[config reference](https://learn.chatgpt.com/docs/config-file/config-reference), sorgente `openai/codex` su `main`.

- **[F]** Configurazione in `~/.codex/config.toml` (o `.codex/config.toml` di un progetto fidato), una tabella
  `[mcp_servers.<nome>]`. CLI, estensione IDE e app desktop **condividono** questa configurazione.
- **[F]** stdio: `command` (obbligatorio), `args`, `env`, `env_vars` (variabili da inoltrare), `cwd`.
  Streamable HTTP: `url` (obbligatorio), `bearer_token_env_var`, `http_headers`, `env_http_headers`, `auth`.
- **[F]** Comuni: `startup_timeout_sec` (predefinito 10), `tool_timeout_sec` (predefinito 60), `enabled`, `required`,
  `enabled_tools`, `disabled_tools`, `default_tools_approval_mode` (`auto`, `prompt`, `writes`, `approve`: con `writes`
  chiede conferma solo per i tool **non** marcati read-only), `tools.<tool>.approval_mode`, `tools.<tool>.output_token_limit`.
- **[F]** CLI: `codex mcp add <nome> --env VAR=VAL -- <comando>`, `codex mcp list`, `/mcp` nella TUI.
- **[F]** Codex legge il campo `instructions` del server: i primi 512 caratteri devono stare in piedi da soli.
- **[F]** Su Windows Codex passa al server stdio solo un elenco di variabili (`WINDOWS_CORE_ENV_VARS` in
  `codex-rs/protocol/src/shell_environment.rs`: `PATH`, `SYSTEMROOT`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, `TEMP`…),
  più `env` ed `env_vars`. `APPDATA` e `LOCALAPPDATA` ci sono.
- **[F]** `readOnlyHint` conta: Codex lo usa per la modalità `writes` e per eseguire in parallelo le chiamate ai tool
  read-only (`codex-rs/core/src/tools/handlers/mcp.rs`).
- **[F]** Se il risultato ha `structuredContent` non nullo, Codex passa al modello **il JSON serializzato al posto dei blocchi
  di testo** (`as_function_call_output_payload` in `codex-rs/protocol/src/models.rs`, test
  `preserves_structured_mcp_content`). Claude Code invece dà a Claude il `content` **[I]**. Con `Json<T>` i due client
  vedono quindi cose diverse.
- **[F]** In TOML un percorso Windows si scrive meglio come stringa letterale tra apici singoli (niente escape).

```toml
[mcp_servers.sbobino]
command = 'C:\Users\<utente>\AppData\Local\Sbobino\sbobino.exe'
args = ["--mcp"]
default_tools_approval_mode = "writes"

# in alternativa, Streamable HTTP sulla loopback
# [mcp_servers.sbobino]
# url = "http://127.0.0.1:47800/mcp"
```

## 3. stdio da un exe con `windows_subsystem = "windows"`

Fonti: [GetStdHandle](https://learn.microsoft.com/en-us/windows/console/getstdhandle),
[`std::io::Stdin`](https://doc.rust-lang.org/std/io/struct.Stdin.html) (Rust 1.99.0),
[tokio `Stdin`](https://docs.rs/tokio/latest/tokio/io/struct.Stdin.html).

- **[F]** Funziona se il genitore passa le pipe. Microsoft: con `/SUBSYSTEM:CONSOLE` il sistema riempie gli handle con una
  console "se il genitore non ha già riempito la tabella degli handle standard per ereditarietà"; con `/SUBSYSTEM:WINDOWS`
  non li riempie, ma quelli ereditati (`STARTF_USESTDHANDLES`) restano. Un client MCP crea sempre le pipe per stdin e stdout,
  quindi `GetStdHandle` restituisce le pipe e `std`/`tokio` le usano normalmente **[I]** (da provare con Claude Code e Codex).
- **[F]** Senza handle ereditati (avvio dal menu Start, da Esplora file, o da un genitore GUI che usa `inherit` invece delle
  pipe) gli handle sono NULL. In quel caso la `std` di Rust dice che `Read` e `Write` "non fanno nulla e riescono in silenzio":
  un server stdio leggerebbe subito 0 byte (EOF) e uscirebbe. Lo stesso guasto con Python e `windowsHide` è nell'issue
  [orcaslicer-mcp#3](https://github.com/MaxEllis/orcaslicer-mcp/issues/3).
- **[F]** La finestra di console che lampeggia è il problema opposto: un exe **console** avviato da un genitore che non usa
  `windowsHide`/`CREATE_NO_WINDOW` apre un `conhost` visibile. Claude Code e Desktop lo fanno con i server stdio su Windows
  (issue [#79219](https://github.com/anthropics/claude-code/issues/79219), chiusa "not planned", e
  [#97657](https://github.com/anthropics/claude-code/issues/97657)). Un exe GUI non ha console e non lampeggia: è il
  workaround citato in #79219.
- **[F]** Il rovescio: un exe GUI lanciato a mano da PowerShell o cmd non è attaccato alla console, quindi `--help` o i
  messaggi di prova non si vedono senza `AttachConsole(ATTACH_PARENT_PROCESS)`. In debug `src-tauri/src/main.rs` è
  comunque console (`cfg_attr(not(debug_assertions), windows_subsystem = "windows")`).
- **[F]** Regole stdio della spec ([stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio)):
  su stdout solo messaggi MCP, uno per riga e senza a capo interni; i log vanno su stderr (Claude Desktop li salva in
  `mcp-server-<nome>.log`); il server dovrebbe uscire appena stdin si chiude (EOF), che è il segnale di chiusura principale.
- **[F]** `tokio::io::stdin` legge con una lettura bloccante in un thread separato che non si può annullare: va bene con le
  pipe, perché il client chiude stdin e la lettura finisce.
- **[I]** Se il server MCP è `sbobino.exe --mcp`, il ramo va preso in `main` **prima** di `tauri::Builder`: il plugin
  single-instance passerebbe gli argomenti all'istanza aperta e uscirebbe, e non va aperta nessuna WebView. Un exe a parte
  (`sbobino-mcp.exe`, GUI, nel bundle) eviterebbe anche di caricare le DLL importate da `sbobino.exe` (ONNX Runtime,
  transcribe.cpp) a ogni avvio del server: da misurare.

## 4. Testi lunghi nei risultati dei tool

- **[F]** Claude Code: avviso oltre 10 000 token (soglia fissa), limite predefinito **25 000 token** (`MAX_MCP_OUTPUT_TOKENS`).
  Oltre il limite un risultato senza immagini viene salvato in un file in `tool-results` della sessione e nella
  conversazione resta un riferimento al file. Un tool può alzare la propria soglia con
  `_meta["anthropic/maxResultSizeChars"]` nella voce di `tools/list`, fino a 500 000 caratteri (in `rmcp` con l'attributo
  `meta` di `#[tool]`). La documentazione suggerisce agli autori di server anche di paginare.
  Altri tempi: `MCP_TIMEOUT` (avvio), `MCP_TOOL_TIMEOUT` (circa 28 ore se non impostato), timeout d'inattività 30 minuti per
  stdio, passaggio in background dopo 2 minuti ([docs](https://code.claude.com/docs/en/mcp), "MCP output limits and warnings").
- **[F]** Codex: troncamento per tool dato dal modello. Nel catalogo dei modelli del repo (`codex-rs/models-manager/models.json`)
  tutti i modelli hanno `truncation_policy = { mode = "tokens", limit = 10000 }`, più il 20% di "serialization allowance";
  il fallback per un modello sconosciuto è 10 000 byte. Si cambia con `tool_output_token_limit` (globale) o
  `mcp_servers.<id>.tools.<tool>.output_token_limit`. **[I]** Il catalogo può arrivare aggiornato dal server: il valore
  effettivo va controllato.
- **[F]** La spec non pagina `tools/call`; la paginazione dei risultati si fa con parametri del tool (per esempio `limit` e
  un `cursor` opaco restituito nel risultato), come per gli "handle" espliciti della sezione Stateful Tools della spec.
- **[F]** Linee guida di Anthropic ([Writing effective tools for agents](https://www.anthropic.com/engineering/writing-tools-for-agents),
  2025-09-11): paginazione, selezione di intervalli, filtri e/o troncamento "con valori predefiniti sensati"; un parametro
  `response_format` (`concise`/`detailed`); nomi leggibili al posto di id opachi; errori che dicono come correggere la
  chiamata. Gli errori di esecuzione vanno nel risultato con `isError: true`, non come errore JSON-RPC (spec, Tools →
  Error Handling).
- **[I]** Conseguenza: restare sotto circa 10 000 token per risposta copre entrambi i client senza configurazione. Per esempio
  la ricerca restituisce estratti (già `snippet()` e `FRASI_PER_BINO`) e un tool a parte legge un Bino a pezzi, per
  intervallo di tempo o di Frasi.

## 5. SQLite condiviso tra l'app e il server

Fonti: [File locking and concurrency (lockingv3)](https://www.sqlite.org/lockingv3.html),
[result codes](https://www.sqlite.org/rescode.html), [`sqlite3_busy_timeout`](https://www.sqlite.org/c3ref/busy_timeout.html),
`src/os_win.c` di SQLite, sorgente di `rusqlite` 0.40.2, `src-tauri/src/library.rs`.

- **[F]** Oggi l'indice (`app_local_data_dir\libreria\<hash>.sqlite`) è in modalità rollback journal predefinita (DELETE).
  L'app tiene una `Connection` aperta in lettura e scrittura, e `sync` scrive in una transazione unica.
- **[F]** Lock: i lettori prendono SHARED (quanti vogliono). Lo scrittore prende RESERVED (i lettori continuano), poi per
  scrivere sul file (al commit o quando la cache si riempie) PENDING (niente **nuovi** SHARED) ed EXCLUSIVE, che aspetta
  la fine di tutti gli SHARED. Quindi:
  - mentre l'app fa il commit di `sync`, il server riceve `SQLITE_BUSY` e il busy handler riprova;
  - mentre il server legge, il commit dell'app aspetta: se una lettura dura più del busy timeout dell'app, è **l'app** a
    ricevere `SQLITE_BUSY` e `sync` fallisce.
- **[F]** `rusqlite` imposta già `sqlite3_busy_timeout(db, 5000)` su ogni connessione aperta (`inner_connection.rs`). Anche
  la connessione dell'app ha quindi 5 s, senza averlo chiesto.
- **[F]** `OpenFlags::SQLITE_OPEN_READ_ONLY` (con `SQLITE_OPEN_NO_MUTEX`) non crea il file: se manca l'apertura fallisce
  (`SQLITE_CANTOPEN`). Con un **hot journal** (l'app è caduta a metà scrittura) il lettore dovrebbe fare il rollback, ma
  read-only non può: `SQLITE_READONLY_ROLLBACK` (776) finché l'app non riapre l'indice.
- **[F]** Su Windows SQLite apre il file con `FILE_SHARE_READ | FILE_SHARE_WRITE`, senza `FILE_SHARE_DELETE`
  (`os_win.c`). Finché il server ha l'indice aperto, il `remove_file` di `Library::open` (ricostruzione per versione diversa
  o file corrotto) fallisce, e l'app riapre lo stesso file **[I]** (da provare).
- Read-only + busy timeout basta, a queste condizioni:
  - **[F]** il server non deve passare da `connect`, che crea lo schema e su una versione diversa porta l'app a cancellare
    il file: deve aprire read-only e rifiutare un `user_version` diverso da `SCHEMA` con un errore leggibile ("aggiorna o apri
    Sbobino");
  - **[I]** letture brevi: una query per chiamata, risultati raccolti in memoria e statement chiusi subito, nessuna
    transazione di lettura aperta tra una chiamata e l'altra. Aprire la connessione a ogni chiamata è il modo più semplice
    per non tenere lock e handle (e libera il file per la ricostruzione);
  - **[I]** `immutable=1` va evitato: SQLite non prenderebbe lock e leggerebbe un file che l'app sta cambiando.
- **[F]** In WAL i lettori non bloccano lo scrittore. Però un lettore read-only ha bisogno del `-shm`
  (`SQLITE_READONLY_RECOVERY` se serve un recovery), e con il livello privato MSIX di Claude Desktop **[I]** un `-shm`
  creato dal server non sarebbe quello dell'app. Con il rollback journal i lock sono byte-range sul file stesso, che il
  processo vede: per ora meglio restare così.
