# Ricerca: segmentazione in WhisperX e Buzz

Data: 2026-10-06. Scopo: confrontare il livello dei segmenti ASR, delle frasi
linguistiche, dei paragrafi e dell'assegnazione dei Parlanti in due prodotti
locali/open source. Non è una misura di qualità e non propone ancora una
modifica a Memotape.

## Fonti e limiti

Sono stati letti i repository ufficiali ai commit `whisperX@771b4a14a9486f8fd5aef18ef49e35d639523dd3`
e `buzz@100589fd8c492869f2994b525e1092eda81e4e88`, più la documentazione
ufficiale di Buzz. Le fonti descrivono il comportamento del codice corrente;
non dimostrano che ogni versione distribuita o ogni backend produca gli stessi
segmenti. Non sono stati eseguiti benchmark o prove su audio italiano reale.

## Fatti verificati

### WhisperX

- Il primo livello è un segmento di parlato prodotto da VAD e `merge_chunks`;
  il trascrittore batched lavora su quei chunk e, di default, esegue Whisper
  senza timestamp (`without_timestamps=True`).
  [Codice ASR, VAD e costruzione dei segmenti](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/whisperx/asr.py#L262-L369)
  e [opzioni di decodifica](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/whisperx/asr.py#L474-L493).
- L'allineamento successivo introduce un livello diverso: per ogni segmento
  usa NLTK `sentence_splitter.span_tokenize(text)`, calcola i tempi di parole e
  crea subsegmenti con un intervallo per ciascuna frase linguistica.
  [Codice di sentence splitting e subsegmenti](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/whisperx/alignment.py#L136-L194)
  e [tempi e risultato finale](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/whisperx/alignment.py#L305-L388).
- La diarizzazione non crea automaticamente un nuovo confine testuale: interseca
  gli intervalli dei Parlanti con ogni segmento e sceglie il Parlante con la
  durata di sovrapposizione maggiore; applica la stessa regola alle parole che
  hanno tempi. `fill_nearest` è falso per default.
  [Assegnazione dei Parlanti](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/whisperx/diarize.py#L168-L234).
- WhisperX è una libreria/CLI: nelle fonti ufficiali consultate non c'è un
  editor con comandi manuali di dividi/unisci. Il README documenta output e
  pipeline, e dichiara che la sovrapposizione e la diarizzazione restano limiti.
  [Pipeline ufficiale](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/README.md#L291-L334)
  e [limiti dichiarati](https://github.com/m-bain/whisperX/blob/771b4a14a9486f8fd5aef18ef49e35d639523dd3/README.md#L348-L390).

### Buzz

- Il modello interno è un `Segment` con `start`, `end`, `text` e `speaker`.
  Nei backend Whisper/Faster-Whisper la modalità con tempi di parola può
  trasformare ogni parola in un segmento; senza quella modalità resta il
  segmento restituito dal backend.
  [Modello `Segment`](https://github.com/chidiwilliams/buzz/blob/100589fd8c492869f2994b525e1092eda81e4e88/buzz/transcriber/transcriber.py#L29-L36)
  e [conversione dei risultati](https://github.com/chidiwilliams/buzz/blob/100589fd8c492869f2994b525e1092eda81e4e88/buzz/transcriber/whisper_file_transcriber.py#L339-L373).
- L'assegnazione dei Parlanti è un'operazione successiva alla trascrizione.
  La finestra raccoglie il testo dei segmenti, lo riallinea a parole, aggiunge
  la punteggiatura quando disponibile e costruisce le frasi per la mappatura dei
  Parlanti. Al salvataggio, la casella **Merge speaker sentences**, attiva di
  default, fonde tutti i segmenti consecutivi dello stesso Parlante in un solo
  segmento, senza un limite di durata documentato nel codice.
  [Flusso di identificazione](https://github.com/chidiwilliams/buzz/blob/100589fd8c492869f2994b525e1092eda81e4e88/buzz/widgets/transcription_viewer/speaker_identification_widget.py#L176-L199),
  [mappatura e punteggiatura](https://github.com/chidiwilliams/buzz/blob/100589fd8c492869f2994b525e1092eda81e4e88/buzz/widgets/transcription_viewer/speaker_identification_widget.py#L356-L397)
  e [unione](https://github.com/chidiwilliams/buzz/blob/100589fd8c492869f2994b525e1092eda81e4e88/buzz/transcriber/speaker_identifier.py#L12-L64).
- Buzz documenta un livello di visualizzazione/export separato: TXT apre un
  nuovo paragrafo quando il vuoto tra segmenti raggiunge 2 secondi
  (`BUZZ_PARAGRAPH_SPLIT_TIME`, configurabile); SRT/VTT mantengono un elemento
  per segmento. La funzione **Resize** può ricombinare sottotitoli per durata
  massima e, se esistono tempi di parola, dividerli sulla punteggiatura.
  [Export TXT/SRT/VTT](https://github.com/chidiwilliams/buzz/blob/100589fd8c492869f2994b525e1092eda81e4e88/buzz/transcriber/file_transcriber.py#L231-L274)
  e [documentazione Resize](https://chidiwilliams.github.io/buzz/docs/usage/edit_and_resize).
- La documentazione del viewer descrive anche la regolazione manuale degli
  estremi temporali con comandi da tastiera; nelle fonti consultate non è
  documentato un comando manuale equivalente per dividere o unire liberamente
  il testo. L'unione esplicita verificata è quella automatica dei segmenti
  consecutivi dello stesso Parlante.
  [Viewer e regolazione dei tempi](https://chidiwilliams.github.io/buzz/docs/usage/transcription_viewer#timestamp-adjustment)
  e [speaker identification](https://chidiwilliams.github.io/buzz/docs/usage/speaker_identification).

## Indicazione per Memotape

Il confronto conferma che “segmento ASR”, “frase linguistica”, “turno del
Parlante” e “paragrafo visualizzato” sono livelli diversi. WhisperX conserva i
chunk VAD come base, ma aggiunge frasi usando la punteggiatura durante
l'allineamento; Buzz offre una scelta esplicita di fusione dei turni dello
stesso Parlante e separa i paragrafi TXT dalla lista dei segmenti. Nessuna delle
due fonti dimostra una regola universale per il parlato misto: la gestione delle
sovrapposizioni e l'assegnazione dei Parlanti restano limitate o probabilistiche.
