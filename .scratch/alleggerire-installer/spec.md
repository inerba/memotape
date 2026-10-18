# Alleggerire l'installer: Silero nativo, niente ONNX Runtime

Status: ready-for-agent

Data: 18 ottobre 2026. Decisioni concordate in una sessione di grilling dopo l'inventario dell'installer 0.5.3. La prova d'ascolto sulle alternative a DeepFilterNet3 è un binario separato e non blocca questa spec.

## Problem Statement

L'installer di Memotape 0.5.3 pesa 29,6 MB e 126,5 MB una volta installato. Una parte non porta valore all'utente: ONNX Runtime (14,1 MB installati, circa 3,7 MB nell'installer, più 325 KB di note di terze parti) serve soltanto a Silero VAD v4, che transcribe-cpp, già nel bundle, sa ormai eseguire da solo. In più ONNX Runtime obbliga chi compila a scaricare a mano uno zip fuori dal repo e a impostare `ORT_LIB_LOCATION`. Infine l'exe è compilato senza LTO.

## Solution

Silero passa a Silero VAD v6.2 eseguito da transcribe-cpp (ruolo VAD, `VadSession`). ONNX Runtime, `vad-rs` e il modello `silero_vad.onnx` escono dal bundle e dalla build. Al loro posto entra il GGUF di Silero v6.2, incluso nell'installer come risorsa e verificato prima del caricamento come il modello DFN3. Per l'utente non cambia niente: le Frasi si trovano come prima, i livelli di Sensibilità restano gli stessi, l'app funziona offline senza download aggiuntivi. Dopo, se misurato utile, il profilo release attiva LTO.

## User Stories

1. Come utente, voglio scaricare un installer più leggero, così l'installazione e gli aggiornamenti sono più rapidi.
2. Come utente, voglio che l'app installata occupi meno spazio su disco, così non pago 15 MB per un componente che serve a una cosa sola.
3. Come utente, voglio che una Trascrizione di un file trovi le stesse Frasi di prima, così il cambio di VAD non peggiora i miei Tape.
4. Come utente, voglio che la Trascrizione dal vivo spezzi le Frasi come prima, così i Parziali e le Frasi concluse arrivano con gli stessi tempi.
5. Come utente, voglio che Sensibile, Bilanciato, Selettivo e Spento si comportino come prima sul rumore, sul silenzio, sui respiri e sulla voce debole, così le mie scelte di Sensibilità restano valide.
6. Come utente, voglio che le risposte brevi («sì», «no», «grazie») restino Frasi, così il cambio non ne perde.
7. Come utente, voglio che la pulizia dell'audio (DFN3) continui a precedere il VAD come prima, così l'audio ripulito alimenta ancora Silero.
8. Come utente con due Ingressi, voglio che Microfono e Audio di sistema restino trascritti in parallelo, così il cambio di VAD non rallenta le Registrazioni con gli Ingressi separati.
9. Come utente, voglio che il VAD funzioni senza rete e senza scaricare nulla, così l'app resta usabile offline dal primo avvio.
10. Come utente, voglio un errore chiaro se il modello del VAD manca o è corrotto, così so che l'installazione va riparata.
11. Come utente su un PC senza GPU o con una CPU vecchia, voglio che il VAD giri comunque, così la Trascrizione non dipende dalla scheda video.
12. Come utente, voglio che i crediti e le licenze in Informazioni riflettano i componenti davvero distribuiti, così le attribuzioni sono corrette.
13. Come utente, voglio che i Tape già trascritti restino identici e leggibili, così il cambio non tocca il mio archivio.
14. Come utente, voglio che Trascrivi di nuovo su un vecchio Tape usi il nuovo VAD senza errori, così posso rifare una Trascrizione quando voglio.
15. Come utente, voglio che Annulla fermi la Trascrizione con la stessa prontezza di prima, così il nuovo VAD non la blocca.
16. Come sviluppatore, voglio compilare senza scaricare ONNX Runtime e senza `ORT_LIB_LOCATION`, così preparare una macchina o un worktree richiede un passo in meno.
17. Come sviluppatore, voglio che `bun run worktree:setup` non chieda più le variabili ORT, così un worktree nuovo compila con la configurazione condivisa.
18. Come sviluppatore, voglio che la build tolga da `runtime-libs/` le DLL che non servono più, così un `onnxruntime.dll` di una build vecchia non finisce nell'installer.
19. Come sviluppatore, voglio un ADR che spieghi perché ONNX Runtime non c'è più e come è stata riverificata la Sensibilità, così un agente futuro non lo reintroduce senza saperlo.
20. Come sviluppatore, voglio che i test del corpus di protezione girino con il nuovo VAD, così la taratura resta riproducibile.
21. Come sviluppatore, voglio che i frame della pipeline abbiano la stessa durata dei frame del modello, così non serve uno strato che traduca le probabilità tra due griglie.
22. Come sviluppatore, voglio che le durate del segmentatore (prefill, onset, hangover, durata massima) restino espresse in millisecondi, così il cambio di frame non cambia il loro significato.
23. Come sviluppatore, voglio sapere quanto LTO riduce l'exe prima di accettare build release più lente, così la scelta si basa su una misura.
24. Come sviluppatore, voglio che la documentazione di sviluppo (prerequisiti, insidie audio, installer, verifica manuale) descriva la nuova situazione, così chi legge non cerca ONNX Runtime.

## Implementation Decisions

### VAD

- `VoiceDetector` resta il seam: `probability(frame) -> f32` e `reset()`. Cambia l'implementazione, non il trait. L'implementazione nuova sta sopra `transcribe_cpp::VadSession` e usa `stream_feed` con un frame alla volta e `stream_reset` per `reset()`.
- Modello: `handy-computer/silero-vad-v6.2-gguf`, file `silero-vad-v6.2-F32.gguf`, 1 241 056 byte, SHA-256 `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`, licenza MIT. Va fissata la revisione del repo Hugging Face. Il file è una risorsa del bundle al posto di `silero_vad.onnx` e si verifica prima del caricamento, con un errore distinto per assenza e incompatibilità, come il modello DFN3.
- Backend CPU (la scelta `AUTO` di transcribe-cpp per il VAD): la GPU è meno fedele al riferimento.
- Un modello Silero caricato per pipeline, come oggi un `Silero::new` per pipeline. Transcribe-cpp consente un solo calcolo alla volta per modello: un modello condiviso serializzerebbe due Ingressi in parallelo.
- Il VAD è un modello diverso da quello di Trascrizione: nessun conflitto con lo stream ASR (`Busy` riguarda solo lo stesso modello). I backend sono già inizializzati per l'ASR prima del caricamento.

### Frame

- Il frame della pipeline passa da 480 a 512 campioni a 16 kHz (da 30 a 32 ms), la dimensione di Silero v6.2. Una costante sola per campioni e una per millisecondi; ogni `480` e ogni «30 ms» scritto a mano nella pipeline, nella protezione, nel dal vivo e nel livello deriva da lì.
- Prefill, onset, hangover e durata massima delle Frasi restano in millisecondi; i frame si ricavano per eccesso come oggi.
- I tempi delle Frasi nei Tape restano millisecondi sulla linea del tempo della Sorgente: cambia solo la granularità con cui nascono, non il formato. I Tape esistenti non si toccano.
- La pulizia DFN3 lavora a 48 kHz con un hop proprio e non dipende dal frame del VAD.

### Sensibilità

- Si fanno girare i due test ignorati del corpus di protezione (misura dell'evidenza e corpus a quattro livelli con tre ASR) prima e dopo il cambio, sullo stesso corpus del ticket 05 della pulizia audio.
- Se Spento, Sensibile, Bilanciato e Selettivo danno gli stessi esiti (Frasi tenute e scartate, risposte brevi e voce debole conservate), le soglie restano. Altrimenti si ritarano sullo stesso corpus con lo stesso criterio. L'interfaccia della Sensibilità non cambia.
- Anche le soglie del segmentatore (0,4) e del filtro sugli zeri seguono lo stesso criterio.

### Build e bundle

- Escono `vad-rs` (e con lui `ort`), il copiaggio di `onnxruntime.dll` e la variabile obbligatoria `ORT_LIB_LOCATION` dallo script di build; la pulizia delle DLL vecchie in `runtime-libs/` le toglie da sola. Le DLL del runtime VC++ restano: servono a transcribe e ggml.
- Escono le licenze di ONNX Runtime (comprese le note di terze parti) e di `vad-rs`; quella di Silero passa a v6.2 (MIT) con l'attribuzione del GGUF. I crediti in Informazioni seguono, e il ruolo «onnx» sparisce se non resta nessun componente che lo usa.
- Il modello DFN3 resta nell'installer (`PRODUCT.md`: pulizia senza download aggiuntivi).

### LTO

- Dopo il cambio di VAD si misura l'exe release con `lto = true` e `codegen-units = 1` contro il profilo di oggi. Si adotta solo se l'exe cala di almeno il 10%; nessun `opt-level = "s"` (rallenterebbe il DSP), nessun `panic = "abort"`, nessuno `strip` (su MSVC i simboli sono già nel `.pdb`).

### Documentazione

- Un ADR nuovo: Silero v6.2 su transcribe-cpp al posto di v4 su ONNX Runtime, motivo (dimensione, build), modello e hash, esito della riverifica della Sensibilità.
- Aggiornare `AGENTS.md` (prerequisiti, struttura), `README.md`, `PRODUCT.md` dove nomina ONNX Runtime o Silero v4, e i documenti di sviluppo: prerequisiti, insidie audio, installer (contenuto, verifica dei moduli caricati), verifica manuale. Le ricerche in `docs/research/` restano storiche.

## Testing Decisions

- Un buon test guarda il comportamento esterno: probabilità alte sul parlato e basse sul silenzio, Frasi trovate, esiti della protezione. Non guarda quante chiamate fa l'adattatore né lo stato interno di transcribe-cpp.
- Seam 1, `VoiceDetector`: i test della pipeline, del segmentatore e della protezione con il detector finto restano lì; cambiano solo le dimensioni dei frame. Precedente: i test esistenti in `engine/pipeline` e `audio_toolkit/segmenter`.
- Seam 2, l'adattatore Silero con il modello vero: parlato della fixture `parlato-it.wav` sopra soglia, silenzio sotto, `reset` che riporta lo stato iniziale (stessa sequenza, stesse probabilità), errore distinto con modello assente o non valido. Precedente: i test di caricamento e verifica del modello DFN3.
- Seam 3, il corpus di protezione (test ignorati, release, modelli veri): eseguiti prima e dopo, con i risultati riportati nell'ADR. Precedente: `corpus_protezione_quattro_livelli_tre_asr` e `misura_evidenza_corpus_protezione`.
- I test che oggi caricano `silero_vad.onnx` (pulizia, diarizzazione, dal vivo nativo, pipeline con modelli scaricati) passano al GGUF senza cambiare cosa verificano.
- Verifica dell'installer come in `docs/sviluppo/installer.md`: nessun `onnxruntime.dll` nell'installer né tra i moduli caricati, una Trascrizione della fixture e una Registrazione di prova riuscite. Dimensione dell'installer e dell'installato riportate nel ticket.

## Out of Scope

- Sostituire DeepFilterNet3 o il suo runtime (tract): binario separato, con una prova d'ascolto alla cieca contro RNNoise 0.1 (`nnnoiseless`, come in OBS) e RNNoise 0.2.
- Scaricare il modello DFN3 al primo uso.
- Togliere i sottoinsiemi cirillico, greco e vietnamita dei font.
- Togliere varianti `ggml-cpu-*` o il backend Vulkan.
- Un traguardo di dimensione dell'installer.
- Silero sulla GPU, Silero a 8 kHz, i segmenti offline `VadSession::run` (la pipeline continua a segmentare da sé).

## Further Notes

- Inventario di partenza (0.5.3), peso compresso stimato con xz per file: exe circa 10,3 MB (tract e `df` circa 12 MiB dei 30,5 MiB di codice), modello DFN3 8,0 MB, `ggml-vulkan.dll` 4,9 MB, `onnxruntime.dll` 3,7 MB, `silero_vad.onnx` 0,7 MB, nove `ggml-cpu-*` 0,8 MB insieme.
- Documentazione di transcribe-cpp v0.4.0 sul VAD: `docs/vad.md`, `docs/models/silero-vad-v6.2.md`, `docs/porting/families/silero_vad.md` nel repo `handy-computer/transcribe.cpp` al tag `v0.4.0`.
