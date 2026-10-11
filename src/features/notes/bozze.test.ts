import { expect, test } from "bun:test";
import { type Bozze, bozze, bozzeDi, uscita } from "./bozze";

const A = "D:\\Memotape\\riunione.tape";

test("scrivere lascia una bozza; tornare al testo su disco la toglie", () => {
  let stato = bozze(
    {},
    { nome: "Note", path: A, salvato: "ciao", testo: "ciao!", type: "scritta" }
  );
  expect(bozzeDi(stato, A)).toEqual({ Note: "ciao!" });
  stato = bozze(stato, {
    nome: "Note",
    path: A,
    salvato: "ciao",
    testo: "ciao",
    type: "scritta",
  });
  expect(bozzeDi(stato, A)).toEqual({});
  // Una Nota nuova, non ancora su disco, vuota non ha bozza.
  const nuova: Bozze = bozze(
    {},
    { nome: "Nota 2", path: A, salvato: null, testo: "", type: "scritta" }
  );
  expect(bozzeDi(nuova, A)).toEqual({});
});

test("salvata toglie la bozza solo se è il testo salvato: quello scritto intanto resta", () => {
  const scritta = bozze(
    {},
    { nome: "Note", path: A, salvato: "", testo: "abc", type: "scritta" }
  );
  const salvata = {
    nome: "Note",
    path: A,
    testo: "abc",
    type: "salvata",
  } as const;
  expect(bozze(scritta, salvata)).toEqual({});
  const intanto = bozze(scritta, {
    nome: "Note",
    path: A,
    salvato: "",
    testo: "abcd",
    type: "scritta",
  });
  expect(bozzeDi(bozze(intanto, salvata), A)).toEqual({ Note: "abcd" });
});

/** Due Note con bozza nel Tape `A` e una nel Tape `B`. */
const B = "D:\\Memotape\\lezione.tape";
const TRE: Bozze = { [A]: { Domande: "?", Note: "x" }, [B]: { Note: "y" } };

test("la bozza segue la Nota rinominata e sparisce con la Nota eliminata", () => {
  expect(
    bozze(TRE, { nome: "Note", nuovo: "Mie", path: A, type: "rinominata" })[A]
  ).toEqual({ Domande: "?", Mie: "x" });
  expect(bozze(TRE, { nome: "Note", path: A, type: "eliminata" })[A]).toEqual({
    Domande: "?",
  });
});

test("Non salvare scarta le bozze del Tape, non quelle degli altri", () => {
  expect(bozze(TRE, { path: A, type: "scartate" })).toEqual({
    [B]: { Note: "y" },
  });
});

test("le bozze seguono il Tape spostato e spariscono con il Tape che non c'è più", () => {
  const C = "D:\\Memotape\\Lavoro\\riunione.tape";
  expect(bozze(TRE, { from: A, to: C, type: "tapeMoved" })).toEqual({
    [B]: { Note: "y" },
    [C]: { Domande: "?", Note: "x" },
  });
  expect(bozze(TRE, { path: B, type: "tapeGone" })).toEqual({
    [A]: { Domande: "?", Note: "x" },
  });
});

test("lasciare un Tape con bozze chiede di salvarle; con Salva da solo si salvano senza domande", () => {
  expect(uscita(TRE, { mostrato: A, salvaDaSolo: false })).toBe("chiedi");
  expect(uscita(TRE, { mostrato: A, salvaDaSolo: true })).toBe("salva");
  // Verso un altro Tape vale lo stesso.
  expect(uscita(TRE, { mostrato: A, salvaDaSolo: false, verso: B })).toBe(
    "chiedi"
  );
});

test("niente dialog senza bozze nel Tape mostrato, o se si riapre lo stesso Tape", () => {
  const C = "D:\\Memotape\\intervista.tape";
  expect(uscita(TRE, { mostrato: C, salvaDaSolo: false })).toBe("vai");
  // Le bozze di un altro Tape non riguardano la vista mostrata (Home, Libreria, un file).
  expect(uscita(TRE, { mostrato: null, salvaDaSolo: false })).toBe("vai");
  expect(uscita(TRE, { mostrato: A, salvaDaSolo: false, verso: A })).toBe(
    "vai"
  );
  expect(uscita({}, { mostrato: A, salvaDaSolo: false })).toBe("vai");
});
