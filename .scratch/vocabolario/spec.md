# Vocabolario

Status: resolved

Data: 8 ottobre 2026. Decisioni concordate in una sessione di grilling; requisito in `PRODUCT.md` (sezione Vocabolario, storie 81–83). Termini in `CONTEXT.md`: **Vocabolario**, **Termine**.

## Problem Statement

I modelli scrivono male nomi propri, marchi e parole tecniche ("Charge B" per ChargeBee, "Nicolo" per Niccolò). L'utente corregge a mano sempre gli stessi errori, Tape dopo Tape.

## Solution

Un Vocabolario globale di Termini, gestito in Impostazioni → Trascrizione, che ogni Trascrizione successiva usa in due modi, come fa Handy (`cjpais/Handy`, `src-tauri/src/audio_toolkit/text.rs` e `managers/transcription.rs`):

1. **Prompt di Whisper**: con Whisper i Termini, uniti da `", "`, entrano in `WhisperRunOptions::initial_prompt`.
2. **Correzione approssimativa**: con ogni modello, Whisper compreso, il testo di ogni Frase conclusa passa per una funzione pura che sostituisce i tratti simili a un Termine.

Il runtime fissato (`transcribe.cpp` `e6672a8`) non offre hotword o word boosting per Nemotron e Parakeet: solo Whisper ha `Feature::InitialPrompt`.

## Decisioni

### Dominio

- Il Vocabolario è unico per tutta l'app: niente Vocabolario per Raccolta né per Lingua del parlato.
- Si applica a file, Trascrivi su un Tape e Frasi concluse dal vivo; mai ai Parziali.
- Non tocca i Tape già trascritti; per applicarlo si usa Trascrivi di nuovo. Nessun comando "applica al Tape".
- Una Trascrizione usa il Vocabolario com'era al suo avvio, come modello e Lingua del parlato.
- La sostituzione è testo della Trascrizione, non una Correzione manuale: niente **Corretto a mano**, il Tape non conserva la parola originale né quale Vocabolario è stato usato.
- I nomi dei Parlanti e il Nome predefinito del Microfono non entrano da soli nel Vocabolario.
- Solo Termini da riconoscere: niente coppie esplicite "sbagliato → giusto".
- Il menu Trascrivi ▾ non mostra il Vocabolario.

### Correzione approssimativa

Algoritmo di Handy (`apply_custom_words`), con gli adattamenti indicati:

- Per ogni posizione della Frase si provano n-gram di 3, 2 e 1 parole e vince quello con il punteggio migliore. Un n-gram non attraversa la punteggiatura (una parola interna con punteggiatura finale lo chiude) né il confine della Frase.
- Chiave di confronto: lettere e cifre, minuscole, **senza accenti** (diverso da Handy, che scarta tutto ciò che non è ASCII), parole unite senza spazi. Le chiavi con caratteri di scritture non latine non si confrontano: quei Termini agiscono solo nel prompt di Whisper.
- Si scarta un candidato con differenza di lunghezza oltre il 25% (almeno 2 caratteri ammessi).
- Punteggio: distanza di Levenshtein / lunghezza massima; ×0,3 se le chiavi solo alfabetiche hanno lo stesso Soundex. Si accetta sotto la soglia fissa **0,18**, non configurabile.
- **Un Termine con chiave di meno di 4 caratteri si sostituisce solo se la chiave coincide** (adattamento: le parole italiane brevi darebbero falsi positivi).
- Il sostituto prende maiuscole come la prima parola originale (tutta maiuscola → tutto maiuscolo; iniziale maiuscola → iniziale maiuscola; altrimenti il Termine com'è) e la punteggiatura in testa alla prima e in coda all'ultima parola.
- La correzione viene dopo il filtro della scrittura estranea (`foreign_script`).

### Tempi del testo

- Quando n parole diventano un Termine, il suo Tempo del testo va dall'inizio della prima alla fine dell'ultima parola sostituita, ed è indivisibile per la Diarizzazione. Testo e tempi devono restare allineati, altrimenti si perdono le divisioni ai cambi di Parlante. Con Whisper i tempi sono per segmento: la sostituzione resta dentro il segmento.

### Whisper

- Prompt = Termini nell'ordine della lista, uniti da `", "`, nelle stesse `WhisperRunOptions` di `confident_only()`.
- `transcribe.cpp` tronca il prompt oltre circa 223 token **togliendo l'inizio**: restano gli ultimi Termini. Nessun limite alla lista; una nota fissa sotto la lista lo dice.
- Un `<|…|>` nel prompt fa fallire la Frase con `INVALID_ARG`: per questo un Termine con `<|` o `|>` non si accetta.

### Impostazioni e interfaccia

- Impostazione nuova con `#[serde(default)]` (lista vuota), letta come le altre facoltative.
- Impostazioni → Trascrizione: campo di testo con **Aggiungi**; Invio aggiunge. Incollare testo con più righe aggiunge un Termine per riga.
- Normalizzazione all'aggiunta: spazi in testa e in coda tolti, righe vuote ignorate.
- Errori all'aggiunta: un doppione (senza distinguere maiuscole e accenti) o un Termine con `<|`/`|>` non entra; un messaggio accanto al campo lo dice e il testo resta. Incollando più righe, le valide entrano e il messaggio dice quante sono state scartate. Rust valida di nuovo in `set_settings` (confine di fiducia).
- I Termini sono pillole che vanno a capo, nell'ordine di aggiunta (ultimo in fondo), ognuna con una × dal nome accessibile "Rimuovi <Termine>". Nessuna modifica sul posto: si rimuove e si aggiunge di nuovo.
- Ogni aggiunta e rimozione si salva subito con `set_settings`, senza Annulla.
- Senza Termini, una riga spiega a cosa serve il Vocabolario.
- Testi nelle sei lingue.

## Fuori dal perimetro

- Vocabolario per Raccolta, coppie di sostituzione esplicite, soglia configurabile.
- Applicare il Vocabolario a Tape esistenti, anteprima delle sostituzioni.
- Hotword o boosting nel decoder di Nemotron e Parakeet (non c'è nel runtime).
- Ricordare nel Tape la parola originale o il Vocabolario usato.

## Verifica

- Test della funzione pura: n-gram ("Charge B" → "ChargeBee"), punteggiatura, maiuscole, accenti ("Nicolo" → "Niccolò"), Termine breve solo esatto, scrittura non latina ignorata, tempi uniti e allineati.
- Test delle impostazioni: predefinito vuoto, file di prima senza il campo, rifiuto di `<|`.
- Prova a mano con Whisper e con Nemotron sulla fixture `parlato-it.wav` con un Termine pronunciato, e su audio reale per i falsi positivi.
