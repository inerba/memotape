# 05: Importa trascinando un file

**What to build:** un file audio, video o Bino trascinato da Esplora file sulla finestra si apre come Sorgente, esattamente come con Importa un file. Il drop passa dall'API di Tauri e il trascinamento di un Bino sulle Raccolte sparisce (ADR-0011).

**Blocked by:** —

**Status:** ready-for-agent

- [ ] `dragDropEnabled: true` in `tauri.conf.json`. Il frontend ascolta `getCurrentWebview().onDragDropEvent` (`enter`/`over`/`drop`/`leave`), senza comandi nuovi.
- [ ] Si toglie il trascinamento HTML5 da `AllBini`: `draggable`, `application/x-sbobino-bino`, lo stato `dragging`, gli handler `drag*` delle pillole e i loro stili (bordo tratteggiato, evidenziazione). Restano Sposta in… e il resto della Libreria.
- [ ] Velo a tutta finestra durante il passaggio, con un testo che dipende dai percorsi:
  - un solo file con un'estensione di Importa un file (lo specchio in TS di `SOURCE_EXTENSIONS`, test accanto): "Rilascia per aprire";
  - più di un elemento: rifiutato ("un file alla volta");
  - un'estensione non accettata o una cartella: rifiutato, con i formati accettati;
  - durante un'Attività: un `.bino` si accetta, un file audio o video si rifiuta ("non disponibile durante una Registrazione o una Trascrizione").
  Un rilascio rifiutato non fa nulla. Il velo sparisce a `leave` e al rilascio.
- [ ] Il rilascio accettato:
  - senza Attività: `openPath`, come Importa un file. Un file diventa la Sorgente con Trascrivi ▾, senza far partire nulla; un `.bino` si apre e, se sta fuori dalla Libreria, non si sposta;
  - durante un'Attività: un `.bino` si apre in consultazione (`browse`), come dalla Libreria;
  - con Impostazioni aperto: torna a `/` e apre il file, come `pendingBino`;
  - con il dialog di conferma di Trascrivi di nuovo aperto: velo assente, rilascio ignorato.
- [ ] Il file sparito o illeggibile al momento dell'apertura dà gli stessi errori di Importa un file.
- [ ] La logica di accettazione (percorsi, Attività → esito e testo del velo) è una funzione pura con il suo `*.test.ts`.
- [ ] Etichette nuove nelle sei lingue.
- [ ] Verifica a mano in `bun tauri dev`: rilascio di un `.wav`, di un `.mp4`, di un `.bino` dentro e fuori dalla Libreria, di un `.docx`, di una cartella, di due file, durante una Registrazione e con Impostazioni aperto. Il drop nativo non si simula via CDP: va provato trascinando da Esplora file.
- [ ] `AGENTS.md`: togli la frase sul drag and drop HTML5 che funziona grazie a `dragDropEnabled: false`, descrivi il drop e aggiungi un'Insidia: niente `draggable` né eventi `drag*` nel frontend (ADR-0011).
- [ ] `PRODUCT.md`: in V13 togli il trascinamento sulle Raccolte e aggiungi la storia del drop.
- [ ] Controlli verdi.

## Note per chi lo implementa
- Leggi prima `AGENTS.md`, `CONTEXT.md` e l'ADR-0011.
- Usa la skill `tdd` per la funzione pura del velo. Niente test sui componenti.
- Chiudi con la skill `code-review` in foreground e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. A fine lavoro aggiorna questo ticket (criteri spuntati, `Status: done`).
