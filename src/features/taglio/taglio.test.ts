import { expect, test } from "bun:test";
import type { Ingresso, TranscriptPhrase } from "@/bindings";
import {
  conTaglio,
  correzioniDivise,
  durataTagliata,
  silenziDi,
  tagliato,
  taglioDelleFrasi,
  taglioDelSilenzio,
} from "@/features/taglio/taglio";

function frase(
  phraseId: number,
  inizioMs: number,
  fineMs: number,
  extra: Partial<TranscriptPhrase> = {}
): TranscriptPhrase {
  return {
    fineMs,
    ingresso: "mix" as Ingresso,
    inizioMs,
    parlante: null,
    phraseId,
    text: `Frase ${phraseId}.`,
    ...extra,
  };
}

// Saluti, la riunione, una pausa caffè di 7 minuti, la chiusura e 2 minuti di stanza vuota.
const riunione = [
  frase(0, 4000, 9000),
  frase(1, 12_000, 19_000),
  frase(2, 164_000, 170_000),
  frase(3, 171_000, 180_000),
  frase(4, 645_000, 650_000),
  frase(5, 650_000, 655_000),
];
const DURATA = 800_000;

test("i saluti si tagliano dall'inizio del Tape a 1 s prima della Frase che resta", () => {
  expect(
    taglioDelleFrasi(riunione, new Set(["mix:0", "mix:1"]), DURATA)
  ).toEqual({
    fineMs: 163_000,
    inizioMs: 0,
  });
});

test("le ultime Frasi si tagliano fino alla fine del Tape, lasciando 1 s dopo l'ultima che resta", () => {
  expect(taglioDelleFrasi(riunione, new Set(["mix:5"]), DURATA)).toEqual({
    fineMs: DURATA,
    inizioMs: 650_000,
  });
  expect(
    taglioDelleFrasi(riunione, new Set(["mix:4", "mix:5"]), DURATA)
  ).toEqual({
    fineMs: DURATA,
    inizioMs: 181_000,
  });
});

test("una Frase in mezzo si taglia con i silenzi intorno, a metà se sono corti", () => {
  // Prima 1 s di silenzio: a metà; dopo 465 s: 1 s prima della Frase 4.
  expect(taglioDelleFrasi(riunione, new Set(["mix:3"]), DURATA)).toEqual({
    fineMs: 644_000,
    inizioMs: 170_500,
  });
  expect(taglioDelleFrasi(riunione, new Set(), DURATA)).toBeNull();
});

test("una Frase di un altro Ingresso che parla sopra non viene tagliata", () => {
  const phrases = [
    frase(0, 0, 10_000, { ingresso: "microfono" }),
    frase(0, 8000, 20_000, { ingresso: "sistema" }),
    frase(1, 30_000, 40_000, { ingresso: "microfono" }),
  ];
  const taglio = taglioDelleFrasi(phrases, new Set(["sistema:0"]), 50_000);
  expect(taglio).toEqual({ fineMs: 29_000, inizioMs: 8000 });
  expect(tagliato(taglio ? [taglio] : [], phrases[0] as TranscriptPhrase)).toBe(
    false
  );
});

test("scegliere il testo di un Turno corretto taglia tutte le sue Frasi", () => {
  const phrases = [
    frase(0, 0, 5000),
    frase(1, 10_000, 15_000, { testoTurno: 1 }),
    frase(2, 15_000, 20_000, { testoTurno: 1, text: "" }),
    frase(3, 30_000, 35_000),
  ];
  expect(taglioDelleFrasi(phrases, new Set(["mix:1"]), 40_000)).toEqual({
    fineMs: 29_000,
    inizioMs: 6000,
  });
});

test("i silenzi si tagliano lasciando 1 s per lato, interi all'inizio e alla fine", () => {
  expect(taglioDelSilenzio(0, 4000, DURATA)).toEqual({
    fineMs: 3000,
    inizioMs: 0,
  });
  expect(taglioDelSilenzio(180_000, 645_000, DURATA)).toEqual({
    fineMs: 644_000,
    inizioMs: 181_000,
  });
  expect(taglioDelSilenzio(655_000, DURATA, DURATA)).toEqual({
    fineMs: DURATA,
    inizioMs: 656_000,
  });
});

test("in modalità Taglio i silenzi lunghi sono all'inizio, tra i turni e alla fine", () => {
  const turno = (items: TranscriptPhrase[]) => ({
    ingresso: "mix" as Ingresso,
    items,
    key: String(items[0]?.phraseId),
    label: null,
    name: null,
    parlante: null,
  });
  const turns = [
    turno(riunione.slice(0, 2)),
    turno(riunione.slice(2, 4)),
    turno(riunione.slice(4)),
  ];
  const { fine, inizio, prima } = silenziDi(
    turns,
    [null, 146_000, 466_000],
    DURATA
  );
  expect(inizio).toBeNull();
  expect(prima).toEqual([
    null,
    { a: 19_000, b: 164_000, dove: "pausa", ms: 146_000 },
    { a: 180_000, b: 645_000, dove: "pausa", ms: 466_000 },
  ]);
  expect(fine).toEqual({ a: 655_000, b: DURATA, dove: "fine", ms: 145_000 });
});

test("i Tagli si uniscono dove si toccano e ne contano la durata", () => {
  const tagli = conTaglio(
    conTaglio([{ fineMs: 200, inizioMs: 100 }], { fineMs: 50, inizioMs: 0 }),
    { fineMs: 300, inizioMs: 150 }
  );
  expect(tagli).toEqual([
    { fineMs: 50, inizioMs: 0 },
    { fineMs: 300, inizioMs: 100 },
  ]);
  expect(durataTagliata(tagli)).toBe(250);
});

test("la conferma avvisa quando un Taglio divide un Turno corretto", () => {
  const phrases = [
    frase(1, 10_000, 15_000, { testoTurno: 1 }),
    frase(2, 15_000, 20_000, { testoTurno: 1 }),
  ];
  expect(correzioniDivise(phrases, [{ fineMs: 15_000, inizioMs: 9000 }])).toBe(
    true
  );
  expect(correzioniDivise(phrases, [{ fineMs: 21_000, inizioMs: 9000 }])).toBe(
    false
  );
});
