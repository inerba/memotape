# 07: Diarizzazione dei file

**What to build:** l'utente scarica il modello di diarizzazione dalle Impostazioni, attiva "Riconosci i parlanti" accanto a Trascrivi e trascrive un file. Le Frasi vengono attribuite a Parlanti numerati per ordine di comparsa, sia nell'area di testo sia nel Markdown.

**Blocked by:** 03

**Status:** done

- [x] Il catalogo ha un tipo `diarizzazione` con Sortformer 4spk v2.1 Q8_0 (URL, SHA e dimensione nella spec, sezione Diarizzazione), con download, verifica, ripresa ed Elimina esistenti, mostrato separato dai modelli di trascrizione. La licenza va in Informazioni
- [x] Casella "Riconosci i parlanti" accanto a Trascrivi (`parlanti_file`, salvata), con la nota "al massimo 4 Parlanti"
- [x] Dopo la Trascrizione, nella stessa Attività: Sortformer con `run` e diarizzazione sull'audio a 16 kHz. I segmenti si riordinano per tempo. Annulla usa il `CancelToken`
- [x] Funzione pura di attribuzione: a ogni Frase il Parlante con la sovrapposizione maggiore, rinumerazione per ordine di comparsa. Evento `speakers-assigned`
- [x] Area di testo e `.md` con i turni `**Parlante N:**`
- [x] Modello di diarizzazione non scaricato: errore "modello non scaricato" con il link alle Impostazioni
- [x] Test di attribuzione (sovrapposizioni parziali, Frase senza segmenti, segmenti non ordinati) e smoke test `#[ignore]` con Sortformer vero su una fixture a due voci TTS diverse, che deve trovare 2 Parlanti
- [x] `docs/research/diarizzazione.md`: leggi la sezione su Sortformer (API `Diarize`, `speaker_segments`, limiti)

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.

## Esito

Verificato in `bun tauri dev` il 2026-10-03 (RTX 2070 SUPER, Nemotron, Lingua del parlato Automatica), su una copia di `parlato-due-voci.wav` fuori dal repo:
- con la casella attiva e Sortformer assente, `transcribe` rispondeva subito `modelMissing` "Sortformer 4spk v2.1" (dopo la code review è il codice dedicato `diarizerMissing`);
- Sortformer scaricato con `download_model` (verifica SHA-256 compresa) compare in Impostazioni → Trascrizione sotto "Riconoscimento dei parlanti", senza scelta, e in Informazioni con la licenza e la quantizzazione Q8_0;
- Trascrivi: avanzamento fino al 97%, poi "Riconoscimento dei parlanti…", poi l'area con quattro turni alternati "Parlante 1:" e "Parlante 2:" come le due voci, e lo stesso nel `.md`. La prima Diarizzazione del processo dura 9,3 s (riscaldamento di Vulkan), la seconda 0,5 s;
- su 8,5 minuti (la fixture ripetuta 20 volte) la Diarizzazione dura 3 s; Annulla premuto durante "Riconoscimento dei parlanti…" ferma Sortformer in meno di un secondo, con "Trascrizione annullata" e nessun Markdown.

Lo smoke test `sortformer_trova_due_parlanti_che_si_alternano` (`cargo test -- --ignored`) trova 2 Parlanti, il primo su Elsa e l'ultimo su Cosimo. Nemotron in Automatica scrive a volte token grezzi (`<sl-SI>`, `<unk>`) sulla voce di Cosimo: è un comportamento del modello, non della Diarizzazione.
