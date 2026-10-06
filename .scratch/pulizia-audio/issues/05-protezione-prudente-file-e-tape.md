# 05: Ridurre le false Frasi nei file e nei Tape preservando la voce debole

**What to build:** l'utente sceglie una sensibilità per Ingresso e trascrive file o Tape con una protezione prudente da silenzio, respiri e rumori, indipendente da DeepFilterNet3. Risposte brevi e voce bassa del corpus di regressione restano riconosciute.

**Blocked by:** 04 — Ritrascrivere un Tape senza doppia pulizia o sostituzioni parziali.

**Status:** ready-for-agent

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [ ] I profili Microfono, Audio di sistema e File e audio misto hanno una sensibilità persistente distinta dalla pulizia: Spento, Più sensibile, Bilanciato, Più selettivo. Bilanciato è il default anche per impostazioni precedenti, senza perdere gli altri valori.
- [ ] Impostazioni mostra i quattro livelli con spiegazioni accessibili e tradotte nelle sei lingue. Più selettivo dichiara il possibile costo sulle parole brevi/deboli; Spento disattiva solo la nuova protezione e lascia Silero. Non controlla l'attenuazione DFN3.
- [ ] Nei file e nei Tape la sensibilità segue l'Ingresso effettivo e funziona con pulizia accesa o spenta. Quando accesa, Silero analizza l'audio ripulito; per gli intervalli già trattati usa quello salvato senza una nuova pulizia.
- [ ] La decisione combina evidenza Silero e caratteristiche della Frase. Non usa come unico criterio soglia energetica assoluta, durata minima rigida o parole vietate. Eventuale confidenza ASR viene usata solo se realmente esposta dal runtime con significato verificato.
- [ ] Non si eliminano «grazie», inglese o altre espressioni per il loro testo. Spento con pulizia spenta è equivalente alla selezione Silero precedente; i tre livelli attivi hanno valori riproducibili derivati dalle prove.
- [ ] Un corpus annotato separa parlato/non parlato e include silenzio, respiri, rumori continui/intermittenti, italiano a bassa voce e «sì/no». Include campioni distinti per i profili e il caso Microfono silenzioso mentre l'Audio di sistema parla; gli originali sono conservati soltanto nel banco di prova.
- [ ] Con Whisper, Parakeet e Nemotron si confrontano falsi avvii, false Frasi, parole corrette perse e ascolto, con DFN3 acceso e spento. Bilanciato riduce le false Frasi del corpus rispetto al comportamento precedente senza perdite nelle risposte brevi e nella voce debole designate come regressioni; Più sensibile conserva quelle regressioni.
- [ ] Il resoconto riporta corpus, configurazioni, risultati per livello/Ingresso e limiti. Nessuna promessa di zero allucinazioni o sostituzione della prova con punteggi del paper/demo. Se la taratura non soddisfa il criterio, il ticket resta incompleto.
- [ ] I cambi in Impostazioni sono applicati all'evidenza dei blocchi decodificati successivi, prima della coda delle Frasi. Audio e Frasi già accodati non vengono reinterpretati con impostazioni nuove e non vengono troncate parole in corso ai confini di revisione.
- [ ] Test deterministici con seam VAD/motore esistenti verificano livelli, default, configurazioni ai confini e nessuna nuova Frase su silenzio digitale. Scartare una falsa Frase non rimuove quel tratto dall'audio del Tape né cambia la sua durata.
- [ ] Tape, player, ricerca, copia e Parlanti mantengono coerenza dopo la Trascrizione; errori o Annulla conservano il precedente. Requisiti/decisioni aggiornati e contratti rigenerati.
- [ ] Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; una sola build Rust alla volta. Smoke ASR veri separati e sequenziali, esiti manuali mancanti espliciti. Nessun commit o rilascio autonomo.

## Comments

La calibrazione del denoiser e la sensibilità VAD restano indipendenti. Il ticket 06 estende il comportamento alla cattura live e verifica il caso reale a due Ingressi.
