# 04: Ritrascrivere un Tape senza doppia pulizia o sostituzioni parziali

**What to build:** Trascrivi su un Tape riusa gli intervalli già ripuliti e applica DeepFilterNet3 solo a quelli non trattati quando richiesto. Alla fine audio, mix, Forma d'onda e testo vengono sostituiti insieme; con Annulla o errore il Tape precedente rimane utilizzabile.

**Blocked by:** 02 — Pulire un file con DeepFilterNet3 e salvare un Tape coerente.

**Status:** needs-triage

Bozza in attesa dell'approvazione della suddivisione. Requisiti: [spec approvata](../spec.md).

- [ ] Tape misti e precedenti usano File e audio misto; Tape con Ingressi separati usano i rispettivi profili Microfono e Audio di sistema. I valori sono persistenti e disponibili in Impostazioni per i percorsi Tape implementati, senza esporre come operativi percorsi di Registrazione non ancora integrati.
- [ ] Metadati facoltativi identificano algoritmo/versione e intervalli trattati per Ingresso. Intervalli già ripuliti vengono riutilizzati senza una seconda passata, anche con pulizia attiva; la disattivazione non tenta di recuperare l'originale.
- [ ] L'attivazione elabora solo gli intervalli non trattati, mantenendo contesto, transizioni, durata e sincronizzazione ai confini. I cambi durante la decodifica valgono sull'audio successivo e non sulla Frase consumata più tardi dall'ASR.
- [ ] Con Ingressi separati, tracce elaborate, mix ricostruito, Forma d'onda e testo concordano; il mix non viene pulito una seconda volta dopo le tracce. Con audio misto non si inventano tracce Microfono/Sistema.
- [ ] Sostituzione atomica conserva insieme audio, metadati, testo e Forma d'onda corretti, oltre agli altri dati del Tape. Annulla, guasto del filtro, errore di scrittura o disco pieno conservano il precedente e rimuovono i temporanei senza dichiarare successo.
- [ ] La Trascrizione e l'analisi dei Parlanti utilizzano l'audio corrispondente al nuovo documento. Dopo il successo, riapertura, player, Frasi, copia e ricerca riflettono il nuovo risultato; i dati precedenti non restano nei consumatori downstream.
- [ ] Aprire, riprodurre o cambiare Impostazioni non elabora né riscrive il Tape automaticamente. I vecchi Tape restano leggibili senza migrazione in apertura; assenza di metadati non viene descritta come prova che l'audio esterno sia sempre originale.
- [ ] Nessuna seconda copia audio originale viene aggiunta. Una nuova Trascrizione di un Tape già pulito dimostra, con un processore deterministico, che non vengono riprocessati gli stessi campioni.
- [ ] Test di creazione/riapertura coprono Tape vecchi, misti, separati e parzialmente trattati, errori e Annulla in più fasi della sostituzione. Smoke DFN3 reale verifica parole ai confini e riuso senza doppia pulizia, distinguendo i confronti PCM da quelli dopo Opus.
- [ ] UI, avvisi e spiegazioni sono accessibili e tradotti nelle sei lingue; contratti rigenerati e requisiti/decisioni aggiornati. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi, smoke reali separati e sequenziali. Nessun commit o rilascio autonomo.

## Comments

Può procedere in parallelo al ticket 03: riusa runtime, contratto audio e metadati introdotti dal ticket 02. Le modifiche condivise ai profili vanno coordinate senza aggiungere una dipendenza artificiale.
