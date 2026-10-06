# 07: Regolare pulizia e sensibilità dalla barra della Registrazione

**What to build:** accanto ai controlli di ciascun Ingresso, l'utente modifica pulizia e sensibilità al volo dalla barra della Registrazione. La barra e Impostazioni mostrano gli stessi valori, li conservano al riavvio e applicano i cambi soltanto all'audio successivo.

**Blocked by:** 06 — Proteggere la Trascrizione dal vivo per Ingresso.

**Status:** needs-triage

Bozza in attesa dell'approvazione della suddivisione. Requisiti: [spec approvata](../spec.md).

- [ ] Nella barra, per ciascun Ingresso registrato, sono presenti attivazione della pulizia e quattro livelli di sensibilità, distinti tra loro e dal Guadagno. Non si aggiunge una scala di intensità DFN3 o un controllo File e audio misto per una Registrazione con Ingressi reali.
- [ ] La barra usa lo stesso stato persistente di Impostazioni. Modifiche da entrambe le superfici si riflettono subito sull'altra, senza perdere aggiornamenti ravvicinati o a cavallo dell'avvio; i valori sono conservati al riavvio.
- [ ] I cambi usano il contratto live già implementato: valgono sull'audio successivo prima delle code ASR, in Pausa dalla ripresa. Non ripuliscono l'audio precedente e non ritrattano Frasi/Parziali già pubblicati.
- [ ] I controlli sono utilizzabili con tastiera e lettore di schermo, rendono riconoscibile l'Ingresso, mostrano il valore corrente e spiegano Spento e il compromesso dei livelli. Etichette, spiegazioni e avvisi sono tradotti nelle sei lingue.
- [ ] Un guasto mostra l'avviso nella sessione e nell'Ingresso corretti, distinguendo il valore scelto dal trattamento effettivamente interrotto. Gli eventi di sessioni precedenti non alterano la barra corrente e il bypass rimane verificabile nei metadati salvati.
- [ ] Test frontend delle azioni verificano sincronizzazione, persistenza, default, visibilità per Sorgente di Registrazione, aggiornamenti concorrenti ed eventi obsoleti. Gli accessi backend passano dai contratti applicativi generati, rigenerati attraverso il generatore.
- [ ] Un collaudo nativo prova cambi dalla barra durante parlato, silenzio e Pausa, con due Ingressi, DFN3 e ASR attivi. Confronta audio precedente/successivo al cambio, durata, metadati, ultime parole a Stop e Tape riaperto; i soli test puri del frontend non attestano questo punto.
- [ ] La prova comprende più sessioni, sensibilità Spento con Silero attivo, pulizia indipendente, guasto su un solo Ingresso e valori persistenti dopo riavvio. Il carico resta entro i criteri verificati nel ticket 06; si ripete la misura se le modifiche introducono nuovi rischi o regressioni.
- [ ] Requisiti e decisioni descrivono i controlli ora disponibili e riportano i limiti delle prove. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; una sola build Rust alla volta, smoke reali sequenziali. Verifiche native mancanti restano esplicite. Nessun commit o rilascio autonomo.

## Comments

Questo ticket rende accessibile dalla barra un comportamento già verificabile attraverso Impostazioni. Non avvia nuove azioni autonome di pulizia dei Tape.
