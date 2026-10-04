---
status: accepted
---

# La Forma d'onda sta nel Bino

Calcolare la Forma d'onda vuol dire decodificare tutto il mix: 8 s per 25 minuti di audio nell'app installata, a ogni apertura del Bino. Per non ripetere il calcolo, la Forma d'onda si salva nel Bino (ADR-0005) come voce a parte dello zip, `forma-onda.json`: al più 1000 valori (0–1) dall'inizio alla fine del mix qualunque sia la durata (uno ogni 20 ms se il mix dura meno di 20 s), che il player raggruppa nelle barre che mostra.

- Si calcola mentre l'audio del mix si scrive (Registrazione e Trascrizione di un file), senza decodificare di nuovo, e si scrive con il Bino.
- Un Bino che non la ha (scritto prima di questa decisione, o con la voce illeggibile o vuota) la calcola all'apertura e prova una volta a riscriversi con la voce. Se non riesce (cartella in sola lettura, file bloccato, Attività su quel Bino) la mostra lo stesso, senza errori, e la ricalcola alla prossima apertura.
- È una voce e non un campo di `trascrizione.json`: `rewrite` copia così come sono le voci diverse dal documento, quindi correzioni, rinomine e Trascrivi su un Bino la conservano, anche quelli di una versione dell'app che non la conosce. È facoltativa, quindi `version` resta 1.
- Non ha hash né versione: il mix non cambia dopo la creazione del Bino.

Alternative scartate:
- una colonna nell'indice della Libreria (ADR-0008), che è già una cache: un Bino fuori dalla Libreria o passato a un collega ricalcolerebbe a ogni apertura, e un Bino appena registrato pagherebbe il calcolo alla prima apertura;
- file di cache in `%LOCALAPPDATA%`: un secondo meccanismo di cache accanto all'indice, con gli stessi limiti;
- un valore ogni 20 ms: per un'ora sono 180 000 valori, quando al player ne servono qualche centinaio.

Conseguenza: aprire per la prima volta un Bino vecchio ne cambia la data di modifica (la Libreria lo rilegge, OneDrive lo risincronizza).
