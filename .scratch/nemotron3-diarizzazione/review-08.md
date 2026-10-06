# Revisione isolata ticket 08 — 6 ottobre 2026

Due agenti puliti GPT-6.1 Sol high, sola lettura, revisioni Standards e Spec parallele secondo skill code-review. Punto fisso: manifest SHA-256 dei 194 file nella baseline pre-08, non HEAD (che confonderebbe il lavoro 01–07 non committato). Diff `08.diff`, elenco `08-files.json`; rigenerati dopo il report finale. Nessun commit o ripristino.

## Standards

Nessuna violazione residua sostanziale delle convenzioni AGENTS/CONTEXT; modulo Windows soltanto di test, nessun cambiamento prodotto/binding. Findings di affidabilità degli strumenti corrette:

- Runner poteva scegliere un eseguibile senza il test e accettare zero test: controlla `--list`, identità/runtime/integrità del report e salute live. Binario main errato provato e rifiutato.
- Summarizer poteva omettere casi mancanti: richiede i quattro casi attesi, riporta assenti/in corso, `validReport: false`, exit/timeout e separa `matrix_complete` da `all_realtime_healthy`.

Ultima verifica read-only su report, summary300 e audit300: numeri coincidono; CPU negativo, denominatori assenti, callback, fixture sintetica, memoria host/overhead banco, GPU globale e requisiti mancanti sono espliciti. Nessun difetto residuo individuato nello scope verificato.

## Spec

- Scorer testo poteva includere unità fuori dalla regione DER: stessa regione esplicita, unità esterne/a cavallo escluse e contate; test indipendente aggiunto (otto test scorer verdi).
- La frase “riapertura identica” eccedeva l’assertion: limitata a “rilettura Document identica e stesso numero di Frasi alla riapertura”, senza inventare prove sui metadati manager.
- Annulla nel banco attraversa ora un vero errore nativo `Cancelled` passato al `finalize` di produzione; resta esplicitamente una prova core, con stato provvisorio preparato, non tutta l’orchestrazione Registrazione/UI.

Revisore ha confrontato latenze, conteggi, finali e p95 per blocchi con i JSON; nessuna promozione modello, scope aggiuntivo o certificazione UI. UI/prova reale 2–4/5–8 voci e Registrazioni lunghe sono requisiti aperti, non finding da mascherare. Stato corretto ready-for-human, non done.

Gli esiti degli smoke e dei sei controlli sono verificati dall’agente esecutore nei log indicati in report08.md; i revisori non hanno lanciato build/test né certificato la UI.

## Review dell’estensione corpus

Cinque confronti nativi completati; proxy degli stem distinto dal gold manuale, otto SpeakerID dichiarati distinti ma identità fisiche non certificate. L’audit del coordinatore su hash, codec, RTTM, unità, DER e denominatori è passato. La finding Standards sul namespace raw/proiezione è corretta attraverso le funzioni di produzione e la regression sulla permutazione: i vecchi punteggi sono conservati ma ritirati; Human4 Sortformer ha 41 corrette, 9 errate e 0 non determinate su 50 unità. Il replay delle dieci proiezioni è identico, con DER invariato. L’osservazione Spec su UTF-8 è corretta nella lettura dei jobs; la sintesi era già stata eseguita con pwsh 7.6.5.

Le due riconferme finali Standards e Spec dopo queste correzioni **si sono interrotte per capacità del modello**: non sono revisioni finali concluse. La correzione è coperta da replay nativo e regression indipendenti; il coordinatore ha verificato i log dei sei controlli, con 235 test Rust e 14 Python verdi. Nessuna attestazione UI. Stato ready-for-human.
