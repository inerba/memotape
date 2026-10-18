# 05: Misura di LTO sull'exe

**What to build:** La decisione, basata su una misura, se il profilo release usa `lto = true` e `codegen-units = 1`. Spec: `../spec.md`, sezione "LTO".

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] Due build release dello stesso commit, con e senza il profilo; si riportano dimensione dell'exe, dell'installer e tempo di build.
- [ ] Il profilo si adotta solo se l'exe cala di almeno il 10%; nessun `opt-level = "s"`, `panic = "abort"` o `strip`.
- [ ] Se adottato: una Trascrizione della fixture e una Registrazione di prova riuscite con l'installer nuovo, e la durata della build release aggiornata in `docs/sviluppo/installer.md`.
- [ ] Esito e numeri nei commenti di questo ticket anche se il profilo non si adotta.
- [ ] I sei controlli passano.
