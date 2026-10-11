---
status: accepted
---

# Gli Assistenti modificano la Libreria scrivendo i Tape dal server MCP, con un lock su file

Data: 11 ottobre 2026. Sostituisce la parte «sola lettura» di ADR-0012; il resto (stdio, `memotape.exe --mcp`, percorsi relativi alla Libreria, indice aperto in sola lettura) resta.

Su richiesta dell'utente un Assistente corregge il testo di un Turno, unisce Turni, dà nomi ai Parlanti, cambia titolo e data, applica Tagli (anche dei Silenzi) e manda Tape nel Cestino. Non sposta Tape tra Raccolte e non avvia Attività. Le sue modifiche al testo, ai Turni e ai nomi sono Correzioni manuali, con le stesse protezioni di ADR-0017.

## Decisione

Il server MCP scrive da solo i file della Libreria, anche ad app chiusa. Il lock di riscrittura del Tape, oggi un `Mutex` del processo dell'app, diventa un lock su file nella cartella `.memotape` della Libreria, preso da entrambi i processi. La Libreria sta in Documenti, che sotto Claude Desktop (MSIX) non è virtualizzata: il lock e le scritture arrivano ai file veri.

- **Attività**: l'app tiene un lock per Tape per tutta un'Attività; il server rifiuta di scrivere quel Tape con `activityInProgress`. Le Attività su altri Tape non bloccano l'Assistente. Al contrario, mentre il server scrive un Tape l'app non aspetta: rifiuta di avviarci un'Attività, o una modifica, con lo stesso `activityInProgress` e l'utente riprova; un'attesa terrebbe fermi anche Annulla e gli altri comandi dell'app per tutto un Taglio.
- **Indice**: il server non lo scrive mai, per le ragioni di ADR-0012 (livello privato MSIX, `sync` concorrenti). Rinomina e Cestino spostano solo il file; lo strumento restituisce il percorso nuovo. L'app riallinea l'indice quando la sua finestra torna in primo piano; finché non lo fa, una lettura di un file sparito dice di aprire Memotape.
- **Tape aperto nell'app**: tornando in primo piano l'app ricarica il Tape aperto se è cambiato su disco. Una bozza dell'editor del Turno non si perde: il controllo sul testo atteso di `edit_turno` (ADR-0020) rifiuta il salvataggio su un testo cambiato.
- **Consenso**: un secondo interruttore, «Consenti agli Assistenti di modificare la Libreria», spento di default e attivo solo con la lettura accesa.
- **Recupero**: un Taglio dell'Assistente manda sempre l'originale nel Cestino; su un volume senza Cestino rifiuta con `noRecycleBin`, senza la variante dell'app che sostituisce l'originale. Correzioni, unioni, nomi e data non fanno copie, come nell'app. Le conferme sono quelle del client: gli strumenti di Taglio e Cestino hanno `destructive_hint`.
- **Strumenti semplici**: l'app non ha un modello linguistico, quindi titolo e correzione li scrive l'Assistente. Il server offre strumenti semplici (lettura strutturata dei Turni con id e testo atteso, correzione del Turno, Silenzi, Taglio, ...) e un prompt MCP «Sistema il Tape» con la ricetta, in quest'ordine:
  1. correzione leggera: errori di riconoscimento, nomi propri coerenti, punteggiatura, maiuscole, ripetizioni involontarie e intercalari; stile, ordine e senso restano, e i Turni senza errori evidenti non si toccano (conservano testo originale e tempi delle parole);
  2. Taglio dei Silenzi;
  3. titolo, solo se quello attuale è generico (il predefinito della Registrazione o il nome del file importato): breve, al massimo 100 caratteri, nella Lingua del parlato, sull'argomento, senza data; se è già preso se ne sceglie un altro.
- **Silenzi**: stessa regola della modalità Taglio (`src/features/taglio/taglio.ts`), calcolata in Rust: almeno 5 s di silenzio stimato fra le Frasi di tutti gli Ingressi, tolto tutto all'inizio e alla fine, 1 s per lato in mezzo.

## Alternative scartate

- Passare le scritture all'app aperta con una named pipe: le regole restano in un processo, ma ad app chiusa l'Assistente non scrive nulla, e serve un canale IPC in più.
- Named pipe con ripiego sulla scrittura diretta ad app chiusa: le due strade insieme.
- Una copia `_orig` prima del primo intervento dell'Assistente: scartata dall'utente, basta il Cestino.
- Un unico strumento «riordina» lato app: titolo e correzione richiedono comunque l'Assistente.

## Conseguenze

- `tape::writing()` e i comandi della Libreria (`rename_tape`, `trash_tape`, che oggi chiamano `sync`) vanno divisi tra operazione sul file e riallineamento dell'indice, così il server usa solo la prima.
- Tra una scrittura dell'Assistente e il ritorno in primo piano dell'app, la ricerca può restituire titoli o percorsi vecchi.
- Un Taglio applicato dall'Assistente su un Tape che l'utente sta ascoltando cambia la linea del tempo sotto il player: la ricarica al ritorno in primo piano lo riallinea.
