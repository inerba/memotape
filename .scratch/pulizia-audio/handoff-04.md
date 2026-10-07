# Handoff del ticket 04

Conversazione `01a111c3-fb81-7551-a54c-1869984e0325`. Implementazione locale
conclusa, stato `partial` per collaudo manuale Tauri/ascolto e disco pieno.
Nessun commit, PR, installer o pubblicazione; ticket 05 non avviato.
Preservare le modifiche precedenti e concorrenti: il diff da HEAD non è il delta
del 04. Baseline, delta e elenco dei file sono in `verification-04/`.

## Contratti da conservare

- `ConfiguredCleaning::reuse` protegge campioni già trattati con conversione
  floor/ceil delle frequenze. Nel mix accetta anche metadati Microfono/Sistema
  provenienti dalle Registrazioni a dispositivo singolo. Il profilo è letto
  al blocco decodificato, prima del consumo ASR.
- `AudioProcessor::context` prepara con storia limitata senza emettere audio
  o aggiungere trattamenti. DFN3 allinea il pre-roll alla griglia temporale
  assoluta e scarta l'uscita del contesto; evitare reset a fase zero.
- `TapeAudio` prepara i nuovi Ogg accanto alla Sorgente in una sottocartella
  esclusiva di `.memotape`. Per Ingressi separati ricostruisce il mix dai PCM
  elaborati prima di Opus, senza filtrare nuovamente il mix. ASR e diarizzazione
  leggono gli Ogg destinati al documento. Ogg immutati conservati byte per byte.
- `rewrite_audio_cancellable` sostituisce audio, Forma d'onda, documento e
  metadati in un solo commit; conserva voci ZIP e campi JSON sconosciuti.
  Annulla/errore rimuovono solo i temporanei posseduti e conservano il Tape.
- Il player calcola la Forma d'onda mancante in memoria, senza salvataggio
  implicito; protocollo `Cache-Control: no-store`. Il frontend esistente
  rimonta il player al termine di Trascrivi. Nessun JS del frontend modificato
  nel 04; aggiornate le spiegazioni nelle sei lingue.
- PRODUCT, AGENTS e ADR-0022 descrivono il nuovo contratto e sostituiscono
  la precedente prescrizione di salvataggio automatico dei picchi.

## Verifiche osservate

Sei controlli verdi: typecheck, 118 test frontend, lint frontend, fmt backend,
clippy `-D warnings`, 284 test Rust; 24 smoke ignorati nel controllo ordinario.
Rigenerazione binding riuscita: file identico alla baseline. Review indipendenti
Spec/Standards sul delta; corretti due commenti obsoleti, nessun difetto certo
residuo. Evidenze e limiti dettagliati nel ticket e `verification-04/review.md`.

Smoke DFN3 release separato: passato, 26,89 s. PCM stereo 44,1 kHz con intervallo
parziale: durata e campioni protetti invariati, RMSE 0 fra due partizioni.
Fixture italiana con Nemotron reale: tutte le parole attese conservate;
secondo riuso con Ogg identico e nessun nuovo intervallo. I confronti PCM sono
distinti da quelli dopo Opus. Nessuna ripetizione della calibrazione del 02.

Una sola compilazione Rust per volta; non modificare `.cargo/config.toml`
e non usare il target dell'app aperta o arrestare processi dell'utente.

```powershell
$env:CARGO_TARGET_DIR='D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust'
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --locked --manifest-path src-tauri/Cargo.toml
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --release --locked --manifest-path src-tauri/Cargo.toml dfn3_ritrascrive_tape_parziale_con_parole_ai_confini_e_riusa_gli_ogg -- --ignored --test-threads=1 --nocapture
```

Il test del Cestino richiede il controllo completo fuori sandbox. Lo smoke usa
il modello DFN3 locale e Nemotron già scaricato in AppData; crea solo Tape
temporanei, senza modificare Impostazioni o Libreria dell'utente. Per rigenerare
bindings dalla cwd `src-tauri`, stesse impostazioni Cargo e
`test --locked rigenera_bindings_di_sviluppo -- --ignored --test-threads=1`.
Bun: `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`.

## Limiti aperti

Non attestati clic, accessibilità e aggiornamento player nella finestra Tauri,
ascolto percettivo delle transizioni, modifica profilo durante Trascrivi,
disco pieno reale, sessioni lunghe e altri PC. Annulla, guasto filtro e rinomina
Windows impedita sono coperti. Bundle e licenza dei pesi restano aperti dal 02.
