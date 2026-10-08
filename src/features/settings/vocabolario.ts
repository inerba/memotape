/** L'esito di un'aggiunta al Vocabolario: la lista nuova e le righe che non sono entrate. */
export interface Aggiunta {
  aggiunti: number;
  doppioni: string[];
  nonAmmessi: string[];
  termini: string[];
}

/** Il messaggio accanto al campo; con `keep` il testo resta nel campo, perché nulla è entrato. */
export type AddMessage = { keep: boolean } & (
  | {
      key: "settings.vocabolario.doppione" | "settings.vocabolario.nonAmmesso";
      params: { termine: string };
    }
  | { key: "settings.vocabolario.scartati"; params: { count: number } }
);

const RIGHE = /\r?\n/;
const ACCENTI = /\p{M}/gu;

/** La chiave dei doppioni: senza maiuscole né accenti. */
function chiave(termine: string): string {
  return termine.normalize("NFD").replace(ACCENTI, "").toLowerCase();
}

/** Aggiunge in fondo a `termini` un Termine per riga di `testo`. */
export function addTermini(termini: string[], testo: string): Aggiunta {
  const esito: Aggiunta = {
    aggiunti: 0,
    doppioni: [],
    nonAmmessi: [],
    termini: [...termini],
  };
  const presenti = new Set(termini.map(chiave));
  for (const riga of testo.split(RIGHE)) {
    const termine = riga.trim();
    if (!termine) {
      continue;
    }
    const k = chiave(termine);
    // Nel prompt di Whisper un token speciale fa fallire la Frase: lo rifiuta anche Rust.
    if (termine.includes("<|") || termine.includes("|>")) {
      esito.nonAmmessi.push(termine);
    } else if (presenti.has(k)) {
      esito.doppioni.push(termine);
    } else {
      presenti.add(k);
      esito.termini.push(termine);
      esito.aggiunti += 1;
    }
  }
  return esito;
}

/** Cosa dire degli scarti di `esito`: il Termine se è l'unica riga, altrimenti quanti. */
export function addMessage(esito: Aggiunta): AddMessage | null {
  const { aggiunti, doppioni, nonAmmessi } = esito;
  const keep = aggiunti === 0;
  const scartati = doppioni.length + nonAmmessi.length;
  if (scartati === 0) {
    return null;
  }
  if (scartati > 1 || aggiunti > 0) {
    return {
      keep,
      key: "settings.vocabolario.scartati",
      params: { count: scartati },
    };
  }
  const [doppione] = doppioni;
  if (doppione !== undefined) {
    return {
      keep,
      key: "settings.vocabolario.doppione",
      params: { termine: doppione },
    };
  }
  return {
    keep,
    key: "settings.vocabolario.nonAmmesso",
    params: { termine: nonAmmessi[0] ?? "" },
  };
}

/** `termini` senza `termine`: la × di una pillola. */
export function removeTermine(termini: string[], termine: string): string[] {
  return termini.filter((t) => t !== termine);
}
