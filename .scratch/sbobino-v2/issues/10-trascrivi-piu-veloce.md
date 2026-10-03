# 10: Trascrivi più veloce, testo a Trascrizione finita

**What to build:** Trascrivi su un file o su un Bino non mostra più il testo man mano: l'area resta vuota, la status bar ha la percentuale e Annulla, e il testo compare tutto insieme alla fine. In cambio la Trascrizione è più veloce. Requisiti in `PRODUCT.md` (V7), decisione in ADR-0007, misure in `docs/research/trascrizione-file-veloce.md`. La Trascrizione dal vivo di una Registrazione non cambia: streaming, Parziali e Frasi man mano.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] **Nemotron con `run` sui file.** Su un file ogni Frase va al modello con `Session::run` anche con Nemotron, come già fanno Whisper e Parakeet; lo stream resta per la Trascrizione dal vivo. Oggi `TranscribeCpp` sceglie da `supports_streaming`: la scelta va presa da chi chiama (file o dal vivo).
- [x] **Decodifica e VAD in parallelo al motore** per i file. Oggi la pipeline a trazione li somma al tempo del motore, circa 6 s su 10 minuti. Un thread decodifica, passa per Silero e segmentatore e manda le Frasi intere al motore su un canale limitato (poche Frasi, ognuna al massimo 18 s, circa 1,1 MB), così la memoria non cresce col file. Devono restare uguali:
  - la percentuale, che può anticipare il motore al massimo della capacità del canale;
  - Annulla, entro una Frase;
  - i tempi delle Frasi;
  - l'audio per la Diarizzazione;
  - gli errori della fonte.

  La pipeline dal vivo resta a trazione: il motore in streaming deve ricevere la Frase mentre è in corso.
- [x] **Testo a Trascrizione finita** (frontend). Durante Trascrivi su un file l'area è vuota e non mostra Frasi né Parziali; a fine Trascrizione compare tutto il testo. Dopo Annulla l'area mostra le Frasi già trascritte. Copia testo è disabilitato durante la Trascrizione di un file, non durante la Registrazione. Gli eventi `transcript-phrase` possono restare: è il frontend a non mostrarli finché la Trascrizione di un file non finisce.
- [x] **Riscaldamento nel caricamento in background.** Dopo il caricamento del modello in `preload` (avvio, cambio di modello, fine download) si trascrive circa 1 s di silenzio e si scarta il risultato. Il modello resta `inUse` per quel tempo. La prima Trascrizione dopo l'avvio deve durare quanto le successive (con Whisper oggi 16 s contro 0,8 s sulla fixture).
- [x] Aggiornare `AGENTS.md` (pipeline, `TranscriptionEngine`, Parziali, "Prima Trascrizione con Whisper") e, se servono parole nuove, `CONTEXT.md`.

## Verifica

- [x] Sui 10 minuti di `D:\Video\2026-05-22 11-49-57.mp4` (Lingua del parlato `it`, release, modello già caldo), confrontati con la misura di `docs/research/trascrizione-file-veloce.md`:
  - Nemotron: Trascrivi passa da circa 93 s a circa 45 s o meno;
  - Parakeet e Whisper: circa 6 s in meno di oggi (19,6 s e 41,6 s).
- [x] Il testo non peggiora: le stesse Frasi, salvo differenze minime di Nemotron tra stream e `run`.
- [x] Annulla durante un file lungo ferma entro un secondo con tutti e tre i modelli. Il Markdown non si salva e l'area mostra le Frasi già pronte.
- [x] Riconosci i parlanti su un file funziona come prima. Non provato su un Bino: passa dallo stesso `transcribe_file`.
- [ ] La Trascrizione dal vivo (Mix e Ingressi separati) funziona come prima, con i Parziali di Nemotron. Coperta dai test della pipeline (`transcribe` con i Parziali), non provata nell'app con audio vero.
- [x] I sei controlli verdi.

## Comments

Verifica del 2026-10-04 (release, RTX 2070 SUPER, MP4 da 618 s, Lingua del parlato `it`):

| | prima | dopo |
| --- | --- | --- |
| Nemotron | 93,5 s | 31,6 s |
| Parakeet | 19,6 s | 15,1 s |
| Whisper | 41,6 s | 37,7 s |

- **Riscaldamento.** `TranscribeCpp::load` con il riscaldamento dura 0,7–1,3 s. Dopo, la prima Trascrizione della fixture dura quanto la seconda con tutti e tre i modelli (Whisper 0,70 s contro 0,72 s).
- **Annulla dopo 3 s.** La Trascrizione si ferma entro 0,16 s.
- **Testo.**
  - Parakeet: identico.
  - Nemotron: come la misura di `run` per Frase.
  - Whisper: 4 righe diverse su tratti di rumore, come tra due esecuzioni della stessa versione (non è deterministico).
- **Smoke test.** `i_tre_modelli_trascrivono_il_parlato_italiano` e `sortformer_trova_due_parlanti_che_si_alternano` sono verdi.
- **Nell'app (`bun tauri dev`, via CDP)**, con Trascrivi sul MP4 e Nemotron:
  - durante la Trascrizione l'area è vuota, Copia testo è disabilitato e la percentuale avanza;
  - Annulla ferma in meno di 0,4 s, l'area mostra le Frasi pronte e la status bar "Trascrizione annullata: nessun Markdown salvato";
  - Riconosci i parlanti su `parlato-due-voci.wav` mostra il testo solo alla fine, con "Parlante 1/2", e salva il Markdown.
- **Dopo la revisione.**
  - Il progresso esce dal canale (`Shared`, letto dal motore dopo ogni Frase e in attesa ogni 200 ms), così il canale tiene davvero fino a 4 Frasi.
  - Un guasto del motore ferma subito il thread (`Shared::stopped`).
  - `shownText` e `copyable` stanno in `features/transcription/phrases.ts`, con i test.
  - Il riscaldamento sta in `TranscribeCpp::load`, quindi vale per ogni caricamento, anche per l'istanza in più degli Ingressi separati.
- **Già presente prima.** Due modelli caricati insieme su Vulkan da thread diversi fanno cadere il processo. Gli smoke test vanno lanciati con `--test-threads=1`. Segnalato a parte.
- **Già presente prima.** Con Nemotron e la lingua automatica una Frase corta esce "Grazie. <sl-SI>", e il "%" diventa `<unk>`. Succede uguale con lo stream: transcribe-cpp toglie il tag di lingua solo se arriva come un pezzo unico. Segnalato a parte.
