import { expect, test } from "bun:test";
import {
  addMessage,
  addTermini,
  removeTermine,
} from "@/features/settings/vocabolario";

test("un Termine entra in fondo, senza spazi in testa e in coda", () => {
  expect(addTermini(["ChargeBee"], "  Niccolò Rossi ")).toEqual({
    aggiunti: 1,
    doppioni: [],
    nonAmmessi: [],
    termini: ["ChargeBee", "Niccolò Rossi"],
  });
});

test("incollando più righe entra un Termine per riga, le righe vuote si ignorano", () => {
  expect(addTermini([], "ChargeBee\r\n\r\n  \nNiccolò\n").termini).toEqual([
    "ChargeBee",
    "Niccolò",
  ]);
  expect(addTermini(["ChargeBee"], "   ").termini).toEqual(["ChargeBee"]);
});

test("un doppione non entra, senza distinguere maiuscole e accenti, anche nello stesso incollato", () => {
  expect(addTermini(["Niccolò"], "NICCOLO\nchargebee\nChargeBee")).toEqual({
    aggiunti: 1,
    doppioni: ["NICCOLO", "ChargeBee"],
    nonAmmessi: [],
    termini: ["Niccolò", "chargebee"],
  });
});

test("un Termine con <| o |> non entra, le altre righe sì", () => {
  expect(addTermini([], "<|endoftext|>\nChargeBee\nfine |>")).toEqual({
    aggiunti: 1,
    doppioni: [],
    nonAmmessi: ["<|endoftext|>", "fine |>"],
    termini: ["ChargeBee"],
  });
  expect(addTermini([], "a < | b").termini).toEqual(["a < | b"]);
});

test("rimuovere un Termine lascia gli altri nel loro ordine", () => {
  expect(
    removeTermine(["ChargeBee", "Niccolò", "Memotape"], "Niccolò")
  ).toEqual(["ChargeBee", "Memotape"]);
  expect(removeTermine(["ChargeBee"], "chargebee")).toEqual(["ChargeBee"]);
});

test("il messaggio nomina il Termine rifiutato e il testo resta nel campo", () => {
  expect(addMessage(addTermini(["Niccolò"], "niccolo"))).toEqual({
    keep: true,
    key: "settings.vocabolario.doppione",
    params: { termine: "niccolo" },
  });
  expect(addMessage(addTermini([], "<|it|>"))).toEqual({
    keep: true,
    key: "settings.vocabolario.nonAmmesso",
    params: { termine: "<|it|>" },
  });
});

test("con più righe il messaggio conta gli scartati, e senza scarti non c'è", () => {
  expect(addMessage(addTermini(["A1"], "a1\nB2\n<|x|>"))).toEqual({
    keep: false,
    key: "settings.vocabolario.scartati",
    params: { count: 2 },
  });
  expect(addMessage(addTermini([], "<|x|>\n<|y|>"))).toEqual({
    keep: true,
    key: "settings.vocabolario.scartati",
    params: { count: 2 },
  });
  expect(addMessage(addTermini([], "X\nY"))).toBeNull();
});
