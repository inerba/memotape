# Banco locale Nemotron 3 — ticket 08

Windows x64, runtime fissato a `e6672a8672913b47f1571c66c54bee789d028416`.

Non modifica le Impostazioni, non carica audio su servizi remoti. Nessun risultato

sulla fixture sintetica dimostra qualità sull'italiano reale o su 5–8 voci.

## Misure e stress della pipeline nativa

Da root del worktree, una compilazione Rust alla volta:

```powershell

cargo test --locked --manifest-path src-tauri/Cargo.toml metriche_distinguono

./tools/nemotron3-benchmark/run.ps1 -Seconds 300

```

Lo script avvia quattro processi separati, CPU/Vulkan × uno/due Ingressi.

Ogni processo precarica/riscalda ASR Nemotron e Nemotron 3 Diarization, poi ripete

la fixture sintetica `parlato-due-voci.wav`, alimentandola a tempo reale. La Pausa

sospende per 1 s l'alimentazione a metà, senza inserire silenzio nella traccia;

`ClosePhrase` percorre la pipeline di produzione. Si usa `LowLatency` dal vivo,

`VeryHighLatency` nell'analisi finale sequenziale. Per due Ingressi le tracce sono

uguali: il mix del banco è la prima traccia, non una cattura dei due dispositivi

né un collaudo del mixer. Lo smoke WASAPI esistente prova separatamente il mixer.

Output in `.scratch/nemotron3-diarizzazione/benchmark08/<backend>-<ingressi>-<secondi>`:

- `report.json`: esiti, tempi di avvio/smaltimento/finale, misure e singole unità;

- `stdout.log`, `stderr.log`: runtime effettivo e controlli nativi;

- `process-samples.json`: CPU cumulativa, working set e private bytes ogni ~1 s;

- `whole-gpu-samples.json`: memoria/carico dell'intera GPU ogni ~3 s.

- `Banco.tape`, Ogg: prodotti di prova, mai salvati nella Libreria utente.

Rifiuta cartelle già presenti; per ripetere conserva la vecchia cartella e cambia

`-Seconds`, oppure spostala esplicitamente dopo averne verificato il percorso.

Prima del lancio verifica che il binario contenga il test richiesto; un exit code

zero senza `report.json` valido con identità e controlli di integrità corrispondenti

non passa. Ogni caso distingue inoltre `liveDiarizationHealthy`: integrità/audio/ASR
possono passare anche quando il realtime è fallito. Il runner prosegue la matrice
ma restituisce errore finale se un diarizer dal vivo non tiene il passo.
`-TestExecutable` permette di fissare esplicitamente il binario.

Ogni caso ha timeout di 1.200 s; in quel caso lo script arresta soltanto il

processo di test che ha avviato e lo indica come prova fallita.

Le unità misurate sono i Tempo del testo definitivi restituiti dall'ASR, senza

inventare timestamp. La chiave è `(inizio_ms, fine_ms, testo esatto)` per Ingresso.

Per ciascuna si osserva la prima comparsa con gli stessi tempi/testo e la prima

attribuzione determinata presente nello snapshot del core. Si conservano i casi

senza etichetta: non entrano nei quantili di attribuzione, sono contati a parte.

La prima etichetta può essere rettificata; non è un'asserzione di correttezza.

Audio→testo ed audio→etichetta partono dall'istante effettivo di consegna del frame

che contiene la fine dell'unità; testo→etichetta parte dalla prima comparsa della

stessa unità temporizzata. La griglia di consegna è 30 ms. `diarizer_frontier`

misura il ritardo dell'ultimo tempo coperto dai turni, separatamente dal testo.

Quantili nearest-rank p50/p95/p99, min/max e n; nessun percentile su zero campioni.

L'avvio/riscaldamento precede la consegna, ed è riportato separatamente.

Queste sono misure dei callback e delle proiezioni del core, senza trasporto

Tauri/WebView/rendering. Il banco trattiene mappe di osservazione e JSON proporzionali

al testo; le misure di memoria includono questo overhead. Il PCM trattenuto è solo

la fixture di 25,675 s; il PCM di cinque minuti non è accumulato. Dopo Stop il

finalizzatore rilegge gli Ogg salvati. GPU/VRAM sono campionate per l'intero

dispositivo, comprendono Windows ed altre app, e non sono memoria del solo processo.

## Confronto con una sola ASR per i due diarizer

Fornire audio locale con diritto d'uso e annotazioni indipendenti, comprensive di

silenzi/sovrapposizioni. La sorgente viene solo letta, senza invii remoti.

```powershell

$env:MEMOTAPE_NEMOTRON3_MODEL = 'D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf'

$env:MEMOTAPE_COMPARE_AUDIO = 'D:\campioni\italiano.wav'

$env:MEMOTAPE_COMPARE_OUTPUT = 'D:\campioni\risultati'

$env:MEMOTAPE_COMPARE_BACKEND = 'vulkan'

cargo test --locked --manifest-path src-tauri/Cargo.toml ticket08_confronto_stessa_asr -- --ignored --test-threads=1 --nocapture

```

Esegue la Trascrizione una volta sola, italiano, Ingresso mix; riusa PCM/testo/tempi

per Sortformer e Nemotron 3. Scrive `frozen-asr.json`, RTTM, Frasi e unità con

etichette per entrambi. Ogni campione di ogni Ingresso va provato separatamente:

non concatenare Ingressi e non confrontare ASR differenti. Questo comando è una

misura offline con `VeryHighLatency`, non il runtime realtime.

Annotare le unità ASR congelate ascoltando l'audio, senza leggere la risposta del

diarizer da valutare. Copiare la struttura di `<modello>-units.json`, sostituire

soltanto `speaker` con il nome della voce di riferimento, oppure `null` nei tratti

veramente non attribuibili. Non correggere il testo: questa misura riguarda

l'attribuzione del testo già riconosciuto, separata dagli errori ASR.

```powershell

python -m unittest discover -s tools/nemotron3-benchmark -v

python tools/nemotron3-benchmark/metrics.py reference.rttm nemotron3.rttm --recording sample --start 0 --end 120 --reference-units reference-units.json --hypothesis-units nemotron3-units.json

python tools/nemotron3-benchmark/metrics.py reference.rttm sortformer.rttm --recording sample --start 0 --end 120 --reference-units reference-units.json --hypothesis-units sortformer-units.json

```

Scorer senza dipendenze: DER ad intervalli esatti, collar zero, overlap incluso,

regione esplicita completamente annotata; mapping globale ottimale uno-a-uno

massimizza il tempo di voce comune, fino a otto voci. Le unità del testo devono

essere interamente dentro la stessa regione: quelle esterne o a cavallo del

confine sono escluse e contate in `outside_region`, senza tagli inventati. Miss/false alarm/confusion e

denominatore in speaker-secondi sono separati. L'attribuzione del testo distingue

corretto, errato, non determinato e riferimento ambiguo; unità diverse fra le due

ASR vengono rifiutate. Non calcola WER e non usa l'output del modello come groundtruth.

Registrare licenza, fonte/revisione, SHA-256 audio/annotazioni, numero reale di voci

ed eventuale modifica del campione. Nessun confronto reale risulta eseguito finché

non sono presenti audio e annotazioni validi.
## Annulla nativo e operazioni Tape senza UI

```powershell
$env:MEMOTAPE_FLOW_TAPE = (Resolve-Path '.scratch/nemotron3-diarizzazione/benchmark08/vulkan-1-300/Banco.tape').Path
$env:MEMOTAPE_FLOW_OUTPUT = Join-Path (Get-Location) '.scratch/nemotron3-diarizzazione/benchmark08/flow-ripetizione'
$env:MEMOTAPE_NEMOTRON3_MODEL = 'D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf'
cargo test --locked --manifest-path src-tauri/Cargo.toml ticket08_flusso_tape -- --ignored --test-threads=1 --nocapture
```

Solo fixture locali: copia Tape, player core Range, correzione/rinomina, uscite
Testo/Markdown nelle sei lingue, vero CancelToken su `diarize_saved` e
finalizzatore di produzione, stato provvisorio preparato dal banco, Tape v1.
Non attesta clipboard OS, playback ascoltato, dialoghi o rendering della vista,
né Annulla durante il secondo Ingresso nell'orchestrazione completa.
La procedura UI è `.scratch/nemotron3-diarizzazione/ui08-da-verificare.md`.

```powershell
python tools/nemotron3-benchmark/summarize.py
python tools/nemotron3-benchmark/isolate_diff.py
```

Il riepilogo richiede i quattro casi da 300 s, dichiara cartelle mancanti/ancora
in corso e distingue integrità da salute realtime. `LiveDiarizationLagging` è
un esito negativo anche quando ASR/audio/finale sono integri. Il diff isolato
verifica gli SHA-256 della baseline pre-08 e non ripristina alcun file.

## Corpus finito creato — estensione 08

Provenienza, licenze, parametri e risultati nell’addendum `.scratch/nemotron3-diarizzazione/report08-addendum.md`. Otto piccoli archivi VoxForge con GPL3+ e stem Windows TTS; audio/reference sono materiali locali ignorati da Git. Reference energia su stem puliti è un proxy, non gold manuale; non etichettare l’intera durata TTS silenziosa. Nessun risultato del diarizer crea la reference. Conservare output esistenti: i comandi rifiutano di sovrascriverli; una ripetizione richiede una cartella di prova fresca con gli stessi archivi verificati.

```powershell
python tools/nemotron3-benchmark/corpus.py --prepare-jobs
./tools/nemotron3-benchmark/synthesize.ps1 -Jobs .scratch/nemotron3-diarizzazione/corpus08/tts-jobs.json
python tools/nemotron3-benchmark/corpus.py --construct
./tools/nemotron3-benchmark/compare-corpus.ps1
$env:MEMOTAPE_CORPUS_ROOT = (Resolve-Path '.scratch/nemotron3-diarizzazione/corpus08').Path
cargo test --locked --manifest-path src-tauri/Cargo.toml ticket08_namespace_del_corpus -- --ignored --test-threads=1 --nocapture
python tools/nemotron3-benchmark/score_corpus.py
python -m unittest discover -s tools/nemotron3-benchmark -v
```

Il namespace proiettato→raw proviene dalla trasformazione di produzione, con replay identico alle proiezioni originali; si compone con il mapping DER. Non ottimizzare un mapping separato per il testo. La copertura ASR temporizzata è esplicita: errori delle unità non rappresentano tutto il testo né WER.
