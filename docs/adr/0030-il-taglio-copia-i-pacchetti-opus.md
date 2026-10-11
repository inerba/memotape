---
status: accepted
---

# Il Taglio copia i pacchetti Opus e manda l'originale nel Cestino

Un Taglio toglie da un Tape uno o più tratti, all'inizio, alla fine o in mezzo, insieme al loro audio e alle loro Frasi. È distruttivo: il Tape si riscrive senza i tratti, e l'originale va nel Cestino di Windows con lo stesso nome. Su un disco senza Cestino (`noRecycleBin`) il Taglio chiede una seconda conferma e sostituisce l'originale.

L'audio non si ricodifica. `ogg_opus::taglia` rilegge l'Ogg pacchetto per pacchetto e lo riscrive senza i pacchetti dei tratti: stesso serial, stessi `OpusHead` (con il pre-skip) e `OpusTags`, granule rifatte come `pacchetti × 960`, una pagina ogni 50 pacchetti, e sull'ultima pagina l'end trimming originale se la fine del Tape resta. Lo fa con lo stesso elenco di pacchetti su `mix.ogg`, `microfono.ogg` e `sistema.ogg`, che hanno la stessa linea del tempo. Fatti sul formato in `docs/research/accodare-ogg-opus.md` (la Continuazione, ADR-0013, fa l'operazione inversa).

- I bordi cadono sui pacchetti da 20 ms (`tape::tagli_applicabili`), e il documento si sposta degli stessi multipli di 20 ms; i 6,5 ms di pre-skip non si contano.
- Dopo una giunzione il decoder parte con lo stato del pacchetto prima, non con quello che l'encoder aveva. Non si sente perché i bordi stanno nei silenzi: la selezione si allarga alle Frasi intere e lascia 1 s di pausa per lato. Provato nella WebView2 e con Symphonia (test `il_taglio_toglie_i_pacchetti_senza_ricodificare_e_il_file_si_rilegge`).
- La Forma d'onda salvata perde i valori dei tratti tagliati (ognuno copre lo stesso tempo), senza decodificare.
- Il player aggiunge `?v=<versione>` all'indirizzo del mix: allo stesso percorso la WebView riuserebbe l'audio già letto, nonostante `no-store`. Il protocollo `tape` ignora la query.

Alternative scartate:
- **Tagli non distruttivi** (il Tape ricorda solo i tratti esclusi): ogni lettore, cioè player, Copia testo, Markdown, ricerca, Assistenti e durata, dovrebbe saltarli e rimappare i tempi.
- **Ricodifica**: circa 50 s per ora di audio, una perdita di generazione a ogni Taglio e un'Attività con avanzamento, per una giunzione che nei silenzi non si sente comunque.
- **Originale senza rete di sicurezza**: un Taglio sbagliato non si recupera; il Cestino costa una chiamata già usata da Elimina.
