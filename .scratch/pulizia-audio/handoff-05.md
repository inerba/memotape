# Handoff del ticket 05

Conversazione `01a111e6-0b38-7063-8cb0-49dc32b7f352`. Implementazione e review
locali concluse, stato **partial** per corpus umano, ascolto e collaudo Tauri.
Nessun commit, PR, installer o pubblicazione. Ticket 06/07 non avviati.
Preservare tutte le modifiche precedenti e concorrenti: il diff da HEAD non
è il delta del 05. Baseline, `delta.patch` e `changed-files.txt` sono in
`verification-05/`; questo handoff e gli artefatti di verifica sono separati
dal delta del codice/prodotto.

## Contratti da conservare

- `ProfiloAudio::sensibilita` è indipendente da pulizia e Guadagno. Quattro
  valori, con Bilanciato come default serde anche per impostazioni precedenti.
  Microfono, Sistema e File/audio misto sono persistenti e indipendenti.
- `ProtectionTimeline::capture` avvolge l'`AudioProcessor` esistente. Legge il
  livello all'ingresso del blocco decodificato, prima del ritardo PCM, e salva
  solo i cambi in coordinate assolute di campioni a 16 kHz.
- Import usa File/misto. Il Tape preparato dal 04 conserva una timeline per
  Ingresso, riutilizzata nella successiva ASR senza rileggere le Impostazioni.
  Un Tape senza tracce separate usa File/misto anche se la provenienza della
  pulizia ricorda Microfono/Sistema. Il riuso DFN3 del 04 resta invariato.
- Solo `cut_phrases` dei file/Tape applica la protezione. Il percorso dal vivo
  non cambia nel 05. `FileAudio::from(None)` conserva Spento per i chiamanti
  interni precedenti; la produzione passa la timeline del profilo effettivo.
- `PhraseEvidence` combina probabilità Silero e ZCR, senza RMS assoluta,
  durata minima rigida, AGC, blacklist o confidenza ASR inventata. Segmentation
  Silero e confini precedenti conservati. Parametri in ADR-0023.
- Il frame da 30 ms usa la revisione al proprio inizio. Basta un gruppo ammesso
  per conservare la candidata intera: nessun taglio della parola ai cambi.
  Le candidate già accodate non vengono reinterpretate dalle impostazioni nuove.
- Gli scarti riguardano solo l'invio all'ASR: audio, copia Ogg, PCM per i
  Parlanti, durata e Forma d'onda conservano tutti i campioni. La riscrittura
  atomica, Annulla e gli errori del 04 restano il contratto di salvataggio.
- Un Tape con unica falsa Frase può diventare privo di testo. Salvataggio,
  riapertura anche con `noSpeech`, indice, copia e ricerca eliminano il vecchio
  testo; Ogg e Forma d'onda restano coerenti. Test dedicato passato.
- Impostazioni usa select nativi etichettati, spiegazioni associate e sei
  traduzioni. Dichiarata l'applicazione solo a file/Tape. Binding rigenerati;
  PRODUCT, CONTEXT, AGENTS e ADR-0023 aggiornati.

## Verifiche osservate

Sei controlli verdi: typecheck, **119 test frontend**, lint frontend, fmt
backend, clippy su tutti i target con `-D warnings`, **290 test Rust**. Nel
controllo ordinario 26 smoke ignorati. Binding rigenerati nella cwd `src-tauri`.
Review indipendenti Standards/Spec: nessun bug certo o violazione vincolante,
tre osservazioni stilistiche non bloccanti. Vedi `verification-05/review.md`.

Matrice nativa release separata passata, 53,60 s di prova, compilation esclusa:
Whisper, Nemotron e Parakeet caricati in sequenza, DFN3 acceso/spento,
quattro livelli. Sei fixture sintetiche annotate con hash. Sono 144 misure
di configurazione, esplicitamente replicate in 432 righe per tre profili con
PCM equivalente; non sono catture indipendenti da ogni dispositivo.

Sommando pulizia on/off una volta per profilo equivalente, Bilanciato riduce
gli avvii ASR non parlati da 2 a 0 per modello e le false Frasi Whisper da
2 a 0. Nemotron/Parakeet già davano testo vuoto su quel rumore. Bilanciato e
Più sensibile conservano le candidate della voce TTS attenuata e delle risposte
brevi. Parakeet conserva «C No.», già nella baseline. Più selettivo perde
14 parole della voce attenuata per modello fra i due stati pulizia: rischio
dichiarato, nessuna promessa di protezione universale. Non usa il testo per
la decisione. README, matrice e script riportano denominatori e parole attese.

## Comandi riproducibili

Una sola compilazione Rust per volta. Non cambiare `.cargo/config.toml`, usare
il target isolato e non fermare l'app o altri processi dell'utente.

```powershell
$env:CARGO_TARGET_DIR='D:\local\tauri\sbobino\.scratch\pulizia-audio\target-rust'
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --locked --manifest-path src-tauri/Cargo.toml
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo --config "env.LOCALAPPDATA.value='D:\local\tauri\sbobino-deps'" --config env.LOCALAPPDATA.relative=false test --release --locked --manifest-path src-tauri/Cargo.toml corpus_protezione_quattro_livelli_tre_asr -- --ignored --test-threads=1 --nocapture
```

Suite completa fuori sandbox per il Cestino. Lo smoke usa modelli locali già
presenti senza cambiare Impostazioni o Libreria. Non ripetere la calibrazione
02 né lo smoke appena passato senza una nuova modifica o dubbio concreto.
Per i binding: cwd `src-tauri`, stesse opzioni Cargo e
`test --locked rigenera_bindings_di_sviluppo -- --ignored --test-threads=1`.
Bun: `C:\Program Files\nodejs\node_modules\bun\bin\bun.exe`; comandi ordinari
`run typecheck`, `run test`, `run check`, `run format:backend` (fmt puro).

## Limiti e passaggio al 06

Mancano respiri, voce bassa e risposte brevi umane, registrazioni distinte per
dispositivo, annotazioni/ascolto e collaudo Tauri con tastiera/Narrator e cambi
manuali durante Trascrivi. Soffio sintetico e TTS attenuata non soddisfano
questi criteri. Non attestati altri PC o disco pieno reale. Licenza dei pesi
e bundle restano aperti dal 02, senza nuova validazione nel 05.

Il 06 dovrà estendere la protezione alla cattura live prima delle code ASR,
rispettando timeline, conservazione dell'audio e parola in corso. Il caso
Microfono silenzioso/Sistema parlato del 05 è un test core deterministico,
non la cattura reale a due Ingressi richiesta dal 06. Conservare la distinzione
fra test di selezione, ASR, audio e UI; non trasformare questo `partial` in
`done` sulla sola base della matrice sintetica.
