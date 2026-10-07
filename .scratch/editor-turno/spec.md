# Editor continuo del Turno e cursore visibile

Status: resolved

Data: 6 ottobre 2026. Comportamento approvato e implementazione completata. Esiti dei sei controlli e delle prove Windows in [verifica.md](verifica.md).

## Problem Statement

Quando clicca in mezzo al testo della Trascrizione, l'utente non vede ancora il cursore lampeggiante e non sa dove verranno inseriti i caratteri. La precedente modifica dei colori non ha risolto il difetto nell'app Windows: una prova riuscita dei componenti nel browser non basta ad attestare la risoluzione.

Inoltre, le Frasi di uno stesso Turno sono campi separati, ciascuno nel proprio paragrafo con uno spazio grafico tra i paragrafi. Backspace, Canc e la selezione non consentono di correggere liberamente il testo oltre quei confini. L'utente vuole poter unire e separare il testo all'interno del singolo Turno come in un normale documento.

Questa richiesta è distinta da Unisci, il comando già definito che corregge l'attribuzione di un Turno al Parlante sopra o sotto.

## Solution

Ogni Turno modificabile offre un unico testo semplice editabile sul posto. Cliccare tra due caratteri posiziona lì un cursore lampeggiante chiaramente visibile; selezione, frecce, Backspace e Canc attraversano le Frasi dello stesso Turno. Le separazioni tra Frasi diventano separatori testuali modificabili, anziché ostacoli imposti da campi distinti.

Invio inserisce un a capo. Uscire dal campo salva l'intera correzione; Ctrl+Z annulla le singole modifiche durante la scrittura ed Esc ripristina il testo presente quando si è entrati nel campo, senza salvarne la bozza. È possibile cancellare tutto il testo: il Turno resta raggiungibile con l'indicazione «Blocco senza testo» e l'audio viene conservato.

Una Correzione manuale salvata conserva testo e a capo nella riapertura, nelle copie e nelle uscite testuali. La dicitura «Corretto a mano» e gli avvisi prima di una nuova analisi seguono le regole già concordate. Durante l'ascolto, per il testo corretto si evidenzia l'intero Turno senza fingere un allineamento delle singole parole all'audio. Il testo non modificato conserva i riferimenti disponibili.

## User Stories

1. Come utente che corregge una Trascrizione, voglio cliccare tra due caratteri e vedere il cursore lampeggiante in quel punto, così so dove scriverò.
2. Come utente che usa il tema chiaro, voglio distinguere il cursore e la selezione dal fondo del testo, così posso modificare senza tentativi.
3. Come utente che usa il tema scuro, voglio la stessa leggibilità del cursore e della selezione, così posso correggere con il tema che preferisco.
4. Come utente, voglio un unico campo testuale per ciascun Turno modificabile, così non devo entrare e uscire da una Frase alla volta.
5. Come utente, voglio selezionare con il mouse un tratto che comprende più Frasi dello stesso Turno, così posso sostituirlo insieme.
6. Come utente, voglio estendere la selezione con la tastiera oltre i confini delle Frasi, così posso correggere senza usare il mouse.
7. Come utente, voglio spostare il cursore con le frecce tra le Frasi dello stesso Turno, così posso raggiungere qualsiasi punto del testo.
8. Come utente, voglio usare Backspace all'inizio della seconda Frase per togliere il separatore precedente, così posso unirla alla prima.
9. Come utente, voglio usare Canc alla fine della prima Frase per togliere il separatore successivo, così posso ottenere la stessa unione nell'altra direzione.
10. Come utente, voglio scegliere quali spazi e a capo conservare o rimuovere, così la struttura del testo dipende dalla mia correzione.
11. Come utente, voglio premere Invio per andare a capo senza uscire dall'editor, così posso separare nuovamente il testo.
12. Come utente, voglio inserire testo semplice anche quando lo incollo, così non introduco formattazione indesiderata.
13. Come utente, voglio che uscire dal campo salvi la correzione completa del Turno, così non devo salvare ogni Frase separatamente.
14. Come utente, voglio annullare con Ctrl+Z le singole modifiche mentre scrivo, così posso correggere un errore di digitazione.
15. Come utente, voglio premere Esc per tornare al testo presente all'ingresso nell'editor, così posso scartare tutta la bozza corrente.
16. Come utente, voglio che entrare e uscire senza cambiare il testo non registri una nuova Correzione manuale, così l'indicatore resta significativo.
17. Come utente, voglio cancellare tutto il testo di un Turno senza cancellarne l'audio, così posso eliminare una trascrizione inventata o inutile.
18. Come utente, voglio ritrovare il Turno svuotato con «Blocco senza testo», così posso riascoltarlo e scrivere nuovamente.
19. Come utente, voglio che un errore di salvataggio lasci disponibile il testo che ho scritto e mostri l'errore, così posso recuperare la correzione.
20. Come utente, voglio riaprire il Tape e trovare testo, separazioni e a capo come li ho salvati, così non devo ripetere il lavoro.
21. Come utente, voglio che una copia del Tape conservi le correzioni, così posso trasferirlo senza perderle.
22. Come utente, voglio che Copia turno, Copia testo ed Esporta Markdown riportino il testo corretto e gli a capo, così il risultato esportato corrisponde a quello visibile.
23. Come utente, voglio trovare con la ricerca il nuovo testo salvato, così la Libreria resta coerente con la correzione.
24. Come utente, voglio vedere «Corretto a mano» dopo un salvataggio riuscito, così riconosco i Tape sui quali ho lavorato.
25. Come utente, voglio essere avvisato prima di rifare la Diarizzazione e conservare il testo corretto, così l'analisi non annulla le mie modifiche.
26. Come utente, voglio un avviso esplicito prima di una nuova Trascrizione che sostituirà le correzioni, così posso decidere se perderle.
27. Come utente, voglio che annullare o interrompere con un errore una nuova analisi conservi il Tape precedente, così non perdo il lavoro manuale.
28. Come utente che riascolta un Turno corretto, voglio vedere evidenziato l'intero Turno, così seguo l'audio senza un'associazione ingannevole tra parole e tempi.
29. Come utente, voglio che la riproduzione e l'avanzamento del player non spostino il cursore o la selezione mentre scrivo, così posso correggere durante l'ascolto.
30. Come utente, voglio che correggere il testo non cambi il Parlante né gli altri Turni, così l'intervento resta locale.
31. Come utente che registra da Entrambi, voglio mantenere distinti Microfono e Audio di sistema durante la correzione, così non mescolo il testo di Ingressi diversi.
32. Come utente, voglio continuare a leggere i vecchi Tape, così questa funzione non mi obbliga a convertirli solo per aprirli.

## Implementation Decisions

- Il Turno rimane il raggruppamento definito dal glossario: Frasi consecutive dello stesso Ingresso e Parlante, oppure raggruppamento previsto per il testo senza Parlanti. L'unità di modifica diventa il testo del Turno; Frase non diventa sinonimo di paragrafo scritto dall'utente.
- Il campo contiene tutto il testo modificabile del Turno. Sono eliminati i confini di editing tra Frasi; i separatori iniziali possono conservare l'impaginazione precedente, ma devono essere caratteri modificabili. Non basta ridurre il margine grafico tra campi indipendenti.
- La selezione e la cancellazione attraverso Frasi sono richieste entro il Turno. Nessun attraversamento automatico del confine con un altro Turno, Parlante o Ingresso è introdotto.
- Il contenuto è testo semplice. Gli a capo e gli spazi scelti dall'utente devono restare nel testo salvato; la normalizzazione esistente che appiattisce tutti gli a capo in spazi non soddisfa questa richiesta.
- Il focus apre una sessione con un testo iniziale. Invio inserisce un a capo; uscire dal campo richiede un unico salvataggio dell'intera correzione. Esc ripristina il testo iniziale e chiude la sessione senza salvare la bozza. Ctrl+Z opera sulle modifiche della sessione attiva, senza coinvolgere altri Turni.
- Lo stato del player non può rimontare il campo, sostituire la bozza o spostare il cursore. Lo scorrimento che segue l'audio resta sospeso durante la modifica, secondo il comportamento esistente.
- Il salvataggio del Turno è atomico, con la stessa protezione dalle scritture concorrenti e dalle Attività già usata dalle modifiche del Tape. Riferimenti non più validi devono produrre un errore senza una scrittura parziale. Non si simulano salvataggi riusciti con una sequenza di modifiche indipendenti alle Frasi.
- La rappresentazione persistente deve conservare testo editato, separatori e relazione con gli intervalli audio originari. La scelta del dettaglio dello schema appartiene all'implementazione, vincolata ai risultati osservabili di questa spec. Non è valido distribuire arbitrariamente le nuove parole tra le vecchie Frasi e dichiarare attendibili i tempi risultanti.
- Audio del mix e degli Ingressi, Forma d'onda, durata e riferimenti temporali originari restano conservati. Una correzione testuale non taglia i silenzi, non concatena audio e non elimina i riferimenti dei tratti svuotati. Gli identificativi esistenti restano risolvibili o hanno una corrispondenza esplicita; nessun riferimento viene silenziosamente riassegnato a contenuto diverso.
- Il testo svuotato è un risultato valido. «Blocco senza testo» è un'indicazione dell'interfaccia, non testo della Trascrizione: non deve essere salvata come parlato, indicizzata o esportata. Il campo resta accessibile da mouse e tastiera anche dopo riapertura.
- Le sole modifiche al testo non proteggono automaticamente l'identità del Parlante. Restano le distinzioni tra testo corretto, attribuzione corretta e nome personalizzato definite dalle Correzioni manuali. Una nuova Diarizzazione conserva esattamente il testo editato e non ne ricostruisce separatori o ripartizioni sulla base di tempi ASR ormai invalidi; può aggiornare le attribuzioni non protette senza perdere o duplicare testo. Se cambia il raggruppamento dei Turni, la rappresentazione della correzione deve rimanere coerente e non usare i numeri dei Parlanti come prova di identità.
- Per il testo corretto si usa l'evidenziazione del Turno durante il relativo audio, senza presentare allineamenti delle parole inventati. I salti temporali supportati e i controlli del Turno continuano a usare i riferimenti audio conservati. Il testo non modificato mantiene l'evidenziazione disponibile secondo i tempi esistenti.
- Copia turno, Copia testo, esportazione Markdown, lettura per Assistenti e ricerca devono usare la stessa rappresentazione del testo salvato. Gli a capo intenzionali devono sopravvivere alle uscite testuali senza introdurre duplicati delle Frasi originarie.
- Il formato del Tape deve restare compatibile in lettura con i documenti precedenti. Gli eventuali nuovi metadati sono espliciti; l'assenza di tempi del testo non basta a dedurre una Correzione manuale. Apertura e chiusura senza modifiche non migrano o riscrivono il Tape.
- Le modifiche riuscite fanno comparire «Corretto a mano» e seguono gli avvisi esistenti. La nuova Trascrizione rimuove la vecchia correzione solo dopo una sostituzione riuscita; Annulla o errore conservano testo e metadati precedenti.
- Il comando Unisci continua a correggere l'attribuzione del solo Turno scelto. La sua funzione resta distinta dall'unione testuale ottenuta con Backspace o Canc; non può ripristinare separatori rimossi né perdere una correzione già salvata.
- La modifica si abilita solo per testo salvato e modificabile. Parziali e Tape sul quale lavora un'Attività mantengono le protezioni esistenti. Le nuove indicazioni dell'interfaccia sono coerenti nelle sei lingue.
- La diagnosi del cursore parte dalla finestra Windows reale e distingue focus, posizione della selezione e disegno del cursore. La causa non è stata accertata: nessuna scelta CSS o di componente va presentata come soluzione già verificata.

## Testing Decisions

### Punti di verifica proposti

Si preferiscono due punti di verifica pubblici, perché la persistenza non prova il comportamento dell'editor e il browser non prova il disegno del cursore nella WebView2:

- **Modifica e riapertura del Tape:** esercitare la correzione attraverso il confine pubblico di modifica del documento, riaprire il file e leggere il risultato attraverso le normali uscite. Riutilizzare il percorso reale di riscrittura atomica e l'indice della Libreria. Il precedente test di correzione testuale riaperta e il test di Unione di Turni con ricerca sono i riferimenti; non simulare l'intero documento o lo zip.
- **Interazione con l'editor reale del Turno:** verificare eventi di mouse e tastiera sul componente usato dall'app, collegato al salvataggio. Una fixture nel browser può facilitare la riproduzione, ma il cursore e la selezione devono essere verificati anche nella finestra Windows con un Tape reale e con il player collegato. I test statici di rendering non coprono focus, cursore, undo o a capo.

Non si aggiungono punti di test per ogni funzione interna. I test devono dimostrare comportamenti osservabili: testo riaperto, integrità dell'audio, risultato delle uscite, salvataggio riuscito o fallito, posizione di inserimento e selezione. Non devono dipendere dal numero di nodi DOM, dai nomi delle classi o dalla scelta tra un elemento editabile e un controllo nativo.

### Criteri di accettazione

1. **Cursore e selezione:** nella finestra Windows, cliccare all'inizio, nel mezzo e alla fine di una Frase posiziona un cursore lampeggiante visibile; digitare inserisce in quel punto. Ripetere nei due temi, con player fermo e in riproduzione. Verificare la selezione con mouse e tastiera, anche su testo evidenziato dall'ascolto.
2. **Attraversamento dei confini:** in un Turno con almeno tre Frasi, frecce e selezione attraversano i separatori; Backspace all'inizio della seconda e Canc alla fine della prima li rimuovono senza fermarsi a un campo distinto.
3. **A capo e spazi:** Invio inserisce a capo senza salvare o spostare il focus. Inserire e togliere più a capo, modificare spazi e incollare testo multilinea; dopo salvataggio e riapertura il contenuto corrisponde a quello scritto, senza formattazione ricca.
4. **Salvataggio ed Esc:** uscire salva una sola correzione completa. Esc ripristina il testo dell'ingresso nella sessione, non avvia il salvataggio e non introduce nuovi metadati manuali. Entrare e uscire senza modifiche lascia il documento invariato.
5. **Ctrl+Z:** durante la scrittura, annullare inserimenti, cancellazioni oltre un confine e nuovi a capo ripristina gli stati precedenti nel Turno attivo. Gli aggiornamenti del player non interrompono questa sequenza.
6. **Turno vuoto:** cancellare tutto, salvare e riaprire conserva audio, durata e possibilità di scrivere nel Turno; «Blocco senza testo» è visibile e accessibile ma non appare in Copia testo, Markdown, lettura per Assistenti o ricerca del parlato.
7. **Errore e concorrenza:** provocare un salvataggio fallito o una richiesta con riferimenti obsoleti. Il Tape precedente resta completo, la bozza resta recuperabile, l'errore è visibile e l'interfaccia non dichiara successo. Nessuna modifica parziale del Turno finisce nell'indice.
8. **Località e integrità:** modificare un Turno in un Tape con altri Turni dello stesso Parlante e con Ingressi separati. Gli altri testi e le attribuzioni restano invariati; confrontare contenuto audio e Forma d'onda prima e dopo, oltre a durata e riferimenti temporali.
9. **Riapertura, copie e uscite:** il testo salvato, compresi a capo e separatori cancellati, si conserva nella riapertura e nelle copie; Copia turno, Copia testo, Markdown, lettura per Assistenti e ricerca non riemettono il testo originale né duplicano quello corretto. La ricerca apre il risultato nel Turno pertinente.
10. **Correzioni manuali:** un salvataggio realmente diverso introduce il dato e «Corretto a mano»; annullamento della bozza o salvataggio fallito non lo introducono. Titolo e data restano estranei alla correzione del testo.
11. **Ascolto:** il Turno con testo corretto viene evidenziato durante il suo audio senza precisione parola per parola; il player continua a funzionare e i riferimenti delle altre Frasi restano validi. Durante la scrittura non cambia selezione e non forza lo scorrimento.
12. **Nuova Diarizzazione:** verificare testo unito, testo multilinea e Turno svuotato. L'avviso compare una volta; testo e correzioni restano. Includere un caso con nuova attribuzione o nuovi confini dei Turni, oltre a un'attribuzione protetta, per provare che la correzione testuale non congela automaticamente il Parlante e non viene persa, duplicata o falsamente riallineata.
13. **Nuova Trascrizione:** la conferma spiega la perdita delle correzioni; Annulla prima dell'avvio, Annulla durante l'Attività ed errore conservano il Tape precedente. Una sostituzione riuscita elimina soltanto la vecchia correzione, secondo il comportamento già approvato.
14. **Compatibilità e Unisci:** aprire un Tape precedente senza riscriverlo. Correggere il testo e poi usare Unisci sopra o sotto conserva la correzione e cambia solo l'attribuzione richiesta. Nessuna modifica dei Parziali o del Tape durante un'Attività che lo usa.
15. **Lingue e chiusura:** le nuove indicazioni esistono nelle sei lingue. Chiudere l'implementazione solo con i sei controlli del repository verdi e l'esito documentato delle prove Windows dei criteri 1–5 e 11. Un controllo del colore calcolato o una screenshot nel browser da soli non chiudono il difetto del cursore.

## Out of Scope

- Unire testo tra Turni diversi, Parlanti diversi o Ingressi diversi mediante i tasti di editing.
- Cambiare il significato o ampliare le direzioni disponibili del comando Unisci.
- Tagliare audio, togliere silenzi dalla Registrazione o modificare la Forma d'onda attraverso l'editor del testo.
- Inventare allineamenti parola per parola o rilanciare automaticamente l'ASR per ricavarli.
- Editor con formattazione ricca, stili o strumenti di impaginazione.
- Cronologia persistente delle revisioni e annullamento globale dopo un salvataggio o una riapertura. Ctrl+Z richiesto qui riguarda la sessione di scrittura.
- Modifica manuale dei Parziali e del testo del Tape durante un'Attività che lo sta elaborando.
- Refactoring della pipeline audio o dei modelli di Trascrizione e Diarizzazione estraneo alla conservazione delle correzioni.
- Commit, rilascio o installer in questa fase di specifica.

## Further Notes

- Fonte delle decisioni: conversazione del 6 ottobre 2026, inclusi i due «sì come suggerisci» su Invio/salvataggio/Turno vuoto e su evidenziazione del Turno/Esc/Ctrl+Z.
- Terminologia: il «blocco dello stesso parlante» discusso con l'utente è il Turno del glossario. L'indicazione «Blocco senza testo» mantiene il testo dell'interfaccia concordato.
- Questa spec sostituisce per la nuova funzione il precedente comportamento «Invio salva» e la correzione separata di ogni Frase. La spec QoL precedente rimane lo storico dell'implementazione precedente, non viene riscritta per far apparire già risolto il difetto residuo.
- Riferimenti: [PRODUCT.md](../../PRODUCT.md), [CONTEXT.md](../../CONTEXT.md), [DESIGN.md](../../DESIGN.md), [ADR-0015](../../docs/adr/0015-entrambi-si-trascrive-sempre-per-ingresso.md), [ADR-0017](../../docs/adr/0017-correzioni-manuali-della-trascrizione.md), [spec QoL precedente](../trascrizione-qol/spec.md) e [verifica precedente](../trascrizione-qol/verifica.md).
- La rappresentazione precedente correggeva una Frase alla volta. Il nuovo contratto del Turno, lo schema persistente e il comando atomico sono descritti nell'ADR-0020.
- I risultati delle prove QoL precedenti restano separati. Le nuove prove della funzione, inclusa la risoluzione del cursore nella WebView2 Windows, sono registrate in verifica.md.

## Comments

6 ottobre 2026: pubblicata nel tracker Markdown locale con etichetta `ready-for-agent`. Piano di verifica inviato all'utente; in attesa del suo riscontro, senza riaprire le decisioni di comportamento già approvate.

6 ottobre 2026: implementata su richiesta $implement. Editor continuo, salvataggio atomico e uscite coerenti; review Standards e Spec completate, rilievi corretti. Verifica nella WebView2 Windows con due temi e player fermo/in ascolto, 117 test frontend e 272 Rust superati; 20 smoke espliciti ignorati. Sei controlli verdi. Nessun commit o rilascio. Vedi [verifica.md](verifica.md) e [ADR-0020](../../docs/adr/0020-testo-continuo-del-turno.md).
