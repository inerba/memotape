# 02: Pulizia RNNoise nelle Registrazioni

**What to build:** Una Registrazione con la pulizia accesa usa RNNoise 0.2: per Ingresso, accesa e spenta al volo anche in Pausa, con l'audio originale lasciato passare dopo un guasto, e con la partenza che non aspetta il modello. Spec: `../spec.md`, sezione "Il processore".

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] La preparazione della Registrazione e la protezione dal vivo costruiscono l'adattatore RNNoise. Se il caricamento dei pesi è istantaneo (da misurare e riportare nei commenti), la preparazione in anticipo e il thread dedicato si semplificano, mantenendo il comportamento: la Registrazione parte subito e un modello assente si segnala come oggi.
- [ ] L'adattatore è `Send` senza thread dedicato: lo stato C di RNNoise non è legato al thread (motivo scritto accanto all'`unsafe impl`).
- [ ] Stati separati per Microfono e Audio di sistema; accensione e spegnimento valgono dall'audio successivo, in Pausa dalla ripresa; un guasto lascia passare l'originale senza fermare la Registrazione.
- [ ] I test della pulizia nella Registrazione (due Ingressi, cambio al volo, Pausa, guasto) e della protezione dal vivo nativa passano con RNNoise senza cambiare cosa verificano.
- [ ] Una Registrazione di prova con la pulizia accesa in `bun tauri dev` funziona senza perdere blocchi WASAPI; se servono ottimizzazioni del profilo dev per RNNoise, si aggiungono.
- [ ] I sei controlli passano.
