import { expect, test } from "bun:test";
import { withCleaningFailure } from "./cleaning";

test("l'avviso di pulizia appartiene alla sessione e conserva entrambi gli Ingressi", () => {
  const mic = {
    error: { code: "audioCleaningMissing" as const },
    ingresso: "microfono" as const,
    sessionId: "current",
  };
  const system = { ...mic, ingresso: "sistema" as const };
  expect(
    withCleaningFailure([], { ...mic, sessionId: "old" }, "current")
  ).toEqual([]);
  expect(withCleaningFailure([], mic, null)).toEqual([]);
  const first = withCleaningFailure([], mic, "current");
  expect(withCleaningFailure(first, mic, "current")).toEqual(first);
  expect(
    withCleaningFailure(first, system, "current").map((item) => item.ingresso)
  ).toEqual(["microfono", "sistema"]);
});
