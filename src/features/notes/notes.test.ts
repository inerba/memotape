import { expect, test } from "bun:test";
import type { Nota } from "@/bindings";
import {
  nomeNuovo,
  nomeRifiutato,
  noteIniziali,
  salvataggio,
  senzaNota,
} from "@/features/notes/notes";

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
    { nome: "Riepilogo", salvato: "## Decisioni" },
    { nome: "Mie", salvato: "" },
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
