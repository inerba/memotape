---
status: accepted
---

# Trascrizione dei file: una Frase alla volta, con `run`

Trascrivi su un file non mostra più il testo man mano, quindi non servono Parziali: ogni Frase del VAD va al modello con una chiamata `Session::run`, anche con Nemotron. Lo streaming resta solo per la Trascrizione dal vivo. Decodifica e VAD girano in un thread a parte, in parallelo al modello. Misurato su 10 minuti di riunione:
- Nemotron passa da 93 a 32 s;
- Parakeet da 20 a 15 s;
- Whisper da 42 a 38 s.

Con Parakeet e Whisper il guadagno viene da decodifica e VAD in parallelo.

## Considered Options

- **Tutto il file in una chiamata, senza VAD.** Scartata:
  - nessuna percentuale durante la chiamata;
  - Annulla non ferma Parakeet e Nemotron una volta partiti;
  - la memoria di Parakeet cresce col quadrato della durata;
  - Whisper trascrive anche i silenzi.
- **Frasi a gruppi con `run_batch`.** Scartata:
  - in transcribe-cpp 0.2.4 chiude il processo con Parakeet e Nemotron (assert di ggml);
  - con Whisper non è più veloce, e un gruppo allunga l'attesa di Annulla.
  - Da riprovare con una release di transcribe-cpp che corregga l'assert.

Fonte: `docs/research/trascrizione-file-veloce.md`.
