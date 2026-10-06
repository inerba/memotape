# Memotape

App desktop che trasforma in testo, interamente in locale, il parlato di file audio, video e registrazioni fatte dal computer.

## Language

### Sorgenti e attività

**Sorgente**:
Il file audio, video o Tape selezionato, su cui agiscono le Attività. Il Tape prodotto da una Registrazione o da una Trascrizione diventa la Sorgente.
_Avoid_: input, file sorgente, audio sorgente

**Attività**:
Una tra Registrazione e Trascrizione. Ne è in corso al massimo una alla volta; il download di un modello non è un'Attività.
_Avoid_: job, task, operazione

**Registrazione**:
L'Attività che cattura microfono, audio di sistema o entrambi in un unico file.
_Avoid_: recording, cattura

**Continuazione**:
Una Registrazione che, invece di creare un Tape nuovo, accoda il suo audio e le sue Frasi a un Tape esistente; il Tape ricorda dove e quando comincia ogni Continuazione.
_Avoid_: ripresa, riprendere (è il contrario di Pausa), append, prosecuzione

**Trascrizione**:
L'Attività che trasforma in testo il parlato della Sorgente.
_Avoid_: sbobinatura, riconoscimento

**Trascrizione dal vivo**:
La Trascrizione fatta durante una Registrazione, attivata da un'impostazione; fa parte della Registrazione e non è un'Attività a sé.
_Avoid_: live, trascrizione in tempo reale

**Ingresso**:
Il microfono o l'audio di sistema (loopback del dispositivo di uscita) catturati da una Registrazione; la sorgente di registrazione nelle impostazioni dice quali (Microfono, Audio di sistema, Entrambi).
_Avoid_: sorgente (è la Sorgente), input

**Guadagno**:
Quanto Memotape alza o abbassa l'audio di un Ingresso durante una Registrazione, prima di unirlo agli altri: cambia l'audio salvato e quello trascritto, non il volume di Windows né quello delle altre app.
_Avoid_: volume (è quello del player o di Windows), gain, livello (è quello che mostra l'indicatore)

**Tape**:
Il file `.tape` prodotto da una Registrazione o dalla Trascrizione di un file audio o video: un archivio zip con l'audio del mix, l'audio di ogni Ingresso (quando la Registrazione è da Entrambi), la Forma d'onda e il testo con i suoi metadati. Si apre come Sorgente; il suo titolo è il nome del file.
_Avoid_: Bino (il nome di prima), progetto, archivio, pacchetto, nastro, cassetta

**Forma d'onda**:
L'andamento del volume del mix di un Tape, dall'inizio alla fine, che il player mostra come barra di avanzamento; sta dentro il Tape, così riaprirlo non la ricalcola.
_Avoid_: waveform, picchi, cache

**Libreria**:
La cartella in cui Memotape salva i Tape e li tiene per ritrovarli, cercarli e riascoltarli: un Tape ne fa parte se sta lì dentro, e uno altrove si apre ma non compare.
_Avoid_: Cartella predefinita, archivio, storico, database

**Raccolta**:
Una cartella di primo livello della Libreria che raggruppa Tape, per esempio le call con un cliente; un Tape sta in una Raccolta o in nessuna.
_Avoid_: progetto, cartella, etichetta, fascicolo

**Assistente**:
Un'app di intelligenza artificiale esterna, come Claude o Codex, che con il permesso dell'utente cerca e legge i Tape della Libreria; non li modifica e non avvia Attività.
_Avoid_: agente, bot, integrazione, MCP

### Testo

**Frase**:
Un tratto di parlato delimitato dal VAD, trascritto come unità; con la Diarizzazione può essere diviso al cambio di Parlante quando i tempi del testo lo permettono. Ogni parte resta legata al proprio intervallo nell'audio salvato anche quando se ne corregge il testo.
_Avoid_: segmento, utterance, chunk

**Tempo del testo**:
Il legame fra un tratto del testo e l'intervallo audio realmente fornito dalla Trascrizione. Quel tratto resta indivisibile nell'attribuzione dei Parlanti; se comprende più voci, il suo Parlante resta non determinato.

**Parziale**:
Il testo provvisorio di una Frase ancora in corso. Dal vivo può essere mostrato in più parti ai cambi di Parlante sostenuti dai tempi ASR; una revisione sostituisce tutte le parti insieme. La Frase conclusa le sostituisce senza duplicazioni.
_Avoid_: anteprima, testo tentativo

**Ingressi separati**:
Il modo in cui si trascrive sempre una Registrazione da Entrambi, dal vivo o dopo: ogni Ingresso è trascritto per conto suo e le Frasi compaiono come una conversazione, ognuna con l'Ingresso da cui viene. Non è una scelta dell'utente.
_Avoid_: canali, flussi, modalità chat

**Diarizzazione**:
L'attribuzione di ogni Frase a un Parlante. Dal vivo attribuisce e suddivide anche i Parziali disponibili, in modo provvisorio e rettificabile; l’analisi finale dopo Stop consolida il risultato.
_Avoid_: speaker detection, riconoscimento dei parlanti

**Parlante**:
Una voce distinta individuata dalla Diarizzazione, numerata per ordine di comparsa. Con gli Ingressi separati appartiene sempre a un solo Ingresso e si numera per Ingresso: la stessa voce non è mai un Parlante del Microfono e dell'Audio di sistema insieme.
_Avoid_: speaker, voce, utente

**Parlante non determinato**:
Una Frase che la Diarizzazione non attribuisce a una voce unica, perché contiene più voci o non ha turni utilizzabili. Il testo si conserva senza inventare un'identità; è distinto dall'Ingresso non diarizzato, come il Microfono trattato come una persona sola.

**Attribuzione provvisoria**:
Un Parlante già attribuito a una Frase ma ancora rettificabile dall’analisi finale. Non riguarda la completezza del testo; dopo un’analisi annullata o guasta resta riconoscibile anche nel Tape e nelle copie.

**Analisi finale dei Parlanti**:
La Diarizzazione eseguita dopo Stop sull’audio salvato, quando la Trascrizione ha smaltito la sua coda. Non ritrascrive il testo. Il suo esito è distinto da `completa`; con gli Ingressi separati ogni Ingresso ha il proprio esito e si consolida indipendentemente. Il Tape conserva il risultato disponibile anche con Annulla o errore.

**Turno**:
Frasi consecutive (e Parziali) con lo stesso Ingresso e lo stesso Parlante; senza Parlanti né Ingressi separati, le Frasi fino a una pausa lunga. È l'unità in cui si legge il testo.
_Avoid_: spezzone, blocco, intervento, chat, paragrafo

**Correzione manuale**:
Una modifica dell'utente al testo della Trascrizione, all'attribuzione di un Turno o al nome di un Parlante. È distinta dall'attribuzione automatica della Diarizzazione e dalle modifiche al titolo o alla data del Tape.
_Avoid_: modifica del Tape, revisione automatica

**Unione di Turni**:
La correzione con cui l'utente attribuisce un Turno al Parlante del Turno adiacente, formando un unico Turno. Le Frasi conservano il loro testo e i loro intervalli audio; gli altri interventi del Parlante di partenza restano distinti.
_Avoid_: unione di Frasi, fusione di Parlanti, concatenazione del testo

### Lingue

**Lingua del parlato**:
La lingua in cui è pronunciata la Sorgente, indicata al modello o riconosciuta in automatico.
_Avoid_: lingua, lingua della trascrizione

**Lingua dell'interfaccia**:
La lingua dei testi dell'app.
_Avoid_: lingua, locale
