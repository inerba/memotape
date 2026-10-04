# 01: Libreria e barra laterale

**What to build:** la finestra ha una barra laterale sempre visibile che mostra la Libreria. Dall'alto:
- Registra e Apri file (audio, video o Bino);
- "Attività in corso", solo quando ce n'è una;
- il selettore della Raccolta (Tutta la Libreria, Senza raccolta, le Raccolte, Nuova Raccolta);
- i Bini della Raccolta raggruppati per data, con titolo, ora e durata;
- in fondo "Tutti i Bini della Raccolta" e Impostazioni.

La Libreria è la cartella che prima era la Cartella predefinita (ADR-0008). Le Raccolte sono le sue cartelle di primo livello, e quello che l'utente fa in Esplora file si vede nell'app. Dall'app:
- le Raccolte si creano, si rinominano e si eliminano se vuote;
- i Bini si rinominano, si spostano, si mandano nel Cestino e si aggiungono alla Libreria;
- una Registrazione finisce nella Raccolta scelta.

Il testo di un Bino si mostra ancora nell'area di oggi: la nuova vista arriva con il ticket 03.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Modulo Libreria in Rust, senza Tauri. Usa un indice SQLite (`rusqlite` 0.40 con `bundled`) in `%LOCALAPPDATA%\it.sbobino.desktop`, un file per Cartella della Libreria. Il modulo:
  - elenca Raccolte e Bini;
  - si allinea alla cartella all'avvio, quando la finestra torna in primo piano e prima di aprire un Bino, rileggendo solo i Bini nuovi o con data o dimensione cambiate;
  - ricostruisce il database se manca, è corrotto o ha un'altra `user_version`.
- [x] Le Raccolte sono le cartelle di primo livello, escluse quelle che iniziano con `.`. I Bini più in profondità contano nella Raccolta che li contiene; quelli nella radice sono "Senza raccolta". Un Bino illeggibile o di versione futura resta nell'elenco con il nome del file.
- [x] Impostazioni → Generale ha "Cartella della Libreria"; il campo `recordings_folder` resta in `settings.json`. Il campo nuovo `raccolta` (`#[serde(default)]`) ricorda la scelta del selettore. Una Raccolta che non esiste più vale come Tutta la Libreria.
- [x] Barra laterale come sopra:
  - il Bino aperto è evidenziato;
  - i gruppi sono Oggi, Ieri, Questa settimana e poi per mese;
  - "Attività in corso" mostra timer o percentuale e riporta alla vista dell'Attività.

  Impostazioni resta un livello sopra la finestra, che resta montata.
- [x] "Tutti i Bini della Raccolta" apre l'elenco completo nell'area principale, ordinabile per data o per titolo, con Sposta in… ed Elimina.
- [x] Raccolte: crea; rinomina, che rinomina anche la cartella; elimina, solo se vuota (`raccoltaNotEmpty`). I nomi sono validati per Windows (`invalidName`, `nameTaken`), con la stessa validazione ripetuta nel frontend.
- [x] Bino:
  - un clic sul titolo lo rinomina, cioè rinomina il file, senza mai toccare l'estensione;
  - Sposta in…;
  - Elimina, con conferma, lo manda nel Cestino (`SHFileOperationW` con `FOF_ALLOWUNDO`, via `windows-sys`);
  - Mostra in Esplora file;
  - un Bino fuori dalla Libreria si apre come oggi e mostra "Aggiungi alla Libreria…", che chiede la Raccolta e lo sposta.
- [x] `record` riceve la Raccolta:
  - il Bino si scrive lì, o nella radice con Tutta la Libreria o Senza raccolta;
  - l'Ogg temporaneo resta in `.sbobino`, nella radice;
  - a Stop il Bino compare nella barra laterale e resta aperto.
- [x] Durante un'Attività gli altri Bini si aprono e si leggono. Le scritture sono rifiutate solo sul Bino su cui lavora l'Attività in corso.
- [x] Il backend emette l'evento `library-changed` dopo ogni allineamento e operazione, e il frontend rilegge l'elenco.
- [x] Test Rust:
  - Raccolte e Senza raccolta;
  - modifiche fatte "da Esplora file" recepite;
  - rilettura dei soli Bini cambiati;
  - database ricostruito;
  - operazioni su Raccolte e Bini con nomi non validi o già usati;
  - Cestino;
  - Bino della Registrazione nella Raccolta.
- [x] Test frontend: raggruppamento per data (a mezzanotte, a inizio settimana, al cambio di mese), formato di ora e durata, validazione dei nomi.
- [x] Etichette nuove nelle sei lingue.
- [x] Verifica in `bun tauri dev`.

## Note per chi lo implementa
- Leggi prima:
  - `AGENTS.md` (comandi, prerequisiti, Insidie);
  - `CONTEXT.md`, e usa i suoi termini: Libreria, Raccolta, Bino, Frase…;
  - gli ADR in `docs/adr/`, in particolare 0005, 0008 e 0009;
  - la spec `.scratch/sbobino-libreria/spec.md` e `decisioni.md` accanto.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri su cartelle temporanee con Bini veri (motore e VAD finti dove serve), e logica pura del frontend con `bun test`. Niente test sui componenti.
- Verifica il comportamento in `bun tauri dev` via CDP (vedi "Pilotare l'app" in `AGENTS.md`), poi chiudi app e Vite. Una build Rust alla volta. Per le prove usa una Cartella della Libreria di prova fuori dal repo.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. A fine lavoro aggiorna `AGENTS.md`, `PRODUCT.md` (storie e "Stato", vedi le "Further Notes" della spec) e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
