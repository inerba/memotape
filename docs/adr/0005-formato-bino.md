---
status: accepted
---

# Il formato Bino per le Registrazioni

A Stop una Registrazione produce un solo file `Registrazione <data ora>.bino` nella Cartella predefinita, invece di `.ogg` sciolti. È uno zip con:
- l'Ogg/Opus del mix e, in modalità Ingressi separati, un Ogg per Ingresso, salvati senza ricompressione;
- un JSON con versione, data, modalità, modello, Lingua del parlato e le Frasi (`id`, `inizio_ms`, `fine_ms`, testo, Ingresso, Parlante).

Il TXT resta un file a parte, salvato accanto, perché è il risultato da incollare altrove. Un Bino si riapre come Sorgente senza ritrascrivere. Una nuova Trascrizione riscrive il JSON dentro il Bino, dopo la conferma. L'installer associa `.bino` a Sbobino, con un'istanza unica dell'app.

Il Bino tiene insieme audio, Ingressi e testo senza duplicare l'audio e porta i metadati futuri. Il costo è che un Bino non si ascolta con un player qualsiasi: dentro l'app il clic sul nome di un Bino mostra il file nella cartella invece di aprirlo.

Il JSON ha un campo `version` fin dal primo giorno: un Bino salvato oggi deve aprirsi anche con le versioni future.
