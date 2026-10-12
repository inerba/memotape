import { describe, expect, test } from "bun:test";
import { lasciaMenuNativo, tastoDelBrowser } from "@/lib/webview";

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

function tasto(
  key: string,
  mod: { alt?: boolean; ctrl?: boolean; shift?: boolean } = {}
) {
  return {
    altKey: mod.alt ?? false,
    ctrlKey: mod.ctrl ?? false,
    key,
    shiftKey: mod.shift ?? false,
  };
}

describe("tastoDelBrowser", () => {
  test("ricarica e cronologia sono del browser", () => {
    expect(tastoDelBrowser(tasto("F5"))).toBe(true);
    expect(tastoDelBrowser(tasto("F5", { ctrl: true }))).toBe(true);
    expect(tastoDelBrowser(tasto("r", { ctrl: true }))).toBe(true);
    expect(tastoDelBrowser(tasto("R", { ctrl: true, shift: true }))).toBe(true);
    expect(tastoDelBrowser(tasto("ArrowLeft", { alt: true }))).toBe(true);
    expect(tastoDelBrowser(tasto("ArrowRight", { alt: true }))).toBe(true);
    expect(tastoDelBrowser(tasto("BrowserBack"))).toBe(true);
    expect(tastoDelBrowser(tasto("BrowserForward"))).toBe(true);
    expect(tastoDelBrowser(tasto("BrowserRefresh"))).toBe(true);
  });

  test("gli altri tasti restano: frecce del player, Ctrl+F, Copia, R scritta", () => {
    expect(tastoDelBrowser(tasto("ArrowLeft"))).toBe(false);
    expect(tastoDelBrowser(tasto("ArrowLeft", { alt: true, ctrl: true }))).toBe(
      false
    );
    expect(tastoDelBrowser(tasto("f", { ctrl: true }))).toBe(false);
    expect(tastoDelBrowser(tasto("c", { ctrl: true }))).toBe(false);
    expect(tastoDelBrowser(tasto("r"))).toBe(false);
    expect(tastoDelBrowser(tasto("F2"))).toBe(false);
  });
});
