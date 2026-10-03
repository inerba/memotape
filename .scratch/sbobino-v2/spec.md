Status: ready-for-agent

# Sbobino: spec v2 (Trascrizione dal vivo, Ingressi separati, Bino, Markdown, Diarizzazione)

## Problem Statement

Oggi Sbobino trascrive solo dopo Stop. Chi registra una call o una riunione vede il testo soltanto alla fine, e deve premere Trascrivi a parte. Il testo è un TXT senza struttura che non dice chi ha parlato: in una call non si distingue la propria voce da quella degli altri, e in una riunione non si distinguono le persone. Una Registrazione è un `.ogg` sciolto, separato dal suo testo, e non c'è modo di riaprirla con la trascrizione già fatta.

## Solution

- **Trascrizione dal vivo.** Una casella "Trascrivi dal vivo" accanto a Registra fa comparire il testo mentre si registra: con Nemotron mentre si parla, con Whisper e Parakeet a fine Frase.
- **Ingressi separati.** Registrando da Entrambi, microfono e audio di sistema si possono trascrivere separatamente, e il testo diventa una conversazione con l'Ingresso di ogni Frase.
- **Bino.** Una Registrazione produce un unico file `.bino`, che contiene audio e testo e si riapre con la trascrizione già pronta.
- **Diarizzazione.** Opzionale, con Sortformer: attribuisce le Frasi a Parlanti numerati, sui file con Trascrivi e sulle Registrazioni dopo Stop. I Parlanti si possono rinominare.
- **Markdown.** Il risultato è un documento Markdown formattato invece del TXT.

Le decisioni sono negli ADR-0004 (Trascrizione dal vivo opzionale), ADR-0005 (formato Bino) e ADR-0006 (Diarizzazione con Sortformer a posteriori).

## User Stories

### Trascrizione dal vivo

1. Come utente, voglio una casella "Trascrivi dal vivo" accanto a Registra, così decido a ogni Registrazione se vedere il testo mentre registro.
2. Come utente, voglio che la casella resti salvata tra un avvio e l'altro e sia spenta per default, così su un PC lento la Registrazione resta affidabile finché non la attivo.
3. Come utente, voglio che durante la Registrazione la casella sia bloccata, così il comportamento non cambia a metà sessione.
4. Come utente con Nemotron, voglio vedere le parole comparire mentre parlo, così seguo la call in tempo reale.
5. Come utente con Whisper o Parakeet, voglio vedere ogni Frase appena finisce, così ho comunque il testo durante la Registrazione.
6. Come utente, voglio che Pausa chiuda la Frase in corso e fermi anche la Trascrizione dal vivo, e che Riprendi la continui nella stessa sessione, così le parti in pausa non finiscono nel testo.
7. Come utente, voglio che la Registrazione non rallenti né perda audio se il motore è più lento del parlato, così il file è sempre completo.
8. Come utente, voglio che dopo Stop, se il motore è rimasto indietro, la status bar mostri "Completamento della trascrizione…" con l'avanzamento, così so che il testo sta ancora arrivando.
9. Come utente, voglio poter annullare il completamento della trascrizione, così non aspetto se mi basta l'audio.
10. Come utente, voglio che con il flag attivo e del testo già nell'area, Registra chieda conferma prima di sostituirlo, così non lo perdo.
11. Come utente, voglio che se il modello scelto non è scaricato o non si carica la Registrazione parta comunque, con un avviso e il link alle Impostazioni, così non perdo la Registrazione.
12. Come utente, voglio che a fine Registrazione il testo dal vivo sia il risultato definitivo, salvato nel Bino e nel Markdown, così non devo ritrascrivere.

### Ingressi separati

13. Come utente che registra da Entrambi, voglio scegliere in Impostazioni → Registrazione tra "Mix" e "Ingressi separati" per la Trascrizione dal vivo, così distinguo me dagli altri in una call.
14. Come utente con Ingressi separati, voglio vedere il testo come una conversazione, con ogni messaggio etichettato "Microfono" o "Audio di sistema", così capisco chi ha detto cosa.
15. Come utente, voglio i messaggi in ordine di inizio della Frase e non di fine del calcolo, così una risposta non scavalca una domanda.
16. Come utente, voglio che i due Ingressi si trascrivano davvero in parallelo, ognuno con la sua istanza del modello, così nessuno dei due resta indietro per colpa dell'altro.
17. Come utente, voglio che con Ingressi separati il Bino contenga anche l'audio di ogni Ingresso, così la Diarizzazione può lavorare su ciascuno.

### Bino

18. Come utente, voglio che Stop salvi un solo `Registrazione <data ora>.bino` nella Cartella predefinita, così audio e testo restano insieme.
19. Come utente, voglio aprire un Bino con Sfoglia e ritrovare il testo, compresa la conversazione e i Parlanti, senza ritrascrivere, così riprendo il lavoro.
20. Come utente, voglio trascrivere di nuovo un Bino e che il risultato sostituisca il testo dentro il Bino, previa conferma, così posso provare un altro modello.
21. Come utente, voglio fare doppio clic su un `.bino` in Esplora file e vederlo aperto in Sbobino, nella finestra già aperta se c'è, così lo uso come un documento.
22. Come utente, voglio che il clic sul nome di un Bino nell'app lo mostri nella cartella, così lo trovo.
23. Come utente, voglio che un Bino creato da una versione futura e non compatibile dia un errore chiaro, così capisco che devo aggiornare l'app.
24. Come utente, voglio che le Registrazioni `.ogg` già fatte si aprano ancora come Sorgente, così non perdo nulla.
25. Come utente, voglio che un crash durante la Registrazione non mi faccia perdere tutto l'audio, così una Registrazione lunga è al sicuro.

### Markdown e Copia testo

26. Come utente, voglio che il risultato sia `<nome Sorgente> trascrizione <N>.md` invece del TXT, con le stesse regole sul primo N libero, così ho un documento formattato.
27. Come utente, voglio un'intestazione con titolo, data, durata, modello e Lingua del parlato, così so com'è stato prodotto il testo.
28. Come utente, voglio paragrafi che vanno a capo dopo una pausa lunga (oltre 2 s), così il testo si legge come prosa.
29. Come utente, voglio che con Parlanti o Ingressi separati ogni turno sia un paragrafo `**Parlante 1:**` o `**Microfono · Parlante 2:**`, con le Frasi consecutive della stessa voce unite, così il testo si legge come un verbale.
30. Come utente, voglio scegliere in Impostazioni se "Copia testo" copia testo semplice o Markdown, così incollo nel formato che mi serve.

### Diarizzazione

31. Come utente, voglio una casella "Riconosci i parlanti" accanto a Trascrivi, salvata tra gli avvii, così diarizzo i file di riunioni.
32. Come utente, voglio in Impostazioni → Registrazione una casella "Riconosci i parlanti" per il mix, oppure una per Microfono e una per Audio di sistema con Ingressi separati, così diarizzo solo dove serve (per esempio solo gli altri in una call, oppure anche il mio lato se siamo in più al microfono).
33. Come utente, voglio che la Diarizzazione di una Registrazione giri dopo Stop e aggiunga i Parlanti alle Frasi già comparse, così il testo dal vivo diventa un verbale.
34. Come utente, voglio scaricare il modello di diarizzazione dalle Impostazioni, con percentuale, verifica e ripresa come gli altri modelli, così lo scarico solo se lo uso.
35. Come utente, voglio che senza il modello di diarizzazione le caselle restino attive ma al momento dell'uso compaia l'errore "modello non scaricato" con il link alle Impostazioni, così so cosa fare.
36. Come utente, voglio sapere dall'interfaccia che si riconoscono al massimo 4 Parlanti, così non mi stupisco in riunioni grandi.
37. Come utente, voglio i Parlanti numerati per ordine di comparsa, così "Parlante 1" è chi ha parlato per primo.
38. Come utente, voglio poter annullare la Diarizzazione come il resto della Trascrizione, così non aspetto su file lunghi.
39. Come utente, voglio rinominare "Parlante 1" in un nome, e che la rinomina valga per tutto il testo, per il Bino e per il Markdown, così il verbale ha i nomi veri.

## Implementation Decisions

### Milestone (da dettagliare in ticket)

| # | Milestone | Storie |
|---|---|---|
| V1 | **Trascrizione dal vivo sul mix**: pipeline a frame con tempi delle Frasi, coda, Pausa, completamento dopo Stop | 1–12 (per ora il risultato è nel TXT o MD esistente) |
| V2 | **Bino**: la Registrazione produce un Bino, il Bino si apre come Sorgente e si ritrascrive, associazione `.bino` con istanza unica, salvataggio sicuro durante la Registrazione | 18–25 |
| V3 | **Markdown e Copia testo**: renderer unico per Markdown e testo semplice, `.md` al posto del TXT, impostazione di Copia testo | 26–30 |
| V4 | **Ingressi separati**: due flussi con due istanze del modello, vista a conversazione, audio per Ingresso nel Bino | 13–17 |
| V5 | **Diarizzazione**: Sortformer nel catalogo, file e Registrazioni (mix o Ingressi scelti), attribuzione dei Parlanti, turni nel Markdown | 31–38 |
| V6 | **Rinomina dei Parlanti** | 39 |

Dipendenze:
- V2 e V3 dipendono da V1. V3 può partire con V2.
- V4 dipende da V1 e V2.
- V5 dipende da V2, V3 e V4: gli Ingressi diarizzabili esistono solo con V4.
- V6 dipende da V5.

### Pipeline (estensione dell'esistente)

- **Ingresso della pipeline.** La pipeline di Trascrizione non riceve più un file ma una **fonte di frame a 16 kHz mono**, che produce frame finché c'è audio e poi termina. Ci sono due fonti:
  - il decoder di un file (comportamento attuale, con il progresso);
  - un canale alimentato dalla Registrazione.

  La funzione attuale per i file diventa un involucro sottile sopra la pipeline generica.
- **Tempi delle Frasi.** Ogni Frase emessa porta `inizio_ms` e `fine_ms` sulla linea del tempo della Sorgente, contati in frame da 30 ms, oltre all'`id`. In una Registrazione la linea del tempo esclude le pause, quindi coincide con l'audio salvato. Questi tempi servono alla vista a conversazione, al Bino, al Markdown (pause lunghe) e all'attribuzione dei Parlanti.
- **Coda.** La Registrazione non aspetta mai il motore:
  - il thread della Registrazione spinge i frame in un canale senza limite, e un thread di Trascrizione dedicato li consuma;
  - dopo Stop il canale si chiude e la pipeline smaltisce quello che resta;
  - l'avanzamento dello smaltimento è la percentuale di frame consumati rispetto a quelli ricevuti.
- **Pausa.** All'inizio di una pausa il thread della Registrazione segnala alla pipeline una "chiusura della Frase", che chiude la Frase in corso come se fosse finito il parlato. Riprendi apre una Frase nuova.
- **Origine dei frame dal vivo.**
  - Modalità mix: dall'uscita del mixer, con downmix e ricampionamento a 16 kHz tramite il resampler esistente.
  - Modalità Ingressi separati: il mixer espone anche il flusso allineato di ogni Ingresso, con i silenzi nei buchi e le pause escluse. Lo stesso flusso alimenta l'Ogg per Ingresso.

### Motore e modelli

- `TranscriptionEngine` non cambia.
- Il manager del modello caricato deve poter fornire **due istanze** dello stesso modello per gli Ingressi separati, perché `transcribe-cpp` ammette un solo stream per modello. La seconda istanza si carica all'avvio di una Registrazione a Ingressi separati e si scarica alla fine.
- Il modello e la Lingua del parlato sono quelli delle impostazioni.
- Se il caricamento fallisce, la Registrazione continua senza Trascrizione dal vivo e mostra un avviso (codice d'errore dedicato).
- **Attività**: la Trascrizione dal vivo e lo smaltimento dopo Stop fanno parte dell'Attività Registrazione, che termina quando la coda è vuota (e la Diarizzazione è finita) oppure quando la si annulla.

### Bino

- Modulo `bino` senza Tauri, con due funzioni: scrivere un Bino (audio + documento) e leggerlo.
- Zip (crate `zip`) con queste voci:
  - `mix.ogg`;
  - `microfono.ogg` e `sistema.ogg`, solo con Ingressi separati;
  - `trascrizione.json`.

  Gli Ogg sono salvati senza ricompressione, il JSON compresso.
- **Schema del JSON, versione 1**:

  ```
  {
    version: 1,
    creato: <ISO 8601>,
    durata_ms,
    modalita: "mix" | "ingressi_separati",
    modello: <id del catalogo> | null,
    lingua_parlato: "auto" | <codice>,
    completa: bool,
    parlanti: { "<ingresso>:<n>": "<nome>" },
    frasi: [ { id, inizio_ms, fine_ms, testo, ingresso: "mix" | "microfono" | "sistema", parlante: n | null } ]
  }
  ```

  I campi sconosciuti si ignorano in lettura. Una `version` maggiore di quella supportata dà l'errore dedicato `unsupportedBino`.
- **Durante la Registrazione** gli Ogg si scrivono in file temporanei, in una cartella nascosta accanto alla destinazione nella Cartella predefinita. A Stop, finito lo smaltimento e la Diarizzazione, si compone il Bino e si cancellano i temporanei. Se si annulla il completamento, il Bino si scrive comunque con le Frasi già pronte e `completa: false`, mentre il Markdown non si salva.
- **Aprire un Bino come Sorgente.** Il testo e le Frasi vengono dal JSON. Trascrivi decodifica `mix.ogg` estraendolo in una cartella temporanea dell'app (o leggendolo direttamente, se Symphonia lo consente su una voce dello zip non compressa), e alla fine riscrive il JSON nel Bino in modo atomico (nuovo file e poi rename).
- **Associazione**: `fileAssociations` di Tauri per `.bino` nell'installer NSIS, più `tauri-plugin-single-instance`. Un secondo avvio con un percorso passa il file alla finestra esistente, che lo apre come Sorgente (con la conferma se l'area contiene testo).
- Il clic sul nome di un Bino usa "mostra nella cartella" (`tauri-plugin-opener`) invece di "apri".

### Markdown

- Un renderer puro riceve i metadati (titolo = nome della Sorgente, data, durata, modello, Lingua del parlato) e le Frasi con tempi, Ingresso e Parlante. Produce due formati: Markdown e testo semplice.
  - **Senza Parlanti e senza Ingressi separati**: le Frasi si uniscono in paragrafi, e se ne apre uno nuovo quando la pausa tra una Frase e la successiva supera i 2 s.
  - **Con Parlanti o Ingressi separati**: un paragrafo per turno, con etichetta `**Parlante N:**`, `**Microfono:**` o `**Microfono · Parlante N:**`. Le Frasi consecutive con la stessa etichetta si uniscono. I nomi dei Parlanti rinominati sostituiscono "Parlante N".
- **File**: `<stem della Sorgente> trascrizione <N>.md` accanto alla Sorgente, con la regola esistente del primo N libero. Il TXT sparisce del tutto, e il risultato del comando di Trascrizione restituisce il percorso del `.md`.
- **Copia testo**: impostazione `copia_come: testo | markdown`, default testo, che usa lo stesso renderer tramite un comando.

### Diarizzazione

- Il catalogo dei modelli acquista un tipo `diarizzazione`, con Sortformer 4spk v2.1 Q8_0:
  - URL a revisione fissata, `https://huggingface.co/handy-computer/diar_streaming_sortformer_4spk-v2.1-gguf/resolve/ae4afbb5c3d33b71cf2dbf600022b655ee706dd0/diar_streaming_sortformer_4spk-v2.1-Q8_0.gguf`;
  - 139 310 336 byte, SHA-256 `a5dacdc650790266c7a362e54e6bf51952015487edaa606c4e11632bc32442a9`;
  - licenza NVIDIA Open Model License, da mostrare in Informazioni.

  Download, verifica, ripresa ed Elimina sono quelli esistenti. La sezione Trascrizione delle Impostazioni lo mostra separato dai modelli di trascrizione.
- **Esecuzione**: `Session::run` con diarizzazione sull'audio intero a 16 kHz (file, mix o Ingresso) produce segmenti `(t0_ms, t1_ms, speaker_id)`. I segmenti arrivano ordinati per parlante, e Sbobino li riordina per tempo. L'annullamento usa il `CancelToken` esistente.
- **Attribuzione**: una funzione pura assegna a ogni Frase il Parlante con la maggiore sovrapposizione temporale (nessuno se la sovrapposizione è nulla) e rinumera i Parlanti per ordine di comparsa.
- **Quando gira**:
  - Trascrivi su un file con "Riconosci i parlanti" attivo: dopo la Trascrizione, nella stessa Attività;
  - dopo Stop di una Registrazione, sul mix o sugli Ingressi con la casella attiva, dopo lo smaltimento della coda e prima di comporre il Bino;
  - aprendo un Bino e ritrascrivendolo: sul `mix.ogg`.
- **Impostazioni nuove**:
  - `trascrizione_dal_vivo: bool` (false);
  - `modalita_dal_vivo: mix | ingressi_separati` (mix; ingressi separati vale solo con Entrambi);
  - `parlanti_file: bool` (false);
  - `parlanti_mix: bool`, `parlanti_microfono: bool`, `parlanti_sistema: bool` (false);
  - `copia_come` (testo).

  Il modello di diarizzazione è unico, quindi non c'è un'impostazione per sceglierlo.
- **Rinomina (V6)**: clic su un'etichetta Parlante nell'area. Il nome vale per tutte le Frasi di quel Parlante, si salva in `parlanti` nel Bino (se la Sorgente è un Bino) e riscrive l'ultimo Markdown prodotto per quella Sorgente.

### Frontend

- `features/transcription`:
  - vista a conversazione (messaggi con etichetta, ordinati per `inizio_ms`, raggruppati per voce consecutiva);
  - casella "Riconosci i parlanti";
  - stato di completamento.
- `features/recording`: casella "Trascrivi dal vivo" accanto a Registra, e Parziali e Frasi durante la Registrazione.
- `features/settings`: modalità dal vivo, caselle dei Parlanti per Ingresso, Copia testo come.
- `features/models`: il modello di diarizzazione.
- Gli eventi `transcript-partial` e `transcript-phrase` portano anche `ingresso`, `inizioMs`, `fineMs` e `parlante` (opzionale). Un nuovo evento `speakers-assigned` porta le attribuzioni dopo la Diarizzazione.

## Testing Decisions

- **Cosa è un buon test**: verifica il comportamento osservabile dalla seam pubblica (Frasi emesse con i loro tempi, file prodotti, testo reso), non i dettagli interni.
- **Seam 1, core Rust senza Tauri.** Finti solo `TranscriptionEngine` e `VoiceDetector`.
  - *Pipeline a frame*:
    - una fonte di frame sintetici produce Frasi con `inizio_ms` e `fine_ms` corretti;
    - con un motore finto più lento dell'audio, tutte le Frasi arrivano e lo smaltimento termina dopo la chiusura del canale;
    - la chiusura della Frase per la Pausa;
    - Annulla durante lo smaltimento;
    - con due fonti (Ingressi), le Frasi in ordine di inizio.
  - *Mixer*: il flusso per Ingresso è allineato al mix, con i buchi in silenzio e le pause escluse.
  - *Bino*:
    - scrittura e lettura con lo stesso documento;
    - campi sconosciuti ignorati;
    - `version` futura → `unsupportedBino`;
    - `mix.ogg` decodificabile come Sorgente;
    - riscrittura atomica del JSON;
    - Bino con `completa: false`.
  - *Markdown*: intestazione; paragrafi con la soglia di 2 s; turni e unione per Parlante e per Ingresso; nomi rinominati; testo semplice.
  - *Attribuzione dei Parlanti*: sovrapposizioni parziali, Frase senza segmenti, rinumerazione per ordine di comparsa, segmenti non ordinati.
  - *Smoke test `#[ignore]`*: Sortformer vero su una fixture con due voci (per esempio due voci TTS diverse), che deve trovare 2 Parlanti.
- **Seam 2, logica pura del frontend** con `bun test`: ordinamento e raggruppamento della conversazione, schema delle impostazioni nuove, testo della status bar durante lo smaltimento e la Diarizzazione.
- **Prior art**: test della pipeline, del segmentatore, del mixer e del writer Ogg/Opus già nel repo (fixture TTS `parlato-it.wav`/`.mp4`, helper per WAV e sinusoidi sintetici, rilettura con Symphonia); smoke test `#[ignore]` dei tre modelli; test di `settings` e `models` nel frontend.

## Out of Scope

- Diarizzazione dal vivo e Nemotron-3-Diarization (ADR-0006): si riaprono quando la PR #175 di transcribe.cpp entra in una release.
- Scelta dell'algoritmo di diarizzazione, `speakrs`/pyannote, e numero di Parlanti impostabile.
- Un modello diverso per ciascun Ingresso.
- Cancellazione dell'eco (le cuffie sono responsabilità dell'utente, senza avvisi nell'interfaccia).
- Bino per i file aperti con Sfoglia: per loro c'è solo il Markdown accanto al file.
- Timestamp nel testo, modifica del testo dentro il Bino, metadati del Bino oltre lo schema v1 ("li vedremo in seguito").
- Ricollegare a un Bino gli `.ogg` sciolti delle Registrazioni precedenti.

## Further Notes

- **Decisioni prese nella spec, da confermare con l'utente**:
  - annullare il completamento dopo Stop salva comunque il Bino, con le Frasi pronte e `completa: false`, ma non il Markdown;
  - la rinomina riscrive l'ultimo Markdown della Sorgente invece di crearne uno nuovo;
  - i nomi interni delle voci dello zip (`mix.ogg`, `microfono.ogg`, `sistema.ogg`, `trascrizione.json`).
- **Rischi**:
  - Sortformer non ha dati sull'italiano, anche se l'utente riferisce che funziona bene;
  - la RAM della Diarizzazione cresce con la durata, perché tutto l'audio passa in memoria;
  - due istanze di Nemotron significano circa 2 × 560 MB di RAM o VRAM;
  - lo smaltimento della coda dopo Stop può durare molto con Whisper su CPU.
- **Pendenze del giro precedente, fuori da questa spec**:
  - ticket 11, che aspetta repo GitHub ed editore;
  - impostazioni perse se `settings.json` è bloccato solo per un momento all'avvio;
  - verifica dell'installer su un PC pulito.
- **Riferimenti**: ADR-0004, 0005 e 0006; `docs/research/diarizzazione.md` (§ Sortformer e §8 Nemotron-3); `CONTEXT.md` (Trascrizione dal vivo, Ingressi separati, Bino, Diarizzazione, Parlante).
