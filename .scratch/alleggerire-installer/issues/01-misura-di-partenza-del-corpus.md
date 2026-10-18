# 01: Misura di partenza del corpus con Silero v4

**What to build:** I risultati di riferimento della protezione con il VAD di oggi (Silero v4 su ONNX Runtime, frame da 30 ms), prima di qualunque cambio, così il ticket 03 può confrontarli. Spec: `../spec.md`, sezione "Sensibilità".

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] I due test ignorati del corpus di protezione (misura dell'evidenza e corpus a quattro livelli con tre ASR) girano in release, in sequenza, sul corpus `.scratch/pulizia-audio/verification-05/corpus`, sul codice di `main` senza modifiche.
- [ ] Per ogni campione e livello (Spento, Sensibile, Bilanciato, Selettivo), con e senza pulizia: Frasi candidate, Frasi tenute e scartate, parole delle risposte brevi e della voce debole conservate, probabilità media e picco di Silero.
- [ ] Risultati e comandi esatti salvati in `.scratch/alleggerire-installer/baseline-v4.md`, con commit e modelli usati.
- [ ] Nessuna modifica al codice.
