# Handoff del ticket 03

Implementazione locale della pulizia delle Registrazioni per Ingresso, nella
conversazione `01a1118a-4bb6-7161-8c05-62d6ec3b395d`. Il ticket 04 non è stato
avviato. Nessun commit, PR, installer o pubblicazione. Il checkout contiene
modifiche precedenti e concorrenti: non usare il diff da HEAD come delta del 03.

Stato finale: `partial` per collaudo manuale Tauri/ascolto. Sei controlli verdi,
118 test frontend e 278 test Rust; review Standards/Spec senza finding residui.
Smoke DFN3 a 48/44,1/24 kHz e cattura release a due Ingressi passati. La misura
finale di 28 s ha zero perdite di buffer e rapporto medio mixer/DSP 0,5196 s/s.

## File e contratti

Il delta isolato è in `verification-03/delta.patch`; l'elenco completo è in
`verification-03/changed-files.txt`. La baseline fotografa l'inizio del ticket.

- `audio_toolkit/cleaning.rs`: `ConfiguredCleaning::recovering` conserva soltanto
  il PCM non ancora emesso e recupera in bypass il singolo Ingresso guasto.
  `prepare` carica prima della cattura; `CleaningLog` registra solo intervalli
  effettivamente consegnati come elaborati. Nei file il guasto resta fatale.
- `audio_toolkit/deepfilter.rs`: reset da piano Tract vergine già ottimizzato,
  senza ricaricare il modello a ogni Pausa/cambio.
- `audio_toolkit/mixer.rs`: `configuration` scarica prima il ricampionatore
  upstream e poi il PCM con il controllo precedente. I frame del dispositivo
  e quelli consegnati al PCM hanno contatori distinti; le code rispettano la
  durata globale anche con cambi ripetuti su segmenti non interi.
- `audio_toolkit/resample.rs`: `continuation` usa storia limitata e pre-roll
  allineato al periodo razionale, conservando fase/durata senza riconsegnare
  quel contesto. Non sostituirlo con un reset a origine zero o un taglio finale.
- `managers/recording.rs`: runtime separati per Ingresso; controllo richiesto
  distinto da quello applicato. `Recorder::set_audio` e avvio condividono il
  lock; Stop congela le richieste. I metadati vengono salvati nel Tape.
- `RecordingCleaningFailed { sessionId, ingresso, error }`, registrato in
  `lib.rs` e generato in `bindings.ts`. Home filtra sessione, conserva un
  avviso per Ingresso e usa il banner accessibile. Controlli persistenti nelle
  Impostazioni e testi nelle sei lingue.
- `PRODUCT.md` e ADR-0021 descrivono il comportamento e il costo di preparazione
  dei runtime anche a pulizia spenta.

Per il 04: con un solo dispositivo il Tape conserva `mix.ogg`, ma
`pulizia_audio` identifica `Microfono` o `Sistema`, non `Mix`. Il riuso degli
intervalli deve considerare questo caso per evitare una seconda pulizia del
mix. Gli intervalli sono semiaperti, con frequenza esplicita; non attribuire
pulizia alla coda recuperata in bypass. Non recuperare un originale dal Tape.

## Comandi

Una sola compilazione Rust alla volta. Non cambiare `.cargo/config.toml`,
non usare il target dell'app aperta, non arrestare processi dell'utente.

```powershell
$env:CARGO_TARGET_DIR='D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust'
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --locked --manifest-path src-tauri/Cargo.toml
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Per il test del Cestino eseguire il comando completo fuori sandbox. Per
rigenerare i contratti, dalla cwd `src-tauri`, usare le stesse impostazioni
Cargo con `test --locked rigenera_bindings_di_sviluppo -- --ignored --test-threads=1`.
Non modificare `bindings.ts` a mano.

Smoke nuovi, separati e sequenziali dalla root:

```powershell
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --release --locked --manifest-path src-tauri/Cargo.toml dfn3_registrazione_coda_pause_e_stato_indipendente -- --ignored --test-threads=1 --nocapture
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --release --locked --manifest-path src-tauri/Cargo.toml dfn3_due_ingressi_nativi_release -- --ignored --test-threads=1 --nocapture
```

Il secondo apre i dispositivi predefiniti, riproduce due volte `parlato-it.wav`
e acquisisce 28 s. Richiede Nemotron locale e il modello DFN3 già presente;
non modifica le Impostazioni né scrive nella Libreria dell'utente. Salva
un Tape temporaneo e lo riapre. I log del 03 sono in `verification-03`.

Bun installato: `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`.
Per i controlli frontend usare `run typecheck`, `run test`, `run check`;
per la formattazione backend `run format:backend`.

## Verifiche e limiti

Esiti conclusivi, misura nativa e review sono riportati nel ticket 03.
La prova nativa del core non attesta i clic nella finestra Tauri, il player
visibile, l'ascolto percettivo delle transizioni, una voce reale al microfono,
sessioni lunghe o prestazioni su altri PC. Bundle, licenza dei pesi e taratura
DFN3 del ticket 02 restano fuori dal lavoro del 03 e non sono stati riaperti.
