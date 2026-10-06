# DeepFilterNet3: prova nativa Rust su Windows

Verifica del 6 ottobre 2026, richiesta dall'utente dopo il confronto con
RNNoise e la prova della demo DeepFilterNet2. Il bersaglio finale è DFN3.

## Esito verificato

DeepFilterNet3 carica e processa audio nel runtime Rust ufficiale originale
su questa macchina Windows x64. Non è necessario usare Python, PyTorch,
Hugging Face o GPU nel percorso provato. Nessuna patch di libDF o Tract è
usata. Non è ancora integrato in Memotape.

| Componente | Versione o artefatto |
| --- | --- |
| Compilatore | Rust 1.96.1, host x86_64-pc-windows-msvc |
| libDF | 0.5.6, commit 978576aa8400552a4ce9730838c635aa30db5e61 |
| Tract | 0.19.16, fissato dal lockfile della prova |
| Feature di libDF | default-features=false, tract |
| Modello | DeepFilterNet3_onnx.tar.gz dal medesimo commit |
| Dimensione archivio | 7 983 136 byte; non è la memoria occupata né il peso totale del programma |
| SHA-256 osservato | c94d91f70911001c946e0fabb4aa9adc37045f45a03b56008cb0c8244cb63616 |
| Configurazione caricata | 48 kHz, hop 480 campioni, FFT 960, lookahead 2 |

Fonti: [modello ufficiale](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/models/DeepFilterNet3_onnx.tar.gz),
[API e inferenza native](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/src/tract.rs),
[feature e dipendenze](https://github.com/Rikorose/DeepFilterNet/blob/v0.5.6/libDF/Cargo.toml).
L'hash identifica l'artefatto scaricato per questa prova, non sostituisce una
verifica separata del catalogo di distribuzione futuro.

## Test eseguiti

Prova indipendente: `.scratch/sbobino/deepfilternet-rust-probe/`. Il suo
Cargo.toml e Cargo.lock sono separati da quelli di Memotape e il target è
interno alla stessa cartella. La copia sperimentale DFN2 in `vendor/` non è
referenziata dal manifest finale.

1. Compilazione della libreria e dell'eseguibile con MSVC: riuscita.
2. Caricamento e inferenza su 120 hop, sinusoide seguita da silenzio: riusciti;
   output finito. Anche il setter del limite di attenuazione è stato chiamato
   senza errori; non è un test percettivo della sua efficacia.
3. File `src-tauri/tests/fixtures/parlato-it.wav`: lettura, conversione mono,
   ricampionamento a 48 kHz con il helper di libDF, elaborazione a hop,
   compensazione del ritardo secondo la formula della CLI ufficiale,
   scrittura e rilettura WAV: riusciti.
4. Output finito e non nullo; 430 080 campioni a 48 kHz, esattamente 8,960 s.

Misura singola della sola fase di elaborazione del file: **3,175 s**, RTF
**0,354**, build **debug**, un Ingresso. Esclude caricamento del modello,
ricampionamento, scrittura, WASAPI, Ogg, Silero, ASR e Diarizzazione. Non è una
percentuale di utilizzo CPU e non dimostra il comportamento con due Ingressi
e l'ASR in esecuzione. Non abbiamo misurato memoria, installer o la variante
low latency.

L'output di prova è
`.scratch/sbobino/deepfilternet-rust-probe/parlato-it-dfn3.wav`. La fixture è
voce sintetica già pulita: il test attesta il percorso tecnico, non una
qualità di denoising superiore a RNNoise.

Comando di riproduzione del test sintetico, dalla root:

```powershell
cargo run --locked --manifest-path .scratch/sbobino/deepfilternet-rust-probe/Cargo.toml --target-dir .scratch/sbobino/deepfilternet-rust-probe/target -- .scratch/sbobino/deepfilternet-rust-probe/DeepFilterNet3_onnx.tar.gz
```

Per il test file aggiungere il percorso WAV e un percorso di output nuovo.
Il probe usa `create_new`, quindi non sovrascrive un WAV esistente.

## Confronto e possibile integrazione

Rispetto a RNNoise, DFN3 offre un percorso direttamente Rust e controlli
nativi di limite di attenuazione, post-filter e soglie SNR. Porta però anche
Tract e tre grafi ONNX; il costo totale non coincide con i circa 8 MB dei pesi.
Il controllo della pulizia è distinto dai quattro livelli di sensibilità del
parlato già concordati. Il denoiser non sostituisce Silero né prova di avere
risolto le false Frasi di Whisper o Parakeet.

La proposta è uno stato indipendente per Ingresso, a 48 kHz e prima della
somma, nel worker audio. Il medesimo audio ripulito dovrebbe alimentare Ogg,
player, Silero e ASR; nei file serve uno stadio comune prima della diramazione
tra copia e ASR. Perché questa proposta diventi comportamento di prodotto
servono gestione e verifiche di ritardo, cambio al volo, Pausa/Stop, riempimento
dei buchi e scarico finale. In particolare la scorciatoia sui frame a energia
molto bassa presente in libDF 0.5.6 va verificata rispetto alle code del filtro.

Prima dell'integrazione completa restano: prova su parlato italiano debole,
«sì/no», respiri e rumori reali; confronto ascoltato con RNNoise sugli stessi
campioni; misura con entrambi gli Ingressi e ASR; grafo di dipendenze completo
di Memotape, sincronizzazione di tracce/testo e assenza di doppia pulizia nei
Tape. Nessuna preferenza già concordata su default, audio salvato o
sensibilità viene modificata da questa prova.
