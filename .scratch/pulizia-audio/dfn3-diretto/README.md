# DFN3 diretto, revisione del 6 ottobre 2026

Richiesta dell'utente dopo ascolto del confronto sul campione reale: eliminare
la miscela aggiuntiva e usare l'uscita DFN3 diretta. Non sono aggiunti altri
filtri, guadagni automatici o soglie. I controlli di Sensibilità restano distinti.

La modifica rimuove curva triangolare SNR, limite aggiuntivo a 6 dB, smoothing,
crossfade e trattenimento dei 100 ms destinati al crossfade. Mantiene ritardo
compensato, formato, durata, contesto, scarico e recupero del percorso condiviso.
La versione dei nuovi intervalli nel Tape è `direct-v1`. Gli intervalli vecchi
continuano a essere riusati senza doppia pulizia. L'originale non si recupera
da un Tape precedentemente elaborato: per ascoltare la modifica serve una nuova
Registrazione oppure importare di nuovo il file esterno originale.

## Prove della revisione

- Regressione sul ticchettio tramite `DeepFilter`, osservata rossa prima della
  modifica: scarto massimo 0,34624627 rispetto a DFN3 diretto. Dopo la modifica
  passa; il ticchettio sintetico è soppresso come nell'uscita diretta.
- Formati 8/16/24/44,1/48 kHz e 1/2/6 canali: durata esatta, valori finiti e
  canale silenzioso indipendente dopo Pausa, Configurazione e Finish.
- Corpus breve «Sì. No.»: 35 200 frame prima/dopo; round-trip 16/48/16 kHz
  senza spostamento temporale. Questa prova non misura il riconoscimento ASR.
- Percorso spettrale equivalente a libDF originale su audio non silenzioso.
- Fixture TTS più rumore noto: errore rispetto al parlato pulito ridotto di
  4,8464193 dB. È una misura su questa fixture sintetica, non sul ristorante.
- Fixture TTS attenuata di 30 dB: onset Silero originale a 480 ms, nessun onset
  dopo DFN3 diretto. Durata sempre 143 360 frame. La conservazione dell'onset
  tramite miscela è un requisito precedente esplicitamente sostituito.
  Anche il test sul canale debolissimo confronta ora l'uscita con il modello
  diretto: non impone un livello residuo artificiale reinserendo l'originale.

Il confronto sul campione reale è passato: 22,097625 s, mono 48 kHz, scarto
massimo f32 pari a **0** rispetto a DFN3 diretto. Attenuazione dell'energia
complessiva 4,83469916544449 dB per entrambe le uscite. Scarto rispetto al PCM16
decodificato 0,00003743171691894531, entro la sola tolleranza di quantizzazione.
Il WAV esportato dall'app ha lo stesso SHA-256 del WAV ascoltato dall'utente:
è identico byte per byte, compresi inizio e fine. Prova debug 89,94 s, esclusa
compilazione. La prova aggiornata di silenzio/coda/canale debole è passata
separatamente in 6,41 s; le altre cinque prove native erano già passate.

Il riferimento ascoltato rimane in `../diagnosi-rumore/`, senza sovrascrittura.
SHA-256 del riferimento PCM16 e di `02-app-dfn3-diretto.wav`:
`8b19eddca457b19f2e7699d6855c47e6ff1dcf50e83fd6beb34bdca9392037b4`.
SHA-256 dell'MP3 originale:
`61971c86f6b04df644c54bc4813aac6927f7a1ff8003129ab9d6330bbe739994`.

## Riproduzione del confronto reale

PowerShell dalla root del checkout; non modifica Impostazioni o Libreria:

```powershell
$env:CARGO_TARGET_DIR = 'D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust'
$env:MEMOTAPE_DFN3_SAMPLE = 'D:\local\tauri\sbobino\campioni\restaraunt_noisy.mp3'
$env:MEMOTAPE_DFN3_REFERENCE = 'D:\local\tauri\sbobino\.scratch\pulizia-audio\diagnosi-rumore\03-dfn3-diretto.wav'
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --locked --manifest-path src-tauri/Cargo.toml dfn3_campione_reale_confronta_pcm_e_modello -- --ignored --test-threads=1 --nocapture
```

Genera `01-originale.wav`, `02-app-dfn3-diretto.wav`, `03-dfn3-diretto.wav` e
`confronto.json` in questa cartella. Nessuna normalizzazione del volume. Confronta
PCM f32 fra processore pubblico `PcmStream`/`DeepFilter` e uscita diretta; confronta
anche tutto il WAV scelto prima della modifica, con tolleranza limitata alla
quantizzazione PCM16. Le misure di energia non attestano SNR o parole conservate.

## Controlli finali

Tutti e sei i controlli richiesti passano: typecheck, 126 test frontend,
check Biome (122 file), Cargo fmt, Clippy all-targets con `-D warnings`,
296 test Rust ordinari (31 ignorati, non dichiarati eseguiti).
I sette test DFN3 reali descritti sopra sono stati eseguiti separatamente,
in sequenza, con il criterio silenzio/coda aggiornato verificato dopo la prima
esecuzione. Log ordinari in `cargo-test.log` e `clippy.log`.

Revisione locale della modifica rispetto alle copie `baseline/`: nessun
cambiamento a Silero, soglie della protezione, code ASR, mixer, Ogg o percorsi
di riuso/recupero. Il limite di consegna è il conteggio dei frame originari;
il ricampionamento può arrotondare la coda, che viene limitata alla durata reale.
Il reset mantiene il piano Tract e lo stato indipendente per canale; il prefisso
di contesto continua a essere scartato senza duplicare audio. Nessun problema
residuo individuato nella revisione, entro queste prove.

Non sono stati eseguiti un nuovo collaudo umano delle Registrazioni a due
Ingressi, una nuova matrice con tre ASR o un installer. Le prove di qualità
delle precedenti matrici `pcm-snr-v1` non validano `direct-v1`.
Nessun commit o rilascio.
