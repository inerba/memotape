Status: ready-for-agent

# Sbobino: spec v1

## Problem Statement

Chi deve sbobinare lezioni, riunioni, interviste o video oggi carica i file su servizi online, che chiedono account e chiavi, mandano l'audio a terzi e non funzionano senza rete. Spesso non c'è nemmeno un modo semplice per registrare insieme la propria voce e l'audio del computer, per esempio in una videochiamata, e poi trascriverli. L'utente vuole un'app Windows che faccia tutto sul proprio PC: aprire un file audio o video, oppure registrare, e ottenere il testo, senza account, senza chiavi e senza rete una volta scaricato il modello.

## Solution

Sbobino è un'app desktop solo Windows x64 (Tauri 2 + React).
- **Sorgente.** L'utente apre un file audio o video con "Sfoglia", oppure lo crea con una Registrazione da microfono, audio di sistema o entrambi. Il file diventa la Sorgente.
- **Trascrizione.** Trascrivi riconosce il parlato in locale con uno di tre modelli (Nemotron Streaming consigliato, Whisper Large v3 Turbo, Parakeet TDT v3). Il testo compare Frase per Frase in un'area dedicata: con Nemotron anche come Parziale, mentre la Frase è in corso. Alla fine il testo viene salvato in un TXT accanto alla Sorgente.
- **Impostazioni.** Restano salvate tra un avvio e l'altro. L'interfaccia è disponibile in sei lingue.

Nessun ffmpeg: la decodifica è in Rust (Symphonia), le Registrazioni sono in OGG/Opus scritte in Rust e "Estrai solo audio" non fa parte del prodotto (ADR-0002). La Trascrizione parte solo dopo Stop (ADR-0003).

## User Stories

### Sorgente

1. Come utente, voglio un pulsante "Sfoglia" che apra il dialog di sistema, così scelgo un file senza digitare percorsi.
2. Come utente, voglio che il dialog proponga solo le estensioni accettate (MP3, WAV, M4A, FLAC, OGG, OPUS, WEBM, MPGA, MPEG, AIFF, MP4, MKV, MOV, M4V), così non scelgo file inutili.
3. Come utente, voglio vedere il nome della Sorgente nella finestra, così so su cosa sto lavorando.
4. Come utente, voglio cliccare il nome della Sorgente e aprirla con il programma associato, così la ascolto o la guardo prima di trascrivere.
5. Come utente, voglio vedere il percorso completo della Sorgente nella status bar, così so dove si trova.
6. Come utente, voglio che dopo la scelta di un file mi venga proposta l'azione Trascrivi, così so cosa posso fare.
7. Come utente, voglio un errore dedicato se il file contiene un codec audio non supportato (per esempio AC-3 dentro un MKV o un `.mpeg` che è un video MPEG-PS), così capisco che il problema è il formato e non l'app.
8. Come utente, voglio un errore dedicato se il file non esiste più o non è leggibile, così so che va scelto di nuovo.

### Trascrizione

9. Come utente, voglio premere Trascrivi su un file audio e vedere comparire il testo, così ottengo la sbobinatura.
10. Come utente, voglio trascrivere anche un video MP4, MOV, M4V o MKV senza prima estrarne l'audio, così risparmio un passaggio.
11. Come utente, voglio una Frase per riga nell'area di testo, così il testo si legge e si modifica facilmente.
12. Come utente con Nemotron, voglio vedere il Parziale della Frase in corso mentre viene riconosciuta, così seguo il lavoro in tempo reale.
13. Come utente con Whisper o Parakeet, voglio vedere ogni Frase appena è conclusa, così vedo comunque l'avanzamento.
14. Come utente, voglio la percentuale di avanzamento nella status bar quando la durata è nota, così so quanto manca.
15. Come utente, voglio un avanzamento senza percentuale quando la durata non è nota, così so comunque che l'app sta lavorando.
16. Come utente, voglio che a fine Trascrizione il testo venga salvato in `<nome Sorgente> trascrizione <N>.txt` accanto alla Sorgente, con N il primo numero libero, così non perdo il risultato e non sovrascrivo trascrizioni precedenti.
17. Come utente, voglio che la status bar mostri a fine Trascrizione che è finita, il numero di caratteri e il percorso del TXT, così so dove trovarlo.
18. Come utente, voglio un pulsante "Copia testo" che copi l'area negli appunti, così incollo il testo altrove.
19. Come utente, voglio una conferma prima che una nuova Trascrizione sostituisca il testo già presente nell'area, così non lo perdo per errore.
20. Come utente, voglio un pulsante "Annulla" durante la Trascrizione, così fermo un lavoro lungo avviato per sbaglio.
21. Come utente, voglio che dopo Annulla il testo già comparso resti nell'area ma che nessun TXT venga salvato, così il TXT esiste solo per Trascrizioni complete.
22. Come utente, voglio scegliere la Lingua del parlato con un selettore accanto a Trascrivi, così aiuto il modello quando il riconoscimento automatico sbaglia.
23. Come utente, voglio che il selettore della Lingua del parlato abbia "Automatica" come default e offra solo le lingue tra it, en, fr, es, de e pl supportate dal modello selezionato, così non scelgo combinazioni impossibili.
24. Come utente, voglio che la Lingua del parlato scelta resti salvata tra un avvio e l'altro, così non la reimposto ogni volta.
25. Come utente, voglio che la Trascrizione funzioni senza rete una volta scaricato il modello, così lavoro anche offline.
26. Come utente, voglio che senza un modello scaricato Trascrivi mostri un errore dedicato con un link a Impostazioni → Trascrizione, così so cosa fare.
27. Come utente, voglio che mentre una Trascrizione è in corso Registra e Sfoglia siano disabilitati, così non avvio due Attività insieme.
28. Come utente, voglio che una Trascrizione che fallisce mostri un errore dedicato nella status bar, così capisco cosa è successo.

### Modelli

29. Come utente, voglio vedere in Impostazioni → Trascrizione i tre modelli con dimensione del download e modalità del testo (in streaming o a fine frase), così scelgo consapevolmente.
30. Come utente, voglio che Nemotron sia indicato come consigliato e selezionato per default, così parto dalla scelta migliore.
31. Come utente, voglio scaricare un modello vedendo l'avanzamento in percentuale, così so quanto manca.
32. Come utente, voglio che un modello diventi utilizzabile solo dopo la verifica d'integrità (dimensione e SHA-256), così non trascrivo con un file corrotto.
33. Come utente, voglio un errore dedicato se la verifica fallisce, così riprovo il download.
34. Come utente, voglio annullare un download in corso e che il file parziale venga cancellato, così libero spazio.
35. Come utente, voglio che un download interrotto (connessione persa, app chiusa) riprenda da dove si era fermato, così non riscarico centinaia di MB.
36. Come utente, voglio eliminare un modello scaricato, così libero spazio su disco.
37. Come utente, voglio che "Elimina" sia disabilitato per il modello che una Trascrizione in corso sta usando, così non la rompo.
38. Come utente, voglio che il download continui in background mentre registro o trascrivo con un altro modello, così non aspetto.
39. Come utente, voglio scegliere quale modello usare, così confronto qualità e velocità.

### Registrazione

40. Come utente, voglio registrare dal Microfono, dall'Audio di sistema o da Entrambi, così catturo la mia voce, una videochiamata o tutte e due.
41. Come utente, voglio che con "Entrambi" le due sorgenti finiscano mixate in un solo file, così ho un'unica traccia da trascrivere.
42. Come utente, voglio usare i dispositivi predefiniti oppure sceglierne uno tra quelli rilevati, così registro dal microfono o dalle cuffie giuste.
43. Come utente, voglio un timer della durata registrata che non conti le pause, così so quanto dura il file.
44. Come utente, voglio un indicatore di livello per ciascuna sorgente attiva, così vedo che entrambe stanno arrivando.
45. Come utente, voglio Pausa e Riprendi nella stessa sessione, così salto le parti che non mi interessano senza creare più file.
46. Come utente, voglio che il file non contenga vuoti per le pause, così l'ascolto e la Trascrizione sono continui.
47. Come utente, voglio che Stop salvi il file nella Cartella predefinita come `Registrazione <data ora>.ogg`, con il prefisso nella Lingua dell'interfaccia, così trovo le registrazioni in ordine.
48. Come utente, voglio che se quel nome esiste già venga aggiunto " 2", " 3"…, così nulla viene mai sovrascritto.
49. Come utente, voglio che dopo Stop la Registrazione diventi la Sorgente, così posso subito premere Trascrivi.
50. Come utente, voglio che con l'Audio di sistema la registrazione continui anche quando il PC è in silenzio, inserendo silenzio nel file, così timer e audio restano allineati.
51. Come utente, voglio che se un dispositivo si scollega durante la Registrazione questa si fermi come con Stop, salvando quanto registrato, e la status bar mostri un errore con il nome del dispositivo, così non perdo nulla.
52. Come utente, voglio che la Registrazione usi il bitrate, i canali e la frequenza delle Impostazioni, così controllo qualità e dimensione.
53. Come utente, voglio che durante una Registrazione Sfoglia e Trascrivi siano disabilitati, così non avvio due Attività insieme.

### Impostazioni

54. Come utente, voglio scegliere la sorgente di registrazione predefinita e i dispositivi, così non li reimposto ogni volta.
55. Come utente, voglio scegliere il bitrate tra 16, 24, 32, 48, 64, 96, 128, 192 e 320 kbps, così adatto la qualità.
56. Come utente, voglio scegliere mono o stereo e la frequenza tra 8 000, 16 000, 24 000 e 48 000 Hz, così adatto il file all'uso.
57. Come utente, voglio come predefiniti 32 kbps, mono, 48 kHz, così ho subito un buon compromesso per la voce.
58. Come utente, voglio scegliere la Cartella predefinita, che in mancanza è `Documenti\Sbobino` e viene creata se non esiste, così so dove finiscono le Registrazioni.
59. Come utente, voglio scegliere la Lingua dell'interfaccia tra it, en, fr, es, de e pl, con un avviso che si applica al riavvio, così uso l'app nella mia lingua.
60. Come utente al primo avvio, voglio l'interfaccia nella lingua del sistema se è tra le sei, altrimenti in inglese, così non devo cercare l'impostazione.
61. Come utente, voglio che tutte le impostazioni restino salvate tra un avvio e l'altro, così l'app riparte come l'ho lasciata.
62. Come utente, voglio che un file impostazioni corrotto non impedisca l'avvio ma riporti ai valori predefiniti, così l'app parte sempre.
63. Come utente, voglio una sezione Informazioni con la versione dell'app e le licenze dei componenti (modelli, Symphonia, ONNX Runtime, Silero, transcribe-cpp), così l'app rispetta le attribuzioni richieste.

### Finestra e aggiornamenti

64. Come utente, voglio che le sezioni visibili seguano l'Attività (Sorgente, Registrazione, Trascrizione), così la finestra mostra solo ciò che serve.
65. Come utente, voglio che la status bar mostri sempre la fase in corso, la percentuale quando c'è e gli errori con messaggi dedicati, così so sempre cosa succede.
66. Come utente, voglio che all'avvio, se c'è connessione, l'app controlli se esiste una versione più recente e mi proponga il link per scaricarla, così resto aggiornato.
67. Come utente offline, voglio che il controllo aggiornamenti fallisca in silenzio, così non vedo errori inutili.
68. Come utente, voglio che il tema chiaro o scuro segua quello di Windows, così l'app si integra con il sistema.

### Sviluppo e distribuzione

69. Come sviluppatore, voglio che `typecheck`, `test`, `check`, `format:backend`, `lint:backend` e `cargo test` passino a ogni milestone, così il progetto resta sano.
70. Come sviluppatore, voglio che i comandi e gli eventi Tauri siano tipizzati da un `bindings.ts` generato da Rust, così frontend e backend non divergono.
71. Come sviluppatore, voglio che `AGENTS.md` documenti comandi, prerequisiti di build, architettura e insidie, così un agente o un collega riparte senza chiedere.
72. Come utente finale, voglio un installer NSIS che includa tutto il necessario (DLL di runtime, modello Silero, testi delle licenze), così installo e uso l'app su qualsiasi PC Windows x64 recente.

## Implementation Decisions

### Milestone

Ognuna si vede funzionare in `bun tauri dev`. Ogni milestone diventa un ticket, e a fine milestone ci si ferma in attesa del via.

| # | Milestone | Requisiti coperti (storie) |
|---|---|---|
| M0 | **Scaffold**: progetto dal template MrLightful/create-tauri-react e `git init` pulito. Dentro: lo stack completo, `PRODUCT.md` (fonte di verità dei requisiti), `AGENTS.md` (comandi, prerequisiti, architettura, Insidie corrette), `CLAUDE.md` = solo `@AGENTS.md` (il blocco "Agent skills" attuale si sposta in `AGENTS.md`), `README.md` per sviluppatori, Husky + lint-staged con `bunx ultracite fix`. La home chiama un comando tipizzato da `bindings.ts`. i18next è configurato con il solo italiano | 69, 70, 71 |
| M1 | **Tracer bullet**: Sfoglia su un file audio, poi Trascrivi. Il percorso è decodifica Symphonia → resampler → VAD + segmentatore → motore `transcribe-cpp` (Nemotron in modalità `run`, con il modello messo a mano nella cartella dei modelli e la procedura in `AGENTS.md`) → evento `transcript-phrase` → una Frase per riga nell'area. Silero e ONNX Runtime sono già quelli definitivi | 1, 2, 9, 11, 13 |
| M2 | **Sorgente e Trascrizione complete**: nome cliccabile, percorso nella status bar, video MP4/MOV/M4V/MKV, errori dedicati (codec, file), percentuale o avanzamento senza percentuale, TXT con il primo N libero, status bar finale, Copia testo, conferma di sostituzione, Annulla, una Attività alla volta | 3–8, 10, 14–21, 27, 28, 64, 65 |
| M3 | **Modelli e impostazioni persistenti**: catalogo `models.json`, download con percentuale, verifica, ripresa, Annulla, Elimina, selezione del modello. Il file impostazioni JSON nasce qui (modello e Lingua del parlato), con l'errore "nessun modello". Whisper e Parakeet in modalità `run`, e selettore della Lingua del parlato filtrato per capability | 22–26, 29–39, 61, 62 |
| M4 | **Parziali in streaming**: Nemotron passa all'API stream, con `transcript-partial` e `transcript-phrase` che condividono l'id della Frase | 12 |
| M5 | **Registrazione dal microfono**: dispositivo predefinito o scelto, timer, livello, Pausa/Riprendi/Stop, writer Ogg/Opus con le impostazioni audio, nome con data e ora, " 2", Cartella predefinita, la Registrazione diventa Sorgente, dispositivo scollegato | 40 (Microfono), 42, 43, 44, 45–49, 51–58 |
| M6 | **Audio di sistema ed Entrambi**: loopback WASAPI, mixer per timestamp QPC, silenzio nei buchi, livello per sorgente, scelta del dispositivo di uscita | 40, 41, 44, 50 |
| M7 | **Lingue dell'interfaccia e Informazioni**: le altre cinque lingue, impostazione applicata al riavvio, lingua di sistema al primo avvio, sezione Informazioni con le licenze, tema che segue il sistema | 59, 60, 63, 68 |
| M8 | **Controllo aggiornamenti**: GitHub `releases/latest`, confronto semver, link. *Bloccata finché non sono decisi repo GitHub ed editore.* | 66, 67 |
| M9 | **Installer NSIS**: `transcribe-cpp` con `dynamic-backends`, DLL dell'ONNX Runtime ufficiale, Silero, testi delle licenze, editore | 72 |

Dipendenze tra milestone:
- M1 richiede M0.
- M2 e M3 richiedono M1.
- M4 richiede M3.
- M5 richiede M2 e M3: le impostazioni audio e la Cartella predefinita vivono nel file impostazioni di M3.
- M6 richiede M5.
- M7 richiede M3.
- M8 richiede M0 e la decisione su repo ed editore.
- M9 richiede tutte le precedenti tranne M8.

### Pipeline: crate e modulo per componente

| Componente | Crate | Modulo (Tauri-free salvo i manager) |
|---|---|---|
| Cattura (callback cpal → canale → worker per sorgente; loopback = stream di input su un dispositivo di output) | `cpal` 0.18.2 | `audio_toolkit::capture` |
| Normalizzazione (mono → resampler → frame da 30 ms a 16 kHz) | `rubato` 5.0.1 (`Fft` con `FixedSync::Input`, adapter `audioadapter`); `FftFixedIn` non esiste più | `audio_toolkit::resample` |
| VAD | `vad-rs` (fork `cjpais/vad-rs` @ `2a412ed858695b9251f3f5a1a20d95b59fa7c498`) su `ort`, con ONNX Runtime ufficiale 1.24.2 caricato dinamicamente | `audio_toolkit::vad` (trait `VoiceDetector` + implementazione Silero) |
| Segmentazione (prefill / onset / hangover / durata massima) | nessuno, solo std | `audio_toolkit::segmenter` |
| Decodifica dei file | `symphonia` 0.6.1 (feature `isomp4`, `mkv`, `ogg`, `wav`, `aiff`, `aac`, `mp3`, `mp1`, `mp2`, `flac`, `vorbis`, `pcm`, `alac`, `adpcm`) + `symphonia-adapter-libopus` 0.3.0 | `audio_toolkit::decode` |
| Motore | `transcribe-cpp` 0.2.4 (feature `vulkan`; `dynamic-backends` in M9) | `engine` (trait `TranscriptionEngine`) e `engine::transcribe_cpp` |
| Pipeline di Trascrizione di un file | composizione dei moduli sopra | `engine::pipeline` |
| Mixer della Registrazione | nessuno, solo std | `audio_toolkit::mixer` |
| Writer Ogg/Opus | `opus` 0.4.0 (libopus 1.6.1 statica, via cmake) + `ogg` 0.9.2 | `audio_toolkit::ogg_opus` |
| Pausa | `AtomicBool` di std letto dai worker | `managers::recording` |
| Download dei modelli | `reqwest` 0.13 (`native-tls`, `http2`, `system-proxy`, senza default) + `sha2` 0.11 | `managers::models` |
| Impostazioni | `serde` + `serde_json` | `managers::settings` |
| Controllo aggiornamenti | `reqwest` + `semver` | `managers::updates` |
| Una Attività alla volta | std | `managers::activity` |

I `commands/` restano sottili: validano gli argomenti e delegano ai manager registrati in `tauri::State`. I manager traducono i callback della pipeline in eventi tauri-specta.

### Interfacce

- **`TranscriptionEngine`**:
  - Elabora una Frase per chiamata, con l'audio come iteratore di frame f32 mono a 16 kHz in [-1, 1].
  - Riceve la Lingua del parlato opzionale e una callback per i Parziali (mai invocata dai motori in modalità `run`), e restituisce il testo della Frase.
  - Espone le capability: streaming sì/no e le lingue supportate, lette a runtime da `capabilities().languages`. Per Nemotron "it" diventa il locale `it-IT`.
  - Errori: `Busy` (riprovabile, mappato da `transcribe-cpp`), `Cancelled`, altro.
  - L'implementazione `transcribe-cpp` usa `Session::run` per Whisper e Parakeet, e in M4 l'API stream (`feed` / `text().committed|tentative` / `finalize`) per Nemotron.
  - Il motore vive in un thread dedicato, con `catch_unwind` attorno alle chiamate native.
- **`VoiceDetector`**: dato un frame da 480 campioni (30 ms a 16 kHz) restituisce la probabilità di parlato; ha anche `reset`.
- **Segmentatore**: macchina a stati pura. Riceve `(frame, probabilità)` ed emette inizio Frase, audio della Frase e fine Frase. Parametri di partenza, da calibrare sull'audio reale:
  - prefill 300 ms, onset 60 ms;
  - chiusura dopo 700 ms di silenzio;
  - taglio a 18 s;
  - soglia 0,4.

  Handy usa 0,3 di soglia, 450 ms di prefill e 450 ms di hangover: se la calibrazione non converge si riparte da lì.
- **Pipeline di un file**: riceve la Sorgente, un motore, un detector, le opzioni (lingua), un token di annullamento e un sink di eventi. Gira alla velocità del calcolo. Emette progresso (frame decodificati / `n_frames` del container, oppure "indeterminato"), Parziali, Frasi e fine. Non sa nulla di Tauri né di file TXT.
- **Eventi tauri-specta**:
  - `transcript-partial { phrase_id, text }` e `transcript-phrase { phrase_id, text }`: l'evento del brief `transcript-segment` è rinominato secondo il glossario;
  - `transcription-progress { percent?: number }`;
  - la fine della Trascrizione (`txt_path`, `chars`) è il risultato del comando `transcribe`, non un evento;
  - `activity-failed { error }`;
  - `model-download-progress { model_id, percent }`, limitato a 10 al secondo, e `model-state-changed`;
  - `recording-tick { elapsed_ms, levels }`.

  Nei tipi esportati niente `u64`/`i64`, perché bloccano l'export: si usano `u32` (tutti i modelli stanno sotto i 4 GiB) o `f64`.
- **Errori**: un unico enum di errori applicativi serializzato con un codice. Il frontend mappa ogni codice a un messaggio dedicato tradotto. Codici previsti: codec non supportato, file illeggibile, modello assente, verifica fallita, download fallito, dispositivo scollegato (con nome), Attività in corso, cartella non scrivibile.

### Dati

- **`models.json`**: incluso in compilazione con `include_str!`. Per ogni modello ha `id`, `nome`, `url` fissato a una revision HF, `sha256`, `size`, `modalita` (`stream` | `frase`) e `licenza`. I valori verificati sono in `docs/research/transcribe-cpp-e-handy.md` §8. Default: Nemotron.
- **Cartella dei modelli**: `app_data_dir/models`.
  - Il download scrive in `<file>.partial`, riprende con `Range: bytes=n-`, verifica dimensione e SHA-256 e alla fine rinomina il file in modo atomico.
  - Annulla cancella il parziale, un'interruzione lo conserva.
  - Un SHA errato cancella il parziale e segnala l'errore.
- **Impostazioni**: JSON in `app_data_dir`, letto e validato da Rust all'avvio. Se il file manca o non è valido si usano i valori predefiniti, senza bloccare l'avvio. Campi:
  - sorgente di registrazione (`mic` | `system` | `both`) e id dei dispositivi (opzionali; assenti = predefiniti di sistema);
  - modello;
  - Lingua del parlato (`auto` | it | en | fr | es | de | pl);
  - bitrate (uno dei nove valori), canali, frequenza;
  - Cartella predefinita (opzionale; assente = `Documenti\Sbobino`);
  - Lingua dell'interfaccia (opzionale; assente = lingua di sistema se supportata, altrimenti en).
  - Il frontend la legge con un comando prima del primo render, e i18next parte con quella lingua.
- **Nomi dei file**:
  - TXT: `<stem della Sorgente> trascrizione <N>.txt` accanto alla Sorgente, con N da 1 il primo libero.
  - Registrazione: `<prefisso tradotto> AAAA-MM-GG HH-MM-SS.ogg`, con " 2", " 3"… se il nome esiste.

  Le due regole sono funzioni pure.
- **Identifier Tauri**: `it.sbobino.desktop`. Non `sbobino`, perché `%APPDATA%\sbobino` appartiene a una vecchia app con lo stesso nome.

### Registrazione

- **Cattura**: ogni sorgente ha il suo worker. La callback cpal non alloca e mette i buffer con il timestamp `capture` (QPC) in un canale. Il worker fa il downmix e ricampiona alla frequenza delle impostazioni.
- **Formato del loopback**: per il dispositivo di uscita la config si prende da `default_output_config()`, perché `default_input_config()` lì dà errore.
- **Mixer**: posiziona i blocchi per timestamp rispetto all'inizio della sessione e mette silenzio nei buchi. Il loopback non consegna pacchetti a riproduzione ferma. Somma le sorgenti con clamp a [-1, 1].
- **Canali**: in stereo il microfono mono si duplica sui due canali, in mono le sorgenti stereo si mediano.
- **Pausa**: un `AtomicBool` letto dai worker. Il tempo di pausa si sottrae dai timestamp, così timer e file non hanno vuoti. Il timer deriva dai timestamp, non dal numero di campioni.
- **Writer Ogg/Opus**:
  - frame da 20 ms, `Application::Audio`;
  - `OpusHead` con l'input rate reale e il pre-skip convertito a 48 kHz;
  - `OpusTags`;
  - granule position in unità a 48 kHz;
  - chiusura forzata della pagina circa ogni secondo, ed `EndStream` a Stop.
- **Dispositivo scollegato**: l'errore della callback cpal ferma la sessione con lo stesso percorso di Stop.

### Frontend

- Struttura bulletproof-react: `app/` (router, provider, `global.css` con i token), `components/` (shadcn in `components/ui`), `lib/`, `features/`. Le feature sono `source`, `transcription`, `recording`, `models`, `settings`, `status`, `updates`, `about`.
- La logica pura vive nelle feature, con un `*.test.ts` accanto.
- Due route: la finestra principale e Impostazioni, con le sezioni Registrazione e audio, Trascrizione e modelli, Generale (Cartella predefinita, Lingua dell'interfaccia) e Informazioni.
- Il selettore della Lingua del parlato sta nella finestra principale accanto a Trascrivi.
- Il form delle impostazioni usa react-hook-form + zod 4, e `react-error-boundary` avvolge l'app.

### Versioni e configurazione

- **Tauri**: `tauri` 2.12.1 con `@tauri-apps/api` e `@tauri-apps/cli` 2.12.1 (stessa minor, major 2 fissata), `tauri-plugin-log` 2.10, `tauri-plugin-opener` 2.7, `tauri-plugin-dialog` 2.8.1 (chiamato da Rust dentro un comando async).
- **specta**: `tauri-specta =2.0.0-rc.25` (feature `derive`, `typescript`), `specta =2.0.0-rc.25`, `specta-typescript =0.0.12`. I bindings si esportano solo in debug.
- **Frontend**:
  - React 19.3, TypeScript 6.0.3 (non la 7), Vite 8.3, `@vitejs/plugin-react` 6.1, Tailwind e `@tailwindcss/vite` 4.3, `tw-animate-css` importato in `global.css` (`tailwindcss-animate` tolto);
  - `react-router` 7.18.x, `react-hook-form` 7.89, `@hookform/resolvers` 5.9, `zod` 4.6, `react-error-boundary` 6.1;
  - i18next + react-i18next, con le forme plurali `_many` per it/fr/es oltre al polacco;
  - Inter Variable da `@fontsource-variable/inter`.
- **shadcn**: stile `new-york`, base `neutral`, `cssVariables`, lucide, `radix-ui` unificato.
  - Il `components.json` si scrive a mano e si usa solo `shadcn add`, perché `init` ora parte dai preset.
  - Si adotta il pacchetto `cn` di shadcn, con `@/lib/utils` che lo ri-esporta. La Insidia del brief va corretta: è un comportamento voluto dal changelog del 2026-09-03.
- **Tooling**:
  - ultracite 7.12 su Biome 2.5, con `includes` `["**", "!**/components/ui", "!**/bindings.ts"]`: la config del template oggi non controlla nessun file sorgente;
  - `bun.lock` rigenerato.
- **Prerequisiti di build** da documentare in `AGENTS.md`:
  - Rust stable MSVC;
  - CMake nel PATH (lo usano `opus` e `transcribe-cpp`);
  - Vulkan SDK LunarG;
  - lo zip dell'ONNX Runtime ufficiale 1.24.2, con `ORT_LIB_LOCATION` e `ORT_PREFER_DYNAMIC_LINK=1`;
  - il `.cargo/config.toml` locale per `LOCALAPPDATA` e `VULKAN_SDK`.
- **Bundle**:
  - Silero è `files/silero_vad.onnx` dal tag v4.0 di snakers4/silero-vad (1 807 522 byte, SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`), incluso come risorsa;
  - le DLL di ONNX Runtime e dei backend di `transcribe-cpp` vanno accanto all'exe, copiate da `build.rs` e mappate da `tauri.windows.conf.json`.

## Testing Decisions

- **Cosa è un buon test**: verifica il comportamento osservabile da una seam pubblica (Frasi emesse, file scritti, eventi, valori restituiti), non i dettagli interni. Deve fallire se il comportamento si rompe e restare verde dopo un refactoring.
- **Seam 1, core Rust senza Tauri.** I test chiamano le API pubbliche di `audio_toolkit`, `engine::pipeline` e dei manager costruiti senza `AppHandle`. Si fingono solo `TranscriptionEngine` e `VoiceDetector`; tutto il resto gira davvero.
  - *Pipeline di un file*: WAV generati nel test, con il motore finto che restituisce testi noti e il detector finto guidato dall'energia. Verifica:
    - l'ordine e il numero delle Frasi;
    - che i Parziali arrivino prima della Frase con lo stesso id;
    - il taglio a 18 s;
    - che Annulla fermi senza produrre la fine;
    - il progresso, determinato e indeterminato;
    - l'errore per codec non supportato.
    Un piccolo MP4/AAC di fixture, generato una volta e committato, copre la strada dei video.
  - *Segmentatore*: sequenze di probabilità per verificare prefill, onset, hangover e durata massima.
  - *Registrazione*: buffer sintetici con timestamp passano per mixer e writer. Verifica che i buchi diventino silenzio, che le pause siano escluse dalla durata, e che il file Ogg/Opus rilegga con Symphonia + adapter libopus con la durata attesa per ogni combinazione di frequenza e canali. cpal non è testato.
  - *Download*: un server HTTP locale sulla loopback e reqwest vero. Verifica la percentuale, la ripresa con Range, che uno SHA errato renda il modello non utilizzabile, che Annulla cancelli il parziale, Elimina, e che l'interruzione conservi il parziale.
  - *Impostazioni*: in una cartella temporanea. Verifica predefiniti, file corrotto → predefiniti, salvataggio e rilettura, e la Lingua dell'interfaccia di default dalla lingua di sistema.
  - *Funzioni pure*: nomi di TXT e Registrazioni (primo N libero, " 2"), confronto semver con `v` iniziale e prerelease.
- **Seam 2, logica pura del frontend** con `bun test`:
  - accettazione delle estensioni;
  - testo della status bar per fase ed errore;
  - schema zod delle impostazioni;
  - filtro della Lingua del parlato per capability del modello.
  Niente test sui componenti.
- **Smoke test** `#[ignore]` con Nemotron vero su un file di parlato reale, da lanciare a mano quando il modello è scaricato.
- **Prior art**: nessuna nel repo, che è nuovo. Come riferimento d'idee ci sono i test di Handy sulla risoluzione della lingua (`effective_language`) e sul downloader con ripresa.

## Out of Scope

- "Estrai solo audio" e qualsiasi conversione video. Niente ffmpeg (ADR-0002).
- I formati AVI, WMV, FLV, TS, MTS, MPEG-PS e i codec AC-3, E-AC-3, HE-AAC, WMA, DTS.
- La Trascrizione durante la Registrazione (ADR-0003) e la Trascrizione di più file in coda.
- Timestamp nel testo, diarizzazione, traduzione, prompt iniziale di Whisper e Lingue del parlato oltre le sei dell'interfaccia.
- Storico delle trascrizioni, editor avanzato del testo, esportazioni diverse dal TXT.
- Installazione automatica degli aggiornamenti (`tauri-plugin-updater`) e firma del codice.
- macOS, Linux, Windows ARM.
- La scelta della GPU: `transcribe-cpp` usa Vulkan se disponibile, altrimenti la CPU.

## Further Notes

- **Decisioni ancora aperte**: il repo GitHub per `releases/latest` e l'editore dell'installer bloccano M8 e la parte "editore" di M9.
- **Correzioni alle Insidie del brief** da riportare in `AGENTS.md` (M0):
  - rubato 5 non ha `FftFixedIn`;
  - niente `DirectML.dll`: si usano le DLL dell'ONNX Runtime ufficiale, perché la build prebuilt di `ort` va in crash sulle CPU pre-Haswell;
  - il file Silero è `silero_vad.onnx`;
  - l'import `cn` è voluto;
  - le Insidie su ffmpeg vanno tolte.
- **Da verificare sull'hardware reale**:
  - che il loopback non consegni callback a riproduzione ferma su Windows 11 (la fonte è del 2008);
  - il drift tra il clock del microfono e quello del dispositivo di uscita nelle sessioni lunghe (eventuale `rubato::Async` regolato sui timestamp);
  - il pre-skip dell'OpusHead quando l'encoder lavora sotto i 48 kHz, con `opusinfo`;
  - se serve `vcomp140.dll`.
- **Licenze** da mostrare in Informazioni e includere nel bundle:
  - Parakeet CC-BY-4.0, con attribuzione obbligatoria;
  - Nemotron OpenMDW-1.1;
  - Whisper MIT (si cita la licenza upstream di OpenAI);
  - Silero MIT;
  - Symphonia MPL-2.0;
  - transcribe-cpp MIT;
  - ONNX Runtime MIT;
  - vad-rs, che dichiara la licenza solo nel `Cargo.toml`.
- **Ricerche di riferimento**: `docs/research/template-vs-stack.md`, `transcribe-cpp-e-handy.md`, `opus-crate-e-decisioni-minori.md`, `decodifica-ffmpeg.md`.
