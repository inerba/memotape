# Ricerca: Opus/Ogg, crate backend e decisioni minori

Fonti consultate il 2026-10-02/03. Legenda: **[V]** = verificato su fonte primaria (spec, sorgente, doc ufficiale, API del registry); **[I]** = inferenza mia, da confermare. Le proposte sono solo proposte: la decisione spetta all'utente.

---

## 1. Opus

### 1.1 Frequenze di ingresso dell'encoder

- **[V]** `opus_encoder_create(Fs, …)`: `Fs` "must be one of 8000, 12000, 16000, 24000, or 48000" ([opus.h](https://github.com/xiph/opus/blob/main/include/opus.h)). Le quattro frequenze delle impostazioni (8/16/24/48 kHz) sono tutte valide. 44 100 Hz **non** lo è: se il mix format WASAPI è a 44,1 kHz bisogna ricampionare prima dell'encoder.
- **[V]** L'encoder limita la banda alla Nyquist dell'ingresso: `Fs<=8000` dà NB, `<=12000` MB, `<=16000` WB, `<=24000` SWB. Il commento nel sorgente: "Prevents Opus from wasting bits on frequencies that are above the Nyquist rate" ([opus_encoder.c, righe ~1641-1650](https://github.com/xiph/opus/blob/main/src/opus_encoder.c)).
- **[V]** Lo strumento di riferimento (libopusenc/opusenc) crea **sempre** l'encoder a 48 kHz e ricampiona l'ingresso con lo speex resampler (`opeint_encoder_surround_init(&enc->st, 48000, …)`, `speex_resampler_init(channels, rate, 48000, …)`). Mette poi la frequenza originale in `input_sample_rate` ([libopusenc/src/opusenc.c, righe 416-435](https://github.com/xiph/libopusenc/blob/master/src/opusenc.c)).

### 1.2 Cosa significa in Ogg Opus (RFC 7845)

- **[V]** La granule position delle pagine audio è "in units of PCM audio samples at a fixed rate of 48 kHz" (per canale), qualunque sia la frequenza dell'encoder ([RFC 7845 §4](https://www.rfc-editor.org/rfc/rfc7845#section-4)). Pagina dell'ID header e pagina dove si chiude il comment header: granule = 0.
- **[V]** `pre-skip` (16 bit, LE) è il "number of samples (at 48 kHz) to discard from the decoder output" e va sottratto dalla granule per ottenere la posizione PCM ([§5.1](https://www.rfc-editor.org/rfc/rfc7845#section-5.1), [§4.2](https://www.rfc-editor.org/rfc/rfc7845#section-4.2)). Si ricava da `OPUS_GET_LOOKAHEAD`, che le applicazioni "should call … rather than hard-coding a value" ([opus_defines.h](https://github.com/xiph/opus/blob/main/include/opus_defines.h)).
- **[I]** `OPUS_GET_LOOKAHEAD` restituisce campioni alla `Fs` dell'encoder (nel sorgente il valore di partenza è `st->Fs/400`, [opus_encoder.c ~3089](https://github.com/xiph/opus/blob/main/src/opus_encoder.c)). Se l'encoder gira a 8/16/24 kHz bisogna convertire: `pre_skip = lookahead * 48000 / Fs`. Allo stesso modo ogni pacchetto da 20 ms fa avanzare la granule di 960, anche a 16 kHz, dove i campioni reali sono 320.
- **[V]** `Input Sample Rate` (32 bit, LE) è "the sample rate of the original input … _not_ the sample rate to use for playback". Il decoder di riferimento decodifica a 8/12/16/24/48 kHz e un player "SHOULD" decodificare a 48 kHz se l'hardware lo supporta. 0 vuol dire "unspecified" ([§5.1 punto 5](https://www.rfc-editor.org/rfc/rfc7845#section-5.1)).
- **[V]** Struttura obbligatoria: il primo pacchetto è `OpusHead`, da solo sulla prima pagina con flag BOS. Il secondo è `OpusTags`, che "MUST finish the page on which it completes". Seguono le pagine audio ([§3](https://www.rfc-editor.org/rfc/rfc7845#section-3)).
- **[V]** `OpusHead` contiene: magic `OpusHead`, version = 1, channel count, pre-skip, input sample rate, output gain (Q7.8, LE), mapping family. Con mapping family 0 (mono/stereo) la channel mapping table "MUST be omitted" ([§5.1](https://www.rfc-editor.org/rfc/rfc7845#section-5.1)). `OpusTags` contiene: magic, vendor string length + vendor string, numero di commenti, poi `len + "KEY=value"` per ogni commento, senza framing bit ([§5.2](https://www.rfc-editor.org/rfc/rfc7845#section-5.2)). Tutti gli interi sono little endian.
- **[V]** End trimming: la pagina con EOS può avere una granule minore dei campioni decodificabili, così l'ultimo frame viene troncato al campione esatto ([§4.4](https://www.rfc-editor.org/rfc/rfc7845#section-4.4)). libopusenc arrotonda per eccesso: `(end_sample*48000 + rate - 1)/rate + preskip` ([opusenc.c ~592](https://github.com/xiph/libopusenc/blob/master/src/opusenc.c)).

### 1.3 Canali e bitrate

- **[V]** Canali: 1 o 2 ([opus.h](https://github.com/xiph/opus/blob/main/include/opus.h)).
- **[V]** La documentazione di `OPUS_SET_BITRATE` dice: "Rates from 500 to 512000 bits per second are meaningful", più i valori speciali `OPUS_AUTO` e `OPUS_BITRATE_MAX` ([opus_defines.h](https://github.com/xiph/opus/blob/main/include/opus_defines.h)). RFC 6716 dice "all bitrates from 6 kbit/s to 510 kbit/s" ([§2.1.1](https://www.rfc-editor.org/rfc/rfc6716#section-2.1.1)).
- **[V]** Comportamento effettivo di libopus quando il bitrate richiesto è fuori range: **nessun errore, clamp silenzioso**. `value <= 0` dà `OPUS_BAD_ARG`. `value <= 500` diventa 500. `value > 750000*channels` diventa `750000*channels` ([opus_encoder.c, `OPUS_SET_BITRATE_REQUEST` ~2816-2829](https://github.com/xiph/opus/blob/main/src/opus_encoder.c)). Poi, a ogni frame, `user_bitrate_to_bitrate` prende il minimo tra il bitrate utente e quello consentito da `max_data_bytes`, e `max_data_bytes` è limitato a 1276 byte per frame ([~733-745, ~1893](https://github.com/xiph/opus/blob/main/src/opus_encoder.c)). Con frame da 20 ms il tetto pratico è circa 510 kbps in totale, non per canale.
- **[V]** Bitrate oltre cui non c'è guadagno: 1275 byte a 20 ms "represents a bitrate of 510 kbit/s, which is approximately the highest useful rate for lossily compressed fullband stereo music" ([RFC 6716 §3.2.1](https://www.rfc-editor.org/rfc/rfc6716#section-3.2.1)). "Sweet spots" a 20 ms: 8-12 kbps NB voce, 16-20 WB voce, 28-40 FB voce, 48-64 FB mono musica, 64-128 FB stereo musica ([RFC 6716 §2.1.1](https://www.rfc-editor.org/rfc/rfc6716#section-2.1.1)). Il wiki Xiph dà 24 kbps mono / 32 stereo per podcast e audiolibri, 96-128 stereo per l'archiviazione musicale, e dice che a 128 kbps VBR Opus è "pretty much transparent" ([Opus Recommended Settings](https://wiki.xiph.org/Opus_Recommended_Settings)).
- **[V]** Default: VBR attivo (`use_vbr = 1`) e "Constrained VBR (default)" (`vbr_constraint = 1`). Hard CBR "can cause noticeable quality degradation" a bitrate molto bassi in LPC/hybrid ([opus_defines.h](https://github.com/xiph/opus/blob/main/include/opus_defines.h), [opus_encoder.c ~292-294](https://github.com/xiph/opus/blob/main/src/opus_encoder.c)). Complexity va da 0 a 10.

### 1.4 Frame e application

- **[V]** Frame validi: 2,5 / 5 / 10 / 20 / 40 / 60 ms, più 80 / 100 / 120 ms (`frame_size_select` accetta `Fs/400`, `Fs/200`, `Fs/100`, `Fs/50`, `Fs/25`, `3Fs/50` … `6Fs/50`) ([opus_encoder.c ~827-848](https://github.com/xiph/opus/blob/main/src/opus_encoder.c), costanti `OPUS_FRAMESIZE_*` in opus_defines.h). Sotto i 10 ms l'encoder non può usare i modi LPC/hybrid ([opus.h](https://github.com/xiph/opus/blob/main/include/opus.h)). Il wiki indica 20 ms come buon default: frame più grandi riducono l'overhead e aumentano la latenza ([wiki Xiph](https://wiki.xiph.org/Opus_Recommended_Settings)).
- **[V]** `OPUS_APPLICATION_VOIP` serve per la voce. Applica un filtro passa-alto ed enfatizza formanti e armoniche, "even at high bitrates the output may sound different from the input". `OPUS_APPLICATION_AUDIO` serve per "music and mixed (music/voice) content" ([opus.h, doc di `opus_encoder_create`](https://github.com/xiph/opus/blob/main/include/opus.h)).
- **[I] Proposta**: usare `AUDIO` quando la sorgente include l'audio di sistema (contenuto misto) e anche ai bitrate medio-alti, perché per la trascrizione conta la fedeltà più dell'"enhancement". `VOIP` si può tenere solo per microfono a 8/16 kHz con bitrate ≤ 24 kbps. Un'alternativa più semplice è un solo valore fisso, `AUDIO`, con `OPUS_SET_SIGNAL(VOICE)` come suggerimento.

### 1.5 Proposta: nove valori di bitrate (16-320 kbps)

**[I]** Proposta: **16, 24, 32, 48, 64, 96, 128, 192, 320 kbps**.

| kbps | Motivazione |
|---|---|
| 16 | Sweet spot WB voce (16-20) e soglia minima FB indicata dal wiki (14-16) |
| 24 | Wiki: podcast/audiolibri mono. Wiki: VoIP "24 Kb/s should give fullband" |
| 32 | Wiki: podcast stereo. Nel range FB voce (28-40) |
| 48 | Inizio sweet spot FB mono musica (48-64) |
| 64 | Fine sweet spot FB mono e inizio FB stereo (64-128) |
| 96 | Wiki: streaming stereo 64-96 e archiviazione 96-128 |
| 128 | Wiki: "pretty much transparent" (VBR, stereo) |
| 192 | Margine per l'archiviazione stereo. Oltre questo il guadagno percepito è dubbio [I] |
| 320 | Estremo richiesto. Legale ovunque (≤ 510 kbps a 20 ms), ma quasi sempre sprecato |

Alternativa per la coda alta: `…, 128, 160, 256` al posto di `128, 192, 320`, se 320 non è un vincolo di prodotto.

**Combinazioni da limitare o avvisare** [I]. Le soglie sono mie e derivano da sweet spot e banda massima per Fs, non sono limiti di libopus:

| Fs | Banda max | Avviso sopra (mono / stereo) |
|---|---|---|
| 8 kHz | NB (4 kHz) | 32 / 48 kbps |
| 16 kHz | WB (8 kHz) | 64 / 96 kbps |
| 24 kHz | SWB (12 kHz) | 96 / 128 kbps |
| 48 kHz | FB (20 kHz) | 128 / 256 kbps |

Nessuna combinazione dei nove valori è illegale: libopus accetta e limita da sé (§1.3). L'avviso serve solo a dire che si sprecano byte senza guadagno, perché con Fs bassa l'encoder taglia la banda ma non abbassa il bitrate richiesto [I]. Anche 16 kbps stereo a 48 kHz è lecito: l'encoder ridurrà banda e stereo [I].

---

## 2. Crate backend

Versioni dall'API di crates.io (`https://crates.io/api/v1/crates/<nome>`, con User-Agent), lette il 2026-10-02.

| Crate | Ultima stabile | Data | Note |
|---|---|---|---|
| `opus` | 0.4.0 | 2026-08-23 | dipende da `opusic-sys ^0.7.3`, nessuna feature propria |
| `opusic-sys` | 0.7.5 | 2026-08-06 | feature `bundled` (default, usa cmake) |
| `opusic-c` | 1.6.1 | 2026-04-27 | binding di alto livello dello stesso autore |
| `audiopus` / `audiopus_sys` | 0.2.0 / 0.2.2 | 2021-04-22 | fermo dal 2021 |
| `opus-rs` (restsend) | 0.1.34 | 2026-09-22 | Opus pure Rust |
| `ropus` | 0.12.18 | 2026-05-11 | port fixed-point pure Rust |
| `ogg` | 0.9.2 | 2025-01-12 | pure Rust |
| `cpal` | 0.18.2 | 2026-08-16 | `windows` 0.62 |
| `rubato` | 5.0.1 | 2026-10-01 | MSRV 1.87 (changelog v5.0.0) |
| `reqwest` | 0.13.5 | 2026-09-08 | default TLS = rustls + aws-lc-rs |
| `sha2` | 0.11.0 | 2026-03-25 | `digest ^0.11` |
| `semver` | 1.0.28 | 2026-04-04 | |
| `tauri` | 2.12.1 | n/d | `tauri-utils` dipende da `semver ^1` |
| `tauri-plugin-dialog` | 2.8.1 | 2026-10-01 | dipende da `rfd ^0.16` |
| `tauri-plugin-store` | 2.5.0 | n/d | esiste 3.0.0-alpha.2 |
| `tauri-plugin-updater` | 2.13.1 | n/d | esiste 3.0.0-alpha.2 |
| `rfd` | 0.17.2 | 2026-01-12 | |

### 2.1 `opus` e alternative

- **[V]** `opus` 0.4.0 (SpaceManiac): il README dice "By default, you need `cmake` and a C compiler. These requirements come from opusic-sys" ([README](https://github.com/SpaceManiac/opus-rs)). La documentazione upstream di riferimento è [opus_api-1.6](https://opus-codec.org/docs/opus_api-1.6/).
- **[V]** `opusic-sys` incorpora libopus **1.6.1** (con patch documentate in `opus.patch`), compila con cmake (usa Ninja se presente) e linka **statico** (`rustc-link-lib=static=opus`). Con `default-features = false` cerca libopus nel `PATH` o in `OPUS_LIB_DIR`, con `OPUS_LIB_STATIC=true` per preferire il link statico. Requisito: `cmake` ([README](https://github.com/DoumanAsh/opusic-sys), [build.rs](https://github.com/DoumanAsh/opusic-sys/blob/master/build.rs)). Non ho trovato vcpkg nel build script. Si può usare indirettamente: libopus installata da vcpkg e `OPUS_LIB_DIR` puntato lì [I].
- **[I]** Siccome `opus` 0.4.0 dipende da `opusic-sys` con le feature di default e non espone feature proprie, **con il crate `opus` libopus viene sempre compilata da sorgente via cmake**. Su MSVC serve quindi cmake nel PATH: i Build Tools di Visual Studio lo includono nel componente "C++ CMake tools", da verificare sulla macchina di build.
- **[V]** API di `opus` 0.4.0: `Encoder::new(sample_rate: u32, Channels, Application)`, `encode(&[i16], &mut [u8])`, `encode_float(&[f32], &mut [u8])`, `set_bitrate(Bitrate::Bits(i32) | Max | Auto)`, `set_vbr`, `set_vbr_constraint`, `set_complexity`, `set_signal`, `set_max_bandwidth`, `get_lookahead`, `set_expert_frame_duration(FrameSize)`. `Application` può essere `Voip | Audio | LowDelay` ([src/lib.rs](https://github.com/SpaceManiac/opus-rs/blob/master/src/lib.rs)). Non c'è un muxer Ogg: serve `ogg`.
- **[V]** `audiopus_sys` 0.2.2 compila con cmake/pkg-config ed è fermo dal 2021 (crates.io). Lo sconsiglio per l'età [I].
- **[V]** `opus-rs` (restsend): "pure-Rust implementation … ported from the reference C implementation (libopus 1.6)", "no C dependencies", si dichiara "Production-ready". API `OpusEncoder::new(16000, 1, Application::Voip)`, `encode(&[f32], frame, &mut out)`, `encode_i16`. I benchmark del README sono dell'autore, non indipendenti ([README](https://github.com/restsend/opus-rs)). `ropus`: port fixed-point "bit-exact against the reference" (descrizione crates.io), con pochi download (~5,7k).
- **[I] Proposta**: `opus` 0.4.0. È il binding più usato (~2,5 M download), porta libopus 1.6.1 ufficiale e il link è statico, quindi l'installer non deve distribuire DLL. Il costo è cmake in fase di build. `opus-rs` è il piano B se cmake su MSVC dà problemi; prima andrebbe verificata la conformità dell'output con `opusinfo`/`opusdec`.

### 2.2 `ogg` e scrittura di un file Ogg Opus

- **[V]** `ogg` 0.9.2: `PacketWriter::new(W: Write)` e `write_packet(data, serial, PacketWriteEndInfo, absgp: u64)`, dove `PacketWriteEndInfo` può essere `NormalPacket | EndPage | EndStream`. La pagina si chiude da sola quando raggiunge 255 segmenti di lacing, oppure con `EndPage`/`EndStream`. La granule scritta nell'header di pagina è l'`absgp` dell'ultimo pacchetto che **termina** su quella pagina (-1 se nessuno termina). Il CRC è calcolato dal crate. I flag BOS/EOS/continued sono impostati in automatico ([src/writing.rs](https://github.com/RustAudio/ogg/blob/master/src/writing.rs)).
- **[I]** Ricetta per un file valido:
  1. `OpusHead`: 19 byte con family 0. Pre-skip = lookahead convertito a 48 kHz. Input rate = Fs reale. Gain = 0. Scriverlo con `EndPage` e absgp 0.
  2. `OpusTags`: vendor string (es. `opus::version()`), anche con 0 commenti. Scriverlo con `EndPage` e absgp 0.
  3. Ogni pacchetto audio con absgp = `pre_skip + campioni_48k_cumulativi_fino_alla_fine_del_pacchetto`.
  4. L'ultimo pacchetto con `EndStream` e absgp = `pre_skip + ceil(campioni_reali * 48000 / Fs)` (end trimming).
  5. Serial casuale a 32 bit.
- **[I]** Con pacchetti piccoli (~1 segmento ciascuno) la pagina si chiude solo dopo ~255 pacchetti, cioè ~5 s a 20 ms. Per limitare la perdita in caso di crash conviene forzare `EndPage` ogni ~1 s. Un file senza EOS (crash) resta in gran parte decodificabile, ma la durata può risultare sbagliata (da verificare con un player).
- Nota: RFC 7845 §6 consiglia ai demuxer di tollerare pacchetti fino a 61 440 byte. Con pacchetti da 20 ms (≤ 1276 byte) il problema non si pone [V]/[I].

### 2.3 `cpal` 0.18.2 (WASAPI)

Verificato nel sorgente del tag `v0.18.2`: [host/wasapi/mod.rs](https://github.com/RustAudio/cpal/blob/v0.18.2/src/host/wasapi/mod.rs), [device.rs](https://github.com/RustAudio/cpal/blob/v0.18.2/src/host/wasapi/device.rs), [stream.rs](https://github.com/RustAudio/cpal/blob/v0.18.2/src/host/wasapi/stream.rs), [CHANGELOG](https://github.com/RustAudio/cpal/blob/v0.18.2/CHANGELOG.md).

- **[V] Loopback**: la documentazione dell'host dice "If you use a WASAPI output device as an input device it will transparently enable loopback mode". In `build_input_stream_raw_inner`: `if self.data_flow() == Audio::eRender { stream_flags |= AUDCLNT_STREAMFLAGS_LOOPBACK; }`, insieme a `AUDCLNT_STREAMFLAGS_EVENTCALLBACK` e in shared mode.
- **[V] Trappole** su un device di output:
  - `supported_input_configs()` restituisce una lista **vuota**, quindi `supports_input()` è `false`.
  - `default_input_config()` restituisce **errore** ("Device does not support input").
  - La config del loopback va presa da `default_output_config()`, che corrisponde a `GetMixFormat`.
  - Gli stream di capture **non** usano `AUTOCONVERTPCM`: "only native formats will work". Il ricampionamento a 16 kHz va quindi fatto nell'app.
- **[V] Timestamp**: `InputCallbackInfo::timestamp().capture` viene dal `qpc_position` di `IAudioCaptureClient::GetBuffer` (unità da 100 ns), convertito in nanosecondi. `callback` e `Stream::now()` usano `QueryPerformanceCounter`, sulla stessa griglia da 100 ns. **Sì, deriva da QPC**: mic e loopback hanno la stessa base temporale e si possono allineare.
- **[V] Discontinuità**: `AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY` (tranne sul primo buffer) è riportato come `ErrorKind::Xrun` all'error callback (novità 0.18.2). Se `GetNextPacketSize()` restituisce 0 il ciclo esce e la data callback **non** viene chiamata.
- **[V] Enumerazione**: `host.devices()`, `input_devices()`, `output_devices()`, `default_input_device()`, `default_output_device()`, `device.id()` (`DeviceId` stabile tra riavvii "where possible"), `host.device_by_id(&id)`, `device.description()`. Il nome si ottiene con `device.to_string()`. `DeviceTrait::name()` è stato rimosso in 0.18 ([traits.rs](https://github.com/RustAudio/cpal/blob/v0.18.2/src/traits.rs)). Su WASAPI il nome preferito è il `FriendlyName`.
- **[V] Default device** (0.18.0): "Default output and input streams now automatically reroute when the system default device changes". Gli stream non partono più da soli: serve `play()`. La config di default preferisce 48 kHz, poi 44,1 kHz.
- **[V] Loopback senza riproduzione**:
  - Microsoft documenta che il loopback event-driven è supportato da Windows 10 1703 e che il loopback funziona solo in shared mode ([Loopback Recording](https://learn.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording)). La pagina non dice cosa succede durante il silenzio.
  - Matthew van Eerde (team audio Microsoft, blog 2008) scrive: "WASAPI will only push data down to the render endpoint when there are active streams. When nothing is playing, there is nothing to capture". Come workaround suggerisce di riprodurre silenzio ([post](https://matthewvaneerde.wordpress.com/2008/12/16/sample-wasapi-loopback-capture-record-what-you-hear/)).
- **[I]** Conseguenza per Sbobino: durante il silenzio di sistema la callback di loopback non arriva e restano buchi. Due strade:
  - (a) riempire i buchi con zeri, usando i timestamp QPC (`capture`) rispetto all'orologio del mic;
  - (b) aprire uno stream di output che scrive silenzio sullo stesso device.
  
  La (a) non tocca il sistema audio ed è coerente con QPC. Da verificare su Windows 11 attuale, perché la fonte è del 2008.
- **[I]** Mic e device di output hanno clock hardware distinti, quindi c'è drift. Per mixare sessioni lunghe può servire un resampler `Async` di rubato con ratio regolato sui timestamp, oppure `Slip`.

### 2.4 `rubato` 5.0.1

- **[V]** **`FftFixedIn` non esiste più.** La v1.0.0 ha introdotto una "New API using the AudioAdapter crate" e "Merged the FixedIn, FixedOut and FixedInOut resamplers into single types". Oggi esistono `Fft` (sincrono), `Async` (`new_sinc` / `new_poly`) e `Slip` ([README e changelog](https://github.com/HEnquist/rubato/blob/v5.0.1/README.md)).
- **[V]** L'equivalente di `FftFixedIn` è `Fft::<T>::new(sample_rate_input, sample_rate_output, chunk_size, nbr_channels, FixedSync::Input)`, con `FixedSync::{Input, Output, Both}` ([src/synchro.rs](https://github.com/HEnquist/rubato/blob/v5.0.1/src/synchro.rs)). `Fft::new_custom` espone `sub_chunks` e la finestra (v4.0.0).
- **[V]** Elaborazione: `process_into_buffer(&input_adapter, &mut output_adapter, Option<&Indexing>)` con adapter di `audioadapter-buffers` (es. `InterleavedSlice::new(&buf, channels, frames)`). Va chiamato `input_frames_next()` prima di fornire i dati. L'ultimo chunk corto si gestisce con `Indexing::partial_len`. Per clip intere esiste `process_all()`. Il README suggerisce `SequentialSliceOfVecs` per chi migra da v0.16 e sconsiglia di fare resampling dentro la callback audio.
- **[V]** v5.0.1 (2026-10-01) corregge `process_all` e `process_all_into_buffer`, che lasciavano frame stantii all'inizio. v5.0.0 richiede rustc ≥ 1.87 e `audioadapter` 5.0.

### 2.5 `reqwest` 0.13.5, `sha2` 0.11, `semver` 1.0.28

- **[V]** Default features di reqwest: `default-tls`, `charset`, `http2`, `system-proxy`. **`default-tls` = `rustls`**, e `rustls` attiva `__rustls-aws-lc-rs` e `rustls-platform-verifier` (feature map da crates.io). Il changelog 0.13.0: "`rustls` is now the default TLS backend, instead of `native-tls`", il provider di default è aws-lc, `rustls-tls` è stato rinominato `rustls` ([CHANGELOG](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md)). Nota: la pagina docs.rs descrive ancora `default-tls` come "platform's native": è in contrasto con Cargo.toml e changelog, che hanno la precedenza.
- **[V]** aws-lc-sys su `x86_64-pc-windows-msvc` (non FIPS) richiede compilatore C/C++ e NASM. CMake serve solo per FIPS. Gli oggetti NASM precompilati sono "only used as a fallback when NASM is not available", oppure si attivano con `prebuilt-nasm` o `AWS_LC_SYS_PREBUILT_NASM=1` ([aws-lc-rs requirements Windows](https://aws.github.io/aws-lc-rs/requirements/windows.html)).
- **[V]** `native-tls` su Windows usa Schannel ([docs.rs reqwest 0.13.5](https://docs.rs/reqwest/0.13.5/reqwest/)). Dalla 0.13.4 ha TLS 1.3 (changelog).
- **[V]** Download in streaming con progresso: `Response::chunk() -> Result<Option<Bytes>>` **non richiede feature**. `bytes_stream()` richiede la feature `stream`. `content_length()` fornisce il totale per la percentuale ([Response](https://docs.rs/reqwest/0.13.5/reqwest/struct.Response.html)). Il client asincrono richiede Tokio, che Tauri fornisce già con `tauri::async_runtime` [I].
- **[I] Proposta TLS**:
  - **Opzione A**: `reqwest = { version = "0.13", default-features = false, features = ["native-tls", "http2", "system-proxy"] }`. Usa Schannel e il trust store di Windows, niente NASM, binario più piccolo.
  - **Opzione B**: default (rustls + aws-lc + platform verifier). È cross-platform ma aggiunge aws-lc-sys alla build MSVC.
  
  L'app è solo Windows, quindi propendo per **A**. Se `tauri-plugin-updater` entrasse nel progetto porterebbe comunque reqwest 0.13 con le sue feature (updater 2.13.1 dipende da `reqwest ^0.13`).
- **[V]** `sha2` 0.11.0: `Sha256::new()`, `update`, `finalize` (trait `Digest`). Feature: `alloc`, `oid`, `zeroize`. **Non c'è una feature `std`** ([docs.rs sha2 0.11.0](https://docs.rs/sha2/0.11.0/sha2/), feature map su crates.io).
- **[I]** Senza `std` probabilmente non c'è `impl io::Write` per l'hasher: si fa `update(&chunk)` per ogni chunk di reqwest, che è comunque il flusso naturale per hash e progresso. Per l'output esadecimale non ho verificato che l'output di digest 0.11 implementi `LowerHex`. Bastano poche righe con `format!("{:02x}")` per byte.
- **[V]** `semver` 1.0.28: `Version::parse` richiede `MAJOR.MINOR.PATCH` e rifiuta zeri iniziali e pre-release/build vuoti. `Ord` include anche i metadata di build, mentre `cmp_precedence` li ignora ([docs.rs](https://docs.rs/semver/1.0.28/semver/struct.Version.html)). `tauri::PackageInfo.version` è già un `semver::Version` ([docs.rs PackageInfo](https://docs.rs/tauri/latest/tauri/struct.PackageInfo.html)).
- **[I]** La documentazione non dice nulla sul `v` iniziale. La grammatica SemVer non lo prevede, quindi va tolto da `tag_name` prima del parse (`trim_start_matches('v')`). Per confrontare due versioni conviene `cmp_precedence`. Probabilmente non serve aggiungere `semver` come dipendenza diretta, perché Tauri lo usa già (`tauri-utils` → `semver ^1`), ma per importarlo è più pulito dichiararlo comunque.

---

## 3. Decisioni minori

### 3.1 Dialog di apertura: `tauri-plugin-dialog` vs `rfd`

Fatti:
- **[V]** `tauri-plugin-dialog` 2.8.1 **usa rfd** (`rfd = "0.16"`, feature `common-controls-v6`) e dipende da `tauri-plugin-fs` ([Cargo.toml](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/dialog/Cargo.toml), crates.io).
- **[V]** Si può chiamare da Rust: `app.dialog().file().add_filter("Audio", &["wav","mp3"]).pick_file(|p| …)` (non bloccante), oppure `blocking_pick_file()`. Quest'ultima "should *NOT* be used when running on the main thread": va usata nei comandi `async`. Restituisce `Option<FilePath>` ([src/lib.rs](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/dialog/src/lib.rs), [doc](https://v2.tauri.app/plugin/dialog/)).
- **[V]** Si può chiamare anche da JS: `open({ multiple, directory, filters: [{ name, extensions }] })` di `@tauri-apps/plugin-dialog` 2.8.1. Serve la capability: il permesso `dialog:default` include `allow-message`, `allow-save` e `allow-open` ([permissions/default.toml](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/dialog/permissions/default.toml)).
- **[V]** `rfd` 0.17.2 è l'ultima versione. Usandolo direttamente accanto al plugin si compilerebbero due versioni di rfd (0.16 e 0.17) [I].

Pro/contro [I]:
- **Plugin**: API Rust e JS, parent window gestita (`set_parent`), integrazione con le capabilities. Contro: si porta dietro `tauri-plugin-fs` come dipendenza.
- **rfd diretto**: nessun plugin né permessi JS. Contro: niente API JS, parent HWND e thread vanno gestiti a mano, e c'è il rischio di versione doppia se in futuro entra il plugin.

**Proposta [I]**: `tauri-plugin-dialog`, chiamato **da Rust** dentro un comando `async`, con filtri per estensione definiti nel backend. Il frontend chiama solo il comando dell'app, quindi non serve concedere `dialog:allow-open` alla webview. È il dialog "di fatto" di Tauri e usa già rfd.

### 3.2 Persistenza impostazioni: `tauri-plugin-store` vs JSON gestito da Rust

Fatti:
- **[V]** Lo store risolve i percorsi rispetto a `BaseDirectory::AppData` (`resolve_store_path`) ([src/store.rs](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/store/src/store.rs)). `app_data_dir` = `data_dir/${bundle_identifier}`, che su Windows è `{FOLDERID_RoamingAppData}`, cioè `%APPDATA%\<identifier>`. `app_local_data_dir` invece è `%LOCALAPPDATA%\<identifier>`, che è anche la cartella dati della webview ([PathResolver, tauri 2.12.1](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html)).
- **[V]** Lo store è accessibile da Rust in `setup` (`app.store("store.json")?`), quindi prima del frontend, e condivide l'istanza con JS. I valori sono `serde_json::Value`. Ha `defaults`, `auto_save(debounce)` e `serialize`/`deserialize` personalizzabili. Senza `autoSave: false` salva con un debounce di 100 ms e all'uscita ordinata ([doc store](https://v2.tauri.app/plugin/store/), sorgente).
- **[V]** Esiste la 3.0.0-alpha.2 (crates.io), quindi l'API potrebbe cambiare.

Pro/contro [I]:
- **Store**: zero codice di I/O, accesso anche da JS. Contro: valori non tipizzati e nessuna validazione o migrazione integrata. Dà anche accesso in scrittura alla webview se si concedono i permessi. Dipende inoltre da una major in arrivo.
- **JSON da Rust**: `struct Settings` con `serde` (`#[serde(default)]` per i campi nuovi e un campo `version` per le migrazioni), letta in modo sincrono in `setup` da `app.path().app_config_dir()` e scritta in modo atomico (file temporaneo + rename). È esposta con `get_settings`/`set_settings`, che validano. Contro: ~50 righe di codice nostro.

**Proposta [I]**: JSON gestito da Rust. Le impostazioni (Fs, bitrate, device, lingua) vanno validate lato backend, come i vincoli bitrate/Fs del §1.5, e lette prima che parta qualunque altra cosa. Lo store non aggiunge nulla che `serde_json` + `std::fs` non facciano già.

### 3.3 i18n per React 19 (6 lingue, lingua applicata al riavvio)

Versioni npm (registry, 2026-10-02): `i18next` 26.4.2 + `react-i18next` 17.0.15. `@lingui/core` / `@lingui/react` / `@lingui/vite-plugin` 6.9.0. `@inlang/paraglide-js` 2.25.4. `react-intl` 12.1.3. `use-intl` 4.14.9. `typesafe-i18n` 5.27.1, con l'ultimo rilascio a febbraio 2026. Vite 8.3.2, `@vitejs/plugin-react` 6.1.1 (Vite 8), React 19.3.0.

**Plurali [V]**: `Intl.PluralRules` (Node 25, ICU 77 / CLDR) restituisce queste categorie:

| Lingua | Categorie |
|---|---|
| it | one, many, other |
| en | one, other |
| fr | one, many, other |
| es | one, many, other |
| de | one, other |
| pl | one, few, many, other |

In polacco: 2 è `few`, 5 è `many`, 22 è `few`, 25 è `many`, 1,5 è `other`. Anche it/fr/es hanno `many` (per 1 000 000), cosa che si dimentica spesso.

| | i18next + react-i18next | Lingui 6 | Paraglide JS 2 |
|---|---|---|---|
| Tipizzazione chiavi | [V] augmentation `CustomTypeOptions.resources` ([doc](https://github.com/i18next/i18next)): controlla chiavi e interpolazioni | [V] guida "typed message IDs" ([lingui.dev](https://lingui.dev/guides/typed-message-ids)) | [I] i messaggi sono funzioni generate (`m.key(params)`), quindi tipizzazione nativa, con `emitTsDeclarations` (TS ≥ 5.6) [V] |
| Plurali | [V] suffissi `_one/_few/_many/_other` scelti con `Intl.PluralRules` (PluralResolver) | [V] ICU MessageFormat (`plural()` / `{count, plural, …}`) | [V] declarations `local x = count: plural` + `match`, oppure plugin ICU |
| Build/Vite | [V] runtime puro, nessun plugin | [V] serve una trasformazione delle macro. Consigliato: `@lingui/vite-plugin` con `macroTransform: true` ("doesn't need" babel/swc plugin), compatibile con Vite 8 ([installation](https://lingui.dev/installation)). Catalogi `.po` estratti con la CLI | [V] `paraglideVitePlugin({ project, outdir })` compila i messaggi ([README](https://github.com/opral/paraglide-js)) |
| Bundle | [I] runtime più pesante dei due compilati. Non misurato | [V] "3 kb" (autodescrizione) | [V] "tree-shakable", "up to 70%" più piccolo (autodescrizione) |
| Cambio lingua | runtime, a caldo | runtime (`i18n.activate`) | [V] `setLocale` + strategie (`localStorage`, custom con `getLocale`/`setLocale` **sincroni**) |

Con Bun come package manager e runner non vedo vincoli specifici: sono tutti pacchetti npm standard e la build la fa Vite [I]. La CLI di Lingui ha `esbuild`/`rolldown` tra le peer [V].

**Proposta [I]**: **i18next + react-i18next**, con i JSON per lingua importati staticamente (6 lingue, poche centinaia di stringhe in un'app desktop) e tipi presi da `it` come riferimento. È la soluzione con meno parti mobili: nessun compilatore né macro, gestione dei plurali polacchi documentata, nessun legame con la versione di Vite. Paraglide è la scelta migliore se si vuole la tipizzazione più forte e il bundle minimo, accettando un passo di compilazione. Lingui ha senso se si preferiscono i messaggi in sorgente con estrazione automatica.

**Dove leggere la lingua all'avvio [I]**:
- (1) Rust legge le impostazioni in `setup` e le espone con `get_settings`; in `main.tsx` si fa `await invoke(...)`, poi `i18n.init({ lng })`, poi `createRoot().render`. È semplice, ma l'avvio aspetta una IPC.
- (2) Rust inietta `window.__SBOBINO_BOOT__ = { lang }` con `WebviewWindowBuilder::initialization_script`, che gira "before the HTML document is parsed and before any other included script runs" ([docs.rs](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)). La lingua è disponibile in modo sincrono, e questo serve a Paraglide, il cui `getLocale` è sincrono. Richiede però di creare la finestra da codice (es. `create: false` nella config + `WebviewWindowBuilder::from_config`, da verificare).

Con "lingua applicata al riavvio" la (1) è sufficiente e più semplice. La (2) diventa interessante solo con Paraglide.

### 3.4 Controllo aggiornamenti

Fatti:
- **[V]** `GET /repos/{owner}/{repo}/releases/latest`: "the most recent non-prerelease, non-draft release, sorted by the created_at attribute". Restituisce 404 se non c'è nessuna release. Campi `tag_name`, `html_url`, `prerelease`, `draft`, `name`, `body`, `assets` ([GitHub REST releases](https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release)). Per includere le prerelease serve `GET /repos/{owner}/{repo}/releases` e un filtro lato client [I].
- **[V]** Limite di rate non autenticato: "60 requests per hour" per IP. Header `x-ratelimit-remaining` / `x-ratelimit-reset`, risposta 403 o 429 quando si supera ([rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api?apiVersion=2022-11-28)). Le richieste condizionali con risposta 304 non contano "if … made while correctly authorized with an `Authorization` header", quindi per un client anonimo **contano comunque** ([best practices](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api?apiVersion=2022-11-28)).
- **[V]** "Requests with no `User-Agent` header will be rejected". Consigliati `Accept: application/vnd.github+json` e `X-GitHub-Api-Version` ([getting started](https://docs.github.com/en/rest/using-the-rest-api/getting-started-with-the-rest-api?apiVersion=2022-11-28)).
- **[V]** Versione dell'app: `app.package_info().version` (`semver::Version`).
- **[V]** `tauri-plugin-updater`: "needs a signature … This cannot be disabled". `pubkey` è obbligatoria nella config e l'endpoint deve servire un JSON (statico `latest.json`, con `version` e `platforms.<target>.url`/`signature`) oppure un server dinamico. `check()` restituisce un `Update` con `version`/`date`/`body` senza installare ([doc updater](https://v2.tauri.app/plugin/updater/)). Esiste la 3.0.0-alpha.2.

**Proposta [I]**: chiamata diretta all'API GitHub da Rust con reqwest (già presente per i download), al massimo una volta all'avvio o una volta al giorno, con UA `Sbobino/<versione>`. Poi `semver::Version::parse(tag.trim_start_matches('v'))` confrontato con `package_info().version`, e se la release è più recente si mostra un link a `html_url`. Errori, 403/429 e 404 si ignorano in silenzio. L'updater è sproporzionato per "proporre un link": richiede firma, chiavi, artefatti e `latest.json` in CI per una funzione che l'app non usa (l'installazione).

---

## Incertezze e punti da verificare

1. **Pre-skip con encoder a Fs < 48 kHz**: la conversione `lookahead*48000/Fs` è un'inferenza dal sorgente. libopusenc evita il problema lavorando sempre a 48 kHz. Va verificata con `opusinfo` su un file a 16 kHz.
2. **Loopback durante il silenzio**: la fonte (van Eerde) è del 2008 e la doc Microsoft attuale non ne parla. Va provato su Windows 11 se la callback cpal si ferma davvero o arrivano buffer con `AUDCLNT_BUFFERFLAGS_SILENT`.
3. **cmake su MSVC**: va verificato che la macchina di build e la CI lo abbiano nel PATH (il crate `cmake` lo cerca lì).
4. **Soglie di avviso bitrate/Fs (§1.5)**: sono euristiche mie, non limiti di libopus.
5. **i18next con `_many` mancante** (it/fr/es per numeri ≥ 10⁶): non ho verificato il fallback (verso `_other` o verso la chiave base). Da verificare oppure fornire sempre `_many`.
6. **`LowerHex` sull'output di sha2 0.11** e **`impl io::Write`**: non verificati.
7. **`tauri.conf.json` `create: false`** per creare la finestra da codice (strada 2 del §3.3): non verificato in questa sessione.
8. **Bundle size delle librerie i18n**: ci sono solo le autodichiarazioni dei progetti, nessuna misura.
9. **Conflitto nella doc reqwest**: `default-tls` è descritto come "native" su docs.rs ma è `rustls` nel Cargo.toml. Ho seguito Cargo.toml e changelog.
