# 08: Registrazione dal microfono

**What to build:** l'utente registra dal microfono, quello predefinito o uno scelto tra quelli rilevati. Durante la Registrazione vede il timer (senza le pause) e l'indicatore di livello, e può fare Pausa e Riprendi nella stessa sessione. Stop salva un OGG/Opus nella Cartella predefinita, con data e ora nel nome, e il file diventa la Sorgente. Se il microfono si scollega, la Registrazione si salva come con Stop.

**Blocked by:** 04 (Controllo della Trascrizione), 06 (Impostazioni persistenti, scelta del modello e Lingua del parlato)

**Status:** ready-for-agent

- [ ] Cattura con cpal 0.18.2:
  - la callback non alloca e passa i buffer con il timestamp `capture` a un worker;
  - il worker fa il downmix e ricampiona alla frequenza delle impostazioni.
- [ ] Writer Ogg/Opus (`opus` + `ogg`):
  - frame da 20 ms;
  - `OpusHead` con il pre-skip convertito a 48 kHz, e `OpusTags`;
  - granule a 48 kHz;
  - pagina chiusa circa ogni secondo, `EndStream` a Stop;
  - bitrate, canali e frequenza dalle impostazioni.
- [ ] Pausa con `AtomicBool`: il tempo di pausa è escluso dai timestamp, quindi timer e file non hanno vuoti. Il timer deriva dai timestamp. Evento `recording-tick` con durata e livello
- [ ] Nome `<prefisso tradotto> AAAA-MM-GG HH-MM-SS.ogg`, con " 2", " 3"… se esiste già. Funzione pura testata
- [ ] Cartella predefinita da impostazioni, default `Documenti\Sbobino`, creata se manca
- [ ] Impostazioni audio: i nove bitrate, mono/stereo, 8/16/24/48 kHz, default 32 kbps mono 48 kHz. Scelta del dispositivo microfono
- [ ] Dopo Stop il file diventa la Sorgente. La Registrazione rispetta la regola di una Attività alla volta
- [ ] Un dispositivo scollegato ferma e salva la Registrazione, con un errore dedicato che riporta il nome del dispositivo
- [ ] Test: buffer sintetici passano per il writer, e il file rilegge con Symphonia con la durata attesa per ogni frequenza e canale; pause escluse; nomi dei file
- [ ] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi, verifica manuale in `bun tauri dev`
