# 06: Proteggere la Trascrizione dal vivo per Ingresso

**What to build:** durante una Registrazione da Entrambi il Microfono silenzioso non produce abitualmente Frasi inventate mentre l'Audio di sistema parla. La sensibilità prudente preserva risposte brevi e voce bassa e può essere cambiata da Impostazioni senza fermare la Registrazione.

**Blocked by:** 03 — Pulire le Registrazioni separatamente per Ingresso; 05 — Ridurre le false Frasi nei file e nei Tape preservando la voce debole.

**Status:** partial — implementazione, controlli e review locali conclusi il 6 ottobre 2026; corpus umano, ascolto e collaudo Tauri aperti.

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [x] I quattro livelli di sensibilità già disponibili nei profili funzionano nella Trascrizione dal vivo di ogni Ingresso e con tutti i motori ASR. DeepFilterNet3 e nuova protezione possono essere accesi o spenti indipendentemente; Spento mantiene Silero.
- [x] La revisione della sensibilità accompagna l'audio acquisito prima delle code ASR. Un motore in ritardo usa la configurazione di quell'audio; il salvataggio di Impostazioni non applica il valore nuovo a tutte le Frasi già in coda.
- [x] Cambi a cavallo dell'avvio, durante parlato e in Pausa sono coerenti con il valore persistente. Il nuovo livello influenza la nuova evidenza VAD senza troncare parole già ammesse o rettificare retroattivamente Frasi/Parziali pubblicati.
- [x] Lo stato di evidenza e configurazione resta indipendente per Ingresso e sessione. Silenzio o rumore su Microfono non altera il riconoscimento dell'Audio di sistema; sessioni precedenti non contaminano quella corrente.
- [x] Test deterministici coprono code ASR arretrate, cambi di livello dentro/fuori una Frase, Pausa/Riprendi e Stop. La protezione non cancella campioni dal salvataggio o cambia l'allineamento del testo rispetto all'audio.
- [ ] Prove native con DFN3 reale e Whisper, Parakeet e Nemotron coprono Microfono silenzioso con parlato loopback, respiri, rumore, voce debole e «sì/no». Riportano per livello/Ingresso false Frasi e parole perse, confrontando pulizia accesa/spenta e comportamento precedente.
- [x] Bilanciato riduce le false Frasi del corpus live senza perdere le risposte brevi o la voce bassa designate come regressioni; Più sensibile preserva quelle regressioni. La configurazione viene corretta sulla base dei risultati, senza blacklist testuali o promesse universali.
- [x] Una prova in release combina due Ingressi, denoiser, nuova protezione, ASR e salvataggio. Hardware, durata, memoria, tempo di elaborazione, andamento delle code e perdite sono riportati; il tratto stabile tiene il passo senza accumulo crescente o disallineamenti. Le prove con modelli reali sono sequenziali.
- [ ] Dopo Stop, Tape riaperto, player, testo e analisi finale dei Parlanti sono coerenti con l'audio salvato. La Diarizzazione non viene avviata durante la cattura.
- [x] Guasti della pulizia continuano a produrre bypass e avviso del solo Ingresso/sessione interessati. La protezione resta definita anche in bypass; nessuna inferenza/attesa aggiuntiva entra nella callback WASAPI.
- [x] Contratti generati, requisiti e decisioni riflettono il comportamento live verificato. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi. Collaudi nativi mancanti sono dichiarati e non considerati passati. Nessun commit o rilascio autonomo.

## Comments

Entrambi i predecessori sono necessari: percorso audio della Registrazione e protezione calibrata. Il ticket non riapre la segmentazione del testo.

### Esito del 6 ottobre 2026

Implementazione streaming con ammissione irrevocabile della candidata sui
prefissi Silero/ZCR. `Mixer::protect` fissa la sensibilità prima di
ricampionamento, buffer DFN3 e code ASR; timeline per Ingresso/sessione.
Un cambio della sola sensibilità conserva PCM e parola ammessa. Il backend
usa i profili persistenti, anche in Pausa e a cavallo di Stop. Nessun nuovo
lavoro nella callback; Diarizzazione di produzione resta dopo Stop e fine ASR.
Decisione: [ADR-0024](../../../docs/adr/0024-protezione-prudente-dal-vivo.md).

Sei controlli verdi: typecheck, **119 frontend**, lint frontend, fmt,
clippy su tutti i target con `-D warnings`, **296 Rust**, 29 smoke ignorati.
Binding rigenerato senza delta. Review Standards senza difetti bloccanti;
Spec ha rilevato limiti delle misure, corretti o dichiarati:
[review](../verification-06/review.md).

Matrice streaming con tre ASR: **288 inferenze reali**, 144 configurazioni
sintetiche distinte ripetute con etichette Microfono/Sistema sullo stesso PCM,
un feed per esecuzione. Non prova simultaneità né due dispositivi distinti.
Per profilo, sommando pulizia on/off, Bilanciato porta false Frasi Whisper
da **2 a 0**; Nemotron e Parakeet già erano a 0. Livelli prudenti: **0 parole
perse rispetto a Spento**, incluso «C No.» di Parakeet già nella baseline.
Più selettivo: **14 parole perse** della TTS attenuata per modello/profilo.
Nemotron conserva i Parziali reali della baseline sul parlato lungo.
Criterio prudente spuntato soltanto per queste regressioni sintetiche.

Banco WASAPI release finale: Ryzen 7 3700X (8 core/16 thread), 64 GB RAM,
RTX 2070 SUPER, Windows 11 Pro 10.0.26200; Microfono KLIM Talk e loopback
Focusrite, entrambi 48 kHz stereo. **28 s** di cattura, Pausa 12–13 s,
audio salvato **27.030 ms**, due DFN3 e due Nemotron concorrenti.
Cambi dei due profili via **SettingsStore temporaneo → Recorder::set_audio**,
in cattura/Pausa e congelamento dopo Stop. Store utente e Libreria preservati.

Mixer/DSP, controlli e scarico finale: **9,494 s**, **0,351 s/s** audio;
somma che esclude carico ASR, pre-caricamenti e writer concorrenti.
Code ASR campionate ogni secondo: picco **660 ms per Ingresso** al t=12,
entrambe a 0 al t=13 e alla fine, senza accumulo crescente nel tratto misurato.
Pool cattura massimo campionato **[5, 1]**, frame persi **[0, 0]**.
Microfono: nessuna Frase; Sistema: **4 Frasi e 12 Parziali** sul TTS loopback.
RAM dopo worker: working **134.234.112 B**, private **397.643.776 B**,
picco working di processo **405.663.744 B**, senza misura VRAM.
Tape riaperto uguale al documento, Forma d'onda uguale, tre tracce decodificate
con durata entro 1 ms. Prova passata in **31,59 s**, compilazione esclusa.

Restano aperti i due criteri non spuntati: corpus **umano** con respiri/voce
bassa/risposte brevi e catture WASAPI con tutti i tre ASR; player visibile,
ascolto, UI Tauri/tastiera/Narrator e analisi finale reale dei Parlanti.
Questa prova breve non attesta ore di lavoro o altri PC. Licenza dei pesi e
bundle restano aperti dal 02. Stato `partial`, nessun commit/PR/installer/rilascio.
Il 07 resta in attesa. Dettagli, log e denominatori:
[verifica](../verification-06/README.md), [handoff](../handoff-06.md).
