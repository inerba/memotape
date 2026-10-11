import { elapsedText } from "@/features/recording/recording";
import type { Taglio } from "@/features/taglio/taglio";

/**
 * I Tempi nella Nota: `[m:ss]`, `[mm:ss]` o `[h:mm:ss]` fuori dal codice inline e dai link. La
 * stessa regola, con gli stessi casi (`tempi.casi.json`), vive in `src-tauri/src/tempi_nota.rs`:
 * si guardano solo caratteri ASCII, quindi le posizioni in UTF-16 qui e in byte là coincidono
 * sul testo che si tiene.
 */

/** Un Tempo nella Nota: il tratto `[da, a)` con le quadre e il punto dell'audio. */
export interface TempoNota {
  a: number;
  da: number;
  ms: number;
}

const CIFRE = /^\d+$/;

/** Il tempo tra le quadre che cominciano in `i`, con la posizione dopo la chiusa. */
function tempoIn(testo: string, i: number): TempoNota | null {
  const chiusa = testo.indexOf("]", i);
  if (chiusa < 0 || chiusa - i > 9) {
    return null;
  }
  const parti = testo.slice(i + 1, chiusa).split(":");
  const lunghe = parti.map((p) => p.length);
  const valide =
    parti.every((p) => CIFRE.test(p)) &&
    ((parti.length === 2 && lunghe[0] <= 2 && lunghe[1] === 2) ||
      (parti.length === 3 &&
        lunghe[0] <= 2 &&
        lunghe[1] === 2 &&
        lunghe[2] === 2 &&
        Number(parti[1]) < 60));
  const [s, m = 0, h = 0] = parti.map(Number).reverse();
  if (!valide || s >= 60) {
    return null;
  }
  return { a: chiusa + 1, da: i, ms: ((h * 60 + m) * 60 + s) * 1000 };
}

/** La posizione dopo la chiusa che pareggia `apre` in `i`, saltando le annidate; -1 se manca. */
function chiusura(testo: string, i: number, apre: string, chiude: string) {
  let livello = 0;
  for (let j = i; j < testo.length; j += 1) {
    if (testo[j] === "\\") {
      j += 1;
    } else if (testo[j] === apre) {
      livello += 1;
    } else if (testo[j] === chiude) {
      livello -= 1;
      if (livello === 0) {
        return j + 1;
      }
    }
  }
  return -1;
}

/** Quanti `c` di fila da `i`. */
function fila(testo: string, i: number, c: string): number {
  let j = i;
  while (testo[j] === c) {
    j += 1;
  }
  return j - i;
}

/** Dove finisce il codice inline che comincia in `i`: apici senza chiusura uguale sono testo. */
function codice(testo: string, i: number): number {
  const n = fila(testo, i, "`");
  for (let j = testo.indexOf("`", i + n); j >= 0; ) {
    const m = fila(testo, j, "`");
    if (m === n) {
      return j + m;
    }
    j = testo.indexOf("`", j + m);
  }
  return i + n;
}

const AUTOLINK = /^[^\s<]*[:@][^\s<]*$/;
const INDIRIZZO = /^(?:https?:\/\/|www\.)/;
const SPAZIO = /\s/;

/** Dove finisce ciò che comincia in `i` e non contiene tempi (codice, link, indirizzi), o -1. */
function salta(testo: string, i: number): number {
  const c = testo[i];
  if (c === "`") {
    return codice(testo, i);
  }
  if (c === "<") {
    const fine = testo.indexOf(">", i);
    const dentro = fine < 0 ? "" : testo.slice(i + 1, fine);
    return AUTOLINK.test(dentro) ? fine + 1 : -1;
  }
  if (INDIRIZZO.test(testo.slice(i, i + 8))) {
    const spazio = testo.slice(i).search(SPAZIO);
    return spazio < 0 ? testo.length : i + spazio;
  }
  if (c === "[") {
    const quadra = chiusura(testo, i, "[", "]");
    if (quadra > 0 && testo[quadra] === "(") {
      const tonda = chiusura(testo, quadra, "(", ")");
      return tonda > 0 ? tonda : -1;
    }
  }
  return -1;
}

/** I Tempi nella Nota di `testo`, in ordine. */
export function tempiNellaNota(testo: string): TempoNota[] {
  const tempi: TempoNota[] = [];
  let i = 0;
  while (i < testo.length) {
    if (testo[i] === "\\") {
      i += 2;
      continue;
    }
    const tempo = testo[i] === "[" ? tempoIn(testo, i) : null;
    if (tempo && testo[tempo.a] !== "(" && testo[tempo.a] !== "[") {
      tempi.push(tempo);
      i = tempo.a;
      continue;
    }
    const dopo = salta(testo, i);
    i = dopo > i ? dopo : i + 1;
  }
  return tempi;
}

/** Il punto `ms` sulla linea del tempo dopo aver tolto i tratti `tagli`; `null` se è tolto. */
export function spostaMs(ms: number, tagli: Taglio[]): number | null {
  let tolti = 0;
  for (const { fineMs, inizioMs } of tagli) {
    if (inizioMs <= ms && ms < fineMs) {
      return null;
    }
    tolti += Math.max(0, Math.min(ms, fineMs) - inizioMs);
  }
  return ms - tolti;
}

/**
 * `testo` dopo un Taglio dei tratti `tagli`: i tempi dopo un tratto si spostano indietro, quelli
 * dentro perdono le quadre e tornano testo, il resto resta carattere per carattere.
 */
export function spostaTempi(testo: string, tagli: Taglio[]): string {
  let fuori = "";
  let pos = 0;
  for (const { a, da, ms } of tempiNellaNota(testo)) {
    const dopo = spostaMs(ms, tagli);
    if (dopo === ms) {
      continue;
    }
    fuori += testo.slice(pos, da);
    fuori +=
      dopo === null ? testo.slice(da + 1, a - 1) : `[${elapsedText(dopo)}]`;
    pos = a;
  }
  return fuori + testo.slice(pos);
}
