# 03: DFN3 fuori, testi, ADR e documenti

**What to build:** Un installer senza DeepFilterNet3, con testi, crediti e documenti che nominano RNNoise. Spec: `../spec.md`, sezioni "Cosa esce" e "Testi e documenti".

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Via `deep_filter`, i quattro crate tract fissati e le dipendenze che servivano solo a loro; via il modello DFN3 dalle risorse, le sue licenze e quella di tract, le ottimizzazioni del profilo dev dedicate; via il codice e i test di DFN3.
- [ ] Messaggi d'errore della pulizia, descrizioni della pulizia per file e Registrazione e crediti in Informazioni nominano RNNoise nelle sei lingue; i test dei crediti seguono.
- [ ] Un ADR nuovo che sostituisce la scelta del modello dell'ADR-0019 (marcato «superato in parte»): prova d'ascolto alla cieca, misure di Silero e CPU, peso, versione e hash. Rimanda a `.scratch/prova-denoiser/esito.md`.
- [ ] `PRODUCT.md`, `CONTEXT.md` se nomina DFN3, `AGENTS.md`, i documenti di sviluppo di trascrizione, registrazione e installer aggiornati; una prova su una CPU senza AVX2 aggiunta tra le verifiche su hardware reale.
- [ ] Verifica dell'installer come in `docs/sviluppo/installer.md`: nessun file DFN3, pulizia accesa su un file e in una Registrazione di prova riuscite; dimensioni dell'installer e dell'installato nei commenti.
- [ ] I sei controlli passano.
