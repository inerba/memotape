# Note del Tape

Status: ready-for-agent

Data: 11 ottobre 2026. Requisito in `PRODUCT.md` («Note — requisiti concordati l'11 ottobre 2026»), formato ed editor in [ADR-0032](../../docs/adr/0032-note-del-tape-in-note-json-con-codemirror.md), permessi degli Assistenti in [ADR-0031](../../docs/adr/0031-gli-assistenti-modificano-la-libreria.md). Termini in `CONTEXT.md` (Nota, Tempo nella Nota, Assistente, Taglio, Correzione manuale). Anteprime approvate, fuori da git nel checkout principale: `.impeccable/mocks/note-anteprime.html` (variante A, ibrida) e `.impeccable/mocks/note-multiple.html` (variante A, riga di nomi).

## Problem Statement

Dopo una riunione, una lezione o un'intervista l'utente vuole tenere accanto al testo le sue note: decisioni, cose da fare, domande, link. Oggi le scrive altrove (un editor, un'app di appunti) e perde il legame con il Tape e con i punti dell'audio a cui si riferiscono. Un Assistente collegato a Memotape sa leggere il Tape ma non ha dove lasciare un riepilogo: lo restituisce nella chat, da copiare a mano. Durante una Registrazione non c'è modo di appuntarsi qualcosa in Memotape.

## Solution

Ogni Tape ha una scheda «Note» accanto a Trascrizione e Parlanti, con zero o più Note in Markdown, ciascuna con un nome. Si scrivono in un editor ibrido come il Live Preview di Obsidian: impaginato fuori dalla riga del cursore, Markdown visibile sulla riga che si scrive. Un tempo tra quadre come `[12:40]` porta il player a quel punto. Le Note si scrivono anche durante la Registrazione e allo Stop entrano nel Tape. Si salvano con «Salva» o da sole, con un interruttore; l'app avvisa prima di perdere modifiche. Un Assistente legge, crea, riscrive e rinomina Note con gli strumenti MCP, e un prompt «Scrivi le Note» prepara un riepilogo. Le Note stanno nel Tape, si cercano dalla Libreria e si esportano con il Markdown.

## User Stories

1. Come utente, voglio una scheda «Note» accanto a Trascrizione e Parlanti nel Tape aperto, così le note stanno con il testo e l'audio.
2. Come utente, voglio vedere nella scheda quante Note ha il Tape, così so che ce ne sono senza aprirla.
3. Come utente, voglio scrivere subito in un Tape senza Note, senza premere un pulsante, così prendere un appunto costa un clic.
4. Come utente, voglio che il primo carattere crei la Nota «Note», così non devo darle un nome per cominciare.
5. Come utente, voglio più Note nello stesso Tape, per esempio «Riepilogo», «Mie note» e «Domande per Sara», così separo cose diverse.
6. Come utente, voglio vedere i nomi di tutte le Note in una riga sotto le schede e passare dall'una all'altra con un clic.
7. Come utente, voglio creare una Nota con «+ Nuova nota» e darle subito un nome, che parte da «Nota 2», «Nota 3»…
8. Come utente, voglio rinominare una Nota con un doppio clic sul nome, con F2 o dal «…», così il nome dice cosa contiene.
9. Come utente, voglio che due Note dello stesso Tape non abbiano lo stesso nome e che un nome non superi 60 caratteri.
10. Come utente, voglio eliminare una Nota dal «…» con una conferma che dice che non si potrà recuperare.
11. Come utente, voglio copiare una Nota dal «…», così la incollo in una mail o in un altro editor.
12. Come utente, voglio che una Nota nuova lasciata vuota sparisca da sola, così non resto con Note vuote create per sbaglio.
13. Come utente, voglio che le Note restino nell'ordine in cui le ho create.
14. Come utente, voglio scrivere in Markdown e leggere il risultato impaginato mentre scrivo: titoletti, grassetto, corsivo, link, elenchi e caselle da spuntare.
15. Come utente, voglio vedere i segni del Markdown sulla riga in cui scrivo, così posso correggerli a mano.
16. Come utente, voglio una barretta sospesa quando seleziono del testo, con grassetto, corsivo, link, elenco e da fare, così formatto senza ricordare la sintassi.
17. Come utente, voglio Ctrl+B, Ctrl+I e Ctrl+K per grassetto, corsivo e link mentre scrivo una Nota, e che fuori dal campo restino barra laterale e ricerca.
18. Come utente, voglio spuntare una casella con un clic, così segno le cose fatte.
19. Come utente, voglio che un link si apra nel browser di Windows, così leggo la pagina che ho annotato.
20. Come utente, voglio che un tempo scritto tra quadre (`[12:40]`, `[1:02:05]`) diventi una pillola che porta il player a quel punto, così dalla nota torno al momento della riunione.
21. Come utente, voglio che un tempo oltre la durata del Tape resti testo, così non porta il player nel nulla.
22. Come utente, voglio inserire con Ctrl+T il tempo attuale del player nel punto in cui scrivo, così annoto mentre ascolto.
23. Come utente, voglio che un Taglio sposti i tempi delle Note con la linea del tempo, e che quelli dentro il tratto tolto tornino testo, così le Note puntano ancora al momento giusto.
24. Come utente, voglio un pulsante «Salva» e Ctrl+S per salvare le Note.
25. Come utente, voglio un interruttore «Salva da solo», acceso di partenza e uguale per tutti i Tape, così chi preferisce non deve pensarci.
26. Come utente, voglio che con «Salva da solo» acceso le Note si salvino dopo una breve pausa nella scrittura e uscendo dal campo, e che lo dica «Salvate».
27. Come utente, voglio che con «Salva da solo» spento una Nota con modifiche abbia un pallino accanto al nome, così so cosa non ho salvato.
28. Come utente, voglio passare da una Nota all'altra o da una scheda all'altra senza perdere le bozze e senza domande.
29. Come utente, voglio che aprire un altro Tape, la Home, la Libreria o le Impostazioni, o chiudere la finestra, mi chieda «Salvare le modifiche alle Note?» con Salva, Non salvare e Annulla, così non perdo niente per distrazione.
30. Come utente, voglio la stessa domanda prima di avviare un'Attività o un Taglio sul Tape.
31. Come utente, voglio che durante un'Attività sul Tape le Note si leggano ma non si modifichino, come i Turni.
32. Come utente, voglio scrivere Note durante la Registrazione, così annoto mentre parlo o ascolto la call.
33. Come utente, voglio che le Note scritte durante la Registrazione si salvino da sole in una bozza che sopravvive a un guasto, qualunque sia l'interruttore.
34. Come utente, voglio che allo Stop le Note entrino nel Tape insieme all'audio e al testo.
35. Come utente, voglio che durante la Registrazione Ctrl+T inserisca il tempo della Registrazione (pause escluse), lo stesso che avrà il Tape.
36. Come utente, voglio che, se il Tape non si crea e resta solo l'Ogg, ogni Nota diventi un file Markdown accanto all'Ogg, così non perdo niente.
37. Come utente, voglio vedere chi ha scritto una Nota l'ultima volta e quando («Assistente (Claude) · 17:52», «Tu · 18:03»).
38. Come utente, voglio che una Nota cambiata da un Assistente si ricarichi da sola se non ho modifiche in sospeso.
39. Come utente, voglio che, se un Assistente cambia una Nota mentre ho modifiche non salvate, il mio testo resti e un avviso mi faccia scegliere tra «Tieni le mie» e «Usa le sue».
40. Come Assistente, voglio l'elenco delle Note di un Tape con nome, autore e ora.
41. Come Assistente, voglio leggere il testo di una Nota.
42. Come Assistente, voglio creare una Nota con un nome o riscriverne una mandando il testo atteso, così non sovrascrivo una modifica dell'utente.
43. Come Assistente, voglio rinominare una Nota, con un errore distinto se il nome è già usato.
44. Come utente, voglio che un Assistente non possa eliminare Note, perché non si recuperano.
45. Come utente, voglio che leggere le Note serva solo il permesso di lettura e scriverle quello di modifica, come per il resto del Tape.
46. Come Assistente, voglio un prompt «Scrivi le Note» che mi dica come fare il riepilogo di un Tape.
47. Come utente, voglio che il riepilogo dell'Assistente sia una Nota nuova «Riepilogo» (nella Lingua del parlato, «Riepilogo 2» se c'è già) con Decisioni, Da fare e Aperto, ogni punto con il suo tempo, senza toccare le mie Note.
48. Come utente, voglio che la ricerca della Libreria trovi le parole delle Note e che il risultato dica in quale Nota sono.
49. Come utente, voglio che un risultato in una Nota apra il Tape sulla scheda Note, su quella Nota.
50. Come utente, voglio che Esporta Markdown… metta le Note prima della Trascrizione, ognuna sotto «## nome».
51. Come utente, voglio che Copia testo resti la sola Trascrizione.
52. Come utente, voglio che scrivere Note non accenda «Corretto a mano», perché non cambia la Trascrizione.
53. Come utente, voglio che correggere un Turno, rinominare un Parlante o ritrascrivere il Tape non tocchi le Note.
54. Come utente, voglio scheda, pulsanti, avvisi, dialog e prompt in tutte e sei le lingue dell'interfaccia.
55. Come utente di un lettore di schermo, voglio che la riga dei nomi sia un elenco di schede, che il pallino dica «Modifiche non salvate» e che i tempi dicano «Ascolta da 12:40».

## Implementation Decisions

- **Formato (ADR-0032).** Voce `note.json` nello zip del Tape: `{ "note": [{ "nome", "testo", "autore": { "tipo": "utente" | "assistente", "client"? }, "modificata": ISO 8601 }] }` in ordine di creazione. Manca = nessuna Nota. `trascrizione.json` e la sua `version` non cambiano. Nel modulo `tape`: lettura delle Note e una funzione che le sostituisce con `replace` sotto il lock di riscrittura (ADR-0031), con il controllo del testo atteso della Nota toccata. `taglia` aggiunge `note.json` alle voci riscritte e sposta i Tempi nella Nota con la stessa mappa dei tempi delle Frasi; `rewrite` e `replace_transcription` la copiano già come voce estranea.
- **Tempi nella Nota.** Un modulo puro, usato da Rust (Taglio, controllo in `write_note`) e con la stessa regola nel frontend (pillole): riconosce `[m:ss]`, `[mm:ss]` e `[h:mm:ss]` fuori dal codice inline e dai link; sposta e smonta i tempi dato un elenco di tratti tolti. Casi di test uguali nei due lati, come per i Silenzi.
- **Comandi Tauri.** Sottili, delegano a un manager delle Note: leggere le Note del Tape, salvare una Nota (nome, testo, testo atteso), creare, rinominare, eliminare. Errori nuovi in `AppError` con il loro codice: nome già usato, nome non valido (vuoto o oltre 60 caratteri), Nota cambiata da fuori (testo atteso diverso), Nota non trovata. Niente `u64` nei tipi esportati.
- **Registrazione.** Il manager della Registrazione tiene una bozza `<nome dell'Ogg>.note.json` nella cartella temporanea degli Ogg, riscritta a ogni salvataggio automatico (sempre acceso durante la Registrazione). Allo Stop, quando si scrive il Tape, la bozza diventa `note.json`; senza Tape (ripiego sull'Ogg) ogni Nota non vuota diventa `<titolo> <nome>.md` accanto all'Ogg, senza sovrascrivere.
- **Impostazioni.** Nuovo booleano `note_salva_da_solo`, vero di default e quando manca. L'interruttore nella riga delle Note lo cambia; anche Impostazioni → Generale lo mostra.
- **Indice e ricerca.** La tabella `ricerca` riceve righe per le Note (una per Nota, con il nome e il testo; `frase` vuota), con la versione dello schema alzata per ricostruire l'indice. Il riallineamento rilegge `note.json`. Un risultato in una Nota porta il nome della Nota e un passaggio (`snippet`), e la sua apertura passa alla route la Nota da mostrare.
- **Esportazione.** `transcript::render` in Markdown mette prima le Note come «## nome» con il testo com'è; il testo semplice e Copia testo non cambiano.
- **MCP (ADR-0031).** Strumenti `list_notes` e `read_note` (lettura), `write_note(tape, nome, testo, atteso)` che crea se la Nota non c'è (atteso vuoto) o riscrive se il testo coincide, e `rename_note` (modifica), tutti con il lock su file e il rifiuto `activityInProgress`; nessuna eliminazione. L'autore è «assistente» con il nome del client dall'`initialize` MCP. Prompt «Scrivi le Note» con la ricetta di `PRODUCT.md`: legge i Turni, scrive una Nota nuova «Riepilogo» tradotta nella Lingua del parlato (con il primo nome libero), sezioni Decisioni, Da fare (`- [ ] Parlante: …`), Aperto, ogni punto con il tempo d'inizio della Frase da cui viene.
- **Editor (frontend).** Nuova feature `features/notes`: CodeMirror 6 con `@codemirror/lang-markdown` e un plugin di decorazioni nostro che nasconde i segni fuori dalla riga del cursore, sostituisce `- [ ]`/`- [x]` con caselle e i Tempi nella Nota con pillole; la barretta sospesa sulla selezione riusa lo stile della barretta del Turno. La logica pura (tempi, nomi predefiniti e unici, stato delle bozze, regola di salvataggio) sta in file `*.ts` con il loro `*.test.ts`. Token di `DESIGN.md` soltanto; le dipendenze nuove a versione fissa come le altre.
- **Bozze e uscita.** Lo stato delle bozze per Nota vive nella route del Tape (reducer accanto a `home-state.ts`). Il dialog «Salvare le modifiche alle Note?» blocca la navigazione della route, Ctrl+N/Ctrl+O, l'avvio di Attività e Tagli, e la chiusura della finestra (evento di chiusura richiesta di Tauri).
- **Cambiamenti da fuori.** Al ritorno in primo piano (ADR-0031) l'app rilegge le Note del Tape aperto: le Note senza bozza si aggiornano, quelle con bozza mostrano l'avviso con «Tieni le mie» (salva sopra con il testo nuovo come atteso) e «Usa le sue» (scarta la bozza).
- **Scorciatoie.** Nel campo di una Nota Ctrl+B, Ctrl+I, Ctrl+K formattano e non arrivano alla barra laterale o alla ricerca; Ctrl+S salva e Ctrl+T inserisce il tempo valgono solo nella scheda Note. Il pannello «Scorciatoie da tastiera» guadagna il gruppo Note.
- **Documenti da aggiornare.** `docs/sviluppo/tape-e-libreria.md` (formato, ricerca, export), `docs/sviluppo/registrazione.md` (bozza delle Note), `docs/sviluppo/impostazioni-e-servizi.md` (strumenti e prompt MCP, impostazione), `docs/sviluppo/frontend.md` (feature Note), `DESIGN.md` (componente Note), `AGENTS.md` se cambia la struttura.

## Testing Decisions

- Un buon test controlla il comportamento visibile da fuori: cosa c'è nel Tape dopo un'operazione, cosa restituisce uno strumento, quale testo produce l'esportazione. Niente test sui componenti React.
- **Rust, `tape`:** un Tape senza `note.json` ha zero Note; salvare, creare, rinominare ed eliminare riscrivono solo `note.json` e lasciano le altre voci byte-identiche; il testo atteso diverso rifiuta senza scrivere; nomi duplicati o oltre 60 caratteri rifiutati; il Taglio sposta i tempi, smonta quelli tolti e conserva il resto del Markdown carattere per carattere; correzioni del Turno e ritrascrizione conservano `note.json`. Esempi: i test esistenti di `taglia` e `rewrite`.
- **Rust, Registrazione:** la bozza nella cartella temporanea entra nel Tape allo Stop; senza Tape diventa file `.md` accanto all'Ogg senza sovrascrivere. Esempio: `la_trascrizione_dal_vivo_salva_il_markdown_solo_senza_tape`.
- **Rust, MCP:** permessi (lettura e modifica), `activityInProgress`, creazione e riscrittura con atteso, rinomina con nome preso, autore con il nome del client. Esempi: i test degli strumenti di ADR-0031 in `mcp.rs`.
- **Rust, indice ed export:** la ricerca trova una parola di una Nota e restituisce il suo nome; Esporta Markdown mette le Note prima della Trascrizione.
- **Frontend (`bun test`):** riconoscimento e spostamento dei Tempi nella Nota (gli stessi casi di Rust), nomi predefiniti e unici, regola delle bozze (cosa chiede il dialog, cosa no), conflitto con una Nota cambiata da fuori.
- **Verifica a mano** (`docs/sviluppo/verifica-manuale.md`): editor ibrido nei due temi con cursore e selezione visibili, IME e Ctrl+Z, pillole dei tempi, dialog all'uscita, Note durante una Registrazione vera e dopo Stop, un Assistente vero che scrive il Riepilogo con l'app aperta.

## Out of Scope

- Note fuori da un Tape (un file aperto e non ancora trascritto, la Home).
- Immagini, tabelle, blocchi di codice evidenziati e altri elementi del Markdown oltre a titoletti, grassetto, corsivo, link, elenchi, caselle e codice inline (si scrivono ma restano testo).
- Cronologia delle versioni di una Nota e Cestino delle Note.
- Eliminazione di Note da parte degli Assistenti.
- Riordino delle Note a trascinamento.
- Sincronizzazione o condivisione delle Note fuori dal Tape.
- Collegamento automatico dei tempi alle Frasi (il tempo è un punto dell'audio, non un riferimento a una Frase).

## Further Notes

- I Tape restano leggibili dalle versioni precedenti dell'app, che ignorano `note.json`; una riscrittura fatta da una versione vecchia la copia com'è, tranne il Taglio, che la copia senza spostarne i tempi.
- Le anteprime sono in `.impeccable/`, ignorata da git: restano nel checkout principale come riferimento per la revisione finale.
