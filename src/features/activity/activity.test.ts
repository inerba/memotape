import { expect, test } from "bun:test";
import type { Ingresso, TapeInfo } from "@/bindings";
import {
  type ActivityAction,
  type ActivityState,
  activity,
  INITIAL_ACTIVITY,
} from "@/features/activity/activity";
import { visiblePhrases, withTesto } from "@/features/transcription/phrases";

const phrase = (
  phraseId: number,
  inizioMs: number,
  text: string,
  ingresso: Ingresso = "mix",
  sessionId: string | null = null
) => ({
  fineMs: inizioMs + 900,
  ingresso,
  inizioMs,
  parlante: null as number | null,
  phraseId,
  sessionId,
  text,
});

const stop: ActivityAction = {
  status: { path: "a.tape", phase: "finished" },
  type: "outcome",
};

const run = (actions: ActivityAction[], from = INITIAL_ACTIVITY) =>
  actions.reduce(activity, from);

const texts = (state: ActivityState) =>
  visiblePhrases(state.conversation).map((p) => p.text);

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

const tape: ActivityAction = {
  tape: { info, parlanti: {}, phrases: [phrase(0, 0, "Tape precedente.")] },
  type: "sourceLoaded",
};

const record = (sessionId: string, partials = true): ActivityAction => ({
  kind: "recording",
  nomeMicrofono: null,
  partials,
  sessionId,
  type: "start",
});

test("durante una Registrazione la correzione del Tape precedente non cambia il testo dal vivo", () => {
  const state = run([
    tape,
    record("live"),
    { payload: phrase(0, 0, "Dal vivo.", "mix", "live"), type: "phrase" },
    {
      change: (c) =>
        withTesto(c, { ingresso: "mix", phraseId: 0 }, "Tape corretto."),
      type: "sourceChanged",
    },
  ]);
  expect(texts(state)).toEqual(["Dal vivo."]);
});

test("i Parziali entrano solo se la Registrazione è partita con Trascrivi dal vivo", () => {
  const partial: ActivityAction = {
    payload: phrase(0, 0, "Parzi", "mix", "live"),
    type: "partial",
  };
  expect(texts(run([record("live", true), partial]))).toEqual(["Parzi"]);
  expect(texts(run([record("live", false), partial]))).toEqual([]);
});

const snapshot = (
  ingresso: Ingresso,
  phrases: ReturnType<typeof phrase>[],
  sessionId = "live"
): ActivityAction => ({
  payload: { ingresso, phrases, sessionId },
  type: "liveTranscript",
});

test("lo snapshot finale sostituisce le Frasi e il Parziale del suo Ingresso, una volta sola", () => {
  const state = run([
    record("live"),
    {
      payload: phrase(0, 0, "Uno. Due.", "microfono", "live"),
      type: "phrase",
    },
    { payload: phrase(0, 500, "Sistema.", "sistema", "live"), type: "phrase" },
    {
      payload: phrase(1, 3000, "Parzi", "microfono", "live"),
      type: "partial",
    },
    { payload: phrase(1, 3500, "Altro", "sistema", "live"), type: "partial" },
    snapshot("microfono", [
      { ...phrase(0, 100, "Uno. ", "microfono"), parlante: 1 },
      { ...phrase(1, 1500, "Due.", "microfono"), parlante: 2 },
    ]),
    // Un secondo snapshot e il testo tardivo dello stesso Ingresso si scartano.
    snapshot("microfono", [phrase(0, 0, "Un altro testo.", "microfono")]),
    {
      payload: phrase(2, 4000, "Tardiva", "microfono", "live"),
      type: "phrase",
    },
    {
      payload: phrase(2, 4000, "Tardivo", "microfono", "live"),
      type: "partial",
    },
  ]);
  expect(texts(state)).toEqual(["Uno. ", "Sistema.", "Due.", "Altro"]);
  // L'altro Ingresso riceve ancora il suo.
  expect(
    texts(
      activity(state, snapshot("sistema", [phrase(0, 500, "Fine.", "sistema")]))
    )
  ).toEqual(["Uno. ", "Fine.", "Due."]);
});

test("dopo Stop la Registrazione accetta solo il suo snapshot finale, finché non si apre un'altra Sorgente", () => {
  const stopped = run([
    record("live"),
    { payload: phrase(0, 0, "Parzi", "mix", "live"), type: "partial" },
    stop,
    { payload: phrase(0, 0, "Frase tardiva", "mix", "live"), type: "phrase" },
  ]);
  expect(texts(stopped)).toEqual([]);
  // Il fallback Ogg: il Tape non c'è e il testo finale arriva dopo la risposta a Stop.
  const final = snapshot("mix", [phrase(0, 0, "Testo finale completo")]);
  expect(texts(activity(stopped, final))).toEqual(["Testo finale completo"]);
  // Con il Tape riaperto lo snapshot tardivo non lo sostituisce.
  expect(texts(run([tape, final], stopped))).toEqual(["Tape precedente."]);
});

test("una nuova Registrazione rifiuta il testo tardivo della precedente", () => {
  const state = run([
    record("first"),
    stop,
    record("second"),
    { payload: phrase(0, 0, "Vecchia", "mix", "first"), type: "phrase" },
    { payload: phrase(0, 0, "Vecchio", "mix", "first"), type: "partial" },
    snapshot("mix", [phrase(0, 0, "Vecchio finale")], "first"),
  ]);
  expect(texts(state)).toEqual([]);
});

const start = (kind: "transcription" | "diarization"): ActivityAction => ({
  kind,
  type: "start",
});

const speakers = (parlante: number): ActivityAction => ({
  payload: { speakers: [{ ingresso: "mix", parlante, phraseId: 0 }] },
  type: "speakers",
});

test("dopo l'esito di Trascrivi Frasi e Parlanti tardivi non entrano nel Tape riaperto", () => {
  const running = run([
    tape,
    start("transcription"),
    { payload: phrase(0, 0, "Nuova."), type: "phrase" },
    speakers(1),
  ]);
  expect(running.conversation.phrases.map((p) => [p.text, p.parlante])).toEqual(
    [["Nuova.", 1]]
  );
  const late = run(
    [
      stop,
      tape,
      { payload: phrase(1, 2000, "Tardiva."), type: "phrase" },
      speakers(2),
    ],
    running
  );
  expect(late.conversation.phrases.map((p) => [p.text, p.parlante])).toEqual([
    ["Tape precedente.", null],
  ]);
});

test("Riconosci i parlanti tiene in vista il testo e accetta solo le sue attribuzioni", () => {
  const state = run([tape, start("diarization"), speakers(1)]);
  expect(state.conversation.phrases.map((p) => [p.text, p.parlante])).toEqual([
    ["Tape precedente.", 1],
  ]);
  // Le attribuzioni di una Registrazione non sono sue.
  expect(
    activity(state, {
      payload: { sessionId: "live", speakers: [] },
      type: "speakers",
    }).conversation
  ).toBe(state.conversation);
});

test("il Microfono ha il suo nome dal primo istante; annullare la Preparazione riporta la Sorgente di prima", () => {
  const preparing = run([
    tape,
    { ...record("live"), nomeMicrofono: "Francesco" } as ActivityAction,
  ]);
  expect(preparing.conversation).toMatchObject({
    parlanti: { microfono: "Francesco" },
    phrases: [],
  });
  const restored = activity(preparing, { type: "restore" });
  expect(texts(restored)).toEqual(["Tape precedente."]);
  // Senza Attività le correzioni della Sorgente si vedono subito.
  expect(
    texts(
      activity(restored, {
        change: (c) =>
          withTesto(c, { ingresso: "mix", phraseId: 0 }, "Corretto."),
        type: "sourceChanged",
      })
    )
  ).toEqual(["Corretto."]);
  // Gli eventi della Registrazione annullata non entrano.
  expect(
    activity(restored, {
      payload: phrase(1, 2000, "Tardiva", "mix", "live"),
      type: "phrase",
    })
  ).toBe(restored);
});

test("il guasto della Trascrizione dal vivo toglie i Parziali e lo dice la fase; le Frasi restano", () => {
  const failed = (sessionId: string): ActivityAction => ({
    payload: {
      error: { code: "liveTranscriptionUnavailable", detail: "Nemotron" },
      sessionId,
    },
    type: "liveFailed",
  });
  const state = run([
    record("live"),
    {
      payload: { phase: "recording", sessionId: "live" },
      type: "recordingPhase",
    },
    { payload: phrase(0, 0, "Mi senti?", "mix", "live"), type: "phrase" },
    { payload: phrase(1, 1000, "Sì", "mix", "live"), type: "partial" },
    failed("other"),
  ]);
  expect(texts(state)).toEqual(["Mi senti?", "Sì"]);
  expect(state.status).toEqual({ paused: false, phase: "recording" });
  const after = activity(state, failed("live"));
  expect(texts(after)).toEqual(["Mi senti?"]);
  expect(after.status).toEqual({
    liveError: { code: "liveTranscriptionUnavailable", detail: "Nemotron" },
    paused: false,
    phase: "recording",
  });
});

test("fase, progresso e analisi dei Parlanti seguono solo la Registrazione in corso", () => {
  const phase = (
    sessionId: string,
    value: "cleaning" | "recording"
  ): ActivityAction => ({
    payload: { phase: value, sessionId },
    type: "recordingPhase",
  });
  const progress = (
    sessionId: string | null,
    percent: number | null
  ): ActivityAction => ({ payload: { percent, sessionId }, type: "progress" });
  const preparing = run([
    record("first"),
    { status: { path: "a.tape", phase: "recorded" }, type: "outcome" },
    record("second"),
  ]);
  expect(preparing.status).toEqual({
    phase: "preparingRecording",
    stage: "saving",
  });
  const recording = run(
    [
      phase("first", "recording"),
      phase("second", "cleaning"),
      phase("second", "recording"),
      progress("first", 100),
      progress(null, 100),
      { payload: { sessionId: "first" }, type: "diarizationStarted" },
    ],
    preparing
  );
  expect(recording.status).toEqual({ paused: false, phase: "recording" });
  const completing = run([progress("second", 10)], recording);
  expect(completing.status).toEqual({ percent: 10, phase: "completing" });
  expect(
    activity(completing, {
      payload: { sessionId: "second" },
      type: "diarizationStarted",
    }).status
  ).toEqual({ diarizing: true, percent: null, phase: "completing" });
});

const recordingIn = (sessionId: string): ActivityAction[] => [
  record(sessionId),
  { payload: { phase: "recording", sessionId }, type: "recordingPhase" },
];

const tick = (sessionId: string, elapsedMs: number): ActivityAction => ({
  payload: { elapsedMs, levels: { microphone: 0, system: null }, sessionId },
  type: "tick",
});

const cleaningFailed = (
  sessionId: string,
  ingresso: Ingresso
): ActivityAction => ({
  payload: { error: { code: "audioCleaningMissing" }, ingresso, sessionId },
  type: "cleaningFailed",
});

test("il timer riparte da zero a ogni Registrazione e ignora i tick della precedente", () => {
  const first = run([...recordingIn("first"), tick("first", 28_000), stop]);
  expect(first.elapsedMs).toBe(28_000);
  const second = run(recordingIn("second"), first);
  expect(second.elapsedMs).toBe(0);
  expect(
    run([tick("first", 29_000), tick("second", 1100)], second).elapsedMs
  ).toBe(1000);
});

test("un tick nello stesso secondo non cambia lo stato", () => {
  const recording = run([...recordingIn("live"), tick("live", 1040)]);
  expect(activity(recording, tick("live", 1080))).toBe(recording);
});

test("Pausa e Riprendi cambiano solo la fase della Registrazione", () => {
  const paused = run([...recordingIn("live"), { paused: true, type: "pause" }]);
  expect(paused.status).toEqual({ paused: true, phase: "recording" });
  const idle = run([{ paused: true, type: "pause" }]);
  expect(idle.status).toBe(INITIAL_ACTIVITY.status);
});

test("i guasti della pulizia restano uno per Ingresso; chiuso l'avviso, un nuovo guasto lo riapre", () => {
  const state = run([
    ...recordingIn("live"),
    cleaningFailed("old", "microfono"),
    cleaningFailed("live", "microfono"),
    cleaningFailed("live", "microfono"),
    cleaningFailed("live", "sistema"),
    { type: "cleaningDismissed" },
  ]);
  expect(state.cleaningFailures.map((f) => f.ingresso)).toEqual([
    "microfono",
    "sistema",
  ]);
  expect(state.cleaningDismissed).toBe(true);
  // Chiudere l'avviso non toglie il guasto: la barra mostra ancora il bypass.
  expect(state.cleaningFailures).toHaveLength(2);
  expect(
    activity(state, cleaningFailed("live", "sistema")).cleaningDismissed
  ).toBe(false);
});

test("la preparazione della pulizia è della Registrazione in corso e sparisce con la pulizia spenta", () => {
  const preparing = (
    sessionId: string,
    ingresso: Ingresso
  ): ActivityAction => ({
    payload: { ingresso, preparing: true, sessionId },
    type: "cleaningPreparing",
  });
  const state = run([
    ...recordingIn("live"),
    preparing("old", "microfono"),
    preparing("live", "microfono"),
    preparing("live", "sistema"),
    { ingresso: "microfono", type: "cleaningOff" },
  ]);
  expect(state.cleaningPreparing.map((p) => p.ingresso)).toEqual(["sistema"]);
});

test("annullare la Preparazione riporta fase, avvisi di pulizia e sessione di prima", () => {
  const before = run([
    ...recordingIn("first"),
    cleaningFailed("first", "microfono"),
    { type: "cleaningDismissed" },
    stop,
  ]);
  const preparing = run(
    [
      record("second"),
      {
        payload: { phase: "cleaning", sessionId: "second" },
        type: "recordingPhase",
      },
    ],
    before
  );
  expect(preparing.status).toEqual({
    phase: "preparingRecording",
    stage: "cleaning",
  });
  expect(preparing.cleaningFailures).toEqual([]);
  const restored = activity(preparing, { type: "restore" });
  expect(restored.status).toEqual({ path: "a.tape", phase: "finished" });
  expect(restored.cleaningFailures.map((f) => f.ingresso)).toEqual([
    "microfono",
  ]);
  expect(restored.cleaningDismissed).toBe(true);
  // La Registrazione annullata non cambia più nulla.
  expect(
    run(
      [
        {
          payload: { phase: "recording", sessionId: "second" },
          type: "recordingPhase",
        },
        tick("second", 500),
        cleaningFailed("second", "sistema"),
      ],
      restored
    )
  ).toBe(restored);
});

test("le informazioni della Sorgente cambiano solo senza Attività; partita la Registrazione non ce ne sono", () => {
  const corrected: TapeInfo = { ...info, correttoAMano: true };
  const replace: ActivityAction = {
    info: () => corrected,
    type: "sourceChanged",
  };
  expect(run([tape, replace]).info).toEqual(corrected);
  // Durante la Preparazione la vista di prima resta, con le sue informazioni.
  const preparing = run([tape, record("live"), { type: "recordingRequested" }]);
  expect(preparing.status).toEqual({
    phase: "preparingRecording",
    stage: "preparing",
  });
  expect(run([replace], preparing).info).toEqual(info);
  const started = run(
    [
      {
        payload: { phase: "recording", sessionId: "live" },
        type: "recordingPhase",
      },
      replace,
    ],
    preparing
  );
  expect(started.info).toBeNull();
  // Nel fallback Ogg la Sorgente è l'Ogg: la correzione del Tape consultato non la tocca.
  expect(run([stop, replace], started).info).toBeNull();
});
