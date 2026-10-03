# 08: Registrazione dal microfono

**What to build:** l'utente registra dal microfono, quello predefinito o uno scelto tra quelli rilevati. Durante la Registrazione vede il timer (senza le pause) e l'indicatore di livello, e può fare Pausa e Riprendi nella stessa sessione. Stop salva un OGG/Opus nella Cartella predefinita, con data e ora nel nome, e il file diventa la Sorgente. Se il microfono si scollega, la Registrazione si salva come con Stop.

**Blocked by:** 04 (Controllo della Trascrizione), 06 (Impostazioni persistenti, scelta del modello e Lingua del parlato)

**Status:** done

- [x] Cattura con cpal 0.18.2:
  - la callback non alloca e passa i buffer con il timestamp `capture` a un worker;
  - il worker fa il downmix e ricampiona alla frequenza delle impostazioni.
- [x] Writer Ogg/Opus (`opus` + `ogg`):
  - frame da 20 ms;
  - `OpusHead` con il pre-skip convertito a 48 kHz, e `OpusTags`;
  - granule a 48 kHz;
  - pagina chiusa circa ogni secondo, `EndStream` a Stop;
  - bitrate, canali e frequenza dalle impostazioni.
- [x] Pausa con `AtomicBool`: il tempo di pausa è escluso dai timestamp, quindi timer e file non hanno vuoti. Il timer deriva dai timestamp. Evento `recording-tick` con durata e livello
- [x] Nome `<prefisso tradotto> AAAA-MM-GG HH-MM-SS.ogg`, con " 2", " 3"… se esiste già. Funzione pura testata
- [x] Cartella predefinita da impostazioni, default `Documenti\Sbobino`, creata se manca
- [x] Impostazioni audio: i nove bitrate, mono/stereo, 8/16/24/48 kHz, default 32 kbps mono 48 kHz. Scelta del dispositivo microfono
- [x] Dopo Stop il file diventa la Sorgente. La Registrazione rispetta la regola di una Attività alla volta
- [x] Un dispositivo scollegato ferma e salva la Registrazione, con un errore dedicato che riporta il nome del dispositivo
- [x] Test: buffer sintetici passano per il writer, e il file rilegge con Symphonia con la durata attesa per ogni frequenza e canale; pause escluse; nomi dei file
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi, verifica manuale in `bun tauri dev`

## Note di chiusura

- Verifica in `bun tauri dev` sul microfono reale (KLIM Talk, Anker PowerConf C200): timer, livello, Pausa/Riprendi, Stop, file nella Cartella predefinita (anche una cartella scelta e non esistente), file come Sorgente, Activity rifiutata durante la Registrazione, `microphoneMissing`. Le durate (ffprobe) coincidono con il timer e le pause sono escluse. Trascrizione con Nemotron avviata sul file: "Nessun parlato rilevato", perché la sintesi vocale suonata dalle casse non arrivava ai microfoni (livello −60 dBFS); nessuno ha parlato al microfono durante la prova.
- Il dispositivo scollegato non è stato provato staccando l'hardware: il percorso (errore della callback cpal → salvataggio come Stop → `deviceDisconnected` con il nome) è verificato solo sul codice di cpal 0.18.2.
- Aggiunto il codice d'errore `microphoneMissing` (nessun microfono, o quello scelto non collegato), non elencato nella spec.
