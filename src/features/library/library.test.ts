import { describe, expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import type { BinoEntry, BinoInfo } from "@/bindings";
import {
  biniOf,
  chosenRaccolta,
  clockText,
  groupByDate,
  infoParts,
  nameProblem,
  sortBini,
} from "./library";

/** Un Bino creato all'ora locale indicata. */
function bino(
  titolo: string,
  [y, mo, d, h = 12, mi = 0]: number[],
  raccolta: string | null = null
): BinoEntry {
  return {
    creato: new Date(y ?? 0, (mo ?? 1) - 1, d, h, mi).toISOString(),
    durataMs: 60_000,
    path: `C:\\Sbobino\\${raccolta ? `${raccolta}\\` : ""}${titolo}.bino`,
    raccolta,
    titolo,
  };
}

const groups = (bini: BinoEntry[], today: Date) =>
  groupByDate(bini, today).map((g) => [g.key, g.bini.map((b) => b.titolo)]);

describe("raggruppamento per data", () => {
  test("a cavallo di mezzanotte", () => {
    // Mercoledì 7 ottobre 2026, poco dopo mezzanotte.
    const today = new Date(2026, 9, 7, 0, 5);
    expect(
      groups(
        [
          bino("mezzanotte", [2026, 10, 7, 0, 0]),
          bino("prima", [2026, 10, 6, 23, 59]),
          bino("ieri mattina", [2026, 10, 6, 0, 0]),
          bino("lunedì", [2026, 10, 5, 9, 0]),
          bino("futuro", [2026, 10, 8]),
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
          bino("domenica", [2026, 10, 4]),
          bino("sabato", [2026, 10, 3]),
          bino("oggi", [2026, 10, 5, 8]),
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
          bino("martedì", [2026, 9, 29]),
          bino("domenica", [2026, 9, 27]),
          bino("agosto", [2026, 8, 31]),
          bino("dicembre", [2025, 12, 31]),
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
});

test("l'ora è HH:mm nell'ora locale", () => {
  expect(clockText(new Date(2026, 9, 4, 9, 5).toISOString())).toBe("09:05");
  expect(clockText("2026-10-04T23:59:00")).toBe("23:59");
});

test("l'elenco completo si ordina per data o per titolo", () => {
  const bini = [
    bino("beta", [2026, 10, 1]),
    bino("Alfa", [2026, 9, 1]),
    bino("gamma", [2026, 10, 3]),
  ];
  expect(sortBini(bini, "date").map((b) => b.titolo)).toEqual([
    "gamma",
    "beta",
    "Alfa",
  ]);
  expect(sortBini(bini, "title").map((b) => b.titolo)).toEqual([
    "Alfa",
    "beta",
    "gamma",
  ]);
});

test("la Raccolta scelta, Senza raccolta e Tutta la Libreria", () => {
  const bini = [
    bino("sciolto", [2026, 10, 1]),
    bino("call", [2026, 10, 2], "Acme"),
    bino("altra", [2026, 10, 3], "Beta"),
  ];
  const titoli = (raccolta: string | null) =>
    biniOf(bini, raccolta).map((b) => b.titolo);
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

describe("informazioni di un Bino", () => {
  const t = i18n.t.bind(i18n);
  const info: BinoInfo = {
    completa: true,
    creato: new Date(2026, 9, 3, 17, 5).toISOString(),
    durataMs: 754_000,
    ingressiSeparati: false,
    linguaParlato: "it",
    modello: "Nemotron",
    origine: null,
  };

  test("data e ora, durata, Raccolta, modello e Lingua del parlato", () => {
    const [date, ...rest] = infoParts(info, "Acme", t, "it");
    expect(date).toContain("17:05");
    expect(date).toContain("2026");
    expect(rest).toEqual(["12:34", "Acme", "Nemotron", "Italiano"]);
  });

  test("Senza raccolta, fuori dalla Libreria, Ingressi separati, incompleto e file d'origine", () => {
    const all = {
      ...info,
      completa: false,
      ingressiSeparati: true,
      linguaParlato: "auto" as const,
      modello: null,
      origine: "Call Teams.mp4",
    };
    expect(infoParts(all, null, t, "it").slice(1)).toEqual([
      "12:34",
      "Senza raccolta",
      "Automatica",
      "Ingressi separati",
      "incompleto",
      "dal file Call Teams.mp4",
    ]);
    expect(infoParts(info, undefined, t, "it")[2]).toBe("fuori dalla Libreria");
  });
});
