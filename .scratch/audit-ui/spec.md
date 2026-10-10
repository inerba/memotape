Status: done

# Audit UI del 10 ottobre 2026: correzioni

Branch `fix/audit-ui`, un solo commit alla fine. Le misure dell'audit sono state prese in dev, via CDP, sul Tape "sbrocco github pm" (2 ore, 439 turni, circa 20 000 nodi).

## 1. Prestazioni durante l'ascolto

Problema: a ogni `timeupdate` `TranscriptView` (`src/features/transcription/transcript-view.tsx:109-117`) ricalcola `turnsOf`, `voiceColors` e `playingAt` e ridisegna tutti i `TurnBlock`. Misure: 12 long task da 280–390 ms in 4 s di ascolto e 1,9 s per tornare da Parlanti a Trascrizione.

Correzione:
- `useMemo` per `turns` e `colors` (dipendono da `conversation` e dalla lingua, non dalla posizione);
- `memo` su `TurnBlock`;
- `playing` e `followed` solo al turno attivo, agli altri una costante vuota stabile;
- l'altezza delle textarea del Turno con `field-sizing: content` invece di `fit()` e `ResizeObserver`, che forzavano un layout per turno al montaggio.

Provato e scartato: `content-visibility: auto` sui turni. Con l'altezza stimata dei turni mai impaginati, il salto da un risultato della ricerca a una Frase lontana finiva fuori vista.

Esito misurato in dev sul Tape di 2 ore: nessun long task in 4 s di ascolto (prima 12 da 280–390 ms); da Parlanti a Trascrizione 650 ms (prima 1903).

Niente virtualizzazione: romperebbe Ctrl+F, `scrollIntoView` sulle Frasi, il focus nelle textarea e il riconoscimento dello scorrimento a mano.

Criterio: sullo stesso Tape, in dev, nessun long task oltre 50 ms per aggiornamento della posizione. Se si estrae una funzione pura (quali turni ricevono `playing`), il suo test sta accanto.

## 2. Avvisi che coprono il contenuto

Problema: `BannerView` (`src/app/routes/home.tsx`) è `absolute top-24` e copre il titolo della Home, il titolo del Tape e, scorrendo, il testo.

Correzione:
- l'avviso di aggiornamento (`Banner.updateUrl`) passa in fondo alla barra laterale, vicino a Impostazioni; si chiude come oggi;
- tutti gli altri avvisi (errori, esiti, `notice`, `message`) stanno nel flusso sotto la `TopBar` e spingono giù il contenuto.

Aggiornare PRODUCT.md (principio 3), AGENTS.md (descrizione di `BannerView`) e `.impeccable/surfaces/src-app-routes-home-tsx.md`.

## 3. Titoli in Libreria

Problema: `AllTapes` mostra il nome del file troncato ("Registrazione 2026-10-…"), mentre Recenti e Home mostrano "Registrazione delle 17:32".

Correzione: la Libreria usa `recentTitle` (`src/features/library/recent-tapes.ts`) con il nome del file nel tooltip; la colonna Titolo prende lo spazio libero; l'ordinamento per Titolo segue il titolo mostrato.

## 4. Chip del Tape alla finestra minima

Problema: a 880×600 le chip vanno su tre righe e resta una sola riga di testo sopra il player.

Correzione: in `InfoChips` (`src/features/library/tape-header.tsx`) la chip della Diarizzazione compare solo se non è completata (tono warning). Le altre chip restano.

## 5. Nome nei testi per l'utente

Le chiavi `transcript.diarization`, `transcript.diarizationCompleted` e `transcript.diarizationIncomplete` (usate dalle chip, da `status.ts` e dall'intestazione del Markdown in `src-tauri/src/transcript.rs`) cambiano solo il testo, nelle sei lingue: "Riconoscimento dei parlanti" / "completato" / "non completato" e gli equivalenti. Il nome delle chiavi resta. Il glossario (`CONTEXT.md`, Diarizzazione) è già aggiornato.

## 6. Pallino del Parlante non determinato

Pallino neutro (`bg-muted-foreground/40`), senza consumare un colore in `voiceColors` (`src/features/transcription/phrases.ts`): i Parlanti veri tengono i colori dal primo. Lo stesso nella scheda Parlanti, se vi compare.

## 7. Aree di clic

Almeno 24 px di area cliccabile, senza cambiare l'aspetto (padding e margine negativo): tempo del turno, "Segui l'audio", intestazioni ordinabili della Libreria, "Scarica".

## Verifica

I sei controlli di AGENTS.md verdi. Rivedere l'app via CDP a 1280×800 e 880×600, chiaro e scuro: Home con un avviso, Tape in ascolto, Libreria. Ripetere la misura dei long task del punto 1.
