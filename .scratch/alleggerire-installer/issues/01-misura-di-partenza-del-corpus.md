# 01: Misura di partenza del corpus con Silero v4

**What to build:** I risultati di riferimento della protezione con il VAD di oggi (Silero v4 su ONNX Runtime, frame da 30 ms), prima di qualunque cambio, così il ticket 03 può confrontarli. Spec: `../spec.md`, sezione "Sensibilità".

**Blocked by:** None (can start immediately)

**Status:** done

- [x] I due test ignorati del corpus di protezione (misura dell'evidenza e corpus a quattro livelli con tre ASR) girano in release, in sequenza, sul corpus `.scratch/pulizia-audio/verification-05/corpus`, sul codice di `main` senza modifiche.
- [x] Per ogni campione e livello (Spento, Sensibile, Bilanciato, Selettivo), con e senza pulizia: Frasi candidate, Frasi tenute e scartate, parole delle risposte brevi e della voce debole conservate, probabilità media e picco di Silero.
- [x] Risultati e comandi esatti salvati in `.scratch/alleggerire-installer/baseline-v4.md`, con commit e modelli usati.
- [x] Nessuna modifica al codice.

**Esito:** `../baseline-v4.md`. La misura dell'evidenza passa. Il test del corpus scorre tutto il catalogo (oggi 18 modelli) e si ferma al primo non scaricato, `parakeet-primeline`: misurati Nemotron, Cohere, Whisper medium e Parakeet, non Whisper large-v3 turbo; asserzione finale e `matrix.json` non arrivano. Per i tre ASR originali servono i modelli del catalogo che precedono turbo, o un test che salti quelli assenti (modifica al codice, fuori da questo ticket). Misura su quattro ASR accettata il 18 ottobre 2026. Le probabilità con la pulizia vengono da una sonda temporanea riportata lì. Rispetto alla matrice del 05 le candidate sono le stesse; cambia solo il testo dell'ASR sulla voce attenuata senza pulizia.
