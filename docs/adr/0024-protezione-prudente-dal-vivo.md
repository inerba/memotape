# ADR-0024 — Protezione del parlato dal vivo per Ingresso

Data: 6 ottobre 2026. Adottata per l'implementazione locale del ticket 06.
Taratura sintetica e carico WASAPI documentati in `verification-06`; corpus
umano, ascolto e collaudo Tauri restano criteri aperti. Nessuna distribuzione.

## Decisione

Si riusano `Sensibilita`, `ProtectionTimeline` e `PhraseEvidence` del 05,
con gli stessi parametri Silero/ZCR. File e Tape mantengono la decisione sulle
candidate complete. Dal vivo la decisione è verificata sui prefissi: appena
l'evidenza del prefisso ammette un gruppo, la candidata passa intera al motore,
con prefill e tempi originali. L'ammissione resta fino a fine Frase, anche se
il seguito è meno convincente o il livello cambia. Nessun testo viene pubblicato
prima dell'ammissione, né ritirato successivamente dalla protezione. Nemotron
continua a leggere a trazione e a emettere Parziali durante il parlato.

La candidata non ammessa conserva al massimo il limite del segmentatore
(600 frame, 18 s). Le osservazioni Silero necessarie al prefill occupano
12 frame; dopo l'ammissione non resta un secondo buffer della Frase intera.
Nessuna nuova coda illimitata o inferenza nella callback WASAPI. Il canale
ASR preesistente resta invariato. Spento conserva la selezione Silero.

## Configurazione e PCM

`LiveFeed` e `LiveFrames` condividono una timeline distinta per Ingresso e
sessione. `Mixer::protect` registra il livello al prossimo campione acquisito,
usando i frame del dispositivo sulla linea del tempo del mixer, pause escluse,
convertiti a coordinate assolute a 16 kHz. È l'estensione live di `capture`
del 05: qui il punto di cattura precede anche il ricampionatore del dispositivo,
non solo i buffer del processore PCM. I campioni ancora nei buffer mantengono
la loro revisione. Il frame Silero usa il livello del suo primo campione.

Un cambio della sola sensibilità non scarica né resetta DFN3: tracce, mix,
durata, Forma d'onda e Ogg restano gli stessi. Il worker legge i controlli
persistenti prima di elaborare nuovo PCM. Avvio e `Recorder::set_audio`
condividono il lock già adottato nel 03. In Pausa i cambi si applicano dalla
ripresa; Stop congela i controlli applicati anche per lo smaltimento successivo.
Un singolo dispositivo usa il proprio profilo, benché l'ASR si chiami Mix.

I guasti DFN3 recuperano il PCM pendente in bypass sul solo Ingresso, con
l'avviso di sessione del 03. La timeline resta indipendente dal successo della
pulizia. Gli scarti riguardano soltanto ASR: audio salvato e analisi finale
dei Parlanti conservano tutti i campioni. Diarizzazione dopo Stop e fine ASR.
Impostazioni è l'unico controllo al volo in questo ticket; toolbar nel 07.

## Evidenza e limiti

I test osservano selezione, Parziali prima della fine del parlato, PCM, durata,
revisioni con coda arretrata, cambi dentro/fuori Frase, taglio massimo,
Pausa/Riprendi, Stop, sessioni e Ingressi indipendenti e bypass.

`protezione_live_prefissi_corpus` usa DFN3/Silero reali e le sei fixture
annotate del 05. `protezione_live_corpus_tre_asr` usa realmente il percorso
streaming, con tre ASR sequenziali, quattro livelli, pulizia on/off e due
esecuzioni etichettate Microfono/Sistema sul medesimo PCM sintetico. Ogni
esecuzione ha un solo feed: non attesta isolamento simultaneo. Non deduce
l'esito dal batch. `protezione_due_ingressi_nativi_release` estende il banco WASAPI del 03
con la protezione, i Parziali, cambi di sensibilità e riapertura del Tape.
Risultati e denominatori sono nel ticket e nell'handoff, insieme ai sei controlli.

TTS attenuata e soffio sintetico non attestano voce debole o respiri umani.
Le prove del core non attestano il player visibile, tastiera/Narrator o ascolto
percettivo delle transizioni. Il carico di 28 s su questo PC non attesta ore
di Registrazione o altro hardware. Licenza dei pesi e bundle restano aperti
dal 02; nessuna nuova calibrazione DFN3 o rilascio è parte del 06.
