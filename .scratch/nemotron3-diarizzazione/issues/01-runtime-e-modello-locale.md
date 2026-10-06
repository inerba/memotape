# 01: Verificare il runtime sperimentale e il modello locale

**What to build:** La prova dispone di un runtime e di un modello locale riproducibili, verificati su Windows prima di esporre Nemotron Diarization nell'app. I tre modelli di Trascrizione già presenti continuano a funzionare.

**Blocked by:** None (can start immediately).

**Status:** done

**Modello consigliato:** GPT-6 Astra (`gpt-6-astra`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Compatibilità fra runtime sperimentale, binding e DLL, conversione del modello e regressioni native da diagnosticare.

- [x] Runtime nativo, crate Rust/sys e DLL sono fissati al commit e6672a8672913b47f1571c66c54bee789d028416 della PR #175; non si mescolano binding e DLL di release diverse.
- [x] Il BF16 viene prodotto dal checkpoint NVIDIA f667ed73aee57d40cc39428eb768b4fd87a0a29e con il converter dello stesso commit; origine, hash dell'ingresso, ambiente/comando di conversione, dimensione e SHA-256 dell'uscita sono registrati.
- [x] Un'esecuzione nativa verifica caricamento e inferenza offline e streaming del modello su Windows x64, sia con Vulkan sia con CPU; se un backend non funziona, il limite è riportato e il ticket non dichiara la prova superata.
- [x] Le Trascrizioni con Nemotron ASR, Parakeet e Whisper conservano il comportamento richiesto, compresi i Parziali di Nemotron e la Lingua del parlato; Sortformer continua a funzionare.
- [x] Sono accertate le capacità dei tempi token/segmento nel commit scelto, senza promettere una precisione che il runtime non espone.
- [x] Se il commit o il modello risultano incompatibili, si documenta l'impedimento; non si cambia automaticamente runtime o GGUF. Nessun peso viene pubblicato e nessun download pubblico viene aggiunto.
- [x] I sei controlli del repository passano; le compilazioni Rust sono sequenziali. Gli smoke con modelli reali sono sequenziali e sono distinti dai test del core.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.

## Esito del 2026-10-05

Implementato nel worktree `a918`, su richiesta dell'utente. Il percorso fornito in
`drafts` era stato spostato in `issues` nel worktree sorgente `0acd`; questa copia
registra la chiusura insieme al codice del worktree di implementazione.

Runtime e binding fissati nei manifest/lock dell'app e del probe; modello BF16
locale di 198937280 byte, SHA-256
`4b11ce10e009fedf496cc9f879dc634e67605ecbf240463a155e0128657019e3`.
Due conversioni identiche, 20 smoke nativi CPU/Vulkan riusciti, sei controlli
verdi (89 test frontend, 192 Rust), cinque test del probe e due smoke integrati dell'app riusciti,
eseguiti separatamente e in sequenza. Le 13 DLL native preparate per l'app
coincidono con quelle provate; i quattro pesi esistenti coincidono con gli hash
del catalogo. Non sono stati pubblicati pesi o aggiunti download all'app.
La matrice è stata ripetuta dopo la revisione con confronto del testo ASR
completo, controllo della lingua, alternanza/copertura/limiti dei due Parlanti
e verifica automatica di dimensioni e SHA-256 di tutti i modelli e delle fixture.
Revisione Standards/Spec conclusa: nessun rilievo residuo.

Comandi, ambiente, hash, risultati e limiti dei timestamp nel
[resoconto riproducibile](../../../docs/research/nemotron3-runtime.md).
Qualità sull'italiano reale, ritardo nell'app e collegamento dei Parlanti dal vivo
restano nei ticket successivi: la prova qui è del runtime.

