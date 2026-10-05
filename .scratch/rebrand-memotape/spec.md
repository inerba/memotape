Status: ready-for-agent

# Rebrand: Sbobino diventa Memotape, il Bino diventa Tape

## Problem Statement

L'app si chiama Sbobino e il suo documento è il Bino (`.bino`). Il prodotto adesso si chiama Memotape e il documento Tape (`.tape`), ma il vecchio nome compare ovunque: nella finestra, nell'installer, in Esplora file, nelle cartelle sul disco, nell'eseguibile che gli Assistenti avviano, nel codice e nei documenti che gli agenti leggono. Se i due nomi convivono, l'utente vede un prodotto incoerente e chi lavora sul codice ricomincia a scrivere `bino` accanto a `tape`.

## Solution

Il rebrand è un taglio netto, perché l'app non è mai uscita (ADR-0014). Memotape non legge i `.bino` e non migra niente all'avvio. Tutto ciò che si vede (finestra, traduzioni, installer, Esplora file, server MCP) e tutto ciò che sta sotto (identifier, eseguibile, cartelle, protocollo, codice, documenti vivi) passa a Memotape e Tape. Il formato dentro lo zip non cambia: `version` resta 1, con `mix.ogg`, `trascrizione.json` e `forma-onda.json`. Basta quindi rinominare un `.bino` in `.tape`. I dati del PC di sviluppo li sposta una volta sola l'agente, nell'ultimo ticket.

## User Stories

### Nome dell'app

1. Come utente, voglio che la finestra, la barra delle applicazioni e il menu Start dicano "Memotape", così riconosco l'app con il suo nome.
2. Come utente, voglio che Informazioni mostri il nome Memotape con il simbolo della musicassetta, così so cosa ho installato.
3. Come utente, voglio che l'installer si chiami `Memotape_<versione>_x64-setup.exe`, installi in `%LOCALAPPDATA%\Memotape` e compaia in App installate come Memotape, così l'app è facile da trovare e da disinstallare.
4. Come utente, voglio che l'eseguibile sia `memotape.exe`, così lo riconosco in Gestione attività e nella configurazione degli Assistenti.
5. Come utente, voglio che il log si chiami `Memotape.log`, così lo trovo se devo mandarlo a qualcuno.

### Il Tape

6. Come utente, voglio che le Registrazioni e i file trascritti diventino `.tape`, così il file porta il nome del prodotto.
7. Come utente, voglio che Esplora file mostri un `.tape` come "Tape (Memotape)" con l'icona dell'app e che il doppio clic lo apra in Memotape, così come oggi con il Bino.
8. Come utente, voglio che un secondo doppio clic su un Tape con Memotape aperto porti il Tape alla finestra esistente, così non si apre una seconda finestra.
9. Come utente, voglio che la disinstallazione tolga l'associazione di `.tape` e la classe `Memotape.Tape`, così il sistema resta pulito.
10. Come utente, voglio che Importa un file, Sfoglia e il trascinamento accettino i `.tape` e dicano "Audio, video e Tape", così so cosa posso aprire.
11. Come utente, voglio che ogni testo dell'interfaccia, in tutte e sei le lingue, chiami il documento "Tape" (invariabile: "un Tape", "i Tape", "the Tape", "die Tape"…), così il nome è lo stesso ovunque.
12. Come utente, voglio che i messaggi d'errore dicano "Tape" ("Tape non trovato", "Tape di una versione più nuova"), così capisco di quale file si parla.
13. Come utente, voglio che il contatore della Libreria dica il numero dei Tape, così so quanti ne ho.
14. Come utente, voglio che un Tape continui a contenere audio del mix, audio degli Ingressi, Forma d'onda, Frasi, Parlanti, correzioni e data come il Bino, così il rebrand non mi toglie nulla.

### Cartelle sul disco

15. Come utente, voglio che la Libreria predefinita sia `Documenti\Memotape`, creata se manca, così i Tape nuovi finiscono nella cartella del prodotto.
16. Come utente, voglio che il percorso predefinito mostrato in Impostazioni usi il nome della cartella Documenti nella mia lingua (`Dokumente\Memotape`, `Documentos\Memotape`…), così leggo il percorso vero.
17. Come utente, voglio che l'audio di una Registrazione in corso stia nella cartella nascosta `.memotape` della Libreria e che la Libreria la ignori, così dopo un crash so dove trovarlo.
18. Come utente, voglio che impostazioni e modelli stiano in `%APPDATA%\it.memotape.desktop` e indici e log in `%LOCALAPPDATA%\it.memotape.desktop`, così Memotape non tocca le cartelle di altre app.
19. Come utente, voglio che `%APPDATA%\sbobino` della vecchia app omonima resti intatta, così non perdo i dati di quella.

### Assistenti

20. Come utente, voglio che Impostazioni → Assistenti mi dia i blocchi da copiare con il server `memotape` e il percorso di `memotape.exe`, così configuro Claude e Codex con il nome nuovo.
21. Come Assistente, voglio uno strumento `list_tapes`, con le descrizioni degli strumenti che parlano di Tape, così capisco cosa sto elencando.
22. Come Assistente, voglio che gli errori dicano di aprire Memotape (indice assente o di un'altra versione, interruttore spento), così so cosa suggerire all'utente.
23. Come utente, voglio che le richieste degli Assistenti lascino la loro riga nel log di Memotape, così vedo chi ha letto cosa.

### Chi lavora sul codice

24. Come sviluppatore, voglio che il codice usi `Tape` e `tape` dove usava `Bino` e `bino` (moduli, tipi, comandi, eventi, codici d'errore, chiavi i18n, componenti, file, test), così codice e glossario dicono la stessa cosa.
25. Come sviluppatore, voglio che negli identificatori il plurale sia `tapes` e non `tape`, così `tapes: Vec<Tape>` non è ambiguo (ADR-0014).
26. Come sviluppatore, voglio che AGENTS.md, PRODUCT.md, DESIGN.md, il contratto visivo in `.impeccable` e i commenti nel codice parlino di Memotape e Tape, così un agente non reintroduce il vecchio nome.
27. Come sviluppatore, voglio che le vecchie ADR, `.scratch/sbobino*` e `docs/research` restino come sono, con l'ADR-0014 a fare da traduzione, così lo storico resta lo storico.
28. Come sviluppatore, voglio che `bindings.ts` rigenerato e committato rifletta i nomi nuovi, così il test sui bindings resta verde.
29. Come sviluppatore, voglio che i file del marchio in `docs/brand` si chiamino `memotape-*` e che `LINEE-GUIDA.md` parli di Memotape, così il kit corrisponde al prodotto.
30. Come sviluppatore, voglio che, a lavoro finito, una ricerca di `sbobino`, `bino`, `bini` e `.bino` nel repo trovi solo i percorsi ammessi dall'ADR-0014, così so che la rinomina è completa.

### Dati del PC di sviluppo (una volta)

31. Come sviluppatore, voglio che i miei 6 Bini finiscano in `Documenti\Memotape` come `.tape`, mantenendo la Raccolta `Mediolanum`, così ritrovo il mio lavoro nell'app rinominata.
32. Come sviluppatore, voglio che i file della vecchia app restino in `Documenti\Sbobino`, così la Libreria nuova ha solo Tape.
33. Come sviluppatore, voglio che gli Ogg delle due Registrazioni interrotte passino da `.sbobino` a `Documenti\Memotape\.memotape`, così restano recuperabili.
34. Come sviluppatore, voglio che `settings.json` e i modelli (1,8 GB) passino a `it.memotape.desktop` con una rinomina della cartella e non con una copia, così non riscarico niente.
35. Come sviluppatore, voglio un comando PowerShell di una riga da lanciare fuori da Claude che faccia la stessa rinomina sul `%APPDATA%` reale, se la cartella c'è, così anche Memotape installato e aperto da Start trova modelli e impostazioni.
36. Come sviluppatore, voglio che `~/.codex/config.toml` abbia `[mcp_servers.memotape]` con il percorso di `memotape.exe`, così Codex trova il server dopo l'installazione.
37. Come sviluppatore, voglio che ogni dato si sposti appena l'app rinominata lo cerca nel posto nuovo e i sei controlli sono verdi (i dati dell'app con il nome, la Libreria con l'estensione), così l'app di sviluppo non resta mai senza modelli o Tape.

## Implementation Decisions

- **Taglio netto (ADR-0014).** Nessun codice di compatibilità: niente lettura dei `.bino`, niente migrazione all'avvio, niente pulizia del registro per la classe `Sbobino.Bino`. Lo schema del Tape è quello del Bino, con `version` 1 e gli stessi nomi delle voci nello zip.
- **Nomi tecnici:**
  - `productName` e titolo della finestra: "Memotape";
  - identifier `it.memotape.desktop` (il test dell'identifier del server MCP lo confronta con la config di Tauri);
  - crate `memotape`, libreria `memotape_lib`, package npm `memotape`, eseguibile `memotape.exe`;
  - associazione con classe `Memotape.Tape`, descrizione "Tape (Memotape)", estensione `tape`, e l'hook di disinstallazione aggiornato a `.tape`;
  - protocollo del player `tape` (`http://tape.localhost/…`);
  - cartella nascosta `.memotape`, Libreria predefinita `Documenti\Memotape`, log `Memotape.log` e il prefisso `[mcp]` invariato;
  - chiavi `localStorage` `memotape.order` e `memotape.volume`;
  - il mutex dell'istanza unica segue l'identifier da solo;
  - prefissi delle cartelle temporanee dei test `memotape-test-…`.
- **Codice Rust.** I moduli `bino` e `managers::pending_bino` diventano `tape` e `managers::pending_tape`. Tipi, funzioni, costanti e test passano da `Bino`/`bino`/`Bini`/`bini` a `Tape`/`tape`/`tapes`:
  - tipi `TapeInfo`, `OpenedTape`, `TapeEntry`, `PendingTape`, `FoundTape`, `TapeOut`;
  - costanti `TAPES_PER_RICERCA` e `FRASI_PER_TAPE`;
  - funzioni `tape_path`, `file_to_tape`, `write_tape`, `is_tape`, `from_args`. `from_args` cerca il primo `.tape`.
- **Comandi ed eventi (contratto IPC, rigenerato in `bindings.ts`):**
  - comandi `open_tape`, `tape_text`, `tape_peaks`, `rename_tape`, `move_tape`, `trash_tape`, `take_pending_tape`;
  - evento `tape-requested`;
  - codici d'errore `unsupportedTape` e `tapeNotFound`;
  - i campi delle risposte che oggi si chiamano `bini` diventano `tapes` (per esempio in `LibraryList`).
- **Indice della Libreria.** La tabella `bini` diventa `tapes`. `SCHEMA` sale di uno, così l'indice si ricostruisce. Il nome del file dell'indice dipende dal percorso della cartella e cambia comunque. `sync` cerca solo i `.tape`.
- **Server MCP.** Lo strumento `list_bini` diventa `list_tapes`. Descrizioni, errori e testi degli strumenti dicono "Tape" e "Memotape". I campi delle risposte strutturate che contengono `bini` diventano `tapes`. `assistantConfigs` dà il server con il nome `memotape` e il percorso di `memotape.exe`. Il log di Claude Desktop diventa `mcp-server-memotape.log`.
- **Frontend.** I componenti, gli hook e le funzioni pure passano a `Tape`:
  - componenti `TapePane`, `TapeMenu`, `TapeHeader`, `TapeEmpty`, `TapeRow`, `AllTapes`;
  - funzioni e hook `useTapeOperations`, `isTape`, `sortTapes`, `tapesOf`;
  - i file `bino-*` diventano `tape-*`;
  - `SOURCE_EXTENSIONS` contiene `tape`, nei due specchi.
- **Traduzioni.** Le chiavi `bino`, `binoNotFound` e `unsupportedBino` diventano `tape`, `tapeNotFound` e `unsupportedTape`. Nelle sei lingue ogni "Bino/Bini" diventa "Tape", invariabile; in italiano "il Tape", "i Tape". I percorsi predefiniti localizzati (`Documenti\Memotape`, `Dokumente\Memotape`, `Documentos\Memotape`, `Dokumenty\Memotape`…) cambiano. Le forme plurali di i18next non cambiano (`bino.parlanti` → `tape.parlanti`, con le stesse forme).
- **Marchio.** I file in `docs/brand` diventano `memotape-*`, e così i riferimenti in `LINEE-GUIDA.md` e nei commenti di `BrandMark`. Il simbolo e le icone in `src-tauri/icons` restano. I logotipi con "sbobino" disegnato a tracciati restano fuori dall'app fino al nuovo wordmark.
- **Documenti.** Si riscrivono AGENTS.md (con l'esempio di `.cargo/config.toml` che resta valido), PRODUCT.md (storia del nome: "Memotape, memo + tape, la musicassetta delle note vocali"; tipo "Tape (Memotape)"), DESIGN.md, i contratti in `.impeccable` e i commenti. Restano invariati le ADR da 0001 a 0013, `.scratch/sbobino*`, `docs/research` e i nomi `sbobino-deps` e cartella del repo. Il glossario è già aggiornato.
- **Lavoro.** Si fa sul branch `rebrand-memotape`, dopo aver committato il lavoro in corso (la Continuazione). Ogni dato si sposta a controlli verdi, nel ticket che cambia il posto in cui l'app lo cerca: i dati dell'app con il cambio di nome, la Libreria con il cambio di estensione.
  - Libreria: nuova `Documenti\Memotape`, dove si spostano solo i 6 `.bino` rinominati e gli Ogg di `.sbobino`;
  - dati: rinomina di `%APPDATA%\it.sbobino.desktop` in `it.memotape.desktop` nel livello privato di Claude. `%LOCALAPPDATA%\it.sbobino.desktop` resta, e la cancella l'utente;
  - comando PowerShell per il `%APPDATA%` reale;
  - modifica di `~/.codex/config.toml`.

## Testing Decisions

- Il rebrand non cambia il comportamento, quindi un buon test resta quello che c'è: si rinominano nomi e fixture, non le asserzioni. Nessun test nuovo, tranne dove un test controlla un nome che cambia (associazione, `from_args` con `.tape`, `SOURCE_EXTENSIONS`).
- Fanno da guardia, senza modifiche:
  - `i_bindings_committati_sono_aggiornati` (contratto IPC);
  - `l_identifier_e_quello_di_tauri` (identifier);
  - lo specchio di `SOURCE_EXTENSIONS` tra Rust e TS;
  - i test di `i18n` (chiavi e plurali nelle sei lingue);
  - `credits.test.ts`.
- Moduli toccati e già testati: `tape` (scrittura, lettura, riscrittura, correzioni contemporanee), `library` (Raccolte, `sync`, ricerca, operazioni, indice che si ricostruisce con lo `SCHEMA` nuovo), `mcp` (strumenti, errori, `Places`), `managers::recording` e `managers::transcription` (`file_to_tape`, nomi dei file), `player` (protocollo `tape`), più i `*.test.ts` del frontend.
- I sei controlli di AGENTS.md devono essere verdi a ogni ticket.
- Criterio di accettazione finale, senza test permanente: `git grep -i -E 'sbobino|\bbin[oi]\b|\.bino'` trova solo le ADR da 0001 a 0013, l'ADR-0014, `.scratch/sbobino*`, `docs/research`, l'_Avoid_ del glossario e i riferimenti a `sbobino-deps`.
- Verifica a mano nell'ultimo ticket:
  - l'app di sviluppo trova modelli, impostazioni e i 6 Tape, e li apre con il player;
  - Registra e Trascrivi creano `.tape` in `Documenti\Memotape`;
  - l'installer passa la "Verifica dell'installer" di AGENTS.md, con nome, cartella, associazione di `.tape` e disinstallazione pulita;
  - `memotape.exe --mcp` risponde a `tools/list` con `list_tapes`.

## Out of Scope

- Il nuovo wordmark "Memotape" (logotipo, versioni orizzontale e verticale): è un lavoro a parte con `/logo-design`.
- La lettura dei `.bino`, la migrazione automatica e la pulizia delle chiavi di registro di Sbobino.
- La rinomina della cartella del repo e di `..\sbobino-deps`.
- La riscrittura di ADR 0001–0013, `.scratch/sbobino*` e `docs/research`.
- La cancellazione dei dati vecchi (`%LOCALAPPDATA%\it.sbobino.desktop`, `Documenti\Sbobino`): la fa l'utente.
- L'editore dell'installer, che resta il segnaposto.

## Further Notes

- **Virtualizzazione MSIX.** Dentro l'app desktop di Claude, `%APPDATA%` e `%LOCALAPPDATA%` sono virtualizzati: oggi i dati di sviluppo (modelli, `settings.json`, indici, WebView) stanno nel livello privato `%LOCALAPPDATA%\Packages\Claude_…\LocalCache`. La rinomina fatta dall'agente vale lì, e il comando PowerShell copre il `%APPDATA%` reale.
- `Documenti\Sbobino` contiene anche centinaia di `.ogg` e `.txt` della vecchia app Sbobino: non vanno spostati.
- Claude Desktop e Claude Code non hanno il server MCP configurato. Codex sì, con un percorso di Sbobino installato che oggi non esiste.
