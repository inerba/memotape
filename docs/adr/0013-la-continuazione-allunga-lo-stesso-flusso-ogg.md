---
status: accepted
---

# La Continuazione allunga lo stesso flusso Ogg

Una Continuazione accoda audio nuovo al `mix.ogg` di un Bino (e a `microfono.ogg` e `sistema.ogg`, se ci sono). Non si ricodifica il vecchio e non si fa una catena Ogg: le pagine dell'Ogg nuovo si copiano in coda allo **stesso flusso logico**, con serial, sequence number e granule rinumerati e il CRC ricalcolato, saltando il loro `OpusHead` e `OpusTags`. L'ultima pagina vecchia perde EOS e prende come granule il totale dei suoi campioni decodificati (`pacchetti × 960`) al posto dell'end trimming. Fatti e prove sono in `docs/research/accodare-ogg-opus.md`.

- Così il file resta conforme all'RFC 7845 §4, che ammette l'end trimming solo sulla pagina con EOS.
- L'audio nuovo comincia a `pacchetti vecchi × 20 ms`, non a `durata_ms` del Bino: in mezzo restano il riempimento dell'ultimo pacchetto vecchio (meno di 20 ms) e il pre-skip nuovo, circa 13 ms di quasi silenzio, senza clic.
- Il Bino si riscrive comunque, perché la voce `Stored` cresce, ma è una copia di byte.

Alternative scartate:
- **catena Ogg** (secondo flusso con serial suo): valida per l'RFC 3533, ma WebView2 suona solo il primo flusso, con durata e seek sbagliati, e il `Decoder` (Symphonia) si ferma al `ResetRequired`;
- **stesso flusso con l'end trimming tenuto sull'ultima pagina vecchia**: viola un MUST dell'RFC 7845, e Symphonia e ffmpeg divergono di qualche millisecondo sulla linea del tempo;
- **ricodifica completa**: circa 50 s per ora di audio e una perdita di qualità a ogni Continuazione.

Conseguenza: canali e frequenza della Continuazione sono quelli del Bino, non quelli delle impostazioni, perché un flusso Ogg Opus ha un solo `OpusHead`.
