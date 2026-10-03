import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import {
  afterRecording,
  elapsedText,
  levelPercent,
} from "@/features/recording/recording";
import { statusText } from "@/features/status/status";

const t = i18n.t.bind(i18n);

test("il timer mostra minuti e secondi, e le ore solo quando servono", () => {
  expect(elapsedText(0)).toBe("0:00");
  expect(elapsedText(5999)).toBe("0:05");
  expect(elapsedText(754_000)).toBe("12:34");
  expect(elapsedText(3_723_000)).toBe("1:02:03");
});

test("il livello va da 0 a 100 su una scala in decibel", () => {
  expect(levelPercent(0)).toBe(0);
  expect(levelPercent(null)).toBe(0);
  // -60 dBFS e sotto: nulla; 0 dBFS: pieno; -30 dBFS: a metà.
  expect(levelPercent(0.0005)).toBe(0);
  expect(levelPercent(1)).toBe(100);
  expect(levelPercent(10 ** (-30 / 20))).toBe(50);
});

test("durante la Registrazione la status bar dice se è in pausa", () => {
  expect(statusText({ paused: false, phase: "recording" }, t)).toBe(
    "Registrazione in corso…"
  );
  expect(statusText({ paused: true, phase: "recording" }, t)).toBe(
    "Registrazione in pausa"
  );
});

test("dopo Stop il file diventa la Sorgente e la status bar dice dov'è", () => {
  const path =
    "C:\\Users\\me\\Documents\\Sbobino\\Registrazione 2026-10-03 10-00-00.ogg";
  const after = afterRecording({ data: { error: null, path }, status: "ok" });
  expect(after.source).toBe(path);
  expect(statusText(after.status, t)).toBe(`Registrazione salvata in ${path}`);
});

test("un dispositivo scollegato salva comunque e mostra l'errore con il suo nome", () => {
  const path = "D:\\Reg\\Registrazione 2026-10-03 10-00-00.ogg";
  const after = afterRecording({
    data: {
      error: { code: "deviceDisconnected", detail: "Microfono USB" },
      path,
    },
    status: "ok",
  });
  expect(after.source).toBe(path);
  expect(after.status.phase).toBe("failed");
  expect(statusText(after.status, t)).toBe(
    "Il dispositivo Microfono USB non è più disponibile (scollegato o cambiato in Windows): la Registrazione è stata fermata e salvata"
  );
});

test("una Registrazione che non parte lascia la Sorgente com'era", () => {
  const after = afterRecording({
    error: { code: "microphoneMissing" },
    status: "error",
  });
  expect(after.source).toBeUndefined();
  expect(statusText(after.status, t)).toBe(
    "Nessun microfono collegato, o quello scelto in Impostazioni non è disponibile"
  );
});
