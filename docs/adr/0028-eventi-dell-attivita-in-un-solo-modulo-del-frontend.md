# ADR-0028 — Gli eventi dell'Attività passano da un solo modulo del frontend

Data: 8 ottobre 2026. Accettata, da implementare (`.scratch/attivita-frontend/issues/02-modulo-dell-attivita-nel-frontend.md` e `03-status-timer-e-pulizia-nel-modulo.md`).

## Contesto

`home.tsx` registra 13 listener e decide quali eventi accettare con tre token di sessione (`recordingSession`, `pendingRecording.sessionId`, `Conversation.sessionId`) e un flag (`acceptPartials`). Un evento tardivo viene rifiutato solo perché `loadTape` ricostruisce la Conversation e ne toglie il `sessionId` come effetto collaterale; nel fallback Ogg lo stesso evento viene accettato perché quel setter non gira. Nessuna di queste regole è testata: i test coprono i reducer di `phrases.ts` e `status.ts`, non come HomePage li combina. La stessa Conversation è anche il documento della Sorgente che si corregge, e `updateView` può applicare una correzione del Tape consultato al testo di una Registrazione in corso.

## Decisione

Un modulo puro, `src/features/activity/`, riceve gli eventi dell'Attività in corso e sei azioni dai comandi (avvio, esito, ripristino, pausa, Sorgente aperta, Sorgente modificata) e restituisce Status, Conversation, sessione, timer e guasti di pulizia. Un hook sottile registra i listener, fa dispatch e offre `ready`, la promessa che `record` aspetta.

La sessione ha stati espliciti: nessuna → aperta → in chiusura → chiusa. In chiusura si accetta solo lo snapshot finale di quella sessione (il fallback Ogg); chiusa scarta tutto. Trascrivi e Riconosci i parlanti aprono una sessione senza id: lo stato basta a scartare i loro eventi tardivi, senza cambiare i comandi Rust. Le modifiche alla Sorgente valgono solo senza sessione aperta o in chiusura.

## Alternative scartate

- **Solo il testo nel modulo**: lo Status resterebbe in HomePage con gli stessi eventi, quindi due proprietari dello stesso stato.
- **Anche il ciclo dei comandi** (`restore`, `previousStatus`, Preparazione): è il candidato "comandi dell'Attività" della revisione architetturale, da decidere a parte.
- **Un UUID per ogni Attività**: richiede di cambiare `transcribe` e `diarize`; lo stato esplicito della sessione ottiene lo stesso effetto. Da riconsiderare solo se un test mostra che non basta.
- **Un hook che contiene la logica**: senza test sui componenti resterebbe non verificato.

## Conseguenze

- Le regole di accettazione degli eventi si testano in `activity.test.ts` attraverso l'interfaccia del modulo; i test di sessione di `phrases.test.ts` si spostano lì, `event-session.ts` e `cleaning.ts` spariscono.
- `withPhrase`, `withPartial` e `withParlanti` restano trasformazioni pure della Conversation, senza controllo di sessione.
- `RecordingPanel` riceve liste già filtrate e conserva il proprio listener dei livelli, per non far passare i 40 ms da HomePage.
- Presuppone la revisione dell'8 ottobre di ADR-0016: resta solo lo snapshot finale di `LiveTranscriptUpdated`.
