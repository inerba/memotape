# Memotape: regole d'uso del logo

![Kit](memotape-kit.png)

## L'idea

Il simbolo è il mozzo di una bobina visto da vicino: la bobina di nastro in inchiostro esce dal quadrato salvia in basso a destra, il mozzo color carta ha il foro a sei denti, e in alto corrono due strisce, senape e carta, come sull'etichetta di una musicassetta. Memotape trasforma il nastro delle note vocali in testo. Salvia, inchiostro e carta sono i colori dell'app; la senape esiste solo nel marchio.

## Versioni

| File (`svg/`) | Quando |
|---|---|
| `memotape-simbolo` | Versione principale, da 48 px in su: icona dell'app, avatar, dove c'è spazio per un solo segno |
| `memotape-simbolo-piccolo` | Da 24 a 32 px: quadrato a tutta tela, una striscia sola più spessa, mozzo più grande |
| `memotape-simbolo-minimo` | 16 px: niente strisce, foro tondo |
| `memotape-simbolo-…-negativo` | Le stesse tre su fondi scuri: la bobina è bruna, per staccarsi dal fondo |
| `memotape-orizzontale` / `-negativo` | Simbolo + logotipo, su fondo chiaro / scuro |
| `memotape-verticale` / `-negativo` | Simbolo sopra il logotipo, per spazi stretti e alti |
| `memotape-logotipo` / `-negativo` | Solo la scritta, quando il simbolo è già vicino |
| `memotape-simbolo-nero` / `-bianco` | A un colore: timbri, incisioni, stampa monocromatica. Il quadrato è pieno, strisce, bobina e foro sono ritagliati, il mozzo resta pieno |
| `memotape-orizzontale-nero` / `-bianco` | Lockup a un colore |
| `memotape-mozzo` (inchiostro, `-nero`, `-bianco`, `-salvia`) | Il solo mozzo, a un colore: come segno grafico o filigrana |

`png/` ha le versioni principali in raster: il simbolo a 1024 px, il simbolo a un colore, il mozzo, il logotipo e i lockup.

## Colori

| Nome | HEX | RGB | CMYK (indicativo) | Nell'app |
|---|---|---|---|---|
| Salvia | `#576b3c` | 87 107 60 | 19 0 44 58 | token `play`, `oklch(0.5 0.075 128)` |
| Inchiostro | `#241e1a` | 36 30 26 | 0 17 28 86 | token `ink`, `oklch(0.24 0.012 60)` |
| Carta | `#fcfaf6` | 252 250 246 | 0 1 2 1 | token `paper`, `oklch(0.985 0.006 85)` |
| Senape | `#f2c14e` | 242 193 78 | 0 20 68 5 | solo nel marchio |
| Bruno | `#45392f` | 69 57 47 | 0 17 32 73 | solo nelle versioni `-negativo`, al posto dell'inchiostro |

- I CMYK sono una conversione diretta, non un profilo di stampa: per la stampa vanno provati, e il Pantone va scelto su mazzetta.
- La senape non diventa un token: l'accento dell'interfaccia resta uno solo, la salvia (`DESIGN.md`).
- Contrasti: inchiostro su salvia 2,8:1, carta su salvia 5,6:1, carta su inchiostro 15,8:1. La senape sulla salvia regge come colore, non come testo.

## Logotipo

Il nome in minuscolo, "memotape", in Commissioner come nell'app: peso 500, assi `FLAR` 100 e `VOLM` 50, spaziatura −0,01 em, convertito in tracciati, quindi non serve il font per usarlo. Commissioner è sotto SIL Open Font License, che ne permette l'uso in un logo. Il logotipo è sempre in inchiostro (su chiaro) o carta (su scuro), mai salvia o senape.

Nei lockup il logotipo ha corpo pari a metà del lato del simbolo, con la fascia della x centrata sul simbolo (orizzontale), o corpo 3/8 del lato centrato sotto il simbolo (verticale).

## Spazio libero e dimensioni minime

- Intorno al logo lascia libero almeno un quinto del lato del quadrato su ogni lato.
- Simbolo: `-minimo` a 16 px, `-piccolo` da 24 a 32 px, la versione principale da 48 px in su.
- Lockup orizzontale: almeno 120 px (25 mm) di larghezza.

## Sfondi

- Su carta o bianco: versioni a colori.
- Su inchiostro o fondi scuri: `-negativo` (la bobina diventa bruna, il logotipo carta).
- Su foto o fondi colorati: il simbolo a colori, che porta con sé il suo quadrato, o le versioni a un colore.

## Da non fare

- Cambiare i colori di quadrato, strisce, bobina o mozzo, o metterci gradienti e ombre.
- Spostare il mozzo, togliere le strisce dalla versione principale o aggiungerne altre.
- Usare la versione principale sotto i 48 px.
- Allungare, ruotare o ricomporre simbolo e logotipo con altre proporzioni.
  Nel solo preload di avvio, approvato il 7 ottobre 2026, il mozzo può ruotare
  intorno al suo centro originale; tutte le altre parti restano ferme. Con
  movimento ridotto anche il mozzo resta fermo.
- Riscrivere "memotape" con un altro font o con la maiuscola nel logo (nel testo resta "Memotape").
- Mettere il quadrato salvia su un fondo salvia o verde scuro, dove sparisce.

## Nell'app e nelle icone

- `BrandMark` (`src/components/brand-mark.tsx`) usa direttamente le SVG di `svg/`: `-piccolo` a 28 px nella barra laterale, la versione principale a 64 px in Informazioni, e le `-negativo` quando il tema è scuro.
- `src-tauri/icons/` viene da `png/memotape-simbolo-1024.png` con `bun tauri icon`; `icon.ico` (da 16 a 256 px), `32x32.png`, `Square30x30Logo.png` e `Square44x44Logo.png` sono rifatti con i tagli piccoli. Il file `.tape` usa l'icona dell'exe (`bundle.fileAssociations`), quindi prende la stessa. Le icone di Windows usano sempre la versione standard: un `.ico` non segue il tema.

## Rigenerare il kit

Tutto il kit viene da `genera.py`: le misure del simbolo stanno in `CUTS`, su una griglia da 8 nel quadrato da 256. Serve il TTF variabile di Commissioner (`Commissioner[FLAR,VOLM,slnt,wght].ttf` dal repository `google/fonts`, cartella `ofl/commissioner`), fuori dal repo, e i pacchetti Python `fonttools` e `uharfbuzz`. I PNG li fa `@resvg/resvg-js-cli` con `bunx`, senza dipendenze nel progetto.

```bash
python docs/brand/genera.py --font <percorso>/Commissioner.ttf --icons
```

Senza `--icons` si rigenerano solo `svg/`, `png/` e la tavola `memotape-kit.png`.

## Note

- Prima di registrare il segno come marchio serve una ricerca di anteriorità.
