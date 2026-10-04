# 03: Vista di un Bino e Trascrizione di un file in un Bino

**What to build:** selezionare un Bino mostra nell'area principale:
- il titolo;
- una riga di informazioni: data e ora, durata, Raccolta, modello, Lingua del parlato, Ingressi separati, incompleto;
- la trascrizione a turni, con l'etichetta del Parlante o dell'Ingresso a ogni turno.

Inoltre:
- il testo si corregge Frase per Frase, sul posto, e la correzione si salva nel Bino senza toccare i tempi;
- le opzioni di Trascrizione e di Registrazione passano nei menu dei pulsanti Trascrivi ▾ e Registra ▾;
- il Markdown non si salva più da solo: c'è "Esporta Markdown…";
- Trascrivi su un file audio o video crea un Bino nella Raccolta scelta, con l'audio in Opus e il testo con i tempi (ADR-0009). Il Bino diventa la Sorgente.

**Blocked by:** 01

**Status:** done

- [x] Vista di un Bino come nella spec:
  - titolo, rinominabile con la funzione del ticket 01;
  - informazioni;
  - azioni: Trascrivi ▾, Copia testo, Esporta Markdown…, Mostra in Esplora file, e "…" con Sposta in… ed Elimina;
  - Parlanti, con la rinomina di oggi;
  - trascrizione a turni, che scorre.

  La textarea libera e lo stato `edited` spariscono.
- [x] Correzione per Frase:
  - un clic sul testo lo rende modificabile;
  - Invio, o l'uscita dalla Frase, salva nel Bino con `bino::rewrite` (tempi, Parlanti, `parlanti` e audio restano invariati) e aggiorna l'indice;
  - Esc ripristina il testo;
  - una Frase svuotata resta;
  - un errore di scrittura lascia il testo modificato e lo dice nella status bar;
  - il testo non si corregge durante l'Attività che lo sta producendo.
- [x] Misura il tempo di `bino::rewrite` su un Bino di un'ora. Se supera qualche centinaio di ms, il salvataggio va in background e la Frase mostra lo stato. Misurato: 45 ms in release (130 in debug) su 14 MB di mix e 600 Frasi, quindi niente background.
- [x] Menu Trascrivi ▾ (modello, Lingua del parlato, Riconosci i parlanti) e Registra ▾ (Trascrivi dal vivo, Riconosci i parlanti della Registrazione), sui campi delle impostazioni che esistono già. Il pulsante Trascrivi dice la scelta corrente ("Trascrivi · Italiano").
- [x] Markdown e Copia testo:
  - Copia testo usa sempre le Frasi, con le correzioni e i nomi dei Parlanti;
  - "Esporta Markdown…" salva il render del Bino aperto con il dialog di sistema, proponendo `<titolo>.md`;
  - `rename_parlante` e la Trascrizione non scrivono più Markdown, quindi `md` e `latest_md` escono da `LastTranscript`;
  - resta il caso `keep_ogg` della Registrazione: se il Bino non si scrive, il Markdown si salva accanto all'Ogg.
- [x] `transcribe` riceve la Raccolta. Su un file audio o video:
  - l'audio originale passa per il ricampionamento e per `OggOpusWriter` con le impostazioni di Registrazione, in un Ogg temporaneo in `.sbobino`;
  - a Trascrizione finita, Diarizzazione compresa, diventa `<nome del file>.bino` nella Raccolta (con " 2", " 3"… se esiste già), con `creato` = inizio della Trascrizione e il campo nuovo `origine`;
  - se la Trascrizione è annullata, guasta o senza parlato, non restano né Bino né temporaneo;
  - il risultato porta il Bino, che diventa la Sorgente.
- [x] Su un Bino, la conferma di Trascrivi avvisa anche che le correzioni si perdono.
- [x] Il documento ha il campo facoltativo `origine`, mostrato nelle informazioni; `version` resta 1.
- [x] Test Rust:
  - una correzione cambia solo quel testo;
  - `origine` si scrive e si rilegge, e un Bino senza `origine` si legge come prima;
  - con il motore finto, `parlato-it.mp4` e `parlato-it.wav` danno un Bino nella cartella indicata: `mix.ogg` ha la durata dell'originale (tolleranza da misurare), la frequenza e i canali chiesti, e le Frasi hanno i tempi;
  - annullando, o senza parlato, nessun file resta.
- [x] Test frontend: `withTesto` sulla `Conversation`, raggruppamento in turni.
- [x] Etichette nuove nelle sei lingue.
- [x] Verifica in `bun tauri dev`:
  - una correzione salvata si ritrova riaprendo il Bino;
  - Trascrizione di una copia di `parlato-it.mp4` in una Raccolta;
  - Esporta Markdown….

## Note per chi lo implementa
- Leggi prima:
  - `AGENTS.md` (comandi, prerequisiti, Insidie);
  - `CONTEXT.md`, e usa i suoi termini: Libreria, Raccolta, Bino, Frase…;
  - gli ADR in `docs/adr/`, in particolare 0005, 0007, 0008 e 0009;
  - la spec `.scratch/sbobino-libreria/spec.md` e `decisioni.md` accanto.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine` e `VoiceDetector` finti, e logica pura del frontend con `bun test`. Niente test sui componenti.
- Verifica il comportamento in `bun tauri dev` via CDP (vedi "Pilotare l'app" in `AGENTS.md`), poi chiudi app e Vite. Una build Rust alla volta. Trascrivi copie delle fixture fuori da `src-tauri`.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. A fine lavoro aggiorna `AGENTS.md`, `PRODUCT.md` (storie e "Stato", vedi le "Further Notes" della spec) e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
