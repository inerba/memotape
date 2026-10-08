import { expect, test } from "bun:test";
import type { Ingresso } from "@/bindings";
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

const run = (actions: ActivityAction[], from = INITIAL_ACTIVITY) =>
  actions.reduce(activity, from);

const texts = (state: ActivityState) =>
  visiblePhrases(state.conversation).map((p) => p.text);

const tape: ActivityAction = {
  tape: { parlanti: {}, phrases: [phrase(0, 0, "Tape precedente.")] },
  type: "sourceOpened",
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
    { type: "outcome" },
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
    { type: "outcome" },
    record("second"),
    { payload: phrase(0, 0, "Vecchia", "mix", "first"), type: "phrase" },
    { payload: phrase(0, 0, "Vecchio", "mix", "first"), type: "partial" },
    snapshot("mix", [phrase(0, 0, "Vecchio finale")], "first"),
  ]);
  expect(texts(state)).toEqual([]);
});

const start = (kind: "transcription" | "diarization"): ActivityAction => ({
  kind,
  nomeMicrofono: null,
  partials: false,
  sessionId: null,
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
      { type: "outcome" },
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

test("il guasto della Trascrizione dal vivo toglie i Parziali, le Frasi restano", () => {
  const state = run([
    record("live"),
    { payload: phrase(0, 0, "Mi senti?", "mix", "live"), type: "phrase" },
    { payload: phrase(1, 1000, "Sì", "mix", "live"), type: "partial" },
    { sessionId: "other", type: "liveFailed" },
  ]);
  expect(texts(state)).toEqual(["Mi senti?", "Sì"]);
  expect(
    texts(activity(state, { sessionId: "live", type: "liveFailed" }))
  ).toEqual(["Mi senti?"]);
});
