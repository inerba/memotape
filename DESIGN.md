---
name: Memotape
description: Trascrizioni locali che si leggono come un documento e si ascoltano come un nastro.
colors:
  paper: "oklch(0.985 0.006 85)"
  ink: "oklch(0.24 0.012 60)"
  sheet: "oklch(0.995 0.004 85)"
  ink-button: "oklch(0.27 0.012 60)"
  ink-button-foreground: "oklch(0.975 0.007 85)"
  paper-shade: "oklch(0.95 0.009 80)"
  paper-muted: "oklch(0.955 0.008 80)"
  ink-muted: "oklch(0.5 0.016 65)"
  paper-hover: "oklch(0.935 0.012 80)"
  rule: "oklch(0.905 0.011 75)"
  rule-input: "oklch(0.875 0.013 75)"
  ring-sage: "oklch(0.55 0.07 128)"
  brick: "oklch(0.55 0.17 30)"
  sidebar-paper: "oklch(0.958 0.009 80)"
  sidebar-selected: "oklch(0.915 0.02 105)"
  sidebar-rule: "oklch(0.9 0.011 75)"
  play: "oklch(0.5 0.075 128)"
  play-foreground: "oklch(0.985 0.006 85)"
  play-soft: "oklch(0.935 0.03 118)"
  voice-1: "oklch(0.76 0.07 15)"
  voice-2: "oklch(0.74 0.055 245)"
  voice-3: "oklch(0.77 0.085 75)"
  voice-4: "oklch(0.72 0.065 150)"
  voice-5: "oklch(0.7 0.06 305)"
  voice-6: "oklch(0.72 0.05 200)"
  close: "#c42b1c"
typography:
  display:
    fontFamily: "Commissioner Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "2.75rem"
    fontWeight: 500
    lineHeight: 1.1
    letterSpacing: "-0.015em"
    fontFeature: "font-variation-settings: 'FLAR' 100, 'VOLM' 50"
  display-tape:
    fontFamily: "Commissioner Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "2rem"
    fontWeight: 500
    lineHeight: 1.1
    letterSpacing: "-0.015em"
    fontFeature: "font-variation-settings: 'FLAR' 100, 'VOLM' 50"
  wordmark:
    fontFamily: "Commissioner Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "1.375rem"
    fontWeight: 500
    letterSpacing: "-0.01em"
  reading:
    fontFamily: "Inter Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 400
    lineHeight: 1.7
    fontFeature: "\"cv11\", \"ss01\""
  meta:
    fontFamily: "Inter Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 400
    fontFeature: "\"cv11\", \"ss01\", \"tnum\""
  body:
    fontFamily: "Inter Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.43
    fontFeature: "\"cv11\", \"ss01\""
  label:
    fontFamily: "Inter Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 500
    lineHeight: 1.43
  caption:
    fontFamily: "Inter Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 400
    lineHeight: 1.33
  group-label:
    fontFamily: "Inter Variable, ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.6875rem"
    fontWeight: 500
    letterSpacing: "0.07em"
rounded:
  sm: "6px"
  md: "8px"
  lg: "10px"
  xl: "14px"
  2xl: "16px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "20px"
  xl: "32px"
  2xl: "40px"
  3xl: "48px"
components:
  button-primary:
    backgroundColor: "{colors.ink-button}"
    textColor: "{colors.ink-button-foreground}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "0 16px"
    height: "40px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "0 14px"
    height: "36px"
  button-ghost-hover:
    backgroundColor: "{colors.paper-hover}"
  button-outline:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "32px"
  button-icon:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    size: "32px"
  button-play:
    backgroundColor: "{colors.ink-button}"
    textColor: "{colors.ink-button-foreground}"
    rounded: "{rounded.full}"
    size: "40px"
  input-search:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 56px 0 36px"
    height: "36px"
  select-native:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "0 32px 0 10px"
    height: "32px"
  chip:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "4px 10px"
  tab:
    backgroundColor: "transparent"
    textColor: "{colors.ink-muted}"
    typography: "{typography.label}"
    height: "44px"
  tab-selected:
    textColor: "{colors.ink}"
  menu-item:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "0 8px"
    height: "32px"
  menu-item-hover:
    backgroundColor: "{colors.paper-hover}"
  sidebar-item-selected:
    backgroundColor: "{colors.sidebar-selected}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
  turn:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.reading}"
    rounded: "{rounded.xl}"
    padding: "12px 16px"
  turn-active:
    backgroundColor: "{colors.play-soft}"
  float-panel:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    rounded: "{rounded.2xl}"
    padding: "10px 16px 12px"
  window-close-hover:
    backgroundColor: "{colors.close}"
    textColor: "#ffffff"
    width: "46px"
---

# Design System: Memotape

## Overview

**Creative North Star: "Il foglio e il nastro"**

Un Tape si legge come un documento e si ascolta come un nastro, e le due cose sono collegate. Il testo sta al centro, su un foglio di carta tiepida, in una colonna da lettura; i comandi stanno ai margini (barra laterale, riga del titolo, player sospeso in basso) e compaiono solo dove servono. Rifiuta l'impianto da pannello di controllo: niente fila di pulsanti sopra un'area di testo, niente status bar.

La densità è quella di un'app da lettura e ascolto lunghi: molto respiro attorno alla colonna, controlli piccoli e quieti, un solo pulsante pieno per vista. La carta è calda in entrambi i temi (lo scuro è "la stessa carta, di sera"), l'inchiostro è bruno-nero, e un unico accento salvia segna soltanto l'audio in ascolto. Il tema visivo è espresso da `html[data-theme]`, inizializzato prima della prima pittura con la scelta salvata; `Sistema` segue Windows tramite `prefers-color-scheme`. In Impostazioni → Generale si può fissarlo chiaro o scuro.

Il rischio dichiarato del mondo è il generico "carta calda + serif": lo si tiene preciso con la tipografia (un sans glifico solo per i titoli, sans leggibile per tutto il resto, cifre tabulari per i tempi), con il ritmo della colonna e con l'accento usato con parsimonia.

**Key Characteristics:**
- Carta tiepida, barra laterale di un tono più scura, inchiostro bruno-nero; tema chiaro e scuro gemelli.
- Un solo accento, salvia (`play`), riservato all'audio: ascolto, avanzamento, conferma.
- Sei colori tenui per i Parlanti, solo come pallini.
- Titoli in Commissioner con le aste svasate (`FLAR`); interfaccia e testo in Inter; tempi in cifre tabulari.
- Bordi da un pixel, angoli morbidi, una sola ombra (`shadow-float`) per gli strati sospesi.
- Barra del titolo integrata con i pulsanti di Windows a destra.

## Colors

Una carta e un inchiostro caldi (tinta 60–85), un accento salvia (tinta 118–128) e una piccola famiglia di pastelli per le voci; i valori OKLCH di `src/app/global.css` sono la fonte normativa, con i gemelli scuri sotto `:root[data-theme="dark"]`. `Sistema` aggiorna l'attributo quando cambia la preferenza di Windows.

### Primary
- **Salvia d'ascolto** (`play`): l'unico accento. Pillola Segui l'audio accesa (`play-soft`), parte già ascoltata della forma d'onda, riempimento delle barre di avanzamento, barre animate del turno in ascolto, spunte di conferma (Copia testo, avviso riuscito), freccia di "Torna al punto in ascolto", selezione del testo (con testo `play-foreground`), cursore del testo nei campi e cursore del volume (`accent-color`). Nel tema scuro si schiarisce per restare leggibile sull'inchiostro. Nelle Frasi in correzione il cursore usa `foreground`, per distinguersi dal fondo e dalla selezione in entrambi i temi.
- **Salvia tenue** (`play-soft`): il fondo del turno in ascolto (al 55%), la traccia delle barre di avanzamento e l'evidenziazione dei risultati di ricerca. La Frase in ascolto prende invece `play` al 18%.
- **Anello salvia** (`ring-sage`): l'anello di focus da 3 px, quasi sempre al 40% (al 25–30% sui campi).

### Secondary
- **Voci 1–6** (`voice-1` … `voice-6`: rosa, azzurro polvere, ocra, salvia, lilla, verde acqua): un colore per Parlante, nell'ordine di comparsa: il pallino da 7 px del Copione, il nodo da 10 px del Nastro, la sottolineatura da 3 px del nome nell'Intervista e il pallino da 10 px della scheda Parlanti. Il Parlante non determinato e il mix senza Parlanti hanno un cerchio vuoto con il bordo `ink-muted`.

### Tertiary
- **Mattone** (`brick`, il `destructive` di shadcn): errori, Elimina nei menu, chip di avviso (bordo al 30%), pallino della Registrazione in corso.
- **Rosso di Chiudi** (`close`): solo il fondo in hover e focus del pulsante Chiudi della finestra, uguale nei due temi, come in Windows.
- **Giorno e sera** (`day-paper`, `day-sidebar`, `day-ink`, `night-paper`, `night-sidebar`, `night-ink`): copie fisse di carta, barra laterale e inchiostro dei due temi, solo per le miniature della scelta del tema in Impostazioni → Generale ("Come Windows" le taglia in diagonale).

### Neutral
- **Carta** (`paper`): fondo del pannello centrale.
- **Foglio** (`sheet`, il `card` e `popover` di shadcn): superfici sospese e piccole superfici sopra la carta: player, dock, avvisi, menu, chip, pulsanti outline, select. Gli avvisi stanno nel flusso sotto la barra in alto, senza ombra: non coprono mai il testo.
- **Carta della barra laterale** (`sidebar-paper`): un tono più scura e calda della carta; nel tema scuro più scura del pannello.
- **Selezione della barra laterale** (`sidebar-selected`): il Tape aperto nei Recenti, una carta appena olivastra.
- **Inchiostro** (`ink`) e **Inchiostro dei pulsanti** (`ink-button`): testo e pulsante pieno; nel tema scuro i ruoli si invertono (pulsante carta chiara su fondo scuro).
- **Inchiostro tenue** (`ink-muted`): metadati, breadcrumb, tempi dei turni, schede non scelte, etichette dei gruppi, segnaposti.
- **Carta in hover** (`paper-hover`, l'`accent` di shadcn): fondo in hover di pulsanti ghost, icone e voci di menu. **Carte di servizio** (`paper-shade`, `paper-muted`): fondi secondari di shadcn.
- **Filo** (`rule`) e **Filo dei campi** (`rule-input`): bordi da 1 px, divisori, traccia dell'interruttore spento e pollice delle barre di scorrimento.

### Named Rules
**The One Accent Rule.** La salvia significa "audio": ciò che suona, è stato ascoltato o è andato a buon fine, e ogni interruttore acceso (7 ottobre 2026: "Segui l'audio", dal 10 ottobre una pillola nel player, e le impostazioni che si salvano subito). Non colora pulsanti primari, link, schede scelte o titoli; il pulsante pieno resta inchiostro.

**The Dot-Only Voices Rule.** I colori dei Parlanti compaiono solo nel pallino o nel nodo, e nell'Intervista nella sottolineatura del nome: mai sul testo, sui fondi dei turni o sui bordi.

**The Token-Only Rule.** Nei componenti si usano solo i token (`bg-background`, `text-destructive`, `bg-play`…), mai colori fissi; le eccezioni sono `close`, che è già un token, e il logo (`BrandMark`), che ha i colori fissi del marchio: salvia `#576b3c`, inchiostro `#241e1a`, carta `#fcfaf6` e senape `#f2c14e`, con la bobina bruna `#45392f` nel tema scuro (`docs/brand/LINEE-GUIDA.md`).

## Typography

**Display Font:** Commissioner Variable (con ui-sans-serif, system-ui, sans-serif), con `FLAR` 100 (aste svasate alle estremità, come le lettere incise) e `VOLM` 50 (forme più piene)
**Body Font:** Inter Variable (con ui-sans-serif, system-ui, sans-serif), con `cv11` e `ss01` su tutto il body
**Label/Mono Font:** nessun mono; i tempi usano le cifre tabulari di Inter (`tabular-nums`)

**Character:** Un sans glifico, con le aste svasate, per il nome delle cose (titolo del Tape, benvenuto, marchio) e un sans neutro e molto leggibile per tutto ciò che si legge a lungo o si usa; l'accoppiamento dà al documento un'aria editoriale senza rendere l'interfaccia decorativa.

### Hierarchy
- **Display** (500, 2.75rem, 1.1, −0.015em): il titolo del `DocumentHeader` (Libreria, Registrazione dal vivo, file da trascrivere) e il titolo del benvenuto, al massimo 20ch con `text-wrap: balance`. Un solo display per vista.
- **Display del Tape** (500, 2rem, 1.1, −0.015em, 10 ottobre 2026): il titolo del Tape aperto e il suo campo di rinomina, con `text-wrap: pretty`, così va a capo meno spesso e mai con una parola sola in fondo; il testo sale di una riga.
- **Wordmark** (500, 1.375rem, −0.01em): "memotape" in minuscolo nella barra laterale accanto al simbolo; in Informazioni a 1.75rem. Nel resto dell'interfaccia e nei testi il nome resta "Memotape"; Impostazioni usa lo stesso font a 2rem per il titolo della pagina, con un sottotitolo in `ink-muted`.
- **Reading** (400, 1.0625rem, 1.7): il testo dei turni, nella colonna da 46rem (circa 70 caratteri a riga dopo il rientro).
- **Meta** (400, 1.0625rem, cifre tabulari, `ink-muted`): la riga data · ora · durata sotto il titolo e il sottotitolo del benvenuto (con interlinea rilassata).
- **Body** (400, 0.875rem): l'interfaccia: voci di menu, breadcrumb, ricerca, chip, tempi dei turni, testi d'aiuto.
- **Label** (500, 0.875rem): pulsanti, schede, nomi dei Parlanti.
- **Caption** (400, 0.75rem): Segui l'audio nel player, giorno nei Recenti, titoletti nei menu, il tempo di una Frase in hover.
- **Group label** (500, 0.6875rem, +0.07em, maiuscolo): solo i titoli dei gruppi della barra laterale (ATTIVITÀ, RECENTI, RISULTATI).

### Named Rules
**The Display-Names-Only Rule.** Commissioner compare solo su titoli e marchio; mai su pulsanti, menu, testo dei turni o dati.

**The Tabular Time Rule.** Ogni tempo, durata, conteggio o percentuale che può cambiare usa le cifre tabulari, così non fa saltare la riga.

## Layout

Due colonne a tutta altezza: la barra laterale a sinistra (288 px, `w-72`; chiusa una striscia da 60 px) e il pannello centrale fluido. Nessun breakpoint: la finestra va da 1200×800 al minimo di 880×600 e solo il pannello centrale si stringe; le chip vanno a capo, i titoli lunghi si troncano nel breadcrumb. La barra si chiude solo per scelta dell'utente (Ctrl+B o il pulsante nella riga del titolo), mai da sola, e lo stato si ricorda in `localStorage` (`memotape.barraLaterale`, `features/library/barra-laterale.ts`).

- **Riga del titolo** (48 px in ogni vista, trascinabile, bordo inferiore da 1 px): a sinistra il pulsante icona della barra laterale (32 px, a 12 px dal bordo e 8 px dall'alto; `PanelLeftClose`/`PanelLeftOpen`), poi il breadcrumb (52 px di margine, `SIDEBAR_TOGGLE_PADDING`), azioni del documento a destra, subito prima dei pulsanti di Windows (46×36 px), fissi nell'angolo in alto a destra. Il pulsante della barra sta sopra la riga (`SidebarToggle`, assoluto nel pannello centrale), come i pulsanti di Windows: la riga resta trascinabile intorno.
- **Colonna del documento**: centrata, al massimo 46rem, con 40 px di margine laterale e 48 px sopra il titolo. Tutte le registrazioni usa una colonna più larga (60rem); il player e il dock di Registrazione 52rem; gli avvisi 46rem, come il documento.
- **Barra delle schede**: appiccicata in cima allo scorrimento, sul fondo carta, con il filo inferiore; le schede distano 24 px. Solo le schede: Segui l'audio sta nel player (10 ottobre 2026).
- **Ritmo**: nel Copione e nell'Intervista i turni hanno 5–6 px sopra e sotto e 12 px di sfondamento laterale (lo sfondo del turno attivo sporge oltre la colonna del testo); nel Nastro i turni si toccano lungo la linea, con 14 px sotto il testo. Gruppi della barra laterale a 20 px l'uno dall'altro.
- **Player e dock**: ancorati in basso al pannello centrale, sospesi sopra il testo, mai a tutta larghezza.
- Le barre di scorrimento sono sottili (12 px con 4 px di bordo trasparente, quindi 4 px visibili), color filo, senza frecce; il pannello riserva lo spazio (`scrollbar-gutter: stable`).

## Elevation & Depth

Sistema piatto su carta, con un solo livello sospeso. La profondità di base viene dal tono (barra laterale più scura, foglio più chiaro della carta) e dai fili da 1 px; l'ombra è riservata a ciò che galleggia sopra il testo.

### Shadow Vocabulary
- **Float** (`box-shadow: 0 12px 32px -14px var(--float-shadow), 0 2px 6px -2px var(--float-shadow)`; `--float-shadow` è `oklch(0.3 0.03 60 / 0.22)` di giorno e `oklch(0.05 0.01 60 / 0.6)` di sera): player, dock di avanzamento e di Registrazione, menu a comparsa, la pillola "Torna al punto in ascolto" e l'etichetta del tempo di una Frase in hover.
- Il pollice dell'interruttore porta lo `shadow-sm` di Tailwind; i pulsanti outline di shadcn il loro `shadow-xs`. Non sono ruoli del sistema.

### Named Rules
**The One Shadow Rule.** Solo `shadow-float`, e solo per gli strati sospesi. Card, chip, turni e la barra laterale restano piatti: la separazione è tonale o un filo da 1 px.

## Shapes

Angoli morbidi su una base di 10 px (`--radius: 0.625rem`): 6 px per le piccole superfici incise (evidenziazioni, tasti), 8 px per pulsanti, campi, select e voci di menu, 10 px per chip, ricerca e voci della barra laterale, 14 px per i menu, gli avvisi e lo sfondo dei turni, 16 px per player e dock. Il cerchio pieno è per il Play, i pallini, l'interruttore, le barre di avanzamento e la pillola di ritorno. Ogni bordo è un filo da 1 px del colore `rule` (nel tema scuro carta chiara al 10%). L'unica geometria ricorrente oltre ai rettangoli morbidi è la barra verticale dell'audio: nella forma d'onda del player e nelle tre barre del turno in ascolto. Il marchio è a parte: il mozzo di una bobina visto da vicino, la bobina in inchiostro che esce dal quadrato salvia in basso a destra, il mozzo color carta con il foro a sei denti e due strisce da musicassetta in alto, senape e carta (`components/brand-mark.tsx`, che usa le SVG di `docs/brand/svg`; a 28 px nella barra laterale con la versione piccola a una striscia, a 64 px in Informazioni con quella principale).

## Components

### Buttons
Quieti e piccoli; uno solo pieno per vista.
- **Shape:** angoli morbidi (8 px).
- **Primary:** inchiostro pieno con testo carta, 40 px di altezza: Nuova registrazione (pulsante diviso con la freccia del menu, separata da un filo carta al 15%), le azioni del benvenuto, Trascrivi. In hover l'inchiostro al 90%.
- **Ghost:** trasparente, fondo `paper-hover` in hover; Importa un file e le voci in fondo alla barra laterale.
- **Outline:** foglio con filo, 32 px: Copia testo, che diventa una spunta salvia per conferma.
- **Icon:** 32 px, icone da 16 px a inchiostro all'80%, fondo `paper-hover` in hover; "…" del documento e i salti di ±10 s.
- **Focus:** anello salvia da 3 px al 40%, senza outline del browser.

### Chips
- **Style:** testo `ink-muted` a 14 px con l'icona da 14 px, senza fondo né bordo, a 16 px l'una dall'altra: Parlanti e Corretto a mano sotto il titolo. Nelle finestre alte meno di 700 px la testata si stringe (24 px sopra; il display della Libreria a 2.25rem).
- **Dettagli:** l'unica chip-pulsante, pillola outline da 28 px con ⓘ e chevron, apre un popover (il menu nativo di `popover-menu.tsx`, 320 px) con origine, modello, Lingua del parlato e Ingressi in due colonne: etichette `ink-muted`, valori inchiostro a 13,5 px.
- **Warning:** senza fondo, bordo e testo mattone.

### Cards / Containers
- **Corner Style:** 16 px per player e dock, 14 px per avvisi e menu, 10 px per la scheda Attività della barra laterale.
- **Background:** foglio (`sheet`); nella barra laterale la scheda Attività è carta al 70%.
- **Shadow Strategy:** `shadow-float` solo per ciò che è sospeso (vedi Elevation & Depth).
- **Border:** sempre il filo da 1 px.
- **Internal Padding:** player 14×10 px; dock 20×14 px; avvisi 16×12 px; menu 6 px.

### Inputs / Fields
- **Ricerca:** 36 px, 10 px di angolo, carta al 70% sulla barra laterale, lente a sinistra e il tasto "Ctrl K" (11 px, filo) a destra.
- **Select nativo:** 32 px, foglio, filo `rule-input`, chevron da 16 px; è un `<select>` vero, nel tema giusto grazie a `color-scheme`.
- **Nome del Parlante:** campo da 28 px sul posto, carta, filo `rule-input`, peso 500.
- **Interruttore:** lo `Switch` di shadcn con la traccia salvia da acceso (`SWITCH_CLASS`), per ogni impostazione che si salva subito; in Impostazioni prima dell'etichetta, nei menu a destra.
- **Segmenti:** `Segmented` (`components/segmented.tsx`), poche scelte affiancate su una traccia `secondary` da 6 px di angolo; il segmento scelto è foglio con il filo e l'inchiostro, gli altri `ink-muted`, 24 px, 12 px di testo (nel menu di Nuova registrazione 28 px e 13 px, a tutta larghezza: un'etichetta lunga allarga il suo segmento). Etichette brevi a parole (anche "Spento"), nome intero nel tooltip e per i lettori di schermo.
- **Indicatore di livello:** `LevelMeter`, 8 px (6 px nella barra ridotta), traccia `secondary`, zone fisse salvia fino a −15 dBFS, senape (`level-warm`, la senape del marchio con il gemello scuro) fino a −3, mattone oltre; tacche ogni 10 dB e segno di picco in inchiostro che resta 1 s. In Muto grigio tratteggiato.
- **Focus:** il bordo diventa salvia e compare l'anello da 3 px al 25–30%. **Errore:** bordo mattone. **Disabilitato:** opacità al 50%.

### Navigation
- **Barra laterale:** marchio (simbolo e "memotape"), pulsante primario, Importa, ricerca, poi i gruppi Attività e Recenti con l'etichetta di gruppo; i Recenti si raggruppano per giorno (titoletti a 12 px in `ink-muted`), al più 10, poi il collegamento «Tutti i N Tape nella Libreria» (14 px, `ink-muted`, freccia a destra). Sotto il titolo, a 12 px in `ink-muted`, quando (l'ora in Oggi e Ieri, «gio 8, 17:32» nella settimana, «31 lug, 11:10» nei mesi) e la durata a parole dopo un orologio da 11 px («2 h 05 min»): la durata in cifre resta per tabella, player e Frasi. La voce aperta ha il fondo `sidebar-selected` e 10 px di angolo. In fondo, separati da un filo: Libreria con il conteggio e Impostazioni.
- **Striscia (barra laterale chiusa):** 60 px, carta della barra laterale con il filo a destra. In alto il simbolo da 28 px (Home) in una fascia da 56 px trascinabile; sotto, a 6 px l'una dall'altra, icone da 40 px con 10 px di angolo e glifo da 16 px: Nuova registrazione in inchiostro pieno (il pulsante pieno della vista), Importa, Cerca, Recenti; in fondo, a 14 px dal bordo, Libreria e Impostazioni. Hover `sidebar-selected` al 60%; la vista corrente (Home, Libreria) e i Recenti con un Tape aperto hanno il fondo `sidebar-selected`. L'Attività in corso è un pallino da 8 px in alto a destra dell'icona dei Recenti, con l'anello carta da 2 px: salvia, mattone se la Trascrizione dal vivo si è fermata (qui la Registrazione non è mattone: il pallino dice che l'Attività va). Tooltip nativi con la scorciatoia.
- **Breadcrumb:** Raccolta / Tape a 14 px, la Raccolta in `ink-muted` e cliccabile, il Tape in inchiostro; separatore `/` tenue.
- **Schede:** Trascrizione e Parlanti a 44 px, peso 500, `ink-muted`; la scelta passa all'inchiostro con un trattino inferiore da 2 px a capi tondi, sopra il filo della barra. Il conteggio accanto è in cifre tabulari tenui.
- **Menu a comparsa:** popover nativo ancorato (`position-area`), largo almeno 240 px, voci da 32 px con icona tenue a sinistra, divisori da 1 px e titoletti a 12 px; Elimina in mattone, in fondo.
- **Menu di un Tape (contestuale):** lo stesso popover, largo almeno 224 px, aperto sotto il «…» o nel punto del clic destro. Il tasto della voce sta a destra come `kbd` a 12 px in `ink-muted` («Invio», «F2», «Canc»); Sposta in ha il chevron a destra e apre un secondo pannello di fianco, con le Raccolte. Sposta nel Cestino in mattone, dopo un filo.
- **Tabella della Libreria:** `<table>` a tutta larghezza (margini da 40 px), il titolo nella colonna libera con «…» se non ci sta, Raccolta 144 px (nascosta sotto i 672 px di tabella, per esempio a 880 px di finestra), data 160 px, durata 80 px a destra, azioni 80 px. Intestazioni a 14 px in `ink-muted`, righe separate da un filo. In hover, con il focus o con il menu aperto la riga prende `accent` al 50%; il focus aggiunge un contorno salvia (`ring`) da 2 px dentro la riga; il Cestino e il «…» da 32 px compaiono in hover o con il focus.
- **Menu di Nuova registrazione:** 352 px; un riquadro per Ingresso (filo `rule`, 10 px di angolo, 6 px di margine) con nella testata icona da 16 px, nome a peso 500 e interruttore; spento, «Spento» a 12 px in `ink-muted` prima dell'interruttore, senza opacità. Le opzioni hanno un solo rientro di 30 px, allineato al nome; «Sensibilità» è un'etichetta a 12 px in `ink-muted` sopra i segmenti. In fondo, dopo un filo, Trascrivi dal vivo a peso 500 con la spiegazione a 12 px in `ink-muted`.
- **Controlli della finestra:** tre pulsanti da 46 px con icone da 16 px a tratto 1.25; Riduci e Ingrandisci con fondo inchiostro all'8% in hover, Chiudi con il rosso di Windows e l'icona bianca.

### Turno (componente firma)

L'editor continuo del Turno, definito in `.scratch/editor-turno/spec.md`, offre un unico campo di testo semplice per Turno. Le separazioni tra Frasi sono spazi o a capo modificabili: selezione, frecce, Backspace e Canc le attraversano. Invio va a capo, uscire dal campo salva, Esc scarta la bozza corrente e Ctrl+Z annulla mentre si scrive. Un Turno svuotato resta accessibile con «Blocco senza testo». Il Parziale in corso ha il proprio paragrafo e non è modificabile. Sul testo corretto l'ascolto evidenzia l'intero Turno, senza suggerire un allineamento delle parole; sul testo non modificato resta l'evidenziazione della Frase disponibile. Il cursore lampeggiante deve essere visibile nel punto cliccato e verificato nella finestra Windows, nei due temi e anche durante l'ascolto.

Tre rese degli stessi turni, scelte in Impostazioni → Generale (`vista_trascrizione`), con il testo sempre in Reading:

- **Copione** (predefinita): il nome a 12 px peso 600 in maiuscolo (+0.07em) in una colonna da 88 px, preceduto dal pallino da 7 px; con gli Ingressi separati la colonna è da 140 px e il nome ha davanti l'icona Mic o Speaker da 14 px. Un nome troppo lungo finisce con «…» e il tooltip lo dà intero. Il testo accanto, il tempo a destra (12,5 px tabulare, `ink-muted` al 55%, pieno in hover e sul turno in ascolto).
- **Intervista**: il tempo nel margine sinistro (56 px); il nome apre il paragrafo in peso 600, sottolineato di 3 px nel colore della voce, e la prima riga del testo (anche nell'editor) rientra della sua larghezza.
- **Nastro**: il tempo a sinistra, allineato a destra in una colonna da 48 px, una linea verticale da 1,5 px (inchiostro al 16%) con un nodo da 10 px per turno nel colore della voce (alone carta da 4 px), il nome a 13 px peso 500 sopra il testo. Il turno in ascolto ha le tre barre salvia al posto del nodo.

La battuta del Parlante non determinato è in `ink-muted`, con «?» al posto del nome (Copione, Intervista) o il nodo vuoto senza nome (Nastro); il lettore di schermo legge «Parlante non determinato». In Copione e Nastro un silenzio stimato di almeno 5 s tra due turni diventa un separatore «Pausa di 9 s» a 12 px `ink-muted` (nel Copione con un filo tratteggiato fino al margine, nel Nastro come tratto tratteggiato della linea). Il turno in ascolto prende il fondo `play-soft` al 55% su 10 px di angolo (nel Nastro solo dietro il testo); la Frase in ascolto ha un fondo `play` al 18% con 5 px di angolo che segue le righe. Il tempo d'inizio porta lì il player e il clic sul nome rinomina il Parlante. ▶, Riascolta (sul turno in ascolto), Unisci e Copia turno stanno in una barretta foglio con `shadow-float`, sospesa in alto a destra del turno, che compare in hover o con il focus; Copia turno per 1,5 s diventa una spunta salvia. Al passaggio su una Frase il suo tempo compare sopra, come una piccola etichetta sospesa. I Parziali sono in corsivo `ink-muted`. Il testo è un campo che si corregge sul posto senza mai toccare il player.

### Player (componente firma)
Una striscia sospesa su una riga sola, foglio, 16 px di angolo, `shadow-float`, senza il nome dell'audio: i salti di ±10 s, il Play rotondo da 40 px in inchiostro (si ingrandisce del 4% in hover), il tempo corrente, la forma d'onda (barre salvia per la parte ascoltata, inchiostro al 22% per il resto, testina verticale in inchiostro con pomello da 10 px), la durata tenue, la velocità, il volume (un'icona che apre il cursore e Silenzia a comparsa) e Segui l'audio come pillola da 28 px a 12 px: `play-soft` con l'inchiostro da accesa, filo e `ink-muted` da spenta. Durante la Registrazione la stessa striscia diventa la barra di Registrazione con livelli, timer, Pausa e Stop. Durante Trascrizione, completamento e analisi finale dei Parlanti il dock scompare: fase, avanzamento e Annulla stanno solo in Attività nella barra laterale.

### Avviso
Sospeso in alto al centro del pannello, foglio, 14 px di angolo, icona di stato (mattone per l'errore, spunta salvia per l'esito), testo a 14 px, link sottolineato alle Impostazioni quando serve e una X per chiuderlo. Entra con una dissolvenza e uno scivolamento dall'alto, solo se il movimento è consentito.

### Scorciatoie
- **Tooltip:** il `title` nativo del pulsante con la combinazione tra parentesi, nella lingua dell'interfaccia: «Copia tutto il testo (Ctrl+Maiusc+C)», «Velocità ([ ])». Niente tooltip disegnati: quelli di Windows bastano e non coprono il contenuto.
- **Pannello «Scorciatoie da tastiera»:** `<dialog>` modale al centro, largo 560 px (meno 32 px sotto quella larghezza), `popover`, filo, 16 px di angolo, `shadow-float`, padding 24×20 px; lo sfondo dietro è carta al 55%. Titolo Commissioner a 22 px con la X a destra; sotto due colonne (gruppi Generale, Ascolto, Testo, Registrazione, Libreria) con il titoletto a 11 px maiuscolo `ink-muted` (+0.07em) e righe a 13,5 px: il nome a sinistra, i tasti a destra come `kbd` da 11 px, carta (`card`), filo, 5 px di angolo, `ink-muted`.

### Movimento
Il preload di avvio approvato il 7 ottobre 2026 usa il simbolo originale da 76 px,
il logotipo da 184 px e una sola animazione: il mozzo gira intorno al suo centro
originale in 2,8 s, con velocità costante. Quadrato, strisce, bobina esterna e
logotipo restano fermi. Niente barrette o percentuali. Carta e inchiostro seguono
il tema salvato prima che compaia la finestra. Lo stato è a 14 px, il messaggio
dopo 10 s a 12 px; quest'ultimo compare sotto senza spostare il gruppo centrale.
Il preload si rimuove appena il primo contenuto dell'app è montato, senza durata
minima o attese per i modelli. Con `prefers-reduced-motion` non ruota.

Transizioni di colore da 150–200 ms con `ease-out`; avanzamento indeterminato con un tratto che scorre (1.4 s); barre dell'ascolto a 0.9 s sfasate. Animazioni d'ingresso e barre animate solo con `motion-safe`.

## Do's and Don'ts

### Do:
- **Do** usare solo i token di `global.css`; ogni nuovo colore nasce lì, con il suo gemello scuro. Il logo è l'unica eccezione.
- **Do** riservare la salvia (`play`, `play-soft`) all'audio e alle conferme, e tenere l'inchiostro per il pulsante pieno.
- **Do** mettere ogni nuovo testo lungo nella colonna da 46rem, in Inter 1.0625rem con interlinea 1.7.
- **Do** usare `shadow-float` e il foglio per ogni nuovo strato sospeso (menu, pillole), con 14–16 px di angolo. Un avviso non è sospeso: sta nel flusso e non copre il testo.
- **Do** usare cifre tabulari per ogni tempo e conteggio.
- **Do** dare a ogni controllo interattivo l'anello salvia da 3 px al 40% su `focus-visible`.
- **Do** proteggere ogni animazione con `motion-safe` (niente movimento con `prefers-reduced-motion`).

### Don't:
- **Don't** colorare di salvia pulsanti, link, schede scelte o titoli.
- **Don't** usare i colori dei Parlanti fuori dal pallino.
- **Don't** usare Commissioner fuori da titoli e marchio, né un font di sistema per i titoli.
- **Don't** aggiungere ombre a card, chip, turni o alla barra laterale, né ombre dure sfalsate.
- **Don't** segnare il turno o la Frase in ascolto con una barra colorata sul bordo sinistro: si usa il fondo salvia tenue.
- **Don't** usare l'etichetta di gruppo maiuscola e spaziata fuori dalla barra laterale, e mai come occhiello sopra un titolo.
- **Don't** reintrodurre una status bar o una fila di pulsanti sopra il testo: fase, avanzamento e Annulla stanno solo nella barra laterale, gli esiti negli avvisi.
- **Don't** usare un colore fisso o una classe `.dark`: i token scuri arrivano da `html[data-theme="dark"]`, inizializzato prima della prima pittura e aggiornato dal provider; `Sistema` segue `prefers-color-scheme`, mentre `Tema::apply` sincronizza la finestra nativa.

## Home — ripresa del lavoro (6 ottobre 2026)

Riferimento approvato: `.impeccable/mocks/home-riprendi.png`, generato con imagegen dal precedente screenshot e scelto dall'utente. Implementazione: `src/features/library/library-home.tsx`.

La Home mantiene carta e inchiostro nei due temi. Barra del titolo da 48 px, colonna fino a 64 rem, margini da 32 a 56 px, titolo da 36 px. Un solo pannello con bordo e fondo card identifica il Tape da riprendere; sotto, tre righe separate da divisori. Le azioni di creazione rimangono nella sidebar, e compaiono anche nel centro solo con Libreria vuota. Il marchio è il ritorno alla Home, raggiungibile da tastiera. I titoli automatici compatti e i titoli personalizzati possono andare a capo senza coprire durata o comandi. Contenuto scorrevole alla dimensione minima Windows di 880×600.

L'ultima apertura è ricordata dopo un'apertura riuscita e validata rispetto alla Libreria corrente. Senza ricordo valido il pannello è etichettato Tape più recente. I valori nel mockup sono esemplificativi; l'interfaccia usa i dati della Libreria.
