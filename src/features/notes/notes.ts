import type { Nota } from "@/bindings";

/** La Nota mostrata: il nome e il testo su disco, `null` se la Nota non c'è ancora. */
export interface NotaAperta {
  nome: string;
  salvato: string | null;
}

/** Il salvataggio di una bozza: `atteso` è il testo su disco da cui parte (ADR-0032). */
export interface Salvataggio {
  atteso: string;
  nome: string;
  testo: string;
}

/**
 * La Nota da mostrare: la prima del Tape o, in un Tape senza Note, una nuova con il nome
 * `predefinito`, che nasce con il primo carattere.
 */
export function notaAperta(note: Nota[], predefinito: string): NotaAperta {
  const [prima] = note;
  return prima
    ? { nome: prima.nome, salvato: prima.testo }
    : { nome: predefinito, salvato: null };
}

/**
 * Cosa salvare della `bozza` della Nota `nome`, il cui testo su disco è `salvato` (`null` se non
 * c'è): niente se coincide, e una Nota nuova ancora vuota non si scrive.
 */
export function salvataggio(
  nome: string,
  salvato: string | null,
  bozza: string
): Salvataggio | null {
  const atteso = salvato ?? "";
  return bozza === atteso ? null : { atteso, nome, testo: bozza };
}
