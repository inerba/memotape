# 03: Riverifica della Sensibilità e ADR

**What to build:** La Sensibilità che si comporta come prima con Silero v6.2: stessi esiti sul corpus del ticket 01, o soglie ritarate sullo stesso corpus con lo stesso criterio. L'interfaccia non cambia. Spec: `../spec.md`, sezione "Sensibilità".

**Blocked by:** 01, 02

**Status:** done

- [x] Gli stessi test ignorati del ticket 01 rigirano sul codice del ticket 02; confronto campione per campione e livello per livello con `baseline-v4.md`.
- [x] Se gli esiti coincidono (Frasi tenute e scartate, risposte brevi e voce debole conservate), le soglie restano. Altrimenti si ritarano le soglie della protezione, del segmentatore e del filtro sugli zeri finché Bilanciato riduce le false Frasi senza perdere le regressioni designate; il commento della taratura cita il nuovo corpus di riferimento.
- [x] Un ADR nuovo in `docs/adr/`: Silero v6.2 su transcribe-cpp al posto di v4 su ONNX Runtime, motivo (dimensione dell'installer, build senza ONNX Runtime), modello e hash, frame da 32 ms, esito della riverifica con i numeri prima e dopo.
- [x] `docs/sviluppo/trascrizione.md` e `registrazione.md` descrivono il nuovo VAD e il frame.
- [x] I sei controlli passano.

**Esito:** confronto in `../riverifica-v62.md`, ADR-0034, nota «superato in parte» in ADR-0023. Stessi comandi del ticket 01, stesso corpus (copiato nel worktree, hash uguali) e stessi quattro ASR; il test del corpus si ferma ancora su `parakeet-primeline`. Gli esiti non coincidono, ma nessuno peggiora per la selezione: le false Frasi scendono a zero a ogni livello (Spento 2 → 0, Bilanciato 0 → 0), le candidate di «Sì. No.» restano 2 a ogni livello, la voce attenuata passa da 3/3/3/1 a 2/2/2/2 candidate e le regressioni designate del test passano. Le soglie restano: il ramo di ritaratura non ha più un obiettivo, perché Spento non ha false Frasi. La voce attenuata perdeva «Buongiorno a tutti» (3 → 0 parole su 14): con v6.2 la prima frase diventa una candidata unica sotto −50 dBFS di RMS, e la Frase di rumore non va al modello (`livello::rumore`). Non era un effetto delle soglie della Sensibilità. Su richiesta dell'utente la Frase di rumore ora si misura sul quarto più forte dei frame (soglia −50 dBFS invariata), verificata sul Microfono muto dei Tape reali: la voce attenuata dà 14 parole su 14 a ogni livello con i quattro ASR. L'asserzione finale del test del corpus diventa «Bilanciato senza false Frasi», e il corpus non mette più alla prova lo scarto del rumore. `trascrizione.md` e `registrazione.md` descrivono VAD e frame. Resta aperto, fuori da questo ticket: un corpus con rumori che v6.2 scambia per parlato.
