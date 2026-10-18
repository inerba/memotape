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

## Riverifica della Sensibilità

Il corpus è quello della pulizia audio 05. Sono stati eseguiti in release, prima e dopo, i due test ignorati (misura dell'evidenza, corpus a quattro livelli) e la sonda temporanea delle probabilità con la pulizia, con quattro ASR: Nemotron, Cohere, Whisper medium e Parakeet. Il test del corpus si ferma al primo modello non scaricato del catalogo, sia prima che dopo. I nomi dei livelli sono quelli del codice: Sensibile e Selettivo sono Più sensibile e Più selettivo.

| Esito | Silero v4 | Silero v6.2 |
| --- | --- | --- |
| Candidate senza pulizia: rumore intermittente (Spento / Sensibile / Bilanciato / Selettivo) | 1 / 1 / 0 / 0 | 0 / 0 / 0 / 0 |
| Candidate senza pulizia: campione `voce-attenuata-30db` | 3 / 3 / 3 / 1 | 2 / 2 / 2 / 2 |
| Candidate «Sì. No.», con e senza pulizia | 2 a ogni livello | 2 a ogni livello |
| Candidate con la pulizia: rumore e voce attenuata | 0 | 0 |
| False Frasi a Spento / Bilanciato, somma dei quattro ASR | 2 / 0 | 0 / 0 |
| Parole della voce attenuata senza pulizia (Spento–Bilanciato) | 3 su 14 | 0 su 14 |
| Picco di Silero sul rumore intermittente / sulla voce attenuata | 0,878 / 0,775 | 0,020 / 0,99999 |

v6.2 separa il parlato dal rumore molto meglio di v4.

- **Rumore:** sparisce l'unica falsa Frase del corpus.
- **Voce attenuata:** resta candidata anche a Selettivo, perché il picco supera 0,98.
- **Regressioni:** quelle designate passano, cioè stesse candidate a Spento, Sensibile e Bilanciato sulla voce attenuata e su «Sì. No.».
- **Soglie:** restano. Il ramo di ritaratura del criterio («finché Bilanciato riduce le false Frasi») non ha più un obiettivo, perché Spento non ne ha. Le candidate della voce attenuata superano già 0,98 di picco, e la protezione le ammette a ogni livello.

La sola perdita è nel testo della voce attenuata, ma non viene dalla selezione:

- Con v6.2 la prima frase parlata è un tratto unico. La probabilità per frame (`log/frame-v62.log`) sta sopra 0,4 nei frame 6..42 e 48..108; nei frame 43–47 (160 ms) scende sotto 0,4, meno dell'hangover di 22 frame. Le prime due candidate di v4 diventano una, a −52,0 dBFS di RMS.
- Con v4 Silero vedeva solo l'inizio della frase: la prima candidata (frame 6..43 da 30 ms, −49,7 dBFS) aveva 3 frame sopra 0,4 e conteneva «Buongiorno a tutti». A Selettivo, che la scartava, si perdevano proprio quelle 3 parole.
- Per spezzare la candidata servirebbe un hangover sotto i 160 ms, cioè una durata del segmentatore, che resta in millisecondi. Le soglie non bastano.
- Una Frase di rumore (`livello::rumore`, −50 dBFS) non va al modello. Con v4 così si perdevano già due candidate su tre, 11 parole su 14; con v6.2 si perde anche la candidata unita. La causa è dedotta dall'RMS delle candidate, calcolato sul WAV: il test non ha un logger che stampi le Frasi di rumore.

L'asserzione finale del test del corpus («Bilanciato ha meno false Frasi di Spento») non regge più quando Spento non ne ha: diventa «Bilanciato non tiene false Frasi». Non è mai stata eseguita, né prima né dopo, perché il test si ferma prima sul modello mancante; il risultato vale sia per v4 sia per v6.2, calcolato a mano sulle righe stampate.

## Conseguenze

- ONNX Runtime, `vad-rs` e `silero_vad.onnx` possono uscire da build e bundle: `ORT_LIB_LOCATION` non serve più. Lo fa il ticket 04 di alleggerire-installer; non reintrodurre ONNX Runtime senza rileggere questo ADR.
- Con v6.2 il corpus non contiene più false Frasi a Spento, quindi non mette più alla prova lo scarto del rumore della protezione: la nuova asserzione passerebbe anche con la protezione spenta. Per ritarare la Sensibilità in futuro serve un corpus con rumori che v6.2 scambia per parlato.
- Una Frase di rumore si riconosce dall'RMS dell'intera Frase, quindi il filtro penalizza la voce molto debole, e di più con candidate lunghe che includono le pause. Una correzione (RMS sui soli frame con parlato, o una soglia diversa) va misurata sul microfono muto che ha motivato il filtro.
- La riverifica è sul corpus sintetico. L'ascolto umano e i respiri restano non collaudati, come nell'ADR-0023.

## Alternative scartate

- **Tenere Silero v4 su ONNX Runtime:** costa peso e un passo di build per un solo componente.
- **Silero v4 convertito in GGUF:** il porting di transcribe-cpp 0.4.0 è Silero v6.2 (il caricatore accetta anche i `ggml-silero-v5.1.2.bin` di whisper.cpp, non v4).
- **Frame da 30 ms con uno strato che traduce le probabilità tra due griglie:** è più codice, per conservare una durata che non ha valore in sé.
- **`VadSession::run` con i segmenti di transcribe-cpp:** la pipeline segmenta da sé, con prefill, hangover, durata massima e protezione propri.
