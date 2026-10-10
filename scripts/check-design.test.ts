import { expect, test } from "bun:test";
import { violazioni } from "./check-design.ts";

test("i token e shadow-float passano", () => {
  expect(
    violazioni(
      'className="bg-background text-play shadow-float shadow-none focus-visible:ring-ring/50"'
    )
  ).toEqual([]);
});

test("un colore scritto nel codice è una violazione, in ogni forma", () => {
  const testi = violazioni(
    'a="bg-[#fff]" b={{ color: "rgb(0 0 0)" }} c="oklch(0.5 0.1 120)" d="#12345678"'
  ).map((v) => v.testo);
  expect(testi).toEqual(["#fff", "rgb(", "oklch(", "#12345678"]);
});

test("ogni ombra diversa da shadow-float è una violazione, con la sua posizione", () => {
  expect(
    violazioni('x\ny className="shadow-md shadow-[0_0_0_4px_red]"')
  ).toEqual([
    {
      colonna: 14,
      regola: "ombra diversa da shadow-float (The One Shadow Rule)",
      riga: 2,
      testo: "shadow-md",
    },
    {
      colonna: 24,
      regola: "ombra diversa da shadow-float (The One Shadow Rule)",
      riga: 2,
      testo: "shadow-[0_0_0_4px_red]",
    },
  ]);
});
