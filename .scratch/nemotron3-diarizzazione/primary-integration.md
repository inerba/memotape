# Integrazione nel progetto principale — 6 ottobre 2026

La funzione era implementata nel worktree 5e50 ma non nel progetto principale. Dopo la segnalazione «non trovo queste modifiche», trasferiti tutti i file della funzione in **D:\local\tauri\sbobino**, mantenendo il branch main e senza creare commit.

Trasferiti 93 file (43 tracked modificati, 50 nuovi fra sorgenti, documenti, ticket e strumenti). I due checkout partivano dallo stesso HEAD e il progetto principale non aveva modifiche tracked locali. Le copie precedenti sono conservate in C:\Users\inerba\.codex\worktrees\0acd\sbobino\.scratch\nemotron3-diarizzazione\primary-integration\before; manifest con hash e patch nella cartella superiore. La cartella locale .claude e la configurazione .cargo non sono state modificate. Tutti i file trasferiti sono stati verificati identici alla sorgente anche dopo i controlli.

## Verifiche nel progetto principale

Sei controlli verdi: typecheck; 106 test frontend; Biome; cargo fmt; clippy; 235 test Rust, 12 ignored. Build frontend riuscita; 14 test Python degli strumenti riusciti. Smoke nativo Nemotron con modello ASR e GGUF locali: Trascrizione di un file, creazione e riapertura Tape con Parlanti riuscite. Log primary-*.log nella stessa cartella. Nessuna compilazione Rust concorrente.

Runtime ricompilato dal commit fissato `e6672a8672913b47f1571c66c54bee789d028416`; binding esportati dal test tauri-specta, verificati identici e salvati. Il GGUF rimane locale; nessun peso, corpus audio o archivio copiato o distribuito. Le evidenze complete del benchmark e gli audio di prova restano nel worktree 5e50.

## Come vedere la funzione

Aprire il progetto principale D:\local\tauri\sbobino e avviare `bun tauri dev` da quella cartella. In Impostazioni → Trascrizione → Modello dei parlanti scegliere **Nemotron 3 Diarization (sperimentale)**, quindi selezionare il GGUF locale compatibile. Il modello locale già verificato sulla macchina è D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf. Riconosci i parlanti sui file usa la scelta; per le Registrazioni attivare Trascrivi dal vivo e i Parlanti degli Ingressi desiderati.

La copia installata di Memotape non è stata aggiornata. Nessun installer, commit, push, cambio branch o pubblicazione. Il ticket 08 rimane ready-for-human: non sono certificati UI nativa, conversazioni gold manuali, altri PC e Registrazioni lunghe; il limite realtime CPU resta documentato.
