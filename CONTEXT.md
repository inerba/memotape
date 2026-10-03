# Sbobino

App desktop che trasforma in testo, interamente in locale, il parlato di file audio, video e registrazioni fatte dal computer.

## Language

### Sorgenti e attività

**Sorgente**:
Il file audio o video selezionato, su cui agiscono le Attività. Una Registrazione conclusa diventa la Sorgente.
_Avoid_: input, file sorgente, audio sorgente

**Attività**:
Una tra Registrazione e Trascrizione. Ne è in corso al massimo una alla volta; il download di un modello non è un'Attività.
_Avoid_: job, task, operazione

**Registrazione**:
L'Attività che cattura microfono, audio di sistema o entrambi in un unico file.
_Avoid_: recording, cattura

**Trascrizione**:
L'Attività che trasforma in testo il parlato della Sorgente.
_Avoid_: sbobinatura, riconoscimento

**Ingresso**:
Il microfono o l'audio di sistema (loopback del dispositivo di uscita) catturati da una Registrazione; la sorgente di registrazione nelle impostazioni dice quali (Microfono, Audio di sistema, Entrambi).
_Avoid_: sorgente (è la Sorgente), input

**Cartella predefinita**:
La cartella in cui si salvano le Registrazioni.
_Avoid_: cartella di output, destinazione

### Testo

**Frase**:
Un tratto di parlato delimitato dal VAD, trascritto come unità.
_Avoid_: segmento, utterance, chunk

**Parziale**:
Il testo provvisorio di una Frase ancora in corso.
_Avoid_: anteprima, testo tentativo

### Lingue

**Lingua del parlato**:
La lingua in cui è pronunciata la Sorgente, indicata al modello o riconosciuta in automatico.
_Avoid_: lingua, lingua della trascrizione

**Lingua dell'interfaccia**:
La lingua dei testi dell'app.
_Avoid_: lingua, locale
