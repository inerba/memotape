# Libreria e player: SQLite, Ogg/Opus in WebView2, protocollo per il mix

Data delle verifiche: 2026-10-04, durante il grilling della spec v3 (`.scratch/sbobino-libreria/spec.md`, "Further Notes").
Riportate qui con il ticket 01 della Libreria. Servono ai ticket della ricerca (02), della vista del Bino (03) e del player.

Legenda: **[F]** = verificato sulla fonte o con una build; **[I]** = inferenza, da confermare a mano.

## rusqlite e FTS5

- **[F]** `rusqlite` 0.40.2 porta `libsqlite3-sys` 0.38.2 con SQLite 3.53.2. Con la feature `bundled` SQLite si compila con `cc`
  e `-DSQLITE_ENABLE_FTS5`, con bindings pregenerati: niente bindgen, CMake o NASM, CRT dinamico come il resto del progetto.
  Confermato dalla build del ticket 01 su MSVC (Rust stable, VS 2022).
- **[F]** FTS5 ha il tokenizer `unicode61 remove_diacritics 2`, le query per prefisso (`parola*`), `snippet()`, `highlight()` e
  `bm25()`.
- **[I]** L'opzione `prefix='2 3'` della tabella FTS5 velocizza i prefissi corti: da valutare con le misure del ticket 02.

## Ogg/Opus in WebView2

- **[F]** WebView2 (Chromium) riproduce `audio/ogg; codecs=opus` nativamente: codec aperto, demuxer FFmpeg di Chromium, con il
  pre-roll nello spostamento e il discard padding, quindi l'end trimming del writer vale.
- **[F]** L'input rate dell'OpusHead sotto i 48 kHz è solo informativo (RFC 7845 §5.1): il decoder lavora sempre a 48 kHz.
- **[F]** Ogg non ha un indice: lo spostamento nell'audio fa più richieste `Range`, che il protocollo deve servire bene.
- **[F]** Spostamento misurato via CDP (ticket 04, vedi sotto): arriva al punto chiesto entro 2 ms.

## Protocollo per `mix.ogg`

- **[F]** L'esempio ufficiale `examples/streaming` di Tauri usa `register_asynchronous_uri_scheme_protocol` e il crate
  `http-range` 0.1.5, e risponde 206 con `Content-Range` limitando ogni risposta a circa 1 MB. Va adattato alla finestra della
  voce `mix.ogg` nello zip (è `Stored`, vedi `bino::Mix`), con `Accept-Ranges: bytes` e `Content-Type: audio/ogg`.
- **[F]** Su Windows l'URL di un protocollo personalizzato è `http://<schema>.localhost/…`.
- **[F]** wry passa il corpo della risposta come `Vec<u8>` intero: il limite per risposta serve.
- **[F]** Il protocollo `asset` gestisce `Range` ma solo su file interi, quindi non va bene per una voce dello zip.
- **[F]** Il protocollo apre il Bino a ogni richiesta e non lo tiene aperto (verificato con il ticket 04): su Windows un file aperto bloccherebbe la
  riscrittura (correzioni, rinomina dei Parlanti), la rinomina e il Cestino mentre il player lo usa.

## Correzione per Frase (ticket 03)

- **[F]** `bino::edit_frase` (lettura del documento, `bino::rewrite` con `raw_copy_file` del mix) su un Bino di un'ora,
  14 MB di mix e 600 Frasi, su SSD NVMe: 45 ms in release, 130 ms in debug. Sotto qualche centinaio di ms: la correzione si
  salva subito, senza stato di salvataggio nella Frase.
- **[F]** Il `mix.ogg` di un file trascritto (`OggCopy`: downmix, `Resampler`, `OggOpusWriter`) ha la durata dell'originale
  entro 0,1 ms, per `parlato-it.wav` (16 kHz mono → 48 kHz stereo) e `parlato-it.mp4` (AAC → 16 kHz mono).

## Player (ticket 04)

Misure del 2026-10-04 in `bun tauri dev` via CDP (WebView2 su Windows 11, Ryzen 7 3700X). Fonte: 255 s di parlato TTS
(`System.Speech`, 16 kHz mono) con un bip da 150 ms a 2 kHz in ogni pausa, a tempi noti; trascritta con Nemotron in quattro
Bini con le impostazioni di Registrazione a 16 e 48 kHz, mono e stereo (32 kbps), quindi con `mix.ogg` scritto da `OggOpusWriter`.

- **[F]** Il protocollo `bino` (`player.rs`, `register_asynchronous_uri_scheme_protocol` con la lettura in `spawn_blocking`)
  serve `mix.ogg` con `Range`; senza `Range` risponde 200 con il mix intero (come il protocollo `asset`), ma `<audio>` manda
  sempre `Range`. Niente `http-range`: un intervallo solo (`a-b`, `a-`, `-n`) si legge in poche righe, il resto si ignora (200).
- **[F]** Precisione dello spostamento: `currentTime` a 600 ms prima di un bip, Play, inizio del bip rilevato con un
  `AnalyserNode` (indice del primo campione sopra soglia nel buffer, rispetto a `currentTime` della lettura). Errore su 6 bip
  sparsi nei 255 s: da −2 a 0 ms per tutte e quattro le varianti. La misura ha bisogno di `crossOrigin` e di
  `Access-Control-Allow-Origin` sul protocollo (altrimenti l'audio cross-origin arriva muto al grafo WebAudio), aggiunti solo
  per la prova: il player non ne ha bisogno.
- **[F]** `duration` del tag `<audio>`: 255,3165 s per tutte le varianti, contro 255,31 s della fonte (+6,5 ms). Il player la
  usa al posto di `durata_ms` del Bino appena è nota.
- **[F]** Correzione di una Frase (`edit_frase`) e rinomina di un Parlante (`rename_parlante`) durante la riproduzione: si
  salvano, e la riproduzione continua senza errori (`readyState` 4) anche dopo lo spostamento successivo.
- **[I]** La rinomina del Bino aperto cambia il percorso e quindi l'URL del player: la vista si rimonta e la riproduzione si ferma.
- **[F]** Comportamento verificato via CDP sul Bino a 48 kHz stereo (22 Frasi):
  - il pulsante del tempo in pausa sposta il player all'inizio della Frase e lo lascia in pausa;
  - in riproduzione il player continua da lì e la Frase si evidenzia in vista;
  - la barra verso la fine porta in vista la Frase evidenziata;
  - `wheel` e PageUp fanno comparire "Segui l'audio" e il testo resta dov'è anche spostando l'audio; il comando lo riporta alla Frase;
  - con il focus in una Frase lo scorrimento resta fermo mentre l'audio va avanti;
  - Spazio fa Play/Pausa con il focus fuori dai campi;
  - il clic su una Frase trovata con la ricerca apre il Bino con il player in pausa all'inizio di quella Frase.
