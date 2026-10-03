import { expect, test } from "bun:test";
import {
  afterPhrase,
  appendPhrase,
  withPartial,
} from "@/features/transcription/phrases";

test("le Frasi finiscono una per riga, nell'ordine di arrivo", () => {
  const text = ["Buongiorno a tutti.", "Oggi parliamo di trascrizione."].reduce(
    appendPhrase,
    ""
  );
  expect(text).toBe("Buongiorno a tutti.\nOggi parliamo di trascrizione.");
});

test("il Parziale occupa la riga in corso e la Frase con lo stesso id lo fissa", () => {
  const text = "Buongiorno a tutti.";
  let partial = { phraseId: 1, text: "Oggi parl" };
  expect(withPartial(text, partial)).toBe("Buongiorno a tutti.\nOggi parl");
  partial = { phraseId: 1, text: "Oggi parliamo di" };
  expect(withPartial(text, partial)).toBe(
    "Buongiorno a tutti.\nOggi parliamo di"
  );
  // Arriva la Frase 1: la riga diventa definitiva e il Parziale sparisce.
  const fixed = appendPhrase(text, "Oggi parliamo di trascrizione.");
  expect(withPartial(fixed, afterPhrase(partial, 1))).toBe(
    "Buongiorno a tutti.\nOggi parliamo di trascrizione."
  );
  // Un Parziale vuoto o di un'altra Frase non aggiunge e non toglie righe.
  expect(withPartial(text, { phraseId: 1, text: "" })).toBe(text);
  expect(afterPhrase(partial, 0)).toBe(partial);
  expect(withPartial("", { phraseId: 0, text: "Buon" })).toBe("Buon");
});
