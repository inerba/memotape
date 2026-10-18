# 03: Riverifica della Sensibilità e ADR

**What to build:** La Sensibilità che si comporta come prima con Silero v6.2: stessi esiti sul corpus del ticket 01, o soglie ritarate sullo stesso corpus con lo stesso criterio. L'interfaccia non cambia. Spec: `../spec.md`, sezione "Sensibilità".

**Blocked by:** 01, 02

**Status:** ready-for-agent

- [ ] Gli stessi test ignorati del ticket 01 rigirano sul codice del ticket 02; confronto campione per campione e livello per livello con `baseline-v4.md`.
- [ ] Se gli esiti coincidono (Frasi tenute e scartate, risposte brevi e voce debole conservate), le soglie restano. Altrimenti si ritarano le soglie della protezione, del segmentatore e del filtro sugli zeri finché Bilanciato riduce le false Frasi senza perdere le regressioni designate; il commento della taratura cita il nuovo corpus di riferimento.
- [ ] Un ADR nuovo in `docs/adr/`: Silero v6.2 su transcribe-cpp al posto di v4 su ONNX Runtime, motivo (dimensione dell'installer, build senza ONNX Runtime), modello e hash, frame da 32 ms, esito della riverifica con i numeri prima e dopo.
- [ ] `docs/sviluppo/trascrizione.md` e `registrazione.md` descrivono il nuovo VAD e il frame.
- [ ] I sei controlli passano.
