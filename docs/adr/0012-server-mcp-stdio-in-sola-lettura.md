---
status: accepted
---

# Gli Assistenti leggono la Libreria da `sbobino.exe --mcp`, in stdio e in sola lettura

Claude Code, Claude Desktop e Codex interrogano la Libreria con un server MCP. Il server è lo stesso exe lanciato dall'Assistente con `--mcp`: parla in stdio, gira anche con Sbobino chiuso e muore con la sessione dell'Assistente. Il ramo `--mcp` parte prima di `tauri::Builder`, altrimenti `tauri-plugin-single-instance` passerebbe gli argomenti all'app aperta. L'exe resta senza console (`windows_subsystem = "windows"`): i client passano le pipe, quindi stdin e stdout funzionano e non lampeggia nessuna finestra.

Il server legge e basta:
- l'indice SQLite si apre con `SQLITE_OPEN_READ_ONLY`, una connessione per chiamata, senza mai crearlo, migrarlo o allinearlo (`sync`); se manca o ha un altro `user_version` l'errore chiede di aprire Sbobino;
- i Bini si leggono dal file e si chiudono subito, come fa il player, così correzioni e rinomine dell'app non li trovano bloccati;
- non ci sono strumenti che scrivono.

Risponde solo se in Impostazioni l'utente ha acceso "Consenti agli Assistenti di leggere la Libreria" (spento di default), perché il testo letto va ai server dell'Assistente: è un'uscita esplicita dal PC. Vede tutta la Libreria e nient'altro: i Bini si nominano con il percorso relativo alla Libreria, e un percorso assoluto o con `..` si rifiuta.

Alternative scartate:
- server HTTP su localhost dentro l'app aperta: funziona solo con l'app aperta, vuole porta e token, e Claude Desktop non lo raggiunge (i connettori remoti partono dal cloud di Anthropic);
- un `sbobino-mcp.exe` separato: un secondo binario da installare e da tenere alla stessa versione dello schema dell'indice;
- `sync` dal server prima di rispondere: due processi che scrivono lo stesso SQLite si bloccano a vicenda (il timeout di 5 s di rusqlite fa fallire il `sync` dell'app); sotto Claude Desktop (MSIX) le scritture in `%LOCALAPPDATA%` finiscono in un livello privato e l'indice divergerebbe; un indice tenuto aperto impedisce all'app di cancellarlo e ricostruirlo;
- strumenti che scrivono (correggere Frasi, rinominare, trascrivere): le regole di `Activity::write` e il lock di `bino::update` vivono nel processo dell'app, e servirebbe un coordinamento tra processi.

Conseguenze:
- un Bino spostato a mano con l'app chiusa risulta `binoNotFound` finché l'app non si riapre e riallinea l'indice;
- ogni chiamata va nel log, ma sotto Claude Desktop (MSIX) anche il log del server finisce nel livello privato di Claude e l'utente non lo trova in `%LOCALAPPDATA%\it.sbobino.desktop\logs`;
- la ricerca è quella della Libreria (FTS5, per parole): un Assistente riformula le domande da solo. La ricerca per significato è un progetto a sé.

Fatti e versioni in `docs/research/server-mcp.md`.
