import { expect, test } from "bun:test";
import type { TranscriptPhrase } from "@/bindings";
import { diarizationNeedsConfirmation, wasDiarized } from "./diarization";

const phrase: TranscriptPhrase = {
  fineMs: 1000,
  ingresso: "mix",
  inizioMs: 0,
  parlante: null,
  parlanteNonDeterminato: false,
  parlanteProvvisorio: false,
  phraseId: 0,
  sessionId: null,
  text: "Testo corretto.",
};

test("il nome della prima azione distingue anche i Tape diarizzati senza metadati", () => {
  expect(wasDiarized(null, [phrase])).toBe(false);
  expect(wasDiarized(null, [{ ...phrase, parlante: 1 }])).toBe(true);
  expect(wasDiarized(null, [{ ...phrase, parlanteNonDeterminato: true }])).toBe(
    true
  );
});

test("si avvisa per ogni correzione salvata, anche per nomi nei Tape precedenti", () => {
  expect(diarizationNeedsConfirmation(null, {})).toBe(false);
  expect(diarizationNeedsConfirmation({ correttoAMano: false }, {})).toBe(
    false
  );
  expect(diarizationNeedsConfirmation({ correttoAMano: true }, {})).toBe(true);
  for (const key of ["mix:1", "microfono", "sistema:2"]) {
    expect(diarizationNeedsConfirmation(null, { [key]: "Mario" })).toBe(true);
  }
});
