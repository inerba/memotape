# 03: Pulire le Registrazioni separatamente per Ingresso

**What to build:** l'utente attiva DeepFilterNet3 per Microfono e/o Audio di sistema da Impostazioni e registra, anche senza Trascrivi dal vivo, conservando audio ripulito e Ingressi sincronizzati. Può cambiare questi valori in Impostazioni durante la Registrazione.

**Blocked by:** 02 — Pulire un file con DeepFilterNet3 e salvare un Tape coerente.

**Status:** partial — implementazione e review concluse, sei controlli verdi e prove native locali passate; collaudo manuale Tauri/ascolto ancora da eseguire.

Suddivisione approvata con la richiesta di implementazione del 6 ottobre 2026. Requisiti: [spec approvata](../spec.md).

- [x] Microfono e Audio di sistema hanno controlli persistenti indipendenti in Impostazioni, inizialmente spenti, accessibili e tradotti nelle sei lingue. File e audio misto mantiene il suo profilo separato.
- [x] Un'istanza con stato indipendente elabora ciascun Ingresso nel worker, dopo il Guadagno e prima di somma e diramazione Ogg/ASR. Attivare un Ingresso non altera il trattamento dell'altro; stereo e frequenza di Registrazione restano corretti.
- [ ] Registrazione solo Microfono, solo Audio di sistema ed Entrambi funzionano con e senza Trascrivi dal vivo. Tracce, mix, Forma d'onda, player e ASR derivano dall'audio effettivamente acquisito ed elaborato; non viene conservata una copia originale aggiuntiva.
- [x] Ritardo del filtro e percorso bypass sono compensati senza spostare gli Ingressi o i tempi delle Frasi. Test includono frequenze diverse, mono/stereo, loopback fermo, buchi e impulsi/parole ai confini, usando le tolleranze temporali già richieste dall'app.
- [x] Impostazioni aggiornate a cavallo dell'avvio non si perdono. Durante cattura il cambio vale sull'audio successivo prima delle code ASR; in Pausa vale dalla ripresa; dopo Stop non modifica il Tape acquisito.
- [ ] Accensione e spegnimento mantengono continuità e transizioni senza scatti evidenti. Pausa e Stop scaricano la coda utile senza tagliare le ultime parole; lo scarico viene verificato con il runtime reale, non dedotto dall'invio di zeri. Sessioni successive non ereditano lo stato audio precedente.
- [x] I metadati del Tape descrivono gli intervalli realmente trattati per Ingresso, compresi cambi al volo e bypass. I tratti acquisiti con pulizia spenta restano non trattati.
- [x] Un guasto durante la Registrazione conserva i campioni acquisiti e prosegue in bypass sul solo Ingresso interessato, mantenendo la linea del tempo. Un avviso accessibile e tradotto identifica Ingresso e sessione; eventi di sessioni precedenti vengono ignorati.
- [x] Nessuna inferenza o attesa nella callback WASAPI, nessuna nuova coda illimitata o perdita di buffer causata dallo stadio. La Diarizzazione resta dopo Stop e completamento ASR, sugli Ogg salvati.
- [x] Una prova nativa in release usa due Ingressi, DFN3 attivo, ASR e salvataggio, alternando parlato e silenzio. Riporta hardware, durata, tempo per secondo di audio, memoria, code e perdite; il tratto stabile tiene il passo senza accumulo crescente o disallineamenti. Gli esiti non vengono generalizzati ad altri PC.
- [ ] Test di orchestrazione verificano audio comune, ciclo di vita, guasti, persistenza e sessioni. Collaudo nativo verifica cambi da Impostazioni durante cattura/Pausa e riapertura del Tape; eventuali verifiche mancanti sono esplicite.
- [x] Contratti rigenerati, requisiti e decisioni aggiornati. Typecheck, test frontend, lint frontend, formattazione backend, clippy e test Rust verdi; una sola build Rust alla volta, smoke ASR reali sequenziali. Nessun commit o rilascio autonomo.

## Comments

I controlli nella barra della Registrazione sono nel ticket 07; qui i cambi al volo sono già operativi attraverso Impostazioni.

## Evidenze del 6 ottobre 2026

Delta isolato rispetto all'inizio del ticket: [patch](../verification-03/delta.patch)
e [file modificati](../verification-03/changed-files.txt). Le modifiche precedenti
dei ticket 01/02 e quelle estranee sono preservate. [Handoff per il 04](../handoff-03.md).

I sei controlli finali hanno exit code 0: `typecheck`, `test` frontend (118 passati,
0 falliti), `check` (119 file), `format:backend`, clippy `--all-targets -D warnings`
e suite Rust completa (278 passati, 0 falliti, 23 ignorati; doc-test verdi).
La suite Rust è stata eseguita fuori sandbox per il Cestino Windows, nel target
isolato. Gli smoke ignorati nuovi sono stati eseguiti separatamente in release.
I contratti sono rigenerati dal builder tauri-specta, senza modifiche manuali.
Log completi e provenienza in [verification-03](../verification-03/README.md).

TDD alla seam approvata: guasto con output parziale/coda e recupero PCM,
guasto nello scarico, indisponibilità del modello a controllo spento,
persistenza/avvio/Pausa/Stop, indipendenza degli Ingressi e metadati. Il test
del percorso comune controlla Ogg, ASR, Forma d'onda e Tape riaperto. Due
regressioni riprodotte e corrette: durata con cambi rapidi (4300 invece di 4267
frame a 48→16 kHz), fase del segnale dopo reset (scarto 0,03547 anche lontano
dal confine). Il ricampionamento ora riusa contesto e griglia razionale.

DFN3 reale: a 48 kHz stereo, 44,1 kHz stereo e 24 kHz mono, tre segmenti da
19.213 frame con Pausa, Configurazione e Fine mantengono lunghezza esatta e
impulso finale, stato vergine bit-identico fra segmenti e metadati continui.
Lo stereo conserva il canale silenzioso. Non è una nuova taratura del ticket 02.

Prova nativa finale `dfn3_due_ingressi_nativi_release`: Ryzen 7 3700X (8 core,
16 thread), RTX 2070 SUPER, 64 GiB RAM; Microfono KLIM Talk e uscita Focusrite,
entrambi 48 kHz stereo. Due runtime DFN3 e due istanze Nemotron Streaming 3.5
0.6B, fixture `parlato-it.wav` riprodotta due volte con silenzio intermedio.
Microfono spento a 7 s e riattivato durante Pausa 12–13 s. Cattura di 28 s,
audio salvato 27.062 ms, 4 Frasi; mix e due tracce riaperti con formato/durata
coerenti entro 1 ms, documento e Forma d'onda riletti dal Tape.

Tempo wall del mixer/DSP 14,061 s, rapporto medio 0,5196 s/s di audio: misura
dello scenario con cambi, distinta dal tempo ASR e dalla scrittura. Perdite
di buffer: `[0,0]` frame. Pool osservato ogni secondo: massimo `[2,2]` buffer,
non un picco continuo. Coda ASR aggregata osservata: massimo 2190 ms a 4 s,
zero a 6 s, variazioni transitorie fino a 330 ms in seguito e zero dopo Stop.
Non cresce nel tratto stabile della prova. Memoria di processo dopo ASR:
working set 175.251.456 byte, privata 391.024.640 byte, picco working set
496.390.144 byte (circa 473,4 MiB); VRAM non misurata. L'esito vale per questo
PC e questa acquisizione breve, non per tutti gli hardware o carichi.

## Review

**Standards:** corretti il focus da tastiera dei checkbox e i nuovi hunk di
Mixer/ricampionamento; revisione finale senza finding residui azionabili.

**Spec:** corretti osservazione del controllo durante lo scarico, durata
cumulativa e perdita di fase. Verificate anche le transizioni DFN3 a frequenze
diverse da 48 kHz. Nessun difetto certo residuo; i limiti del collaudo restano
distinti dalla correttezza del codice e motivano lo stato `partial`.

## Verifiche ancora aperte

Non è stato eseguito il collaudo manuale nella finestra Tauri: matrice dei
tre tipi di Sorgente con/senza Trascrivi dal vivo, clic in Impostazioni durante
cattura/Pausa/Stop, annuncio effettivo degli avvisi e player visibile dopo
riapertura. Il core e l'orchestrazione sono coperti da test, ma non sostituiscono
questa verifica. Mancano ascolto delle transizioni e parole reali esattamente
sui confini, sessioni lunghe e prove su altri PC. Per questo i criteri 3, 6 e 11
restano aperti. Bundle/licenza dei pesi e calibrazione DFN3 del ticket 02 non
sono stati riaperti. Nessun commit, PR, rilascio o avvio del ticket 04.
