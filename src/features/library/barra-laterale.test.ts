import { expect, test } from "bun:test";
import {
  barraAperta,
  ICONE_DELLA_STRISCIA,
  riapertura,
  ricordaBarra,
} from "@/features/library/barra-laterale";

test("Cerca e Recenti riaprono la barra, Cerca con il cursore nel campo", () => {
  expect(riapertura("cerca")).toBe("ricerca");
  expect(riapertura("recenti")).toBe("recenti");
});

test("le altre icone della striscia agiscono subito, a barra chiusa", () => {
  const subito = ICONE_DELLA_STRISCIA.filter((i) => riapertura(i) === null);
  expect(subito).toEqual([
    "home",
    "nuovaRegistrazione",
    "importa",
    "libreria",
    "impostazioni",
  ]);
});

/** Una memoria del browser finta: una mappa. */
function memoria() {
  const valori = new Map<string, string>();
  return {
    getItem: (chiave: string) => valori.get(chiave) ?? null,
    setItem: (chiave: string, valore: string) => {
      valori.set(chiave, valore);
    },
  };
}

const guasta = () => {
  throw new Error("SecurityError");
};

test("la barra laterale parte aperta e ricorda la scelta", () => {
  const m = memoria();
  expect(barraAperta(() => m)).toBe(true);
  ricordaBarra(() => m, false);
  expect(barraAperta(() => m)).toBe(false);
  ricordaBarra(() => m, true);
  expect(barraAperta(() => m)).toBe(true);
});

test("senza memoria del browser la barra resta aperta e salvare non fallisce", () => {
  expect(barraAperta(guasta)).toBe(true);
  expect(() => ricordaBarra(guasta, false)).not.toThrow();
  const piena = {
    getItem: () => null,
    setItem: () => {
      throw new Error("QuotaExceededError");
    },
  };
  expect(() => ricordaBarra(() => piena, false)).not.toThrow();
});
