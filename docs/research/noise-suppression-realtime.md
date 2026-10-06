# Noise suppression realtime locale

Data della verifica: 6 ottobre 2026.

## Risposta breve

Sì, è tecnicamente fattibile aggiungere un denoiser leggero locale prima del VAD e dell'ASR. Non va però presentato come una cura certa alle allucinazioni: un noise suppressor attenua il rumore, mentre l'ASR può ancora inventare testo da un falso avvio. Il problema dell'audio del loopback che rientra fisicamente nel microfono è anche un problema di **echo cancellation**: per quello serve una copia separata del segnale riprodotto come riferimento.

L'utente ha scelto **RNNoise** per la riduzione del rumore sul ramo di ciascun Ingresso in cui viene attivata. DeepFilterNet resta documentato come alternativa valutata, senza un confronto obbligatorio prima di procedere con RNNoise. Se il caso principale è il loopback che entra nel microfono dagli altoparlanti, bisogna valutare anche WebRTC Audio Processing Module con AEC, non soltanto un denoiser.

## Ambito richiesto dall'utente

Scelta del modello confermata dall'utente il 6 ottobre 2026: «scelgo io, va benissimo RNNoise».

Nella discussione del 6 ottobre 2026 l'utente ha richiesto la riduzione del rumore **anche sull'audio salvato e riprodotto**, oltre che sul segnale destinato a Silero e all'ASR. La proposta iniziale di filtrare soltanto una copia per l'ASR è quindi superata. Ha scelto di conservare nel Tape soltanto il risultato ripulito quando il filtro è attivo, senza una seconda copia originale, e di lasciare la riduzione del rumore **spenta per default e attivabile per Ingresso**. Il filtro resta distinto dai quattro livelli di sensibilità del parlato già discussi. Questa nota documenta la fattibilità; nessun filtro è stato implementato o misurato nel prodotto.

## Candidati verificati

| Candidato | Cosa offre | Integrazione nel progetto | Rischi e limiti |
| --- | --- | --- | --- |
| **RNNoise 0.2** (`904a876dce1f9ab8860c0a5000ed151f9f6eef58`) | Rete ricorrente per la soppressione del rumore; API C a frame; il repository include anche un modello “little” più piccolo e sparso. | Libreria C con API esportata per Windows. `rnnoise_process_frame` riceve un frame e restituisce anche una probabilità VAD. | L'API ufficiale è fissata a 480 campioni e il README documenta 48 kHz: 10 ms per frame. Nel percorso attuale, che porta l'audio all'ASR a 16 kHz, occorre una conversione 48 kHz → 16 kHz oppure un ramo dedicato. Non è AEC e può attenuare voci o audio di sistema utili. La libreria è BSD-3-Clause; per ridistribuire i pesi del modello va verificata separatamente la licenza, perché il repository ha una richiesta aperta su questo punto. |
| **DeepFilterNet 0.5.6** (`978576aa8400552a4ce9730838c635aa30db5e61`) | Speech enhancement full-band; la versione Rust `libDF` esegue STFT e inferenza dei modelli DeepFilterNet3, con parametri per limite di attenuazione e post-filtro. | Il framework dichiara supporto a Windows e contiene codice Rust riutilizzabile; `libDF` produce `rlib`, `staticlib` e `cdylib`. | La documentazione supporta file a 48 kHz. Il default `DFState` usa FFT 960 e hop 480, cioè 10 ms, ma il ritardo complessivo dipende da finestra, lookahead e configurazione del modello. È più complesso e pesante da distribuire di RNNoise. I paper riportano RTF 0,19 su un notebook single-thread per DeepFilterNet e 0,04 per DeepFilterNet2: non sono misure della macchina dell'utente. Licenza duale MIT/Apache-2.0 per il codice. |
| **WebRTC Audio Processing Module** | Moduli distinti di Noise Suppression, Echo Cancellation e AGC; API standalone frame-by-frame. | C++/binding da integrare nella pipeline Windows. Richiede di passare il segnale di cattura e, per AEC, il segnale render/playback nella direzione inversa. | È la strada più pertinente quando il problema è la voce degli altoparlanti che rientra nel microfono, ma la più costosa da integrare. Se Microfono e loopback sono già miscelati prima del modulo, il riferimento separato non è più disponibile per una cancellazione affidabile. Licenza e modalità di distribuzione vanno verificate sulla revisione WebRTC che si decide di incorporare. |

Fonti primarie: [RNNoise 0.2](https://github.com/xiph/rnnoise/releases/tag/v0.2), [API `rnnoise.h`](https://github.com/xiph/rnnoise/blob/main/include/rnnoise.h?plain=1), [dimensioni dei frame](https://github.com/xiph/rnnoise/blob/main/src/denoise.h?plain=1), [licenza RNNoise](https://github.com/xiph/rnnoise/blob/main/COPYING?plain=1), [questione sui pesi](https://github.com/xiph/rnnoise/issues/284), [paper RNNoise](https://arxiv.org/abs/1709.08243); [DeepFilterNet 0.5.6](https://github.com/Rikorose/DeepFilterNet/releases/tag/v0.5.6), [README e supporto Windows/48 kHz](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/README.md?plain=1), [crate Rust `libDF`](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/Cargo.toml?plain=1), [stato STFT e default 48 kHz/960/480](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/lib.rs?plain=1), [paper DeepFilterNet](https://arxiv.org/abs/2305.08227), [paper DeepFilterNet2](https://arxiv.org/abs/2205.05474); [WebRTC APM](https://webrtc.googlesource.com/src/+/refs/heads/main/api/audio/audio_processing.h) e [guida APM](https://webrtc.googlesource.com/src/+/refs/heads/main/modules/audio_processing/g3doc/audio_processing_module.md).

## Adozione verificata di RNNoise

Il dubbio che RNNoise sia “pochissimo utilizzato” va qualificato. La documentazione ufficiale di OBS presenta RNNoise come metodo del filtro Noise Suppression su Windows, macOS e Linux; il codice ufficiale di OBS lo seleziona inoltre come metodo predefinito quando la build include RNNoise. È una prova di integrazione reale in un'app desktop multipiattaforma, non una misura del numero di utenti né una garanzia di qualità per Memotape. Non dimostra neppure che OBS usi esattamente RNNoise 0.2 o gli stessi pesi che valuteremmo qui.

Discord non va usato come prova a favore di RNNoise: la sua FAQ ufficiale attribuisce il filtro di soppressione a **Krisp**, quindi non è corretto contarlo come adozione di RNNoise.

Fonti: [filtro Noise Suppression di OBS](https://obsproject.com/kb/noise-suppression-filter), [implementazione e default di OBS](https://github.com/obsproject/obs-studio/blob/master/plugins/obs-filters/noise-suppress-filter.c), [FAQ ufficiale Discord/Krisp](https://support.discord.com/hc/en-us/articles/360040843952-Krisp-FAQ).

## Cosa risolve davvero il caso Microfono + loopback

La soppressione del rumore guarda principalmente il segnale del microfono e prova a ridurre componenti non vocali. Non sa, da sola, che una voce proveniente dagli altoparlanti è “eco” da eliminare. L'APM di WebRTC descrive due flussi: `ProcessStream` per la cattura vicina e `ProcessReverseStream` per il flusso lontano/render, che costituisce il riferimento dell'echo canceller. Se il microfono e l'audio di sistema vengono prima sommati in un unico buffer, si perde proprio la separazione necessaria.

Per Memotape, con l'ambito richiesto, il percorso da valutare sarebbe quindi:

1. catturare separatamente Microfono e Audio di sistema;
2. applicare il denoiser a ogni Ingresso per cui l'utente lo attiva, prima di downmix/ricampionamento verso i 16 kHz del VAD e dell'ASR;
3. usare l'Ingresso ripulito per la sua traccia salvata, per il mix e per la Trascrizione, compensando il ritardo del filtro per mantenere allineati Ingressi, timer e tempi del testo; nel Tape non si salva una seconda copia originale;
4. valutare AEC usando l'Audio di sistema come riferimento, prima di sommare i rami, se il problema osservato è il rientro acustico;
5. lasciare Silero e la protezione contro i falsi avvii come seconda barriera.

Per i file con un solo mix già pronto, non è possibile ricostruire in modo affidabile quale parte sia il microfono e quale il playback. In quel caso un denoiser può migliorare il rumore, ma non promette la cancellazione dell'eco.

## Proposta di prova

Preparerei prima un benchmark locale con lo stesso frame path della Registrazione e quattro condizioni: silenzio con respiri, rumore continuo, voce bassa e voce del loopback con microfono che non riceve parlato diretto. Misurerei almeno false Frasi, parole perse, qualità dell'ascolto, tempo CPU, memoria e ritardo. La prova deve conservare l'originale per il confronto; questo non decide il formato di salvataggio finale del Tape.

Come ordine di valutazione:

- **RNNoise “little”** come baseline leggera, perché il frame path e l'API sono semplici e il costo di integrazione è contenuto;
- **DeepFilterNet3 low-latency** se RNNoise attenua poco il rumore reale o peggiora la voce, misurando il ritardo sul PC di riferimento;
- **WebRTC APM/AEC** se le prove confermano che le false attivazioni arrivano dal rientro del loopback nel microfono.

Nessuna delle fonti garantisce l'eliminazione di allucinazioni, respiri o parole inglesi spurie con Whisper/Parakeet. La verifica deve usare registrazioni rappresentative e mantenere separate le metriche di soppressione audio, VAD e qualità ASR.
