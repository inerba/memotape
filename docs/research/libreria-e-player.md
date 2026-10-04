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
- **[I]** La precisione dello spostamento si misura a mano via CDP con il ticket del player.

## Protocollo per `mix.ogg`

- **[F]** L'esempio ufficiale `examples/streaming` di Tauri usa `register_asynchronous_uri_scheme_protocol` e il crate
  `http-range` 0.1.5, e risponde 206 con `Content-Range` limitando ogni risposta a circa 1 MB. Va adattato alla finestra della
  voce `mix.ogg` nello zip (è `Stored`, vedi `bino::Mix`), con `Accept-Ranges: bytes` e `Content-Type: audio/ogg`.
- **[F]** Su Windows l'URL di un protocollo personalizzato è `http://<schema>.localhost/…`.
- **[F]** wry passa il corpo della risposta come `Vec<u8>` intero: il limite per risposta serve.
- **[F]** Il protocollo `asset` gestisce `Range` ma solo su file interi, quindi non va bene per una voce dello zip.
- **[I]** Il protocollo deve aprire il Bino a ogni richiesta e non tenerlo aperto: su Windows un file aperto bloccherebbe la
  riscrittura (correzioni, rinomina dei Parlanti), la rinomina e il Cestino mentre il player lo usa.

## Correzione per Frase (ticket 03)

- **[F]** `bino::edit_frase` (lettura del documento, `bino::rewrite` con `raw_copy_file` del mix) su un Bino di un'ora,
  14 MB di mix e 600 Frasi, su SSD NVMe: 45 ms in release, 130 ms in debug. Sotto qualche centinaio di ms: la correzione si
  salva subito, senza stato di salvataggio nella Frase.
- **[F]** Il `mix.ogg` di un file trascritto (`OggCopy`: downmix, `Resampler`, `OggOpusWriter`) ha la durata dell'originale
  entro 0,1 ms, per `parlato-it.wav` (16 kHz mono → 48 kHz stereo) e `parlato-it.mp4` (AAC → 16 kHz mono).
