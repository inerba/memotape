# Home Riprendi — verifica del 6–7 ottobre 2026

Implementazione del mockup approvato in `.impeccable/mocks/home-riprendi.png`.

## Modifiche

- Home con ultimo Tape aperto, tre Tape recenti e collegamento alla Libreria; apertura solo su azione esplicita.
- Ultimo percorso conservato localmente e aggiornato dopo apertura riuscita, rinomina, spostamento e cestinamento nell'app. Se assente dalla Libreria corrente o illeggibile, viene proposto il Tape leggibile più recente.
- Logo come ritorno alla Home, senza cancellare lo stato dell'Attività.
- Nomi automatici abbreviati nella presentazione; secondi mantenuti quando servono a distinguere due Registrazioni dello stesso minuto. Titoli personalizzati e file invariati.
- Stato iniziale di caricamento, benvenuto per Libreria vuota e testi nelle sei lingue.
- PRODUCT.md, DESIGN.md e brief della superficie aggiornati.

## Controlli automatici

| Controllo | Esito |
|---|---|
| `bun run typecheck` | Passato |
| `bun run test` | 132 passati, 0 falliti, 5113 asserzioni |
| `bun run check` | Passato, 128 file |
| `bun run format:backend` | Passato |
| `bun run lint:backend` | Passato |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 296 passati, 0 falliti, 33 ignorati |
| `bun run build` | Passato, 2224 moduli; avviso per chunk JavaScript oltre 500 kB |
| Impeccable detector sui tre file UI modificati | Nessuna segnalazione |

I tre nuovi test verificano selezione/fallback, percorsi Windows senza distinzione di maiuscole, nomi automatici nelle sei lingue, titoli personalizzati e collisioni nello stesso minuto.

## Verifica visiva e limiti

`home-dark.png` è una cattura del componente HomePage reale a 1200×800, tema scuro, con IPC Tauri simulato e Tape di prova. Verificati gerarchia, spaziature, metadati, tre recenti, footer e distinzione delle Registrazioni alle 22:18:04 e 22:18:08.

Il primo tentativo di apertura Tape ha incontrato dati mancanti nel mock (`list_models` e `tape_peaks`); la fixture è stata corretta, ma successivi timeout del server locale hanno impedito di completare la verifica interattiva. Non vengono dichiarati verificati nel browser il ciclo apertura/ritorno/riavvio, il tema chiaro e la finestra minima. Questi restano da collaudare nella WebView2 reale, insieme a trascinamento della finestra e ritorno alla Home durante una Registrazione. I test nativi con modelli/audio reali non sono stati eseguiti.

La Libreria reale non è stata modificata dai controlli. Nessun commit creato.

## Riproduzione dell'anteprima

`preview.tsx` usa HomePage reale e fixture in memoria. Dopo `bun run build`, adeguare il nome CSS in `preview.html` all'asset prodotto, quindi:

```powershell
bunx vite --config .scratch/home-riprendi/vite.config.ts
```

Aprire `http://127.0.0.1:1431/.scratch/home-riprendi/preview.html`. Varianti: `?seed`, `?empty`, `?long`, `?lang=de`, `?slow`, `?error`. La configurazione isolata usa il CSS compilato per evitare una seconda scansione Tailwind del checkout.

I file in `baseline/` conservano il contenuto precedente a questo intervento, distinto dalle altre modifiche già presenti nel checkout.
