# Handoff del ticket 07

Conversazione `01a1122c-7f30-7ac2-99b3-8a90f0710a37`, 6 ottobre 2026.
Implementazione locale completata; ticket **partial** per collaudo Tauri,
tastiera/Narrator e ascolto. Nessun commit, PR, installer o rilascio.
Preservate le modifiche precedenti; baseline e delta in `verification-07`.

## Delta

- `recording-panel.tsx`: pulizia on/off e quattro sensibilità per Ingresso,
  distinti da Guadagno; descrittore unico `AUDIO_INPUTS`, nessun File/misto.
  Bypass effettivo per Ingresso separato dalla scelta persistente.
- `cleaning-profile.tsx`: variante compatta condivisa con Impostazioni,
  controlli nativi, id unici, descrizioni e stato disabled al 50% dopo Stop.
- `settings-writer.ts`, `settings-context.tsx`: aggiornamenti funzionali
  serializzati, scelte pendenti proiettate subito, rollback e rebase dopo
  errore, risultato normalizzato del backend riusato per il salvataggio seguente.
  `flush` attende anche nuovi aggiornamenti giunti durante l'attesa.
- Guadagno, caselle, Sorgente, menu Trascrivi, Raccolta e SettingsPage passano
  funzioni di aggiornamento, evitando snapshot obsoleti. Il rollback del form
  segue il provider, senza ripristinare una vecchia cattura di `settings`.
- Home attende `flush` prima di Registra, impedisce clic duplicati e passa
  i guasti già ascoltati alla barra. Chiudere il banner nasconde solo il banner;
  il bypass locale rimane. Nuove sessioni eliminano i guasti precedenti.
- Sei traduzioni; PRODUCT, AGENTS e ADR-0025 aggiornati. CONTEXT invariato:
  nessun nuovo termine. Backend/binding invariati, nessuna dipendenza nuova.

Lista file completa: `verification-07/changed-files.txt`; diff:
`verification-07/delta.patch` rispetto alla baseline, non HEAD.
Non includere in una futura review tutte le modifiche del checkout come se
fossero di questo ticket. La baseline contiene già i ticket precedenti.

## Prove

Sei controlli verdi: typecheck, **126 frontend** (7 nuovi, 4729 assertion,
20 file), lint frontend, fmt, clippy all-targets con warnings negati,
**296 Rust** e 29 ignored. Binding verificati dalla suite Rust.
TDD sulle seam richieste: concorrenza, rollback e attesa dell'avvio; rendering
reale dei componenti per tre Sorgenti, default e bypass/eventi obsoleti.
Le prove sullo storage frontend simulano IPC: non attestano il riavvio nativo.
Review Spec/Standards ed eventuali correzioni: `verification-07/review.md`.

Stessi runtime/override del 06; config Cargo non modificata. Una sola build
Rust alla volta; suite completa fuori sandbox per Cestino Windows. Errori
di scrittura su file mappati (1224) risolti eseguendo le stesse operazioni
fuori sandbox. Bun assoluto in `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`.
Detector meccanico UI senza rilievi; non è un controllo visuale/nativo.

## Limiti

App installate dell'utente aperte e senza CDP; nessun controllo nativo CUA
disponibile. Chrome CUA indisponibile e IAB non raggiunge localhost/127.0.0.1.
Vite del banco su 1427 fermato dopo la prova, nessuna app dell'utente chiusa.
`verification-07/ui.html` e `ui.tsx` sono un banco con IPC finto, conservato
**non eseguito**. README contiene il collaudo manuale preciso.

Ancora da provare: cambi dalla barra durante parlato/silenzio/Pausa con due
Ingressi, DFN3/ASR attivi, ascolto prima/dopo, ultime parole a Stop, durata,
metadati, Tape riaperto, più sessioni, riavvio reale, tastiera/Narrator e layout
nelle sei lingue/temi. Il core audio del 06 è riusato senza modifiche: non si
sono ripetute matrice/carico/cattura. I risultati del 06 restano del suo banco,
non della nuova UI. Corpus umano, altro hardware/ore, Diarizzazione finale
reale, licenza pesi e prova installer restano aperti, distinti da questo delta.
