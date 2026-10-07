import { expect, test } from "bun:test";
import i18n from "i18next";
import "@/lib/i18n";
import type { TapeEntry } from "@/bindings";
import { homeTapes, recentTitle } from "./recent-tapes";

function tape(titolo: string, creato = "2026-10-06T17:14:00+02:00"): TapeEntry {
  return {
    creato,
    durataMs: 0,
    path: `D:\\Tape\\${titolo}.tape`,
    raccolta: null,
    titolo,
  };
}
const t = i18n.getFixedT("it");

test("la Home distingue l'ultimo aperto dal più recente e ignora file mancanti o illeggibili", () => {
  const old = tape("vecchio");
  const latest = tape("nuovo", "2026-10-06T23:20:00+02:00");
  const broken = {
    ...tape("rotto", "2026-10-07T12:00:00+02:00"),
    durataMs: null,
  };
  const list = [old, broken, latest];
  expect(homeTapes(list, old.path.toUpperCase())).toEqual({
    featured: old,
    recent: [broken, latest, old],
    resumed: true,
  });
  expect(homeTapes(list, "D:\\altro\\fuori.tape").featured).toBe(latest);
  expect(homeTapes(list, broken.path).resumed).toBe(false);
  expect(homeTapes([], old.path)).toEqual({
    featured: null,
    recent: [],
    resumed: false,
  });
  expect(homeTapes([broken], null).featured).toBeNull();
  expect(list).toEqual([old, broken, latest]);
});

test("abbrevia solo nomi automatici completi, anche creati in un'altra lingua", () => {
  for (const prefix of [
    "Registrazione",
    "Recording",
    "Aufnahme",
    "Grabación",
    "Enregistrement",
    "Nagranie",
  ]) {
    const entry = tape(`${prefix} 2026-10-06 17-14-23`);
    expect(recentTitle(entry, [entry], t)).toBe("Registrazione delle 17:14");
    expect(entry.titolo).toBe(`${prefix} 2026-10-06 17-14-23`);
  }
  for (const title of [
    "restaurant_noisy",
    "Registrazione intervista",
    "Registrazione 2026-10-06 17-14-23 corretta",
  ]) {
    expect(recentTitle(tape(title), [], t)).toBe(title);
  }
});

test("distingue registrazioni nello stesso minuto e copie numerate", () => {
  const first = tape("Registrazione 2026-10-06 22-18-04");
  const second = tape("Recording 2026-10-06 22-18-08");
  const copy = tape("Registrazione 2026-10-06 22-18-04 2");
  const list = [first, second, copy];
  expect(recentTitle(first, list, t)).toBe("Registrazione delle 22:18:04");
  expect(recentTitle(second, list, t)).toBe("Registrazione delle 22:18:08");
  expect(recentTitle(copy, list, t)).toBe("Registrazione delle 22:18:04 2");
});
