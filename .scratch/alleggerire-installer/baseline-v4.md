# Misura di partenza: protezione con Silero v4 (ONNX Runtime, frame da 30 ms)

Ticket 01. Riferimento per il confronto del ticket 03. Eseguita il 18 ottobre 2026.

## Codice, corpus e modelli

- Codice: commit `8fd8901d`, che rispetto a `main` (`430117bb`) aggiunge solo Markdown in
  `.scratch/`: il codice è quello di `main`, senza modifiche (`git diff main HEAD -- src-tauri`
  vuoto). Durante le misure la cartella era su `feat/rnnoise-file-e-tape`, un'altra sessione, allo
  stesso commit; le sue modifiche a `Cargo.toml` e `build.rs` sono arrivate alle 15:38, dopo
  l'ultima misura (15:37), e `src-tauri/vendor/` (15:36) non era ancora referenziato dalla build.
  Rust 1.96.1, profilo `release`.
- VAD: `vad-rs` 0.1.6 su `ort` 2.0.0-rc.12 (ONNX Runtime 1.24.2), `src-tauri/resources/silero_vad.onnx`
  (Silero v4, 1 807 522 byte, SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`).
  Segmentatore con `Params::default()`: prefill 300 ms, onset 60 ms, hangover 700 ms, massimo 18 s,
  soglia 0,4.
- Pulizia: DFN3 da `src-tauri/resources/models/DeepFilterNet3_onnx.tar.gz`
  (SHA-256 `c94d91f70911001c946e0fabb4aa9adc37045f45a03b56008cb0c8244cb63616`).
- Corpus: `.scratch/pulizia-audio/verification-05/corpus` (sei WAV, hash in `manifest.json`).
- ASR, in `%APPDATA%\it.memotape.desktop\models`, dagli URL fissati in `models.json`:
  `nemotron-3.5-streaming-0.6b-q5km`, `cohere-transcribe-03-2026-q5km`, `whisper-medium-q8`,
  `parakeet-tdt-0.6b-v3-q5km`.

## Comandi

In `src-tauri`, uno dopo l'altro:

```
cargo test --release --lib -- --ignored --test-threads=1 --nocapture misura_evidenza_corpus_protezione
cargo test --release --lib -- --ignored --test-threads=1 --nocapture corpus_protezione_quattro_livelli_tre_asr
```

Il primo passa (0,61 s dopo la compilazione). Il secondo **finisce in panico per un limite del test,
non della protezione**: scorre tutti i modelli di Trascrizione del catalogo (oggi 18, non più tre)
e si ferma al primo non scaricato, `parakeet-primeline` (quinto), con `ModelMissing`. Prima del
panico ha già superato le asserzioni di regressione (stesse candidate per Spento, Sensibile e
Bilanciato sulla voce attenuata e su «Sì. No.», con e senza pulizia) e stampato le 576 righe dei
primi quattro ASR (4 ASR × 6 casi × 2 stati della pulizia × 4 livelli × 3 profili equivalenti).
Whisper large-v3 turbo, uno dei tre ASR del ticket 05, viene dopo nel catalogo e non è misurato;
al suo posto ci sono Whisper medium e, in più, Cohere. `matrix.json` non viene riscritto e
l'asserzione finale non gira; calcolata a mano sui quattro ASR passa: false Frasi Spento 2,
Bilanciato 0. Il ticket 03 deve rilanciare lo stesso comando con gli stessi modelli scaricati, così
il confronto è a parità di condizioni.

Le probabilità con la pulizia non le misura nessuno dei due test: vengono da una sonda temporanea
(sotto), applicata, eseguita e tolta; il codice committato non cambia.

```
cargo test --release --lib -- --ignored --test-threads=1 --nocapture sonda_probabilita_con_pulizia
```

## Frasi candidate, tenute e scartate

Non dipendono dall'ASR. «Candidate» sono quelle di Spento; per ogni livello il numero di Frasi
tenute, le scartate sono la differenza con Spento. Uguali nei tre profili (Microfono, Sistema,
File/misto), perché il PCM e la decisione sono gli stessi.

| Campione | Pulizia | Spento | Sensibile | Bilanciato | Selettivo |
| --- | --- | --- | --- | --- | --- |
| silenzio | no / sì | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| rumore-continuo | no / sì | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| rumore-intermittente | no / sì | 1 / 0 | 1 / 0 | 0 / 0 | 0 / 0 |
| soffio-sintetico | no / sì | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| voce-attenuata-30db | no / sì | 3 / 0 | 3 / 0 | 3 / 0 | 1 / 0 |
| si-no | no / sì | 2 / 2 | 2 / 2 | 2 / 2 | 2 / 2 |

## Testo delle Frasi tenute

Lo stesso a ogni livello che tiene la candidata; «—» è testo vuoto.

| Campione, pulizia | Livelli | Nemotron | Cohere | Whisper medium | Parakeet |
| --- | --- | --- | --- | --- | --- |
| rumore-intermittente, no | Spento, Sensibile | — | «Il resto è finito.» | «Sottotitoli creati dalla comunità Amara.org» | — |
| voce-attenuata-30db, no | Spento, Sensibile, Bilanciato | «Buongiorno a tutti» | «Buongiorno a tutti.» | «Buongiorno a tutti.» | «Buongiorno a tutti.» |
| voce-attenuata-30db, no | Selettivo | — | — | — | — |
| si-no, no | tutti | «Sì. No.» | «Sì. No.» | «Sì. No.» | «C No.» |
| si-no, sì | tutti | «Sì. No.» | «D. No.» | «Sì. No.» | «T No.» |

- False Frasi (testo non vuoto su campioni non parlati): solo il rumore intermittente senza pulizia,
  con Cohere e Whisper medium, a Spento e Sensibile. Zero a Bilanciato e Selettivo.
- Risposte brevi: «Sì. No.» resta a ogni livello, con e senza pulizia; le due candidate ci sono
  sempre. Gli errori di Cohere e Parakeet («D.», «C», «T») sono dell'ASR, non della selezione.
- Voce debole, senza pulizia: le tre candidate insieme danno solo «Buongiorno a tutti» (3 delle
  14 parole attese) con tutti e quattro gli ASR. Selettivo scarta due candidate e perde quelle
  3 parole; la candidata che tiene esce vuota. Il log stampa i testi già uniti, quindi non dice
  se le 3 parole vengono da una candidata sola.
- Voce debole, con la pulizia: nessuna candidata a nessun livello, zero parole.

## Probabilità di Silero

Media e picco sui frame con probabilità ≥ 0,4 di ogni candidata (dalla misura dell'evidenza e,
con la pulizia, dalla sonda), più picco e media sull'intero file. Non dipendono dal livello.

| Campione | Pulizia | Candidata (frame) | Frame ≥ 0,4 | Picco | Media |
| --- | --- | --- | --- | --- | --- |
| rumore-intermittente | no | 0..67 | 44 | 0,878327 | 0,620594 |
| voce-attenuata-30db | no | 6..43 | 3 | 0,445281 | 0,419942 |
| voce-attenuata-30db | no | 50..150 | 56 | 0,684943 | 0,528102 |
| voce-attenuata-30db | no | 182..299 | 92 | 0,775435 | 0,603881 |
| si-no | no | 0..37 | 9 | 0,999219 | 0,820039 |
| si-no | no | 37..73 | 8 | 0,983800 | 0,886298 |
| si-no | sì | 0..36 | 6 | 0,992012 | 0,853031 |
| si-no | sì | 36..72 | 7 | 0,924384 | 0,793943 |

| Campione | Frame | Picco senza / con pulizia | Media senza / con pulizia |
| --- | --- | --- | --- |
| silenzio | 67 | 0,042866 / 0,042866 | 0,020149 / 0,020149 |
| rumore-continuo | 67 | 0,215969 / 0,042866 | 0,037986 / 0,020149 |
| rumore-intermittente | 67 | 0,878327 / 0,042866 | 0,503498 / 0,020149 |
| soffio-sintetico | 67 | 0,051180 / 0,242118 | 0,016716 / 0,030054 |
| voce-attenuata-30db | 299 | 0,775435 / 0,042866 | 0,370512 / 0,016039 |
| si-no | 74 | 0,999219 / 0,992012 | 0,248766 / 0,179791 |

Con la pulizia, rumore continuo, rumore intermittente e voce attenuata danno a Silero gli stessi
valori del silenzio digitale (picco 0,042866): DFN3 restituisce di fatto silenzio, e con esso
toglie la voce a −30 dB.

## Differenze dal ticket 05 della pulizia audio

Confronto con `verification-05/matrix.json` (non con il suo README, che descrive la voce
attenuata con tutto il testo in entrambi gli stati della pulizia mentre la sua matrice dice
altro):

- Con la pulizia nessuna differenza: già nel 05 voce attenuata e rumore intermittente davano
  0 candidate a ogni livello.
- Le candidate senza pulizia sono le stesse (3, 3, 3, 1 sulla voce attenuata), come le
  probabilità della misura preliminare: il VAD si comporta come allora.
- Cambia il testo senza pulizia. Nel 05 la voce attenuata dava tutte le 14 parole a Spento,
  Sensibile e Bilanciato, e Selettivo teneva «Il computer trasforma la voce in testo.» (7 parole
  perse); oggi escono 3 parole e la candidata tenuta da Selettivo è vuota. Stesse candidate PCM,
  quindi la differenza sta tra la selezione e l'ASR (transcribe-cpp 0.4.0, `430117bb` o i modelli
  cambiati), non in Silero né in DFN3, che qui è spento.

Il ticket 03 deve confrontarsi con questa misura, non con il 05, e non attribuire a Silero v6.2
le parole già perse oggi.

## Sonda temporanea

Aggiunta in fondo a `src-tauri/src/engine/pipeline/protection_tests.rs`, prima di `fn report`,
eseguita e tolta con `git checkout`. Per v6.2 basta cambiare la riga del `Silero::new`.

```rust
#[test]
#[ignore = "sonda temporanea: probabilità Silero con e senza pulizia"]
fn sonda_probabilita_con_pulizia() {
    use crate::audio_toolkit::cleaning::{CleaningLog, ConfiguredCleaning};
    use crate::audio_toolkit::deepfilter;
    use crate::transcript::Ingresso;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let corpus = root.join("../.scratch/pulizia-audio/verification-05/corpus");
    for name in [
        "silenzio",
        "rumore-continuo",
        "rumore-intermittente",
        "soffio-sintetico",
        "voce-attenuata-30db",
        "si-no",
    ] {
        for cleaning in [false, true] {
            let processor: Box<dyn AudioProcessor> = if cleaning {
                Box::new(ConfiguredCleaning::new(
                    root.join(deepfilter::MODEL_FILE),
                    || true,
                    Ingresso::Mix,
                    CleaningLog::default(),
                ))
            } else {
                Box::new(Bypass)
            };
            let label = format!("{name} pulizia={cleaning}");
            let mut vad = Silero::new(&root.join("resources/silero_vad.onnx")).unwrap();
            let mut segmenter = Segmenter::new(Params::default());
            let mut evidence = Vec::new();
            let mut starts = Vec::new();
            for feed in FileFrames::with_processor(
                Decoder::open(&corpus.join(format!("{name}.wav"))).unwrap(),
                None,
                processor,
            ) {
                if let Feed::Frame(frame) = feed.unwrap() {
                    let p = vad.probability(&frame).unwrap();
                    evidence.push((p, 0.0));
                    for event in segmenter.push(frame, p) {
                        match event {
                            Event::PhraseStart(s) => starts.push(s),
                            Event::PhraseEnd => {
                                report(&label, *starts.last().unwrap(), evidence.len(), &evidence)
                            }
                            Event::Audio(_) => {}
                        }
                    }
                }
            }
            if !segmenter.close_phrase().is_empty() {
                report(&label, *starts.last().unwrap(), evidence.len(), &evidence);
            }
            let all: Vec<f32> = evidence.iter().map(|e| e.0).collect();
            println!(
                "SONDA {label}: frames={} candidates={} file_peak={:.6} file_mean={:.6}",
                all.len(),
                starts.len(),
                all.iter().copied().fold(0.0_f32, f32::max),
                all.iter().sum::<f32>() / all.len() as f32
            );
        }
    }
}
```

I log completi restano in locale, non versionati: `.scratch/alleggerire-installer/log/`
(`misura-v4.log`, `corpus-v4.log`, `sonda-v4.log`).
