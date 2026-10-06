# Implementazione sequenziale della pulizia audio

Richiesta dell'utente del 6 ottobre 2026: implementare i sette ticket uno alla volta, con conversazioni nuove, modello 6.1 Sol e ragionamento scelto tra basso, medio e alto. La richiesta approva la suddivisione dei ticket. Ogni conversazione parte dal ticket e dalla spec, senza fork della conversazione precedente.

Tutti lavorano sul checkout locale esistente, preservando le modifiche precedenti e senza commit o rilascio autonomi come stabilito dalla spec. La skill implement viene applicata per TDD alle seam concordate, controlli e code-review.

| Ticket | Modello | Ragionamento | Motivo | Stato |
| --- | --- | --- | --- | --- |
| 01 | gpt-6.1-sol | high | Contratto PCM condiviso e preservazione dei tempi su tutti i percorsi | In corso: 01a110d3-e03c-7a30-aa79-f10dfc9c363e |
| 02 | gpt-6.1-sol | high | Runtime nativo, bundle, DSP e percorso completo file/Tape | In attesa |
| 03 | gpt-6.1-sol | high | Due Ingressi, tempi, transizioni, bypass e cattura reale | In attesa |
| 04 | gpt-6.1-sol | high | Riscrittura atomica e riuso degli intervalli trattati | In attesa |
| 05 | gpt-6.1-sol | high | Taratura prudente e regressioni con tre motori ASR | In attesa |
| 06 | gpt-6.1-sol | high | Configurazione prima delle code e carico live a due Ingressi | In attesa |
| 07 | gpt-6.1-sol | medium | Controlli UI su contratti già implementati e collaudo al volo | In attesa |

Non vengono avviati due ticket contemporaneamente, anche quando le dipendenze lo consentirebbero. Un ticket successivo parte dopo verifica dell'esito del precedente; eventuali criteri mancanti vengono riportati senza considerarli passati.
