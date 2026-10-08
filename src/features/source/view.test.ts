import { expect, test } from "bun:test";
import type { OpenedTape } from "@/bindings";
import {
  INITIAL_VIEW,
  opening,
  pendingTape,
  type SourceViewAction,
  sourceView,
} from "@/features/source/view";

const opened = (text: string): OpenedTape => ({
  info: null as unknown as OpenedTape["info"],
  parlanti: {},
  phrases: [
    {
      fineMs: 900,
      ingresso: "mix",
      inizioMs: 0,
      parlante: null,
      phraseId: 0,
      text,
    },
  ],
});

const idle = { phase: "idle", source: null } as const;

const transcribing = { percent: null, phase: "transcribing" } as const;

const run = (actions: SourceViewAction[], from = INITIAL_VIEW) =>
  actions.reduce(sourceView, from);

test("un Tape spostato resta aperto con il percorso nuovo, come Sorgente e come Tape consultato", () => {
  const state = run([
    { path: "C:\\Lib\\a.tape", type: "sourceOpened" },
    { path: "C:\\Lib\\b.tape", tape: opened("B."), type: "tapeConsulted" },
    { from: "C:\\Lib", to: "C:\\Nuova", type: "tapeMoved" },
  ]);
  expect(state.source).toBe("C:\\Nuova\\a.tape");
  expect(state.consulted?.path).toBe("C:\\Nuova\\b.tape");
  expect(state.consulted?.conversation.phrases[0]?.text).toBe("B.");
});

test("un Tape nel Cestino si chiude se era aperto, come Sorgente o come Tape consultato", () => {
  const consulted = run([
    { path: "a.tape", type: "sourceOpened" },
    { path: "b.tape", tape: opened("B."), type: "tapeConsulted" },
    { path: "b.tape", type: "tapeTrashed" },
  ]);
  expect(consulted.source).toBe("a.tape");
  expect(consulted.consulted).toBeNull();
  const own = sourceView(consulted, { path: "a.tape", type: "tapeTrashed" });
  expect(own.source).toBeNull();
});

test("una modifica vale per il Tape consultato con quel percorso e solo per quello", () => {
  const change = (path: string): SourceViewAction => ({
    change: (c) => ({ ...c, parlanti: { "mix:1": "Anna" } }),
    path,
    type: "tapeChanged",
  });
  const state = run([
    { path: "b.tape", tape: opened("B."), type: "tapeConsulted" },
    change("altro.tape"),
  ]);
  expect(state.consulted?.conversation.parlanti).toEqual({});
  expect(
    sourceView(state, change("b.tape")).consulted?.conversation.parlanti
  ).toEqual({ "mix:1": "Anna" });
});

test("senza Attività un Tape diventa la Sorgente; dalla Libreria la Sorgente riparte dall'inizio", () => {
  const open = run([{ path: "a.tape", type: "sourceOpened" }]);
  expect(opening(INITIAL_VIEW, idle, "c.mp3", false)).toBe("source");
  expect(opening(open, idle, "a.tape", true)).toBe("restart");
  // Apri file sullo stesso Tape lo rilegge, senza ripartire.
  expect(opening(open, idle, "a.tape", false)).toBe("source");
});

test("durante un'Attività un Tape si consulta; la Sorgente di Trascrivi riporta alla sua vista", () => {
  const open = run([{ path: "a.tape", type: "sourceOpened" }]);
  expect(opening(open, transcribing, "b.tape", true)).toBe("consult");
  expect(opening(open, transcribing, "c.tape", false)).toBe("consult");
  expect(opening(open, transcribing, "a.tape", true)).toBe("activity");
  expect(opening(open, { phase: "diarizing" }, "a.tape", true)).toBe(
    "activity"
  );
  // Durante una Registrazione la Sorgente è un Tape come gli altri.
  const recording = { paused: false, phase: "recording" } as const;
  expect(opening(open, recording, "a.tape", true)).toBe("consult");
});

test("il Tape del doppio clic aspetta la fine dell'Attività e della conferma, poi si apre una volta", () => {
  const waiting = run([{ path: "c.tape", type: "tapeRequested" }]);
  expect(pendingTape(waiting, transcribing, false)).toBeNull();
  expect(pendingTape(waiting, idle, true)).toBeNull();
  expect(pendingTape(waiting, idle, false)).toBe("c.tape");
  const open = run(
    [{ opening: "source", path: "c.tape", type: "openRequested" }],
    waiting
  );
  expect(pendingTape(open, idle, false)).toBeNull();
});

test("il Tape del doppio clic consultato durante l'Attività diventa la Sorgente alla fine", () => {
  const state = run([
    { path: "c.tape", type: "tapeRequested" },
    { opening: "consult", path: "c.tape", type: "openRequested" },
    { path: "c.tape", tape: opened("C."), type: "tapeConsulted" },
  ]);
  expect(pendingTape(state, idle, false)).toBe("c.tape");
});
