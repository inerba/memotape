import { expect, test } from "bun:test";
import { type Bozze, bozze, bozzeDaSalvare, bozzeDi, uscita } from "./bozze";

const A = "D:\\Memotape\\riunione.tape";

/** Le bozze dopo aver scritto `testo` nelle Note di `path`, con `salvato` su disco (`""`). */
function scritte(
  stato: Bozze,
  path: string,
  testi: Record<string, string>,
  salvato = ""
): Bozze {
  return Object.entries(testi).reduce(
    (s, [nome, testo]) =>
      bozze(s, { nome, path, salvato, testo, type: "scritta" }),
    stato
  );
}

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
  expect(stato).toEqual({});
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
const TRE: Bozze = scritte(scritte({}, A, { Domande: "?", Note: "x" }), B, {
  Note: "y",
});

test("la bozza segue la Nota rinominata e sparisce con la Nota eliminata", () => {
  expect(
    bozzeDi(
      bozze(TRE, { nome: "Note", nuovo: "Mie", path: A, type: "rinominata" }),
      A
    )
  ).toEqual({ Domande: "?", Mie: "x" });
  expect(
    bozzeDi(bozze(TRE, { nome: "Note", path: A, type: "eliminata" }), A)
  ).toEqual({ Domande: "?" });
});

test("Non salvare scarta le bozze del Tape, non quelle degli altri", () => {
  const stato = bozze(TRE, { path: A, type: "scartate" });
  expect(Object.keys(stato)).toEqual([B]);
  expect(bozzeDi(stato, B)).toEqual({ Note: "y" });
});

test("le bozze seguono il Tape spostato e spariscono con il Tape che non c'è più", () => {
  const C = "D:\\Memotape\\Lavoro\\riunione.tape";
  const spostato = bozze(TRE, { from: A, to: C, type: "tapeMoved" });
  expect(Object.keys(spostato).sort()).toEqual([B, C].sort());
  expect(bozzeDi(spostato, C)).toEqual({ Domande: "?", Note: "x" });
  const via = bozze(TRE, { path: B, type: "tapeGone" });
  expect(Object.keys(via)).toEqual([A]);
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

test("chiudere la finestra considera le bozze di ogni Tape, anche di uno non più mostrato", () => {
  // Un'Attività finita ha chiuso il Tape consultato `B`, che aveva bozze.
  const soloB = scritte({}, B, { Note: "y" });
  const C = "D:\\Memotape\\intervista.tape";
  for (const mostrato of [null, C]) {
    expect(
      uscita(soloB, { chiusura: true, mostrato, salvaDaSolo: false })
    ).toBe("chiedi");
    expect(uscita(soloB, { chiusura: true, mostrato, salvaDaSolo: true })).toBe(
      "salva"
    );
  }
  expect(
    uscita({}, { chiusura: true, mostrato: null, salvaDaSolo: false })
  ).toBe("vai");
});

test("le bozze dei Tape non mostrati si salvano dal testo su disco da cui partono", () => {
  let stato = scritte(TRE, B, { Domande: "chi?" }, "");
  stato = bozze(stato, {
    nome: "Note",
    path: B,
    salvato: "su disco",
    testo: "y!",
    type: "scritta",
  });
  // Salvato «y!» mentre intanto si scriveva «y!!»: si riparte da «y!».
  stato = bozze(stato, {
    nome: "Note",
    path: B,
    salvato: "su disco",
    testo: "y!!",
    type: "scritta",
  });
  stato = bozze(stato, { nome: "Note", path: B, testo: "y!", type: "salvata" });
  // Le bozze di `A` le salva la sua vista.
  expect(bozzeDaSalvare(stato, A)).toEqual([
    { atteso: "y!", nome: "Note", path: B, testo: "y!!" },
    { atteso: "", nome: "Domande", path: B, testo: "chi?" },
  ]);
  expect(bozzeDaSalvare(stato, null).map((b) => b.path)).toEqual([A, A, B, B]);
});
