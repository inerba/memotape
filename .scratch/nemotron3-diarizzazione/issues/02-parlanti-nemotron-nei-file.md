# 02: Scegliere Nemotron e riconoscere i Parlanti nei file

**What to build:** L'utente sceglie Nemotron Diarization sperimentale nelle Impostazioni e trascrive un file o un Tape con fino a 8 Parlanti per Ingresso. Il risultato si legge, si riapre e si esporta; Sortformer resta disponibile.

**Blocked by:** 01 — Verificare il runtime sperimentale e il modello locale.

**Status:** done

**Modello consigliato:** GPT-6.1 Sol (`gpt-6.1-sol`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Integrazione circoscritta di scelta del modello, Trascrizione dei file, persistenza e interfaccia su un runtime già verificato.

- [x] La selezione del diarizer è distinta dal modello di Trascrizione e viene ricordata; gli utenti esistenti conservano Sortformer come scelta iniziale. Non si cancellano i modelli già presenti.
- [x] L'artefatto locale verificato nel ticket 01 viene riconosciuto e può essere usato; la sua assenza o incompatibilità produce un avviso esplicito senza sostituzione silenziosa del modello e senza URL di download inventati.
- [x] Trascrivi su un file audio/video e Trascrivi di nuovo su un Tape passano attraverso il modello selezionato. La conferma prima di sostituire testo, correzioni e nomi resta quella esistente.
- [x] Le Frasi a voce unica ricevono il Parlante coerente; se una Frase contiene più voci non separabili con i tempi disponibili, resta Parlante non determinato. La divisione precisa viene completata dal ticket 04.
- [x] Il risultato supporta fino a 8 Parlanti per Ingresso, con numerazione per comparsa e identità separate; un Tape con le tracce degli Ingressi conserva la regola dell'Audio di sistema e del Microfono prevista dalla spec.
- [x] Il Tape viene salvato e riaperto senza perdere testo, audio, nomi, tempi o Forma d'onda; lettura, Copia testo, Markdown e player concordano. I Tape vecchi restano leggibili senza riscrittura in massa.
- [x] Riconosci i parlanti spento e Sortformer selezionato conservano i loro flussi attuali; tutti i testi visibili sono presenti nelle sei lingue.
- [x] Test significativi coprono scelta, modello assente, attribuzioni e riapertura. I sei controlli del repository passano; il comportamento nativo è verificato separatamente.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.

## Comments

### Chiusura del 2026-10-05

Implementato nel worktree `5e50` su richiesta dell'utente. Il ticket richiesto in
`drafts` si trova in `issues` nel worktree sorgente `0acd`; questa copia registra
l'esito insieme al codice. Importato il prerequisito 01 già chiuso nel worktree
`a918`: dipendenza fissata, probe e resoconto riproducibile del runtime. Il GGUF
rimane fuori dal repository, senza download pubblico aggiunto.

La scelta dei Parlanti è separata dall'ASR e persistente, con Sortformer per le
impostazioni precedenti. Il selettore locale accetta solo l'artefatto BF16
verificato nel ticket 01 (198937280 byte, SHA-256
`4b11ce10e009fedf496cc9f879dc634e67605ecbf240463a155e0128657019e3`).
Assenza o incompatibilità danno errori dedicati, senza fallback. La scelta vale
per file audio/video e ritrascrizione di Tape; le Registrazioni usano ancora
Sortformer, come dichiarato nelle Impostazioni.

Nemotron attribuisce una Frase solo se i turni sovrapposti appartengono a una
voce unica. Negli altri casi conserva testo e tempi con il campo facoltativo
`parlante_non_determinato`, leggibile anche nel documento v1. Vista, copia,
Markdown e MCP mostrano la dicitura tradotta; la ricerca esclude i nomi delle
Frasi non determinate. Numerazione per comparsa nell'audio, identità separate
per Ingresso, regole esistenti per Microfono e Audio di sistema conservate.
La divisione del testo con i tempi delle parole resta nel ticket 04.

Sei controlli verdi sulla versione finale:

- `bun run typecheck`;
- `bun run test`: 92 test superati;
- `bun run check`;
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml`;
- `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`;
- `cargo test --locked --manifest-path src-tauri/Cargo.toml`: 199 superati, 3 smoke ignorati nella suite ordinaria.

Smoke nativo eseguito separatamente e in sequenza, con Nemotron ASR e Nemotron 3
Diarization reali su Windows x64/Vulkan (RTX 2070 SUPER):
`MEMOTAPE_NEMOTRON3_MODEL` impostato al GGUF locale, poi
`cargo test --locked --manifest-path src-tauri/Cargo.toml nemotron3_trascrive_un_file -- --ignored --test-threads=1 --nocapture`.
Riuscito anche dopo le correzioni della revisione: fixture sintetica a due voci,
numeri 1/2, creazione e riapertura del Tape, audio, Forma d'onda e Markdown.
La verifica di 8 identità e degli Ingressi separati è nei test del core; non
dimostra qualità su 8 voci reali. Interfaccia nativa, qualità sull'italiano reale
e confronto con Sortformer non sono stati verificati manualmente in questo ticket.

#### Standards

Revisione separata conclusa con 0 violazioni documentate e 0 odori rilevanti.
Rimossa la duplicazione nell'assegnazione per Ingresso e adeguato il nome del
helper al glossario (`assign_to_ingresso`).

#### Spec

Revisione separata conclusa con 0 rilievi residui. Corretti i due lettori che
ignoravano il nuovo stato (MCP `read_around` e indice FTS), con un test integrato
di regressione verificato prima rosso e poi verde.

Nessun commit o pubblicazione: resta necessaria l'autorizzazione successiva
prevista dalla spec.

