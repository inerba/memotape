# 03: DFN3 fuori, testi, ADR e documenti

**What to build:** Un installer senza DeepFilterNet3, con testi, crediti e documenti che nominano RNNoise. Spec: `../spec.md`, sezioni "Cosa esce" e "Testi e documenti".

**Blocked by:** 02

**Status:** ready-for-agent

- [x] Via `deep_filter`, i quattro crate tract fissati e le dipendenze che servivano solo a loro; via il modello DFN3 dalle risorse, le sue licenze e quella di tract, le ottimizzazioni del profilo dev dedicate; via il codice e i test di DFN3.
- [x] Messaggi d'errore della pulizia, descrizioni della pulizia per file e Registrazione e crediti in Informazioni nominano RNNoise nelle sei lingue; i test dei crediti seguono.
- [x] Un ADR nuovo che sostituisce la scelta del modello dell'ADR-0019 (marcato «superato in parte»): prova d'ascolto alla cieca, misure di Silero e CPU, peso, versione e hash. Rimanda a `.scratch/prova-denoiser/esito.md`.
- [x] `PRODUCT.md`, `CONTEXT.md` se nomina DFN3, `AGENTS.md`, i documenti di sviluppo di trascrizione, registrazione e installer aggiornati; una prova su una CPU senza AVX2 aggiunta tra le verifiche su hardware reale.
- [ ] Verifica dell'installer come in `docs/sviluppo/installer.md`: nessun file DFN3, pulizia accesa su un file e in una Registrazione di prova riuscite; dimensioni dell'installer e dell'installato nei commenti.
- [x] I sei controlli passano.

## Note dell'implementazione (18 ottobre 2026)

- Il modulo Rust di DFN3 era già uscito nel ticket 02, perché era rimasto codice morto. Qui escono:
  - `deep_filter`, i quattro crate tract e `ndarray` 0.15 (resta lo 0.17 di `ort`);
  - il modello, le licenze di DFN3 e di tract;
  - le ottimizzazioni dev di `deep_filter`, tract e `ndarray`; restano quelle di rubato e rustfft.
- Crediti: RNNoise 0.2 (BSD-3-Clause) al posto di DFN3 e tract. Testi nelle sei lingue. ADR-0034, con l'ADR-0019 e l'ADR-0026 superati in parte.
- `bun tauri build`:
  - installer 19,7 MB contro i 29,6 della 0.5.3 (−9,9 MB);
  - app installata 103,4 MB contro 126,5 (−23 MB);
  - exe release 26,9 MB;
  - `7z l` non trova file DFN3 né tract, e c'è `rnnoise-0b50c45.bin`.
- **Da fare**: installazione silenziosa e prova dell'app installata con la pulizia accesa su un file e in una Registrazione (`docs/sviluppo/installer.md`, passi 1–5). Tocca le impostazioni vere dell'utente, che vanno copiate prima e rimesse dopo.
