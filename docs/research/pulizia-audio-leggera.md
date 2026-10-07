# Pulizia audio leggera per la Registrazione

Data della verifica: 7 ottobre 2026.

Questa nota risponde alla domanda se DeepFilterNet3 sia troppo costoso per la
Registrazione dal vivo e se l'alternativa già valutata, RNNoise, abbia un
percorso più leggero su Windows. Non sostituisce ancora il filtro e non
trasferisce numeri di benchmark da un computer all'altro.

## Verifica della documentazione

Sono stati eseguiti questi comandi Context7, senza errori di quota:

- `npx ctx7@latest library "RNNoise" ...` ha restituito `/xiph/rnnoise` e
  `/jneem/nnnoiseless` come fonti pertinenti;
- `npx ctx7@latest docs /xiph/rnnoise ...` ha confermato il frame da 480
  campioni e il percorso di build/ottimizzazione CPU;
- `npx ctx7@latest docs /jneem/nnnoiseless ...` ha confermato
  `DenoiseState::process_frame`, il frame da 480 campioni e il formato PCM.

Le conclusioni sulle versioni e sulle licenze sotto citano poi i file raw dei
repository upstream, così non dipendono dalla numerazione delle righe nelle
pagine GitHub.

## Esito

DeepFilterNet3 non è soltanto lento per un dettaglio della coda audio: il suo
percorso usa STFT/ISTFT, stato spettrale e inferenza Tract su tre grafi ONNX.
Nel percorso Memotape ogni hop crea inoltre un vettore di spettro e copia lo
spettro nelle code e nel risultato (`Hop::process`). Quindi ci sono due
domande separate: si possono eliminare costi evitabili dell'adattatore locale,
ma anche dopo questa ottimizzazione DFN3 resta un denoiser molto più ampio di
RNNoise.

La candidata più credibile per una Registrazione che deve stare al passo è
RNNoise, in particolare il modello ufficiale “little”. Il README di RNNoise
lo descrive come un'alternativa ridotta e più sparsa; precisa anche che i due
blob binari hanno la stessa dimensione, salvo la maggiore sparsità del little.
La stessa fonte raccomanda AVX2 oppure il rilevamento runtime SSE4.1/AVX2. Questo rende
RNNoise una scelta concreta da misurare, non una garanzia di qualità o di costo
sul PC dell'utente.

Fonti: [percorso locale `Hop::process`](../../src-tauri/src/audio_toolkit/deepfilter/hop.rs),
[runtime Tract di libDF 0.5.6](https://raw.githubusercontent.com/Rikorose/DeepFilterNet/v0.5.6/libDF/src/tract.rs),
[README RNNoise v0.2: ottimizzazioni e modello little](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/README).

## Confronto tecnico

| Candidato | Percorso realtime | Cosa è verificato | Punto da misurare in Memotape |
| --- | --- | --- | --- |
| DeepFilterNet3/libDF 0.5.6 | 48 kHz, hop di 480 campioni; STFT e stato del filtro, poi inferenza Tract | La versione Rust e il modello DFN3 sono già caricabili su Windows nel progetto; il modello/configurazione impiegati sono quelli fissati nel repository | Tempo per secondo di audio con uno e due Ingressi, allocazioni per hop e coda di blocchi mentre ASR è attivo |
| RNNoise C v0.2, modello standard | `rnnoise_process_frame` su 480 campioni; rete ricorrente con convoluzioni, GRU e uscite per guadagni/VAD | API frame-based, input documentato mono PCM a 48 kHz; il codice seleziona l'architettura CPU | Binding/build MSVC, ricampionamento e qualità su voce debole/rumore |
| RNNoise C v0.2, modello little | Stesso contratto del modello standard | Il progetto ufficiale lo distribuisce come alternativa più piccola e più sparsa | Se l'attenuazione conserva parole brevi e parlato basso abbastanza per l'ASR |
| `nnnoiseless` 0.5.2 | Port Rust di RNNoise, `DenoiseState::process_frame` su 480 campioni | Crate Rust BSD-3-Clause; stato basso livello pensato per ridurre le copie; formato d'esempio 48 kHz | Costo reale del percorso Rust/FFT su Windows e identità dei pesi scelti |

Fonti: [API RNNoise](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/include/rnnoise.h),
[contratto frame e frequenza nel README RNNoise](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/README),
[struttura della rete RNNoise v0.2](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/src/rnn.c),
[manifest di `nnnoiseless` 0.5.2](https://raw.githubusercontent.com/jneem/nnnoiseless/v0.5.2/Cargo.toml),
[`DenoiseState` e contratto 48 kHz](https://raw.githubusercontent.com/jneem/nnnoiseless/v0.5.2/src/denoise.rs).

## Versioni e licenze

Il README di `nnnoiseless` dice che è un port Rust di RNNoise, ma non dichiara
la versione upstream. Dal codice si può fare una distinzione verificabile: il
crate usa 22 bande e i livelli `input_dense`, `vad_gru`, `noise_gru` e
`denoise_gru`; RNNoise v0.1 usa la stessa struttura e lo stesso frame da 480
campioni. RNNoise v0.2 usa invece 32 bande, due convoluzioni e tre GRU. La
conclusione corretta è quindi: `nnnoiseless` 0.5.2 è strutturalmente della
generazione RNNoise v0.1, non un port di RNNoise v0.2. È un confronto di
architettura, non una prova che i pesi o l'output siano bit per bit identici.

Il manifest di `nnnoiseless` 0.5.2 dichiara `BSD-3-Clause`; il suo `COPYING`
contiene il testo BSD a tre clausole con i copyright di Joe Neeman, Mozilla,
Jean-Marc Valin, Xiph.Org e Mark Borgerding. Anche i `COPYING` ufficiali di
RNNoise v0.1 e v0.2 contengono quel testo BSD a tre clausole. Questo identifica
la licenza del codice/crate; un modello o un blob di pesi scelto separatamente
va comunque verificato come artefatto distinto.

Fonti: [RNNoise v0.1, struttura RNN](https://raw.githubusercontent.com/xiph/rnnoise/v0.1/src/rnn.c),
[RNNoise v0.1, costanti del denoiser](https://raw.githubusercontent.com/xiph/rnnoise/v0.1/src/denoise.c),
[RNNoise v0.2, costanti e rete diversa](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/src/denoise.h),
[manifest `nnnoiseless` 0.5.2](https://raw.githubusercontent.com/jneem/nnnoiseless/v0.5.2/Cargo.toml),
[COPYING `nnnoiseless` 0.5.2](https://raw.githubusercontent.com/jneem/nnnoiseless/v0.5.2/COPYING),
[COPYING RNNoise v0.1](https://raw.githubusercontent.com/xiph/rnnoise/v0.1/COPYING),
[COPYING RNNoise v0.2](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/COPYING).

## Windows e integrazione

RNNoise ufficiale espone macro di esportazione per `WIN32`, ma il README
upstream documenta soprattutto la compilazione autotools (`autogen.sh`,
`configure`, `make`), non un progetto MSVC pronto da aggiungere a Memotape.
Il costo di integrazione quindi è soprattutto il wrapper C/build e il ramo di
ricampionamento: l'API richiede 48 kHz mentre la Trascrizione lavora a 16 kHz.

`nnnoiseless` evita il wrapper C ed è una dipendenza Rust ordinaria. La sua API
bassa alloca lo stato e accetta buffer già preparati; il modello incorporato
usa pesi `i8` e la libreria dichiara di usare solo due punti `unsafe` per
ridurre il costo delle FFT. La documentazione non pubblica però un RTF su
Windows e il crate non equivale automaticamente ai pesi o ai risultati audio
del binario C ufficiale.

Fonti: [header RNNoise per WIN32](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/include/rnnoise.h),
[istruzioni di build RNNoise](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/README),
[README di `nnnoiseless`](https://raw.githubusercontent.com/jneem/nnnoiseless/v0.5.2/README.md),
[pesi e livelli della rete Rust](https://raw.githubusercontent.com/jneem/nnnoiseless/v0.5.2/src/rnn.rs).

In entrambi i casi servono uno stato indipendente per Ingresso e una gestione
esplicita del ritardo. Con Microfono e Audio di sistema attivi, il lavoro del
denoiser raddoppia; non bisogna confondere il denoising con la cancellazione
dell'eco. RNNoise, come DFN3, non riceve automaticamente il riferimento del
segnale riprodotto necessario a un AEC.

## Raccomandazione

Conviene conservare due risultati distinti nella diagnosi:

1. mantenere la correzione concreta già applicata nel [profilo dev di
   `src-tauri/Cargo.toml`](../../src-tauri/Cargo.toml): `ndarray`,
   `tract-data` e `num-complex` sono ottimizzati a livello 3 insieme alle
   dipendenze DSP già ottimizzate. Il codice dell'app resta debug, ma il lavoro
   numerico non gira più interamente senza ottimizzazione;
2. preparare un confronto nativo con RNNoise little e `nnnoiseless` sullo stesso
   flusso, usando build release, uno/due Ingressi, ASR dal vivo e gli stessi
   campioni di voce e rumore.

Se DFN3 resta fuori passo dopo il primo punto, la scelta pragmatica è provare
RNNoise little come filtro realtime leggero. La decisione tra wrapper C ufficiale
e port Rust va presa dopo il benchmark: il C ha il percorso upstream e le
ottimizzazioni CPU documentate, mentre `nnnoiseless` riduce il lavoro di build
MSVC ma aggiunge una implementazione Rust diversa. Le misure da confrontare
sono tempo CPU per secondo di audio, crescita della coda, memoria, latenza e
parole perse; il solo numero RTF dei paper o la dimensione del modello non basta.

## Misure locali e correzione

Le misure qui sotto sono del progetto, non fonti upstream. Con due Ingressi
mono a 16 kHz e 8,96 s di fixture, ottimizzare anche `ndarray`, `tract-data` e
`num-complex` nel profilo dev ha portato questi risultati:

| Configurazione | Prima | Dopo |
| --- | ---: | ---: |
| Elaborazione DFN3 | 5,059 s | 2,8404 s |
| Inizializzazione | 4,4075 s | 2,5183 s |
| p99 del batch DFN da 10 ms | 76,306 ms | 31,950 ms |

Il p99 riportato non è la latenza di una singola inferenza DFN da 10 ms: il
wrapper riceve il blocco, lo ricampiona e può eseguire più hop interni a 48 kHz.
È il tempo del batch misurato dal percorso a 16 kHz, quindi resta un indicatore
del carico sulla coda, non una misura end-to-end della UI. Anche dopo la
correzione, 31,950 ms per un batch nominale da 10 ms non dimostrano ancora che
la configurazione a due Ingressi tenga il passo durante tutta la Registrazione.

Con un solo Ingresso mono a 48 kHz, il wrapper dell'app è passato da 1,7264 s a
0,8449 s sulla stessa fixture; sono build debug dell'app con le librerie
ottimizzate. Questo conferma che la correzione riduce il costo, ma non rende
DFN3 automaticamente leggero.

Il confronto isolato con `nnnoiseless` 0.5.2 è molto più rapido, ma non è ancora
la stessa pipeline: harness separato completamente ottimizzato, nessun
ricampionamento, nessun worker WASAPI e nessun ASR. Sulla stessa fixture mono a
48 kHz da 8,96 s ha misurato:

| Stati `DenoiseState` | Elaborazione | Inizializzazione |
| ---: | ---: | ---: |
| 1 | 0,0397 s | 0,000077 s |
| 2 | 0,0821 s | 0,000025 s |
| 4 | 0,1651 s | non rilevante per il confronto |

Sul campione utente mono a 48 kHz da 22,097625 s, `nnnoiseless` ha impiegato
0,113 s con uno stato e 0,2202 s con due. Il risultato è stato scritto in
[`rnnoise-campione.wav`](../../.scratch/diagnosi-registrazione/rnnoise-campione.wav):
stato fresco, nessuna normalizzazione, fade/ritardo iniziale non compensato,
durata originale conservata e padding zero finale trimmato. Questi tempi
dimostrano che il candidato merita una prova nello stesso percorso, ma non
costituiscono ancora un rapporto di produzione confrontabile con DFN3.

La conclusione operativa è breve: l'ottimizzazione del profilo dev era
necessaria e va mantenuta, ma il p99 a due Ingressi resta alto. RNNoise, e in
particolare il percorso Rust `nnnoiseless` della generazione v0.1, è un
candidato concretamente più leggero; prima di sostituire DFN3 bisogna misurarlo
con lo stesso worker, ricampionamento, cattura, ASR e salvataggio.

Fonti per il criterio di misura: [paper RNNoise](https://arxiv.org/abs/1709.08243),
[paper DeepFilterNet3](https://arxiv.org/abs/2305.08227),
[README RNNoise v0.2, modello little](https://raw.githubusercontent.com/xiph/rnnoise/v0.2/README).

