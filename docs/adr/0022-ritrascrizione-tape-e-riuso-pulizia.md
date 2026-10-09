---
status: accepted
---

> Superato in parte da ADR-0029 (9 ottobre 2026): Trascrivi su un Tape non pulisce più
> l'audio. Restano valide la Forma d'onda calcolata in memoria e `Cache-Control: no-store`.

# Ritrascrizione atomica del Tape e riuso dei tratti puliti

Il ticket 04 della spec `.scratch/pulizia-audio/spec.md` estende il percorso
di ADR-0019 ai Tape. Non aggiunge protezione VAD, controlli nella toolbar,
segmentazione o Diarizzazione durante la cattura.

`TapeAudio` prepara gli Ingressi reali in una cartella temporanea esclusiva
accanto alla Sorgente. Ogni blocco passa dal contratto `AudioProcessor` prima
delle destinazioni. Senza tracce Microfono/Sistema usa il profilo File e audio
misto. I profili restano quelli persistenti di Impostazioni e vengono letti
alla decodifica, prima delle code ASR. Le preferenze non modificano un Tape
fino a una nuova Trascrizione esplicita.

`ConfiguredCleaning::reuse` protegge i tratti precedenti qualunque sia la
versione dell'algoritmo. Per un mix considera anche i tratti Microfono/Sistema
delle Registrazioni con un solo dispositivo. Per tracce separate considera
solo l'Ingresso corrispondente. La conversione dei confini usa rapporti interi
fra frequenze: protegge ogni campione che interseca un tratto già pulito,
senza passare dai millisecondi. Conserva i metadati precedenti e aggiunge solo
gli intervalli nuovi consegnati con successo. Le lacune non attestano che
un audio esterno sia originale.

La seam PCM ammette una storia di sola lettura all'inizio di un nuovo tratto.
DFN3 la usa per inizializzare il contesto, senza riconsegnarne l'audio o
registrarla come nuovo trattamento. La storia è limitata a mezzo secondo più
8192 frame; il pre-roll è allineato alla griglia assoluta dell'hop da 10 ms
e quindi a quella razionale del ricampionamento. Il runtime conserva la fase
entro il tratto e le transizioni verso il riferimento originale. Nessuna
correzione della durata mediante taglio delle ultime parole. Il mixer e la
continuazione del ricampionatore di ADR-0021 restano nel loro percorso.

Le tracce elaborate alimentano temporanei PCM e Ogg. Se almeno una traccia
cambia, il mix viene ricostruito dai PCM allineati prima di Opus, senza DFN3
sul mix. Ogni traccia senza nuovo trattamento conserva l'Ogg precedente; se
nessuna cambia, anche mix e Forma d'onda vengono ricopiati senza modifiche.
ASR e analisi dei Parlanti leggono gli Ogg destinati al Tape, inclusi i tratti
già trattati. Il confronto PCM prima di Opus è distinto da quello dopo il
codec, che non garantisce identità dei campioni nelle tracce ricodificate.

`rewrite_audio_cancellable` crea con `create_new` uno zip esclusivo accanto
al Tape, ricopia le voci non sostituite, conserva i campi JSON sconosciuti,
scrive documento, nuovi Ogg e Forma d'onda e sincronizza il file prima del
rename. Controlla Annulla durante la copia dei nuovi Ogg e prima del rename;
errori o Annulla eliminano soltanto i temporanei posseduti dalla transazione.
La sostituzione testuale continua a eliminare le correzioni e i nomi precedenti
dopo la conferma dell'utente. Data, origine, durata e dati aggiuntivi rimangono.
Il Tape v1 non archivia un originale aggiuntivo.

Questa decisione supera la riscrittura automatica in apertura di ADR-0010:
una Forma d'onda mancante viene calcolata soltanto in memoria. Il protocollo
del player usa `Cache-Control: no-store`, perché lo stesso percorso può ora
servire un mix sostituito. Il player viene già smontato durante Trascrivi;
riapertura e ricerca si aggiornano tramite il percorso esistente.

Test deterministici coprono riuso, mix/separati, metadati parziali, Annulla,
guasto e rifiuto reale del rename su Windows, oltre a copia/ricerca/player.
Lo smoke DFN3 è mirato a PCM, partizioni, parole della fixture e riuso degli
Ogg. Le prove non attestano ascolto percettivo, clic/accessibilità Tauri,
disco pieno reale o altri PC. Bundle e licenza dei pesi restano aperti nei
ticket precedenti; nessuna distribuzione è autorizzata.
