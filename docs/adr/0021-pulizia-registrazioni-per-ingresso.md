---
status: accepted
---

# Pulizia delle Registrazioni separata per Ingresso

Il ticket 03 della spec `.scratch/pulizia-audio/spec.md` collega i profili
Microfono e Audio di sistema al percorso PCM di ADR-0018 e a DFN3 di ADR-0019.
Il profilo File e audio misto resta indipendente. Nessuna protezione VAD
aggiuntiva, toolbar, AGC o nuova Diarizzazione durante la cattura.

Ogni Ingresso selezionato ha un `ConfiguredCleaning` e un runtime indipendenti.
L'ordine resta conversione dei canali, Guadagno, ricampionamento alla frequenza
della Registrazione, pulizia, somma e destinazioni. Tracce, mix e ASR condividono
il risultato; la Forma d'onda viene dal writer del mix. La Diarizzazione usa
gli Ogg dopo Stop e completamento ASR, come prima.

Il worker prepara i runtime degli Ingressi selezionati prima di aprire WASAPI,
anche quando la pulizia è spenta: un'accensione durante cattura non carica né
ottimizza il modello. Questo costa tempo e memoria all'avvio. Pausa e cambio
scaricano il segmento e riusano il piano Tract vergine già ottimizzato, con
stato PCM/STFT/ricampionamento nuovo. Non si forza `Send` sullo stato Tract;
il suo thread dedicato e il canale sincrono limitato restano quelli di ADR-0019.

Il mixer scarica prima il ricampionatore a monte con il vecchio valore del
controllo, poi il processore. Alla ripresa crea il nuovo tratto senza includere
la Pausa. Il ricampionatore conserva soltanto due chunk di contesto e un periodo
della griglia razionale: il pre-roll parte da un multiplo di `Fs/gcd(Fs,Fout)`
e non viene consegnato due volte. La fase e la durata globale restano continue,
senza tagliare campioni per correggere gli arrotondamenti di ciascun segmento.
I valori applicati al processore sono pubblicati dal worker, distinti
dalle richieste atomiche dei controlli. Impostazioni e avvio pubblicano i
controlli sotto lo stesso lock del `Recorder`; Stop congela i loro valori.

Nelle Registrazioni il filtro conserva temporaneamente soltanto il PCM ancora
in attesa, entro il limite del processore e un blocco ricevuto. Se caricamento,
inferenza o scarico falliscono, scarta l'output parziale della chiamata fallita,
consegna quella coda originale senza spostarne la posizione e resta in bypass
per il resto della sessione sull'Ingresso interessato. Un filtro assente
preparato a controllo spento avvisa soltanto alla prima richiesta effettiva.
Gli errori nei file restano fatali e non diventano bypass.

`RecordingCleaningFailed` porta UUID, Ingresso e errore. La vista conserva gli
avvisi dei due Ingressi, ignora sessioni diverse e li annuncia tramite il banner
accessibile esistente. I sei dizionari e le etichette native coprono i controlli
e l'avviso. La preferenza rimane salvata; il bypass è uno stato della sessione.

`TrattoPulizia` descrive soltanto frame elaborati consegnati con successo:
la coda recuperata non viene marcata pulita. I metadati arrivano al Tape dopo
la chiusura degli Ingressi. Da un solo dispositivo, il Tape conserva il solo
mix ma il metadato identifica il dispositivo trattato: il ticket 04 dovrà
considerare anche questi intervalli quando riusa un Tape misto. I documenti
precedenti e il loro schema v1 restano invariati.

I test usano la sola seam `AudioProcessor`, oltre a motore e VAD esistenti,
e verificano destinazioni, metadati, persistenza, cambi, code e guasti.
Le prove native e i loro limiti sono registrati nel ticket 03. La misura locale
non prova il comportamento su altri PC; UI reale, bundle e licenza dei pesi
restano verifiche distinte. Nessun commit, installer pubblicato o rilascio.
