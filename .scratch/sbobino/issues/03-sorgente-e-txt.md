# 03: Sorgente e TXT

**What to build:** l'utente vede il nome della Sorgente, lo clicca per aprirla con il programma associato e ne legge il percorso nella status bar. Può trascrivere anche video MP4, MOV, M4V e MKV. Durante la Trascrizione vede la percentuale, oppure un avanzamento senza percentuale se la durata non è nota. Alla fine trova il testo salvato in `<nome Sorgente> trascrizione <N>.txt` accanto alla Sorgente, e lo copia con "Copia testo".

**Blocked by:** 02 (Tracer bullet)

**Status:** ready-for-agent

- [ ] Il nome della Sorgente è cliccabile (`tauri-plugin-opener`) e il percorso compare nella status bar
- [ ] I video MP4, MOV, M4V e MKV con audio supportato si trascrivono senza file intermedi
- [ ] Ci sono errori dedicati per codec non supportato (es. AC-3, MPEG-PS) e per file mancante o illeggibile
- [ ] Evento `transcription-progress`: percentuale da frame decodificati / `n_frames`, altrimenti indeterminato
- [ ] TXT accanto alla Sorgente con il primo N libero da 1. La regola è una funzione pura testata
- [ ] La status bar finale mostra che è finito, il numero di caratteri e il percorso del TXT
- [ ] "Copia testo" copia l'area negli appunti
- [ ] Le sezioni visibili seguono l'Attività. La status bar mostra la fase e gli errori con messaggi tradotti per codice
- [ ] Test: fixture MP4/AAC committata, errore codec, progresso determinato e indeterminato, nomi dei TXT, logica della status bar (`bun test`)
- [ ] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
