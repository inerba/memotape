---
status: accepted
---

# Silero v6.2 su transcribe-cpp al posto di Silero v4 su ONNX Runtime

Data: 18 ottobre 2026. Spec: `.scratch/alleggerire-installer/spec.md`. Misure: `.scratch/alleggerire-installer/baseline-v4.md` (prima) e `riverifica-v62.md` (dopo). Termini in `CONTEXT.md` (Frase, Frase di rumore, Sensibilità, Ingresso).

## Contesto

ONNX Runtime serviva solo a Silero v4, eseguito da `vad-rs`. Pesava 14,1 MB installati, circa 3,7 MB nell'installer 0.5.3, più 325 KB di note di terze parti. Chi compila doveva scaricare a mano lo zip di ONNX Runtime e impostare `ORT_LIB_LOCATION`. transcribe-cpp, già nel bundle per la Trascrizione, dalla 0.4.0 esegue Silero v6.2 con il ruolo VAD.

## Decisione

- **Esecuzione:** Silero VAD v6.2 gira su transcribe-cpp: `VadSession`, un frame alla volta con `stream_feed`, `stream_reset` per `VoiceDetector::reset`. Il trait `VoiceDetector` non cambia.
- **Modello:** `handy-computer/silero-vad-v6.2-gguf`, file `silero-vad-v6.2-F32.gguf`, revisione `a9b2d61c20ae4ddf3644edb34969e8185a485448`, 1 241 056 byte, SHA-256 `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`, licenza MIT. È una risorsa del bundle. Si verifica prima del caricamento come DFN3, con gli errori `VadModelMissing` e `VadModelIncompatible`.
- **Backend:** CPU, la scelta di `AUTO` di transcribe-cpp per il VAD. La GPU è meno fedele al riferimento.
- **Un modello per pipeline:** transcribe-cpp fa un calcolo alla volta per modello, e un modello condiviso serializzerebbe due Ingressi in parallelo. Il VAD è un modello diverso da quello di Trascrizione, quindi non entra in conflitto con lo stream ASR.
- **Frame:** il frame della pipeline passa da 480 a 512 campioni a 16 kHz, da 30 a 32 ms: è la dimensione dei frame di Silero v6.2. `resample::FRAME_SAMPLES` e `resample::FRAME_MS` sono le sole costanti.
  - Prefill, onset, hangover e durata massima restano in millisecondi; i frame si ricavano per eccesso (10, 2, 22 e 563).
  - I tempi delle Frasi restano millisecondi sulla linea del tempo della Sorgente. Ora nascono in multipli di 32 ms, e i Tape esistenti non cambiano.
- **Soglie:** le soglie della Sensibilità (ADR-0023) e la soglia 0,4 del segmentatore restano.
- **Frase di rumore:** il livello di una Frase (`livello::rumore`) è l'RMS del quarto più forte dei suoi frame da 32 ms, non più l'RMS dell'intera Frase. La soglia resta −50 dBFS. Le pause di una Frase lunga non nascondono più una voce debole, e qualche frame forte nel rumore non basta ad alzare il livello.

## Riverifica della Sensibilità

Il corpus è quello della pulizia audio 05. Sono stati eseguiti in release, prima e dopo, i due test ignorati (misura dell'evidenza, corpus a quattro livelli) e la sonda temporanea delle probabilità con la pulizia, con quattro ASR: Nemotron, Cohere, Whisper medium e Parakeet. Il test del corpus si ferma al primo modello non scaricato del catalogo, sia prima che dopo. I nomi dei livelli sono quelli del codice: Sensibile e Selettivo sono Più sensibile e Più selettivo.

| Esito | Silero v4 | Silero v6.2 |
| --- | --- | --- |
| Candidate senza pulizia: rumore intermittente (Spento / Sensibile / Bilanciato / Selettivo) | 1 / 1 / 0 / 0 | 0 / 0 / 0 / 0 |
| Candidate senza pulizia: campione `voce-attenuata-30db` | 3 / 3 / 3 / 1 | 2 / 2 / 2 / 2 |
| Candidate «Sì. No.», con e senza pulizia | 2 a ogni livello | 2 a ogni livello |
| Candidate con la pulizia: rumore e voce attenuata | 0 | 0 |
| False Frasi a Spento / Bilanciato, somma dei quattro ASR | 2 / 0 | 0 / 0 |
| Parole della voce attenuata senza pulizia (Spento–Bilanciato) | 3 su 14 | 0 su 14; 14 su 14 a ogni livello con la Frase di rumore sul quarto più forte |
| Picco di Silero sul rumore intermittente / sulla voce attenuata | 0,878 / 0,775 | 0,020 / 0,99999 |

v6.2 separa il parlato dal rumore molto meglio di v4.

- **Rumore:** sparisce l'unica falsa Frase del corpus.
- **Voce attenuata:** resta candidata anche a Selettivo, perché il picco supera 0,98.
- **Regressioni:** quelle designate passano, cioè stesse candidate a Spento, Sensibile e Bilanciato sulla voce attenuata e su «Sì. No.».
- **Soglie:** restano. Il ramo di ritaratura del criterio («finché Bilanciato riduce le false Frasi») non ha più un obiettivo, perché Spento non ne ha. Le candidate della voce attenuata superano già 0,98 di picco, e la protezione le ammette a ogni livello.

Con il filtro del rumore sull'intera Frase si perdeva il testo della voce attenuata, ma la perdita non veniva dalla selezione:

- Con v6.2 la prima frase parlata è un tratto unico. La probabilità per frame (`log/frame-v62.log`) sta sopra 0,4 nei frame 6..42 e 48..108; nei frame 43–47 (160 ms) scende sotto 0,4, meno dell'hangover di 22 frame. Le prime due candidate di v4 diventano una, a −52,0 dBFS di RMS.
- Con v4 Silero vedeva solo l'inizio della frase: la prima candidata (frame 6..43 da 30 ms, −49,7 dBFS) aveva 3 frame sopra 0,4 e conteneva «Buongiorno a tutti». A Selettivo, che la scartava, si perdevano proprio quelle 3 parole.
- Per spezzare la candidata servirebbe un hangover sotto i 160 ms, cioè una durata del segmentatore, che resta in millisecondi. Le soglie non bastano.
- Una Frase di rumore (`livello::rumore`, −50 dBFS) non va al modello. Con v4 così si perdevano già due candidate su tre, 11 parole su 14; con v6.2 si perdeva anche la candidata unita. La causa è dedotta dall'RMS delle candidate, calcolato sul WAV: il test non ha un logger che stampi le Frasi di rumore.

### Frase di rumore sul quarto più forte dei frame

Il filtro è nato per il Microfono muto dei Tape reali (`docs/research/eco-e-loop-sui-tape-reali.md`), su cui Whisper ricopiava il prompt e Cohere inventava. Una sonda temporanea (`log/livello-sonda.log`) ha confrontato più livelli sulle Frasi del corpus e sui due Tape reali, copiati senza toccare gli originali: RMS intero, RMS senza i frame muti, RMS del 10%, del 25% e del 50% più forte dei frame, frame più forte. Il quarto più forte separa i due casi sulle Frasi misurate, con 2,3 dB di margine sulla voce e 2,7 dB sul muto:

| Frasi | RMS intero | Quarto più forte |
| --- | --- | --- |
| Voce attenuata, le due candidate di v6.2 | −52,0 / −52,3 | −47,1 / −47,7 |
| Microfono muto, le 12 Frasi tagliate da v4 (tabella A della ricerca) | da −58,7 a −75,8 | da −52,7 a −71,1 |
| Microfono muto, le 2 Frasi tagliate da v6.2 | −73,0 / −77,8 | −67,6 / −72,3 |
| Audio di sistema dei due Tape, 26 Frasi | da −45,3 a −14,5 | da −39,5 a −9,2 |

Il frame più forte e il 10% più forte non separano: nel muto ci sono frame a −42 dBFS. Il 50% lascia la voce attenuata a 0,6 dB dalla soglia. Sulla Frase 23250–25410 la sonda misura −58,7 dBFS di RMS intero contro i −62,3 della ricerca (le altre 11 coincidono entro 0,3 dB): la sonda la ritaglia sui frame da 32 ms della traccia ricampionata, non dal PCM che il motore ricevette allora; la differenza non è spiegata, e il valore della sonda è il più sfavorevole.

Su finestre scorrevoli dell'intero Microfono muto il quarto più forte vale al massimo −52,0 su 2 s, ma −49,1 su 1 s e −46,4 su 0,5 s. Una Frase di circa 1 s nel tratto più forte del muto passerebbe quindi il filtro. Frasi così corte esistono: prefill, onset e hangover fanno circa 1,1 s, ma la fine dell'audio, la Pausa o l'inizio della Sorgente le accorciano, e la tabella A ha una Frase da 990 ms. Sui due Tape v6.2 non apre Frasi su quei tratti.

Con il nuovo filtro il test del corpus dà le 14 parole della voce attenuata con tutti e quattro gli ASR e a ogni livello, Selettivo compreso. Le false Frasi restano zero, e «Sì. No.» resta uguale a v6.2 con il filtro vecchio.

L'asserzione finale del test del corpus («Bilanciato ha meno false Frasi di Spento») non regge più quando Spento non ne ha: diventa «Bilanciato non tiene false Frasi». Non è mai stata eseguita, né prima né dopo, perché il test si ferma prima sul modello mancante; il risultato vale sia per v4 sia per v6.2, calcolato a mano sulle righe stampate.

## Conseguenze

- ONNX Runtime, `vad-rs` e `silero_vad.onnx` possono uscire da build e bundle: `ORT_LIB_LOCATION` non serve più. Lo fa il ticket 04 di alleggerire-installer; non reintrodurre ONNX Runtime senza rileggere questo ADR.
- Con v6.2 il corpus non contiene più false Frasi a Spento, quindi non mette più alla prova lo scarto del rumore della protezione: la nuova asserzione passerebbe anche con la protezione spenta. Per ritarare la Sensibilità in futuro serve un corpus con rumori che v6.2 scambia per parlato.
- La Frase di rumore sul quarto più forte dei frame ha 2,3 dB di margine sulla voce a −30 dB e 2,7 dB sulle Frasi di muto misurate. Una Frase di muto di 1 s o meno nel suo tratto più forte passerebbe il filtro (−49,1 dBFS), se Silero ci aprisse una Frase: il filtro resta una rete di sicurezza, e il primo argine sul muto è Silero v6.2. Una voce più debole di questa, o un muto con più frame forti di quelli misurati, va riverificata con lo stesso criterio.
- La riverifica è sul corpus sintetico. L'ascolto umano e i respiri restano non collaudati, come nell'ADR-0023.

## Alternative scartate

- **Tenere Silero v4 su ONNX Runtime:** costa peso e un passo di build per un solo componente.
- **Silero v4 convertito in GGUF:** il porting di transcribe-cpp 0.4.0 è Silero v6.2 (il caricatore accetta anche i `ggml-silero-v5.1.2.bin` di whisper.cpp, non v4).
- **Frame da 30 ms con uno strato che traduce le probabilità tra due griglie:** è più codice, per conservare una durata che non ha valore in sé.
- **`VadSession::run` con i segmenti di transcribe-cpp:** la pipeline segmenta da sé, con prefill, hangover, durata massima e protezione propri.
