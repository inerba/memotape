# Verifica del ticket 06

6 ottobre 2026, checkout `D:\local\tauri\sbobino`, conversazione
`01a11206-a2b7-7d73-9d2c-5056119f19c4`. Esito locale **partial**:
implementazione e controlli conclusi; corpus umano, ascolto e UI aperti.

## Delta e controlli ordinari

`baseline/` conserva lo stato precedente al 06. `delta.patch` e
`changed-files.txt` isolano i file del ticket rispetto a quella baseline,
preservando i predecessori e le modifiche concorrenti. Gli artefatti di questa
cartella e l'handoff sono separati dal delta del codice. Il diff da HEAD non
è il delta del 06. `snapshot-delta.py` rigenera questo confronto.
Il controllo finale dei file copiati in baseline non rileva modifiche fuori
dai file assegnati al ticket. `status-after.txt` conserva lo stato Git finale.

| Controllo | Esito | Log |
| --- | --- | --- |
| Typecheck | passato | typecheck.log |
| Frontend | 119 passati, 0 falliti, 4.659 assertion | frontend-test.log |
| Lint frontend | 119 file, nessuna correzione | frontend-check.log |
| Formattazione Rust | passata | fmt.log |
| Clippy, tutti i target, `-D warnings` | passato | clippy.log |
| Rust | 296 passati, 0 falliti, 29 ignorati | rust-tests.log |
| Binding generati | test generatore passato, nessun delta nel binding | bindings.log |

TDD sulle seam concordate: scarto prima di ASR/Parziali, ammissione irrevocabile
durante parlato, coda arretrata, cambi in Pausa/Stop, Ingressi/sessioni,
revisione prima del ricampionatore 44,1→48 kHz e bypass DFN3 con PCM pendente.
Il caso limite del livello Spento nell'ultimo frame di una candidata da 18 s
ha fallito prima della correzione ed è verde nella suite finale. La verifica
osserva anche PCM e durata, senza dedurre il risultato dal solo stato interno.

## Corpus sintetico e ASR

Corpus riusato dal 05: sei WAV annotati, hash e provenienza in
`../verification-05/corpus/manifest.json`. Silenzio, rumore continuo,
intermittente, soffio sintetico, TTS attenuata di 30 dB e «sì/no» TTS.
I rumori sono non parlato; le due fixture TTS sono regressioni designate.

`prefix.log`: DFN3 e Silero reali, controllo dell'ammissione sui prefissi
e del PCM esatto inviato al motore finto; passato, 36,59 s.

`matrix.json` e `native-matrix.log`: 288 inferenze reali sequenziali sul
percorso streaming LiveFeed/LiveFrames. Sono 3 ASR × 6 fixture × 2 stati
pulizia × 4 livelli × 2 ripetizioni etichettate Microfono/Sistema. Le due
etichette usano lo stesso PCM con un solo feed per esecuzione: non sono
catture native distinte né dimostrano isolamento simultaneo. Le 144
configurazioni distinte sono misurate due volte, non replicate nel JSON.
Il banco WASAPI e i test deterministici verificano separatamente due Ingressi.
DFN3 prepara il PCM prima della coda sintetica; il banco WASAPI verifica anche
il denoiser durante la cattura. La matrice è passata in 116,74 s, compilazione
esclusa. Nessuna nuova calibrazione DFN3.

Somme di pulizia accesa/spenta, **per ogni ripetizione di profilo**:

| ASR | False Frasi Spento / Più sensibile / Bilanciato / Più selettivo | Parole perse rispetto a Spento nei due livelli prudenti / selettivo |
| --- | --- | --- |
| Whisper | 2 / 2 / 0 / 0 | 0 / 14 |
| Nemotron | 0 / 0 / 0 / 0 | 0 / 14 |
| Parakeet | 0 / 0 / 0 / 0 | 0 / 14 |

Gli avvii ASR sui rumori passano da 2 a 0 con Bilanciato per modello/profilo.
Il testo delle regressioni in Più sensibile e Bilanciato è esattamente quello
di Spento. Parakeet conserva «C No.», già errato nella baseline. Il conteggio
delle parole perse è relativo alla baseline ASR, non una misura WER rispetto
alla verità umana. Più selettivo perde 7 parole per stato pulizia sulla voce
attenuata: rischio esplicito, nessuna promessa universale.

Nemotron sulla voce attenuata conserva i 6 Parziali della baseline anche nei
livelli prudenti; il primo cade a 1.260 ms e prima della fine della Frase a
1.290 ms. I test deterministici attestano separatamente un Parziale mentre
il parlato continua. «Sì/no» non produce Parziali nemmeno in Spento: il banco
non impone capacità inesistenti su parole così brevi. La prima esecuzione ha
fallito questa asserzione troppo forte del banco; log conservato in
`native-matrix-initial.log`, asserzione corretta rispetto alla baseline.

## Carico WASAPI

`native-load-first.log` conserva la prima misura passata. La review ha
richiesto entrambe le code e il contratto Settings → Recorder. Il banco
aggiornato usa uno SettingsStore temporaneo e `Recorder::set_audio` sui due
profili durante cattura e Pausa, verificando il congelamento dopo Stop.
Non modifica lo store o la Libreria dell'utente. `native-load.log` è la misura
finale; hardware in `hardware.json`. Cattura di 28 s, Pausa 12–13 s, due DFN3,
due Nemotron, protezione, mix/tracce Ogg e Tape riaperto.

Finale passata, **31,59 s** di prova, compilazione esclusa. Audio **27.030 ms**,
tempo worker mixer/DSP/controlli **9,4942934 s**, rapporto **0,3512502 s/s**.
Ryzen 7 3700X (8 core/16 thread), 64 GB RAM, RTX 2070 SUPER, Windows 11 Pro
10.0.26200. Microfono KLIM Talk e Focusrite loopback, 48 kHz stereo.
Massimo campionato code **[660, 660] ms** al t=12; entrambe 0 al t=13 e
alla fine. Pool cattura massimo campionato **[5, 1]**, frame persi **[0, 0]**.
Microfono: 0 Frasi; Sistema: 4 Frasi, 12 Parziali. Nessun accumulo crescente
nel tratto osservato. Tape uguale al documento riaperto, Forma d'onda uguale,
mix e tracce decodificati con durata entro 1 ms.
RAM finale working/private/peak-working **134.234.112 / 397.643.776 /
405.663.744 byte**. Log grezzo conserva serie temporale, testo e percorso Tape.
`summarize-native.py` estrae questi valori in `native-load-summary.json`
e verifica denominatore della matrice, perdite e code finali.

Le code sono campionate ogni secondo, nell'ordine Microfono/Sistema.
`processing_s` misura mixer/ricampionamento/DSP, controlli (incluse scritture
Settings temporanee ai cambi) e scarico finale: ASR e writer
sono concorrenti ma i loro tempi non sono inclusi in questa somma. I modelli
sono caricati e riscaldati prima della cattura. La RAM è letta dopo la fine
dei worker; il picco è quello di processo registrato da Windows, senza VRAM.
La prova breve non attesta ore di Registrazione, latenza UI o altri PC.

## Review e limiti

Review indipendenti Standards/Spec e risoluzione dei rilievi in `review.md`.
Mancano respiri e voce debole umani, annotazioni/ascolto, catture WASAPI con
tutti e tre gli ASR, player visibile e cambi manuali da Impostazioni con
tastiera/Narrator. Il banco finale usa Nemotron; la matrice copre i tre
motori sul PCM sintetico. Diarizzazione finale non collaudata in questa prova,
percorso di produzione conservato dopo Stop e smaltimento ASR. Licenza dei
pesi e bundle restano aperti dal 02. Nessun commit, PR, installer o rilascio.

Comandi e contratti per il ticket 07 in `../handoff-06.md`.
