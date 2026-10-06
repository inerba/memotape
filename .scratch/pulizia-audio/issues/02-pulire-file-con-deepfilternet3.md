# 02: Pulire un file con DeepFilterNet3 e salvare un Tape coerente

**What to build:** l'utente attiva la pulizia nel profilo File e audio misto, trascrive un file audio o video e ottiene un Tape il cui audio, player, Forma d'onda e Trascrizione derivano dal medesimo risultato ripulito.

**Blocked by:** 01 — Preparare un percorso audio condiviso senza cambiare il risultato.

**Status:** ready-for-agent

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [ ] DeepFilterNet3 standard gira nel backend Rust originale, su CPU, senza Python, processo esterno o servizio remoto. libDF 0.5.6 è fissato alla revisione 978576aa8400552a4ce9730838c635aa30db5e61 con Tract 0.19.16 e sole feature necessarie; il grafo completo dell'app compila su Windows/MSVC.
- [ ] Il modello distribuito con l'app è quello verificato: 7 983 136 byte, SHA-256 c94d91f70911001c946e0fabb4aa9adc37045f45a03b56008cb0c8244cb63616. Un artefatto assente o incompatibile dà un errore esplicito, senza sostituzione. Un avvio dal bundle verifica disponibilità senza download manuale o dipendenze della prova separata.
- [ ] Licenze e attribuzioni di codice e pesi sono verificate e incluse nel bundle e in Informazioni. La prova dell'installer è distinta dai sei controlli ordinari e non autorizza pubblicazione o rilascio.
- [ ] I tre profili persistenti hanno un campo indipendente di attivazione della pulizia, inizialmente falso e compatibile con le impostazioni precedenti. In questo ticket il controllo utilizzabile è File e audio misto; non si espongono controlli su percorsi ancora privi di effetto.
- [ ] Il controllo in Impostazioni è accessibile e tradotto nelle sei lingue. Spiega che il Tape conserva il risultato elaborato e che spegnere la pulizia non recupera l'originale. Non aggiunge un selettore di intensità.
- [ ] Il processore condiviso mantiene stato tra blocchi, lavora internamente a 48 kHz con hop di 480 campioni e restituisce formato e durata corretti. Audio stereo o multicanale non diventa involontariamente mono; il profilo misto non tenta di separare gli Ingressi.
- [ ] L'audio elaborato alimenta sia copia Ogg sia Silero/ASR. Il Tape e la Forma d'onda derivano da quell'audio; non viene archiviata una seconda copia originale. Le eventuali analisi dei Parlanti usano il risultato salvato.
- [ ] Metadati facoltativi compatibili con Tape v1 descrivono algoritmo/versione, Ingresso e intervalli effettivamente elaborati. Il contratto supporta fin da ora più Ingressi, cambi di configurazione e intervalli non trattati, senza attribuire a file misti Ingressi inesistenti.
- [ ] Un cambio del profilo durante la decodifica vale sui blocchi successivi, prima della coda ASR; blocchi già elaborati conservano la loro configurazione. Transizioni e chiusura compensano il ritardo senza tagliare parole o cambiare durata, verificando anche la scorciatoia libDF sui frame a energia molto bassa.
- [ ] Con pulizia spenta la selezione Silero resta quella precedente. Non vengono introdotti filtri aggiuntivi, AGC, normalizzazione, nuovi confini testuali o nuovi comportamenti di Diarizzazione.
- [ ] Annulla, errore di elaborazione o scrittura e assenza di parlato rispettano gli esiti esistenti, non lasciano un Tape parziale e non presentano una pulizia fallita come riuscita. Il file originale non viene sovrascritto.
- [ ] Test deterministici verificano coerenza fra destinazioni, durata, canali, transizioni, metadati e pulizia dei temporanei. Smoke con DFN3 reale verificano silenzio, respiri, rumore, voce bassa e risposte brevi, riportando risultati e limiti senza dedurre qualità dalla sola uscita finita.
- [ ] Contratti frontend rigenerati dal generatore, requisiti di prodotto e decisioni aggiornati per ciò che ora funziona. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; smoke ASR reali separati e sequenziali. Nessun commit o rilascio autonomo.

## Comments

La prova Rust precedente è un punto di partenza, non una verifica del bundle o della qualità nell'app.
