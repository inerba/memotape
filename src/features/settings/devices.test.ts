import { expect, test } from "bun:test";
import { shortDeviceName } from "./devices";

test("il nome breve toglie il prefisso generico di Windows", () => {
  expect(shortDeviceName("Microfono (Anker PowerConf C200)")).toBe(
    "Anker PowerConf C200"
  );
  expect(shortDeviceName("Microfono (Realtek(R) Audio)")).toBe(
    "Realtek(R) Audio"
  );
  expect(shortDeviceName("Line In (Focusrite USB Audio)")).toBe(
    "Focusrite USB Audio"
  );
});

test("un nome senza prefisso resta com'è", () => {
  expect(shortDeviceName("WH-1000XM4")).toBe("WH-1000XM4");
  expect(shortDeviceName("Realtek(R) Audio")).toBe("Realtek(R) Audio");
  expect(shortDeviceName("()")).toBe("()");
});
