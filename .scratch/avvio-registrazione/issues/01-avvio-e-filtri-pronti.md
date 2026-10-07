# Avvio confermato della Registrazione e filtri pronti

Type: task
Status: resolved

Spec: ../spec.md. Decisione: ../../../docs/adr/0026-preparazione-della-registrazione.md.

## Implementazione

Barra di preparazione dal clic e attesa dei salvataggi, Annulla immediato prima
IPC, comando Annulla con UUID prima dei controlli nativi, readiness dopo
Ingressi e writer, conferma visiva e timer. Audio indipendente dal caricamento
ASR, filtri DFN3 preparati in background e riutilizzabili con stato nuovo.
Pulizia richiesta all'avvio attesa, pending al volo con PCM originale,
attivazione futura valida solo per la sessione, guasti espliciti e bypass.
Coda ASR limitata e non bloccante. Sei lingue e movimento ridotto.

Il delta rispetto allo stato iniziale del checkout è in ../delta.patch;
../changed-files.txt elenca i sorgenti del ticket. Non comprende le modifiche
preesistenti dell'utente in altri file. Evidenze e limiti in ../verifica.md.

## Comments

- 2026-10-07: Implementazione autorizzata con `$implement`. TDD alle seam
  di readiness, salvataggi, annullamento e AudioProcessor. Review Standards
  e Spec separate; rilievi corretti prima della chiusura.

## Answer

Implementazione completata nel checkout. Sei controlli verdi: 134 test
frontend, 304 test Rust (35 ignored); build frontend finale riuscita.
Benchmark DFN3 reale e probe UI con IPC simulato eseguiti; collaudo
Tauri/WASAPI e misure dal clic alla prima parola ancora aperti e dichiarati
in [verifica](../verifica.md). Review separate concluse senza rilievi
residui: [report](../review.md). Nessun installer o commit creato.

- 2026-10-07: Chiusura dopo la correzione OFF/ON della notifica di pulizia
  pendente. Modifiche precedenti preservate; nessuna operazione Git di scrittura.
