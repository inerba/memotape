# Diagnosi: Registrazione silenziosa con DFN3 nella build debug

Prova del 6-7 ottobre 2026, sul checkout principale. Nessun commit.

## Causa verificata

Il worker avanza il mixer con QPC prima di elaborare un blocco già acquisito.
Quando il DSP ritarda oltre 500 ms, `advance` inserisce silenzio proprio dove
si trova audio in coda. `push` scarta poi quell'audio come sovrapposto.
DFN3 non ottimizzato in debug accentua il ritardo e può esaurire il pool WASAPI.
A Stop la cattura restava aperta durante lo smaltimento: la coda continuava
a ricevere blocchi, rendendo possibile un completamento senza limite.

## Evidenza

- Tape utente `Registrazione 2026-10-06 23-39-33.tape`: 22.037 ms,
  zero Frasi, Forma d'onda con massimo 0.0.
- Test sui dispositivi isolati: la fixture arriva al loopback con picco 0.4218534;
  il mixer conserva il segnale anche con DFN3 e ConfiguredCleaning.
- Comando completo, prima: audio di sistema ricevuto 1.282 ms,
  scartato 1.272 ms, silenzio inserito 6.777 ms. Un secondo avvio
  supera il watchdog di 45 s.
- Test deterministico `blocchi_acquisiti_in_coda_non_diventano_silenzio_se_il_worker_ritarda`:
  rosso prima, verde dopo. Simula due Ingressi con DSP ritardato senza sleep.
- Comando completo dopo correzione della coda: Tape 5.270 ms, picco 0.4208393,
  audio di sistema scartato 0 ms; Parakeet produce testo.
- Con i componenti DSP ottimizzati: Tape 5.146 ms, picco 0.41466266,
  microfono ricevuto 5.290 ms su 5.293 ms di timestamp;
  audio di sistema ricevuto 4.524 ms su 4.572 ms di timestamp, scartato 0 ms.
  Il log completo è `probe-optimized.log`.
  Riapertura del documento: `completa = true`, una Frase dell'Ingresso Sistema.

## Controlli finali

- Typecheck, test frontend (132 passati) e Ultracite: exit 0.
- `cargo fmt --check` e Clippy su tutti i target con warning vietati: exit 0.
- Suite Rust: 297 passati, 0 falliti, 33 ignorati; `rust-tests.log`.
- Nessuna strumentazione `[DEBUG-...]` o hook del probe nel codice dell'app.

## Correzione

`Mixer::advance_pending` usa il timestamp del blocco in attesa per non
sostituire dati già acquisiti con silenzio. Pausa/Riprendi restano ancorati
all'orologio reale. Il manager chiude le catture prima di svuotare la coda
di Stop. I profili dev ottimizzano libDF, Tract core/linalg, FFT e Rubato;
il codice dell'app resta debug.

`probe.rs` è un harness temporaneo conservato qui come artefatto di diagnosi.
I collegamenti temporanei in `lib.rs` e `main.rs` sono stati rimossi.
Usava il comando reale `record`, un AppHandle nascosto, le impostazioni
copiate in `output/settings.json` e una fixture riprodotta per la prova.
Non modificava le impostazioni dell'utente; disattivava soltanto la
Diarizzazione finale nella copia della diagnosi. I Tape in `output/`
sono artefatti diagnostici.

La prova attesta cattura, trattamento, salvataggio, Forma d'onda e ASR
del segnale noto di sistema. Il microfono consegna PCM; la conservazione
del parlato sui due Ingressi è verificata dal test deterministico.
Non attesta un ascolto umano della propria voce nella UI. Gli Xrun
occasionali all'apertura del driver restano visibili e non fermano la cattura.
