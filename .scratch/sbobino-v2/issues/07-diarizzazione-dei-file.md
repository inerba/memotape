# 07: Diarizzazione dei file

**What to build:** l'utente scarica il modello di diarizzazione dalle Impostazioni, attiva "Riconosci i parlanti" accanto a Trascrivi e trascrive un file. Le Frasi vengono attribuite a Parlanti numerati per ordine di comparsa, sia nell'area di testo sia nel Markdown.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Il catalogo ha un tipo `diarizzazione` con Sortformer 4spk v2.1 Q8_0 (URL, SHA e dimensione nella spec, sezione Diarizzazione), con download, verifica, ripresa ed Elimina esistenti, mostrato separato dai modelli di trascrizione. La licenza va in Informazioni
- [ ] Casella "Riconosci i parlanti" accanto a Trascrivi (`parlanti_file`, salvata), con la nota "al massimo 4 Parlanti"
- [ ] Dopo la Trascrizione, nella stessa Attività: Sortformer con `run` e diarizzazione sull'audio a 16 kHz. I segmenti si riordinano per tempo. Annulla usa il `CancelToken`
- [ ] Funzione pura di attribuzione: a ogni Frase il Parlante con la sovrapposizione maggiore, rinumerazione per ordine di comparsa. Evento `speakers-assigned`
- [ ] Area di testo e `.md` con i turni `**Parlante N:**`
- [ ] Modello di diarizzazione non scaricato: errore "modello non scaricato" con il link alle Impostazioni
- [ ] Test di attribuzione (sovrapposizioni parziali, Frase senza segmenti, segmenti non ordinati) e smoke test `#[ignore]` con Sortformer vero su una fixture a due voci TTS diverse, che deve trovare 2 Parlanti
- [ ] `docs/research/diarizzazione.md`: leggi la sezione su Sortformer (API `Diarize`, `speaker_segments`, limiti)

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
