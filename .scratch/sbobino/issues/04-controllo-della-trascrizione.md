# 04: Controllo della Trascrizione

**What to build:** l'utente può annullare una Trascrizione lunga. Se nell'area c'è già del testo, una nuova Trascrizione chiede conferma prima di sostituirlo. Mentre un'Attività è in corso, le azioni che ne avvierebbero un'altra sono disabilitate.

**Blocked by:** 03 (Sorgente e TXT)

**Status:** ready-for-agent

- [ ] Durante la Trascrizione c'è un pulsante "Annulla". Dopo Annulla il testo già comparso resta, nessun TXT viene salvato e la status bar lo dice
- [ ] L'annullamento usa il `CancelToken` di `transcribe-cpp` (se il modello supporta la cancellazione) e il controllo tra una Frase e l'altra
- [ ] Se l'area contiene testo, Trascrivi chiede conferma prima di sostituirlo
- [ ] Il backend ha una sola Attività alla volta: avviarne una seconda restituisce l'errore "Attività in corso". La UI disabilita Sfoglia e Trascrivi (e in seguito Registra) durante un'Attività
- [ ] Una Trascrizione che non trova Frasi non salva il TXT e la status bar mostra "Nessun parlato rilevato"
- [ ] Test: Annulla ferma la pipeline senza emettere la fine, e il guard rifiuta una seconda Attività
- [ ] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
