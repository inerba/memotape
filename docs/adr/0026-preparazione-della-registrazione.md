---
status: accepted
---

# Preparazione della Registrazione e riuso della Pulizia audio

Decisione del 7 ottobre 2026, concordata con l'utente e implementata nel checkout
locale. Spec, diagnosi e verifiche in `.scratch/avvio-registrazione/`.

L'app distingue la Preparazione della Registrazione dall'avvio confermato
dell'audio. La barra mostra l'attesa e Annulla dal clic; conferma «puoi parlare»
e passa a timer e Stop solo quando Ingressi e scrittura sono pronti. ASR può
prepararsi in parallelo e fornire testo dopo; l'audio deve essere conservato
dall'avvio. Un annullamento che precede l'avvio non crea Tape né partenze tardive;
se l'avvio è già avvenuto, Stop conserva l'audio.

Si sostituisce il compromesso di ADR-0021 che prepara DFN3 a ogni Registrazione
anche da spento: il lavoro costoso viene anticipato in background e riusato
tra sessioni, accettando memoria residente aggiuntiva. Ogni Ingresso e sessione
devono avere stato audio nuovo e isolato. La preparazione non apre i dispositivi;
le risorse incompatibili con la configurazione vengono invalidate. Memoria e
tempi effettivi devono essere misurati prima e dopo sullo stesso hardware.

La pulizia richiesta al clic si attende, perché vale dall'inizio; quella spenta
non blocca l'audio. Se accesa durante la Registrazione mentre si prepara, la
barra mostra l'attesa e il filtro si applica appena pronto all'audio successivo,
senza interrompere la Registrazione. Spegnimento, Stop e annullamento impediscono
attivazioni tardive; Pausa conserva l'effetto dalla ripresa.

Guasti a pulizia o ASR consentono la Registrazione con un avviso esplicito;
dispositivi o scrittura non disponibili impediscono l'avvio. Le regole di
recupero dell'audio già acquisito restano valide. La Diarizzazione continua
solo dopo Stop. Non si introduce un segnale sonoro di avvio.

## Implementazione e limiti verificati

`RecordingFilters` conserva una sola configurazione di DFN3 (risorsa, formato,
numero di Ingressi). Il runtime Tract resta sul worker che lo crea: nessun
`unsafe impl Send`. Un prestito esclusivo restituisce il piano al termine, dopo
azzeramento di stato DSP e ricampionatori; anche un prestito abbandonato senza
Finish viene azzerato. Il piano ottimizzato è riusabile, lo stato audio no.
I guasti vengono riprovati alla configurazione successiva, senza sostituire
implicitamente il modello. Il caricamento già iniziato può concludersi in
background dopo Annulla, ma non apre dispositivi.

La Registrazione pubblica fasi con UUID. Annulla è registrabile anche prima
della pubblicazione dei controlli; un lock decide la corsa con l'avvio e
impedisce attivazioni pendenti dopo Stop. Il frontend conserva la vista
precedente fino alla conferma. Annulla durante i salvataggi torna subito,
mentre le scritture già richieste proseguono senza avviare la cattura.

La coda ASR contiene al massimo 2000 elementi per Ingresso (fino a 60 s di PCM,
circa 3,8 MB; le chiusure di Frase occupano un elemento). Non blocca la cattura:
a saturazione segnala subito un guasto della Trascrizione dal vivo e ne chiude
l'alimentazione; non riparte con buchi. Il Tape conserva l'audio e le Frasi già
concluse. Il caricamento ASR nativo non è interrompibile: Stop può ancora
attenderne la conclusione prima del Tape, con Ogg già acquisito sul disco.

Test DFN3 reale debug: preparazione 1,17–1,27 s; prestito già pronto 5,9–14,9 µs.
Output identico fra sessioni e dopo abbandono senza Finish. Questi tempi sono
misure del componente, non della latenza clic → primo audio. Reset e rilascio
sono sincroni sul worker; non è introdotto un timeout nativo. Memoria e limiti
del campionamento sono documentati nella verifica. Il collaudo browser usa IPC
simulato; non attesta WASAPI, italiano parlato o la matrice debug/release.
