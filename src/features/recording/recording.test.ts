import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import {
  activityText,
  afterRecording,
  elapsedText,
  levelPercent,
  meters,
  silentLevels,
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

test("c'è un indicatore per ogni ingresso registrato, prima il microfono", () => {
  expect(meters({ microphone: 1, system: null })).toEqual([
    { input: "microphone", percent: 100 },
  ]);
  expect(meters({ microphone: null, system: 0 })).toEqual([
    { input: "system", percent: 0 },
  ]);
  expect(meters({ microphone: 0, system: 1 })).toEqual([
    { input: "microphone", percent: 0 },
    { input: "system", percent: 100 },
  ]);
});

test("prima del primo recording-tick gli indicatori seguono la sorgente di registrazione delle impostazioni", () => {
  expect(meters(silentLevels("mic")).map((m) => m.input)).toEqual([
    "microphone",
  ]);
  expect(meters(silentLevels("system")).map((m) => m.input)).toEqual([
    "system",
  ]);
  expect(meters(silentLevels("both")).map((m) => m.input)).toEqual([
    "microphone",
    "system",
  ]);
});

test("senza dispositivo di uscita la Registrazione non parte con un errore dedicato", () => {
  const after = afterRecording({
    error: { code: "outputDeviceMissing" },
    status: "error",
  });
  expect(after.source).toBeUndefined();
  expect(statusText(after.status, t)).toBe(
    "Nessun dispositivo di uscita per l'audio di sistema, o quello scelto in Impostazioni non è disponibile"
  );
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
  const after = afterRecording({
    data: { error: null, path, transcription: null },
    status: "ok",
  });
  expect(after.source).toBe(path);
  expect(statusText(after.status, t)).toBe(`Registrazione salvata in ${path}`);
});

test("un dispositivo scollegato salva comunque e mostra l'errore con il suo nome", () => {
  const path = "D:\\Reg\\Registrazione 2026-10-03 10-00-00.ogg";
  const after = afterRecording({
    data: {
      error: { code: "deviceDisconnected", detail: "Microfono USB" },
      path,
      transcription: { outcome: "saved" },
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

test("con la Trascrizione dal vivo la status bar dice dov'è il testo salvato", () => {
  const path = String.raw`D:\Reg\Registrazione 2026-10-03 10-00-00.bino`;
  const after = afterRecording({
    data: { error: null, path, transcription: { outcome: "saved" } },
    status: "ok",
  });
  expect(after.source).toBe(path);
  expect(statusText(after.status, t)).toBe(`Trascrizione salvata in ${path}`);
  const silent = afterRecording({
    data: { error: null, path, transcription: { outcome: "noSpeech" } },
    status: "ok",
  });
  expect(silent.status).toEqual({ phase: "noSpeech" });
});

test("annullare il completamento della trascrizione salva comunque la Registrazione", () => {
  const path = "D:\\Reg\\Registrazione 2026-10-03 10-00-00.ogg";
  const cancelled = afterRecording({
    data: {
      error: null,
      path,
      transcription: { error: { code: "cancelled" }, outcome: "failed" },
    },
    status: "ok",
  });
  expect(cancelled.source).toBe(path);
  expect(cancelled.status).toEqual({ phase: "cancelled" });
  const missing = afterRecording({
    data: {
      error: null,
      path,
      transcription: {
        error: { code: "liveTranscriptionUnavailable", detail: "Nemotron" },
        outcome: "failed",
      },
    },
    status: "ok",
  });
  expect(missing.source).toBe(path);
  expect(missing.status.phase).toBe("failed");
});

test("l'Attività in corso dice il timer o la fase con la percentuale", () => {
  expect(activityText({ paused: false, phase: "recording" }, 83_000, t)).toBe(
    "Registrazione · 1:23"
  );
  expect(activityText({ percent: 45, phase: "transcribing" }, 0, t)).toBe(
    "Trascrizione in corso… 45%"
  );
  expect(activityText({ percent: null, phase: "completing" }, 0, t)).toBe(
    "Completamento della trascrizione…"
  );
  expect(activityText({ phase: "idle", source: null }, 0, t)).toBeNull();
  expect(activityText({ phase: "cancelled" }, 0, t)).toBeNull();
});
