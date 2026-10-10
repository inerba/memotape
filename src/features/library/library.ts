import type { TFunction } from "i18next";
import type { TapeEntry } from "@/bindings";

/**
 * Un gruppo della barra laterale: `today`, `yesterday`, `week` (il resto della settimana, da
 * lunedì) o un mese, `AAAA-MM`.
 */
export interface DateGroup {
  key: string;
  tapes: TapeEntry[];
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

/** I Tape dal più recente, raggruppati in Oggi, Ieri, Questa settimana e poi per mese. */
export function groupByDate(tapes: TapeEntry[], today: Date): DateGroup[] {
  const groups: DateGroup[] = [];
  let last: DateGroup | undefined;
  for (const tape of sortTapes(tapes, DEFAULT_ORDER)) {
    const key = groupKey(new Date(tape.creato), today);
    if (last?.key === key) {
      last.tapes.push(tape);
    } else {
      last = { key, tapes: [tape] };
      groups.push(last);
    }
  }
  return groups;
}

/** L'ora di un Tape, `HH:mm`. */
export function clockText(creato: string): string {
  const date = new Date(creato);
  return `${two(date.getHours())}:${two(date.getMinutes())}`;
}

/** `creato` per `<input type="datetime-local">`: `AAAA-MM-GGTHH:mm` nell'ora locale. */
export function dateTimeInput(creato: string): string {
  const date = new Date(creato);
  return `${date.getFullYear()}-${two(date.getMonth() + 1)}-${two(date.getDate())}T${clockText(creato)}`;
}

/** Le colonne dell'elenco completo per cui si ordina. */
export type TapeColumn = "date" | "title" | "duration";
const COLUMNS: TapeColumn[] = ["date", "title", "duration"];

export interface TapeOrder {
  column: TapeColumn;
  descending: boolean;
}

/** Dal più recente. */
export const DEFAULT_ORDER: TapeOrder = { column: "date", descending: true };

const COMPARE: Record<
  Exclude<TapeColumn, "title">,
  (a: TapeEntry, b: TapeEntry) => number
> = {
  date: (a, b) => Date.parse(a.creato) - Date.parse(b.creato),
  // Senza durata (Tape illeggibile) è il più corto.
  duration: (a, b) => (a.durataMs ?? -1) - (b.durataMs ?? -1),
};

/** `title` è il titolo mostrato, per cui si ordina la colonna Titolo. */
export function sortTapes(
  tapes: TapeEntry[],
  order: TapeOrder,
  title: (tape: TapeEntry) => string = (tape) => tape.titolo
): TapeEntry[] {
  const compare =
    order.column === "title"
      ? (a: TapeEntry, b: TapeEntry) => title(a).localeCompare(title(b))
      : COMPARE[order.column];
  return [...tapes].sort((a, b) =>
    order.descending ? compare(b, a) : compare(a, b)
  );
}

/**
 * Il clic sull'intestazione `column`: la stessa colonna inverte il verso, un'altra parte dal più
 * recente o più lungo, il titolo dalla A.
 */
export function nextOrder(current: TapeOrder, column: TapeColumn): TapeOrder {
  if (current.column === column) {
    return { column, descending: !current.descending };
  }
  return { column, descending: column !== "title" };
}

/** L'ordinamento ricordato (JSON), o quello predefinito se manca o non vale. */
export function orderOf(saved: string | null): TapeOrder {
  try {
    const order = JSON.parse(saved ?? "null");
    if (
      COLUMNS.includes(order?.column) &&
      typeof order.descending === "boolean"
    ) {
      return { column: order.column, descending: order.descending };
    }
  } catch {
    // Un valore rotto vale come assente.
  }
  return DEFAULT_ORDER;
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

/** I Tape della Raccolta `raccolta` (vedi `chosenRaccolta`). */
export function tapesOf(
  tapes: TapeEntry[],
  raccolta: string | null
): TapeEntry[] {
  if (raccolta === null) {
    return tapes;
  }
  return tapes.filter((b) => (b.raccolta ?? "") === raccolta);
}

const FORBIDDEN = /[<>:"/\\|?*\p{Cc}]/u;
const DEVICE = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;

/**
 * Perché `name` non va bene per una Raccolta o il titolo di un Tape, come `library::validate_name`
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
 * Il giorno di un Tape per il suo titolo: "Oggi, 4 ottobre", "Ieri, 3 ottobre", senza l'anno se è
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
