import type { TFunction } from "i18next";
import type { BinoEntry } from "@/bindings";

/**
 * Un gruppo della barra laterale: `today`, `yesterday`, `week` (il resto della settimana, da
 * lunedì) o un mese, `AAAA-MM`.
 */
export interface DateGroup {
  bini: BinoEntry[];
  key: string;
}

const two = (n: number) => String(n).padStart(2, "0");

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function daysBefore(date: Date, days: number): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() - days);
}

/** Il gruppo di `date` rispetto a `today`. Una data futura (orologio spostato) è Oggi. */
function groupKey(date: Date, today: Date): string {
  const midnight = startOfDay(today);
  if (date >= midnight) {
    return "today";
  }
  if (date >= daysBefore(midnight, 1)) {
    return "yesterday";
  }
  // Lunedì: getDay() conta da domenica.
  if (date >= daysBefore(midnight, (midnight.getDay() + 6) % 7)) {
    return "week";
  }
  return `${date.getFullYear()}-${two(date.getMonth() + 1)}`;
}

/** I Bini dal più recente, raggruppati in Oggi, Ieri, Questa settimana e poi per mese. */
export function groupByDate(bini: BinoEntry[], today: Date): DateGroup[] {
  const groups: DateGroup[] = [];
  let last: DateGroup | undefined;
  for (const bino of sortBini(bini, "date")) {
    const key = groupKey(new Date(bino.creato), today);
    if (last?.key === key) {
      last.bini.push(bino);
    } else {
      last = { bini: [bino], key };
      groups.push(last);
    }
  }
  return groups;
}

/** L'ora di un Bino, `HH:mm`. */
export function clockText(creato: string): string {
  const date = new Date(creato);
  return `${two(date.getHours())}:${two(date.getMinutes())}`;
}

/** L'ordinamento dell'elenco completo: dal più recente o per titolo. */
export type BinoOrder = "date" | "title";

export function sortBini(bini: BinoEntry[], order: BinoOrder): BinoEntry[] {
  return [...bini].sort(
    order === "date"
      ? (a, b) => Date.parse(b.creato) - Date.parse(a.creato)
      : (a, b) => a.titolo.localeCompare(b.titolo)
  );
}

/**
 * La Raccolta in uso: `null` Tutta la Libreria, `""` Senza raccolta, altrimenti il nome. Una
 * Raccolta che non esiste più vale Tutta la Libreria.
 */
export function chosenRaccolta(
  saved: string | null | undefined,
  raccolte: string[]
): string | null {
  return saved === "" || (saved && raccolte.includes(saved)) ? saved : null;
}

/** Il nome mostrato della Raccolta `raccolta` (vedi `chosenRaccolta`). */
export function raccoltaLabel(raccolta: string | null, t: TFunction): string {
  if (raccolta === null) {
    return t("library.all");
  }
  return raccolta === "" ? t("library.none") : raccolta;
}

/** I Bini della Raccolta `raccolta` (vedi `chosenRaccolta`). */
export function biniOf(
  bini: BinoEntry[],
  raccolta: string | null
): BinoEntry[] {
  if (raccolta === null) {
    return bini;
  }
  return bini.filter((b) => (b.raccolta ?? "") === raccolta);
}

const FORBIDDEN = /[<>:"/\\|?*\p{Cc}]/u;
const DEVICE = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;

/**
 * Perché `name` non va bene per una Raccolta o il titolo di un Bino, come `library::validate_name`
 * in Rust: `invalidName` (vuoto, caratteri non ammessi da Windows, nome riservato, punto o spazio in
 * fondo, punto in testa) o `nameTaken` (uno di `taken`, senza distinguere maiuscole e minuscole).
 * `null` se va bene.
 */
export function nameProblem(
  name: string,
  taken: string[]
): "invalidName" | "nameTaken" | null {
  const base = (name.split(".")[0] ?? "").trimEnd();
  if (
    name === "" ||
    FORBIDDEN.test(name) ||
    name.endsWith(".") ||
    name.endsWith(" ") ||
    name.startsWith(".") ||
    DEVICE.test(base)
  ) {
    return "invalidName";
  }
  const lower = name.toLowerCase();
  return taken.some((t) => t.toLowerCase() === lower) ? "nameTaken" : null;
}

/**
 * Il giorno di un Bino per il suo titolo: "Oggi, 4 ottobre", "Ieri, 3 ottobre", senza l'anno se è
 * quello di `today`, altrimenti la data intera.
 */
export function dayText(
  creato: string,
  today: Date,
  t: TFunction,
  locale: string
): string {
  const date = new Date(creato);
  const sameYear = date.getFullYear() === today.getFullYear();
  const day = date.toLocaleDateString(locale, {
    day: "numeric",
    month: "long",
    year: sameYear ? undefined : "numeric",
  });
  const key = groupKey(date, today);
  return key === "today" || key === "yesterday"
    ? `${t(`library.groups.${key}`)}, ${day}`
    : day;
}
