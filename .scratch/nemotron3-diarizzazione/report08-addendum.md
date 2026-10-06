# Ticket 08 — addendum con prove create ed eseguite

6 ottobre 2026. **Ready-for-human, non done.** L’invito «Inventale tu queste prove» ha autorizzato la costruzione degli esperimenti, senza inventare risultati. Cinque confronti nativi finiti sono ora eseguiti; rimangono i limiti delle annotazioni, il collaudo UI e le Registrazioni lunghe. Il fallimento realtime CPU della prima fase resta valido; Sortformer resta predefinito.

## Corpus e riferimento

`corpus08/corpus-manifest.json`, `public-sources.json`, `cases/*/manifest.json`: due campioni Windows TTS e tre montaggi di letture italiane pubbliche. Otto archivi VoxForge, 41.445.579 byte complessivi, senza dataset completo. README per archivio dichiara Language IT e username distinto: codex, wperw, Grigomax, rrobotics, marianomarini, DavideMiccich, remix_tj, OscarCappa. LICENSE di tutti verificata GPL-3.0-or-later, coerente con [VoxForge italiano](https://www.voxforge.org/it); URL primarie di ciascun archivio/pagina, hash/byte/licenza/codec sono nel manifest. Download HTTPS da repository.voxforge1.org con TLS valido. Nessun audio locale caricato su servizi.

Sono otto **SpeakerID dichiarati dalla sorgente**, non prova assoluta di otto persone fisiche. Tutti i contributori selezionati sono maschi, con dispositivi/ambienti diversi; montaggio di letture e tagli fino a 4,5 s non equivalgono a conversazione spontanea. TTS: Elsa e Cosimo italiani, Zira en-US pronuncia italiano con accento. Elsa Desktop/OneCore non contate come persone diverse; niente pitch/rate per fabbricare identità. Sintesi locale pwsh 7.6.5; il lettore JSON ora specifica UTF-8 anche per ripetizioni su Windows PowerShell 5.

Audio finale PCM mono 16 kHz/16 bit, ricampionamento ffmpeg, guadagno noto per stem, nessun clipping, seed 8082026, silenzio di 4 s a metà. Condizioni difficili aggiungono sovrapposizioni di 0,85 s, tagli rapidi da 2,25 s e rumore gaussiano RMS 0,006. Stem emessi, crop/start/end in campioni e hash conservati. Prima dei diarizer si rileva attività **sugli stem puliti**, frame RMS 10 ms, padding 20 ms, merge gap 120 ms; tre soglie −55/−45/−35 dB. RTTM include tutte le 2/3/2/4/8 identità rispettivamente a tutte le soglie.

**Reference costruita/proxy energetico indipendente, non gold manuale.** Non copre la certezza dei confini speech/silence o la corretta identità fisica. I silenzi TTS non sono etichettati come intero parlato. Waveform TTS2/human8 generate e visualizzate (`signal.png`), con pausa evidente e picchi verificati sotto clipping; nessun ascolto umano attestato. Audit indipendente del coordinatore: audio/frame/hash/stem/codec/intervalli/identità/misure consistenti. Archivi, audio, baseline e risultati locali protetti da `.gitignore` nelle loro cartelle; nessun peso o dato pubblicato.

## Misure effettive

Stesso hardware/runtime/modelli/hash della prima fase. Cinque processi nativi sequenziali, Vulkan0, ASR italiano riconosciuta una volta per caso, identici PCM/Ingresso mix/testo/tempi per entrambi i diarizer; offline `VeryHighLatency` Nemotron. `scores-vulkan.json`: cinque casi completi, zero mancanti, manual_gold=false. Hash del frozen-ASR e hash canonico testo/tempi delle due ipotesi identici in ogni caso e soglia. Non sono misure realtime o rendering.

DER %, collar zero, overlap incluso, intera durata; celle mostrano −55 / **−45 principale** / −35 dB:

| Caso | s / identità | Sortformer DER % | Nemotron DER % |
|---|---|---|---|
| TTS2 clean | 60 / 2 | 4,77 / **5,48** / 6,82 | 0,78 / **1,56** / 3,08 |
| TTS3 overlap/rumore | 60 / 3 | 5,39 / **6,33** / 7,97 | 1,36 / **1,25** / 2,66 |
| Human2 clean | 83,85 / 2 | 18,62 / **9,30** / 12,30 | 16,36 / **6,48** / 8,94 |
| Human4 overlap | 81,30 / 4 | 21,23 / **10,14** / 7,54 | 18,68 / **8,03** / 5,85 |
| Human8 rapido/overlap/rumore | 72,94 / 8 | 61,09 / **57,06** / 55,47 | 34,81 / **28,22** / 26,39 |

La sensibilità è sostanziale: il proxy non permette di trattare questi valori come DER gold sul parlato spontaneo. Nemotron Human8 produce **sette etichette**, non otto: a −45 dB 13,09 speaker-secondi mancati, 0,08 falsi allarmi, 3,10 confusione, denominatore 57,66 speaker-secondi. Risultato critico anche con DER inferiore a Sortformer; il limite Sortformer di quattro voci è dichiarato e questo scenario non è una sua regressione.

Attribuzione del testo alla reference −45 dB, soltanto unità temporizzate; C/E/N = corrette/errate/non determinate. Byte UTF-8 indicano copertura temporizzata, non percentuale di parole corrette:

| Caso | Byte temporizzati/totali; Frasi senza tempi | Unità totali/valutabili/ambigue | Sortformer C/E/N | Nemotron C/E/N |
|---|---|---|---|---|
| TTS2 | 351/699; 2/4 | 53/49/4 | 25/24/0 | 49/0/0 |
| TTS3 | 589/589; 0/4 | 90/78/12 | 27/51/0 | 78/0/0 |
| Human2 | 719/949; 3/13 | 123/112/11 | 83/29/0 | 112/0/0 |
| Human4 | 324/875; 6/11 | 53/50/3 | 41/9/0 | 50/0/0 |
| Human8 | 386/685; 5/12 | 61/55/6 | 45/10/0 | 49/1/5 |

Tutte le unità fuori regione sono zero. La reference delle unità viene dai tempi e dagli stem, mai dalle etichette ipotizzate: una sola identità attiva, copertura almeno 25%; cambi/overlap/supporto insufficiente restano null. Queste unità sono contate ambigue, escluse dal denominatore. WER non misurato. Risultati perfetti nei subset non significano tutto il testo corretto o attribuito: Human4 valuta appena 324/875 byte. Sortformer assegna una voce all’intera Frase, Nemotron può dividerla ai tempi ASR; perciò DER con poca confusione acustica può convivere con errori di attribuzione delle parole. Le metriche restano separate.

## Correzione del banco e integrazione nativa

Review Standards ha scoperto il namespace raw RTTM diverso dai Parlanti rinumerati nella proiezione. Corretto senza cambiare prodotto: test Rust richiama `assign`/`assign_unambiguous` di produzione con ancore fuori dalla timeline, esporta projected→raw e verifica replay delle **dieci proiezioni identico agli output originali**, senza reference. Lo scorer compone questa mappa con raw→reference del DER. Regression raw3↔4 con tempi invariati e test Python indipendente verificano l’invarianza. Solo Human4 Sortformer cambia da 22 corrette/28 errate a **41/9**; quei numeri precedenti sono ritirati. Altri conteggi e tutti DER invariati. Audit obsoleto conservato in `obsolete-before-namespace-fix`; usare solo `scores-vulkan.json` e `cases/*/comparison-vulkan/scores.json` attuali.

`native08-corpus-tape.log`: vero writer Opus/Tape/read Document/open_tape/player core sul risultato Human8, 20 Frasi, testo/audio/unknown preservati. `native08-corpus-flow.log`: vero Cancelled del runtime passato al finalizzatore, correzione/rinomina/player Range/copie Testo e Markdown nelle sei lingue, unknown tradotto nelle 12 uscite, provvisorietà e Tape v1 senza riscrittura; passato. Queste sono integrazioni core/manager con file reali, non clipboard OS/playback ascoltato/UI Windows. Browser/IAB non fornisce il bridge della finestra Tauri; non sono stati introdotti mock del bridge per certificare il prodotto.

## Riproducibilità e stato finale

Strumenti: `tools/nemotron3-benchmark/{corpus.py,synthesize.ps1,compare-corpus.ps1,score_corpus.py,test_corpus.py}` e README. Cartelle di caso/output esistenti rifiutate; stem, raw RTTM, ASR e namespace conservati. `generator-at-build.py` fotografa il generatore usato, prima delle sole correzioni di metadati URL e guardia duplicati tar. Estrazione tar valida tutti i membri prima di scrivere e rifiuta traversal, link, special files, duplicati e target esistenti.

Sei controlli ripetuti verdi: typecheck; 106 frontend; Biome 160 file; fmt; clippy; **235 Rust, 12 ignored**, con i nuovi ignored eseguiti esplicitamente. **14 test Python verdi**, compresa composizione namespace. Log `checks08-extension-*.log`, `metrics08-extension-tests.log`, `namespace08-regression.log`, `native08-corpus-namespace.log` e `corpus08-{build,compare,scores}.log`. Le due riconferme finali Standards/Spec si sono interrotte per capacità del modello: **review finale non conclusa**. Findings corrette e coperte da replay nativo e regression; il coordinatore ha verificato i log dei controlli. Dettagli in `review-08.md`; baseline pre-estensione 204 file verificata, diff `08-extension.diff`.

Restano: conversazioni reali con confini e identità annotati manualmente, rendering UI e dispositivi reali, Registrazioni lunghe e Annulla durante l’orchestrazione del secondo Ingresso. Non si chiude done e non si promuove Nemotron automaticamente. Nessun commit/push/reset/configurazione globale, upload, installer o pubblicazione.
