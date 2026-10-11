import { expect, test } from "bun:test";
import type { Nota } from "@/bindings";
import {
  apertaIniziale,
  bozzaNote,
  nomeNuovo,
  nomeRifiutato,
  noteIniziali,
  oraDellaFirma,
  rilette,
  salvataggio,
  senzaNota,
} from "@/features/notes/notes";

const firma = {
  autore: { tipo: "utente" },
  modificata: "2026-10-11T18:03:00+02:00",
} as const;

const nota = (nome: string, testo: string): Nota => ({
  autore: { tipo: "utente" },
  modificata: "2026-10-11T18:03:00+02:00",
  nome,
  testo,
});

test("un Tape senza Note mostra una Nota nuova con il nome predefinito, che non c'è su disco", () => {
  expect(noteIniziali([], "Note")).toEqual([{ nome: "Note", salvato: null }]);
  expect(
    noteIniziali([nota("Riepilogo", "## Decisioni"), nota("Mie", "")], "Note")
  ).toEqual([
    { firma, nome: "Riepilogo", salvato: "## Decisioni" },
    { firma, nome: "Mie", salvato: "" },
  ]);
});

test("una Nota nuova si chiama «Nota 2», «Nota 3»… saltando i nomi già usati", () => {
  const nome = (n: number) => `Nota ${n}`;
  expect(nomeNuovo(["Note"], nome)).toBe("Nota 2");
  expect(nomeNuovo(["Note", "Nota 2"], nome)).toBe("Nota 3");
  expect(nomeNuovo(["Riepilogo", "Nota 3"], nome)).toBe("Nota 4");
  expect(nomeNuovo([], nome)).toBe("Nota 1");
});

test("un nome vuoto, oltre 60 caratteri o già usato si rifiuta con il codice del backend", () => {
  const altre = ["Note", "Riepilogo"];
  expect(nomeRifiutato("Domande per Sara", altre)).toBeNull();
  expect(nomeRifiutato("è".repeat(60), altre)).toBeNull();
  expect(nomeRifiutato("", altre)).toEqual({
    code: "noteNameInvalid",
    detail: "",
  });
  expect(nomeRifiutato("   ", altre)).toEqual({
    code: "noteNameInvalid",
    detail: "   ",
  });
  expect(nomeRifiutato("è".repeat(61), altre)).toEqual({
    code: "noteNameInvalid",
    detail: "è".repeat(61),
  });
  expect(nomeRifiutato("Riepilogo", altre)).toEqual({
    code: "noteNameTaken",
    detail: "Riepilogo",
  });
});

test("eliminata la Nota aperta si apre la seguente, o la precedente se era l'ultima", () => {
  const note = [
    { nome: "A", salvato: "a" },
    { nome: "B", salvato: "b" },
    { nome: "C", salvato: "c" },
  ];
  expect(senzaNota(note, "B", "Note")).toEqual({
    aperta: "C",
    note: [note[0], note[2]],
  });
  expect(senzaNota(note, "C", "Note")).toEqual({
    aperta: "B",
    note: [note[0], note[1]],
  });
  // Senza più Note torna la Nota nuova con il nome predefinito.
  expect(senzaNota([note[0]], "A", "Note")).toEqual({
    aperta: "Note",
    note: [{ nome: "Note", salvato: null }],
  });
});

test("un risultato della ricerca apre la sua Nota, la prima se non c'è più", () => {
  const note = noteIniziali(
    [nota("Mie note", "Uno"), nota("Riepilogo", "## Decisioni")],
    "Note"
  );
  expect(apertaIniziale(note, "Riepilogo")).toBe("Riepilogo");
  expect(apertaIniziale(note, "Rinominata")).toBe("Mie note");
  expect(apertaIniziale(note)).toBe("Mie note");
});

test("si salva solo una bozza diversa dal testo su disco, che diventa l'atteso", () => {
  // Il primo carattere crea la Nota: l'atteso è vuoto.
  expect(salvataggio("Note", null, "D")).toEqual({
    atteso: "",
    nome: "Note",
    testo: "D",
  });
  // Una Nota nuova ancora vuota non si scrive.
  expect(salvataggio("Note", null, "")).toBeNull();
  expect(salvataggio("Note", "Uno", "Uno")).toBeNull();
  expect(salvataggio("Note", "Uno", "Uno due")).toEqual({
    atteso: "Uno",
    nome: "Note",
    testo: "Uno due",
  });
  // Svuotare una Nota che c'è è una modifica.
  expect(salvataggio("Note", "Uno", "")).toEqual({
    atteso: "Uno",
    nome: "Note",
    testo: "",
  });
});

test("rilette dal disco, le Note senza bozza prendono testo e firma nuovi", () => {
  const claude = { client: "Claude", tipo: "assistente" } as const;
  const locali = noteIniziali(
    [nota("Mie note", "Uno"), nota("Vecchia", "v")],
    "Note"
  );
  const { conflitti, note } = rilette(
    [...locali, { nome: "Nota 3", salvato: null }],
    [
      { ...nota("Mie note", "Uno due"), autore: claude },
      { ...nota("Riepilogo", "## Decisioni"), autore: claude },
    ],
    {},
    "Note"
  );
  expect(conflitti).toEqual({});
  // La Nota nuova ancora vuota resta in fondo; quella sparita dal disco se ne va.
  expect(note).toEqual([
    {
      firma: { ...firma, autore: claude },
      nome: "Mie note",
      salvato: "Uno due",
    },
    {
      firma: { ...firma, autore: claude },
      nome: "Riepilogo",
      salvato: "## Decisioni",
    },
    { nome: "Nota 3", salvato: null },
  ]);
  // Senza più Note torna la Nota nuova con il nome predefinito.
  expect(rilette(locali, [], {}, "Note").note).toEqual([
    { nome: "Note", salvato: null },
  ]);
});

test("una Nota con bozza cambiata da fuori tiene bozza e testo da cui parte, in conflitto", () => {
  const locali = noteIniziali(
    [nota("Mie note", "Uno"), nota("B", "b")],
    "Note"
  );
  const bozze = { B: "b mia", "Mie note": "Uno mio" };
  const { conflitti, note } = rilette(
    locali,
    [nota("Mie note", "Uno suo"), nota("B", "b")],
    bozze,
    "Note"
  );
  // «B» non è cambiata: la sua bozza si salverà come sempre.
  expect(conflitti).toEqual({ "Mie note": nota("Mie note", "Uno suo") });
  expect(note).toEqual(locali);
  // Una Nota con bozza sparita dal disco (rinominata da un Assistente) resta, da ricreare.
  expect(rilette(locali, [nota("B", "b")], bozze, "Note").note).toEqual([
    locali[1],
    { nome: "Mie note", salvato: null },
  ]);
});

test("la firma dice l'ora per oggi, giorno e ora per gli altri giorni", () => {
  const adesso = new Date("2026-10-11T20:00:00");
  expect(oraDellaFirma("2026-10-11T17:52:00", adesso, "it")).toBe("17:52");
  expect(oraDellaFirma("2026-10-09T08:05:00", adesso, "it")).toBe(
    "9 ott, 08:05"
  );
});

test("la bozza della Registrazione ha le Note scritte, in ordine, anche svuotate, non quelle nuove mai scritte", () => {
  expect(
    bozzaNote([
      { nome: "Note", salvato: "Decidere [1:05]" },
      { nome: "Nota 2", salvato: null },
      { nome: "Domande", salvato: "" },
    ])
  ).toEqual([
    { nome: "Note", testo: "Decidere [1:05]" },
    { nome: "Domande", testo: "" },
  ]);
  expect(bozzaNote([{ nome: "Note", salvato: null }])).toEqual([]);
});
