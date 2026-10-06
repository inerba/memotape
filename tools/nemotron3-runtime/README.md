# Prova del runtime Nemotron 3

Eseguibile di verifica Windows x64 separato dall'app. Il manifest e il lock fissano
`transcribe-cpp` e `transcribe-cpp-sys` al commit
`e6672a8672913b47f1571c66c54bee789d028416`, con DLL compilate da quella sorgente.
Non scarica modelli, non modifica le Impostazioni e non pubblica pesi.

Richiede gli strumenti nativi di `AGENTS.md` e `VULKAN_SDK`. La configurazione
Cargo locale della root vale anche per questa prova. Non impostare
`TRANSCRIBE_DIR` su una release diversa: quel percorso disattiva la compilazione
della sorgente fissata. Il probe controlla versione e commit della DLL caricata.

```powershell
cargo build --locked --manifest-path tools/nemotron3-runtime/Cargo.toml --target-dir src-tauri/target/nemotron3-proof/probe-target
```

Le DLL e l'exe restano nella directory `debug` di quel target, separati dalle DLL
dell'app. Eseguire una sola compilazione Rust e un solo smoke alla volta.

```powershell
$probe = 'src-tauri/target/nemotron3-proof/probe-target/debug/memotape-runtime-probe.exe'
& $probe cpu 'D:/modelli/Nemotron-3-Diarization-BF16.gguf' 'src-tauri/tests/fixtures/parlato-due-voci.wav' diar-offline auto
& $probe cpu 'D:/modelli/Nemotron-3-Diarization-BF16.gguf' 'src-tauri/tests/fixtures/parlato-due-voci.wav' diar-stream auto
& $probe vulkan 'D:/modelli/Nemotron-3-Diarization-BF16.gguf' 'src-tauri/tests/fixtures/parlato-due-voci.wav' diar-offline auto
& $probe vulkan 'D:/modelli/Nemotron-3-Diarization-BF16.gguf' 'src-tauri/tests/fixtures/parlato-due-voci.wav' diar-stream auto
```

Ogni chiamata è un processo nuovo: il backend è esplicito e il probe rifiuta un
fallback. Offline usa `VeryHighLatency`, streaming usa `LowLatency` con frame da
30 ms e richiede segmenti disponibili **prima** di `finalize`. Sulla fixture
`parlato-due-voci.wav` richiede esattamente due Parlanti alternati A/B/A/B,
almeno 14 s coperti e 1,5 s di copertura in ciascuna delle quattro finestre
centrali note. Rifiuta sovrapposizioni oltre 200 ms e segmenti fuori dall'audio
(tolleranza di padding 80 ms). Sono criteri grossolani per questa fixture;
non certificano qualità o latenza sull'italiano.
`diar-stream` è riservato al modello Nemotron 3, `diar-offline` vale anche per
Sortformer. La fixture deve essere WAV PCM16 mono a 16 kHz.

Per le regressioni native usare `parlato-it.wav`, `asr-offline` e Lingua del
parlato `it` oppure `auto`. Parakeet usa `auto` (non accetta un'indicazione).
Per Nemotron ASR usare anche `asr-stream`: il probe richiede Parziali prima della
fine e il testo finale deve coincidere con tutto il parlato della fixture,
ignorando solo punteggiatura, maiuscole e spazi. Verifica l'hint italiano
(`it`/`it-IT`) oppure la sua assenza in automatica; se il runtime restituisce
una lingua rilevata deve essere italiana. Whisper in automatica deve rilevare
`it`; Nemotron e Parakeet possono non esporre una lingua rilevata.
Parakeet e Whisper nell'app restano a Frasi intere.

Le righe JSON su stdout riportano dispositivo, capacità, tempi, testo,
segmenti e campioni dei timestamp. Un errore termina con codice 1; un crash
nativo può terminare prima della riga finale e va registrato dal chiamante.
Conservare stdout, stderr e codice di uscita anche per gli smoke falliti.

La matrice completa (20 processi, CPU e Vulkan) si ripete con:

```powershell
& tools/nemotron3-runtime/run.ps1 -Nemotron3Model 'D:/modelli/Nemotron-3-Diarization-BF16.gguf'
```

Il runner verifica dimensione e SHA-256 dei quattro modelli esistenti contro
il catalogo dell'app, e del BF16 e delle fixture contro `artifacts.json`, prima
di avviare le inferenze. `-ModelsDirectory`, `-ProbePath` e `-ResultsDirectory` permettono di
usare percorsi diversi. Conserva gli esiti anche dei casi falliti e termina con
codice 1 se uno smoke non riesce.

I cinque test dei criteri di accettazione si eseguono con `cargo test --locked
--manifest-path tools/nemotron3-runtime/Cargo.toml --target-dir
src-tauri/target/nemotron3-proof/probe-target` e non caricano modelli.

La preparazione del modello, gli hash e gli esiti misurati sono in
[`docs/research/nemotron3-runtime.md`](../../docs/research/nemotron3-runtime.md).
