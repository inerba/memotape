# Libreria e player: decisioni del grilling (2026-10-04)

Base per la spec. Termini in `CONTEXT.md` (Libreria, Raccolta, Bino, Frase…); scelte di fondo in ADR-0008 (Libreria = cartella, database = indice) e ADR-0009 (ogni file trascritto diventa un Bino).

## Libreria e Raccolte

- La Libreria è la cartella che prima era la Cartella predefinita (`Documenti\Sbobino`); in Impostazioni diventa "Cartella della Libreria". Cambiarla cambia Libreria; i Bini restano dove sono.
- Le Raccolte sono le cartelle di primo livello. Un solo livello: i Bini in sottocartelle più profonde contano nella Raccolta che le contiene. I Bini nella radice sono "Senza raccolta". `.sbobino` è esclusa.
- Le Raccolte si creano dalla barra laterale, si rinominano (rinomina la cartella), si eliminano solo se vuote.
- I Bini nuovi (Registrazione o Trascrizione di un file) vanno nella Raccolta scelta nella barra laterale; con "Tutta la Libreria" in "Senza raccolta". "Sposta in…" sposta il file.
- Un Bino fuori dalla Libreria si apre e si consulta (player compreso) ma non compare; "Aggiungi alla Libreria…" chiede la Raccolta e lo sposta.
- "Elimina" manda il Bino nel Cestino di Windows, con conferma. Mai cancellazioni definitive.
- Il titolo è il nome del file: un clic sul titolo lo rinomina (nome già presente nella Raccolta o caratteri non ammessi: non si conferma). I Bini nuovi mantengono il nome di oggi.

## Indice e ricerca

- Un database SQLite con FTS5 in `%LOCALAPPDATA%\it.sbobino.desktop`: indice ricostruibile, allineato confrontando data di modifica e dimensione dei Bini, all'avvio e quando si apre un Bino; ogni scrittura dell'app su un Bino aggiorna anche l'indice.
- Ricerca per parola e inizio di parola, senza maiuscole né accenti, su titoli, testo delle Frasi e nomi dei Parlanti; risultati per pertinenza.
- Il campo cerca nella Raccolta scelta; in fondo ai risultati "Cerca in tutta la Libreria". Ogni risultato mostra il Bino e le Frasi trovate con un estratto; il clic apre il Bino, scorre alla Frase, la evidenzia e porta lì il player senza avviarlo.

## Trascrizione di un file

- Crea `<nome del file>.bino` nella Raccolta, con l'audio in Opus alle impostazioni di Registrazione; l'originale non si tocca, il suo nome resta nel documento. Annullata o senza parlato: nessun Bino. Poi il Bino è la Sorgente; ritrascrivere = Trascrivi sul Bino.
- Il Markdown non si salva più da solo: "Esporta Markdown…" con il dialog di salvataggio. Copia testo resta. Cambiano le storie 16, 17, 21 e la riscrittura del Markdown alla rinomina dei Parlanti.

## Finestra

- Barra laterale sempre visibile, dall'alto: Registra ▾ e Apri file (audio, video, Bino); "Attività in corso" (solo se c'è; stato in breve, porta alla sua vista); selettore della Raccolta (Tutta la Libreria, Senza raccolta, le Raccolte, Nuova Raccolta); ricerca; i Bini della Raccolta dal più recente, raggruppati per data (Oggi, Ieri, Questa settimana, poi per mese) con titolo, ora e durata; in fondo "Tutti i Bini della Raccolta" (elenco completo nell'area principale, con ordinamento, Sposta in…, Elimina) e Impostazioni.
- La status bar resta: fase, percentuale, esiti, errori.
- Opzioni in menu a comparsa: Trascrivi ▾ (modello, Lingua del parlato, Riconosci i parlanti; il pulsante dice la scelta, "Trascrivi · Italiano"), Registra ▾ (Trascrivi dal vivo, Riconosci i parlanti). Le impostazioni complete restano in Impostazioni.
- Vista di un Bino, dall'alto: titolo; informazioni (data e ora, durata, Raccolta, modello, Lingua del parlato, Ingressi separati, incompleto); azioni (Trascrivi ▾, Copia testo, Esporta Markdown…, Mostra in Esplora file, "…" con Sposta in… ed Elimina); Parlanti con la rinomina; trascrizione a turni che scorre; player fisso in basso sopra la status bar. Durante la Registrazione al posto del player timer, livelli, Pausa e Stop, e il testo dal vivo.
- Durante un'Attività si aprono, leggono, cercano e copiano altri Bini, e si correggono o si rinominano Parlanti, ma non nel Bino su cui lavora l'Attività. Il player è disabilitato durante una Registrazione (l'audio riascoltato finirebbe nel file).

## Testo

- Correzione per Frase: il testo di una Frase si modifica sul posto e si salva nel Bino (e nell'indice) uscendo dalla Frase. Niente unione o divisione di Frasi; una Frase svuotata resta con i suoi tempi. Sparisce la textarea libera con lo stato "modificato a mano": Copia testo usa sempre le Frasi. Ritrascrivere sostituisce anche le correzioni, dopo la conferma. Il testo non si modifica durante l'Attività che lo produce.
- Ogni Frase ha un pulsante con il tempo (`12:34`), visibile al passaggio del mouse, sempre per la Frase in riproduzione e all'inizio di ogni turno, raggiungibile da tastiera: porta il player all'`inizio_ms` della Frase; in pausa resta in pausa, in riproduzione continua da lì. Il clic sul testo serve solo a modificarlo. I tempi non compaiono nel testo copiato o esportato.

## Player

- Riproduce `mix.ogg` del Bino (anche con gli Ingressi separati). Play/Pausa, posizione, durata, barra di avanzamento, indietro e avanti di 10 s, velocità 1×/1,25×/1,5×/2×, Spazio per Play/Pausa quando non si scrive. Niente volume.
- Solo sui Bini: un file audio o video aperto e non ancora trascritto non ha player.
- Evidenziazione: la Frase che contiene la posizione; nel silenzio tra due Frasi resta la precedente; con gli Ingressi separati le Frasi sovrapposte si evidenziano tutte e lo scorrimento segue quella iniziata prima.
- Seguire l'audio: durante la riproduzione la Frase evidenziata resta visibile. Uno scorrimento a mano sospende il seguire e mostra "Segui l'audio", che torna alla Frase e lo riprende; lo riprendono anche il pulsante del tempo e lo spostamento della barra del player. Mentre si modifica una Frase lo scorrimento automatico è sospeso.
- I tempi sono già quelli dell'audio salvato, pause escluse; `inizio_ms` comprende 300 ms di prefill, quindi il salto parte poco prima del parlato.

## Da verificare prima della spec

Verificati i primi due punti: esito in `spec.md`, "Further Notes". Il terzo si misura con il ticket della correzione.

- WebView2 riproduce Ogg/Opus in `<audio>` e un protocollo personalizzato di Tauri serve `mix.ogg` dentro lo zip (voce `Stored`) con le richieste `Range` per lo spostamento.
- `rusqlite` con `bundled` compila su MSVC senza dipendenze nuove e include FTS5 (tokenizer `unicode61 remove_diacritics 2`, query per prefisso).
- Tempo di riscrittura di un Bino di un'ora a ogni correzione (`bino::rewrite` copia il mix con `raw_copy_file`).
