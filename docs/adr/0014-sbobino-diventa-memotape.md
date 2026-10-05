# Sbobino diventa Memotape, il Bino diventa Tape

L'app si chiama Memotape (memo + tape, la musicassetta delle note vocali) e il suo documento è il Tape, file `.tape`. Prima erano Sbobino e il Bino, `.bino`: le ADR, le spec in `.scratch/sbobino*` e le ricerche in `docs/research/` scritte prima di questa decisione usano ancora quei nomi e non si riscrivono. Dove leggi Bino intendi Tape, dove leggi Sbobino intendi Memotape.

Il rebrand è un taglio netto, perché l'app non è mai uscita: Memotape non legge i `.bino` e non migra niente all'avvio. I dati del PC di sviluppo li ha spostati una volta sola l'agente che ha fatto il rebrand, compresi i `.bino` rinominati in `.tape`. Si poteva fare perché dentro lo zip non c'è nulla del vecchio nome: `version` resta 1 e lo schema non cambia. Tutti i nomi tecnici sono passati a `memotape`: identifier `it.memotape.desktop`, `memotape.exe`, cartella nascosta `.memotape`, Libreria predefinita `Documenti\Memotape`, protocollo `tape`, classe `Memotape.Tape`. Restano con il vecchio nome solo la cartella del repo e `..\sbobino-deps`.

"Tape" è invariabile in tutte e sei le lingue ("i Tape") ed è maschile in italiano. Fa eccezione il plurale negli identificatori del codice, che è `tapes` (`tapes: Vec<Tape>`, `list_tapes`): un plurale uguale al singolare renderebbe ambigui i nomi.
