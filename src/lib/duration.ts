/**
 * Le durate a parole, con le unità brevi di `Intl` nella lingua `locale`: «24 s», «59 min», «2 h».
 * Tabella della Libreria e player le scrivono in cifre (`elapsedText`).
 */

type Unita = "second" | "minute" | "hour";

function unita(n: number, unit: Unita, locale: string, digits = 1): string {
  return new Intl.NumberFormat(locale, {
    minimumIntegerDigits: digits,
    style: "unit",
    unit,
    unitDisplay: "short",
  }).format(n);
}

/**
 * La durata di un Tape nei Recenti: «24 s» sotto il minuto, «59 min» sotto l'ora, poi
 * «2 h 05 min». Arrotondata, non si confonde con un'ora.
 */
export function durationWords(ms: number, locale: string): string {
  const seconds = Math.round(ms / 1000);
  if (seconds < 60) {
    return unita(seconds, "second", locale);
  }
  const minutes = Math.round(ms / 60_000);
  if (minutes < 60) {
    return unita(minutes, "minute", locale);
  }
  return `${unita(Math.floor(minutes / 60), "hour", locale)} ${unita(minutes % 60, "minute", locale, 2)}`;
}

/** La durata di una pausa al secondo: «9 s», «1 min 20 s», «1 min». */
export function pauseDuration(ms: number, locale: string): string {
  const seconds = Math.round(ms / 1000);
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  if (m === 0) {
    return unita(s, "second", locale);
  }
  const min = unita(m, "minute", locale);
  return s === 0 ? min : `${min} ${unita(s, "second", locale)}`;
}
