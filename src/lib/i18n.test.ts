import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";

test("l'interfaccia è in italiano e interpola senza escape", () => {
  expect(i18n.language).toBe("it");
  expect(i18n.t("home.version", { version: "0.1.0 <dev>" })).toBe(
    "Versione 0.1.0 <dev>"
  );
});
