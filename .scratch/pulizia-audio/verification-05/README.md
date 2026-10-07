# Corpus e taratura del ticket 05

Il corpus è **sintetico**, annotato in `corpus/manifest.json` con SHA-256,
origine, parlato/non parlato e regressioni designate. Gli originali sono
conservati solo qui, non nei Tape degli utenti. Sei casi: silenzio digitale,
rumore continuo, rumore intermittente (4 impulsi da 125 ms ogni 500 ms),
soffio sintetico, voce TTS attenuata di 30 dB, «Sì. No.» da Elsa TTS.

`generate-corpus.py` ricostruisce i cinque WAV float32 a 16 kHz dal seed 42
e da `parlato-it.wav`, e copia `verification-02/si-no.wav`. Il file delle
risposte brevi si genera col comando già presente nel 02. `annotate-corpus.py`
aggiorna manifest e hash; non attribuisce annotazioni umane ai surrogati.
Le annotazioni di parlato sono per fixture, non allineamenti parola per parola.

## Risultati reali

`native-matrix.log`: un solo smoke release, tre ASR caricati in sequenza,
53,60 s di prova (compilazione esclusa). DFN3 preparato una volta per caso;
Silero seleziona per livello; l'inferenza identica viene riusata quando cambiano
solo le candidate ammesse. Nessuna confidenza ASR ricavata dai log.

`matrix.json`: 432 righe (3 ASR × 6 casi × 2 stati DFN3 × 4 livelli × 3 profili).
Sono **144 misure di selezione/ASR equivalenti**, replicate esplicitamente per
Microfono, Sistema e File/misto perché il PCM e la decisione sono gli stessi.
Non sono 432 registrazioni indipendenti o campioni distinti da ogni dispositivo.
`analyze-matrix.py` verifica le parole attese delle regressioni e produce
`matrix-summary.txt`. I conteggi seguenti sommano pulizia accesa e spenta,
una sola volta per ciascun profilo equivalente:

| ASR | Livello | Falsi avvii ASR | False Frasi | Parole baseline perse (parlato) |
| --- | --- | --- | --- | --- |
| Nemotron | Spento / Più sensibile | 2 | 0 | 0 |
| Nemotron | Bilanciato | 0 | 0 | 0 |
| Nemotron | Più selettivo | 0 | 0 | 14 |
| Whisper | Spento / Più sensibile | 2 | 2 | 0 |
| Whisper | Bilanciato | 0 | 0 | 0 |
| Whisper | Più selettivo | 0 | 0 | 14 |
| Parakeet | Spento / Più sensibile | 2 | 0 | 0 |
| Parakeet | Bilanciato | 0 | 0 | 0 |
| Parakeet | Più selettivo | 0 | 0 | 14 |

Un falso avvio qui è una candidata **non parlata inviata all'ASR**, dopo la
protezione; il segmentatore Silero continua a trovare una candidata sul
rumore intermittente per stato DFN3. Nei casi silenzio, rumore continuo e
soffio sintetico non ne trova alcuna. Bilanciato evita gli avvii ASR della
candidata intermittente, senza riscrivere i confini delle altre candidate.
Le false Frasi sono output ASR non vuoti su fixture annotate non parlate.
Le parole perse sono parole presenti nella trascrizione Spento che spariscono
per scarto; non sono una misura di WER rispetto a una registrazione umana.

Bilanciato/Più sensibile mantengono **le stesse candidate PCM**, con tutto il
testo italiano atteso e «Sì. No.» per Nemotron/Whisper. Parakeet conserva
«C No.», errore già nella baseline del 02 e Spento del 05. Più selettivo
scarta due candidate della voce attenuata, sette parole per stato DFN3.
Non perde le due risposte brevi in questa prova; non è una garanzia generale.

`measure.log` è la misura Silero preliminare senza ASR/DFN3: mostra che la
prima candidata debole ha picco 0,445 e media 0,420. Alzare una sola soglia
Silero l'avrebbe scartata. La ZCR lì include i frame Silero positivi ma zero;
il criterio finale esclude gli zeri dal conteggio acustico. Non usare quella
media preliminare come se fosse la ZCR finale dei soli frame non nulli.

## Prove deterministiche e limiti

`focused.log`, `tape-test.log`, `no-speech.log` e la suite Rust finale verificano:
parole deboli/brevi ammesse, rumore scartato, zero digitale, revisioni prima
del ritardo PCM, parola in corso non tagliata, coda non reinterpretata, Tape
separato Microfono silenzioso/Sistema parlato, audio intero e consumatori.
Il caso con unica falsa Frase precedente diventa un Tape senza testo: vecchio
contenuto rimosso anche da copia/ricerca; Ogg e Forma d'onda conservati.
Frontend rilegge il Tape esistente anche con `noSpeech`; il comando sincronizza
la Libreria per qualunque esito. Il collaudo clic/clipboard Tauri non è eseguito.

Restano da raccogliere **respiri umani, voce umana bassa e campioni distinti
registrati per dispositivo**, con annotazioni e ascolto. Non c'è un operatore
umano nella prova. Il soffio sintetico non è un respiro umano; attenuare Elsa
non riproduce una diversa fonazione. Tastiera/Narrator, ascolto percettivo,
errori reali di disco pieno e altri PC non sono attestati. Il ticket resta
partial; il criterio sintetico Bilanciato è passato. La validazione futura
non deve partire da una promessa di zero allucinazioni. DFN3, pesi e bundle
non sono stati ricalibrati/rivalidati da questo ticket.
