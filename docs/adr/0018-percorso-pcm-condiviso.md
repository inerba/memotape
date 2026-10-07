---
status: accepted
---

# Un contratto PCM prima delle destinazioni audio

Il ticket 01 della [spec della pulizia audio](../../.scratch/pulizia-audio/spec.md)
prepara il percorso comune. Non introduce DeepFilterNet, protezione aggiuntiva
del parlato, impostazioni o controlli: il processore dell'app è `Bypass`.

`audio_toolkit::processing` resta indipendente da Tauri. `PcmStream` possiede un
solo `AudioProcessor`, il formato interleaved e le posizioni ricevuta/emessa in
frame dalla stessa origine. Il processore conserva formato, ordine e durata;
può trattenere una coda entro `max_pending_frames`, ma non aggiunge silenzio per
rappresentare il ritardo di elaborazione. Un adattatore futuro gestirà al proprio
interno frequenza di inferenza, buffer e compensazione del ritardo: non viene
introdotta una seam per ciascuna operazione DSP.

I confini `Pause`, `Configuration` e `Finish` scaricano tutta la coda utile.
I primi due consentono altro audio senza azzerarne l'origine; `Finish` chiude
la sessione. `reconfigure` consegna prima il risultato del vecchio processore,
quindi sostituisce lo stato per l'audio successivo. È un contratto del core,
ancora privo di controlli utente. Non si assume che inviare zeri a un runtime
equivalga a scaricarne i buffer.

Nella Registrazione ogni Ingresso possiede il proprio `PcmStream` nel mixer.
L'ordine è conversione dei canali, Guadagno con la rampa esistente,
ricampionamento al formato della Registrazione, elaborazione, somma e clamp,
destinazioni. Buchi e loopback fermo continuano a essere riempiti secondo QPC,
prima del processore; le pause sono escluse. Il mixer somma soltanto campioni
utili già elaborati e consegna lo stesso tratto alle tracce. Il worker scrive
quel PCM negli Ogg e lo passa ai canali ASR. Le callback WASAPI non cambiano.
Il ricampionatore del dispositivo conserva il suo ciclo di vita precedente:
la Pausa scarica la coda del processore, Stop scarica prima anche il resampler.

Nei file `FileFrames` elabora i blocchi decodificati prima della diramazione
verso `OggCopy` e VAD/ASR. L'ASR conserva la propria conversione mono a 16 kHz;
la copia conserva canali e frequenza delle impostazioni. La Forma d'onda viene
dal writer dell'audio salvato. La fine scarica prima il processore, poi copia
Ogg e ricampionatore ASR. Trascrivi su un Tape usa lo stesso percorso per il
mix o per ciascun Ingresso. Un cambio di formato nel medesimo flusso, già non
supportato, viene ora segnalato esplicitamente anziché interpretato nel formato
del primo blocco.

Un errore del contratto PCM si propaga. Nei file impedisce il completamento;
nel worker della Registrazione segue il percorso che chiude e conserva quanto
già acquisito. Bypass per guasto, avvisi e metadati di intervallo di un denoiser
restano ai ticket successivi: nessun filtro reale è presente qui.

La Diarizzazione resta dopo Stop e smaltimento ASR, come nella revisione
corrente di ADR-0016. Nessuna modifica ai confini delle Frasi o ai Parlanti.

## Verifica

Una sola nuova seam, `AudioProcessor`, affianca motore e VAD esistenti. Il
processore deterministico dei test dimezza il PCM e trattiene una coda finita.
Le prove verificano posizione, stereo, scarico, cambi di processore, mix e
tracce prima di Opus, PCM consegnato all'ASR, Forma d'onda e Tape riaperto.
Per il codec si confrontano durata e PCM con RMSE inferiore a 0,015, senza
richiedere identità binaria. I test del mixer coprono frequenze diverse,
Ingressi mono/stereo, buchi, loopback fermo, pause e Stop.

Queste prove del core non attestano qualità di pulizia, cattura WASAPI reale,
carico con DFN3/ASR, altri PC o installer. Il prodotto continua a descrivere
le funzionalità disponibili; la pulizia audio resta assente.
