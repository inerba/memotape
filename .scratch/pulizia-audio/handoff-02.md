# Checkpoint del ticket 02

Il coordinatore ha seguito l'implementazione nella conversazione 01a110f3-1e47-75d0-9063-f924feaa8182. Il 6 ottobre, leggendo la stessa conversazione, è comparso un nuovo turno iniziale senza i turni precedenti. I file DFN3 già presenti sono il lavoro parziale di QUESTO ticket, non un'altra implementazione da ignorare o duplicare. Le modifiche concorrenti all'editor dei turni e al logo sono invece estranee e vanno preservate.

La baseline ORIGINALE del ticket 02 è ancora disponibile in `C:\Users\inerba\.codex\visualizations\2026\10\06\01a110f3-1e47-75d0-9063-f924feaa8182\baseline-02` (directory verificata dal coordinatore). Usarla per il diff completo della review. Le nuove baseline scattate nella ripresa contengono già il lavoro parziale del ticket e servono soltanto a identificare gli edit successivi.

## Evidenze già osservate

- Ticket 01 completato e revisionato, sei controlli verdi.
- libDF originale 0.5.6/revisione approvata con Tract 0.19.16 compila nel grafo completo Windows/MSVC.
- Profili persistenti di pulizia inizialmente spenti, controllo File e audio misto, metadati per intervallo, risorse/modello/licenze e contratti sono già stati introdotti dal ticket 02. Vanno verificati e completati, non ricreati.
- Runtime confinato nel thread che lo crea, senza forzare Send. Percorso spettrale pubblico avanza sul silenzio, evitando la scorciatoia energetica di libDF e senza segnali pilota.
- Tre smoke reali sono passati: equivalenza numerica iniziale con l'originale su audio non silenzioso; silenzio/voce debole/coda; ricampionamento, pause, sessioni e formato fino a sei canali.
- Importazione completa WAV e MP4 con Nemotron, Whisper e Parakeet: sei casi passati, Tape riaperto con testo, audio, Forma d'onda e metadati. I log stanno nella cartella verification-02 accanto a questo documento.
- In una verifica intermedia sono passati 115 test frontend e 261 test Rust (18 smoke esclusi), oltre a fmt e clippy. NON sono esiti del codice finale dopo le ultime correzioni.

## Regressioni e correzioni da confermare

Il primo limite globale a 12 dB perdeva parole iniziali della voce attenuata di 30 dB. Anche 6 dB e il riferimento spettrale non bastavano. La diagnosi ha dimostrato durata/posizione dei campioni conservate, ma onset Silero più tardivo: circa 480 ms sull'originale contro 1800 ms sul percorso precedente. Non cambiare le soglie Silero per mascherare il problema.

È stato introdotto il riferimento PCM originale per il mix prudente e per il percorso che DFN3 considera già pulito. L'attenuazione interna viene limitata quando la stima del modello è poco affidabile. L'ultima comunicazione del vecchio turno riferiva che la voce debole conserva le parole con tutti e tre gli ASR e il caso breve mantiene «Sì. No.» con Nemotron e Whisper; Parakeet mantiene il risultato baseline «C No». Verifica direttamente i test e i log: non assumere che questo basti a chiudere il ticket.

Il coordinatore ha richiesto due condizioni congiunte: conservare parole brevi/deboli e ridurre il rumore in modo misurabile. Non scegliere un limite globale quasi nullo soltanto per far passare le parole. Una misura intermedia del caso con rumore uniforme mostrava +1,26 dB rispetto alla voce pulita, contando la distorsione; va ripetuta sulla taratura finale. Verificare anche il solo round-trip frequenza originale → 48 kHz → originale a attenuazione realmente nulla, separando ricostruzione, ricampionamento e VAD/ASR. Nessun AGC/normalizzazione automatica dell'audio salvato o blacklist testuale.

## Lavoro ancora necessario

- Riferimento PCM a attenuazione nulla, conservazione delle parole attese, rumore e prove reali sulla taratura finale.
- Review Spec e Standards del lavoro COMPLETO del ticket rispetto alla baseline originale, non soltanto della ripresa dopo questo checkpoint. Erano state avviate, ma il coordinatore non ha ricevuto un esito conclusivo.
- Sei controlli verdi sul codice finale, con il test del Cestino fuori sandbox quando necessario.
- Pacchetto/bundle locale e verifiche richieste. La prima build release è stata interrotta dopo una fase CMake senza attività visibile; non è un successo. Verificare processi prima di riprendere, includendo eventuali watcher di rigenerazione bindings rimasti aperti. Una sola build Rust alla volta, arrestare soltanto propri processi e preservare le attività dell'utente.
- Aggiornare ticket con evidenze e criteri mancanti, senza chiusura falsa; non avviare ticket 03, commit o rilascio.

## Licenza dei pesi

La redistribuzione dei pesi resta non verificata: upstream dichiara MIT/Apache per il codice, ma le richieste 697, 700 e 709 sono aperte. Il coordinatore ha verificato le fonti e registrato il limite nel piano implementazione. Proseguono implementazione/prove locali; quel criterio non può essere dichiarato passato e nessun rilascio è autorizzato.
