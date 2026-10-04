# 02: Ricerca

**What to build:** un campo di ricerca nella barra laterale trova le parole nei titoli, nel testo delle Frasi e nei nomi dei Parlanti dei Bini della Raccolta scelta. In fondo ai risultati c'è "Cerca in tutta la Libreria". Ogni risultato mostra il Bino e le Frasi trovate, con un estratto e la parola evidenziata. Il clic su una Frase apre il Bino, scorre fino a lei e la evidenzia. Tutto resta in locale: l'indice è quello del ticket 01, ricostruibile dai Bini.

**Blocked by:** 01

**Status:** done

- [x] Tabella virtuale FTS5 nel database della Libreria, con tokenizer `unicode61 remove_diacritics 2`:
  - una riga per Frase: Bino, id, Ingresso, `inizio_ms`, testo, nome mostrato del Parlante;
  - una riga per il titolo;
  - inserita, aggiornata e tolta insieme alla tabella dei Bini.
- [x] Ricerca:
  - la query si spezza in parole, ognuna cercata come prefisso (`parola*`), tutte in AND;
  - i caratteri speciali di FTS5 si neutralizzano;
  - ordinamento con `bm25`, estratti con `snippet()`;
  - risultati raggruppati per Bino con le Frasi trovate, con un limite di Frasi per Bino e di Bini.
- [x] Ambito: la Raccolta scelta, oppure Senza raccolta, oppure tutta la Libreria. In fondo ai risultati c'è "Cerca in tutta la Libreria".
- [x] Il clic su un risultato apre il Bino, scorre fino alla Frase e la evidenzia. Il clic sul titolo apre il Bino dall'inizio.
- [x] La rinomina di un Parlante aggiorna l'indice di quel Bino, quindi il nuovo nome si trova subito.
- [x] Test Rust:
  - maiuscole e accenti ignorati;
  - prefisso;
  - più parole in AND;
  - titolo;
  - nome rinominato di un Parlante;
  - ambito Raccolta contro tutta la Libreria;
  - estratto con la parola segnata;
  - caratteri speciali senza errori;
  - database ricostruito con gli stessi risultati.
- [x] Etichette nuove nelle sei lingue.
- [x] Verifica in `bun tauri dev` su una Libreria di prova con qualche decina di Bini.

## Note per chi lo implementa
- Leggi prima:
  - `AGENTS.md` (comandi, prerequisiti, Insidie);
  - `CONTEXT.md`, e usa i suoi termini: Libreria, Raccolta, Bino, Frase…;
  - gli ADR in `docs/adr/`, in particolare 0005, 0008 e 0009;
  - la spec `.scratch/sbobino-libreria/spec.md` e `decisioni.md` accanto.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri su cartelle temporanee con Bini veri, e logica pura del frontend con `bun test`. Niente test sui componenti.
- Verifica il comportamento in `bun tauri dev` via CDP (vedi "Pilotare l'app" in `AGENTS.md`), poi chiudi app e Vite. Una build Rust alla volta. Per le prove usa una Cartella della Libreria di prova fuori dal repo.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. A fine lavoro aggiorna `AGENTS.md`, `PRODUCT.md` (storie e "Stato", vedi le "Further Notes" della spec) e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.

## Esito (2026-10-04)
- Ordinamento per la somma dei `bm25` delle righe del Bino, non per il migliore: così "il Bino che parla di più dell'argomento viene prima" (storia 42).
- Più parole: tutte nella stessa riga (Frase o titolo), come in FTS5; una parola del titolo e una di una Frase non si trovano insieme.
- Si indicizzano solo i nomi dati ai Parlanti, non "Parlante N".
- Il clic su un risultato seleziona la Frase nell'area di testo di oggi; con la vista a turni (ticket 03) e il player (ticket 04) diventerà evidenziazione e posizione del player.
- Verifica in `bun tauri dev` su 40 Bini sintetici in 4 Raccolte: una ricerca in circa 4 ms, ambito e "Cerca in tutta la Libreria", estratti segnati, salto alla Frase con selezione e scorrimento, titolo dall'inizio, Parlante rinominato trovato subito. Non provata la consultazione durante un'Attività (stesso percorso di `browse`).
