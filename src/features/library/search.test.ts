import { expect, test } from "bun:test";
import { markedParts } from "@/features/library/search";

test("l'estratto si divide in parti, con le parole trovate segnate", () => {
  expect(markedParts("Il \u0001preventivo\u0002 di \u0001Acme\u0002.")).toEqual(
    [
      { mark: false, text: "Il " },
      { mark: true, text: "preventivo" },
      { mark: false, text: " di " },
      { mark: true, text: "Acme" },
      { mark: false, text: "." },
    ]
  );
  expect(markedParts("\u0001Tutto\u0002")).toEqual([
    { mark: true, text: "Tutto" },
  ]);
  expect(markedParts("Niente")).toEqual([{ mark: false, text: "Niente" }]);
  expect(markedParts("")).toEqual([]);
});
