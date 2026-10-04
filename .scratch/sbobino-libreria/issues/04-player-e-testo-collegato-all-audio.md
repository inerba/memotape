# 04: Player e testo collegato all'audio

**What to build:** in fondo alla vista di un Bino c'è un player fisso che riproduce il mix, anche con gli Ingressi separati. Ha Play/Pausa, posizione, durata, barra di avanzamento, indietro e avanti di 10 s e la velocità; Spazio fa Play/Pausa.

Il testo segue l'audio:
- durante l'ascolto la Frase in riproduzione si evidenzia e resta visibile;
- se l'utente scorre da solo, il testo resta dov'è e compare "Segui l'audio";
- accanto a ogni Frase un pulsante con il tempo porta il player all'inizio della Frase, senza cambiare lo stato di Play/Pausa;
- il clic sul testo resta solo per correggerlo;
- anche un risultato di ricerca porta il player alla Frase.

**Blocked by:** 03; anche 02, per il salto dai risultati di ricerca, se è già fatto.

**Status:** done

- [x] Finestra sul mix, una funzione pura:
  - riceve una richiesta `Range` (assente, `a-b`, `a-`, `-n` oppure oltre la fine) più offset e lunghezza della voce `mix.ogg`;
  - restituisce lo stato (200/206/416), le intestazioni (`Content-Range`, `Content-Length`, `Accept-Ranges: bytes`, `Content-Type: audio/ogg`) e i byte da leggere;
  - ogni risposta parziale è limitata a 1 MB. Senza `Range` (o con una non capita) risponde 200 con il mix intero, come il protocollo `asset`: `<audio>` manda sempre `Range`.
- [x] Protocollo personalizzato con `register_asynchronous_uri_scheme_protocol`, sul modello di `examples/streaming` di Tauri (senza `http-range`: un intervallo solo si legge in poche righe):
  - serve solo il `mix.ogg` di un `.bino`;
  - apre il file a ogni richiesta e non lo tiene aperto, così correzioni, rinomina e Cestino non lo trovano bloccato.
- [x] Player con `<audio>`:
  - Play/Pausa;
  - posizione e durata (`m:ss` / `h:mm:ss`);
  - barra di avanzamento;
  - indietro e avanti di 10 s;
  - velocità 1×, 1,25×, 1,5× e 2×;
  - Spazio, quando il focus non è in un campo di testo.

  È disabilitato durante una Registrazione e si ferma quando si apre un altro Bino. I file non trascritti non hanno player.
- [x] Evidenziazione:
  - la Frase che contiene la posizione, `fine_ms` esclusa;
  - nel silenzio, la Frase precedente;
  - con Frasi sovrapposte (Ingressi separati), tutte, e lo scorrimento segue quella iniziata prima;
  - si aggiorna anche spostando la barra.
- [x] "Segui l'audio", una macchina a stati `following`/`free`:
  - lo scorrimento manuale passa a `free` e mostra il comando. Si riconosce da `wheel`, dai tasti e dal trascinamento della barra di scorrimento, non dall'evento `scroll`;
  - il comando, il pulsante del tempo e la barra del player tornano a `following`;
  - durante la correzione di una Frase lo scorrimento automatico è sospeso.
- [x] Pulsante del tempo accanto a ogni Frase:
  - si vede al passaggio del mouse, e sempre per la Frase in riproduzione e all'inizio di ogni turno;
  - si raggiunge con Tab;
  - porta `currentTime` a `inizio_ms` senza cambiare Play/Pausa;
  - i tempi non entrano in Copia testo né nel Markdown.
- [x] Il clic su un risultato di ricerca (ticket 02) porta anche il player alla Frase, in pausa.
- [x] Test Rust: le richieste `Range` alla finestra danno gli stessi byte di `mix.ogg` estratto.
- [x] Test frontend:
  - Frasi da evidenziare: posizione dentro una Frase, sul confine, nel silenzio, prima della prima, dopo l'ultima, con Frasi sovrapposte;
  - la sequenza di "Segui l'audio";
  - il formato dei tempi.
- [x] Etichette nuove nelle sei lingue.
- [x] Verifica in `bun tauri dev` via CDP:
  - riproduzione e spostamento su Bini veri a 16 e 48 kHz, mono e stereo, misurando la precisione dello spostamento;
  - salto a una Frase in pausa e in riproduzione;
  - scorrimento automatico e "Segui l'audio";
  - correzione e rinomina mentre il player suona.
- [x] Riporta in `docs/research/` le verifiche su WebView2, protocollo e rusqlite delle "Further Notes" della spec, con l'esito delle misure.
  Esito in `docs/research/libreria-e-player.md`, "Player": spostamento entro 2 ms a 16 e 48 kHz, mono e stereo.

## Note per chi lo implementa
- Leggi prima:
  - `AGENTS.md` (comandi, prerequisiti, Insidie);
  - `CONTEXT.md`, e usa i suoi termini: Libreria, Raccolta, Bino, Frase…;
  - gli ADR in `docs/adr/`, in particolare 0005, 0008 e 0009;
  - la spec `.scratch/sbobino-libreria/spec.md` e `decisioni.md` accanto.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri e logica pura del frontend con `bun test`. Niente test sui componenti.
- Verifica il comportamento in `bun tauri dev` via CDP (vedi "Pilotare l'app" in `AGENTS.md`), poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. A fine lavoro aggiorna `AGENTS.md`, `PRODUCT.md` (storie e "Stato", vedi le "Further Notes" della spec) e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
