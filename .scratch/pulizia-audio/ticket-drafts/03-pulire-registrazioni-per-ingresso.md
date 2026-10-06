# 03: Pulire le Registrazioni separatamente per Ingresso

**What to build:** l'utente attiva DeepFilterNet3 per Microfono e/o Audio di sistema da Impostazioni e registra, anche senza Trascrivi dal vivo, conservando audio ripulito e Ingressi sincronizzati. Può cambiare questi valori in Impostazioni durante la Registrazione.

**Blocked by:** 02 — Pulire un file con DeepFilterNet3 e salvare un Tape coerente.

**Status:** needs-triage

Bozza in attesa dell'approvazione della suddivisione. Requisiti: [spec approvata](../spec.md).

- [ ] Microfono e Audio di sistema hanno controlli persistenti indipendenti in Impostazioni, inizialmente spenti, accessibili e tradotti nelle sei lingue. File e audio misto mantiene il suo profilo separato.
- [ ] Un'istanza con stato indipendente elabora ciascun Ingresso nel worker, dopo il Guadagno e prima di somma e diramazione Ogg/ASR. Attivare un Ingresso non altera il trattamento dell'altro; stereo e frequenza di Registrazione restano corretti.
- [ ] Registrazione solo Microfono, solo Audio di sistema ed Entrambi funzionano con e senza Trascrivi dal vivo. Tracce, mix, Forma d'onda, player e ASR derivano dall'audio effettivamente acquisito ed elaborato; non viene conservata una copia originale aggiuntiva.
- [ ] Ritardo del filtro e percorso bypass sono compensati senza spostare gli Ingressi o i tempi delle Frasi. Test includono frequenze diverse, mono/stereo, loopback fermo, buchi e impulsi/parole ai confini, usando le tolleranze temporali già richieste dall'app.
- [ ] Impostazioni aggiornate a cavallo dell'avvio non si perdono. Durante cattura il cambio vale sull'audio successivo prima delle code ASR; in Pausa vale dalla ripresa; dopo Stop non modifica il Tape acquisito.
- [ ] Accensione e spegnimento mantengono continuità e transizioni senza scatti evidenti. Pausa e Stop scaricano la coda utile senza tagliare le ultime parole; lo scarico viene verificato con il runtime reale, non dedotto dall'invio di zeri. Sessioni successive non ereditano lo stato audio precedente.
- [ ] I metadati del Tape descrivono gli intervalli realmente trattati per Ingresso, compresi cambi al volo e bypass. I tratti acquisiti con pulizia spenta restano non trattati.
- [ ] Un guasto durante la Registrazione conserva i campioni acquisiti e prosegue in bypass sul solo Ingresso interessato, mantenendo la linea del tempo. Un avviso accessibile e tradotto identifica Ingresso e sessione; eventi di sessioni precedenti vengono ignorati.
- [ ] Nessuna inferenza o attesa nella callback WASAPI, nessuna nuova coda illimitata o perdita di buffer causata dallo stadio. La Diarizzazione resta dopo Stop e completamento ASR, sugli Ogg salvati.
- [ ] Una prova nativa in release usa due Ingressi, DFN3 attivo, ASR e salvataggio, alternando parlato e silenzio. Riporta hardware, durata, tempo per secondo di audio, memoria, code e perdite; il tratto stabile tiene il passo senza accumulo crescente o disallineamenti. Gli esiti non vengono generalizzati ad altri PC.
- [ ] Test di orchestrazione verificano audio comune, ciclo di vita, guasti, persistenza e sessioni. Collaudo nativo verifica cambi da Impostazioni durante cattura/Pausa e riapertura del Tape; eventuali verifiche mancanti sono esplicite.
- [ ] Contratti rigenerati, requisiti e decisioni aggiornati. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; una sola build Rust alla volta, smoke ASR reali sequenziali. Nessun commit o rilascio autonomo.

## Comments

I controlli nella barra della Registrazione sono nel ticket 07; qui i cambi al volo sono già operativi attraverso Impostazioni.
