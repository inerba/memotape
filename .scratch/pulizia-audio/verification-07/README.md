# Verifica del ticket 07

6 ottobre 2026. Implementazione locale; stato partial per collaudo nativo/UI.
Baseline fotografata prima delle modifiche in `baseline/`, stato iniziale in
`status-iniziale.txt`. `delta.patch` confronta questa baseline, non HEAD.
Nessun commit, PR o rilascio. Modifiche precedenti conservate.

## Controlli osservati

| Controllo | Esito |
| --- | --- |
| `bun run typecheck` | Verde, exit 0; `typecheck.log` |
| `bun run test` | 126 pass, 0 fail, 4729 assertion, 20 file; `frontend-tests.log` |
| `bun run check` | Verde, 122 file; `frontend-lint.log` |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | Verde, exit 0 |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Verde, exit 0 |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 296 pass, 0 fail, 29 ignored; main/doc test 0 |

Bun: `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`. Per Rust:
`CARGO_TARGET_DIR=D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust`,
`cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config
env.LOCALAPPDATA.relative=false ...`. Una sola compilazione Rust alla volta.
Suite Rust e fmt fuori sandbox: Cestino Windows e canonicalizzazione dei
percorsi richiedevano accesso nativo. Nessuna modifica alla config Cargo.
Il test `i_bindings_committati_sono_aggiornati` passa: contratti backend e
binding non cambiati, nessuna modifica manuale o rigenerazione necessaria.

La sandbox ha restituito errori Windows 1224 su file mappati durante scritture
e Biome fix; le stesse modifiche/formattazioni fuori sandbox sono riuscite.
Detector Impeccable sulle due superfici modificate: `[]`, nessun rilievo.
Questa prova meccanica non verifica contrasto, layout o tastiera in un browser.

## Seam e prove frontend

Seam concordate dal ticket: azioni di salvataggio persistente condiviso e
proiezione dei controlli/eventi frontend. Nessuna nuova seam DSP.

- TDD: test concorrente inizialmente rosso per modulo assente, poi verde;
  test su fallimento e avvio rosso (`flush` restituiva null), poi verde.
- Cinque test del writer: cambi contemporanei su pulizia/sensibilità/Guadagno,
  scelte rapide sullo stesso campo, riavvio simulato dalla persistenza,
  rollback/rebase dopo errore, trasporto fallito e riprova, attesa all'avvio
  anche per scelte aggiunte durante l'attesa. Quest'ultimo test è diventato
  rosso durante la review Standards e verde dopo la correzione di `flush`.
- Due regressioni di rendering sui componenti reali: Microfono/Sistema/Entrambi,
  nessun profilo File-misto, default, etichette/descrizioni, scelta salvata
  distinta dal bypass, guasto obsoleto ignorato e un solo Ingresso in bypass.
- I test già presenti conservano sessioni/eventi separati e sei traduzioni.

126 è il totale della suite; 7 sono nuovi test. La persistenza del writer è
simulata alla frontiera IPC. I test Rust esistenti verificano SettingsStore
e il contratto di avvio/live; non costituiscono un riavvio dell'app dalla barra.

## Tentativi UI e limiti

Inventario nativo: Memotape installata già aperta; nessuna WebView raggiungibile
su 9222/9223 e nessun Vite su 1420. Non sono state chiuse le app dell'utente.
CUA non offre app native in questa sessione. Il banco Vite è stato avviato
solo per questo task su 1427 (anche fuori sandbox) e poi fermato.
IAB: `ERR_CONNECTION_TIMED_OUT`, poi timeout su localhost; Chrome:
`Browser is not available: chrome`. Nessuna interazione o screenshot riuscito.

`ui.html`/`ui.tsx` costituiscono un banco riproducibile con componenti reali e
IPC finto, senza accesso alle Impostazioni/Libreria dell'utente. **Non eseguito**.
Per aprirlo: `bun run dev --host 127.0.0.1 --port 1427`, quindi
`http://127.0.0.1:1427/.scratch/pulizia-audio/verification-07/ui.html`.
Query `source=mic|system|both` e `lang=it|en|fr|es|de|pl`; usa solo localStorage
del banco. `window.ticket07.emit` può iniettare eventi di sessione corrente/vecchia.
Il banco non è parte dell'app né una prova di WASAPI/ASR.

## Criterio manuale ancora aperto

1. Con l'app aggiornata, attivare Entrambi, DFN3 e Trascrivi dal vivo. Controllare
   che ogni Ingresso abbia pulizia, sensibilità e Guadagno distinti; nessun
   File/misto o intensità DFN3. Tab/Spazio/frecce e Narrator devono identificare
   l'Ingresso, il valore e le spiegazioni. Ripetere con le sei lingue e i temi.
2. Cambiare dalla barra durante parlato e silenzio, aprire Impostazioni e
   verificare stessi valori; cambiare da lì e tornare alla barra. Alternare
   rapidamente pulizia/sensibilità dei due Ingressi e Guadagno. In Pausa
   cambiare valori e riprendere. Non devono sparire Frasi/Parziali già pubblicati.
3. Confrontare ascolto dei tratti precedenti/successivi, durata, Forma d'onda,
   ultima parola a Stop e metadati di pulizia del Tape riaperto. Non è stato
   ripetuto il banco audio del 06: nessun DSP/callback/coda audio modificato.
4. Ripetere più sessioni e riavviare l'app: scelte persistenti, Spento conserva
   Silero e non spegne DFN3. Verificare guasto su un solo Ingresso: preferenza
   salvata conservata, bypass locale visibile anche chiudendo il banner,
   metadati coerenti. Gli eventi vecchi non devono alterare la nuova barra.

Corpus umano, ascolto, hardware/ore di Registrazione, analisi finale reale
dei Parlanti, licenza pesi e installer restano distinti e aperti come negli
handoff precedenti. Nessun nuovo risultato percettivo/native dichiarato.
