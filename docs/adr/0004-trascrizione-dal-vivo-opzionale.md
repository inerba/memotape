---
status: accepted
---

# Trascrizione dal vivo opzionale

La Registrazione può trascrivere mentre registra, se l'utente attiva la Trascrizione dal vivo con una casella accanto a Registra. La casella è spenta per default, perché su PC lenti il motore può non stare al passo con l'audio. Si trascrive il mix, oppure, in modalità Ingressi separati, ogni Ingresso con una propria istanza del modello: `transcribe-cpp` ammette un solo stream attivo per modello, quindi la RAM o la VRAM raddoppiano. Le Frasi in eccesso vanno in coda e la Registrazione non rallenta mai: dopo Stop la Trascrizione finisce di smaltire la coda. In modalità Ingressi separati si salva anche l'audio di ogni Ingresso, dentro il Bino (ADR-0005), perché la Diarizzazione (solo a posteriori, vedi `docs/research/diarizzazione.md`) ha bisogno dell'audio di ciascun Ingresso. La Diarizzazione dal vivo resta fuori finché non è disponibile Nemotron-3-Diarization (ADR-0006). Sostituisce l'ADR-0003.
