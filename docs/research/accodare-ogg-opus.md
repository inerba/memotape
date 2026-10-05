# Ricerca: accodare audio nuovo al `mix.ogg` di un Bino

Domanda: per "continuare una Registrazione" l'audio nuovo si registra in un Ogg temporaneo con `OggOpusWriter` e va attaccato in coda al `mix.ogg` del Bino (e a `microfono.ogg`/`sistema.ogg`, se ci sono). Si può fare senza ricodificare l'audio vecchio? Tre strade:

- **A**, catena Ogg: un secondo flusso logico (serial diverso, `OpusHead`/`OpusTags` suoi) dopo il primo, cioè i due file concatenati byte per byte.
- **B**, stesso flusso logico continuato: le pagine nuove copiate con serial, sequence number e granule riscritti, senza `OpusHead`/`OpusTags` del file nuovo, togliendo EOS dall'ultima pagina vecchia e ricalcolando il CRC. Sotto si distinguono due varianti:
  - **B1**: l'ultima pagina vecchia tiene la granule tagliata (end trimming) e le granule nuove partono da lì;
  - **B2**: l'ultima pagina vecchia prende come granule il totale dei campioni decodificati (`pacchetti × 960`, niente end trimming) e le granule nuove partono da lì.
- **C**, ricodifica completa, solo come riferimento di costo.

Fonti consultate e prove fatte il 2026-10-05. Legenda: **[V]** = verificato su fonte primaria (RFC, sorgente nel registry cargo alla versione di `Cargo.lock`, sorgente di Chromium); **[E]** = provato empiricamente (vedi §6); **[I]** = inferenza mia, da confermare. Qui ci sono solo fatti: la scelta spetta all'utente.

Versioni (`src-tauri/Cargo.lock`): `symphonia` / `symphonia-core` / `symphonia-format-ogg` 0.6.1, `symphonia-adapter-libopus` 0.3.0, `opus` 0.4.0 e `opusic-sys` 0.7.5 (libopus 1.6.1, `opus/package_version`), `ogg` 0.9.2, `zip` 8.6.0.

---

## 1. Come scriviamo e leggiamo il mix oggi

### 1.1 `audio_toolkit::ogg_opus::OggOpusWriter`

- **[V]** Serial **casuale** per ogni writer: `random_serial()` è l'hash di `SystemTime::now()` con `RandomState` (`ogg_opus.rs:239-242`). Due Ogg diversi hanno quasi sempre serial diversi.
- **[V]** `OpusHead` ha una pagina tutta sua, con granule 0, e lo stesso vale per `OpusTags` (`ogg_opus.rs:76-89`). Le pagine audio partono quindi dalla terza.
- **[V]** Pre-skip = `lookahead × 48000 / Fs`, con il lookahead letto da `encoder.get_lookahead()` (`ogg_opus.rs:58-61`). **[I]** In libopus il lookahead (`Application::Audio`) è `Fs/400 + Fs/250`, quindi il pre-skip è 312 a tutte le frequenze delle impostazioni (8, 16, 24 e 48 kHz). ffmpeg con libopus a 48 kHz scrive 312 (§6).
- **[V]** Pacchetti da 20 ms, una pagina ogni 50 pacchetti (`EndPage`). Granule di una pagina intermedia = `pacchetti × 960` (`ogg_opus.rs:149-154`), quindi coerente con RFC 7845 §4 anche sulla prima pagina audio (48 000 = i campioni dei suoi pacchetti).
- **[V]** `finish` completa l'ultimo frame con zeri e continua a codificare finché `(pacchetti + 1) × 960 ≥ pre_skip + campioni`. L'ultimo pacchetto esce con `EndStream` (flag EOS) e granule `pre_skip + ceil(frames_in × 48000 / rate)` (`ogg_opus.rs:116-136`): è l'end trimming. La differenza tra campioni decodificati e granule finale è sempre minore di 960, cioè meno di 20 ms.
- **[V]** `ogg::PacketWriter` 0.9.2 tiene per ogni serial `sequence_num` da 0 e `first_page: true`: la prima pagina scritta per un serial prende sempre il flag BOS (`writing.rs:132-142`, `208-209`). Non c'è modo di far partire un writer "in mezzo" a un flusso esistente. Il CRC di `ogg` (`mod crc`) è privato. `symphonia_core::checksum::Crc32` invece è pubblico (`symphonia-core-0.6.1/src/checksum/crc32.rs:549`) ed è quello che usa il demuxer Ogg (`page.rs:203-212`).

### 1.2 Chi legge il mix

- **[V]** `bino::Mix::open` (`bino.rs:325-345`) apre la voce `mix.ogg`, che deve essere `Stored`, come una finestra di byte sul file. Non sa nulla di Ogg.
- **[V]** `player::respond` (`player.rs:113-146`) serve quei byte con `Content-Type: audio/ogg` e `Range`. Nemmeno lui guarda dentro l'Ogg: tutto dipende dal demuxer di Chromium.
- **[V]** `audio_toolkit::decode::Decoder` (Symphonia, `decode.rs:34-129`) sceglie `default_track` una volta sola, e `next_block` tratta `ResetRequired` come fine del file: "Le catene Ogg con flussi diversi non sono previste: finiscono qui" (`decode.rs:94-95`). Lo usano `transcribe_file` (Trascrivi su un Bino) e `decode::peaks`.
- **[V]** Forma d'onda: `player::forma_onda` usa `forma-onda.json` se c'è, altrimenti `decode::peaks` (`player.rs:98-110`). Quindi, con qualunque strada, se la Forma d'onda manca o viene ricalcolata passa da Symphonia. **[I]** I 1000 valori salvati non si possono accodare esattamente (sono già raggruppati): o si fondono pesandoli con le durate, che è un'approssimazione, o si ricalcolano.

---

## 2. Cosa dicono RFC 7845 e RFC 3533

- **[V]** Catena: "In chaining, complete logical bitstreams are concatenated … the eos page of a given logical bitstream is immediately followed by the bos page of the next. Each chained logical bitstream MUST have a unique serial number within the scope of the physical bitstream" ([RFC 3533 §4](https://www.rfc-editor.org/rfc/rfc3533#section-4)). A è quindi un file Ogg valido, purché i serial siano diversi. Con il nostro serial casuale è quasi sempre vero, ma non è garantito.
- **[V]** Granule delle pagine intermedie: "All other pages with completed packets after the first MUST have a granule position equal to the number of samples contained in packets that complete on that page plus the granule position of the most recent page with completed packets … there cannot be any gaps" ([RFC 7845 §4](https://www.rfc-editor.org/rfc/rfc7845#section-4)). **B1 viola questo MUST**: l'ultima pagina vecchia, senza più EOS, ha una granule minore dei suoi campioni. B2 lo rispetta.
- **[V]** End trimming: "The page with the 'end of stream' flag set MAY have a granule position that indicates the page contains less audio data than would normally be returned" ([§4.4](https://www.rfc-editor.org/rfc/rfc7845#section-4.4)). Il permesso vale solo per la pagina con EOS.
- **[V]** Pre-skip: "exactly 'pre-skip' samples SHOULD be skipped from the beginning of the decoded output" ([§4.5](https://www.rfc-editor.org/rfc/rfc7845#section-4.5)). Il pre-skip vale una volta per flusso logico. In B il pre-skip del file nuovo sparisce con il suo `OpusHead`, quindi i suoi primi 312 campioni (il ritardo del nuovo encoder) restano nell'audio.
- **[V]** "Continuous Chaining" ([§7.2](https://www.rfc-editor.org/rfc/rfc7845#section-7.2)): due segmenti codificati indipendentemente "creates a small discontinuity at the boundary due to the lossy nature of Opus". Per evitarla la RFC propone un ultimo frame senza predizione, `OPUS_SET_PREDICTION_DISABLED`, la sua copia in testa al secondo segmento e un pre-skip calcolato apposta. Nel nostro caso il confine è comunque un vero stacco tra due sessioni, e la discontinuità misurata è trascurabile (§6.3).

---

## 3. Symphonia 0.6.1 (`symphonia-format-ogg` + `symphonia-adapter-libopus` 0.3.0)

### 3.1 A, catena

- **[V]** `OggReader::read_page` (`demuxer.rs:90-96`): una pagina con BOS fa partire `start_new_physical_stream()`, che sostituisce `tracks` e `streams` con quelli del flusso nuovo (con il serial nuovo come `track.id`), e restituisce `Error::ResetRequired`. Symphonia quindi la catena la legge, ma chi chiama deve rileggere le tracce e ricreare il decoder.
- **[V]** Il nostro `Decoder::next_block` si ferma a `ResetRequired` (`decode.rs:95`). **Con A, oggi Trascrivi su un Bino e `decode::peaks` vedono solo l'audio vecchio.** Per leggere tutto servirebbe gestire il reset: rileggere `format.tracks()`, chiamare di nuovo `make_audio_decoder` e cambiare `track_id`.
- **[V]** `num_frames` vale per il solo flusso fisico corrente. `probe_stream_end` non trova pagine del flusso alla fine del file, passa alla bisezione ("media source stream is chained, bisecting end of physical stream", `physical.rs:120-158`) e `inspect_end_page` imposta `num_frames` dall'ultima pagina con EOS di quel flusso (`logical.rs:494-507`). Il progresso di `Decoder::progress` (frame decodificati / `num_frames`) supererebbe il 100%, che viene limitato, già durante il primo flusso.
- **[V]** Anche il seek di Symphonia lavora solo dentro il flusso fisico corrente (`phys_byte_range_start/end`, `demuxer.rs:182-186`). Oggi non lo usiamo.
- **[V]** L'adapter libopus applica il pre-skip dell'`extra_data` (l'`OpusHead`) al primo pacchetto di ogni decoder nuovo (`lib.rs:102-106`, `156-162`). Con un decoder ricreato a ogni reset, il pre-skip del secondo flusso sarebbe applicato.

### 3.2 B, stesso flusso

- **[V]** Il CRC si verifica (`page.rs:203-236`): una pagina con CRC sbagliato si scarta con un `warn`, e il CRC va quindi ricalcolato. Sequence number non contigui danno solo un `warn` e svuotano un eventuale pacchetto a metà (`logical.rs:115-127`). Le nostre pagine finiscono sempre a fine pacchetto, ma conviene comunque rinumerare.
- **[V]** Su **qualsiasi** pagina, non solo sull'ultima, `read_page_init` taglia la coda dell'ultimo pacchetto se i pacchetti superano la granule della pagina: `trim_end = next_pkt_pts − page_end_ts` (`logical.rs:294-301`). La pagina dopo parte dalla granule della precedente (`logical.rs:219-224`), e l'adapter rispetta `trim_end` (`lib.rs:156-160`).
  - Con B1 Symphonia toglie quindi il riempimento in coda all'audio vecchio, in mezzo al flusso: va oltre quanto chiede la RFC, mentre ffmpeg e Chromium non lo fanno (§6.2). Il risultato è un disallineamento sotto i 20 ms tra la linea del tempo di Symphonia (Trascrizione, Forma d'onda) e quella del player.
  - Con B2 non c'è niente da tagliare fino all'ultima pagina, che ha EOS: Symphonia decodifica gli stessi campioni di ffmpeg e Chromium.
- **[V]** Il pre-skip si toglie una volta sola, al primo pacchetto (`self.pre_skip = 0`, `lib.rs:161-162`). Il ritardo del secondo encoder (312 campioni, 6,5 ms) resta nell'audio decodificato, come in ffmpeg e Chromium.
- **[V]** `num_frames` = granule dell'ultima pagina con EOS (`logical.rs:494-507`, `absgp_to_ts` è l'identità: `mappings/mod.rs:63-65`): è la durata totale, pre-skip compreso, come oggi.
- **[I]** Non provato con Symphonia: in questa sessione non si potevano lanciare build cargo. Le affermazioni di §3 vengono dalla lettura del sorgente. Un test Rust che rilegge un mix B2 con `Decoder` e ne confronta i campioni con i due Ogg di partenza chiuderebbe la questione.

---

## 4. Chromium / WebView2 (`<audio>`)

- **[V]** Il demuxer di Chromium è FFmpeg. `media/filters/ffmpeg_demuxer.cc` (ramo main, letto il 2026-10-05) ha delle "chained ogg fixups" per i file Ogg con una sola traccia audio Opus o Vorbis: "FFmpeg doesn't support chained ogg correctly. Instead of guaranteeing continuity across links in the chain it uses the timestamp information from each link directly … Fixing chained ogg is non-trivial, so for now just reuse the last good timestamp" (crbug.com/396864). È solo una correzione dei timestamp, non un supporto completo.
- **[E]** In pratica, con Chromium 152.0.7977.130 (il browser integrato di Claude) e con Edge 154.0.4258 headless (stesso motore e stessa versione del runtime WebView2 installato, 154.0.4258.53), una catena A in `<audio>`:
  - ha come `duration` la durata del **solo ultimo** flusso (2,02 s su un file da 3,01 + 2,01 s);
  - viene limitata a quella durata nel seek: `currentTime = 4` dà 2,02;
  - **suona solo il primo flusso**: `ended` arriva dopo circa 3,06 s in entrambi i motori e in tutte le ripetizioni.

  `decodeAudioData` (Web Audio) invece decodifica tutti e due i flussi, ma senza togliere il pre-skip del secondo (+312 campioni rispetto a ffmpeg).
- **[E]** Con B1 e B2, `duration` è quella totale (granule finale / 48 000, pre-skip compreso, come per i file di oggi), il seek a 4 s atterra a 4 s, la riproduzione attraversa la giunzione e `ended` arriva alla fine. Due esecuzioni headless si sono bloccate a metà (una con B1, una con B2, in punti diversi e prima o dopo la giunzione). Rilanciate, sono arrivate alla fine: è un difetto della sessione headless (anche i timer erano in ritardo), non del file.
- **[I]** WebView2 non è stato provato direttamente: lanciato da solo, `msedgewebview2.exe` in headless non è partito. Edge e WebView2 condividono motore e versione, ma la prova definitiva è nell'app (`bun tauri dev`, via CDP sulla 9222, vedi AGENTS.md "Pilotare l'app").

---

## 5. Costo di C (ricodifica)

- **[E]** Un'ora di parlato mono 48 kHz a 32 kbps (12,3 MB, le impostazioni predefinite) con ffmpeg 8.0 e libopus, su un Ryzen 7 3700X con un thread: la sola decodifica dura 7,6 s, decodifica più codifica 50,6 s. AGENTS.md riporta per `decode::peaks` 8 s per 25 minuti in release.
- **[E]** Perdita di generazione: sulla fixture a 32 kbps l'SNR sulla forma d'onda contro l'originale passa da 10,8 dB (prima codifica) a 6,7 dB (seconda). Non è una misura percettiva, perché Opus non conserva la fase, ma la seconda codifica aggiunge un errore paragonabile alla prima, e a ogni "continua" se ne aggiunge un altro.
- **[V]** Tra le tre strade, C è l'unica che non dipende da come i lettori trattano la giunzione.

---

## 6. Prove empiriche

Strumenti: ffmpeg 8.0 (build gyan.dev, con libopus), Python 3.14 + numpy 2.4, Chromium 152 del browser integrato, Edge 154 headless pilotato via CDP con Node 25. Non c'erano né `opusenc` né `opusinfo`. File e script stavano nella scratchpad della sessione, che non viene conservata.

### 6.1 Procedura

1. Due coppie di file, codificate come `OggOpusWriter` (libopus, `-application audio -frame_duration 20 -b:a 32k`, 48 kHz mono):
   - toni: 440 Hz per 3,0107 s e 880 Hz per 2,0133 s;
   - `parlato-it.wav` tagliato a 3,0071 s, prima parte e resto.

   Le durate sono scelte per avere end trimming in tutti e due i file.
2. Uno script Python, che ricostruisce i file letti byte per byte identici e quindi ha il CRC giusto, produce:
   - **A**: la concatenazione dei due file;
   - **B1**: l'ultima pagina vecchia senza EOS ma con la granule tagliata, e granule nuove + granule finale vecchia;
   - **B2**: l'ultima pagina vecchia con granule = campioni decodificati (`151 × 960 = 144 960`), e granule nuove + 144 960.

   In B1 e B2 il serial è quello vecchio, i sequence number sono continui e le prime due pagine del file nuovo (`OpusHead`, `OpusTags`) sono saltate.
3. Decodifica con ffmpeg (decoder nativo e libopus), poi con `<audio>`/`decodeAudioData` in Chromium e in Edge, serviti da un server HTTP con `Range` (206), come `player::respond`.

### 6.2 Risultati (parlato; i toni danno lo stesso quadro)

| | campioni decodificati (ffmpeg) | `duration` ffprobe | `<audio>` Chromium/Edge |
|---|---|---|---|
| a da solo | 144 341 | 3,0136 s | ok |
| b da solo | 285 739 | 5,9594 s | ok |
| **A** | 430 080 = a + b, entrambi i pre-skip tolti | **5,9594 s (solo b)** | `duration` del solo b, **suona solo a** |
| **B1** | 430 699 | 8,9730 s | ok; il seek di ffmpeg sbaglia di 6,4 ms |
| **B2** | 430 699 | 8,9794 s | ok; il seek di ffmpeg è esatto al campione |

- **[E]** ffmpeg da riga di comando decodifica tutta la catena A, con pre-skip ed end trimming per ogni flusso, ma ffprobe riporta come durata quella dell'ultimo flusso.
- **[E]** In B1 e B2 ffmpeg (come Chromium) decodifica il riempimento vecchio (307 campioni) e il pre-skip nuovo (312): **619 campioni (12,9 ms) di quasi silenzio alla giunzione** (picco 0,023 contro 0,10 del parlato intorno). Con i nostri file sono sempre meno di 20 ms + 6,5 ms.
- **[E]** In B2 il primo campione dell'audio nuovo cade esattamente a `pacchetti vecchi × 960`, cioè 151 × 20 ms = 3,020 s sulla linea del tempo. **[I]** Accade perché i due pre-skip sono uguali (312, §1.1): vecchio pre-skip tolto più nuovo pre-skip tenuto si compensano. Lo scarto tra la fine vera dell'audio vecchio e l'inizio di quello nuovo è il riempimento (< 20 ms) più 312 campioni.
- **[E]** B1 non è conforme e lo si vede: ffmpeg ricava la durata dalla granule finale, che per i toni dà 241 776 invece di 241 910 (241 598 decodificati + 312 di pre-skip, i 134 campioni di riempimento vecchio mancano) e il seek atterra 307 campioni (6,4 ms) dopo il punto giusto. In B2 durata e seek tornano esatti (seek a 6,0 s su parlato: scarto 0).

### 6.3 Giunzione in B2: stato del decoder

- **[E]** Confronto tra l'audio nuovo decodificato dentro B2 (decoder che arriva dallo stato del flusso vecchio) e lo stesso file decodificato da solo:
  - nei primi 10 ms dopo la giunzione la differenza massima è 0,0002 (rms 0,00005, contro un rms del segnale di 0,017);
  - nei primi 200 ms resta sotto 0,001;
  - dopo, sotto 0,0007 (circa −63 dBFS).

  L'SNR contro l'originale è lo stesso (11,2 dB nei primi 100 ms, 10,7 dB su tutto). **Niente clic**: lo stato del decoder ereditato non produce artefatti udibili, anche perché il flusso vecchio finisce con un frame completato con zeri e il nuovo encoder parte da uno stato vuoto.
- **[E]** Canali e frequenza diversi: un B2 con il vecchio mono a 48 kHz e il nuovo stereo, codificato a 16 kHz a 64 kbps, si decodifica senza errori in ffmpeg (430 698 campioni, mono, come dice l'`OpusHead` vecchio). Il numero di canali e l'input rate del flusso sono quelli del primo `OpusHead`. **[I]** Per non perdere un canale conviene registrare la continuazione con i canali del mix vecchio.

---

## 7. Riepilogo per strada

| | Symphonia (Trascrizione, Forma d'onda) | `<audio>` (player) | Conformità | Costo |
|---|---|---|---|---|
| **A** catena | **[V]** si ferma al primo flusso (`ResetRequired` = fine in `decode.rs`); servirebbe gestire il reset; `num_frames` solo del primo | **[E]** suona solo il primo flusso, `duration` dell'ultimo | valida (RFC 3533), serve un serial diverso | concatenazione di byte |
| **B1** granule tagliata a metà | **[V]** taglia il riempimento a metà flusso: linea del tempo diversa dal player di < 20 ms | **[E]** suona tutto; ffmpeg sbaglia seek e durata di qualche ms | **viola RFC 7845 §4** (MUST) | copia + riscrittura delle intestazioni di pagina |
| **B2** granule = campioni | **[V]** dal sorgente legge tutto come un solo flusso; **[I]** non provato | **[E]** suona tutto, durata e seek esatti | conforme | copia + riscrittura delle intestazioni di pagina; < 20 ms + 6,5 ms di quasi silenzio alla giunzione |
| **C** ricodifica | come oggi | come oggi | conforme | **[E]** circa 50 s per ora di audio (3700X), perdita di generazione a ogni continuazione |

Note pratiche su B2:

- **[V]** `PacketWriter` non sa continuare un flusso (BOS e sequenza da 0), quindi le pagine del file temporaneo vanno riscritte a valle: si salta `OpusHead`/`OpusTags`, si cambiano serial, sequence e granule e si ricalcola il CRC, per esempio con `symphonia_core::checksum::Crc32`. Il flag EOS e la granule dell'ultima pagina vecchia vanno corretti con lo stesso CRC.
- **[V]** Il Bino va riscritto: `mix.ogg` è una voce `Stored` del zip e cresce, quindi non si scrive sul posto. È una copia di byte (circa 12 MB per un'ora a 32 kbps) più le altre voci con `raw_copy_file`, come `bino::rewrite`.
- **[I]** I tempi delle Frasi nuove si spostano di `pacchetti vecchi × 20 ms` (§6.2); `durata_ms` di conseguenza.
