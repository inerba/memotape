# 05: Ridurre le false Frasi nei file e nei Tape preservando la voce debole

**What to build:** l'utente sceglie una sensibilità per Ingresso e trascrive file o Tape con una protezione prudente da silenzio, respiri e rumori, indipendente da DeepFilterNet3. Risposte brevi e voce bassa del corpus di regressione restano riconosciute.

**Blocked by:** 04 — Ritrascrivere un Tape senza doppia pulizia o sostituzioni parziali.

**Status:** partial

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [x] I profili Microfono, Audio di sistema e File e audio misto hanno una sensibilità persistente distinta dalla pulizia: Spento, Più sensibile, Bilanciato, Più selettivo. Bilanciato è il default anche per impostazioni precedenti, senza perdere gli altri valori.
- [x] Impostazioni mostra i quattro livelli con spiegazioni accessibili e tradotte nelle sei lingue. Più selettivo dichiara il possibile costo sulle parole brevi/deboli; Spento disattiva solo la nuova protezione e lascia Silero. Non controlla l'attenuazione DFN3.
- [x] Nei file e nei Tape la sensibilità segue l'Ingresso effettivo e funziona con pulizia accesa o spenta. Quando accesa, Silero analizza l'audio ripulito; per gli intervalli già trattati usa quello salvato senza una nuova pulizia.
- [x] La decisione combina evidenza Silero e caratteristiche della Frase. Non usa come unico criterio soglia energetica assoluta, durata minima rigida o parole vietate. Eventuale confidenza ASR viene usata solo se realmente esposta dal runtime con significato verificato.
- [x] Non si eliminano «grazie», inglese o altre espressioni per il loro testo. Spento con pulizia spenta è equivalente alla selezione Silero precedente; i tre livelli attivi hanno valori riproducibili derivati dalle prove.
- [ ] Un corpus annotato separa parlato/non parlato e include silenzio, respiri, rumori continui/intermittenti, italiano a bassa voce e «sì/no». Include campioni distinti per i profili e il caso Microfono silenzioso mentre l'Audio di sistema parla; gli originali sono conservati soltanto nel banco di prova.
- [ ] Con Whisper, Parakeet e Nemotron si confrontano falsi avvii, false Frasi, parole corrette perse e ascolto, con DFN3 acceso e spento. Bilanciato riduce le false Frasi del corpus rispetto al comportamento precedente senza perdite nelle risposte brevi e nella voce debole designate come regressioni; Più sensibile conserva quelle regressioni.
- [x] Il resoconto riporta corpus, configurazioni, risultati per livello/Ingresso e limiti. Nessuna promessa di zero allucinazioni o sostituzione della prova con punteggi del paper/demo. Se la taratura non soddisfa il criterio, il ticket resta incompleto.
- [x] I cambi in Impostazioni sono applicati all'evidenza dei blocchi decodificati successivi, prima della coda delle Frasi. Audio e Frasi già accodati non vengono reinterpretati con impostazioni nuove e non vengono troncate parole in corso ai confini di revisione.
- [x] Test deterministici con seam VAD/motore esistenti verificano livelli, default, configurazioni ai confini e nessuna nuova Frase su silenzio digitale. Scartare una falsa Frase non rimuove quel tratto dall'audio del Tape né cambia la sua durata.
- [x] Tape, player, ricerca, copia e Parlanti mantengono coerenza dopo la Trascrizione; errori o Annulla conservano il precedente. Requisiti/decisioni aggiornati e contratti rigenerati.
- [x] Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; una sola build Rust alla volta. Smoke ASR veri separati e sequenziali, esiti manuali mancanti espliciti. Nessun commit o rilascio autonomo.

## Comments

La calibrazione del denoiser e la sensibilità VAD restano indipendenti. Il ticket 06 estende il comportamento alla cattura live e verifica il caso reale a due Ingressi.

### Implementazione e prove del 6 ottobre 2026

Conversazione `01a111e6-0b38-7063-8cb0-49dc32b7f352`. Implementazione locale
terminata; stato partial per corpus umano/ascolto/UI. Nessun commit, PR,
installer o rilascio. Non avviati 06/07. Delta isolato dalla baseline in
`../verification-05/delta.patch`, elenco `changed-files.txt`.

- Profili persistenti con default Bilanciato anche nei dati precedenti; livelli
  indipendenti da pulizia e validati dal backend/frontend. Selettori nativi
  etichettati, descrizioni associate e traduzioni nelle sei lingue.
- Evidenza Silero + ZCR della candidata, senza soglia RMS assoluta, durata
  minima rigida, confidenza ASR inventata o blacklist testuale. Confini
  Silero/tempi del testo e Diarizzazione esistente conservati.
- Configurazione catturata sul blocco PCM decodificato, prima dei buffer;
  timeline per Ingresso riusata nell'ASR del Tape preparato dal 04. Revisioni
  interne a un frame applicate al successivo frame Silero (30 ms), senza tagli.
  Un gruppo già ammesso conserva la candidata intera al cambio di livello.
- Scarti limitati all'invio ASR; copia Ogg, audio per i Parlanti, durata e
  Forma d'onda completi. Test separato Microfono silenzioso/Sistema parlato.
- Regressione richiesta in review: Tape con unica falsa Frase -> Bilanciato
  zero Frasi. Vecchio testo rimosso da documento, riapertura, copia e ricerca;
  Ogg byte-identico/Forma d'onda conservati. Home rilegge anche `noSpeech`;
  il comando sincronizza l'indice per ogni esito. Nessun bug certo residuo.

Sei controlli verdi: typecheck, **119 frontend**, lint frontend, fmt backend,
clippy `-D warnings`, **290 Rust**; 26 smoke ignorati. Suite Rust fuori sandbox
per Cestino. Binding rigenerati dal test di sviluppo nella cwd `src-tauri`.
Review indipendenti Standards/Spec: 0 violazioni certe, 0 bug; tre osservazioni
stilistiche non bloccanti e limiti manuali/corpus già dichiarati. Vedi review.md.

Smoke reale separato, release e sequenziale: passato in **53,60 s**, tre ASR,
DFN3 acceso/spento, quattro livelli. Nessuna nuova calibrazione del denoiser.
Bilanciato riduce gli avvii ASR sul rumore intermittente da 2 a 0 per modello
(somma on/off); Whisper riduce le false Frasi da 2 a 0. Nemotron/Parakeet
avevano già output vuoto. Più sensibile conserva regressioni ma ammette ancora
quel rumore. Bilanciato/Più sensibile conservano tutte le candidate e parole
attese della voce TTS attenuata e risposte brevi. Parakeet conserva «C No.»:
nessuna correzione ASR attribuita alla protezione. Più selettivo perde 14 parole
della voce attenuata per modello fra i due stati DFN3, senza perdere «sì/no»
in questa prova. Parametri riproducibili in ADR-0023.

Corpus annotato/hash, comandi, matrice e denominatori in
`../verification-05/README.md`, `corpus/manifest.json`, `matrix.json` e
`matrix-summary.txt`. Profili con PCM equivalente sono esplicitamente riusati,
non presentati come registrazioni distinte. Gli avvii sono quelli ASR dopo
il criterio; le candidate Silero iniziali restano invariate.

**Criteri ancora incompleti:** respiri umani, voce umana debole e risposte umane,
campioni registrati distinti per dispositivo, ascolto percettivo, collaudo
Tauri/tastiera/Narrator e cambi manuali durante Trascrivi. Soffio sintetico e
TTS attenuata non li sostituiscono. Non attestati altri PC/disco pieno reale.
La taratura sintetica soddisfa Bilanciato; il ticket non viene dichiarato done.
Licenza pesi e bundle restano aperti dal 02; nessuna nuova validazione.
