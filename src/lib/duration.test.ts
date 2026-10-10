import { expect, test } from "bun:test";
import { durationWords, pauseDuration } from "@/lib/duration";

test("la durata a parole: secondi sotto il minuto, minuti sotto l'ora, poi ore e minuti", () => {
  expect(durationWords(24_000, "it")).toBe("24 s");
  expect(durationWords(61_000, "it")).toBe("1 min");
  expect(durationWords((59 * 60 + 26) * 1000, "it")).toBe("59 min");
  expect(durationWords(((2 * 60 + 4) * 60 + 41) * 1000, "it")).toBe(
    "2 h 05 min"
  );
  // Arrotondati, 59,6 s e 59 min 40 s passano all'unità successiva.
  expect(durationWords(59_600, "it")).toBe("1 min");
  expect(durationWords((59 * 60 + 40) * 1000, "it")).toBe("1 h 00 min");
});

test("la durata della pausa: secondi sotto il minuto, poi minuti e secondi", () => {
  expect(pauseDuration(5000, "it")).toBe("5 s");
  expect(pauseDuration(9400, "it")).toBe("9 s");
  expect(pauseDuration(59_600, "it")).toBe("1 min");
  expect(pauseDuration(80_000, "it")).toBe("1 min 20 s");
  expect(pauseDuration(3_725_000, "it")).toBe("62 min 5 s");
});

test("la pausa usa le unità della lingua dell'interfaccia", () => {
  expect(pauseDuration(80_000, "en")).toBe("1 min 20 sec");
  expect(pauseDuration(9000, "de")).toBe("9 Sek.");
});
