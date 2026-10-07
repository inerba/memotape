# Verifiche del ticket 03

Esiti locali del 6 ottobre 2026. Il delta di codice, contratti, requisiti e ADR
è confrontato con la baseline iniziale in questa cartella, non con HEAD.
Le modifiche di stato al ticket 03, al piano e l'handoff sono documenti separati.

| Controllo finale | Esito | Log |
| --- | --- | --- |
| Typecheck | exit 0 | `typecheck.log` |
| Test frontend | 118 passati, 0 falliti | `frontend-test.log` |
| Lint frontend | 119 file, exit 0 | `frontend-check.log` |
| Formattazione Rust | exit 0 | `backend-format.log` |
| Clippy tutti i target, warning vietati | exit 0 | `backend-clippy.log` |
| Suite Rust + doc-test | 278 passati, 0 falliti, 23 ignorati | `backend-test.log` |
| Generatore bindings | exit 0 | `bindings-generation.log` |
| DFN3 release, pause/cambi/code, 48/44,1/24 kHz | passato | `native-runtime-release.log` |
| Due Ingressi WASAPI, DFN3, ASR, Tape release finale | passato | `native-capture-release.log` |

`duration-red.log` e `phase-red.log` conservano i casi riprodotti prima delle
correzioni; `audio-green.log` documenta il gruppo audio dopo le correzioni
(49 passati, 5 ignorati). `native-capture-release-initial.log` è la prima
misura, precedente al fix del ricampionamento; la misura conclusiva è quella
senza suffisso. Hardware in `hardware.json`. Il test nativo finale riapre il
Tape temporaneo indicato nel log e ne verifica documento, Forma d'onda,
mix e tracce. Non scrive nella Libreria dell'utente né modifica Impostazioni.

Le misure e i limiti sono descritti nel [ticket 03](../issues/03-pulire-registrazioni-per-ingresso.md).
La registrazione nativa dura 28 s e usa parlato sintetico; non attesta la
matrice UI Tauri, ascolto delle transizioni, parlato umano ai confini, sessioni
lunghe, bundle/licenza o hardware diversi. L'intero ticket resta `partial`.

I comandi riproducibili sono nel [handoff](../handoff-03.md). Una sola build
Rust alla volta, target `.scratch/pulizia-audio/target-rust`, nessuna modifica
a `.cargo/config.toml`. La suite completa con Cestino è stata lanciata fuori
sandbox. Non sono stati eseguiti commit, PR, build installer o pubblicazioni.
