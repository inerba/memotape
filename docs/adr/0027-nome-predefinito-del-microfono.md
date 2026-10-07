# ADR-0027 — Il nome predefinito del Microfono si scrive nel Tape alla creazione

Data: 7 ottobre 2026. Accettata e implementata (`Document::set_nome_microfono`, `Settings::nome_microfono_per`).

## Contesto

Con gli Ingressi separati il Microfono non diarizzato è una persona sola, che
l'utente rinomina Tape per Tape (chiave `microfono` in `parlanti`). Quasi sempre
è lui: vuole impostare il nome una volta, in Impostazioni → Generale.

## Decisione

Il nome predefinito si **copia nel Tape quando il Tape nasce**: Registrazione
da Entrambi e ritrascrizione per Ingresso, se il Microfono non è diarizzato.
Diventa la voce `parlanti["microfono"]`, uguale a una rinomina manuale, ed è
già usato dal testo dal vivo. Vuoto, non si scrive nulla.

## Alternativa scartata

Usarlo come ripiego in lettura per ogni Tape senza nome. Avrebbe cambiato anche
i Tape passati a ogni modifica dell'impostazione, e avrebbe richiesto lo stesso
ripiego in render Rust, Copia testo, Markdown, indice di ricerca e server MCP,
che oggi leggono solo il Tape.

## Conseguenze

- Il Tape resta un documento stabile: cambiare il nome predefinito vale solo
  per i Tape successivi; i vecchi si rinominano a mano.
- Ricerca, Markdown e Assistenti vedono il nome senza modifiche, perché è un
  nome come gli altri.
- Ritrascrivere un Tape toglie i nomi (comportamento attuale) e rimette quello
  predefinito; la nuova Diarizzazione conserva il nome del Microfono (ADR-0017).
- Il nome del Microfono persona sola non rende il Tape "corretto a mano" e non fa chiedere
  conferma prima di una nuova Diarizzazione: non è il nome di un Parlante e la Diarizzazione lo
  conserva comunque.
- Il mix (sorgente solo Microfono) non riceve nomi: le sue Frasi non hanno
  etichetta e non si inventa un Parlante.
