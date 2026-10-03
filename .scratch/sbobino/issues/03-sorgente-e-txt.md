# 03: Sorgente e TXT

**What to build:** l'utente vede il nome della Sorgente, lo clicca per aprirla con il programma associato e ne legge il percorso nella status bar. Può trascrivere anche video MP4, MOV, M4V e MKV. Durante la Trascrizione vede la percentuale, oppure un avanzamento senza percentuale se la durata non è nota. Alla fine trova il testo salvato in `<nome Sorgente> trascrizione <N>.txt` accanto alla Sorgente, e lo copia con "Copia testo".

**Blocked by:** 02 (Tracer bullet)

**Status:** done

- [x] Il nome della Sorgente è cliccabile (`tauri-plugin-opener`) e il percorso compare nella status bar
- [x] I video MP4, MOV, M4V e MKV con audio supportato si trascrivono senza file intermedi
- [x] Ci sono errori dedicati per codec non supportato (es. AC-3, MPEG-PS) e per file mancante o illeggibile
- [x] Evento `transcription-progress`: percentuale da frame decodificati / `n_frames`, altrimenti indeterminato
- [x] TXT accanto alla Sorgente con il primo N libero da 1. La regola è una funzione pura testata
- [x] La status bar finale mostra che è finito, il numero di caratteri e il percorso del TXT
- [x] "Copia testo" copia l'area negli appunti
- [x] Le sezioni visibili seguono l'Attività. La status bar mostra la fase e gli errori con messaggi tradotti per codice
- [x] Test: fixture MP4/AAC committata, errore codec, progresso determinato e indeterminato, nomi dei TXT, logica della status bar (`bun test`)
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi

## Note

- Implementato in 9eb0d17 e verificato in `bun tauri dev` (MP4 7→100 %, TXT 1/2/3, Copia testo, errori AC-3 e file mancante, apertura con VLC).
- Code review applicata: `transcribe` non resta "in corso" se l'IPC lancia, TXT scritto con `create_new` (niente sovrascritture in caso di gara), errori di apertura e degli appunti gestiti, percorso completo della Sorgente anche nel tooltip del nome.
- Un `.mpeg` MPEG-PS vero non è stato provato: con `mp1`/`mp2` attivi Symphonia potrebbe agganciarsi ai frame audio dei PES invece di dare errore. Una Trascrizione senza Frasi salva comunque un TXT vuoto.
- `transcription-finished` è il valore di ritorno di `transcribe`, non un evento; gli errori tornano nel `Result`, non in `activity-failed`.
- MOV, M4V e MKV non sono provati con file veri: MOV/M4V passano dallo stesso demuxer dell'MP4, i MKV non dichiarano la durata (avanzamento indeterminato).
