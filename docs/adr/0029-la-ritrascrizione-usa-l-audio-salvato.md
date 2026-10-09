---
status: accepted
---

# La ritrascrizione usa l'audio salvato; le scelte di Trascrivi sono sempre in vista

Data: 9 ottobre 2026. Sostituisce la parte di ADR-0022 sulla pulizia dei Tape e
la parte di ADR-0023 sulla protezione dei Tape.

## Contesto

Con ADR-0022 Trascrivi su un Tape applicava la pulizia dei profili attivi
all'audio salvato: decodificava ogni Ingresso, lo passava a DeepFilterNet3,
scriveva PCM temporanei, ricodificava in Opus, ricostruiva il mix e sostituiva
l'audio del Tape. ADR-0023 applicava anche la Sensibilità del parlato.

L'audio di un Tape è già quello scelto quando il Tape è nato: una Registrazione
è stata pulita (o no) mentre si registrava, un file importato con le scelte
dell'importazione. Ripulirlo degrada di nuovo il campione, e l'utente non
sceglie nessuno di questi parametri quando preme Trascrivi di nuovo. La
preparazione costava inoltre minuti prima della prima Frase su un'ora di audio.

Le scelte di un'importazione stavano in Impostazioni → Trascrizione e nel menu
della freccia di Trascrivi: si poteva avviare senza sapere se il rumore sarebbe
stato filtrato.

## Decisione

- **Trascrivi su un Tape**, la prima volta o di nuovo, trascrive gli Ogg salvati
  così come sono: niente pulizia, Sensibilità spenta, qualunque sia
  l'impostazione. Cambia solo il documento (`tape::replace_transcription`):
  audio, Forma d'onda, `pulizia_audio` e voci sconosciute restano byte per byte,
  i campi JSON sconosciuti si conservano. Annulla o errore lasciano il Tape
  com'era. Vocabolario e nome predefinito del Microfono continuano a valere.
- **Trascrivi apre sempre un dialog** (`TranscribeDialog`) con le scelte in
  vista, salvate subito come predefinite:
  - un file da importare: modello, Lingua del parlato, Filtra rumore,
    Sensibilità e Riconosci i parlanti (profilo `audio_file_misto`);
  - un Tape: modello, Lingua del parlato e Riconosci i parlanti; con testo già
    presente il dialog è anche la conferma "Ritrascrivere il Tape?".
- Il blocco "File e audio misto" esce da Impostazioni → Trascrizione: si sceglie
  solo nel dialog.

Sono stati rimossi `TapeAudio` (preparazione, ricodifica, ricostruzione del
mix), il riuso dei tratti già puliti (`ConfiguredCleaning::reuse`) e il
contesto di sola lettura di DFN3 (`AudioProcessor::context`), che servivano
solo a ripulire un Tape. Restano nella storia git.

## Conseguenze

- Ritrascrivere un Tape non cambia mai il suo audio; per un audio diverso si
  importa di nuovo il file d'origine.
- Un Tape pulito in passato da una ritrascrizione resta com'è.
- La Sensibilità vale per i file importati e per la Trascrizione dal vivo.
