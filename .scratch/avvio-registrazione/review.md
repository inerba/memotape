# Review dell'avvio della Registrazione

Data: 2026-10-07. Due revisioni indipendenti sul delta in `delta.patch`,
rispetto alla copia pre-implementazione `baseline/`, non rispetto a HEAD.
HEAD iniziale: a41ca57b912d6f9a82aee01d54912e8d23910424; branch main.
Il confronto con HEAD mescolerebbe lavoro preesistente e questo ticket.
Spec: `spec.md`; standard: AGENTS.md, DESIGN.md, CONTEXT.md e ADR-0026.

## Standards

Revisione conclusiva: nessuna violazione documentata o smell azionabile.
Pannello coerente con dock e token esistenti, movimento ridotto,
accessibilità e controlli nativi. Test sulle funzioni pure; fixture JSX
fuori da src. Lease DFN3 esclusivi e reset dello stato fra sessioni.
Bindings generati e sei localizzazioni coerenti.

Corretti durante la revisione: collocazione del nuovo probe JSX,
documenti rimasti alla fase di proposta, codifica dei commenti Rust.
Esclusi dai rilievi gli spazi finali nel file generato da Specta,
confermato dal test del generatore.

## Spec

Revisione conclusiva: nessun requisito implementato in modo errato,
nessuna omissione aggiuntiva o ampliamento del perimetro.
Readiness dopo Capture/Mixer/writer, Annulla prima dell'avvio,
Stop dopo, audio indipendente dal testo, preparazione e riuso DFN3,
attivazione futura valida solo nella sessione e guasti espliciti.

Corretti durante la revisione: coda ASR senza limite, attivazione pendente
dopo Stop, indicazione obsoleta dopo OFF, errore ASR anticipato nascosto,
eventi di fase dopo la conclusione. L'ultimo caso OFF/ON mentre DFN3 carica
ora riemette l'attesa anche se i cambi avvengono fra due blocchi PCM;
regressione verificata da un test dedicato.

Rilievi residui: Standards 0; Spec 0. Evidenze e limiti in `verifica.md`:
il collaudo reale Tauri/WASAPI e la latenza complessiva restano da misurare.
