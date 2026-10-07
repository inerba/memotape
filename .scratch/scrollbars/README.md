# Verifica delle barre di scorrimento

La prova monta HomePage, Sidebar e TranscriptView reali con 80 Frasi e Tape fittizi.
Usa i CSS compilati da Vite, i font del bundle e Chromium con le barre native visibili.
Non legge o modifica la Libreria dell'utente e non verifica audio o IPC nativi.

```powershell
bunx vite build
bun build .scratch/scrollbars/preview.tsx --target browser --outdir .scratch/scrollbars/build
node .scratch/scrollbars/check.cjs
```

Controlla 1000×700, 880×600 e 1280×900: nessuna barra esterna, nessuno spostamento
del documento anche con scorrimento programmatico, una sola barra nella Trascrizione,
testo lungo scorrevole, piè di pagina visibile e assenza di "Solo sul tuo PC".

`node .scratch/scrollbars/check.cjs --baseline` ripristina nel browser il posizionamento
originale di #root: la verifica deve fallire. La correzione fissa #root ai bordi della
finestra; cambiare soltanto min-height dei pannelli o overflow del documento non
impediva lo spostamento esterno.

`*.before.*` conserva il contenuto dei file prima di questo intervento, incluse le
modifiche precedenti già presenti nel checkout.
