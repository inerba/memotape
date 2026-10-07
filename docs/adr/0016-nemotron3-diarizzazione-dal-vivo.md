---
status: accepted
---

# Nemotron 3 Diarization anche durante la Registrazione

L'utente vuole migliorare l'attribuzione del testo rispetto a Sortformer 4spk v2.1, distinguere fino a 8 Parlanti e vederli già durante la Registrazione. Questo riapre ADR-0006: la sola Diarizzazione a posteriori non soddisfa il requisito. Si conserva il perimetro degli Ingressi di ADR-0015.

Per la prova si usa il port della PR #175 di transcribe.cpp, fissato al commit `e6672a8672913b47f1571c66c54bee789d028416` insieme ai binding. È una scelta sperimentale accettata nella progettazione, perché evita una seconda integrazione di runtime mentre permette di provare lo streaming senza aspettare una release upstream. Si devono verificare Windows x64, Vulkan e la regressione sui tre modelli ASR esistenti; il solo cambio del GGUF non basta. Il modello BF16 viene preparato localmente con il converter di quel commit dal checkpoint NVIDIA `f667ed73aee57d40cc39428eb768b4fd87a0a29e`, registrando origine, dimensione e SHA-256. Non si assume compatibilità dei GGUF NVIDIA/NeMo-Speech.cpp o Glimpse e non si pubblicano pesi durante la prima prova.

L'obiettivo dal vivo è vedere il Parlante entro circa 1–2 secondi, con attribuzioni provvisorie riconoscibili e correggibili automaticamente. Il testo si divide anche quando due persone si alternano dentro la stessa Frase: attribuire tutto alla voce prevalente manterrebbe un limite dell'attuale comportamento. Con Nemotron e Parakeet le divisioni si appoggiano ai tempi dei token/parole quando affidabili; con Whisper si conservano i segmenti e si lascia non determinata l'attribuzione dei tratti con più voci non separabili. Parakeet e Whisper continuano a mostrare il testo alla chiusura della Frase. La fattibilità e il ritardo vanno misurati nell'app, includendo il tempo di Trascrizione; la latenza di buffer pubblicata dal modello non li garantisce.

Dopo Stop si completa lo stream e si esegue una seconda Diarizzazione più accurata sull'audio salvato, senza ritrascrivere il testo. Se il riconoscimento dal vivo fallisce o non tiene il passo, audio e Trascrizione continuano; si mostra un avviso e si riprova a posteriori quando possibile. Nei tratti con voci sovrapposte le parole non attribuibili restano nel testo come "Parlante non determinato": separare il parlato simultaneo è fuori dalla prima prova.

Nemotron è un'alternativa sperimentale selezionabile, con Sortformer ancora disponibile per il confronto. Questo riapre anche la scelta di ADR-0006 di non avere un selettore finché esiste un solo algoritmo.

Pausa/Riprendi conserva la sessione e le identità; i nomi si possono cambiare solo quando l'analisi finale è terminata. La Continuazione e il riconoscimento di una persona fra sessioni restano fuori da questa prova. Annullare la seconda Diarizzazione, o un suo errore, conserva audio, testo e attribuzioni disponibili, distinguendo quelle provvisorie e indicando "Diarizzazione non completata". Questo stato si conserva nel Tape separatamente dalla completezza della Trascrizione, con metadati facoltativi compatibili con i Tape vecchi.

## Considered Options

- **Aspettare una release upstream:** più stabile, ma rinvia la prova richiesta; resta un percorso successivo se il commit sperimentale non supera le verifiche.
- **NeMo-Speech.cpp via API C:** runtime ufficiale già disponibile, ma introduce binding, packaging e convivenza di due runtime ggml. Non è un cambio automatico se la PR fallisce.
- **Fork Glimpse e suoi GGUF:** dipendenza e conversione divergenti dal port upstream scelto.
- **Un solo Parlante per Frase:** conserva la semplicità attuale, ma mantiene errori visibili quando le voci si alternano nella stessa Frase.
- **Allineatore aggiuntivo per Whisper:** potrebbe consentire divisioni più precise, ma amplia modello, dipendenze e costo; nella prova si sceglie il comportamento conservativo.

Le decisioni sono state confermate il 2026-10-05. La [spec approvata](../../.scratch/nemotron3-diarizzazione/spec.md) descrive salvataggio, contratto e verifiche ancora da eseguire; la [ricerca](../research/diarizzazione.md#9-verifica-del-2026-10-05-per-la-prova-in-memotape) distingue fatti verificati e aspetti da misurare. "Accepted" riguarda la progettazione della prova: non attesta qualità, prestazioni o funzionalità implementate. Questa fase termina ai documenti e richiede un successivo via libera per scrivere codice.


## Adozione sperimentale — 6 ottobre 2026

L’utente ha autorizzato «integrala allora» dopo aver esaminato gli esiti del ticket 08. La funzione implementata è disponibile come scelta sperimentale esplicita, con Sortformer predefinito, modello locale compatibile e avvisi nelle sei lingue. La GPU Vulkan è consigliata per il dal vivo; non si impone un blocco sulla CPU. Il fallimento realtime CPU sul PC provato è documentato e resta gestito dall’avviso e dall’analisi finale, preservando audio e testo.

Il supporto architetturale fino a otto Parlanti non garantisce che siano tutti distinti correttamente. Il confronto su corpus costruito favorisce Nemotron, ma usa confini energetici automatici, non annotazioni manuali di una conversazione spontanea. L’adozione non chiude il ticket 08 e non certifica UI, altri PC o Registrazioni lunghe. [Prove e limiti](../../.scratch/nemotron3-diarizzazione/report08-addendum.md).

## Revisione del 6 ottobre 2026: solo dopo Stop

L’utente ha chiesto di avviare la Diarizzazione soltanto dopo Stop, perché il percorso dal vivo non tiene il passo sul PC in uso. Questo sostituisce la parte di questa decisione che prevedeva uno stream dei Parlanti durante la cattura; restano la scelta sperimentale del modello, gli Ingressi separati e l'analisi finale sull'audio salvato.

La Registrazione avvia soltanto i canali ASR: niente coda, caricamento o thread del diarizer dal vivo. Dopo Stop e lo smaltimento ASR si analizzano gli Ogg degli Ingressi selezionati in sequenza, senza ritrascrivere il testo. La scelta e la validazione dell'artefatto restano fissate all’avvio; eventuali errori di disponibilità fanno parte dell’esito finale. I Tape precedenti conservano la lettura dei metadati provvisori già salvati. Gli smoke realtime del core restano prove sperimentali, non attestano il comportamento corrente della Registrazione nell’app.

## Revisione dell'8 ottobre 2026: via lo strato dei Parlanti dal vivo

Dopo la revisione del 6 ottobre lo strato realtime non è più raggiungibile: `live_sources` crea ogni `LiveSource` con `diarizer: None`, `LiveDiarizer` non ha costruttori e `LiveDiarizationFailed` viene emesso solo da `LiveSession`. Tenerlo come "prova sperimentale" raddoppiava i rami di `transcribe_live` e di `finalize_live_ingressi`, imponeva al frontend un protocollo di revisioni per Ingresso che nessun percorso produce e ha già fatto perdere la protezione del parlato (`LiveSession::run` chiama `pipeline::transcribe` senza timeline). Si elimina, insieme al banco del ticket 08 e agli smoke nativi dei ticket 05–07, che restano nella storia git e nei report.

Restano l'analisi finale dopo Stop, `diarize::divide` e la lettura dei campi facoltativi del Tape v1 (`parlante_provvisorio`, `Diarizzazione::ingressi`) già salvati. `LiveTranscriptUpdated` diventa il solo snapshot finale di un Ingresso (`sessionId`, `ingresso`, `phrases`), emesso da `final_speakers` per le Registrazioni. Un diarizer assente o incompatibile resta parte dell'esito finale, come chiede PRODUCT.md. Ticket: `.scratch/attivita-frontend/issues/01-via-lo-strato-dei-parlanti-dal-vivo.md`.
