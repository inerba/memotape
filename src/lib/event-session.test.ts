import { expect, test } from "bun:test";
import {
  type Status,
  withDiarizing,
  withProgress,
} from "@/features/status/status";
import { inEventSession } from "@/lib/event-session";

test("progresso e analisi tardivi non completano la nuova Registrazione", () => {
  const sessionId = "second";
  let status: Status = { paused: false, phase: "recording" };
  for (const event of [
    { percent: null, sessionId: "first" },
    { percent: 100, sessionId: "first" },
    { percent: 100, sessionId: null },
  ]) {
    if (inEventSession(event, sessionId)) {
      status = withProgress(status, event.percent);
    }
  }
  if (inEventSession({ sessionId: "first" }, sessionId)) {
    status = withDiarizing(status);
  }
  expect(status).toEqual({ paused: false, phase: "recording" });
  const own = { percent: 10, sessionId };
  if (inEventSession(own, sessionId)) {
    status = withProgress(status, own.percent);
  }
  expect(status).toEqual({ percent: 10, phase: "completing" });
});

test("i tick della vecchia sessione non cambiano timer e livelli della nuova", () => {
  let tick = {
    elapsedMs: 0,
    levels: { microphone: 0, system: null as number | null },
    sessionId: "second",
  };
  const old = {
    ...tick,
    elapsedMs: 28_000,
    levels: { microphone: 1, system: null },
    sessionId: "first",
  };
  const next = {
    ...tick,
    elapsedMs: 100,
    levels: { microphone: 0.5, system: null },
  };
  for (const event of [old, next, old]) {
    if (inEventSession(event, "second")) {
      tick = event;
    }
  }
  expect(tick).toBe(next);
  // Il progresso di un file continua ad aggiornare la sua Trascrizione.
  expect(inEventSession({ sessionId: null }, null)).toBe(true);
});
