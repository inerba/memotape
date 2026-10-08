import { expect, test } from "bun:test";
import {
  centerView,
  type HomeAction,
  type HomeState,
  home,
  INITIAL_HOME,
  selection,
} from "@/app/routes/home-state";
import type { OpenedTape, TapeInfo } from "@/bindings";
import { opening, pendingTape } from "@/features/source/view";
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
  const correction = (path: string): HomeAction => ({
    change: (c) => withTesto(c, { ingresso: "mix", phraseId: 0 }, "Corretto."),
    info: (i) => i && { ...i, correttoAMano: true },
    path,
    type: "tapeChanged",
  });
  const diarizing = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
    { kind: "diarization", type: "start" },
    { path: "b.tape", tape: tape("B."), type: "tapeBrowsed" },
  ]);
  const source = run(
    [
      { status: { path: "a.tape", phase: "diarized" }, type: "outcome" },
      correction("a.tape"),
    ],
    diarizing
  );
  expect(texts(source)).toEqual(["Corretto."]);
  expect(source.activity.info?.correttoAMano).toBe(true);
  const browsed = home(diarizing, correction("b.tape"));
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

const transcribing: HomeAction[] = [
  { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
  { kind: "transcription", type: "start" },
];

const found = { ingresso: "mix" as const, phraseId: 0 };

test("aprire un Tape dalla Libreria lo rende la Sorgente, evidenziato nella barra laterale", () => {
  const state = run([
    { type: "libraryShown" },
    { fromLibrary: true, path: "a.tape", type: "openRequested" },
    { path: "a.tape", phrase: found, tape: tape("A."), type: "sourceOpened" },
  ]);
  const center = centerView(state);
  expect(center.kind === "tape" && center.own).toBe(true);
  expect(center.kind === "tape" && center.tape.path).toBe("a.tape");
  expect(selection(state)).toEqual({
    home: false,
    library: false,
    tape: "a.tape",
  });
  expect(state.view.highlight).toEqual(found);
});

test("riaprire la Sorgente dalla Libreria riparte dall'inizio, o dalla Frase trovata", () => {
  const open = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
    { type: "homeShown" },
  ]);
  expect(opening(open.view, open.activity.status, "a.tape", true)).toBe(
    "restart"
  );
  const again = home(open, {
    fromLibrary: true,
    path: "a.tape",
    phrase: found,
    type: "openRequested",
  });
  expect(again.view.revision).toBe(open.view.revision + 1);
  expect(again.view.highlight).toEqual(found);
  expect(centerView(again).kind).toBe("tape");
  // Apri file sullo stesso Tape lo rilegge, senza ripartire.
  expect(opening(open.view, open.activity.status, "a.tape", false)).toBe(
    "source"
  );
});

test("durante un'Attività un Tape della Libreria si consulta senza cambiare la Sorgente", () => {
  const busy = run(transcribing);
  expect(opening(busy.view, busy.activity.status, "b.tape", true)).toBe(
    "browse"
  );
  const state = run(
    [
      {
        fromLibrary: true,
        path: "b.tape",
        phrase: found,
        type: "openRequested",
      },
      { path: "b.tape", phrase: found, tape: tape("B."), type: "tapeBrowsed" },
    ],
    busy
  );
  const center = centerView(state);
  expect(center.kind === "tape" && !center.own).toBe(true);
  expect(center.kind === "tape" && center.tape.path).toBe("b.tape");
  expect(state.view.source).toBe("a.tape");
  expect(state.view.highlight).toEqual(found);
  expect(selection(state).tape).toBe("b.tape");
});

test("durante Trascrivi aprire la Sorgente riporta alla vista dell'Attività", () => {
  const busy = run([
    ...transcribing,
    { path: "b.tape", tape: tape("B."), type: "tapeBrowsed" },
  ]);
  expect(opening(busy.view, busy.activity.status, "a.tape", true)).toBe(
    "activity"
  );
  const state = home(busy, {
    fromLibrary: true,
    path: "a.tape",
    phrase: found,
    type: "openRequested",
  });
  const center = centerView(state);
  expect(center.kind === "tape" && center.own).toBe(true);
  expect(state.view.browsed).toBeNull();
  expect(state.view.highlight).toEqual(found);
  // Durante una Registrazione la Sorgente è un Tape come gli altri.
  const live = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
    ...recording,
  ]);
  expect(opening(live.view, live.activity.status, "a.tape", true)).toBe(
    "browse"
  );
});

test("Attività in corso riporta alla sua vista e toglie il Tape consultato", () => {
  const state = run([
    ...transcribing,
    { path: "b.tape", phrase: found, tape: tape("B."), type: "tapeBrowsed" },
    { type: "libraryShown" },
    { type: "activityShown" },
  ]);
  expect(centerView(state).kind).toBe("tape");
  expect(state.view.browsed).toBeNull();
  expect(state.view.highlight).toBeNull();
  expect(selection(state).tape).toBe("a.tape");
});

test("a fine Attività torna la Sorgente con il suo esito, senza il Tape consultato", () => {
  const state = run([
    ...transcribing,
    { path: "b.tape", tape: tape("B."), type: "tapeBrowsed" },
    { path: "a.tape", tape: tape("Trascritto."), type: "sourceOpened" },
    { status: { path: "a.tape", phase: "finished" }, type: "outcome" },
  ]);
  const center = centerView(state);
  expect(center.kind === "tape" && center.own).toBe(true);
  expect(state.view.browsed).toBeNull();
  expect(texts(state)).toEqual(["Trascritto."]);
});

test("all'avvio della Registrazione la sua vista prende il posto di Home, Libreria e Tape consultato", () => {
  const preparing = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
    record,
    { path: "b.tape", phrase: found, tape: tape("B."), type: "tapeBrowsed" },
    { type: "libraryShown" },
    { type: "homeShown" },
  ]);
  expect(centerView(preparing).kind).toBe("preparation");
  const state = home(preparing, {
    payload: { phase: "recording", sessionId: "live" },
    type: "recordingPhase",
  });
  expect(centerView(state).kind).toBe("live");
  expect(state.view.browsed).toBeNull();
  expect(state.view.highlight).toBeNull();
  expect(selection(state)).toEqual({ home: false, library: false, tape: null });
});

test("Home e Libreria si aprono e si chiudono senza perdere la Sorgente", () => {
  const open = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
  ]);
  const library = home(open, { type: "libraryShown" });
  expect(centerView(library).kind).toBe("library");
  expect(selection(library)).toEqual({
    home: false,
    library: true,
    tape: null,
  });
  const shown = home(library, { type: "homeShown" });
  expect(centerView(shown).kind).toBe("home");
  expect(selection(shown)).toEqual({ home: true, library: false, tape: null });
  const back = home(shown, {
    fromLibrary: true,
    path: "a.tape",
    type: "openRequested",
  });
  expect(centerView(back).kind).toBe("tape");
  expect(back.view.source).toBe("a.tape");
});

test("senza Sorgente si vede la Home; un file aperto ha la sua vista", () => {
  expect(centerView(INITIAL_HOME).kind).toBe("home");
  expect(selection(INITIAL_HOME).home).toBe(true);
  const file = run([{ path: "a.mp3", type: "sourceOpened" }]);
  expect(centerView(file)).toEqual({ kind: "file", path: "a.mp3" });
  expect(selection(file).tape).toBe("a.mp3");
});

test("un Tape rilasciato durante un'Attività si consulta; senza Attività un file diventa la Sorgente", () => {
  const busy = run(transcribing);
  expect(opening(busy.view, busy.activity.status, "c.tape", false)).toBe(
    "browse"
  );
  expect(
    opening(INITIAL_HOME.view, INITIAL_HOME.activity.status, "c.mp3", false)
  ).toBe("source");
});

test("il Tape del doppio clic aspetta la fine dell'Attività e della conferma, poi si apre una volta", () => {
  const waiting = run([
    ...transcribing,
    { path: "c.tape", type: "tapeRequested" },
  ]);
  expect(pendingTape(waiting.view, waiting.activity.status, false)).toBeNull();
  const done = home(waiting, {
    status: { phase: "noSpeech" },
    type: "outcome",
  });
  expect(pendingTape(done.view, done.activity.status, true)).toBeNull();
  expect(pendingTape(done.view, done.activity.status, false)).toBe("c.tape");
  const opened = home(done, { path: "c.tape", type: "openRequested" });
  expect(pendingTape(opened.view, opened.activity.status, false)).toBeNull();
});

test("Trascrivi toglie la Frase evidenziata", () => {
  const state = run([
    { path: "a.tape", phrase: found, tape: tape("A."), type: "sourceOpened" },
    { kind: "transcription", type: "start" },
  ]);
  expect(state.view.highlight).toBeNull();
});

test("aprire un altro Tape o registrare cambia la vista del Tape, e con lei la rinomina in corso", () => {
  const open = run([
    { path: "a.tape", tape: tape("A."), type: "sourceOpened" },
  ]);
  const other = home(open, {
    path: "b.tape",
    tape: tape("B."),
    type: "sourceOpened",
  });
  const center = centerView(other);
  expect(center.kind === "tape" && center.tape.path).toBe("b.tape");
  expect(centerView(run(recording, other)).kind).toBe("live");
});
