---
status: accepted
---

# Ogni Trascrizione di un file diventa un Bino

Trascrivi su un file audio o video crea `<nome del file>.bino` nella Raccolta aperta (o in "Senza raccolta"), con l'audio ricodificato in Ogg/Opus come `mix.ogg` (bitrate, canali e frequenza delle impostazioni di Registrazione) e il testo con i tempi delle Frasi. Il file originale non si tocca e il suo nome resta nel documento. Una Trascrizione annullata o senza parlato non crea il Bino. Finita la Trascrizione il Bino diventa la Sorgente.

Senza questo, un file trascritto lascerebbe solo il testo: niente tempi delle Frasi, quindi niente player sincronizzato, niente Libreria né ricerca. Un Bino autonomo resta intero anche se l'originale si sposta o si cancella, e la Libreria contiene un solo tipo di oggetto.

Alternative scartate:
- il file come Sorgente di passaggio: player e sincronizzazione solo finché resta aperto;
- un Bino con il solo testo e il percorso dell'originale: l'audio si perde appena l'originale si sposta, e la Libreria tornerebbe a riferimenti da riparare (ADR-0008);
- l'audio originale copiato com'è nel Bino: `mix.ogg` è sempre Ogg/Opus e un video porterebbe dentro anche il video.

Costi: circa 14 MB per ora di audio a 32 kbps e il tempo della codifica Opus. La conversione resta interna al Bino: "Estrai solo audio" continua a non far parte del prodotto (ADR-0002).
