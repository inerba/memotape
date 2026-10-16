---
status: accepted
---

# Il Vocabolario corregge dopo la Frase e non entra nel prompt dei modelli

Data: 16 ottobre 2026. Termini in `CONTEXT.md` (Vocabolario, Termine, Frase).

transcribe-cpp 0.4.0 accetta i Termini in `RunOptions::vocabulary` e li dà ai modelli che lo supportano: Whisper li riceve nel prompt come `Glossary: T1, T2`, Qwen3-ASR nel messaggio di sistema. Memotape li ha passati così per un giorno, come Handy. Sui Tape reali (`docs/research/eco-e-loop-sui-tape-reali.md`) Whisper ricopiava il prompt su ogni Frase di rumore («Glossary», «Slaitech, Laravel») e, sul parlato vero, Whisper Medium sostituiva o perdeva 9 Frasi su 11; con Turbo 5 su 11. Basta un Termine. Le soglie no-speech non lo fermano: quando ricopia il prompt Whisper ne è sicuro.

Il Vocabolario quindi agisce solo dopo la Frase, con la correzione approssimata (`engine::vocabolario::correct`), per tutti i modelli. Un modello riceve i Termini nel prompt solo se una misura sul parlato reale, con e senza Termini, mostra che aiuta.

## Alternative scartate

- Il prompt di contesto (`RunOptions::prompt`): aiuta Turbo ma non Medium, e ricopia allo stesso modo sul rumore.
- Il vecchio `WhisperRunOptions::initial_prompt` senza «Glossary»: anche quello ricopiato su 12 Frasi di rumore su 12.
- Ritentare senza prompt le Frasi che sono solo prompt: copre l'eco sul rumore ma non le Frasi vere sostituite.
- Scartare le Frasi fatte solo di Termini: perde una Frase in cui si dice solo un Termine, e non ripara le altre.

## Conseguenze

- La correzione approssimata vale per tutti i modelli: resta il rischio di trasformare una parola vicina in un Termine («scelte» in «slide deck»).
- Qwen3-ASR non è stato misurato: è spento per prudenza, insieme a Whisper.
