import { expect, test } from "bun:test";
import {
  type HomeAction,
  type HomeState,
  home,
  INITIAL_HOME,
} from "@/app/routes/home-state";
import type { OpenedTape, TapeInfo } from "@/bindings";
import { visiblePhrases, withTesto } from "@/features/transcription/phrases";

const info: TapeInfo = {
  completa: true,
  correttoAMano: false,
  creato: "2026-10-08T10:00:00+02:00",
  durataMs: 1000,
  ingressiSeparati: false,
  linguaParlato: "it",
  modello: null,
  origine: null,
};

const phrase = (text: string, sessionId: string | null = null) => ({
  fineMs: 900,
  ingresso: "mix" as const,
  inizioMs: 0,
  parlante: null,
  phraseId: 0,
  sessionId,
  text,
});

const tape = (text: string): OpenedTape => ({
  info,
  parlanti: {},
  phrases: [phrase(text)],
});

const run = (actions: HomeAction[], from = INITIAL_HOME) =>
  actions.reduce(home, from);

const texts = (state: HomeState) =>
  visiblePhrases(state.activity.conversation).map((p) => p.text);

const record: HomeAction = {
  kind: "recording",
  nomeMicrofono: null,
  partials: false,
  sessionId: "live",
  type: "start",
};

const recording: HomeAction[] = [
  record,
  {
    payload: { phase: "recording", sessionId: "live" },
    type: "recordingPhase",
  },
  { payload: phrase("Dal vivo.", "live"), type: "phrase" },
];

test("Apri file rende il file la Sorgente, pronto da trascrivere", () => {
  const state = run([
    { path: "a.tape", tape: tape("Tape."), type: "sourceOpened" },
    { path: "intervista.mp3", type: "sourceOpened" },
  ]);
  expect(state.view.source).toBe("intervista.mp3");
  expect(texts(state)).toEqual([]);
  expect(state.activity.info).toBeNull();
  expect(state.activity.status).toEqual({
    phase: "idle",
    source: "intervista.mp3",
  });
});

test("un Tape aperto diventa la Sorgente con il suo testo e le sue informazioni", () => {
  const state = run([
    { path: "a.tape", tape: tape("Tape."), type: "sourceOpened" },
  ]);
  expect(texts(state)).toEqual(["Tape."]);
  expect(state.activity.info).toEqual(info);
  expect(state.activity.status).toEqual({ phase: "idle", source: "a.tape" });
});

test("il Tape scritto alla fine di Trascrivi prende il testo, lo Status lo dà l'esito", () => {
  const transcribing = run([
    { path: "a.mp3", type: "sourceOpened" },
    { kind: "transcription", type: "start" },
    { path: "a.tape", tape: tape("Trascritto."), type: "sourceOpened" },
  ]);
  expect(texts(transcribing)).toEqual(["Trascritto."]);
  expect(transcribing.activity.status.phase).toBe("transcribing");
  const done = home(transcribing, {
    status: { path: "a.tape", phase: "finished" },
    type: "outcome",
  });
  expect(done.view.source).toBe("a.tape");
  expect(done.activity.status).toEqual({ path: "a.tape", phase: "finished" });
});

test("una correzione vale per la Sorgente e per il Tape consultato con quel percorso", () => {
  const corrected = (path: string) =>
    run([
      { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
      { kind: "diarization", type: "start" },
      { path: "b.tape", tape: tape("B."), type: "tapeBrowsed" },
      { status: { path: "a.tape", phase: "diarized" }, type: "outcome" },
      {
        change: (c) =>
          withTesto(c, { ingresso: "mix", phraseId: 0 }, "Corretto."),
        info: (i) => i && { ...i, correttoAMano: true },
        path,
        type: "tapeChanged",
      },
    ]);
  const source = corrected("a.tape");
  expect(texts(source)).toEqual(["Corretto."]);
  expect(source.activity.info?.correttoAMano).toBe(true);
  expect(source.view.browsed?.conversation.phrases[0]?.text).toBe("B.");
  const browsed = corrected("b.tape");
  expect(texts(browsed)).toEqual(["A."]);
  expect(browsed.view.browsed?.conversation.phrases[0]?.text).toBe("Corretto.");
  expect(browsed.view.browsed?.info?.correttoAMano).toBe(true);
});

test("consultare e correggere la Sorgente durante una Registrazione non tocca il testo dal vivo", () => {
  const state = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
    ...recording,
    { path: "a.tape", tape: tape("A."), type: "tapeBrowsed" },
    {
      change: (c) =>
        withTesto(c, { ingresso: "mix", phraseId: 0 }, "Corretto."),
      path: "a.tape",
      type: "tapeChanged",
    },
  ]);
  expect(texts(state)).toEqual(["Dal vivo."]);
  expect(state.view.browsed?.conversation.phrases[0]?.text).toBe("Corretto.");
});

test("un Tape aperto e spostato resta la Sorgente, anche nello Status", () => {
  const state = run([
    { path: "C:\\Lib\\a.tape", tape: tape("A."), type: "sourceOpened" },
    { from: "C:\\Lib\\a.tape", to: "C:\\Lib\\b.tape", type: "tapeMoved" },
  ]);
  expect(state.view.source).toBe("C:\\Lib\\b.tape");
  expect(state.activity.status).toEqual({
    phase: "idle",
    source: "C:\\Lib\\b.tape",
  });
  expect(texts(state)).toEqual(["A."]);
});

test("la Sorgente nel Cestino si chiude; un altro Tape nel Cestino non la tocca", () => {
  const open = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
  ]);
  expect(home(open, { path: "b.tape", type: "tapeTrashed" })).toEqual(open);
  const state = home(open, { path: "a.tape", type: "tapeTrashed" });
  expect(state.view.source).toBeNull();
  expect(texts(state)).toEqual([]);
  expect(state.activity.status).toEqual({ phase: "idle", source: null });
});

test("durante una Registrazione la Sorgente nel Cestino si chiude ma la vista dell'Attività resta", () => {
  const state = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
    ...recording,
    { path: "a.tape", type: "tapeTrashed" },
  ]);
  expect(state.view.source).toBeNull();
  expect(texts(state)).toEqual(["Dal vivo."]);
  expect(state.activity.status.phase).toBe("recording");
});
