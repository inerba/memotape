# 07: Regolare pulizia e sensibilità dalla barra della Registrazione

**What to build:** accanto ai controlli di ciascun Ingresso, l'utente modifica pulizia e sensibilità al volo dalla barra della Registrazione. La barra e Impostazioni mostrano gli stessi valori, li conservano al riavvio e applicano i cambi soltanto all'audio successivo.

**Blocked by:** 06 — Proteggere la Trascrizione dal vivo per Ingresso.

**Status:** partial

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [x] Nella barra, per ciascun Ingresso registrato, sono presenti attivazione della pulizia e quattro livelli di sensibilità, distinti tra loro e dal Guadagno. Non si aggiunge una scala di intensità DFN3 o un controllo File e audio misto per una Registrazione con Ingressi reali.
- [x] La barra usa lo stesso stato persistente di Impostazioni. Modifiche da entrambe le superfici si riflettono subito sull'altra, senza perdere aggiornamenti ravvicinati o a cavallo dell'avvio; i valori sono conservati al riavvio.
- [x] I cambi usano il contratto live già implementato: valgono sull'audio successivo prima delle code ASR, in Pausa dalla ripresa. Non ripuliscono l'audio precedente e non ritrattano Frasi/Parziali già pubblicati.
- [x] I controlli sono utilizzabili con tastiera e lettore di schermo, rendono riconoscibile l'Ingresso, mostrano il valore corrente e spiegano Spento e il compromesso dei livelli. Etichette, spiegazioni e avvisi sono tradotti nelle sei lingue.
- [x] Un guasto mostra l'avviso nella sessione e nell'Ingresso corretti, distinguendo il valore scelto dal trattamento effettivamente interrotto. Gli eventi di sessioni precedenti non alterano la barra corrente e il bypass rimane verificabile nei metadati salvati.
- [x] Test frontend delle azioni verificano sincronizzazione, persistenza, default, visibilità per Sorgente di Registrazione, aggiornamenti concorrenti ed eventi obsoleti. Gli accessi backend passano dai contratti applicativi generati, rigenerati attraverso il generatore.
- [ ] Un collaudo nativo prova cambi dalla barra durante parlato, silenzio e Pausa, con due Ingressi, DFN3 e ASR attivi. Confronta audio precedente/successivo al cambio, durata, metadati, ultime parole a Stop e Tape riaperto; i soli test puri del frontend non attestano questo punto.
- [ ] La prova comprende più sessioni, sensibilità Spento con Silero attivo, pulizia indipendente, guasto su un solo Ingresso e valori persistenti dopo riavvio. Il carico resta entro i criteri verificati nel ticket 06; si ripete la misura se le modifiche introducono nuovi rischi o regressioni.
- [x] Requisiti e decisioni descrivono i controlli ora disponibili e riportano i limiti delle prove. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; una sola build Rust alla volta, smoke reali sequenziali. Verifiche native mancanti restano esplicite. Nessun commit o rilascio autonomo.

## Comments

Questo ticket rende accessibile dalla barra un comportamento già verificabile attraverso Impostazioni. Non avvia nuove azioni autonome di pulizia dei Tape.

## Esito locale del 6 ottobre 2026

Implementazione locale completata nella conversazione
`01a1122c-7f30-7ac2-99b3-8a90f0710a37`. Le spunte di implementazione
descrivono codice e test sulle seam: non attestano azioni UI native o Narrator.
Il riavvio frontend usa uno storage IPC simulato; la persistenza Rust e il
contratto live sono coperti dalle prove esistenti. Binding invariati e test
del generatore verde. La barra non aggiunge DSP, callback o code audio.

`SettingsProvider` serializza intenzioni funzionali, mostra le scelte pendenti,
riapplica le successive dopo errore e attende i salvataggi prima di Registra.
Tutti i chiamanti sono migrati per non sovrascrivere altri campi. Home passa
i guasti già raccolti alla barra; chiudere il banner conserva il bypass locale.

Sei controlli verdi: 126 test frontend (7 nuovi), 296 test Rust e 29 ignorati,
typecheck, lint frontend, fmt e clippy. Evidenza e denominatori in
`../verification-07/README.md`. Delta rispetto alla baseline iniziale,
non HEAD: `../verification-07/delta.patch`.

Collaudo nativo aperto: app installata già aperta senza CDP, CUA nativo
disabilitato, Chrome indisponibile e IAB in timeout verso il banco locale.
Il banco `ui.html`/`ui.tsx` con IPC finto è conservato ma non eseguito.
Restano da verificare dalla barra due Ingressi con DFN3/ASR, parlato/silenzio,
Pausa/Riprendi, Stop e ultime parole, ascolto precedente/successivo, metadati
e Tape riaperto, più sessioni/riavvio reale, tastiera/Narrator, temi e sei lingue.
Nessuna nuova misura di carico: il percorso audio del 06 non è cambiato.
Licenza pesi e installer restano aperti dal 02. Nessun commit o rilascio.

Review finale indipendente: Standards e Spec senza rilievi di implementazione
residui. Corretti i rilievi Standards su bordo, stato disabilitato, `flush`
durante nuovi salvataggi, rollback obsoleto del form e mapping/nome dei guasti.
I limiti delle prove UI/native sono confermati dalla review Spec; non vengono
considerati passati. Report: `../verification-07/review.md`.
