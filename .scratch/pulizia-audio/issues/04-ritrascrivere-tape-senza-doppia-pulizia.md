# 04: Ritrascrivere un Tape senza doppia pulizia o sostituzioni parziali

**What to build:** Trascrivi su un Tape riusa gli intervalli già ripuliti e applica DeepFilterNet3 solo a quelli non trattati quando richiesto. Alla fine audio, mix, Forma d'onda e testo vengono sostituiti insieme; con Annulla o errore il Tape precedente rimane utilizzabile.

**Blocked by:** 02 — Pulire un file con DeepFilterNet3 e salvare un Tape coerente.

**Status:** partial

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [x] Tape misti e precedenti usano File e audio misto; Tape con Ingressi separati usano i rispettivi profili Microfono e Audio di sistema. I valori sono persistenti e disponibili in Impostazioni per i percorsi Tape implementati, senza esporre come operativi percorsi di Registrazione non ancora integrati.
- [x] Metadati facoltativi identificano algoritmo/versione e intervalli trattati per Ingresso. Intervalli già ripuliti vengono riutilizzati senza una seconda passata, anche con pulizia attiva; la disattivazione non tenta di recuperare l'originale.
- [ ] L'attivazione elabora solo gli intervalli non trattati, mantenendo contesto, transizioni, durata e sincronizzazione ai confini. I cambi durante la decodifica valgono sull'audio successivo e non sulla Frase consumata più tardi dall'ASR.
- [x] Con Ingressi separati, tracce elaborate, mix ricostruito, Forma d'onda e testo concordano; il mix non viene pulito una seconda volta dopo le tracce. Con audio misto non si inventano tracce Microfono/Sistema.
- [ ] Sostituzione atomica conserva insieme audio, metadati, testo e Forma d'onda corretti, oltre agli altri dati del Tape. Annulla, guasto del filtro, errore di scrittura o disco pieno conservano il precedente e rimuovono i temporanei senza dichiarare successo.
- [ ] La Trascrizione e l'analisi dei Parlanti utilizzano l'audio corrispondente al nuovo documento. Dopo il successo, riapertura, player, Frasi, copia e ricerca riflettono il nuovo risultato; i dati precedenti non restano nei consumatori downstream.
- [x] Aprire, riprodurre o cambiare Impostazioni non elabora né riscrive il Tape automaticamente. I vecchi Tape restano leggibili senza migrazione in apertura; assenza di metadati non viene descritta come prova che l'audio esterno sia sempre originale.
- [x] Nessuna seconda copia audio originale viene aggiunta. Una nuova Trascrizione di un Tape già pulito dimostra, con un processore deterministico, che non vengono riprocessati gli stessi campioni.
- [x] Test di creazione/riapertura coprono Tape vecchi, misti, separati e parzialmente trattati, errori e Annulla in più fasi della sostituzione. Smoke DFN3 reale verifica parole ai confini e riuso senza doppia pulizia, distinguendo i confronti PCM da quelli dopo Opus.
- [ ] UI, avvisi e spiegazioni sono accessibili e tradotti nelle sei lingue; contratti rigenerati e requisiti/decisioni aggiornati. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi, smoke reali separati e sequenziali. Nessun commit o rilascio autonomo.

## Comments

Può procedere in parallelo al ticket 03: riusa runtime, contratto audio e metadati introdotti dal ticket 02. Le modifiche condivise ai profili vanno coordinate senza aggiungere una dipendenza artificiale.

## Esito del 6 ottobre 2026

Implementazione locale conclusa. Stato `partial` per i collaudi manuali indicati
sotto; nessun commit, PR, installer o rilascio. Ticket 05 non avviato.
Il delta isolato rispetto all'inizio del 04 è in
`../verification-04/delta.patch`, con elenco in `changed-files.txt` e baseline
in `baseline/`. Il checkout contiene anche modifiche precedenti e concorrenti.

`ConfiguredCleaning::reuse` protegge gli intervalli già trattati, anche quando
un Tape a dispositivo singolo conserva metadati Microfono/Sistema nel mix.
Solo le lacune entrano nel filtro; il contesto precedente prepara lo stato
senza essere riconsegnato o registrato come nuova pulizia. La conversione degli
intervalli protegge ogni campione intersecato, su frequenze razionali.
`TapeAudio` prepara le tracce, ricostruisce il mix dai PCM prima di Opus e offre
lo stesso audio alla Trascrizione e all'analisi dei Parlanti. Il commit sostituisce
audio, Forma d'onda, testo e metadati con una sola rinomina finale. Le voci
immutate e i campi JSON sconosciuti sono conservati. Il player non salva più
automaticamente la Forma d'onda dei Tape precedenti.

### Evidenza raccolta

- TDD deterministico: il test di riuso è fallito prima dell'implementazione e
  poi passato; il test del player ha rilevato la riscrittura precedente prima
  della correzione. Cinque test del manager coprono Tape misti, separati,
  precedenti e parziali; riuso byte per byte degli Ogg invariati; PCM atteso,
  nuovo mix e Forma d'onda; riapertura, copia e ricerca; Annulla prima della
  preparazione, nell'ASR e prima del commit; guasto del processore e rinomina
  impedita da un handle Windows. I temporanei di proprietà sono rimossi e il
  precedente rimane byte per byte identico nei casi falliti.
- Smoke release `dfn3_ritrascrive_tape_parziale_con_parole_ai_confini_e_riusa_gli_ogg`:
  passato separatamente, 26,89 s. A 44,1 kHz stereo, due partizioni dei blocchi
  producono PCM identico (RMSE 0), preservando campioni protetti e durata.
  Con DFN3 e Nemotron reali la fixture mantiene tutte le parole attese:
  «Buongiorno a tutti, oggi parliamo di trascrizione. Il computer trasforma la
  voce in testo.» Un secondo riuso conserva gli Ogg byte per byte e non
  aggiunge intervalli. Il confronto PCM precede Opus; i confronti sull'audio
  decodificato dagli Ogg usano tolleranza, senza attribuire al filtro il codec.
- Sei controlli verdi: typecheck; frontend 118 test, 0 fallimenti; lint frontend
  su 119 file; `cargo fmt --check`; clippy con `-D warnings`; Rust 284 test,
  0 fallimenti, 24 ignorati. Contratti rigenerati con il test dedicato:
  `bindings.ts` identico alla baseline. Log in `../verification-04/`.
- Review indipendenti sul delta isolato: Spec senza difetti certi; Standards
  ha rilevato commenti obsoleti sul salvataggio automatico della Forma d'onda,
  corretti in AGENTS e nel comando `tape_peaks`. Report in `review.md`.
- Spiegazioni e avvisi aggiornati nelle sei lingue; PRODUCT e ADR-0022
  descrivono riuso, profili e sostituzione atomica.

### Criteri ancora aperti

Le caselle 3, 5, 6 e 10 rimangono aperte per non equiparare test del core a
collaudo completo. Non sono stati verificati ascolto percettivo delle
transizioni, clic e accessibilità nella finestra Tauri, aggiornamento del
player visibile e modifica delle Impostazioni durante una vera Trascrizione.
L'errore reale di disco pieno non è stato provocato; la preservazione è
verificata per guasto del filtro, Annulla e rinomina fallita. Non sono
attestate sessioni lunghe, altri PC o voci reali. La licenza/distribuzione dei
pesi e il bundle restano questioni aperte dal ticket 02, fuori da questa prova.
