# Review del ticket 05

6 ottobre 2026. Review indipendenti Standards e Spec sul delta dalla baseline
del ticket, non sull'intero diff da HEAD. Include il controllo richiesto del
Tape che, dopo la protezione, non ha più alcuna Frase. Esito: nessun bug certo
o violazione vincolante residui; ticket **partial** per validazione incompleta.

## Standards

Il core della protezione resta indipendente da Tauri. Usa le seam esistenti
`VoiceDetector`, `TranscriptionEngine` e `AudioProcessor`; non introduce un
secondo contratto DSP. La configurazione è acquisita prima dei buffer e della
coda ASR. Il delta conserva i contratti dei ticket precedenti. I binding sono
rigenerati dal test di sviluppo, non modificati a mano.

Tre osservazioni stilistiche non bloccanti:

1. Le ricerche della timeline per Ingresso nei due percorsi di Trascrizione
   potrebbero essere raccolte in una funzione se il numero di percorsi cresce.
2. Le tre definizioni dello schema dei profili in `settings.ts` ripetono la
   stessa forma; un singolo schema condiviso potrebbe ridurre la duplicazione.
3. `CleaningProfile` ora presenta anche la sensibilità. Il nome potrebbe
   diventare più generale quando si riorganizza la UI dei profili.

Nessuna richiede una modifica per soddisfare il ticket; non è stato esteso il
refactoring alle parti estranee al comportamento richiesto.

## Spec

Nessun bug certo o ampliamento di scope individuati. Livelli/default,
persistenza, traduzioni, separazione da DFN3, evidenza prima delle code,
conservazione del PCM e comportamento delle candidate sono coperti da test.
La matrice sintetica con tre ASR soddisfa il criterio Bilanciato sulle
regressioni designate; i risultati sono limitati a quel corpus.

Due gruppi di criteri restano incompleti:

- Corpus umano annotato: respiri, voce bassa e risposte brevi umane, campioni
  distinti registrati per dispositivo e ascolto. TTS e soffio sintetico sono
  surrogati dichiarati, non prove di questi criteri.
- Collaudo nella finestra Tauri: interazioni, tastiera/Narrator, cambi manuali
  durante Trascrivi e ascolto delle transizioni. Semantica HTML e test core
  non attestano l'esperienza della finestra nativa.

Il caso di `noSpeech` su un Tape con testo precedente è stato controllato:
`TapeAudio::commit` salva anche il documento vuoto; la riscrittura elimina
i vecchi campi del testo prima di fondere quelli sconosciuti; Home riapre il
Tape esistente anche con `noSpeech`; il comando sincronizza la Libreria per
ogni esito. Il test aggiunto parte da un'unica falsa Frase e verifica documento,
riapertura, copia, ricerca, audio byte-identico e Forma d'onda. La criticità
ipotizzata non è confermata e non richiede una modifica del frontend.

## Verifica finale

| Controllo | Esito | Evidenza |
| --- | --- | --- |
| Typecheck | passato | `typecheck.log` |
| Test frontend | 119 passati, 0 falliti | `frontend-test.log` |
| Lint frontend | passato | `frontend-check.log` |
| Formattazione backend | passata | `fmt.log` |
| Clippy, tutti i target, `-D warnings` | passato | `clippy.log` |
| Test Rust | 290 passati, 0 falliti, 26 ignorati | `rust-tests.log` |
| Rigenerazione binding | passata | `bindings.log` |
| Matrice nativa release, separata | passata, 53,60 s di prova | `native-matrix.log` |

La suite Rust finale include la regressione `noSpeech`; eseguita fuori sandbox
per il test del Cestino. Lo smoke nativo carica gli ASR in sequenza e non è
un collaudo UI. Denominatori, riuso delle misure e limiti in `README.md`.
