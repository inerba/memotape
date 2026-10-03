# Decodifica audio da file video/audio: ffmpeg sidecar o librerie Rust

Data ricerca: 2026-10-02. Contesto: Sbobino, Tauri 2 + React, backend Rust MSVC, solo Windows x64, installer NSIS.
Requisiti: (1) PCM mono 16 kHz per la trascrizione, decodificato il più velocemente possibile; (2) "Estrai solo audio" in
OGG/Opus con bitrate, canali (mono|stereo) e frequenza (8/16/24/48 kHz) scelti dall'utente, con avanzamento in %.

Legenda: **[F]** = fatto verificato sulla fonte citata; **[I]** = inferenza mia (non verificata direttamente).

---

## 1. Librerie Rust pure

### 1.1 Symphonia (versione corrente 0.6.1)

- **[F]** Ultima versione stabile: `symphonia` 0.6.1, pubblicata il 2026-08-13 (0.6.0 il 2026-05-15). MSRV 1.85. Licenza MPL-2.0.
  Fonti: API crates.io `https://crates.io/api/v1/crates/symphonia`; README al tag `v0.6.1`
  (`https://raw.githubusercontent.com/pdeljanov/Symphonia/v0.6.1/README.md`).
- **[F]** Demuxer (README v0.6.1):

  | Formato | Stato | Feature | Default |
  |---|---|---|---|
  | AIFF | Great | `aiff` | No |
  | CAF | Good | `caf` | No |
  | ISO/MP4 | Great | `isomp4` | No |
  | MKV/WebM | Good | `mkv` | Sì |
  | OGG | Great | `ogg` | Sì |
  | Wave | Excellent | `wav` | Sì |

  In più: `AdtsReader` (AAC grezzo .aac) nel crate `symphonia-codec-aac` (docs.rs `symphonia-codec-aac` 0.6.1) e il reader
  MPEG audio (MP1/2/3) nel `symphonia-bundle-mp3` ("A `symphonia-bundle-*` package is a combination of a decoder and a
  native demuxer", README). `symphonia-format-riff` 0.6.1 esporta solo `AiffReader` e `WavReader`: **nessun AVI** (docs.rs).
- **[F]** Decoder (README v0.6.1):

  | Codec | Stato | Feature |
  |---|---|---|
  | AAC-LC | Great | `aac` |
  | HE-AAC / HE-AACv2 | **–** (non pronto) | `he-aac`, `he-aac-v2` |
  | ADPCM | Good | `adpcm` |
  | ALAC | Great | `alac` |
  | FLAC | Excellent | `flac` |
  | MP1 / MP2 | Great | `mp1`, `mp2` |
  | MP3 | Excellent | `mp3` |
  | **Opus** | **–** ("In work or not started yet") | `opus` |
  | PCM | Excellent | `pcm` |
  | Vorbis | Excellent | `vorbis` |
  | WavPack | – | `wavpack` |

  Le feature effettivamente pubblicate su crates.io per 0.6.1 sono: `aac, adpcm, aiff, alac, caf, flac, isomp4, mkv, mp1,
  mp2, mp3, ogg, pcm, vorbis, wav` (+ meta/SIMD); **non esiste una feature `opus` pubblicata** (API crates.io
  `/crates/symphonia/0.6.1`, campo `features`).
- **[F]** Nessun decoder AC-3/E-AC-3, WMA, DTS. Il demuxer MKV mappa `A_AC3`, `A_EAC3`, `A_DTS`, `A_TRUEHD` ecc. a codec ID
  (`symphonia-format-mkv/src/codecs.rs` al tag v0.6.1) e il demuxer MP4 contiene gli atom `dac3.rs`/`dec3.rs`/`opus.rs`
  (albero `symphonia-format-isomp4/src/atoms/` al tag v0.6.1), ma senza un decoder registrato quelle tracce non si decodificano.
- **[F]** Decoder di terze parti elencati nel README v0.6.1: `symphonia-adapter-libopus` (Opus via libopus) e
  `symphonia-adapter-fdk-aac` (HE-AAC via libfdk-aac). `symphonia-adapter-libopus` 0.3.0 (2026-05-16, MIT OR Apache-2.0)
  dipende da `opusic-sys ^0.7.3`, **la stessa dipendenza del crate `opus` 0.4.0** già nello stack (API crates.io,
  `/dependencies`). **[I]** Quindi aggiungere Opus a Symphonia non introduce una seconda copia di libopus.
  `symphonia-adapter-fdk-aac` è `(MIT OR Apache-2.0) AND MPL-2.0` lato crate, ma libfdk-aac ha licenza propria:
  FFmpeg la classifica incompatibile con la GPL (`LICENSE.md` di FFmpeg, sezione "Incompatible libraries").
- **[F]** Prestazioni: "Symphonia's decoders are generally +/-15% the performance of FFMpeg" (README v0.6.1). SIMD attivo di
  default in 0.6.x (`opt-simd-*` Default: Yes).
- **[I]** Symphonia non ricampiona: per 16 kHz mono serve un resampler a parte (es. `rubato` 5.0.1, MIT/Apache, crates.io).

### 1.2 Altri crate Rust (demuxer/decoder mancanti)

Dati da API crates.io (versione, data ultimo aggiornamento, download totali), letti il 2026-10-02.

| Bisogno | Crate | Versione / data | Download | Nota |
|---|---|---|---|---|
| Demux MKV | `matroska-demuxer` | 0.8.1 / 2026-08-07 | 174k | maturo, ma Symphonia copre già MKV |
| Demux MPEG-TS | `mpeg2ts-reader` | 0.18.2 / 2025-01-29 | 110k | parser di pacchetti/PES, non un demuxer "media" pronto: la colla verso i decoder va scritta |
| Demux MPEG-TS | `mpeg2ts` | 0.6.1 / 2026-08-11 | 96k | idem |
| Demux FLV | `scuffle-flv` | 0.2.2 / 2025-05-17 | 5.6k | piccolo |
| Demux ASF/WMV | `wmvkit` | 0.1.0 / 2025-12-24 | 386 | MPL-2.0, adozione minima |
| Demux AVI / FLV / TS | `oxideav-avi` / `oxideav-flv` / `oxideav-mpegts` | 0.0.10 / 0.0.5 / 0.0.3 (2026) | 2.1k / 1.2k / 0.4k | serie 0.0.x |
| Decoder AC-3/E-AC-3 | `oxideav-ac3` | 0.0.11 / 2026-09-01 | 3.6k | MIT; dichiara di non essere bit-exact ("not bit-exact … but it is deterministic", README GitHub OxideAV/oxideav-ac3) |
| Decoder WMA | `oxideav-wma` | 0.0.4 / 2026-09-01 | 180 | solo WMA v1/v2; il README lo definisce "clean-room rebuild in progress"; Pro/Lossless assenti |

- **[I]** Nessuno di questi è maturo come Symphonia o FFmpeg per AC-3/WMA/ASF/AVI/FLV. Comporre 4-6 crate eterogenei
  (API diverse, nessun registro comune) per coprire i formati video è un progetto in sé.

### 1.3 Binding a FFmpeg nativo (`ffmpeg-next` / `ffmpeg-sys-next`)

- **[F]** `ffmpeg-next` / `ffmpeg-sys-next` 9.0.0 (2026-08-05), licenza dei binding WTFPL (crates.io). Il README dichiara
  "maintenance-only mode for the most part" e che "any PR to improve existing API is unlikely to be merged"; supporto
  dichiarato per FFmpeg "from 3.4 til 8.0" (README `github.com/zmwangx/rust-ffmpeg`). **[I]** La 9.0.0 del crate
  probabilmente aggiunge FFmpeg 9.x, ma non l'ho verificato.
- **[F]** Linking su MSVC (`build.rs` di `rust-ffmpeg-sys`, wiki "Notes on building"):
  - richiede **FFmpeg con header e import lib** puntati da `FFMPEG_DIR` (oppure trovati via **vcpkg**, `try_vcpkg` solo con
    `target_env = "msvc"`), più **LLVM/libclang** per bindgen (`LIBCLANG_PATH`);
  - la feature `build` (compilare FFmpeg dal sorgente) su Windows richiede `sh.exe` ("Failed to find 'sh.exe', which is
    required for building FFmpeg") e usa `--toolchain=msvc`: in pratica MSYS2 + MSVC;
  - feature `static` per il linking statico.
- **[F]** Per restare in LGPL FFmpeg chiede "Use dynamic linking (on windows, this means linking to dlls)" (checklist su
  `ffmpeg.org/legal.html`). **[I]** Quindi con `ffmpeg-next` si spediscono comunque le DLL `avcodec/avformat/avutil/
  swresample` accanto all'exe, con gli stessi obblighi del sidecar, più la complessità di build (libclang, header, import
  lib, versioni ABI allineate). Rispetto al sidecar si guadagna l'API in-process e si perde l'isolamento dei crash.

### 1.4 Copertura per estensione (senza FFmpeg)

Container/codec tipici: **[F]** dove citata la fonte, altrimenti **[I]** (conoscenza comune dei formati).

| Estensione | Container | Codec audio tipici | Symphonia 0.6.1 (+ adapter libopus) |
|---|---|---|---|
| MP3 | MPEG audio | MP3 | ✅ |
| WAV | RIFF/WAVE | PCM, ADPCM | ✅ (codec esotici in WAV no) |
| M4A | ISO/MP4 | AAC-LC, HE-AAC, ALAC | ⚠️ HE-AAC non supportato |
| FLAC | FLAC | FLAC | ✅ |
| OGG | Ogg | Vorbis, Opus, FLAC | ✅ Vorbis; Opus solo con `symphonia-adapter-libopus` |
| OPUS | Ogg | Opus | ✅ solo con adapter libopus |
| WEBM | Matroska/WebM | Opus, Vorbis | ✅ con adapter (MKV "Good") |
| MPGA | MPEG audio | MP1/MP2/MP3 | ✅ |
| MPEG | MPEG-1/2 Program Stream (o solo audio MPEG) | MP2, AC-3, LPCM | ❌ nessun demuxer PS (✅ solo se è audio MPEG puro) |
| AIFF | AIFF/AIFF-C | PCM | ✅ (AIFF-C compressi: [I] parziale) |
| MP4 / M4V / MOV | ISO/MP4 | AAC, AC-3/E-AC-3, ALAC, PCM, MP3 | ⚠️ AAC-LC/ALAC/PCM/MP3 sì; AC-3/E-AC-3 e HE-AAC no |
| MKV | Matroska | AAC, AC-3/E-AC-3, DTS, Opus, Vorbis, FLAC | ⚠️ AC-3/E-AC-3/DTS/TrueHD no |
| AVI | RIFF/AVI | MP3, PCM, AC-3 | ❌ nessun demuxer AVI |
| WMV | ASF | WMA (v2/Pro/Voice) | ❌ né ASF né WMA. [F] "ASF is the container format for Windows Media Audio and Windows Media Video-based content" (Microsoft Learn, *Overview of the ASF Format*) |
| FLV | FLV | AAC, MP3 (anche PCM, ADPCM, Nellymoser, Speex) | ❌ nessun demuxer. [F] codec ID FLV in `libavformat/flv.h` di FFmpeg |
| TS | MPEG-2 TS | AAC (ADTS/LATM), MP2/MP3, AC-3/E-AC-3 | ❌ nessun demuxer. [F] tabella stream type in `libavformat/mpegts.c` |
| MTS | MPEG-2 TS (M2TS, AVCHD) | AC-3, LPCM | ❌ nessun demuxer, nessun AC-3. [I] AVCHD = AC-3 o LPCM (avchd-info.org non raggiungibile durante la ricerca) |

**Conclusione [I]:** senza FFmpeg restano **scoperte del tutto 6 estensioni su 19** (MPEG, AVI, WMV, FLV, TS, MTS) e
**5 sono parziali** (MP4, M4V, MOV, MKV per AC-3/E-AC-3; M4A, MP4, M4V, MOV per HE-AAC). OPUS/WEBM/OGG-Opus richiedono comunque
libopus nativo (già nello stack). I crate esistenti per gli anelli mancanti sono in serie 0.0.x o molto poco adottati.

---

## 2. FFmpeg come sidecar Tauri

### 2.1 `externalBin` e risoluzione del percorso

- **[F]** Configurazione: `bundle.externalBin` in `tauri.conf.json`, percorsi relativi a `src-tauri`. Per ogni piattaforma
  deve esistere un file con suffisso `-$TARGET_TRIPLE`, triple ricavabile con `rustc --print host-tuple` (Rust ≥ 1.84)
  (Tauri docs, *Embedding External Binaries*, `tauri-docs/src/content/docs/develop/sidecar.mdx`, ramo v2). Per Sbobino:
  `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe` con `"externalBin": ["binaries/ffmpeg"]`.
- **[F]** In bundle e in build il suffisso viene rimosso: `copy_binaries` fa `.replace(&format!("-{}", self.target), "")`
  (`crates/tauri-bundler/src/bundle/settings.rs`); `tauri-build` copia i sidecar in `target/` per lo sviluppo
  (`crates/tauri-build/src/lib.rs`, `fn copy_binaries`). L'installer NSIS li installa in `$INSTDIR`
  (`File /a "/oname={{this}}"` nel template `nsis/installer.nsi`) e li rimuove alla disinstallazione.
- **[F]** `tauri-plugin-shell` (ultima stabile 2.4.0) risolve il sidecar come `current_exe().parent().join(nome)` + `.exe`
  (`relative_command_path` in `plugins/shell/src/process/mod.rs`, ramo v2).
- **[F]** Da Rust: `app.shell().sidecar("ffmpeg")` (solo il nome file, non il percorso di `externalBin`); il plugin va
  inizializzato. I **permessi/capability** (`shell:allow-execute` / `shell:allow-spawn` con `"sidecar": true` e validatori
  degli argomenti) sono descritti nella sezione "Running it from JavaScript" (sidecar.mdx).
  **[I]** Le capability regolano l'IPC dal webview; se ffmpeg è lanciato solo dal backend Rust non servono permessi shell
  nel frontend, ed è la configurazione più sicura (il frontend non può lanciare processi).
- **[F/I]** Serve il plugin? No. Il plugin fa solo: `StdCommand::new(path)`, stdin/stdout/stderr `piped`,
  `creation_flags(CREATE_NO_WINDOW)` su Windows (codice citato sopra), più un thread che legge le pipe e un canale di eventi.
  Su stdout legge **per righe** salvo `set_raw_out(true)` (`spawn_pipe_reader`/`read_raw_bytes`). **[I]** Per stream PCM
  binari ad alto throughput `std::process::Command` con il percorso risolto come sopra è più semplice e diretto (lettura
  bloccante a blocchi da `ChildStdout`), e risparmia una dipendenza. `externalBin` resta utile solo per il bundling.

### 2.2 Niente finestra console su Windows

- **[F]** `CREATE_NO_WINDOW = 0x08000000`: "The process is a console application that is being run without a console
  window" (Microsoft Learn, *Process Creation Flags*).
- **[F]** In Rust: `std::os::windows::process::CommandExt::creation_flags(0x08000000)`, stabile da Rust 1.16; i flag sono
  sempre messi in OR con `CREATE_UNICODE_ENVIRONMENT` (doc std). Lo fanno anche `tauri-plugin-shell` (const
  `CREATE_NO_WINDOW`) e il crate `ffmpeg-sidecar` (`create_no_window()`).

### 2.3 Comandi

Decodifica per trascrizione (PCM f32 mono 16 kHz su stdout):

```
ffmpeg -hide_banner -nostdin -i <file> -map 0:a:0 -vn -sn -dn -ac 1 -ar 16000 -f f32le -progress pipe:2 -nostats pipe:1
```

- **[F]** `f32le` = "PCM 32-bit floating-point little-endian" (ffmpeg-formats.html, demuxer/muxer raw PCM). `-ac` / `-ar`
  impostano canali e frequenza in uscita (ffmpeg.html, *Audio Options*). `-nostdin` disabilita l'interazione su stdin
  (ffmpeg.html).
- **[F]** Velocità: `-re` equivale a `-readrate 1` (lettura a velocità nativa) (ffmpeg.html). **[I]** Senza `-re`/
  `-readrate` ffmpeg non limita la lettura, quindi decodifica alla velocità del calcolo.
- **[I]** `-progress pipe:2` mette l'avanzamento su stderr, lasciando stdout ai campioni; in alternativa si può contare i
  campioni letti (byte / 4 / 16000 = secondi) senza parsing.

Estrai solo audio (OGG/Opus), due varianti:

```
ffmpeg -hide_banner -nostdin -i <file> -map 0:a:0 -vn -c:a libopus -b:a 32k -ac 1 -ar 16000 -progress pipe:1 -nostats -y out.ogg
```

- **[F]** libopus in FFmpeg: `b` in bit/s, `vbr` off/on/constrained, `application` voip/audio/lowdelay,
  `frame_duration`, `compression_level` 0-10 (ffmpeg-codecs.html, *libopus*). La doc **non** elenca le frequenze.
- **[F]** Frequenze accettate dall'encoder `libopus`: `libopus_sample_rates[] = { 48000, 24000, 16000, 12000, 8000 }`
  (`libavcodec/libopusenc.c`, `CODEC_SAMPLERATES_ARRAY`). Quindi **8/16/24/48 kHz sono tutte valide**.
- **[F]** In Ogg Opus il campo "Input Sample Rate" è solo informativo: "This field is _not_ the sample rate to use for
  playback", e la granule position è sempre a 48 kHz (RFC 7845 §5.1, §4). **[I]** Scegliere 16 kHz cambia la banda
  codificata, non la frequenza di riproduzione (i decoder riproducono normalmente a 48 kHz).
- **[F]** L'encoder Opus nativo di FFmpeg "only implements the CELT part of the codec. Its quality is usually worse"
  (ffmpeg-codecs.html): va usato `libopus`.
- **[I] Variante B (consigliata, v. §5):** ffmpeg decodifica soltanto (`-f f32le -ac <1|2> -ar <8000|16000|24000|48000>
  pipe:1`), e l'Ogg/Opus lo scrive il codice Rust già presente (`opus` + `ogg`). Una sola forma di invocazione per
  entrambe le funzioni, FFmpeg non ha bisogno di libopus.

### 2.4 Avanzamento in percentuale

- **[F]** `-progress url`: righe `key=value`, l'ultima chiave di ogni blocco è `progress=continue|end`; periodo impostato con
  `-stats_period` (default 0.5 s) (ffmpeg.html). Chiavi emesse (`fftools/ffmpeg.c`, `print_report`): `out_time_us`,
  `out_time_ms`, `out_time`, `total_size`, `bitrate`, `speed`, `progress`. Nota: **`out_time_ms` contiene lo stesso valore
  di `out_time_us`** (entrambi stampano `pts` in µs) → usare `out_time_us`.
- **[F]** Durata con ffprobe: `ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 <file>` (sintassi di
  `-show_entries` e opzioni `nokey`/`noprint_wrappers` in ffprobe.html).
- **[I]** Alternativa senza ffprobe: la riga `Duration: HH:MM:SS.cc` che ffmpeg stampa su stderr all'apertura dell'input.
  È log destinato alle persone, non un'interfaccia stabile, ma evita di spedire un secondo binario (nella build statica
  BtbN ffprobe.exe pesa altri ~128 MiB, v. §3.3). La durata può mancare (stream senza indice, TS troncati): l'interfaccia
  deve prevedere un avanzamento indeterminato.
- `% = out_time_us / (durata_s × 1e6)`.

### 2.5 Più tracce audio

- **[F]** Senza `-map`, per l'audio ffmpeg sceglie "the stream with the most channels", a parità il più basso indice
  (ffmpeg.html, *Automatic stream selection*). Con `-map` sono incluse solo le tracce mappate; `-map 0:a:0` = prima traccia
  audio; il suffisso `?` rende la mappa opzionale (ffmpeg.html, `-map`).
- **[I]** Per la trascrizione conviene `-map 0:a:0` o una traccia scelta dall'utente (elenco ottenuto via ffprobe o dal
  log): la regola "più canali" sceglie spesso la traccia 5.1 doppiata invece di quella originale stereo.
- **[I]** `-ac 1` fa il downmix attraverso swresample; per sorgenti 5.1 il canale centrale (voce) è incluso nel downmix.

---

## 3. Licenze e build Windows

### 3.1 LGPL vs GPL in FFmpeg

- **[F]** "FFmpeg is licensed under the GNU Lesser General Public License (LGPL) version 2.1 or later. However, FFmpeg
  incorporates several optional parts and optimizations that are covered by the GNU General Public License (GPL) version 2
  or later. If those parts get used the GPL applies to all of FFmpeg." (ffmpeg.org/legal.html).
- **[F]** Le parti GPL si attivano solo con `--enable-gpl`: alcune ottimizzazioni x86 e molti filtri video (elenco in
  `LICENSE.md`). Librerie esterne GPLv2 che obbligano a `--enable-gpl`: avisynth, frei0r, libcdio, libdavs2, librubberband,
  libvidstab, **libx264**, **libx265**, libxavs, libxavs2, **libxvid**. LGPLv3 (`--enable-version3`): gmp, libaribb24,
  liblensfun. Apache 2.0 (mbedTLS, OpenCORE, VisualOn, …) richiedono `--enable-version3`. libfdk-aac e OpenSSL sono
  incompatibili con GPL; `--enable-nonfree` rende il binario "unredistributable" (`LICENSE.md`).
- **[I]** Per Sbobino nessuna di queste serve: decoder e demuxer audio nativi, più eventualmente libopus, sono tutti LGPL/BSD.

### 3.2 Obblighi per distribuire un ffmpeg.exe LGPL accanto all'app

- **[F]** La checklist di legal.html (pensata per chi linka le librerie) chiede: build senza `--enable-gpl`/
  `--enable-nonfree`; DLL dinamiche; **distribuire il sorgente di FFmpeg anche se non modificato**, corrispondente
  esattamente ai binari, con `changes.diff` e la riga di configure, in tarball/zip, **sullo stesso server** del binario;
  attribuzione nella pagina di download, nella finestra "Informazioni" ("This software uses libraries from the FFmpeg project
  under the LGPLv2.1") e nell'EULA; niente divieti di reverse engineering nell'EULA; non rinominare le DLL in modo opaco;
  ripetere per ogni libreria LGPL esterna compilata dentro; nessuna libreria GPL ("notably libx264").
- **[F]** LGPL 2.1 §4: l'eseguibile si può distribuire se accompagnato dal sorgente, oppure offrendo "equivalent access to
  copy the source code from the same place" (`COPYING.LGPLv2.1` nel repo FFmpeg). LGPL 2.1 §2: "mere aggregation of
  another work not based on the Library … on a volume of a storage or distribution medium does not bring the other work
  under the scope of this License".
- **[F]** Le build BtbN "lgpl" sono **LGPL v3**: `defaults-lgpl.sh` contiene `FF_CONFIGURE="--enable-version3 …"` e
  `LICENSE_FILE="COPYING.LGPLv3"`. La LGPLv3 "incorporates the terms and conditions of version 3 of the GNU General Public
  License" (`COPYING.LGPLv3`); per l'oggetto distribuito vale GPLv3 §6 (sorgente insieme al binario, offerta scritta, o
  "offering access from a designated place" con accesso equivalente al sorgente) (`COPYING.GPLv3`).
- **[I]** Un ffmpeg.exe separato, eseguito come processo e comunicante via pipe/argomenti, è aggregazione: il codice di
  Sbobino non diventa LGPL. Gli obblighi restano sul **binario FFmpeg**: sorgente corrispondente (o link/offerta),
  configure usato, testo della licenza, attribuzione, e le note di copyright delle librerie incluse (es. il BSD di libopus
  richiede di riprodurre la nota "in the documentation and/or other materials provided with the distribution").
  Non sono un avvocato: va confermato da chi decide la policy legale.

### 3.3 Build pronte

**BtbN/FFmpeg-Builds** (release `latest`, pubblicata 2026-10-01; API GitHub `repos/BtbN/FFmpeg-Builds/releases/latest`)

- **[F]** Varianti (README): `gpl` (tutte le dipendenze), `lgpl` ("Lacking libraries that are GPL-only. Most prominently
  libx264 and libx265"), `nonfree` (+fdk-aac), e le versioni `-shared` con le DLL libav*. Addin per ramo di release fino a
  `9.0`. Build cross-compilate con MinGW (script `10-mingw.sh`), "minimum supported version is Windows 10 22H2", UCRT richiesto.
- **[F]** **libopus incluso in tutte le varianti**: `scripts.d/50-libopus.sh` ha `ffbuild_enabled() { return 0 }` e
  `--enable-libopus` (libopus da `github.com/xiph/opus`). x264 invece è escluso per `lgpl*` (`50-x264.sh`).
- **[F]** Dimensioni reali, lette dal central directory degli zip via HTTP Range, senza scaricarli (n9.0, 2026-10-01):

  | Asset | Zip | ffmpeg.exe | ffprobe.exe |
  |---|---|---|---|
  | `ffmpeg-n9.0-latest-win64-lgpl-9.0.zip` | 163.6 MiB | **128.3 MiB** (53.2 compresso) | 128.1 MiB |
  | `ffmpeg-n9.0-latest-win64-gpl-9.0.zip` | 185.1 MiB | 157.5 MiB | 157.3 MiB |
  | `ffmpeg-n9.0-latest-win64-lgpl-shared-9.0.zip` | 73.4 MiB | 0.5 MiB + DLL: avcodec-63 87.0, avfilter-12 29.8, avformat-63 21.7, avdevice-63 4.7, avutil-61 2.9, swscale-10 2.2, swresample-7 0.7 MiB | 0.2 MiB |

  Lo zip contiene un solo `LICENSE.txt` (quello di FFmpeg).
- **[I]** Le build BtbN includono anche decine di librerie video/rete/GPU (vulkan, dav1d, aom, openssl, libcurl…):
  spedire 128 MiB per usare solo i decoder audio è sproporzionato. L'installer NSIS (LZMA) dovrebbe comprimerlo intorno
  ai ~45-55 MiB, in linea con il 53 MiB del deflate dello zip (stima non misurata).

**gyan.dev**

- **[F]** "All builds are 64-bit, static and licensed as GPLv3" (gyan.dev/ffmpeg/builds, release 9.0.2 del 2026-09-19);
  essentials 7z ≈ 34 MB, zip ≈ 109 MB; libopus presente nella essentials. **→ Non adatte se si vuole restare in LGPL.**
- **[F]** Il crate `ffmpeg-sidecar` scarica su Windows proprio la gyan "release-essentials" (`src/download.rs`), quindi
  l'auto-download porta una build GPLv3.

**Build minimale custom (solo audio)**

- **[F]** Opzioni di `configure` disponibili: `--disable-everything`, `--disable-autodetect`, `--disable-network`,
  `--enable-small`, `--disable-ffplay`, `--disable-ffprobe`, `--enable-libopus`, `--toolchain=NAME` (MSVC supportato)
  (`configure` nel repo FFmpeg). Componenti presenti in FFmpeg per tutti i formati richiesti (`allformats.c`/
  `allcodecs.c`): demuxer `mov, matroska, avi, asf, flv, mpegts, mpegps, ogg, wav, aiff, mp3, aac, flac`; decoder `aac,
  ac3, eac3, mp2, mp3, wmav1, wmav2, wmapro, wmavoice, wmalossless, opus, vorbis, flac, alac, pcm_*, pcm_bluray, adpcm_*`,
  encoder `libopus`.
- **[I]** Selezione indicativa: `--disable-everything --disable-autodetect --disable-network --disable-ffplay
  --enable-protocol=file,pipe --enable-demuxer=<elenco sopra> --enable-decoder=<elenco sopra> --enable-parser=aac,ac3,
  mpegaudio,opus,vorbis,flac --enable-filter=aresample,aformat,anull,amix… --enable-muxer=ogg,opus,pcm_f32le
  (--enable-libopus --enable-encoder=libopus solo nella variante A)`, senza `--enable-gpl`/`--enable-version3` → LGPL 2.1.
  **Dimensione stimata: pochi MB (ordine 3-8 MB per ffmpeg.exe statico), non verificata**: richiede una build, che qui non
  è stata eseguita. Costo: una pipeline CI (MSYS2+MSVC o cross MinGW in Docker, come fa BtbN) da mantenere e aggiornare
  per le patch di sicurezza.

### 3.4 libopus

- **[F]** Licenza BSD a 3 clausole (`COPYING` di xiph/opus): le ridistribuzioni binarie devono riprodurre la nota di
  copyright e il disclaimer nella documentazione. Opus è coperto da licenze di brevetto royalty-free (stesso file). Ultimo
  tag `v1.6.1` (API GitHub tags; l'ultima "release" GitHub è v1.5.2 del 2024-09-11).
- **[F]** `opus` 0.4.0 (MIT/Apache-2.0) e `ogg` 0.9.2 (BSD-3-Clause) su crates.io.

---

## 4. Tabella comparativa

| Criterio | A. Symphonia (+adapter libopus) | B. Symphonia + crate extra (TS/FLV/AVI/ASF/AC-3/WMA) | C. `ffmpeg-next` (FFI) | D. Sidecar BtbN lgpl statico | E. Sidecar build custom minimale |
|---|---|---|---|---|---|
| Copertura 19 estensioni | 8 piene, 5 parziali, **6 assenti** [I] | teoricamente quasi tutte, con crate 0.0.x [I] | tutte [I] | tutte [I] | tutte (se si abilitano i componenti giusti) [I] |
| Licenza | MPL-2.0 + BSD (libopus) [F] | MPL/MIT/Apache misti [F] | LGPL (DLL obbligatorie) [F] | **LGPLv3** [F] | LGPL 2.1 (senza version3) [I] |
| Bundle aggiuntivo | ~1-3 MB nel binario Rust [I] | qualche MB [I] | DLL: ~120-150 MiB con BtbN shared, pochi MB se custom [F/I] | **128.3 MiB** ffmpeg.exe (+128 MiB se serve ffprobe) [F] | pochi MB [I, da misurare] |
| Complessità build MSVC | bassa (cargo; opusic-sys già presente) [I] | media: colla tra crate con API diverse [I] | alta: header+import lib, libclang, vcpkg o MSYS2 [F] | nulla: file scaricato + `externalBin` [F] | media-alta, una tantum + CI [I] |
| Avanzamento % | da timestamp dei pacchetti / `n_frames` [I] | idem, per crate [I] | da pts [I] | `-progress` + durata [F] | idem D [F] |
| Prestazioni | ±15% di FFmpeg [F] | ignote [I] | FFmpeg nativo, in-process [I] | FFmpeg nativo + costo pipe trascurabile [I] | idem D [I] |
| Robustezza / isolamento | panic gestibili; stato "Good" per MKV ("some streams may panic") [F] | rischio alto su crate immaturi [I] | crash nativo = crash dell'app [I] | crash isolato nel processo figlio [I] | idem D |
| Manutenzione | attiva (0.6.1 ad agosto 2026) [F] | frammentata [I] | binding "maintenance-only" [F] | aggiornare il binario; BtbN conserva i daily solo 14 giorni + uno al mese per 2 anni (README) [F] | ricostruire a ogni CVE [I] |
| Obblighi di distribuzione | note di licenza MPL/BSD [I] | idem [I] | checklist LGPL completa [F] | sorgente FFmpeg + deps, testo LGPLv3, attribuzione [F/I] | sorgente FFmpeg + configure, testo LGPL 2.1, attribuzione [F/I] |

---

## 5. Raccomandazione (proposta, non decisione)

**FFmpeg come sidecar, avviato con `std::process::Command` dal backend, nella variante "ffmpeg solo decodifica"
(variante B di §2.3).** Partire con il binario **BtbN `lgpl` statico** e passare a una **build custom minimale** (E)
quando la dimensione del bundle diventa un problema.

Motivi:
1. **Copertura:** con le sole librerie Rust restano scoperti 6 formati su 19 (AVI, WMV, FLV, TS, MTS, MPEG) più AC-3 in
   MP4/MOV/MKV, cioè proprio il "video da videocamera/registratore" che un'app di sbobinatura riceve. I crate che li
   coprirebbero sono 0.0.x (§1.2).
2. **Un solo percorso di codice:** ffmpeg emette sempre `f32le` alla frequenza e con i canali richiesti. La trascrizione
   lo consuma a 16 kHz mono; "Estrai audio" lo passa all'encoder Ogg/Opus Rust già esistente con canali/frequenza scelti
   dall'utente (8/16/24/48 kHz sono comunque valide anche per libopus in FFmpeg, §2.3). Non serve un resampler Rust né un
   secondo encoder. L'avanzamento si calcola allo stesso modo nei due casi (`out_time_us`, o campioni letti, diviso durata).
3. **Licenza gestibile:** processo separato = aggregazione; nessun codice GPL serve (niente x264/x265). Obblighi:
   pubblicare sorgente e configure del binario FFmpeg spedito (o un link a quello esatto), il testo della licenza e
   l'attribuzione in "Informazioni" e nel sito di download.
4. **Rischio di build minimo:** nessun libclang, header o vcpkg nella toolchain MSVC dell'app (contrario di C).

Cose da evitare: gyan.dev e l'auto-download di `ffmpeg-sidecar` (GPLv3); la variante `-shared` di BtbN (più pesante della
statica, ~150 MiB in tutto); spedire ffprobe dalla build BtbN (+128 MiB) solo per la durata.

Alternativa ragionevole se il peso del bundle è il vincolo principale e i formati video "vecchi" sono accettabili come non
supportati: Symphonia 0.6.1 con `all-formats`/`all-codecs` + `symphonia-adapter-libopus` (stessa `opusic-sys` già in
stack) + `rubato`, e un errore chiaro "formato non supportato" per i 6 formati scoperti.

---

## Incertezze e punti aperti

1. **Dimensione della build custom minimale**: la stima di pochi MB è un'inferenza; serve una build di prova per misurarla
   (con e senza libopus).
2. **`ffmpeg-next` 9.0.0 e FFmpeg 9.x**: il README dice "3.4 til 8.0"; la compatibilità con 9.x non è verificata.
3. **Durata senza ffprobe**: parsare `Duration:` dallo stderr è fragile; non c'è una fonte che ne garantisca la stabilità.
   Alternativa: un ffprobe incluso nella build custom (piccolo).
4. **Compressione NSIS** di un ffmpeg.exe da 128 MiB: non misurata.
5. **Codec tipici per estensione**: per MTS/AVCHD, AVI, MPEG-PS la tabella si basa su conoscenza comune (avchd-info.org non
   raggiungibile); FLV, TS e ASF sono confermati da sorgente FFmpeg o documentazione Microsoft.
6. **Interpretazione legale** (aggregazione, LGPLv3 vs LGPL 2.1, note delle librerie terze nelle build BtbN): da validare
   con chi gestisce la compliance. Lo zip BtbN contiene solo il `LICENSE.txt` di FFmpeg, non le note delle dipendenze.
7. **Media Foundation** (nativo Windows, nessun binario da spedire): la pagina Microsoft *Supported Media Formats in Media
   Foundation* (aggiornata 2018) elenca ASF/WMA, MP4, AVI, MP3, ADTS, WAV ma non MKV, TS, FLV, FLAC, Opus o AC-3, e nelle
   edizioni N il supporto dipende dal Media Feature Pack ([I]). Non approfondita: copertura insufficiente rispetto ai requisiti.
8. **Symphonia "Good" su MKV** ("Some streams may panic"): per file reali andrebbe testato su un corpus.
