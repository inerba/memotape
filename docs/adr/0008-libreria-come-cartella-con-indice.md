---
status: accepted
---

# La Libreria è una cartella, il database è solo un indice

La Libreria è la cartella in cui Sbobino salva i Bini (prima la Cartella predefinita, `Documenti\Sbobino` se non scelta). Le Raccolte sono le sue cartelle di primo livello. Un Bino fa parte della Libreria se sta lì dentro: aggiungerlo vuol dire spostarlo in una Raccolta, toglierlo vuol dire spostarlo fuori o mandarlo nel Cestino. Il Bino resta la fonte di verità di audio, testo, correzioni e nomi dei Parlanti.

Per la ricerca nel testo c'è un solo database SQLite con un indice FTS5 (per parola e inizio di parola, senza maiuscole né accenti) in `%LOCALAPPDATA%\it.sbobino.desktop`, non nella cartella roaming. È una cache: si allinea alla cartella confrontando data di modifica e dimensione dei Bini, e se si perde o si corrompe si ricostruisce rileggendoli.

Alternative scartate:
- un catalogo nel database con i percorsi dei Bini sparsi sul disco: l'appartenenza alla Libreria sarebbe un dato in più da tenere allineato, e un Bino spostato in Esplora file diventerebbe una voce "non trovata" da riparare;
- testo e correzioni nel database, con il Bino come esportazione: due copie da allineare, e un Bino passato a un collega non porterebbe le correzioni;
- più Librerie separate (vault): per tenere distinti i clienti bastano le Raccolte, e la ricerca può attraversarle. Si potrà aggiungere cambiando la cartella della Libreria;
- etichette al posto delle Raccolte: un Bino in più gruppi non dà l'ordine che serve, e non ha un riscontro in Esplora file.

Conseguenza: quello che si vede nell'app è quello che si vede in Esplora file. Spostare, rinominare o cancellare un Bino a mano è un'operazione valida, e l'app la recepisce alla successiva scansione.
