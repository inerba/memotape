# ADR-0025 — Profili audio condivisi nella barra della Registrazione

Data: 6 ottobre 2026. Adottata nell'implementazione locale del ticket 07.
Collaudo nativo e percettivo ancora aperto; nessun rilascio.

## Decisione

La barra riusa `CleaningProfile`, già presente in Impostazioni, in una variante
compatta accanto all'indicatore e al Guadagno di ciascun Ingresso effettivo.
La sensibilità resta indipendente dalla pulizia; nessun profilo File/misto
o intensità DFN3 viene aggiunto alla Registrazione. Le spiegazioni del livello
sono visibili, etichette e descrizioni sono associate ai controlli nativi,
con `fieldset`/`legend` per identificare l'Ingresso e id distinti fra superfici.

`SettingsProvider` serializza funzioni di aggiornamento tramite il comando
generato `setSettings`. Ogni scelta parte dal risultato confermato dell'ultima
scrittura, includendo eventuali normalizzazioni del backend. Lo stato visibile
proietta anche le scelte pendenti. Se una scrittura fallisce, la sua scelta
viene ritirata e le successive sono riapplicate allo stato confermato.
Si migrano tutti i chiamanti esistenti per impedire che uno snapshot di altre
Impostazioni sovrascriva la pulizia o la sensibilità. Nessuna nuova dipendenza.

Prima di `record`, il frontend attende le scritture già richieste (`flush`)
e impedisce avvii duplicati durante l'attesa. Il backend resta quello del 06:
`Recorder::set_audio` e l'avvio condividono il lock; il worker applica i nuovi
profili sul successivo PCM, in Pausa dalla ripresa, prima delle code ASR.
Nessun nuovo DSP, nessuna reinterpretazione di audio o testo precedenti.

Il listener già attivo in Home conserva i guasti per sessione e Ingresso,
anche se arrivano prima del montaggio della barra. La barra mostra il bypass
separatamente dalla preferenza salvata; chiudere il banner nasconde solo il
banner. La nuova sessione elimina gli avvisi precedenti. I metadati effettivi
continuano a essere quelli del contratto di pulizia del 03.

## Verifiche e limiti

Test delle azioni sulla seam di salvataggio, rendering dei tre tipi di Sorgente,
default, isolamento dei guasti obsoleti e regressioni esistenti. Sei controlli
del repo verdi, senza nuove build native parallele. I binding restano invariati
e il test del generatore passa nella suite Rust.

Il browser disponibile non raggiunge il banco Vite locale; Chrome non è
disponibile nel tool CUA. Le app installate aperte non espongono CDP. Il banco
frontend in `verification-07/ui.html` resta riproducibile ma non eseguito.
Queste prove non attestano azioni UI, Narrator, cattura WASAPI dalla barra,
riavvio nativo o ascolto. Le misure native del 06 restano riferite al core
precedente; la barra non aggiunge lavoro nel percorso audio.
