# 02: Pulizia RNNoise nelle Registrazioni

**What to build:** Una Registrazione con la pulizia accesa usa RNNoise 0.2: per Ingresso, accesa e spenta al volo anche in Pausa, con l'audio originale lasciato passare dopo un guasto, e con la partenza che non aspetta il modello. Spec: `../spec.md`, sezione "Il processore".

**Blocked by:** 01

**Status:** ready-for-agent

- [x] La preparazione della Registrazione e la protezione dal vivo costruiscono l'adattatore RNNoise. Se il caricamento dei pesi è istantaneo (da misurare e riportare nei commenti), la preparazione in anticipo e il thread dedicato si semplificano, mantenendo il comportamento: la Registrazione parte subito e un modello assente si segnala come oggi.
- [x] L'adattatore è `Send` senza thread dedicato: lo stato C di RNNoise non è legato al thread (motivo scritto accanto all'`unsafe impl`).
- [x] Stati separati per Microfono e Audio di sistema; accensione e spegnimento valgono dall'audio successivo, in Pausa dalla ripresa; un guasto lascia passare l'originale senza fermare la Registrazione.
- [x] I test della pulizia nella Registrazione (due Ingressi, cambio al volo, Pausa, guasto) e della protezione dal vivo nativa passano con RNNoise senza cambiare cosa verificano.
- [x] Una Registrazione di prova con la pulizia accesa in `bun tauri dev` funziona senza perdere blocchi WASAPI; se servono ottimizzazioni del profilo dev per RNNoise, si aggiungono.
- [x] I sei controlli passano.

## Note dell'implementazione (18 ottobre 2026)

- **Caricamento**: RNNoise si carica in circa 2 ms (lettura dei pesi 0,6–1,2 ms, `RNNoise::new` 1,3–2,2 ms, in debug e in release). Via la preparazione in anticipo: `RecordingFilters`, `PreparedFilter`, il thread dedicato, la fase `cleaning`, l'evento `recording-cleaning-preparing` e le due stringhe nelle sei lingue. Il processore nasce sul worker al primo blocco con la pulizia accesa; un modello assente arriva lì con `recording-cleaning-failed`, non più prima di aprire WASAPI. ADR-0026 è marcato «superato in parte», `PRODUCT.md` aggiornato.
- **DFN3**: tolto il modulo Rust e i suoi test, perché senza le Registrazioni era codice morto e `clippy -D warnings` lo rifiuta. Restano al ticket 03 le dipendenze Cargo, il modello, le licenze, i testi e la documentazione.
- **Test**:
  - `rnnoise_registrazione_coda_pause_e_stato_indipendente` gira ora in ogni `cargo test`. L'impulso finale si verifica con `> 0.05` invece del valore esatto di DFN3.
  - La protezione dal vivo con tre ASR è verde. Per la voce a −30 dB pulita chiede che Silero apra una Frase invece dei Parziali, come deciso per il corpus prudente: la matrice verde storica era di DFN3 miscelato (`pcm-snr-v1`).
  - Il modello assente ha un test sul percorso reale (caricamento al primo blocco).
- **Prove con l'audio vero** (18 ottobre 2026):
  - L'utente ha registrato in `bun tauri dev`, con la pulizia accesa e cambi al volo: il Tape si salva e il rumore cala molto; con un rumore molto forte la voce si degrada un po', com'è prevedibile.
  - Test WASAPI in release, microfono KLIM Talk e altoparlanti Focusrite, 48 kHz stereo:
    - `rnnoise_due_ingressi_nativi_release`: 27 s, 0 frame persi su entrambi gli Ingressi, worker al 4,5% della durata, 4 Frasi.
    - `protezione_due_ingressi_nativi_release`: 0 frame persi, 4 Frasi.
    - `cattura_nativa_non_diventa_silenzio_nel_mixer` con la pulizia: verde.
  - Nel Tape i tratti `rnnoise` seguono i cambi: Microfono fino a 7,02 s, poi dalla ripresa a 12,02 s; Sistema spezzato alla Pausa.
  - Nessuna ottimizzazione del profilo dev da aggiungere.
