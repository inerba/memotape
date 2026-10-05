# 03: L'app si chiama Memotape

**What to build:** l'app è Memotape in tutto ciò che si vede e sul disco:
- `productName` e titolo della finestra "Memotape";
- identifier `it.memotape.desktop` (il test dell'identifier del server MCP lo confronta con la config di Tauri);
- crate `memotape` e `memotape_lib`, package npm `memotape`, eseguibile `memotape.exe`, installer `Memotape_<versione>_x64-setup.exe` che installa in `%LOCALAPPDATA%\Memotape`;
- cartella nascosta `.memotape`, Libreria predefinita `Documenti\Memotape` (con i percorsi localizzati nelle sei lingue), log `Memotape.log`;
- chiavi `localStorage` `memotape.order` e `memotape.volume`, prefissi `memotape-test-…` per le cartelle temporanee dei test;
- in Impostazioni → Assistenti il server `memotape` con il percorso di `memotape.exe`, e gli errori del server MCP che dicono di aprire Memotape;
- "Memotape" al posto di "Sbobino" nei testi tradotti, in Informazioni, in PRODUCT.md (con l'origine del nome: "Memotape, memo + tape, la musicassetta delle note vocali") e in AGENTS.md.

L'estensione resta `.bino` e il documento si chiama ancora Bino fino al ticket 04. `%APPDATA%\sbobino` appartiene alla vecchia app e non si tocca.

A controlli verdi si spostano i dati del PC di sviluppo, una volta sola (ADR-0014):
- da questa sessione, quindi nel livello privato MSIX di Claude, `%APPDATA%\it.sbobino.desktop` (`settings.json` e `models\`, 1,8 GB) si rinomina in `it.memotape.desktop`, senza copiarla;
- `%LOCALAPPDATA%\it.sbobino.desktop` resta dov'è e la cancella l'utente;
- l'utente riceve un comando PowerShell di una riga, da lanciare fuori da Claude, che fa la stessa rinomina sul `%APPDATA%` reale se la cartella c'è;
- in `~/.codex/config.toml` `[mcp_servers.sbobino]` diventa `[mcp_servers.memotape]`, con `%LOCALAPPDATA%\Memotape\memotape.exe`.

Spec: `.scratch/rebrand-memotape/spec.md` (storie 1–5, 15–20, 22, 23, 34–37).

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Nessun file del repo dice Sbobino, tranne `.bino`/Bino, i documenti storici (ADR 0001–0013, `.scratch/sbobino*`, `docs/research`) e `sbobino-deps`
- [ ] `l_identifier_e_quello_di_tauri` è verde con `it.memotape.desktop`
- [ ] `bun tauri dev` apre la finestra "Memotape". Dopo lo spostamento dei dati trova modelli e impostazioni (Impostazioni → Trascrizione mostra i modelli scaricati), e una Registrazione nuova finisce in `Documenti\Memotape`
- [ ] Gli smoke test `cargo test -- --ignored --test-threads=1` trovano i modelli nella cartella nuova
- [ ] Il comando PowerShell per il `%APPDATA%` reale è consegnato all'utente e `~/.codex/config.toml` è aggiornato
- [ ] I sei controlli sono verdi
