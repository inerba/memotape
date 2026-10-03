# Trascrizione solo dopo Stop

Una Registrazione non viene trascritta mentre è in corso. Dopo Stop il file diventa la Sorgente, e la Trascrizione parte quando la si avvia. Così la cattura non compete con il motore per la CPU, e su un modello c'è sempre un solo stream aperto, quindi `transcribe-cpp` non restituisce mai `Busy`. Questo rispetta anche la regola di una sola Attività alla volta. La pipeline (cattura, normalizzazione, segmentazione, motore) resta componibile, quindi la trascrizione dal vivo si può aggiungere in seguito senza riscriverla.
