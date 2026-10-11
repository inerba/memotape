import { expect, test } from "bun:test";
import type { Nota } from "@/bindings";
import { notaAperta, salvataggio } from "@/features/notes/notes";

const nota = (nome: string, testo: string): Nota => ({
  autore: { tipo: "utente" },
  modificata: "2026-10-11T18:03:00+02:00",
  nome,
  testo,
});

test("un Tape senza Note apre una Nota nuova con il nome predefinito, che non c'è su disco", () => {
  expect(notaAperta([], "Note")).toEqual({ nome: "Note", salvato: null });
  expect(
    notaAperta([nota("Riepilogo", "## Decisioni"), nota("Mie", "")], "Note")
  ).toEqual({ nome: "Riepilogo", salvato: "## Decisioni" });
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
