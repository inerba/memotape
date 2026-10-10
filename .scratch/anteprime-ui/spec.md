# Interfaccia più professionale ed ergonomica

Status: ready-for-agent

Data: 10 ottobre 2026. Nasce dall'audit UI/UX dello stesso giorno, dalle anteprime approvate e da una sessione di domande con l'utente. Anteprime: [anteprime.html](anteprime.html) (copia locale, non versionata) e l'artifact privato `https://claude.ai/artifact/LjmfevdZumYm17b1LtB56h`. Le anteprime usano titoli e testi di esempio.

## Problem Statement

L'app ha un'identità curata ma attriti nell'uso quotidiano:

- La trascrizione è frammentata: ogni Frase con «Parlante non determinato» apre un turno con la sua testata e una battuta breve occupa 80–90 px.
- Nei Recenti della barra laterale, fuori da Oggi e Ieri, compare solo l'ora; ora e durata hanno lo stesso formato (`11:23 · 2:04:41`); la lista contiene tutta la Libreria.
- La testata del Tape espone dettagli tecnici (file d'origine, modello) e il titolo da 44 px va spesso su due righe, a volte con il trattino a inizio riga; il player ripete il nome del file.
- Il menu Nuova registrazione ha tre allineamenti diversi e l'Audio di sistema spento sembra disabilitato.
- Le scorciatoie sono solo Ctrl+K e Spazio.
- Non c'è un menu contestuale; nella Libreria Elimina compare solo in hover e 23 pulsanti si chiamano tutti «Elimina».
- Alla larghezza minima (880 px) la barra laterale prende un terzo della finestra.

## Solution

Sette passi, implementati uno alla volta nell'ordine dei ticket, ciascuno su un branch suo. Ogni passo si chiude con i sei controlli verdi, PRODUCT.md, DESIGN.md e AGENTS.md aggiornati e un commit `feat:`; poi si aspetta il via dell'utente per il successivo. La Prosa (quarta vista) è un ticket separato, da specificare.

## Decisioni

### 1 · Vista della trascrizione

- Nuova impostazione `vista_trascrizione` in Impostazioni → Generale: `copione` (predefinita), `intervista`, `nastro`. Solo lì, nessun menu rapido nella vista del Tape. La vista di oggi non resta come opzione.
- Vale ovunque compaia il testo: Tape aperto, Registrazione dal vivo (con i Parziali), Frasi arrivate dopo Annulla su un file.
- Nessuna Frase cambia posto o turno: ordine dei tempi, turni di `turnsOf` come oggi.
- **Copione**: nome in maiuscoletto in una colonna (88 px, fino a 140 px con gli Ingressi separati, poi «…» e nome completo nel tooltip; icona Mic/Speaker davanti), testo accanto, tempo a destra tenue, pieno in hover e sulla riga in ascolto.
- **Intervista**: il nome apre il paragrafo in grassetto, sottolineato nel colore del Parlante; il tempo sta nel margine sinistro.
- **Nastro**: linea verticale con un nodo per turno nel colore del Parlante, tempo a sinistra, nome sopra il testo; il turno in ascolto ha le tre barre animate al posto del nodo.
- Battuta incerta: «?» (Copione, Intervista) o nodo vuoto (Nastro), testo `ink-muted`; il lettore di schermo legge «Parlante non determinato».
- Pause: in Copione e Nastro un separatore «Pausa di 9 s» («Pausa di 1 min 20 s» oltre il minuto) per silenzi stimati di almeno 5 s, con la stessa stima di `silenceMs`.
- Azioni: il tempo porta lì il player; ▶ (ascolta il turno) e Copia turno compaiono in hover o con il focus, accanto al tempo; il clic sul nome rinomina il Parlante; editor del Turno, Unisci, Segui l'audio e «Torna al punto in ascolto» invariati.
- Copia testo, Copia turno ed Esporta Markdown… non dipendono dalla vista.

### 2 · Recenti

- Barra laterale: al massimo 10 Tape, poi «Tutti i N Tape nella Libreria». Con la ricerca attiva nessun limite.
- Dopo Ieri il giorno abbreviato: «gio 8, 17:32» nella settimana, «31 lug, 11:10» nei gruppi mensili.
- Durata a parole dopo un'icona dell'orologio, nei Recenti della barra laterale e della Home: sotto il minuto i secondi («24 s»), sotto l'ora i minuti interi («59 min»), oltre «2 h 05 min». Tabella della Libreria e player restano in cifre.

### 3 · Testata e player

- Titolo del Tape e campo di rinomina a 32 px (2rem), Commissioner, `text-wrap: pretty`. «Riprendi» della Home e il titolo della Libreria restano come sono.
- Popover «Dettagli» (chip-pulsante sotto il titolo): origine, modello, Lingua del parlato, Ingressi. Restano visibili: numero dei Parlanti, Corretto a mano, Trascrizione incompleta, Diarizzazione non completata.
- Player su una riga: ±10 s, Play, tempo, Forma d'onda, durata, velocità, volume (icona con cursore a comparsa), Segui l'audio come pillola salvia. Niente nome del file. Segui l'audio esce dalla barra delle schede.

### 4 · Menu Nuova registrazione

- Un riquadro per Ingresso con l'interruttore nella testata; spento si chiude e dice «Spento», senza aspetto disabilitato.
- Un solo rientro (30 px) per dispositivo, Filtra rumore, Sensibilità, Riconosci i parlanti; Sensibilità a 13 px.
- Trascrivi dal vivo in fondo, separato, con la riga «Il testo compare mentre registri.».

### 5 · Scorciatoie

| Azione | Tasti |
|---|---|
| Nuova registrazione | Ctrl+N |
| Importa un file | Ctrl+O |
| Cerca nella Libreria | Ctrl+K (c'è già) |
| Barra laterale | Ctrl+B |
| Impostazioni | Ctrl+, |
| Riproduci o metti in pausa | Spazio (c'è già) |
| Indietro e avanti di 5 s | ← → |
| Velocità | [ ] |
| Torna al punto in ascolto | Ctrl+J |
| Copia tutto il testo | Ctrl+Maiusc+C |
| Pausa o riprendi la Registrazione | Ctrl+P |
| Pannello delle scorciatoie | Ctrl+/ |

- Frecce, Spazio e `[` `]` solo fuori dai campi di testo, come Spazio oggi. I pulsanti ±10 s restano a 10 s.
- Ogni scorciatoia segue lo stato del suo pulsante: Ctrl+N e Ctrl+O disattivate durante un'Attività, Ctrl+P solo durante una Registrazione. Stop non ha scorciatoia.
- Ctrl+N, Ctrl+O, Ctrl+P e Ctrl+Maiusc+C si intercettano prima che li gestisca WebView2 (in sviluppo l'ispettore resta su F12).
- Le scorciatoie compaiono nei tooltip dei pulsanti. Il pannello «Scorciatoie da tastiera» si apre con Ctrl+/ e da un collegamento in Impostazioni → Generale.

### 6 · Libreria e menu contestuale

- Clic destro, tasto Menu e pulsante «…» (in hover o con il focus) aprono lo stesso menu sulle righe della Libreria e sui Recenti della barra laterale, non sui risultati di ricerca: Apri, Rinomina, Sposta in ▸, Mostra in Esplora file, Copia testo, Sposta nel Cestino.
- La tabella è un solo elemento nel Tab, con le frecce tra le righe: Invio apre, F2 rinomina, Canc chiede la conferma di oggi e sposta nel Cestino.
- La colonna del titolo usa la larghezza disponibile.
- Accessibilità: «Elimina {{titolo}}» al posto di «Elimina»; l'intestazione ordinata dice il verso della sua colonna («Data, dalla più recente», «Titolo, dalla Z alla A»…) invece di una frase unica per tutte.

### 7 · Barra laterale richiudibile

- Si chiude e si apre con Ctrl+B e con un pulsante nella riga del titolo; parte sempre aperta, nessuna chiusura automatica; lo stato si ricorda in `localStorage`.
- Chiusa è una striscia di icone da 60 px con tooltip: marchio (Home), Nuova registrazione, Importa, Cerca, Recenti, Libreria, Impostazioni. Nuova registrazione, Importa, Libreria e Impostazioni agiscono subito; Cerca e Recenti riaprono la barra (Cerca con il cursore nel campo). Un'Attività in corso è un pallino salvia (mattone se guasta) sull'icona dei Recenti.

## Out of Scope

- La Prosa (ticket 08, da specificare: come si corregge una battuta dentro il paragrafo).
- Menu rapido della vista nella barra delle schede.
- Ridisegno di Impostazioni oltre al campo della vista e al collegamento alle scorciatoie.
- Ricerca dentro la trascrizione.

## Testing Decisions

- Logica pura con test accanto (`*.test.ts`): formato della durata a parole, giorno abbreviato dei Recenti, limite dei Recenti, separatori delle pause, mappa delle scorciatoie (abilitata o no per stato), navigazione della tabella.
- Rust: default e lettura di `vista_trascrizione` da un `settings.json` senza il campo; `i_bindings_committati_sono_aggiornati`.
- Ogni passo si prova anche nell'app (vedi «Pilotare l'app» in AGENTS.md), nei due temi e a 880 px.
