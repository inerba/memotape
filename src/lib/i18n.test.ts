import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import de from "@/locales/de.json";
import en from "@/locales/en.json";
import es from "@/locales/es.json";
import fr from "@/locales/fr.json";
import it from "@/locales/it.json";
import pl from "@/locales/pl.json";

const LOCALES = { de, en, es, fr, it, pl };
const PLURAL = /_(zero|one|two|few|many|other)$/;

/** Le chiavi di un file di traduzione, con i punti: `status.finished_one`. */
function flatten(messages: object, prefix = ""): Record<string, string> {
  return Object.fromEntries(
    Object.entries(messages).flatMap(([key, value]) =>
      typeof value === "string"
        ? [[`${prefix}${key}`, value]]
        : Object.entries(flatten(value, `${prefix}${key}.`))
    )
  );
}

/** Le variabili interpolate di un testo: `{{count, number}}` → `count`. */
function variables(text: string): string[] {
  return [...text.matchAll(/\{\{\s*(\w+)/g)].map((m) => m[1] ?? "").sort();
}

const reference = flatten(it);

test("ogni lingua ha le chiavi dell'italiano, con le forme plurali della lingua", () => {
  const bases = new Set(
    Object.keys(reference).map((k) => k.replace(PLURAL, ""))
  );
  const plurals = new Set(
    Object.keys(reference)
      .filter((k) => PLURAL.test(k))
      .map((k) => k.replace(PLURAL, ""))
  );
  for (const [lng, messages] of Object.entries(LOCALES)) {
    const categories = new Intl.PluralRules(lng).resolvedOptions()
      .pluralCategories;
    const expected = [...bases].flatMap((key) =>
      plurals.has(key) ? categories.map((c) => `${key}_${c}`) : [key]
    );
    const keys = flatten(messages);
    expect([lng, Object.keys(keys).sort()]).toEqual([lng, expected.sort()]);
    for (const [key, text] of Object.entries(keys)) {
      const base = key.replace(PLURAL, "");
      const original =
        reference[key] ?? reference[`${base}_other`] ?? reference[base] ?? "";
      expect([lng, key, text.trim() === ""]).toEqual([lng, key, false]);
      expect([lng, key, variables(text)]).toEqual([
        lng,
        key,
        variables(original),
      ]);
    }
  }
});

test("il polacco usa le sue forme plurali", () => {
  const finished = (count: number) =>
    i18n.t("status.finished", { count, lng: "pl", path: "a.txt" });
  const fill = (text: string, count: number) =>
    text
      .replace("{{count, number}}", String(count))
      .replace("{{path}}", "a.txt");
  // 1 → one, 2 e 22 → few, 5 e 12 → many (CLDR).
  expect(finished(1)).toBe(fill(pl.status.finished_one, 1));
  expect(finished(2)).toBe(fill(pl.status.finished_few, 2));
  expect(finished(22)).toBe(fill(pl.status.finished_few, 22));
  expect(finished(5)).toBe(fill(pl.status.finished_many, 5));
  expect(finished(12)).toBe(fill(pl.status.finished_many, 12));
});

test("l'interfaccia parte in italiano e interpola senza escape", () => {
  expect(i18n.language).toBe("it");
  expect(i18n.t("about.version", { version: "0.1.0 <dev>" })).toBe(
    "Versione 0.1.0 <dev>"
  );
});
