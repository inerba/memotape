# Confronto ipotetico: RNNoise 0.2 e DeepFilterNet3

Data della verifica: 6 ottobre 2026.

## Esito

La scelta tra i due candidati resta aperta. Le fonti non sostengono che
DeepFilterNet3 sia automaticamente migliore di RNNoise per l'ASR, né che
RNNoise 0.2 sia la stessa cosa dei port o wrapper che riportano implementazioni
precedenti. I paper di DeepFilterNet non mettono RNNoise in tabella: confrontano
le versioni della famiglia DeepFilterNet e, nel lavoro del 2022, PercepNet e
DCCRN+. Non trasferiamo quindi i loro punteggi percettivi o il loro WER
(che non viene misurato in quei risultati) al caso Memotape.

Il requisito di prodotto già concordato resta indipendente dal candidato:

- riduzione del rumore spenta per default e attivabile per ciascun Ingresso;
- quando attiva, il risultato ripulito alimenta Trascrizione, Silero, salvataggio
  e riproduzione;
- nel Tape si conserva solo il risultato ripulito, senza una seconda copia
  originale;
- la sensibilità del parlato resta una regolazione distinta, con quattro livelli
  indipendenti per Ingresso.

Questa è una valutazione ipotetica. Non modifica codice, modelli o il formato
del Tape e non è un benchmark sulla macchina dell'utente.

## Che cosa sono i due candidati

| Aspetto | RNNoise 0.2 | DeepFilterNet3 con libDF 0.5.6 |
| --- | --- | --- |
| Metodo | Pipeline DSP con una rete ricorrente: estrazione di caratteristiche, filtro del pitch, guadagni per banda e uscita VAD. Il codice della rete contiene due convoluzioni, tre GRU e due uscite dense, una per i guadagni e una per la probabilità VAD. | Speech enhancement a due stadi: guadagni su 32 bande ERB e filtro complesso multi-frame di ordine 5 sulle frequenze più basse. Un encoder stima anche l'SNR locale per decidere quali stadi eseguire. |
| Input/runtime | API C a frame; il ramo documentato lavora a 48 kHz. Il modello è incorporato o caricato come blob. | Runtime Rust con STFT/ISTFT e tre modelli ONNX (enc, erb_dec, df_dec) raccolti in un archivio insieme alla configurazione. Il repository dichiara supporto del framework a Windows; il demo LADSPA/GUI descritto nel README è invece Linux. |
| Controllo esposto | rnnoise_process_frame restituisce la probabilità VAD oltre all'audio ripulito, ma l'API ufficiale non espone una soglia o un limite d'attenuazione per preservare più rumore o voce. Una scala di sensibilità richiederebbe una politica Memotape sopra il risultato. | atten_lim limita l'attenuazione miscelando parte del segnale originale con quello ripulito. Il runtime espone anche soglie SNR, post-filter e riduzione della maschera; sono manopole reali, ma non garantiscono da sole la conservazione della voce bassa. |
| Possibile vantaggio | Superficie d'integrazione ridotta: C, stato per stream e frame fisso; il rilascio 0.2 include ottimizzazioni SSE4.1/AVX2 e rilevamento CPU a runtime. | Controlli più ricchi e modello full-band; il filtro complesso può sfruttare correlazioni tra frame per preservare componenti periodiche della voce. |
| Costo da verificare | Stato e modello sono concettualmente semplici, ma il peso effettivo dipende dai pesi, dalla compilazione e dall'architettura. Le fonti non danno un costo CPU/memoria universale per il nostro PC. | Il grafo ONNX, tract, STFT, buffer di spettro e stato per canale hanno una superficie maggiore; il pacchetto deve includere il runtime scelto e i pesi. La fonte non autorizza a trasformare il RTF pubblicato su un altro computer in una previsione per Memotape. |

Fonti di implementazione: [RNNoise rnn.c v0.2](https://github.com/xiph/rnnoise/blob/v0.2/src/rnn.c#L35-L47), [RNNoise denoise.h v0.2](https://github.com/xiph/rnnoise/blob/v0.2/src/denoise.h#L23-L35), [API pubblica rnnoise.h](https://github.com/xiph/rnnoise/blob/v0.2/include/rnnoise.h#L49-L86), [DeepFilterNet tract.rs v0.5.6](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs#L18-L24), [caricamento dei tre modelli e parametri runtime](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs#L25-L61), [dipendenze Rust di libDF](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/Cargo.toml#L64-L76).

## Frame, lookahead e ritardo

I numeri dei frame non sono il ritardo totale.

RNNoise dichiara FRAME_SIZE = 480 e usa una finestra FFT di 960 campioni.
Sul flusso a 48 kHz sono 10 ms di avanzamento per chiamata e 20 ms di finestra.
Il codice conserva anche spettri ritardati: non deduciamo dal solo hop il ritardo
complessivo della rete e della catena DSP. Il percorso Memotape aggiungerebbe
buffer, ricampionamento e scheduling, da misurare con la versione scelta.

DeepFilterNet3 usa finestre da 20 ms, hop da 10 ms e, nella configurazione
descritta dal paper DFN3, due frame di lookahead. Il paper riporta 40 ms di
latenza complessiva per quella configurazione. Il codice Rust calcola il
lookahead come il massimo tra quello convoluzionale e quello del filtro
profondo; quindi il valore va letto dal modello/config scelto, non dedotto dal
solo hop da 10 ms.

Fonti: [costanti e pipeline RNNoise](https://github.com/xiph/rnnoise/blob/v0.2/src/denoise.c#L311-L323), [spettro ritardato RNNoise](https://github.com/xiph/rnnoise/blob/v0.2/src/denoise.c#L430-L470), [paper DeepFilterNet3, frame e latenza](https://arxiv.org/html/2305.08227#S3.F1), [calcolo del lookahead in libDF](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs#L249-L304).

## Attenuazione e voce bassa

DeepFilterNet offre una forma di controllo dell'intensità della pulizia: il
codice converte atten_lim_db in un guadagno minimo e miscela il risultato con
lo spettro rumoroso. Il suo significato è esplicito: limite 0 dB equivale a non
fare riduzione, mentre un limite alto lascia agire maggiormente il denoiser.
La pulizia resta distinta dai quattro livelli di sensibilità del parlato già
concordati. Nessun mapping tra le due regolazioni è deciso: i parametri di
pulizia devono essere scelti su prove con voce bassa, risposte brevi e respiri.

DeepFilterNet ha inoltre una soglia min_db_thresh: sotto quella soglia il
runtime applica una maschera nulla, interpretando il frame come solo rumore.
In v0.5.6 il default del runtime Rust è -10 dB; è una protezione utile contro
un respiro, ma può cancellare una parola debole se impostata senza calibrazione.
Le soglie non sono quindi una prova che DeepFilterNet “non allucini”. Sono un
punto di controllo da testare.

RNNoise stima invece una probabilità VAD e produce guadagni per banda, ma la
sua API di elaborazione non espone un atten_lim equivalente. Per ottenere una
modalità più conservativa si dovrebbe aggiungere una miscela tra audio
originale e audio ripulito, oppure usare la probabilità VAD nella politica di
Memotape. In entrambi i casi la conservazione delle parole brevi è una
proprietà da misurare, non una garanzia del modello.

Fonti: [parametri runtime e atten_lim](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs#L107-L181), [limite di attenuazione e applicazione della miscela](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs#L364-L374) e [applicazione delle soglie SNR](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs#L596-L623), [uscite guadagni/VAD di RNNoise](https://github.com/xiph/rnnoise/blob/v0.2/src/rnn.c#L37-L47).

## Che cosa dicono davvero i paper

Il paper del 2023 riporta per la propria prova una tabella con DeepFilterNet,
DeepFilterNet2 e DeepFilterNet3. Il paper del 2022 confronta DeepFilterNet con
PercepNet e DCCRN/DCCRN+, con parametri e MACS della propria prova. RNNoise non è
una riga di quelle tabelle e non viene dichiarata una versione RNNoise né un
set di pesi RNNoise come baseline.

Il riferimento a Valin 2018 nella bibliografia DFN è un riferimento al lavoro
sull'approccio ibrido DSP/deep learning; non dimostra un confronto con i pesi
rilasciati in RNNoise 0.2, che appartiene a una release successiva. Non usiamo
quindi quei numeri per dire che DFN3 vince RNNoise 0.2.

Inoltre PESQ, STOI, SI-SDR, MOS e RTF descrivono qualità percettiva o costo
computazionale in condizioni definite dagli autori. Non sono WER di Whisper o
Parakeet. Per il problema osservato in Memotape servono prove separate su
allucinazioni da silenzio, respiri, parlato basso e voce del loopback.

Fonti: [paper DFN3, tabella e RTF](https://arxiv.org/html/2305.08227#S4.F1), [paper DFN 2022, tabella di complessità e confronto](https://arxiv.org/html/2110.05588#S3.T1), [release ufficiale RNNoise 0.2](https://github.com/xiph/rnnoise/releases/tag/v0.2).

## Windows e costo d'integrazione

RNNoise è una libreria C con simboli esportati anche per WIN32; il README
ufficiale documenta compilazione con configure/make e ottimizzazioni x86.
Per Memotape il punto da risolvere è il binding/build MSVC e il ricampionamento
verso i 48 kHz richiesti dal ramo documentato, non l'esistenza di un modello
Python o ONNX.

DeepFilterNet v0.5.6 dichiara il framework compatibile con Windows e fornisce
una strada Rust tramite libDF, ma questa strada porta dentro tract e tre
grafi ONNX più lo stato STFT. Il README del binario precompilato specifica
inoltre file a 48 kHz. Il demo grafico/LADSPA non va confuso con un'integrazione
Windows già pronta per Memotape.

Il confronto corretto dei costi è quindi per categorie: dipendenze e dimensione
del pacchetto, memoria del modello e degli stati per Ingresso, tempo CPU per
secondo di audio, ritardo misurato nel percorso reale. Non confrontiamo il
numero RTF 0,19 del paper DFN3 con un numero ipotetico di RNNoise: sono
macchine, build e misure diverse.

Fonti: [README RNNoise v0.2](https://github.com/xiph/rnnoise/blob/v0.2/README#L10-L28), [supporto WIN32 dell'header](https://github.com/xiph/rnnoise/blob/v0.2/include/rnnoise.h#L27-L43), [README DeepFilterNet v0.5.6, Windows e 48 kHz](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/README.md#L34-L71), [release DFN3 v0.5.6](https://github.com/Rikorose/DeepFilterNet/releases/tag/v0.5.6).

## Implicazione per il caso Microfono + Audio di sistema

Entrambi riducono componenti che il modello considera rumore, ma non ricevono
automaticamente un riferimento separato del
segnale riprodotto. Se la voce del loopback rientra fisicamente nel microfono,
una riduzione del rumore può non riconoscerla come eco e può anche deformarla.
Questo è un limite strutturale diverso dal falso avvio causato da un respiro.
Un test positivo con un denoiser non dimostrerebbe quindi di avere risolto
l'echo; un test negativo non dimostrerebbe che il denoiser sia inutile per il
rumore del microfono.

Per mantenere le regolazioni indipendenti, l'eventuale filtro dovrebbe avere uno
stato per Ingresso e stare prima della somma Microfono/Audio di sistema. Il ramo
ripulito alimenterebbe poi traccia, mix, VAD e ASR. Per un file già miscelato
non si possono recuperare in modo affidabile i due Ingressi originali, quindi
la regolazione per Ingresso non sarebbe applicabile a posteriori.

DeepFilterNet espone anche elaborazione multicanale (`n_ch` e `ReduceMask`), che
non equivale ad avere un riferimento per cancellare l'eco. Questa è un'inferenza
architetturale dal contratto delle API e dalla definizione dei paper x = s + z;
non è una promessa sulle
registrazioni dell'utente. L'eventuale cancellazione dell'eco richiederebbe una
catena diversa con un riferimento separato dell'Audio di sistema.

## Prossimo confronto utile

Prima di scegliere definitivamente, il test minimo dovrebbe eseguire gli stessi
frame a 48 kHz e lo stesso percorso di copia/salvataggio su entrambi i candidati,
con l'originale conservato solo nel banco di prova. Le condizioni dovrebbero
includere silenzio, respirazione, voce bassa, «sì/no/grazie», rumore continuo e
loopback rientrato nel microfono.

Le misure da raccogliere separatamente sono: falsi avvii Silero, parole perse,
WER solo come misura aggiuntiva su trascrizioni annotate, qualità d'ascolto,
CPU, memoria, dimensione dei file distribuiti e ritardo end-to-end. La scelta
può restare RNNoise se il costo semplice e il ritardo prevalgono; DeepFilterNet3
ha un caso forte se le sue attenuazioni controllabili preservano meglio la voce
nel rumore reale. Nessuna delle due conclusioni può essere sostituita dai
risultati dei paper.

### Nota sulle versioni

In questa nota RNNoise 0.2 significa il tag ufficiale xiph/rnnoise v0.2, non un
port, wrapper o modello derivato dal paper del 2018. DeepFilterNet3 significa i
pesi/configurazione della famiglia DFN3 usati con libDF v0.5.6. Prima di
un'integrazione bisogna fissare anche gli artefatti e i loro hash: il nome del
progetto da solo non identifica i pesi caricati.
