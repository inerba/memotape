const MARK_START = "\u0001";
const MARK_END = "\u0002";

/** Un pezzo dell'estratto di una Frase trovata: `mark` se è una parola trovata. */
export interface MarkedPart {
  mark: boolean;
  text: string;
}

/**
 * Le parti dell'estratto della ricerca: le parole trovate stanno tra `\u0001` e `\u0002`
 * (`MARK_START` e `MARK_END` in `library.rs`).
 */
export function markedParts(estratto: string): MarkedPart[] {
  return estratto
    .split(MARK_END)
    .join(MARK_START)
    .split(MARK_START)
    .map((text, i) => ({ mark: i % 2 === 1, text }))
    .filter((part) => part.text !== "");
}
