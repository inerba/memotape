# Review indipendenti del ticket 06

6 ottobre 2026. Skill code-review, due subagenti, solo delta rispetto alla
baseline del 06. Nessun test o file modificato dai revisori.

## Standards

Nessun bug certo o violazione vincolante residui. Confermati seam esistenti,
timeline per Ingresso/sessione, revisione prima di resampling/code, buffer
limitati, streaming e conservazione dei Parziali, bypass e documentazione.

Due osservazioni non bloccanti: vettori paralleli kinds/feeds/timelines con
fallback Spento (la cardinalità attuale è garantita da live_sources);
assertion dei confini interni della timeline, accompagnata dalla verifica
osservabile di PCM/durata. Nessuna refactoring fuori scope applicata.

## Spec

La verifica indipendente conclusiva, dopo le correzioni e la nuova cattura,
conferma che non restano bug certi o dichiarazioni sostanzialmente non sostenute.
L'assertion dopo Stop controlla esplicitamente il Microfono; il guard applicativo
comune congela entrambi i profili. Asimmetria di copertura non bloccante.

Tre rilievi sulla copertura delle prove, nessun difetto certo di produzione:

1. La matrice ripete lo stesso PCM con etichette Microfono/Sistema e un feed
   per esecuzione. Non attesta due catture simultanee o isolamento. Documentato
   esplicitamente: 144 configurazioni distinte, 288 inferenze realmente eseguite.
   Isolamento nel test deterministico e WASAPI Nemotron; corpus umano e WASAPI
   sui tre ASR restano aperti. Criterio nativo complessivo non spuntato.
2. Il banco originario modificava direttamente i controlli del mixer.
   Corretto usando SettingsStore temporaneo → Recorder::set_audio,
   cambi di sensibilità dei due profili, Pausa/Riprendi e congelamento a Stop.
   La cattura finale verifica il contratto backend. UI Settings ancora aperta.
3. La misura originaria osservava soltanto la coda ASR del Microfono.
   Corretto: log ogni secondo e finale per entrambe le code, nell'ordine
   Microfono/Sistema. La nuova cattura breve giustifica la ripetizione.

Ticket, README e handoff riportano `partial` e i limiti residui. I tre warning
clippy del banco sono corretti (inspect, update inutile, if annidato); controllo
finale verde. Nessuna modifica ai parametri di protezione né inferenza dal
testo riconosciuto per decidere l'ammissione.
