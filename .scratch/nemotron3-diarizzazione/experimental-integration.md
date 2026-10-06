# Integrazione sperimentale Nemotron 3 — 6 ottobre 2026

L’utente ha autorizzato «integrala allora» dopo la proposta di rendere disponibile Nemotron come opzione sperimentale, mantenendo Sortformer predefinito. L’implementazione dei ticket 01–07 era già collegata ai file, ai Tape e alle Registrazioni nel worktree 5e50. Questa modifica completa la presentazione nell’app; non cambia motori, impostazioni salvate o contratti backend.

## Esperienza nell’app

Impostazioni → Trascrizione → Modello dei parlanti permette di scegliere Nemotron 3 Diarization (sperimentale) e il modello BF16 locale compatibile. Il modello ASR rimane una scelta distinta. Attivando Riconosci i parlanti sui file/Tape si usa il modello selezionato; nelle Registrazioni si attivano Trascrivi dal vivo e le caselle dei Parlanti degli Ingressi desiderati.

Le Impostazioni mostrano possibili confusioni tra voci e provvisorietà delle attribuzioni dal vivo. Per il dal vivo è consigliata una GPU Vulkan; il testo chiarisce che con la sola CPU la Diarizzazione può interrompersi mentre audio e Trascrizione continuano, con analisi finale dopo Stop. Nessun blocco CPU o cambio automatico di modello.

Il menu Trascrivi mostra una nota Nemotron visibile, collegata alla casella tramite aria-describedby: supporto fino a otto Parlanti per Ingresso con possibili confusioni o voci unite. La nota non promette il riconoscimento certo di otto persone. Traduzioni aggiornate in it/en/fr/es/de/pl. La descrizione del modello locale ora include le Registrazioni con Ingressi separati.

## Verifiche

- Typecheck, 106 test frontend, Biome 160 file, cargo fmt, clippy e 235 test Rust tutti verdi; 12 Rust ignored. Log integration-{typecheck,frontend,biome,fmt,clippy,rust}.log. I modelli nativi già provati nel ticket 08 non sono stati nuovamente caricati per questo cambiamento dell’interfaccia.
- Build frontend tsc/Vite riuscita: integration-build.log. Nessun installer creato.
- Binding esportati da tauri-specta attraverso il test nativo, verificati identici e salvati in src/bindings.ts. Nessun cambiamento di comandi, eventi o tipi Rust.
- Due revisori GPT-6.1 Sol: Spec 0 finding; Standards ha segnalato la nota inizialmente solo title, corretta con descrizione visibile/accessibile e poi ricontrollata: 0 finding residue. Revisione statica, nessuna attestazione UI nativa.
- Baseline dei file modificati e manifest degli altri file preservati in 0acd/.scratch/nemotron3-diarizzazione/baselines/experimental-integration. Tutti i file preesistenti fuori dall’integrazione sono byte-identici al manifest. Diff limitato a questa modifica: experimental-integration.diff.

## Limiti conservati

Il ticket 08 resta ready-for-human: corpus costruito con annotazioni energetiche, non gold manuale; UI Windows, dispositivi reali e Registrazioni lunghe da collaudare. Il realtime CPU sul PC della prova non ha superato il test; le misure Vulkan callback non certificano ritardo della vista o altri PC. Il campione da otto identità produce sette etichette. Report08 e addendum restano le fonti delle misure.

Questa integrazione è completa come scelta sperimentale nel codice del worktree 5e50. Non aggiorna l’app già installata. Nessun commit, cambio branch, push, distribuzione di pesi o pubblicazione. L’automazione della sequenza 06–08 resta in pausa, senza dichiarare il ticket 08 done.
