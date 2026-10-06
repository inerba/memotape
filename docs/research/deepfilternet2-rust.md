# DeepFilterNet2: verifica del runtime Rust su Windows

Verifica del 6 ottobre 2026. Richiesta: verificare una possibile integrazione
Rust dopo l'ascolto positivo della demo `hshr/DeepFilterNet2`. Nessuna
integrazione nell'app, modifica alle Impostazioni o scelta definitiva del modello.

## Modello ascoltato e percorso nativo

La [demo](https://huggingface.co/spaces/hshr/DeepFilterNet2/blob/main/app.py)
carica `init_df("./DeepFilterNet2", config_allow_defaults=True)` e chiama
`enhance(model, df, sample)`. Le
[dipendenze](https://huggingface.co/spaces/hshr/DeepFilterNet2/blob/main/requirements.txt)
fissano `deepfilternet==0.4.0`, Torch 1.13 e torchaudio 0.13. È un percorso
Python/PyTorch, con CUDA se disponibile. Non è una prova della velocità del
runtime Rust locale.

Il progetto ufficiale distribuisce anche
[DeepFilterNet2_onnx.tar.gz](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/models/DeepFilterNet2_onnx.tar.gz):
8 628 375 byte, tre grafi `enc.onnx`, `erb_dec.onnx`, `df_dec.onnx` e `config.ini`.
Esiste anche `DeepFilterNet2_onnx_ll.tar.gz`. Non abbiamo dimostrato l'identità
dei pesi o l'equivalenza numerica con il checkpoint ospitato dalla demo.

Il [runtime libDF](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs)
fornisce `DfParams::new`, `DfTract::new` e `DfTract::process`. Carica i grafi con
Tract e processa un hop alla volta, mantenendo lo stato tra le chiamate. Questa
strada non richiede Python, PyTorch o il servizio Hugging Face nell'app e non
usa automaticamente ONNX Runtime già presente per Silero.

In libDF 0.5.6 il modello incorporato predefinito è DFN3: per provare DFN2
bisogna caricare esplicitamente il suo archivio. La versione `deep_filter`
0.5.6 non è disponibile su crates.io; il test usa il repository ufficiale
fissato al commit `978576aa8400552a4ce9730838c635aa30db5e61` (tag v0.5.6), con
`default-features = false` e soltanto `features = ["tract"]`.

## Prova isolata

Sorgente e lockfile: `.scratch/sbobino/deepfilternet-rust-probe/`.
Target separato dal target di Memotape. Compilatore verificato:
`rustc 1.96.1`, host `x86_64-pc-windows-msvc`.

- `cargo check`: riuscito senza patch di libDF o Tract.
- Il lockfile della prova risolve Tract 0.19.16 e ndarray 0.15.6.
- Artefatto scaricato dal commit fissato, SHA-256:
  `412e81d8d1779b8f925ac4b53472655a9f866c4af73dbf930ab587359f631628`.
- Con libDF 0.5.6 / Tract 0.19.16 il caricamento DFN2 fallisce nel decoder
  ERB: panic `not yet implemented` in `tract-hir/.../reshape.rs`, prima
  dell'inferenza.
- Con libDF 0.4.0 / Tract 0.18.5 si riproduce lo stesso errore.
- Il runtime 0.3.1 legge `emb_hidden_dim` dalla configurazione dei decoder;
  0.4.0 e 0.5.6 ricavano invece la dimensione da `conv_ch * nb_erb / 4`.
  È una differenza rilevante per DFN2, il cui file di configurazione dichiara
  `emb_hidden_dim = 256`, mentre la seconda formula restituisce 512.
  [Fonte 0.3.1](https://github.com/Rikorose/DeepFilterNet/blob/v0.3.1/libDF/src/tract.rs)
  e [fonte 0.5.6](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs).
- È stata preparata una copia sperimentale 0.4.0 con due modifiche alle
  dimensioni; non è stata validata. L'esperimento è stato interrotto quando
  l'utente ha chiesto di passare a DFN3. Questa copia non viene usata dalla
  prova finale né dall'app.

La configurazione finale della cartella di prova usa libDF 0.5.6 originale e
il modello DFN3: vedere `deepfilternet3-rust.md`. Questa nota conserva gli
esiti DFN2, non è una proposta di adottare il vecchio runtime nell'app.

Il test era predisposto per generare una sinusoide e silenzio e verificare
output finito e una modifica del limite di attenuazione, ma con DFN2 si è
fermato al caricamento. Non misura qualità, conservazione della voce,
riduzione delle allucinazioni o prestazioni realtime. La compilazione isolata
non sostituisce la futura verifica del grafo completo di dipendenze dell'app.

## Implicazioni per Memotape

Il modello ufficiale DFN2 ha frequenza 48 kHz e hop di 480 campioni (10 ms).
Questo è diverso dai frame ASR/Silero attuali di 30 ms a 16 kHz. Servono buffer
e ricampionamento; 10 ms di hop non sono il ritardo totale. FFT e lookahead
introducono un ritardo da compensare mantenendo durata, timestamp, tracce
separate e mix allineati. Non basta tagliare il ritardo iniziale dal file.

Nel codice attuale:

- `audio_toolkit::mixer::Mixer` mantiene gli Ingressi separati prima della
  somma e fornisce il mix e le tracce;
- `engine::live::LiveFeed::push` riceve audio già destinato al salvataggio e
  lo porta a mono/16 kHz: filtrare soltanto qui lascerebbe l'Ogg originale;
- `engine::pipeline::FileFrames` e `audio_toolkit::ogg_opus::OggCopy` hanno
  oggi rami distinti per ASR e copia del file: la futura pulizia dovrebbe
  alimentare entrambi dallo stesso audio elaborato.

È quindi plausibile un componente di pulizia per Ingresso prima della somma,
eseguito fuori dalla callback WASAPI. Deve mantenere stato, gestire i buchi di
silenzio, Pausa/Stop e attivazione al volo, svuotando anche la coda finale.
Questa è una proposta architetturale, non un comportamento già implementato.

Per un Tape già ripulito bisognerà evitare una seconda pulizia involontaria
durante la ritrascrizione; politica e metadati restano da progettare.

Il limite di attenuazione e il post-filter sono controlli della pulizia. I
quattro livelli concordati sono invece sensibilità della protezione del
parlato e restano distinti. Spento mantiene Silero attuale. La riduzione del
rumore resta spenta per default e attivabile per Ingresso; quando attiva,
alimenta anche audio salvato e player, conservando solo l'audio ripulito.

Prima di scegliere DFN2 o DFN3 occorre confrontare gli stessi audio reali. La
verifica successiva deve misurare costo con due Ingressi e ASR in funzione,
voce bassa e risposte brevi, durata/sincronizzazione e false Frasi. Un buon
risultato d'ascolto non dimostra da solo che l'ASR smetta di allucinare.
