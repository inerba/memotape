# Gli Assistenti modificano la Libreria

Status: ready-for-agent

Data: 11 ottobre 2026. Decisioni in [ADR-0031](../../docs/adr/0031-gli-assistenti-modificano-la-libreria.md), che sostituisce in parte ADR-0012. Termini in `CONTEXT.md` (Assistente, Silenzio, Taglio, Correzione manuale, Turno).

## Problem Statement

Un Assistente (Claude, Codex) collegato a Memotape oggi può solo cercare e leggere i Tape. Quando l'utente gli chiede di sistemare una Trascrizione (correggere le parole sentite male, ricostruire le frasi, dare un titolo sensato, togliere i Silenzi all'inizio, alla fine o quelli lunghi in mezzo) oppure di rinominare o eliminare un Tape, l'Assistente non può farlo: l'utente deve aprire Memotape e rifare tutto a mano, copiando il testo proposto Turno per Turno.

## Solution

Con un secondo permesso, «Consenti agli Assistenti di modificare la Libreria», l'Assistente può modificare i Tape attraverso il server MCP di Memotape, anche con l'app chiusa. Fa le stesse cose che l'utente fa nell'app: corregge il testo di un Turno, unisce Turni, dà nomi ai Parlanti, cambia titolo e data, trova i Silenzi, applica Tagli e manda Tape nel Cestino. Le sue modifiche sono Correzioni manuali. Un prompt MCP «Sistema il Tape» guida l'Assistente in un aggiustamento leggero: correzione leggera, Taglio dei Silenzi e titolo. Ciò che si perde (un Taglio, un Tape eliminato) finisce nel Cestino di Windows. L'app, quando torna in primo piano, si accorge dei cambiamenti e mostra il Tape e la Libreria aggiornati.

## User Stories

1. Come utente, voglio un interruttore separato per consentire agli Assistenti di modificare la Libreria, così posso farli leggere senza farli scrivere.
2. Come utente, voglio che il permesso a modificare sia spento di default, così niente cambia i miei Tape senza che lo abbia deciso.
3. Come utente, voglio che il permesso a modificare si possa accendere solo con la lettura accesa, così le due cose restano coerenti.
4. Come utente, voglio che spegnendo la lettura si spenga anche la modifica, così non resta un permesso che non ha senso.
5. Come utente, voglio che l'interruttore abbia testi in tutte e sei le lingue dell'interfaccia, come gli altri.
6. Come Assistente, voglio un errore chiaro quando il permesso a modificare è spento, così posso dire all'utente dove accenderlo.
7. Come Assistente, voglio leggere i Turni di un Tape in forma strutturata (Ingresso, id delle Frasi, Parlante, inizio e fine, testo), così posso correggerli indicando esattamente cosa cambio.
8. Come Assistente, voglio leggere i Turni a pagine, così un Tape lungo non supera il limite di una risposta.
9. Come Assistente, voglio correggere il testo di un Turno intero, così posso ricostruire le frasi oltre i confini delle Frasi.
10. Come Assistente, voglio mandare il testo attuale atteso del Turno insieme alla correzione, così una correzione su un testo cambiato nel frattempo viene rifiutata invece di sovrascriverlo.
11. Come Assistente, voglio che una correzione identica al testo attuale non riscriva il Tape, così non segno come corretto un Turno che non ho cambiato.
12. Come utente, voglio che le correzioni dell'Assistente compaiano come «Corretto a mano», così so che il testo non è più quello della Trascrizione.
13. Come utente, voglio che una nuova Diarizzazione conservi le correzioni dell'Assistente come conserva le mie.
14. Come Assistente, voglio unire un Turno a quello adiacente, così correggo un'attribuzione sbagliata della Diarizzazione.
15. Come Assistente, voglio dare un nome a un Parlante, così il testo dice chi parla.
16. Come Assistente, voglio cambiare la data e l'ora di un Tape, così la correggo se l'utente me lo chiede.
17. Come Assistente, voglio rinominare un Tape, cioè cambiarne il titolo, così posso dargli un titolo che dica l'argomento.
18. Come Assistente, voglio un errore distinto quando il titolo è già usato da un altro Tape della stessa cartella, così ne scelgo un altro.
19. Come Assistente, voglio un errore distinto quando il titolo contiene caratteri non validi per un nome di file, così lo correggo.
20. Come Assistente, voglio ricevere il percorso nuovo del Tape dopo una rinomina, così continuo a lavorarci senza cercarlo.
21. Come Assistente, voglio mandare un Tape nel Cestino, così lo elimino quando l'utente me lo chiede.
22. Come utente, voglio che un Tape eliminato dall'Assistente stia nel Cestino di Windows, così lo recupero se era un errore.
23. Come Assistente, voglio l'elenco dei Silenzi di un Tape (inizio, fine e dove: inizio, in mezzo, fine) con il Taglio che l'app applicherebbe, così so cosa togliere senza indovinare dai tempi.
24. Come utente, voglio che i Silenzi trovati dall'Assistente siano gli stessi che vedo in modalità Taglio, così l'Assistente fa quello che farei io.
25. Come utente, voglio che un Tape senza Frasi non abbia Silenzi, così l'Assistente non taglia tutto l'audio di un Tape non trascritto.
26. Come Assistente, voglio applicare uno o più Tagli a un Tape in un'unica operazione, così il Tape si riscrive una volta sola.
27. Come utente, voglio che il Taglio fatto dall'Assistente mandi l'originale nel Cestino, così posso tornare indietro.
28. Come utente, voglio che su un disco senza Cestino il Taglio dell'Assistente venga rifiutato, così niente si perde senza la mia conferma.
29. Come utente, voglio che il mio client chieda conferma prima di un Taglio o di un'eliminazione, così le azioni distruttive passano da me.
30. Come utente, voglio che l'Assistente non possa scrivere un Tape su cui l'app sta lavorando (Registrazione, Trascrizione, Riconosci i parlanti), così non rovina un'Attività in corso.
31. Come Assistente, voglio un errore distinto quando il Tape è occupato da un'Attività, così riprovo più tardi o lo dico all'utente.
32. Come utente, voglio che un'Attività su un Tape non impedisca all'Assistente di sistemare gli altri Tape.
33. Come utente, voglio che una mia correzione nell'app e una dell'Assistente sullo stesso Tape non si cancellino a vicenda, così restano tutte e due.
34. Come utente, voglio che l'Assistente possa modificare i Tape anche con Memotape chiuso.
35. Come utente, voglio che, tornando su Memotape, il Tape aperto mostri le modifiche dell'Assistente, così non lavoro su una versione vecchia.
36. Come utente, voglio che una bozza aperta nell'editor del Turno non venga salvata sopra una correzione dell'Assistente, e che resti visibile con l'errore, così non perdo né la mia né la sua.
37. Come utente, voglio che, tornando su Memotape, la Libreria e la ricerca mostrino titoli, Tagli ed eliminazioni fatti dall'Assistente.
38. Come Assistente, voglio un errore che dica di aprire Memotape quando un Tape che l'indice elenca non c'è più, così non resto bloccato su un indice vecchio.
39. Come utente, voglio che un Tape fuori dalla Libreria, o indicato con un percorso assoluto o con `..`, non si possa modificare, come già non si può leggere.
40. Come Assistente, voglio un prompt «Sistema il Tape» che mi dica cosa fare e in che ordine, così l'aggiustamento leggero è uguale ogni volta.
41. Come utente, voglio che la correzione leggera tocchi solo errori di riconoscimento, nomi propri, punteggiatura, maiuscole, ripetizioni involontarie e intercalari, così stile e senso restano i miei.
42. Come utente, voglio che i Turni senza errori evidenti non vengano toccati, così conservano il testo originale e i tempi delle parole.
43. Come utente, voglio che il titolo cambi solo se è generico (il predefinito della Registrazione o il nome del file importato), così un titolo che ho scelto io resta.
44. Come utente, voglio un titolo di al massimo 100 caratteri, nella Lingua del parlato, sull'argomento e senza data, perché la data il Tape ce l'ha già.
45. Come utente, voglio che ogni operazione dell'Assistente finisca nel log di Memotape, così so cosa ha fatto.

## Implementation Decisions

- **Lock su file tra i processi.** Il lock delle riscritture del Tape, oggi un mutex del solo processo dell'app, diventa un lock esclusivo su un file nella cartella `.memotape` della Libreria (`LockFileEx` o l'equivalente della libreria standard), preso da app e server MCP intorno a ogni riscrittura. Resta un solo lock per tutte le riscritture, come oggi.
- **Lock dell'Attività.** L'app tiene per tutta un'Attività un lock per Tape (un file di lock per Tape in `.memotape`, con nome derivato dal percorso del Tape) che il server prova a prendere senza attendere: se non ci riesce rifiuta con `activityInProgress`. Il controllo dell'Attività dentro l'app resta com'è.
- **Il server non scrive mai l'indice.** Le operazioni della Libreria che oggi fanno insieme operazione sul file e `sync` (rinomina, Cestino) si dividono: il server usa solo l'operazione sul file, l'app continua a fare l'una e l'altra. Il server continua ad aprire l'indice in sola lettura.
- **Consenso.** Le Impostazioni hanno un secondo booleano (es. `assistenti_modifica`), falso di default e falso quando manca nel file. Il server lo rilegge a ogni chiamata, come oggi la lettura; con la lettura spenta la modifica vale spenta. Nella sezione Assistenti delle Impostazioni c'è il secondo interruttore, disabilitato con la lettura spenta, con testi nelle sei lingue.
- **Strumenti MCP nuovi.** I nomi seguono quelli esistenti (inglese):
  - `read_turns`: Turni del Tape a pagine, ciascuno con Ingresso, id delle Frasi, Parlante (nome o etichetta), inizio e fine, testo come lo mostra l'editor del Turno (la stessa proiezione `frasi_visibili`/correzioni del Turno);
  - `edit_turn`: Ingresso, id delle Frasi del Turno, testo atteso, testo nuovo; delega a `edit_turno` (ADR-0020);
  - `merge_turn`: delega a `unisci_turno`;
  - `rename_speaker`: delega a `rename_parlante`;
  - `set_date`: delega a `set_creato`;
  - `rename_tape`: rinomina del file con la validazione del nome dell'app; restituisce il percorso nuovo relativo alla Libreria;
  - `trash_tape`: Cestino di Windows, con `destructive_hint`;
  - `find_silences`: i Silenzi con il Taglio proposto per ciascuno;
  - `cut_tape`: un elenco di tratti in ms, applicati con `tape::taglia` con il Cestino obbligatorio, con `destructive_hint`; su `noRecycleBin` rifiuta.
  - Gli strumenti che scrivono hanno `read_only_hint = false`. Errori in testo leggibile per l'Assistente, come quelli esistenti, distinguendo almeno: permesso spento, Tape occupato da un'Attività, testo atteso obsoleto, titolo già usato, nome non valido, Tape non trovato (aprire Memotape), volume senza Cestino.
- **Silenzi in Rust.** La regola della modalità Taglio si porta nel backend, accanto al documento del Tape: un Silenzio è il tratto senza Frasi di nessun Ingresso prima della prima Frase, dopo l'ultima fino alla fine del Tape, o fra due Turni, quando il silenzio stimato (sottratti hangover e prefill come nel separatore della pausa) è almeno 5 s. Il Taglio proposto toglie tutto all'inizio e alla fine e lascia 1 s per lato in mezzo (a metà se il Silenzio è corto). La regola del frontend resta dov'è; i due insiemi di casi di test sono gli stessi.
- **Prompt MCP «Sistema il Tape».** Un prompt del server, con argomento il Tape, che descrive l'ordine: (1) correzione leggera Turno per Turno con `read_turns`/`edit_turn`, solo dove ci sono errori evidenti; (2) `find_silences` e un solo `cut_tape` con i Tagli proposti; (3) titolo con `rename_tape` solo se quello attuale è generico, al massimo 100 caratteri, nella Lingua del parlato, sull'argomento, senza data; se il titolo è preso se ne sceglie un altro. Il titolo viene per ultimo perché cambia il percorso.
- **App al ritorno in primo piano.** Al focus della finestra l'app riallinea l'indice (come all'avvio) e ricarica il Tape aperto se la sua data di modifica è cambiata. Nessun watcher del file system. Una bozza dell'editor del Turno non si sovrascrive: il salvataggio fallisce sul testo atteso e la bozza resta con l'errore.
- **Log.** Ogni chiamata a uno strumento che scrive va nel log del server, come le letture.
- **Documenti da aggiornare.** `PRODUCT.md` (requisito), `docs/sviluppo/impostazioni-e-servizi.md` (Assistenti), e il rimando ad ADR-0031 dove si cita ADR-0012.

## Testing Decisions

- Un buon test chiama uno strumento del server come lo chiamerebbe un Assistente e controlla l'effetto visibile: il documento del Tape riletto, il file al suo posto o sparito, l'errore restituito. Non controlla come è fatto il lock né le funzioni interne.
- **Unica seam: i metodi degli strumenti del `Server` MCP**, con la Libreria di prova indicizzata che i test di `mcp.rs` già costruiscono (`server(name, assistenti)`, `tape_with`). Precedenti: `senza_il_consenso_non_si_legge_nulla`, `si_cerca_e_si_legge_intorno_a_una_frase_trovata`, `editor_turno_ricerca_e_assistenti_leggono_solo_la_correzione`.
- Casi da coprire da quella seam:
  - permesso a modificare spento (lettura accesa) e lettura spenta: ogni strumento che scrive rifiuta e il Tape resta identico byte per byte;
  - `read_turns` → `edit_turn` corregge, e una seconda correzione con il testo atteso vecchio viene rifiutata senza scrivere; il Tape risulta corretto a mano;
  - `merge_turn`, `rename_speaker`, `set_date` scrivono quello che dicono;
  - `rename_tape` restituisce il percorso nuovo; titolo già usato e nome non valido danno i loro errori;
  - `trash_tape` toglie il file dalla Libreria (il Cestino come in `eliminare_un_tape_lo_manda_nel_cestino`);
  - `find_silences` sugli stessi casi di `taglio.test.ts` (inizio, fine, in mezzo sotto e sopra i 5 s, Tape senza Frasi, Ingressi che si accavallano);
  - `cut_tape` con i Tagli proposti accorcia il Tape e sposta le Frasi;
  - con il lock dell'Attività preso dal test, come lo prenderebbe l'app, gli strumenti che scrivono rifiutano con l'errore di Attività in corso;
  - dopo una scrittura l'indice è identico a prima (il server non lo tocca);
  - percorso assoluto, con `..` o fuori dalla Libreria: rifiutato anche in scrittura.
- **Impostazioni**: un caso nei test esistenti sui valori predefiniti (`settings.rs`, `settings.test.ts`) per il nuovo booleano falso di default e quando manca.
- **Senza test automatici** (regola: niente test sui componenti): ricarica del Tape aperto e riallineamento dell'indice al ritorno in primo piano, interruttore disabilitato con la lettura spenta. Si provano a mano con `bun run app:prova` e CDP, più una prova con Claude Code collegato all'exe di debug che esegue «Sistema il Tape» su un Tape di prova.

## Out of Scope

- Spostare Tape tra Raccolte, creare, rinominare o eliminare Raccolte.
- Avviare Attività dall'Assistente (Registrazione, Trascrizione, Riconosci i parlanti).
- Copie di sicurezza del Tape (`_orig` o simili) prima delle modifiche: basta il Cestino per Tagli ed eliminazioni.
- Annullare dall'app una modifica dell'Assistente, o distinguere nel Tape le correzioni dell'Assistente da quelle dell'utente.
- Un watcher del file system o un canale IPC tra server e app aperta.
- Taglio dei Silenzi in base al volume dell'audio invece che alle Frasi.
- Una conferma aggiuntiva dentro Memotape per le azioni dell'Assistente.

## Further Notes

- Sotto Claude Desktop (MSIX) le scritture in `%LOCALAPPDATA%` finiscono in un livello privato: per questo il server non scrive l'indice e il lock sta nella Libreria (Documenti), che non è virtualizzata. Da verificare a mano che il lock su file funzioni tra il processo dell'app e quello lanciato da Claude Desktop.
- Il Taglio (ADR-0030) è già sul branch: lo strumento `cut_tape` lo riusa senza cambiarlo, tranne il Cestino obbligatorio.
- Il lock dell'Attività deve essere rilasciato anche se l'Attività finisce con errore o Annulla, e se l'app viene chiusa (un lock di sistema sul file si rilascia con il processo).
