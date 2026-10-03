# 02: Trascrizione dal vivo sul mix

**What to build:** l'utente attiva "Trascrivi dal vivo" accanto a Registra e vede il testo mentre registra:
- con Nemotron compaiono i Parziali, con Whisper e Parakeet le Frasi a fine Frase;
- la Registrazione non rallenta mai: se il motore resta indietro le Frasi vanno in coda;
- Pausa chiude la Frase e ferma il testo, Riprendi continua nella stessa sessione;
- dopo Stop la status bar mostra "Completamento della trascrizione…" con l'avanzamento e Annulla, finché la coda non è vuota;
- il testo si salva come risultato della Registrazione, con il formato esistente.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Casella "Trascrivi dal vivo" accanto a Registra: impostazione `trascrizione_dal_vivo`, default spenta, salvata tra un avvio e l'altro, bloccata durante la Registrazione
- [ ] L'uscita del mixer passa per downmix e ricampionamento a 16 kHz, poi va in un canale senza limite consumato da un thread di Trascrizione con il modello caricato e la Lingua del parlato delle impostazioni
- [ ] La Pausa chiude la Frase in corso. Il tempo delle Frasi esclude le pause e coincide con l'audio salvato
- [ ] Dopo Stop il canale si chiude e la coda si smaltisce, con avanzamento = frame consumati / frame ricevuti e Annulla. L'Attività Registrazione finisce quando la coda è vuota
- [ ] Con testo nell'area, Registra con il flag attivo chiede conferma prima di sostituirlo
- [ ] Modello non scaricato o non caricabile: la Registrazione parte comunque, con un avviso dedicato e il link alle Impostazioni
- [ ] Test: motore finto più lento dell'audio (tutte le Frasi arrivano, smaltimento dopo la chiusura), Pausa, Annulla durante lo smaltimento
- [ ] Verifica in `bun tauri dev` con Audio di sistema che riproduce la fixture: il testo compare durante la Registrazione

## Note per chi lo implementa
- Leggi prima `AGENTS.md` (comandi, prerequisiti, Insidie), `CONTEXT.md` (usa i suoi termini), gli ADR in `docs/adr/` e la spec `.scratch/sbobino-v2/spec.md`.
- Usa la skill `tdd` alle seam della spec: core Rust senza Tauri con `TranscriptionEngine`/`VoiceDetector` finti, e logica pura del frontend con `bun test`.
- Verifica il comportamento in `bun tauri dev`, poi chiudi app e Vite. Una build Rust alla volta.
- Chiudi con la skill `code-review`, eseguita in foreground, e applica le correzioni fondate.
- Commit con prefissi convenzionali, niente push. Aggiorna `AGENTS.md`, `PRODUCT.md` e questo ticket (criteri spuntati, `Status: done`).
- Non toccare `%APPDATA%\sbobino` (vecchia app) e non mettere mai l'email dell'utente in richieste HTTP.
