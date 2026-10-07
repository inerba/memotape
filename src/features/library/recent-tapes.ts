import type { TFunction } from "i18next";
import type { TapeEntry } from "@/bindings";
import { DEFAULT_ORDER, sortTapes } from "./library";

// I prefissi delle sei lingue: un cambio di lingua non rende lunghi i vecchi nomi automatici.
const RECORDING_NAME =
  /^(?:Registrazione|Recording|Aufnahme|Grabación|Enregistrement|Nagranie) (\d{4}-\d{2}-\d{2}) (\d{2})-(\d{2})-(\d{2})( \d+)?$/;

/** Solo presentazione: i titoli personalizzati e i nomi dei file non cambiano. */
export function recentTitle(
  tape: TapeEntry,
  tapes: TapeEntry[],
  t: TFunction
): string {
  const match = RECORDING_NAME.exec(tape.titolo);
  if (!match) {
    return tape.titolo;
  }
  const [, day, hour, minute, second, suffix = ""] = match;
  const sameMinute = tapes.some((other) => {
    if (other.path === tape.path) {
      return false;
    }
    const name = RECORDING_NAME.exec(other.titolo);
    return name?.[1] === day && name?.[2] === hour && name?.[3] === minute;
  });
  const time = `${hour}:${minute}${sameMinute ? `:${second}` : ""}`;
  return t("home.recordingTitle", { time }) + suffix;
}

/** Il ricordo vale solo per un Tape leggibile ancora nella Libreria corrente. */
export function homeTapes(
  tapes: TapeEntry[],
  lastPath: string | null
): {
  featured: TapeEntry | null;
  resumed: boolean;
  recent: TapeEntry[];
} {
  const ordered = sortTapes(tapes, DEFAULT_ORDER);
  const readable = ordered.filter((tape) => tape.durataMs !== null);
  const last = readable.find(
    (tape) => tape.path.toLowerCase() === lastPath?.toLowerCase()
  );
  return {
    featured: last ?? readable[0] ?? null,
    recent: ordered.slice(0, 3),
    resumed: last !== undefined,
  };
}
