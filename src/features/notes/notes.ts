import type { AppError, Nota } from "@/bindings";

/** Una Nota della riga dei nomi: il nome e il testo su disco, `null` se la Nota non c'è ancora. */
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

/** Il nome più lungo di una Nota, in caratteri, come `tape::NOME_NOTA_MAX`. */
const NOME_MAX = 60;

/**
 * Le Note della riga dei nomi, in ordine di creazione. In un Tape senza Note una sola, nuova, con
 * il nome `predefinito`: nasce con il primo carattere.
 */
export function noteIniziali(note: Nota[], predefinito: string): NotaAperta[] {
  return note.length > 0
    ? note.map((n) => ({ nome: n.nome, salvato: n.testo }))
    : [{ nome: predefinito, salvato: null }];
}

/**
 * Il nome di una Nota nuova: `nome(n)` (per esempio «Nota 3») con `n` il numero delle Note più uno,
 * o il primo seguente non usato.
 */
export function nomeNuovo(nomi: string[], nome: (n: number) => string): string {
  let n = nomi.length + 1;
  while (nomi.includes(nome(n))) {
    n += 1;
  }
  return nome(n);
}

/**
 * L'errore del backend per il nome `nome`, se lo rifiuterebbe: vuoto, oltre 60 caratteri o tra
 * `altre`, i nomi delle altre Note del Tape. `null` se va bene.
 */
export function nomeRifiutato(nome: string, altre: string[]): AppError | null {
  if (nome.trim() === "" || [...nome].length > NOME_MAX) {
    return { code: "noteNameInvalid", detail: nome };
  }
  return altre.includes(nome) ? { code: "noteNameTaken", detail: nome } : null;
}

/**
 * Le Note senza `nome` e la Nota da aprire: la seguente, o la precedente se era l'ultima. Senza più
 * Note torna la Nota nuova con il nome `predefinito`.
 */
export function senzaNota(
  note: NotaAperta[],
  nome: string,
  predefinito: string
): { aperta: string; note: NotaAperta[] } {
  const i = note.findIndex((n) => n.nome === nome);
  const resto = note.filter((n) => n.nome !== nome);
  if (resto.length === 0) {
    return { aperta: predefinito, note: noteIniziali([], predefinito) };
  }
  return { aperta: resto[Math.min(i, resto.length - 1)].nome, note: resto };
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
