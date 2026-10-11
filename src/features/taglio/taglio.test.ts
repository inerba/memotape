import { expect, test } from "bun:test";
import type { Ingresso, TranscriptPhrase } from "@/bindings";
import { tagliato, taglioDelTurno } from "@/features/taglio/taglio";
import type { Turn } from "@/features/transcription/phrases";

function frase(
  phraseId: number,
  inizioMs: number,
  fineMs: number,
  ingresso: Ingresso = "mix"
): TranscriptPhrase {
  return {
    fineMs,
    ingresso,
    inizioMs,
    parlante: null,
    phraseId,
    text: `Frase ${phraseId}.`,
  };
}

function turno(...items: TranscriptPhrase[]): Turn {
  return {
    ingresso: items[0]?.ingresso ?? "mix",
    items,
    key: String(items[0]?.phraseId),
    label: null,
    name: null,
    parlante: null,
  };
}

// Saluti, la riunione e 2 minuti di stanza vuota alla fine.
const saluti = [frase(0, 4000, 9000), frase(1, 12_000, 19_000)];
const riunione = [frase(2, 164_000, 170_000), frase(3, 171_000, 180_000)];
const fine = [frase(4, 645_000, 655_000)];
const tutte = [...saluti, ...riunione, ...fine];
const DURATA = 800_000;

test("prima di un Turno si taglia dall'inizio del Tape a 1 s prima del suo parlato", () => {
  expect(taglioDelTurno(tutte, turno(...riunione), "prima", DURATA)).toEqual({
    fineMs: 163_000,
    inizioMs: 0,
  });
});

test("dopo l'ultimo Turno si taglia solo il tempo vuoto, da 1 s dopo fino alla fine", () => {
  expect(taglioDelTurno(tutte, turno(...fine), "dopo", DURATA)).toEqual({
    fineMs: DURATA,
    inizioMs: 656_000,
  });
});

test("dopo un Turno in mezzo si tagliano anche i Turni che seguono", () => {
  const taglio = taglioDelTurno(tutte, turno(...riunione), "dopo", DURATA);
  expect(taglio).toEqual({ fineMs: DURATA, inizioMs: 181_000 });
});

test("con meno di 1 s di silenzio il bordo cade subito dopo la Frase tolta", () => {
  const phrases = [frase(0, 0, 9600), frase(1, 10_000, 15_000)];
  expect(
    taglioDelTurno(
      phrases,
      turno(phrases[1] as TranscriptPhrase),
      "prima",
      20_000
    )
  ).toEqual({ fineMs: 9600, inizioMs: 0 });
});

test("non c'è niente da tagliare prima di un Turno che comincia all'inizio del Tape", () => {
  expect(
    taglioDelTurno(saluti, turno(frase(0, 0, 9000)), "prima", DURATA)
  ).toBeNull();
});

test("non c'è niente da tagliare dopo un Turno che arriva alla fine del Tape", () => {
  expect(
    taglioDelTurno(tutte, turno(frase(4, 645_000, DURATA)), "dopo", DURATA)
  ).toBeNull();
});

test("una voce di un altro Ingresso accavallata al Turno resta intera e sposta il bordo", () => {
  const sopra = frase(0, 150_000, 166_000, "sistema");
  const phrases = [frase(0, 4000, 9000, "microfono"), sopra, ...riunione];
  const taglio = taglioDelTurno(phrases, turno(...riunione), "prima", DURATA);
  expect(taglio && tagliato(taglio, sopra)).toBe(false);
});

test("il bordo spostato da una voce accavallata non entra nella Frase tolta prima di lei", () => {
  const phrases = [
    frase(0, 4000, 9000, "microfono"),
    frase(0, 150_000, 166_000, "sistema"),
    ...riunione,
  ];
  expect(taglioDelTurno(phrases, turno(...riunione), "prima", DURATA)).toEqual({
    fineMs: 149_000,
    inizioMs: 0,
  });
});

test("una Frase è tagliata solo se sta tutta dentro il Taglio", () => {
  const taglio = { fineMs: 163_000, inizioMs: 0 };
  expect([
    tagliato(taglio, frase(1, 12_000, 19_000)),
    tagliato(taglio, frase(2, 162_000, 170_000)),
  ]).toEqual([true, false]);
});
