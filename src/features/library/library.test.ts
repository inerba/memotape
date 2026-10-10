import { describe, expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import type { TapeEntry } from "@/bindings";
import {
  chosenRaccolta,
  clockText,
  DEFAULT_ORDER,
  dateTimeInput,
  dayText,
  durationWords,
  groupByDate,
  nameProblem,
  nextOrder,
  orderOf,
  recentWhen,
  sortTapes,
  type TapeColumn,
  tapesOf,
} from "./library";

/** Un Tape creato all'ora locale indicata. */
function tape(
  titolo: string,
  [y, mo, d, h = 12, mi = 0]: number[],
  raccolta: string | null = null
): TapeEntry {
  return {
    creato: new Date(y ?? 0, (mo ?? 1) - 1, d, h, mi).toISOString(),
    durataMs: 60_000,
    path: `C:\\Memotape\\${raccolta ? `${raccolta}\\` : ""}${titolo}.tape`,
    raccolta,
    titolo,
  };
}

const groups = (tapes: TapeEntry[], today: Date) =>
  groupByDate(tapes, today).map((g) => [g.key, g.tapes.map((b) => b.titolo)]);

describe("raggruppamento per data", () => {
  test("a cavallo di mezzanotte", () => {
    // Mercoledì 7 ottobre 2026, poco dopo mezzanotte.
    const today = new Date(2026, 9, 7, 0, 5);
    expect(
      groups(
        [
          tape("mezzanotte", [2026, 10, 7, 0, 0]),
          tape("prima", [2026, 10, 6, 23, 59]),
          tape("ieri mattina", [2026, 10, 6, 0, 0]),
          tape("lunedì", [2026, 10, 5, 9, 0]),
          tape("futuro", [2026, 10, 8]),
        ],
        today
      )
    ).toEqual([
      ["today", ["futuro", "mezzanotte"]],
      ["yesterday", ["prima", "ieri mattina"]],
      ["week", ["lunedì"]],
    ]);
  });

  test("all'inizio della settimana", () => {
    // Lunedì 5 ottobre 2026: domenica è Ieri, sabato è già il mese.
    const today = new Date(2026, 9, 5, 10, 0);
    expect(
      groups(
        [
          tape("domenica", [2026, 10, 4]),
          tape("sabato", [2026, 10, 3]),
          tape("oggi", [2026, 10, 5, 8]),
        ],
        today
      )
    ).toEqual([
      ["today", ["oggi"]],
      ["yesterday", ["domenica"]],
      ["2026-10", ["sabato"]],
    ]);
  });

  test("al cambio di mese", () => {
    // Giovedì 1 ottobre 2026: la settimana è iniziata lunedì 28 settembre.
    const today = new Date(2026, 9, 1, 18, 0);
    expect(
      groups(
        [
          tape("martedì", [2026, 9, 29]),
          tape("domenica", [2026, 9, 27]),
          tape("agosto", [2026, 8, 31]),
          tape("dicembre", [2025, 12, 31]),
        ],
        today
      )
    ).toEqual([
      ["week", ["martedì"]],
      ["2026-09", ["domenica"]],
      ["2026-08", ["agosto"]],
      ["2025-12", ["dicembre"]],
    ]);
  });

  test("con un limite restano i Tape più recenti", () => {
    const today = new Date(2026, 9, 8, 18, 0);
    const days = [8, 2, 7, 1];
    expect(
      groupByDate(
        days.map((d) => tape(String(d), [2026, 10, d])),
        today,
        2
      ).map((g) => [g.key, g.tapes.map((b) => b.titolo)])
    ).toEqual([
      ["today", ["8"]],
      ["yesterday", ["7"]],
    ]);
  });
});

test("l'ora è HH:mm nell'ora locale", () => {
  expect(clockText(new Date(2026, 9, 4, 9, 5).toISOString())).toBe("09:05");
  expect(clockText("2026-10-04T23:59:00")).toBe("23:59");
});

test("l'elenco completo si ordina per data, titolo o durata, nei due versi", () => {
  const tapes = [
    { ...tape("beta", [2026, 10, 1]), durataMs: 5000 },
    { ...tape("Alfa", [2026, 9, 1]), durataMs: null },
    { ...tape("gamma", [2026, 10, 3]), durataMs: 9000 },
  ];
  const titoli = (column: TapeColumn, descending: boolean) =>
    sortTapes(tapes, { column, descending }).map((b) => b.titolo);
  expect(titoli("date", true)).toEqual(["gamma", "beta", "Alfa"]);
  expect(titoli("date", false)).toEqual(["Alfa", "beta", "gamma"]);
  expect(titoli("title", false)).toEqual(["Alfa", "beta", "gamma"]);
  expect(titoli("title", true)).toEqual(["gamma", "beta", "Alfa"]);
  // Senza durata (Tape illeggibile) viene dopo i più corti.
  expect(titoli("duration", true)).toEqual(["gamma", "beta", "Alfa"]);
  expect(titoli("duration", false)).toEqual(["Alfa", "beta", "gamma"]);
});

test("per titolo si ordina il titolo mostrato, non il nome del file", () => {
  const tapes = [
    tape("Registrazione 2026-10-08 17-32-36", [2026, 10, 8]),
    tape("Bilancio", [2026, 10, 7]),
  ];
  const shown = (b: TapeEntry) =>
    b.titolo === "Bilancio" ? "Bilancio" : "Alle 17:32";
  expect(
    sortTapes(tapes, { column: "title", descending: false }, shown).map(shown)
  ).toEqual(["Alle 17:32", "Bilancio"]);
});

test("il clic su una colonna la sceglie con il suo verso, il secondo lo inverte", () => {
  expect(nextOrder(DEFAULT_ORDER, "date")).toEqual({
    column: "date",
    descending: false,
  });
  expect(nextOrder(DEFAULT_ORDER, "title")).toEqual({
    column: "title",
    descending: false,
  });
  expect(nextOrder(DEFAULT_ORDER, "duration")).toEqual({
    column: "duration",
    descending: true,
  });
  expect(nextOrder({ column: "title", descending: false }, "title")).toEqual({
    column: "title",
    descending: true,
  });
});

test("l'ordinamento ricordato si rilegge, o vale quello predefinito", () => {
  const order = { column: "duration", descending: false } as const;
  expect(orderOf(JSON.stringify(order))).toEqual(order);
  expect(orderOf(null)).toEqual(DEFAULT_ORDER);
  expect(orderOf("{rotto")).toEqual(DEFAULT_ORDER);
  expect(orderOf('{"column":"raccolta","descending":true}')).toEqual(
    DEFAULT_ORDER
  );
});

test("la data e l'ora di un Tape per il campo data e ora, nell'ora locale", () => {
  expect(dateTimeInput(new Date(2026, 9, 3, 7, 5, 42).toISOString())).toBe(
    "2026-10-03T07:05"
  );
});

test("la Raccolta scelta, Senza raccolta e Tutta la Libreria", () => {
  const tapes = [
    tape("sciolto", [2026, 10, 1]),
    tape("call", [2026, 10, 2], "Acme"),
    tape("altra", [2026, 10, 3], "Beta"),
  ];
  const titoli = (raccolta: string | null) =>
    tapesOf(tapes, raccolta).map((b) => b.titolo);
  expect(titoli(null)).toEqual(["sciolto", "call", "altra"]);
  expect(titoli("")).toEqual(["sciolto"]);
  expect(titoli("Acme")).toEqual(["call"]);
  const raccolte = ["Acme", "Beta"];
  expect(chosenRaccolta(undefined, raccolte)).toBeNull();
  expect(chosenRaccolta(null, raccolte)).toBeNull();
  expect(chosenRaccolta("", raccolte)).toBe("");
  expect(chosenRaccolta("Acme", raccolte)).toBe("Acme");
  // Una Raccolta che non esiste più vale Tutta la Libreria.
  expect(chosenRaccolta("Sparita", raccolte)).toBeNull();
});

test("validazione dei nomi, come in Rust", () => {
  for (const name of [
    "",
    "a/b",
    "a\\b",
    "a:b",
    "a?b",
    "a*b",
    'a"b',
    "a<b>",
    "a|b",
    "tab\tqui",
    "CON",
    "nul.txt",
    "COM1",
    "lpt9",
    "fine.",
    "fine ",
    ".nascosta",
    "..",
  ]) {
    expect(nameProblem(name, [])).toBe("invalidName");
  }
  for (const name of ["Ferrara Quarzi, preventivo", "CONTO", "COM0", "Già è"]) {
    expect(nameProblem(name, [])).toBeNull();
  }
  expect(nameProblem("acme", ["Acme", "Beta"])).toBe("nameTaken");
  expect(nameProblem("Acme Srl", ["Acme", "Beta"])).toBeNull();
});

test("il giorno del titolo dice Oggi e Ieri, e l'anno solo se non è quello in corso", () => {
  const t = i18n.t.bind(i18n);
  const today = new Date(2026, 9, 4, 18, 0);
  const at = (y: number, m: number, d: number) =>
    new Date(y, m, d, 10, 30).toISOString();
  expect(dayText(at(2026, 9, 4), today, t, "it")).toBe("Oggi, 4 ottobre");
  expect(dayText(at(2026, 9, 3), today, t, "it")).toBe("Ieri, 3 ottobre");
  expect(dayText(at(2026, 8, 28), today, t, "it")).toBe("28 settembre");
  expect(dayText(at(2025, 11, 31), today, t, "it")).toBe("31 dicembre 2025");
});

test("la durata a parole: secondi sotto il minuto, minuti sotto l'ora, poi ore e minuti", () => {
  expect(durationWords(24_000, "it")).toBe("24 s");
  expect(durationWords(61_000, "it")).toBe("1 min");
  expect(durationWords((59 * 60 + 26) * 1000, "it")).toBe("59 min");
  expect(durationWords(((2 * 60 + 4) * 60 + 41) * 1000, "it")).toBe(
    "2 h 05 min"
  );
  // Arrotondati, 59,6 s e 59 min 40 s passano all'unità successiva.
  expect(durationWords(59_600, "it")).toBe("1 min");
  expect(durationWords((59 * 60 + 40) * 1000, "it")).toBe("1 h 00 min");
});

test("nei Recenti Oggi e Ieri hanno l'ora, la settimana il giorno abbreviato, i mesi anche il mese", () => {
  const at = (m: number, d: number, h: number, mi: number) =>
    new Date(2026, m, d, h, mi).toISOString();
  expect(recentWhen(at(9, 8, 17, 32), "today", "it")).toBe("17:32");
  expect(recentWhen(at(9, 7, 9, 5), "yesterday", "it")).toBe("09:05");
  expect(recentWhen(at(9, 8, 17, 32), "week", "it")).toBe("gio 8, 17:32");
  expect(recentWhen(at(6, 31, 11, 10), "2026-07", "it")).toBe("31 lug, 11:10");
});
