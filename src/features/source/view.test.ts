import { expect, test } from "bun:test";
import type { OpenedTape } from "@/bindings";
import {
  INITIAL_VIEW,
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

const run = (actions: SourceViewAction[]) =>
  actions.reduce(
    (state, action) => sourceView(state, action, idle),
    INITIAL_VIEW
  );

test("un Tape spostato resta aperto con il percorso nuovo, come Sorgente e come Tape consultato", () => {
  const state = run([
    { path: "C:\\Lib\\a.tape", type: "sourceOpened" },
    { path: "C:\\Lib\\b.tape", tape: opened("B."), type: "tapeBrowsed" },
    { from: "C:\\Lib", to: "C:\\Nuova", type: "tapeMoved" },
  ]);
  expect(state.source).toBe("C:\\Nuova\\a.tape");
  expect(state.browsed?.path).toBe("C:\\Nuova\\b.tape");
  expect(state.browsed?.conversation.phrases[0]?.text).toBe("B.");
});

test("un Tape nel Cestino si chiude se era aperto, come Sorgente o come Tape consultato", () => {
  const consulted = run([
    { path: "a.tape", type: "sourceOpened" },
    { path: "b.tape", tape: opened("B."), type: "tapeBrowsed" },
    { path: "b.tape", type: "tapeTrashed" },
  ]);
  expect(consulted.source).toBe("a.tape");
  expect(consulted.browsed).toBeNull();
  const own = sourceView(
    consulted,
    { path: "a.tape", type: "tapeTrashed" },
    idle
  );
  expect(own.source).toBeNull();
});

test("una modifica vale per il Tape consultato con quel percorso e solo per quello", () => {
  const change = (path: string): SourceViewAction => ({
    change: (c) => ({ ...c, parlanti: { "mix:1": "Anna" } }),
    path,
    type: "tapeChanged",
  });
  const state = run([
    { path: "b.tape", tape: opened("B."), type: "tapeBrowsed" },
    change("altro.tape"),
  ]);
  expect(state.browsed?.conversation.parlanti).toEqual({});
  expect(
    sourceView(state, change("b.tape"), idle).browsed?.conversation.parlanti
  ).toEqual({ "mix:1": "Anna" });
});
