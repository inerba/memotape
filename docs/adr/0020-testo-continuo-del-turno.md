---
status: accepted
---

# Il testo corretto appartiene al Turno, i riferimenti audio restano alle Frasi

Il 6 ottobre 2026 l'utente ha approvato un editor continuo per ciascun Turno: selezione e cancellazione attraversano le Frasi; Invio inserisce un a capo, uscire salva, Esc scarta la bozza e Ctrl+Z opera durante la scrittura. Un Turno può restare senza testo. Vedi `.scratch/editor-turno/spec.md`.

## Rappresentazione

Il Tape v1 aggiunge il campo facoltativo `correzioni_testo`: ogni `TestoTurno` contiene Ingresso, id delle Frasi e testo esatto, compresi gli a capo. Le Frasi conservano id, intervalli e attribuzioni; il salvataggio elimina i loro tempi ASR del testo e segna `testo_corretto`. Il testo originario resta come provenienza interna e non viene riemesso nelle copie, nella ricerca o nelle letture per Assistenti.

La proiezione `frasi_visibili` conserva tutti i riferimenti audio e presenta una sola copia del testo corretto, sulla prima Frase della correzione. Le altre Frasi del gruppo non portano testo aggiuntivo. Il frontend usa questa stessa rappresentazione per l'editor e Copia turno. Le uscite del documento riuniscono soltanto i Turni con una correzione continua; le Frasi non corrette conservano la rappresentazione precedente.

`edit_turno` valida gli id dell'intero Turno e il testo originale atteso, poi esegue una sola riscrittura atomica. Una richiesta obsoleta viene rifiutata prima di scrivere; un salvataggio identico non riscrive lo zip. Audio e Forma d'onda non vengono ricostruiti. Le azioni Copia e Unisci attendono la correzione in corso e non la avviano due volte.

## Nuova Diarizzazione

La sola correzione testuale non protegge il Parlante: l'analisi aggiorna le attribuzioni delle Frasi originarie. Se tutte concordano, il testo corretto usa quella voce. Se discordano, il testo riunito viene mostrato come «Parlante non determinato»: non si può assegnare una parte del testo a ciascuna voce senza inventare l'allineamento eliminato dalla correzione. Le nuove attribuzioni restano nei dati delle Frasi; l'eventuale attribuzione corretta a mano resta protetta come in ADR-0017.

I tratti con attribuzione protetta contribuiscono alla corrispondenza temporale delle identità, senza venire riassegnati. Se il loro audio è ambiguo, la sua identità non viene propagata a un'altra voce per semplice coincidenza del numero.

## Interfaccia e compatibilità

`TurnEditor` usa una textarea nativa non controllata. Il player e i rerender non riscrivono il valore durante la scrittura; il controllo gestisce cursore, selezione e undo. L'altezza segue il contenuto. Una correzione fallita mantiene la bozza e un errore accanto al campo; un Turno vuoto mostra «Blocco senza testo», omesso dalle uscite del parlato.

I Tape precedenti si aprono con `correzioni_testo` vuoto e non vengono migrati all'apertura. La compatibilità garantita è la lettura dei documenti precedenti nella nuova app; le versioni precedenti non conoscono la proiezione delle nuove correzioni. La nuova Trascrizione sostituisce anche le correzioni soltanto a successo.

Alternative scartate: unire fisicamente le Frasi avrebbe cambiato i riferimenti audio; spartire nuovamente il testo corretto tra Frasi avrebbe inventato tempi delle parole. Un editor per Frase non consente cancellazione e selezione oltre i suoi confini.
