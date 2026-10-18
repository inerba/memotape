# Riverifica della protezione con Silero v6.2 (transcribe-cpp, frame da 32 ms)

Ticket 03. È il confronto con `baseline-v4.md` del ticket 01, fatto sullo stesso corpus, con gli
stessi comandi e gli stessi modelli. Eseguita il 18 ottobre 2026.

## Codice, corpus e modelli

- Codice: commit `2602fa37` (ticket 02), senza modifiche. Rust 1.96.1, profilo `release`.
- VAD: Silero v6.2 su transcribe-cpp 0.4.0 (`VadSession`, backend CPU), con il file
  `src-tauri/resources/silero-vad-v6.2-F32.gguf` (1 241 056 byte, SHA-256
  `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`). Frame da 512 campioni.
  Segmentatore con `Params::default()`, come nel ticket 01: prefill 300 ms (10 frame), onset 60 ms
  (2), hangover 700 ms (22), massimo 18 s (563), soglia 0,4.
- Pulizia, corpus e ASR come nel ticket 01. I WAV sono stati copiati nel worktree dal checkout
  principale, con gli SHA-256 uguali a quelli di `manifest.json`. Gli ASR sono
  `nemotron-3.5-streaming-0.6b-q5km`, `cohere-transcribe-03-2026-q5km`, `whisper-medium-q8` e
  `parakeet-tdt-0.6b-v3-q5km`.

## Comandi ed esito

In `src-tauri`, uno dopo l'altro, come nel ticket 01:

```
cargo test --release --lib -- --ignored --test-threads=1 --nocapture misura_evidenza_corpus_protezione
cargo test --release --lib -- --ignored --test-threads=1 --nocapture corpus_protezione_quattro_livelli_tre_asr
cargo test --release --lib -- --ignored --test-threads=1 --nocapture sonda_probabilita_con_pulizia
```

- **Misura dell'evidenza:** passa.
- **Corpus:**
  - Come nel ticket 01, supera le asserzioni di regressione: Spento, Sensibile e Bilanciato hanno le stesse candidate sulla voce attenuata e su «Sì. No.», con e senza pulizia.
  - Stampa le 576 righe dei quattro ASR, poi si ferma su `ModelMissing` di `parakeet-primeline`, come nel ticket 01. `matrix.json` non viene riscritto.
  - L'asserzione finale `false_balanced < false_off` non sarebbe passata: 0 < 0. Il ticket 03 la sostituisce con «Bilanciato non tiene false Frasi», vera con v4 e con v6.2.
- **Sonda:** è quella di `baseline-v4.md`, con il solo `Silero::new` cambiato. È stata aggiunta, eseguita e tolta.

I tre profili danno righe identiche anche con v6.2: zero differenze.

## Frasi candidate e tenute, v4 → v6.2

Il formato è «senza pulizia / con pulizia».

| Campione | Spento | Sensibile | Bilanciato | Selettivo |
| --- | --- | --- | --- | --- |
| silenzio | 0→0 / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 |
| rumore-continuo | 0→0 / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 |
| rumore-intermittente | **1→0** / 0→0 | **1→0** / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 |
| soffio-sintetico | 0→0 / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 | 0→0 / 0→0 |
| voce-attenuata-30db | **3→2** / 0→0 | **3→2** / 0→0 | **3→2** / 0→0 | **1→2** / 0→0 |
| si-no | 2→2 / 2→2 | 2→2 / 2→2 | 2→2 / 2→2 | 2→2 / 2→2 |

## Testo delle Frasi tenute con v6.2

| Campione, pulizia | Livelli | Nemotron | Cohere | Whisper medium | Parakeet |
| --- | --- | --- | --- | --- | --- |
| voce-attenuata-30db, no | tutti | — | — | — | — |
| si-no, no | tutti | «Sì. No.» | «C. No.» | «Sì. No.» | «C No» |
| si-no, sì | tutti | «Sì. No.» | «D. No.» | «Sì. No.» | «T No.» |

Differenze da v4:

- **False Frasi:** zero a ogni livello. Con v4 il rumore intermittente senza pulizia dava una candidata, che Cohere («Il resto è finito.») e Whisper medium («Sottotitoli creati dalla comunità Amara.org») trascrivevano a Spento e a Sensibile. Totale a Spento: 2 → 0; a Bilanciato: 0 → 0.
- **Risposte brevi:** le due candidate di «Sì. No.» restano a ogni livello, con e senza pulizia. Cambia il testo di Cohere senza pulizia: «Sì. No.» diventa «C. No.», un errore dell'ASR come quelli che Cohere e Parakeet facevano già con v4 («D.», «C», «T»). I confini delle due candidate si spostano di poco: 0..35 e 35..69 frame da 32 ms, contro 0..37 e 37..73 da 30 ms.
- **Voce attenuata, senza pulizia:** le candidate non si perdono più a Selettivo (2 a ogni livello). Si perdono invece le 3 parole che restavano con v4 («Buongiorno a tutti»): il testo è vuoto con tutti e quattro gli ASR e a ogni livello. Sotto, la causa.
- **Voce attenuata, con la pulizia:** nessuna candidata, come con v4.

## Perché la voce attenuata perde «Buongiorno a tutti»

La causa non sono le soglie della Sensibilità. È il filtro del rumore del motore
(`livello::rumore`, commit `430117bb`): una Frase con RMS sotto −50 dBFS non va al modello.

- Con v4 le candidate erano tre. Secondo il calcolo dell'RMS sui frame del WAV del corpus, solo la prima superava la soglia:

  | Candidata v4 (frame) | RMS |
  | --- | --- |
  | 6..43 | −49,7 dBFS |
  | 50..150 | −53,1 dBFS |
  | 182..299 | −52,3 dBFS |

  Quindi «Buongiorno a tutti» veniva solo dalla prima, e le altre 11 parole erano già perse.
- Con v6.2 la prima frase parlata è un tratto unico (dump per frame in `log/frame-v62.log`, sonda temporanea già tolta). La probabilità sta sopra 0,4 nei frame 6..42 e 48..108; nei frame 43–47 (160 ms) scende sotto 0,4, meno dell'hangover di 22 frame. Le prime due candidate di v4 diventano una sola (0..131, −52,0 dBFS) e il filtro scarta anche quella. La seconda candidata (170..280) è a −52,3 dBFS.
- Con v4 Silero vedeva solo l'inizio della frase: 3 frame sopra 0,4 nella candidata 6..43. A Selettivo, che la scartava, si perdevano proprio quelle 3 parole.
- Le soglie non separano le due parti, perché nella pausa la probabilità è già sotto 0,4. Servirebbe un hangover sotto i 160 ms, una durata che resta in millisecondi. Ogni candidata della voce attenuata ha già picco ≥ 0,98, quindi la protezione la ammette a ogni livello.
- La causa è dedotta dall'RMS: il test non ha un logger che stampi le Frasi di rumore.

## Probabilità di Silero v6.2

Media e picco sui frame con probabilità ≥ 0,4 di ogni candidata. Senza pulizia i valori vengono
dalla misura dell'evidenza, con la pulizia dalla sonda.

| Campione | Pulizia | Candidata (frame) | Frame ≥ 0,4 | Picco | Media |
| --- | --- | --- | --- | --- | --- |
| voce-attenuata-30db | no | 0..131 | 98 | 0,999993 | 0,972713 |
| voce-attenuata-30db | no | 170..280 | 79 | 0,999996 | 0,991383 |
| si-no | no | 0..35 | 9 | 0,999955 | 0,952669 |
| si-no | no | 35..69 | 10 | 0,999970 | 0,951086 |
| si-no | sì | 0..35 | 7 | 0,999689 | 0,922564 |
| si-no | sì | 35..69 | 10 | 0,999970 | 0,934257 |

Picco e media sull'intero file, senza e con pulizia; tra parentesi i valori di v4:

| Campione | Frame | Picco senza / con pulizia | Media senza / con pulizia |
| --- | --- | --- | --- |
| silenzio | 63 (67) | 0,008911 / 0,008911 (0,042866 / 0,042866) | 0,004053 / 0,004053 (0,020149 / 0,020149) |
| rumore-continuo | 63 (67) | 0,053366 / 0,008911 (0,215969 / 0,042866) | 0,012465 / 0,004053 (0,037986 / 0,020149) |
| rumore-intermittente | 63 (67) | 0,019977 / 0,008911 (0,878327 / 0,042866) | 0,005272 / 0,004053 (0,503498 / 0,020149) |
| soffio-sintetico | 63 (67) | 0,031838 / 0,034993 (0,051180 / 0,242118) | 0,006421 / 0,007453 (0,016716 / 0,030054) |
| voce-attenuata-30db | 280 (299) | 0,999996 / 0,008911 (0,775435 / 0,042866) | 0,629987 / 0,002346 (0,370512 / 0,016039) |
| si-no | 69 (74) | 0,999970 / 0,999970 (0,999219 / 0,992012) | 0,276294 / 0,239108 (0,248766 / 0,179791) |

v6.2 separa molto di più: sul rumore il picco resta sotto 0,06, sul parlato (anche a −30 dB)
sale sopra 0,9999. Con la pulizia il silenzio digitale vale 0,008911.

## Decisione sulle soglie

Le soglie restano.

- Le asserzioni di regressione del test passano: stesse candidate a Spento, Sensibile e Bilanciato sulla voce attenuata e su «Sì. No.».
- Le false Frasi scendono a zero a ogni livello, quindi Bilanciato non ha più niente da ridurre.
- Il ramo di ritaratura del criterio («finché Bilanciato riduce le false Frasi») non ha più un obiettivo: Spento non ne ha. Le soglie del segmentatore non riportano la voce attenuata (vedi sopra). Conseguenza: questo corpus non mette più alla prova lo scarto del rumore della protezione, e la nuova asserzione passerebbe anche con la protezione spenta.

## Correzione del filtro del rumore

Dopo la riverifica la Frase di rumore si misura sul quarto più forte dei frame da 32 ms, non più
sull'intera Frase. La soglia resta −50 dBFS. Prima la misura veniva da una sonda temporanea
(`log/livello-sonda.log`, già tolta) sul corpus e sui due Tape reali della ricerca sull'eco,
copiati nello scratchpad: le copie hanno lo stesso SHA-256 degli originali.

| Frasi | RMS intero | Quarto più forte |
| --- | --- | --- |
| voce-attenuata-30db, candidate 0..131 e 170..280 | −52,0 / −52,3 | −47,1 / −47,7 |
| si-no, candidate 0..35 e 35..69 | −31,9 / −29,9 | −26,0 / −24,2 |
| Microfono muto di 15-11-13, le 12 Frasi della tabella A della ricerca (tagliate da v4) | da −58,7 a −75,8 | da −52,7 a −71,1 |
| Microfono muto di 15-11-13, le 2 Frasi tagliate da v6.2 (in 15-04-07 nessuna) | −73,0 / −77,8 | −67,6 / −72,3 |
| Audio di sistema dei due Tape, 26 Frasi | da −45,3 a −14,5 | da −39,5 a −9,2 |

Altri criteri misurati e scartati:

- **Frame più forte:** non separa, perché nel muto ci sono frame fino a −42,3 dBFS.
- **10% più forte:** la voce attenuata sta a −45,2 e il muto, su finestre corte, fino a −43,6.
- **50% più forte:** la voce attenuata resta a 0,6 dB dalla soglia.
- **RMS senza i frame muti:** la voce attenuata resta sotto, a −50,6.

Sulle finestre scorrevoli dell'intero Microfono muto, il quarto più forte vale al massimo:

| Finestra | Quarto più forte, massimo |
| --- | --- |
| 0,5 s | −46,9 / −46,4 |
| 1 s | −49,5 / −49,1 |
| 2 s | −52,0 / −51,9 |
| 3 s | −53,7 / −53,6 |

Una Frase dura almeno circa 1,1 s (prefill, onset e hangover).

Il test del corpus è stato rilanciato in release con lo stesso comando (`log/corpus-v62-filtro.log`;
si ferma ancora su `parakeet-primeline`):

- **Voce attenuata senza pulizia:** a ogni livello, Selettivo compreso, e con tutti e quattro gli ASR esce «Buongiorno a tutti, oggi parliamo di trascrizione. Il computer trasforma la voce in testo.», cioè 14 parole su 14 (v4: 3; v6.2 con il filtro vecchio: 0).
- **False Frasi:** zero a ogni livello.
- **«Sì. No.»:** uguale a v6.2 con il filtro vecchio («C. No.» di Cohere e «C No» di Parakeet senza pulizia), con le stesse candidate.
- **Le altre righe:** identiche.

## Log

Sono in locale e non versionati, in `.scratch/alleggerire-installer/log/`: `misura-v62.log`,
`corpus-v62.log`, `sonda-v62.log`, `frame-v62.log`, `livello-sonda.log`, `corpus-v62-filtro.log`, accanto ai `*-v4.log` del ticket 01.
