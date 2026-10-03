import { expect, test } from "bun:test";
import { appendPhrase } from "@/features/transcription/phrases";

test("le Frasi finiscono una per riga, nell'ordine di arrivo", () => {
  const text = ["Buongiorno a tutti.", "Oggi parliamo di trascrizione."].reduce(
    appendPhrase,
    ""
  );
  expect(text).toBe("Buongiorno a tutti.\nOggi parliamo di trascrizione.");
});
