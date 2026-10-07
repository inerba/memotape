---
status: accepted
---

# DeepFilterNet3 sul PCM importato prima delle destinazioni

## Revisione del 6 ottobre 2026: uscita diretta

Dopo il confronto su `campioni/restaraunt_noisy.mp3`, l'utente sceglie
l'uscita DFN3 diretta ascoltata in `diagnosi-rumore/03-dfn3-diretto.wav`.
Questa revisione sostituisce la miscela descritta sotto: si eliminano la curva
SNR triangolare, il limite aggiuntivo di 6 dB, lo smoothing del coefficiente
e i crossfade verso il PCM originale. Nessun nuovo filtro, AGC o soglia prende
il loro posto. Le soglie native del modello e il post-filter spento restano.

La miscela precedente attenuava l'energia complessiva del campione di soli
0,675185 dB, contro 4,834699 dB del modello diretto, e lasciava invariato un
ticchettio sintetico annullato dal modello. Queste sono misure del segnale
misto, non una valutazione universale della qualità o della conservazione voce.

La scelta sostituisce anche la garanzia sperimentale di onset Silero invariato
sulla fixture a -30 dB: l'uscita diretta può attenuare voce molto debole. Il test
ne misura lo scostamento senza modificare Silero o la protezione indipendente.
Restano compensazione del ritardo, formato, durata esatta, scarico della coda,
contesto alle transizioni e stato separato per Ingresso/canale. L'audio consegnato
alle destinazioni è quello elaborato, anche ai bordi del tratto.

Il percorso condiviso vale per file, Tape e Registrazioni, come esteso dai
ticket successivi. I nuovi intervalli usano la versione `direct-v1`; quelli
precedenti conservano la loro versione e continuano a essere riusati senza una
seconda pulizia automatica. Il PCM originale resta soltanto nei buffer necessari
al recupero dopo guasto, senza miscela nella normale uscita o traccia archiviata.

La prova riproducibile usa `PcmStream`/`DeepFilter` e confronta tutta l'uscita,
compresi inizio e fine, con il riferimento PCM16 conservato prima della modifica.
Gli artefatti nuovi sono in `.scratch/pulizia-audio/dfn3-diretto/`.

## Decisione iniziale, sostituita per la miscela dalla revisione sopra

Il ticket 02 della [spec](../../.scratch/pulizia-audio/spec.md) usa il contratto
di ADR-0018 soltanto per l'importazione di audio e video. Registrazioni e
Trascrivi su un Tape conservano il percorso precedente. Tre profili persistenti
contengono `pulizia`, con default falso; il solo controllo utilizzabile è File
e audio misto, letto all'arrivo di ciascun blocco prima del VAD e della coda ASR.

libDF 0.5.6 è fissato alla revisione
`978576aa8400552a4ce9730838c635aa30db5e61`, senza feature di training, Python o
preprocessing; l'inferenza usa Tract 0.19.16 su CPU. Il modello standard ONNX
ha 7 983 136 byte e SHA-256
`c94d91f70911001c946e0fabb4aa9adc37045f45a03b56008cb0c8244cb63616`.
Si distribuisce come risorsa del bundle locale e si verifica prima del caricamento.
Assenza, incompatibilità e guasto hanno errori distinti, senza bypass silenzioso.

Il riferimento del limite di attenuazione è il PCM originale, nello stesso
formato della Sorgente. La ricostruzione STFT e il doppio ricampionamento non
sono un riferimento neutro: sulla fixture italiana attenuata di 30 dB,
la ricostruzione con limite spettrale a 0 dB conserva durata e allineamento
ma sposta l'onset Silero da 480 a 1800 ms. Non è un taglio fisico del PCM:
sono cambiamenti del segnale che fanno escludere il primo tratto dal VAD.

L'attenuazione interna varia da 0 a 6 dB con la stima SNR relativa di DFN3,
con una curva triangolare fra le soglie originali `min_db_thresh=-10`,
`max_db_df_thresh=20` e `max_db_erb_thresh=30`. La trasformazione cresce fino
allo stadio completo e poi scende a zero quando il runtime salterebbe anche
le maschere ERB. Gli hop considerati già puliti usano il PCM originale:
riportarvi il segnale ricostruito può alterare una risposta breve anche con
ritardo compensato e potenza quasi invariata. Quando il modello non distingue il parlato dal rumore,
prevale il riferimento originale; non è un limite globale quasi nullo.
La miscela è riallineata al PCM originale dopo il ricampionamento.
La stima SNR può oscillare durante un attacco breve: il coefficiente di miscela
si stabilizza con una costante di tempo di circa 100 ms. L'inizio e la fine del
tratto hanno inoltre un crossfade di 100 ms verso il PCM originale. Questi sono
due meccanismi distinti, non una promessa di latenza o di conservazione VAD di
100 ms. Non cambia Silero e non introduce AGC, normalizzazione dell'audio salvato
o una nuova soglia energetica.
Soglie e post-filter del runtime restano quelli originali, con post-filter spento.

Un runtime mono indipendente per canale conserva stato fra blocchi:
il multicanale non viene sommato dal denoiser. Il ricampionamento del core
porta a 48 kHz e ritorna al formato originale; il salvataggio Ogg usa poi
le impostazioni già previste per l'importazione.

`DfTract::process` della revisione fissata restituisce immediatamente zeri
con energia media sotto `1e-7`, senza avanzare lo stato. Inviare soltanto zeri
a fine file perderebbe la coda. L'adattatore usa le API pubbliche originali
`process_raw`, analisi, maschera e sintesi, riproducendo il percorso spettrale
senza quella scorciatoia. Non modifica grafo, pesi o runtime upstream e non
aggiunge un segnale pilota. Il confronto nativo su audio non silenzioso misura
una differenza inferiore a `1e-6` rispetto al percorso originale.

Ogni hop di 480 campioni avanza anche sul silenzio. Si compensa il ritardo
di 1440 campioni a 48 kHz e si scarica il padding senza aggiungerlo alla durata.
Una transizione di 100 ms fra audio originale ed elaborato attenua gli scatti
all'attivazione e disattivazione. Il PCM originale resta nel buffer transitorio
necessario al riferimento e alla compensazione del ritardo, senza una seconda
traccia archiviata.

Tract 0.19.16 conserva stato `Rc` e `OpState` non `Send`. Ogni processore
crea il runtime nel proprio thread e trasferisce soltanto PCM e risultati
con canali limitati. Non si dichiarano `unsafe impl Send` né una nuova seam
di runtime: `AudioProcessor` resta il solo confine sostituibile.

Il medesimo PCM passa alla copia Ogg e al ricampionamento mono per Silero/ASR
e per gli eventuali Parlanti. La Forma d'onda si ricava dalla copia Ogg.
`pulizia_audio` è facoltativo nel Tape v1: una lista di Ingresso, algoritmo,
versione, frequenza e intervallo semiaperto in frame della Sorgente. La
frequenza specifica l'unità anche quando l'Ogg è ricampionato. Le lacune
descrivono tratti non elaborati; la lista supporta più Ingressi, ma un file
importato usa esclusivamente Mix. Le riscritture testuali conservano i dati.

Codice libDF MIT e Tract MIT/Apache-2.0 hanno testi e attribuzioni in bundle
e Informazioni. Il README upstream non chiarisce la licenza dei pesi:
le richieste [700](https://github.com/Rikorose/DeepFilterNet/issues/700) e
[697](https://github.com/Rikorose/DeepFilterNet/issues/697) restano aperte
alla verifica del 6 ottobre 2026. Le note del modello dichiarano questa
limitazione: il criterio di licenza dei pesi non è soddisfatto e la prova
locale non equivale ad autorizzazione alla redistribuzione pubblica.
