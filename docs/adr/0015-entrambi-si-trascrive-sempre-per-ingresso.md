---
status: accepted
---

# Una Registrazione da Entrambi si trascrive sempre per Ingresso

Con la Trascrizione dal vivo sul mix, la Diarizzazione girava sul mix: Sortformer conosce solo le voci, non l'Ingresso da cui arrivano. Inoltre il VAD taglia le Frasi sul mix, quindi una Frase poteva contenere la voce del Microfono e quella dell'Audio di sistema insieme. Il risultato erano Parlanti che mescolavano le due parti di una videochiamata (diagnosi del 2026-10-05 sui Tape dell'utente). Per questo una Registrazione da Entrambi si trascrive sempre a Ingressi separati, in ogni caso:
- dal vivo, con due istanze del modello (ADR-0004). Il selettore Mix / Ingressi separati sparisce;
- senza Trascrizione dal vivo, perché il Tape contiene sempre `microfono.ogg` e `sistema.ogg`;
- con Trascrivi su un Tape che ha l'audio degli Ingressi: si trascrive ogni Ingresso, non il mix;
- con una Continuazione da Entrambi su un Tape senza l'audio degli Ingressi: le tracce si creano con silenzio per la parte vecchia, e le Frasi vecchie restano `mix` (corregge ADR-0013, che le accodava solo "se ci sono").

"Riconosci i parlanti" diarizza l'Audio di sistema; il Microfono è una persona e si diarizza solo con la sua casella, spenta di default. I Tape da Entrambi di prima, con solo il mix, si ritrascrivono come prima: non c'è modo di separarli, e il documento non dice nemmeno da quali Ingressi venivano.

Il costo: RAM/VRAM doppie dal vivo, circa 1 s in più all'avvio, e Tape con tre flussi Opus invece di uno. Per contenerlo i predefiniti dell'audio scendono a 16 kbps, mono, 16 kHz, che bastano per la voce; le impostazioni già salvate restano come sono.

## Considered Options

- **Tenere il Mix e disattivare la Diarizzazione lì**: un'opzione che non porta niente in più degli Ingressi separati.
- **Restare sul mix e attribuire ogni Frase all'Ingresso con più energia**: le Frasi che contengono le due voci restano sbagliate.
- **Un'istanza sola che alterna i due Ingressi dal vivo**: con Nemotron, che è in streaming, le Frasi arriverebbero tardi.
- **Salvare solo le tracce e ricavarne il mix**: tocca player, Forma d'onda, Continuazione e Trascrivi per pochi MB l'ora.

Fuori perimetro: l'eco degli altoparlanti nel microfono (senza cuffie la voce remota entra anche nel Microfono). Non c'è cancellazione dell'eco.
