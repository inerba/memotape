# 10: Lingue dell'interfaccia, tema e Informazioni

**What to build:** l'utente usa l'app in italiano, inglese, francese, spagnolo, tedesco o polacco. Al primo avvio la lingua è quella di sistema se è supportata, altrimenti l'inglese. Una nuova lingua si applica al riavvio. Il tema segue quello di Windows. Una sezione Informazioni mostra la versione dell'app e le licenze dei componenti.

**Blocked by:** 06 (Impostazioni persistenti, scelta del modello e Lingua del parlato)

**Status:** done

- [x] File di traduzione per en, fr, es, de e pl completi rispetto all'italiano di riferimento, con le forme plurali giuste (`_many` per it/fr/es, forme del polacco)
- [x] Lingua dell'interfaccia nelle impostazioni, con l'avviso "si applica al riavvio". Al primo avvio la lingua di sistema se supportata, altrimenti en
- [x] Il tema chiaro/scuro segue `prefers-color-scheme` con i token shadcn
- [x] Impostazioni → Informazioni: versione e licenze. Parakeet CC-BY-4.0 con attribuzione, poi Nemotron OpenMDW-1.1, Whisper MIT, Silero MIT, Symphonia MPL-2.0, transcribe-cpp MIT, ONNX Runtime MIT, vad-rs. I testi delle licenze sono inclusi come risorse
- [x] Se `get_settings` fallisce all'avvio (per esempio `settings.json` illeggibile) l'app parte comunque, con i valori predefiniti e la lingua di sistema, e mostra l'errore nella status bar
- [x] Test: tutte le chiavi presenti in ogni lingua (`bun test`) e lingua di default da quella di sistema (Rust)
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
