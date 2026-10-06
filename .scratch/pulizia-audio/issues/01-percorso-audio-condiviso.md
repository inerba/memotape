# 01: Preparare un percorso audio condiviso senza cambiare il risultato

**What to build:** preparare il punto comune di elaborazione dei blocchi audio, così la successiva pulizia potrà raggiungere audio salvato e Trascrizione senza creare percorsi divergenti. Questo è il solo ticket di preparazione: non introduce ancora controlli o un denoiser.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [ ] Registrazione, importazione di file audio/video e Trascrizione di Tape attraversano un contratto condiviso per blocchi PCM, con un comportamento iniziale di passaggio invariato. Audio salvato, tracce, mix, Forma d'onda e ASR conservano il risultato attuale.
- [ ] Il componente del core gestisce formato, posizione temporale, chiusura e confini di configurazione senza dipendere da Tauri. L'interfaccia consente un successivo processore con ritardo e stato per Ingresso; non introduce una nuova astrazione per ciascuna operazione DSP.
- [ ] L'elaborazione avviene prima della diramazione verso salvataggio e ASR, e prima della somma degli Ingressi. L'ordine conversione/Guadagno, elaborazione, somma, destinazioni è esplicito; il Guadagno mantiene il comportamento precedente.
- [ ] Frequenze diverse, mono/stereo, buchi, loopback fermo, Pausa/Riprendi e Stop mantengono durata e allineamento secondo le tolleranze già richieste dall'app. La callback WASAPI non esegue elaborazione o attese aggiuntive.
- [ ] Test dal blocco alle destinazioni usano le seam esistenti di motore e VAD e al massimo una nuova seam alta del processore. Un processore deterministico con trasformazione e ritardo riconoscibili dimostra che le destinazioni ricevono lo stesso risultato e che la chiusura conserva la coda utile.
- [ ] Confronti PCM prima del codec e confronti con tolleranza dopo Opus verificano il comportamento osservabile; non si richiede identità bit per bit a un codec lossy.
- [ ] La Diarizzazione della Registrazione resta dopo Stop e completamento ASR. Nessuna modifica a segmentazione linguistica, aggregazione delle Frasi o comportamento dei Parlanti.
- [ ] Le decisioni architetturali documentano il contratto introdotto. Il prodotto non dichiara disponibile una pulizia ancora assente.
- [ ] Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust sono verdi; una sola build Rust alla volta. Il resoconto distingue controlli automatici ed eventuali prove native mancanti. Nessun commit o rilascio autonomo.

## Comments

Nessuna implementazione avviata. Ticket di preparazione richiesto prima delle successive integrazioni verticali.
