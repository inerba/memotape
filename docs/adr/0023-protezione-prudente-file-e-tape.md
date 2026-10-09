# ADR-0023 — Protezione prudente delle candidate Frasi nei file e Tape

> Superato in parte da ADR-0029 (9 ottobre 2026): la Sensibilità non vale più in Trascrivi su
> un Tape, che usa l'audio salvato così com'è. Per i file importati resta com'è descritta qui.

Data: 6 ottobre 2026. Stato: adottata per l'implementazione locale del ticket 05;
taratura su corpus sintetico, validazione umana/ascolto ancora parziale.

## Decisione

`Sensibilita` ha quattro valori persistenti per Microfono, Sistema e File/audio
misto: Spento, Sensibile, Bilanciato (default serde), Selettivo. Le Impostazioni
mostrano nomi e compromessi nelle sei lingue, separati da pulizia e Guadagno.
Il ticket 06 estende il criterio alla Registrazione con ammissione sui prefissi
(ADR-0024); i controlli nella toolbar restano previsti dal ticket 07.

Si mantiene il segmentatore Silero: soglia 0,4, onset 60 ms, prefill 300 ms,
hangover 700 ms, massimo 18 s. Prima della coda ASR, ogni candidata completa
combina probabilità Silero e attraversamenti dello zero (ZCR) dei frame con
probabilità >= 0,4 e campioni non tutti zero. Non si normalizza, non si usa una
soglia RMS assoluta, non si impone una durata minima e non si legge il testo.
La confidenza ASR non è usata: `conf=… ms` nei log non è una probabilità.

| Livello | ZCR media massima | Probabilità Silero media minima |
| --- | --- | --- |
| Più sensibile | 0,48 | 0,4 |
| Bilanciato | 0,40 | 0,4 |
| Più selettivo | 0,20 | 0,6 |

In tutti i livelli attivi, un picco Silero >= 0,98 ammette anche evidenza non
periodica, preservando consonanti brevi. Questo non garantisce che un rumore
con Silero molto alto venga scartato. Spento ammette ogni candidata della
selezione precedente, anche con pulizia attiva; non modifica Silero né DFN3.
La ZCR è una caratteristica relativa del segnale, non identifica il parlato
umano da sola. I valori sono locali e riproducibili, non soglie universali.

## Configurazione e audio

`ProtectionTimeline::capture` avvolge `AudioProcessor`, senza un'altra seam DSP.
Legge la sensibilità all'ingresso del blocco decodificato, prima dei buffer del
processore. Registra solo cambi in coordinate di campioni a 16 kHz, calcolate
sulla posizione assoluta del PCM; non legge valori al consumo dell'ASR.
Il frame Silero da 30 ms usa la revisione all'inizio del frame: un confine
interno al frame diventa effettivo dal frame successivo, senza tagli acustici.

Nel percorso 04 ogni traccia del Tape viene prima decodificata e preparata in
un Ogg temporaneo. Una timeline per Ingresso resta associata a quel percorso,
anche se l'Ogg originale viene riusato byte per byte. La seconda decodifica
ASR riusa la timeline; cambi successivi in Impostazioni non reinterpretano
quell'audio. Un Tape senza tracce distinte usa File/misto per la sensibilità,
anche quando `pulizia_audio` ricorda il dispositivo singolo precedente.

L'evidenza della candidata è raggruppata per revisione. Basta che un gruppo
sia ammesso per conservare la candidata intera: un cambio più selettivo non
revoca una parte già ammessa e non tronca la parola in corso. Non si chiude la
Frase a un cambio. Candidate già nella coda sono immutabili. I confini e tempi
della candidata ammessa restano quelli precedenti, poi vale la Diarizzazione
esistente. Si conservano soltanto le osservazioni dell'ultima candidata (buffer
limitato a 612 frame), oltre alle revisioni compattate dell'Attività.

Uno scarto riguarda solo l'invio all'ASR. Copia Ogg, durata, PCM per i Parlanti,
Forma d'onda e audio del Tape mantengono tutti i campioni. La sostituzione
atomica/errori/Annulla del 04 resta il punto di commit del documento.

## Prove e limiti

Corpus e risultati: `.scratch/pulizia-audio/verification-05/`. Sei casi annotati:
silenzio digitale, rumore continuo/intermittente a seed 42, soffio sintetico,
TTS italiana attenuata di 30 dB e TTS «Sì. No.». Matrice reale con tre ASR e
DFN3 acceso/spento; lo stesso PCM per profili equivalenti è misurato una volta.
Gli ASR sono caricati una volta per modello, sequenzialmente. Frasi identiche
fra livelli riusano l'inferenza: cambia solo l'ammissione prima della coda.

Bilanciato passa da due avvii ASR sul rumore intermittente a zero per modello;
Whisper passa da due false Frasi a zero. Nemotron e Parakeet già restituivano
vuoto: non si attribuisce al filtro una correzione ASR inesistente. Più sensibile
mantiene quel rumore. Bilanciato/Più sensibile conservano le candidate e tutte
le parole della voce attenuata e delle risposte brevi. Parakeet resta «C No»,
come in Spento: non viene dichiarata correzione di «Sì». Più selettivo perde
14 parole della voce attenuata fra i due stati pulizia, per ciascun ASR.

Questi sono avvii ASR dopo la decisione: le candidate Silero iniziali non
cambiano. La matrice non misura respiri o voci umane reali, ascolto percettivo,
campioni distinti registrati da ogni dispositivo, tastiera/Narrator nella UI
Tauri o prestazioni live. Il Tape separato con Microfono silenzioso/Sistema
parlato è un test deterministico del core, non una cattura umana. Il ticket
resta partial per questi limiti; il criterio sintetico Bilanciato è passato.
Licenza dei pesi e bundle restano aperti dal 02, senza nuove affermazioni.
