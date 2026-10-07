# Handoff del ticket 06

Conversazione `01a11206-a2b7-7d73-9d2c-5056119f19c4`. Implementazione locale
conclusa, stato **partial** per corpus umano, ascolto e UI. Nessun commit,
PR, installer o pubblicazione. Il 07 resta in attesa del coordinatore.

Preservare le modifiche precedenti e concorrenti. Il delta del 06 è
`verification-06/delta.patch`, rispetto a `verification-06/baseline`, non HEAD.
Controlli, denominatori, risultati e limiti in `verification-06/README.md`;
review indipendenti in `verification-06/review.md`.

## Contratti per il 07

- I quattro livelli di `ProfiloAudio::sensibilita` restano indipendenti da
  pulizia e Guadagno. Default Bilanciato; profili Microfono/Sistema/File-misto.
  Impostazioni è il controllo al volo del 06; la toolbar appartiene al 07.
- `Recorder::set_audio` aggiorna i controlli persistenti usando il lock
  dell'avvio. Il worker legge il profilo di `Kind`, anche se un solo Ingresso
  è esposto all'ASR come Mix. Dopo Stop conserva i valori già applicati;
  quelli persistiti saranno usati dalla sessione seguente.
- Ogni coppia `LiveFeed`/`LiveFrames` condivide una `ProtectionTimeline` nuova.
  `Mixer::protect` fissa il livello al prossimo campione sulla timeline raw
  prima di ricampionamento, buffer DFN3 e coda ASR. In Pausa il livello nuovo
  vale dalla ripresa. La revisione segue il PCM anche con ASR arretrata.
- Cambiare solo sensibilità non scarica/resetta DFN3 e non cambia PCM, mix,
  tracce, durata o Forma d'onda. La decisione usa il livello al primo campione
  del frame Silero. Timeline e selezione restano separate per Ingresso/sessione.
- `pipeline::transcribe_protected` riusa `PhraseEvidence` del 05 sui prefissi.
  Appena un prefisso ammette la candidata, invia tutta la Frase accumulata
  con prefill e tempi originali, poi continua a trazione. L'ammissione dura
  fino a fine Frase: Parziali/testo già pubblicati non vengono ritirati e la
  parola in corso non viene tagliata. Spento mantiene Silero.
- Limiti: osservazioni 12 frame, candidata massimo 600 frame/18 s. Dopo
  l'ammissione il secondo buffer della Frase è svuotato. Il canale ASR
  preesistente resta quello precedente. Nessun lavoro aggiunto in callback.
- Bypass DFN3 del 03: recupero PCM pendente e avviso per solo Ingresso/sessione.
  Protezione indipendente dal successo della pulizia. Salvataggio e audio per
  i Parlanti restano completi. Diarizzazione solo dopo Stop e fine ASR.
- Sei traduzioni aggiornate sullo scope live. Binding rigenerato senza delta.
  PRODUCT, AGENTS, ADR-0023 e nuova ADR-0024 descrivono la decisione.

## Verifiche locali

Sei controlli verdi: typecheck, 119 test frontend, lint frontend, fmt,
clippy `--all-targets -- -D warnings`, 296 test Rust (29 ignorati). Generatore
dei binding passato. Sei nuovi test deterministici su selezione/Parziali,
coda, PCM, Pausa/Stop, sessioni, due Ingressi, resampling e bypass.

Matrice streaming nativa passata: 288 inferenze reali sui tre ASR,
144 configurazioni sintetiche distinte ripetute con etichette dei due profili
sullo stesso PCM. Non sono due dispositivi simultanei. Bilanciato riduce
false Frasi Whisper da 2 a 0 per profilo, sommando pulizia on/off;
Nemotron/Parakeet già erano a 0. I livelli prudenti perdono 0 parole rispetto
a Spento; Più selettivo perde 14 nella voce attenuata. Parakeet conserva
«C No.» già nella baseline. Nemotron mantiene i Parziali durante il parlato.

Banco release WASAPI: due catture reali, due DFN3 e due Nemotron,
cambi attraverso uno SettingsStore temporaneo, Pausa/Riprendi, Stop,
code dei due Ingressi e Tape riaperto. Risultati finali nel ticket e README.
Il test non modifica Impostazioni o Libreria dell'utente.
Finale passata: audio 27.030 ms, rapporto worker 0,351 s/s, code al massimo
660 ms per Ingresso e finali entrambe 0, frame persi [0, 0]. Microfono senza
Frasi, Sistema con 4 Frasi e 12 Parziali; mix/tracce e Forma d'onda riaperti.

## Comandi riproducibili

Una sola build Rust alla volta; modelli reali sequenziali. Target isolato
per preservare l'app eventualmente aperta. Usare gli stessi override senza
modificare `.cargo/config.toml`:

```powershell
$env:CARGO_TARGET_DIR='D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust'
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --locked --manifest-path src-tauri/Cargo.toml
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --release --locked --manifest-path src-tauri/Cargo.toml protezione_due_ingressi_nativi_release -- --ignored --test-threads=1 --nocapture
```

Matrice: stesso comando con filtro `protezione_live_corpus_tre_asr`.
Prefissi DFN3/Silero: filtro `protezione_live_prefissi_corpus`, debug sufficiente.
Binding: cwd `src-tauri`, stessi override, `test --locked
rigenera_bindings_di_sviluppo -- --ignored --test-threads=1`.
Bun verificato: `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`.
Suite completa fuori sandbox per il Cestino. Non ripetere prove native appena
passate senza una nuova modifica, errore o dubbio concreto.

## Criteri ancora aperti

Respiri, voce bassa e risposte brevi umane; ascolto percettivo; UI Tauri,
player, tastiera/Narrator e cambi manuali durante cattura. WASAPI con i tre
ASR e analisi finale reale dei Parlanti non collaudati dal 06. Il banco
Nemotron breve non attesta ore di lavoro o altro hardware. Licenza dei pesi
e bundle del 02 restano aperti. Conservare `partial`: i test del core e il
corpus TTS non soddisfano questi criteri mancanti.
