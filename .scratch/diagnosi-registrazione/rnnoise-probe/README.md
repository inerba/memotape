# Prototipo RNNoise separato da Memotape

7 ottobre 2026, Windows x64, Ryzen 7 3700X. Nessun commit.

`nnnoiseless` è fissato a 0.5.2 nel manifest e nel lockfile. È un port Rust della
generazione storica di RNNoise, non il modello ufficiale RNNoise 0.2/little.
Il profilo dev di questo piccolo banco è interamente ottimizzato a livello 3.
Non è una dipendenza dell'app e non cambia Impostazioni, Libreria o Tape.

Il banco accetta PCM16 WAV mono 48 kHz oppure un file f32 little-endian mono
normalizzato a 48 kHz. L'input viene convertito alla scala PCM16 richiesta
dall'API. Un secondo di riscaldamento è escluso dal tempo del flusso continuo.
Uno, due e quattro stati indipendenti vengono eseguiti in sequenza: non sono
registrazioni WASAPI né comprendono ASR, worker o ricampionamento. Il pad
finale è incluso nella durata dichiarata per il benchmark.

```powershell
cargo run --locked --manifest-path .scratch/diagnosi-registrazione/rnnoise-probe/Cargo.toml --target-dir .scratch/diagnosi-registrazione/rnnoise-probe/target -- .scratch/diagnosi-registrazione/benchmark-voce-48k.f32
```

Il test ignorato `dfn3_costo_reale_per_ingresso` crea quel PCM dalla stessa
fixture usata per DFN3, con il ricampionatore dell'app. I risultati sono in
`../dfn3-performance-before.log`, `../dfn3-performance.log` e
`../rnnoise-performance.log`. Non equiparare la pipeline completa DFN3 alla
sola inferenza del port Rust.

```powershell
cargo run --locked --manifest-path .scratch/diagnosi-registrazione/rnnoise-probe/Cargo.toml --target-dir .scratch/diagnosi-registrazione/rnnoise-probe/target -- .scratch/pulizia-audio/dfn3-diretto/01-originale.wav .scratch/diagnosi-registrazione/rnnoise-campione.wav
```

Il secondo argomento facoltativo esporta il campione per ascolto con uno stato
nuovo e durata originale. Non normalizza, non compensa il ritardo iniziale e
non valida la conservazione delle parole. Le misure separate sul campione di
22,097625 s sono in `../rnnoise-campione-performance.log`.

Fonti: [API nnnoiseless 0.5.2](https://github.com/jneem/nnnoiseless/blob/v0.5.2/src/denoise.rs),
[README](https://github.com/jneem/nnnoiseless/blob/v0.5.2/README.md).

Correzione applicata all'app: `ndarray`, `tract-data` e `num-complex` sono ora
ottimizzati anche in dev. Sei controlli verdi: typecheck, 132 test frontend,
Biome, cargo fmt, clippy e 297 test Rust (34 ignorati). Il benchmark DFN3 reale
è stato eseguito separatamente e passa, verificando anche durata e PCM finito.
