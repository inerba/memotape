import { describe, expect, test } from "bun:test";
import { lasciaMenuNativo } from "@/lib/context-menu";

function elemento(campo: boolean, isContentEditable = false) {
  return {
    closest: () => (campo ? {} : null),
    isContentEditable,
  } as unknown as EventTarget;
}

describe("lasciaMenuNativo", () => {
  test("nei campi resta il menu della WebView", () => {
    expect(lasciaMenuNativo(elemento(true), "")).toBe(true);
    expect(lasciaMenuNativo(elemento(false, true), "")).toBe(true);
  });

  test("con del testo selezionato resta, per Copia", () => {
    expect(lasciaMenuNativo(elemento(false), "ciao")).toBe(true);
    expect(lasciaMenuNativo(null, "ciao")).toBe(true);
  });

  test("altrove no", () => {
    expect(lasciaMenuNativo(elemento(false), "")).toBe(false);
    expect(lasciaMenuNativo(null, "")).toBe(false);
  });
});
