# 06: Proteggere la Trascrizione dal vivo per Ingresso

**What to build:** durante una Registrazione da Entrambi il Microfono silenzioso non produce abitualmente Frasi inventate mentre l'Audio di sistema parla. La sensibilità prudente preserva risposte brevi e voce bassa e può essere cambiata da Impostazioni senza fermare la Registrazione.

**Blocked by:** 03 — Pulire le Registrazioni separatamente per Ingresso; 05 — Ridurre le false Frasi nei file e nei Tape preservando la voce debole.

**Status:** needs-triage

Bozza in attesa dell'approvazione della suddivisione. Requisiti: [spec approvata](../spec.md).

- [ ] I quattro livelli di sensibilità già disponibili nei profili funzionano nella Trascrizione dal vivo di ogni Ingresso e con tutti i motori ASR. DeepFilterNet3 e nuova protezione possono essere accesi o spenti indipendentemente; Spento mantiene Silero.
- [ ] La revisione della sensibilità accompagna l'audio acquisito prima delle code ASR. Un motore in ritardo usa la configurazione di quell'audio; il salvataggio di Impostazioni non applica il valore nuovo a tutte le Frasi già in coda.
- [ ] Cambi a cavallo dell'avvio, durante parlato e in Pausa sono coerenti con il valore persistente. Il nuovo livello influenza la nuova evidenza VAD senza troncare parole già ammesse o rettificare retroattivamente Frasi/Parziali pubblicati.
- [ ] Lo stato di evidenza e configurazione resta indipendente per Ingresso e sessione. Silenzio o rumore su Microfono non altera il riconoscimento dell'Audio di sistema; sessioni precedenti non contaminano quella corrente.
- [ ] Test deterministici coprono code ASR arretrate, cambi di livello dentro/fuori una Frase, Pausa/Riprendi e Stop. La protezione non cancella campioni dal salvataggio o cambia l'allineamento del testo rispetto all'audio.
- [ ] Prove native con DFN3 reale e Whisper, Parakeet e Nemotron coprono Microfono silenzioso con parlato loopback, respiri, rumore, voce debole e «sì/no». Riportano per livello/Ingresso false Frasi e parole perse, confrontando pulizia accesa/spenta e comportamento precedente.
- [ ] Bilanciato riduce le false Frasi del corpus live senza perdere le risposte brevi o la voce bassa designate come regressioni; Più sensibile preserva quelle regressioni. La configurazione viene corretta sulla base dei risultati, senza blacklist testuali o promesse universali.
- [ ] Una prova in release combina due Ingressi, denoiser, nuova protezione, ASR e salvataggio. Hardware, durata, memoria, tempo di elaborazione, andamento delle code e perdite sono riportati; il tratto stabile tiene il passo senza accumulo crescente o disallineamenti. Le prove con modelli reali sono sequenziali.
- [ ] Dopo Stop, Tape riaperto, player, testo e analisi finale dei Parlanti sono coerenti con l'audio salvato. La Diarizzazione non viene avviata durante la cattura.
- [ ] Guasti della pulizia continuano a produrre bypass e avviso del solo Ingresso/sessione interessati. La protezione resta definita anche in bypass; nessuna inferenza/attesa aggiuntiva entra nella callback WASAPI.
- [ ] Contratti generati, requisiti e decisioni riflettono il comportamento live verificato. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi. Collaudi nativi mancanti sono dichiarati e non considerati passati. Nessun commit o rilascio autonomo.

## Comments

Entrambi i predecessori sono necessari: percorso audio della Registrazione e protezione calibrata. Il ticket non riapre la segmentazione del testo.
