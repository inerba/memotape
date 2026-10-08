import { expect, test } from "bun:test";
import type { TFunction } from "i18next";
import type { Ingresso } from "@/bindings";
import {
  type Conversation,
  EMPTY_CONVERSATION,
  mergeDestination,
  nomeTaken,
  parlanteStats,
  parlantiOf,
  type Turn,
  turnBody,
  turnsOf,
  turnText,
  VOICE_COLORS,
  visiblePhrases,
  voiceColors,
  withLiveTranscript,
  withNome,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
  withTesto,
} from "@/features/transcription/phrases";

/** Etichetta e testo del turno insieme, per controllare l'attribuzione nei test. */
const labeled = (turn: Turn) =>
  turn.label ? `${turn.label}: ${turnText(turn)}` : turnText(turn);

test("un Turno vuoto unito non aggiunge separatori e conserva gli a capo del testo vicino", () => {
  for (const emptyFirst of [true, false]) {
    const items = [
      { ...phrase(0, 0, ""), parlante: 1, testoCorretto: true, testoTurno: 0 },
      { ...phrase(1, 500, "\nTesto.\n\nParagrafo.\n"), parlante: 1 },
    ];
    const [turn] = turnsOf(
      {
        ...EMPTY_CONVERSATION,
        phrases: (emptyFirst ? items : [...items].reverse()).map(
          (item, index) => ({
            ...item,
            fineMs: index * 500 + 400,
            inizioMs: index * 500,
          })
        ),
      },
      t
    );
    if (!turn) {
      throw new Error("Turno atteso");
    }
    expect(turnBody(turn)).toBe("\nTesto.\n\nParagrafo.\n");
    expect(turnText(turn)).toBe("\nTesto.\n\nParagrafo.\n");
  }
});

test("Copia turno conserva a capo e testo corretto senza duplicare le Frasi originali", () => {
  const conversation: Conversation = {
    ...EMPTY_CONVERSATION,
    phrases: [
      {
        ...phrase(0, 0, "Testo unito.\n\nParagrafo."),
        parlante: 1,
        testoCorretto: true,
        testoTurno: 0,
      },
      {
        ...phrase(1, 500, ""),
        parlante: 1,
        testoCorretto: true,
        testoTurno: 0,
      },
      { ...phrase(2, 1000, "Altra frase."), parlante: 1 },
    ],
  };
  const [turn] = turnsOf(conversation, t);
  if (!turn) {
    throw new Error("Turno atteso");
  }
  expect(turnBody(turn)).toBe("Testo unito.\n\nParagrafo.\nAltra frase.");
  expect(turnText(turn)).toBe("Testo unito.\n\nParagrafo.\nAltra frase.");
  const [first] = conversation.phrases;
  if (!first) {
    throw new Error("Frase attesa");
  }
  conversation.phrases[0] = { ...first, text: "" };
  conversation.phrases.pop();
  const [empty] = turnsOf(conversation, t);
  if (!empty) {
    throw new Error("Turno vuoto atteso");
  }
  expect(turnBody(empty)).toBe("");
  expect(turnText(empty)).toBe("");
});

test("l'unione considera solo il vicino e richiede una voce nota dello stesso Ingresso", () => {
  const conversation: Conversation = {
    ...EMPTY_CONVERSATION,
    phrases: [
      { ...phrase(0, 0, "Mario."), parlante: 1 },
      { ...phrase(1, 1000, "Anna."), parlante: 2 },
      { ...phrase(2, 2000, "Mario."), parlante: 1 },
    ],
  };
  const [above, source, below] = turnsOf(conversation, t);
  if (!(above && source && below)) {
    throw new Error("tre Turni attesi");
  }
  const [targetPhrase] = below.items;
  const [sourcePhrase] = source.items;
  if (!(targetPhrase && sourcePhrase)) {
    throw new Error("Frasi attese");
  }
  expect(mergeDestination(source, above, [])).toMatchObject({
    ingresso: "mix",
    phraseId: 0,
  });
  expect(mergeDestination(source, below, [])).toMatchObject({
    ingresso: "mix",
    phraseId: 2,
  });
  expect(mergeDestination(above, undefined, [])).toBeNull();
  expect(
    mergeDestination(source, { ...below, ingresso: "sistema" }, [])
  ).toBeNull();
  expect(
    mergeDestination(
      source,
      {
        ...below,
        items: [{ ...targetPhrase, parlanteNonDeterminato: true }],
      },
      []
    )
  ).toBeNull();
  expect(mergeDestination(source, { ...below, label: null }, [])).toBeNull();
  expect(mergeDestination(source, below, [sourcePhrase])).toBeNull();
});

test("lo snapshot finale sostituisce le Frasi del suo Ingresso e toglie il suo Parziale", () => {
  let view = [
    phrase(0, 0, "Uno. Due.", "microfono"),
    phrase(0, 500, "Sistema.", "sistema"),
  ]
    .map((p) => ({ ...p, sessionId: "live" }))
    .reduce(withPhrase, { ...EMPTY_CONVERSATION, sessionId: "live" });
  view = withPartial(view, {
    ...phrase(1, 3000, "Parzi", "microfono"),
    sessionId: "live",
  });
  view = withPartial(view, {
    ...phrase(1, 3500, "Altro", "sistema"),
    sessionId: "live",
  });
  view = withLiveTranscript(view, {
    ingresso: "microfono",
    phrases: [
      { ...phrase(0, 100, "Uno. ", "microfono"), parlante: 1 },
      { ...phrase(1, 1500, "Due.", "microfono"), parlante: 2 },
    ],
    sessionId: "live",
  });
  expect(visiblePhrases(view).map((p) => p.text)).toEqual([
    "Uno. ",
    "Sistema.",
    "Due.",
    "Altro",
  ]);
  expect(view.partials.map((p) => p.ingresso)).toEqual(["sistema"]);
});

test("un secondo snapshot dello stesso Ingresso nella stessa sessione si scarta", () => {
  const first = withLiveTranscript(
    { ...EMPTY_CONVERSATION, sessionId: "live" },
    {
      ingresso: "mix",
      phrases: [phrase(0, 0, "Testo finale.")],
      sessionId: "live",
    }
  );
  expect(
    withLiveTranscript(first, {
      ingresso: "mix",
      phrases: [phrase(0, 0, "Un altro testo.")],
      sessionId: "live",
    })
  ).toBe(first);
  const late = { ...phrase(0, 0, "Frase tardiva"), sessionId: "live" };
  expect(withPhrase(first, late)).toBe(first);
  expect(withPartial(first, late)).toBe(first);
  // L'altro Ingresso ha ancora il suo snapshot.
  const other = withLiveTranscript(first, {
    ingresso: "sistema",
    phrases: [phrase(0, 500, "Sistema.", "sistema")],
    sessionId: "live",
  });
  expect(other.phrases.map((p) => p.text)).toEqual([
    "Testo finale.",
    "Sistema.",
  ]);
});

test("la nuova Registrazione rifiuta ogni testo tardivo della precedente", () => {
  const current = { ...EMPTY_CONVERSATION, sessionId: "second" };
  const old = { ...phrase(0, 0, "Vecchia registrazione"), sessionId: "first" };
  expect(
    withLiveTranscript(current, {
      ingresso: "mix",
      phrases: [old],
      sessionId: "first",
    })
  ).toBe(current);
  expect(withPhrase(current, old)).toBe(current);
  expect(withPartial(current, old)).toBe(current);
  expect(withParlanti(current, [], "first")).toBe(current);
});

test("il fallback Ogg accetta lo snapshot finale dopo la risposta a Stop", () => {
  const snapshot = {
    ingresso: "mix" as const,
    phrases: [phrase(0, 0, "Testo finale completo")],
    sessionId: "current",
  };
  const final = withLiveTranscript(
    { ...EMPTY_CONVERSATION, sessionId: "current" },
    snapshot
  );
  expect(final.phrases[0]?.text).toBe("Testo finale completo");
  expect(final.partials).toEqual([]);
  // Una Sorgente già riaperta non viene sostituita dall'evento tardivo.
  expect(withLiveTranscript(EMPTY_CONVERSATION, snapshot)).toBe(
    EMPTY_CONVERSATION
  );
});

const t = ((key: string, options?: { n?: number }) =>
  ({
    "settings.recording.inputs.mic": "Microfono",
    "settings.recording.inputs.system": "Audio di sistema",
    "transcript.parlante": `Parlante ${options?.n}`,
    "transcript.unknownSpeaker": "Parlante non determinato",
  })[key] ?? key) as TFunction;

test("le attribuzioni provvisorie restano riconoscibili nella vista e in Copia turno", () => {
  const provisionalT = ((key: string, options?: { name?: string }) =>
    key === "transcript.provisionalSpeaker"
      ? `${options?.name} (provvisorio)`
      : t(key)) as TFunction;
  const conversation = withPhrase(EMPTY_CONVERSATION, {
    fineMs: 1000,
    ingresso: "mix",
    inizioMs: 0,
    parlante: 1,
    parlanteProvvisorio: true,
    phraseId: 0,
    text: "Testo conservato.",
  });
  const named = withNome(conversation, "mix", 1, "Mario");
  expect(turnsOf(named, provisionalT).map(labeled)).toEqual([
    "Mario (provvisorio): Testo conservato.",
  ]);
  expect(parlantiOf(named, provisionalT)[0]?.label).toBe("Mario (provvisorio)");
  expect(parlantiOf(named, provisionalT)[0]?.provvisorio).toBe(true);
});

test("il Parlante non determinato si legge e si copia senza diventare il nome del Microfono", () => {
  const conversation = withPhrase(EMPTY_CONVERSATION, {
    fineMs: 2000,
    ingresso: "microfono",
    inizioMs: 0,
    parlante: null,
    parlanteNonDeterminato: true,
    phraseId: 0,
    text: "Due voci nella stessa frase.",
  });
  const named = withNome(conversation, "microfono", null, "Mario");
  const turns = turnsOf(named, t);
  expect(turns[0]?.label).toBe("Microfono · Parlante non determinato");
  expect(turns[0]?.name).toBe("Parlante non determinato");
  expect(turns.map(labeled)).toEqual([
    "Microfono · Parlante non determinato: Due voci nella stessa frase.",
  ]);
  expect(parlantiOf(named, t)).toEqual([]);
});

const phrase = (
  phraseId: number,
  inizioMs: number,
  text: string,
  ingresso: Ingresso = "mix"
) => ({
  fineMs: inizioMs + 900,
  ingresso,
  inizioMs,
  parlante: null,
  phraseId,
  text,
});

/** I turni come testo: l'etichetta su una riga, una Frase per riga, una riga vuota tra i turni. */
const render = (c: Conversation) =>
  turnsOf(c, t)
    .map((turn) =>
      [turn.label && `${turn.label}:`, ...turn.items.map((i) => i.text)]
        .filter((line) => line !== null)
        .join("\n")
    )
    .join("\n\n");

test("le Frasi del mix finiscono una per riga, senza etichette", () => {
  const c = [
    phrase(0, 0, "Buongiorno a tutti."),
    phrase(1, 1500, "Oggi parliamo di trascrizione."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  expect(render(c)).toBe("Buongiorno a tutti.\nOggi parliamo di trascrizione.");
  expect(turnsOf(EMPTY_CONVERSATION, t)).toEqual([]);
});

test("le divisioni finali sostituiscono le Frasi per Ingresso senza duplicare il testo", () => {
  let conversation = [
    phrase(0, 0, "Uno. Due.", "microfono"),
    phrase(1, 3000, "Tre.", "microfono"),
    phrase(0, 500, "Sistema.", "sistema"),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  conversation = withPartial(
    conversation,
    phrase(2, 4500, "Parziale", "microfono")
  );
  const finalPhrases = [
    { ...phrase(0, 100, "Uno. ", "microfono"), parlante: 1 },
    { ...phrase(0, 500, "Sistema.", "sistema"), parlante: 1 },
    { ...phrase(1, 1500, "Due.", "microfono"), parlante: 2 },
    { ...phrase(2, 3000, "Tre.", "microfono"), parlante: 2 },
  ];
  // Le Frasi finali possono arrivare prima o dopo il risultato del comando record.
  for (const clearBefore of [false, true]) {
    let final = finalPhrases.reduce(
      withPhrase,
      clearBefore ? withoutPartials(conversation) : conversation
    );
    if (!clearBefore) {
      final = withoutPartials(final);
    }
    expect(final.phrases).toEqual(finalPhrases);
    expect(final.partials).toEqual([]);
    expect(
      final.phrases
        .filter((p) => p.ingresso === "microfono")
        .map((p) => p.text)
        .join("")
    ).toBe("Uno. Due.Tre.");
    expect(turnsOf(final, t).map(labeled)).toEqual([
      "Microfono · Parlante 1: Uno. ",
      "Audio di sistema · Parlante 1: Sistema.",
      "Microfono · Parlante 2: Due. Tre.",
    ]);
    expect(finalPhrases.reduce(withPhrase, final).phrases).toEqual(
      finalPhrases
    );
  }
});

test("il Parziale occupa la riga in corso e la Frase con lo stesso id lo fissa", () => {
  let c = withPhrase(EMPTY_CONVERSATION, phrase(0, 0, "Buongiorno a tutti."));
  c = withPartial(c, phrase(1, 1500, "Oggi parl"));
  expect(render(c)).toBe("Buongiorno a tutti.\nOggi parl");
  c = withPartial(c, phrase(1, 1500, "Oggi parliamo di"));
  expect(render(c)).toBe("Buongiorno a tutti.\nOggi parliamo di");
  // La Frase di un altro id lascia il Parziale; quella con il suo id lo sostituisce.
  expect(withPhrase(c, phrase(0, 0, "x")).partials).toHaveLength(1);
  c = withPhrase(c, phrase(1, 1500, "Oggi parliamo di trascrizione."));
  expect(c.partials).toEqual([]);
  expect(render(c)).toBe("Buongiorno a tutti.\nOggi parliamo di trascrizione.");
  // Un Parziale vuoto (Frase finita vuota) toglie la riga.
  c = withPartial(c, phrase(2, 4000, "Eh"));
  expect(render(withPartial(c, phrase(2, 4000, "")))).toBe(
    "Buongiorno a tutti.\nOggi parliamo di trascrizione."
  );
});

test("con gli Ingressi separati le Frasi sono in ordine di inizio, non di arrivo, e raggruppate per Ingresso", () => {
  const c = [
    phrase(0, 0, "Mi senti?", "microfono"),
    phrase(1, 1200, "Pronto?", "microfono"),
    // Il motore del microfono era avanti: la risposta arriva dopo, ma è iniziata prima.
    phrase(0, 1000, "Sì, ti sento.", "sistema"),
    phrase(2, 3000, "Bene.", "microfono"),
    phrase(1, 3000, "Anch'io.", "sistema"),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  expect(render(c)).toBe(
    [
      "Microfono:",
      "Mi senti?",
      "",
      "Audio di sistema:",
      "Sì, ti sento.",
      "",
      "Microfono:",
      "Pronto?",
      "Bene.",
      "",
      "Audio di sistema:",
      "Anch'io.",
    ].join("\n")
  );
});

test("ogni Ingresso ha il suo Parziale, al suo posto nella conversazione", () => {
  let c = withPhrase(
    EMPTY_CONVERSATION,
    phrase(0, 0, "Mi senti?", "microfono")
  );
  c = withPartial(c, phrase(1, 3000, "Allo", "microfono"));
  c = withPartial(c, phrase(0, 1000, "Sì, ti", "sistema"));
  expect(render(c)).toBe(
    "Microfono:\nMi senti?\n\nAudio di sistema:\nSì, ti\n\nMicrofono:\nAllo"
  );
  // La Frase 0 del microfono non fissa il Parziale 0 dell'audio di sistema.
  expect(withPhrase(c, phrase(0, 0, "x", "microfono")).partials).toHaveLength(
    2
  );
  c = withPhrase(c, phrase(0, 1000, "Sì, ti sento.", "sistema"));
  expect(c.partials).toEqual([phrase(1, 3000, "Allo", "microfono")]);
});

test("a fine Trascrizione i Parziali spariscono e le Frasi restano", () => {
  const c = withPartial(
    withPhrase(EMPTY_CONVERSATION, phrase(0, 0, "Mi senti?", "microfono")),
    phrase(0, 1000, "Sì", "sistema")
  );
  expect(withoutPartials(c)).toEqual({
    parlanti: {},
    partials: [],
    phrases: c.phrases,
  });
});

test("dopo la Diarizzazione ogni turno di un Parlante comincia con la sua etichetta", () => {
  let c = [
    phrase(0, 0, "Buongiorno."),
    phrase(1, 1000, "Cominciamo."),
    phrase(2, 2000, "Grazie."),
    phrase(3, 3000, "Boh."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  c = withParlanti(c, [
    { ingresso: "mix", parlante: 1, phraseId: 0 },
    { ingresso: "mix", parlante: 1, phraseId: 1 },
    { ingresso: "mix", parlante: 2, phraseId: 2 },
    { ingresso: "mix", parlante: null, phraseId: 3 },
  ]);
  expect(render(c)).toBe(
    "Parlante 1:\nBuongiorno.\nCominciamo.\n\nParlante 2:\nGrazie.\n\nBoh."
  );
});

test("con gli Ingressi separati l'etichetta unisce Ingresso e Parlante", () => {
  const c = withParlanti(
    [
      phrase(0, 0, "Mi senti?", "microfono"),
      phrase(0, 1000, "Sì.", "sistema"),
      phrase(1, 2000, "Anch'io.", "sistema"),
    ].reduce(withPhrase, EMPTY_CONVERSATION),
    [
      { ingresso: "sistema", parlante: 1, phraseId: 0 },
      { ingresso: "sistema", parlante: 2, phraseId: 1 },
    ]
  );
  expect(render(c)).toBe(
    "Microfono:\nMi senti?\n\nAudio di sistema · Parlante 1:\nSì.\n\nAudio di sistema · Parlante 2:\nAnch'io."
  );
});

const diarized = () =>
  withParlanti(
    [
      phrase(0, 0, "Mi senti?", "microfono"),
      phrase(0, 1000, "Sì.", "sistema"),
      phrase(1, 2000, "Anch'io.", "sistema"),
      phrase(1, 3000, "Bene.", "microfono"),
    ].reduce(withPhrase, EMPTY_CONVERSATION),
    [
      { ingresso: "microfono", parlante: 1, phraseId: 0 },
      { ingresso: "sistema", parlante: 1, phraseId: 0 },
      { ingresso: "sistema", parlante: 2, phraseId: 1 },
      { ingresso: "microfono", parlante: 1, phraseId: 1 },
    ]
  );

test("un Parlante rinominato ha il suo nome nell'etichetta, solo nel suo Ingresso", () => {
  const c = withNome(diarized(), "sistema", 1, "Mario");
  expect(c.parlanti).toEqual({ "sistema:1": "Mario" });
  expect(render(c)).toBe(
    "Microfono · Parlante 1:\nMi senti?\n\nAudio di sistema · Mario:\nSì.\n\nAudio di sistema · Parlante 2:\nAnch'io.\n\nMicrofono · Parlante 1:\nBene."
  );
  // Le Frasi e i Parziali che arrivano dopo non tolgono i nomi.
  expect(withPhrase(c, phrase(2, 4000, "Ok.")).parlanti).toEqual(c.parlanti);
  expect(withPartial(c, phrase(2, 4000, "O")).parlanti).toEqual(c.parlanti);
  expect(withoutPartials(c).parlanti).toEqual(c.parlanti);
});

test("i Parlanti sono in ordine di comparsa, una volta sola, con etichetta e nome", () => {
  const c = withNome(diarized(), "sistema", 2, "Lucia");
  expect(parlantiOf(c, t)).toEqual([
    {
      ingresso: "microfono",
      label: "Microfono · Parlante 1",
      nome: "Parlante 1",
      parlante: 1,
    },
    {
      ingresso: "sistema",
      label: "Audio di sistema · Parlante 1",
      nome: "Parlante 1",
      parlante: 1,
    },
    {
      ingresso: "sistema",
      label: "Audio di sistema · Lucia",
      nome: "Lucia",
      parlante: 2,
    },
  ]);
  expect(parlantiOf(EMPTY_CONVERSATION, t)).toEqual([]);
});

test("il Microfono non diarizzato è una persona sola e si rinomina come un Parlante", () => {
  const c = withParlanti(
    [
      phrase(0, 0, "Mi senti?", "microfono"),
      phrase(0, 1000, "Sì.", "sistema"),
      phrase(1, 2000, "Bene.", "microfono"),
    ].reduce(withPhrase, EMPTY_CONVERSATION),
    [{ ingresso: "sistema", parlante: 1, phraseId: 0 }]
  );
  const [microfono, sistema1] = parlantiOf(c, t);
  expect(microfono).toEqual({
    ingresso: "microfono",
    label: "Microfono",
    nome: "Microfono",
    parlante: null,
  });
  expect(sistema1?.label).toBe("Audio di sistema · Parlante 1");
  const named = withNome(c, "microfono", null, "Francesco");
  expect(named.parlanti).toEqual({ microfono: "Francesco" });
  expect(render(named)).toBe(
    "Microfono · Francesco:\nMi senti?\n\nAudio di sistema · Parlante 1:\nSì.\n\nMicrofono · Francesco:\nBene."
  );
  // Il mix senza Parlanti non ha nulla da rinominare.
  expect(
    parlantiOf(withPhrase(EMPTY_CONVERSATION, phrase(0, 0, "Ciao.")), t)
  ).toEqual([]);
});

test("un nome già di un altro Parlante dello stesso Ingresso non si accetta", () => {
  const list = parlantiOf(withNome(diarized(), "sistema", 2, "Lucia"), t);
  const [microfono1, sistema1] = list;
  expect(nomeTaken(list, sistema1, "Lucia")).toBe(true);
  expect(nomeTaken(list, sistema1, "Parlante 2")).toBe(false);
  // Il suo stesso nome, o quello di un Parlante di un altro Ingresso, sì.
  expect(nomeTaken(list, sistema1, "Parlante 1")).toBe(false);
  expect(nomeTaken(list, microfono1, "Lucia")).toBe(false);
});

test("ogni turno ha la voce da rinominare e una chiave stabile", () => {
  const turns = turnsOf(withNome(diarized(), "sistema", 2, "Lucia"), t);
  expect(turns.map((turn) => turn.name)).toEqual([
    "Parlante 1",
    "Parlante 1",
    "Lucia",
    "Parlante 1",
  ]);
  expect(
    turns.map(({ ingresso, key, label, parlante }) => ({
      ingresso,
      key,
      label,
      parlante,
    }))
  ).toEqual([
    {
      ingresso: "microfono",
      key: "microfono:0",
      label: "Microfono · Parlante 1",
      parlante: 1,
    },
    {
      ingresso: "sistema",
      key: "sistema:0",
      label: "Audio di sistema · Parlante 1",
      parlante: 1,
    },
    {
      ingresso: "sistema",
      key: "sistema:1",
      label: "Audio di sistema · Lucia",
      parlante: 2,
    },
    {
      ingresso: "microfono",
      key: "microfono:1",
      label: "Microfono · Parlante 1",
      parlante: 1,
    },
  ]);
  // Un turno che comincia con un Parziale non si confonde con la Frase dello stesso id.
  const live = withPartial(
    withPhrase(EMPTY_CONVERSATION, phrase(0, 0, "Mi senti?", "microfono")),
    phrase(0, 1000, "Sì", "sistema")
  );
  expect(turnsOf(live, t).map((turn) => turn.key)).toEqual([
    "microfono:0",
    "sistema:0:parziale",
  ]);
});

test("la correzione cambia solo il testo della sua Frase", () => {
  const c = diarized();
  const corrected = withTesto(
    c,
    { ingresso: "sistema", phraseId: 1 },
    "Anche io."
  );
  expect(corrected.phrases).toEqual(
    c.phrases.map((p) =>
      p.ingresso === "sistema" && p.phraseId === 1
        ? { ...p, text: "Anche io." }
        : p
    )
  );
  expect(corrected.parlanti).toBe(c.parlanti);
  // Una Frase svuotata resta al suo posto.
  const emptied = withTesto(c, { ingresso: "microfono", phraseId: 0 }, "");
  expect(emptied.phrases).toHaveLength(4);
  expect(emptied.phrases[0]?.text).toBe("");
});

test("senza etichette un turno si spezza dopo oltre 2 s di silenzio, come nel Markdown", () => {
  // Fine a +900 ms; il silenzio reale è il buco più hangover e prefill (1 s).
  const c = [
    phrase(0, 0, "Uno."),
    phrase(1, 1900, "Due."),
    phrase(2, 3900, "Tre."),
    phrase(3, 4800, "Quattro."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  expect(render(c)).toBe("Uno.\nDue.\n\nTre.\nQuattro.");
  expect(turnsOf(c, t).map((turn) => turn.key)).toEqual(["mix:0", "mix:2"]);
});

test("ogni voce ha il suo colore per ordine di comparsa, anche rinominata", () => {
  const colors = voiceColors(turnsOf(diarized(), t));
  expect([...colors]).toEqual([
    ["Microfono · Parlante 1", 0],
    ["Audio di sistema · Parlante 1", 1],
    ["Audio di sistema · Parlante 2", 2],
  ]);
  const renamed = voiceColors(
    turnsOf(withNome(diarized(), "sistema", 2, "Lucia"), t)
  );
  expect(renamed.get("Audio di sistema · Lucia")).toBe(2);
  // Il mix senza Parlanti non ha etichette, quindi nemmeno colori.
  const mix = [phrase(0, 0, "Ciao.")].reduce(withPhrase, EMPTY_CONVERSATION);
  expect(voiceColors(turnsOf(mix, t)).size).toBe(0);
  // Oltre i colori disponibili si ricomincia dal primo.
  const many = withParlanti(
    Array.from({ length: VOICE_COLORS + 1 }, (_, i) =>
      phrase(i, i * 1000, `${i}.`)
    ).reduce(withPhrase, EMPTY_CONVERSATION),
    Array.from({ length: VOICE_COLORS + 1 }, (_, i) => ({
      ingresso: "mix" as const,
      parlante: i + 1,
      phraseId: i,
    }))
  );
  expect(
    voiceColors(turnsOf(many, t)).get(`Parlante ${VOICE_COLORS + 1}`)
  ).toBe(0);
});

test("di un Parlante si contano turni, tempo di parola e prima comparsa", () => {
  const c = diarized();
  const turns = turnsOf(c, t);
  const [microfono1, , sistema2] = parlantiOf(c, t);
  expect(parlanteStats(c, turns, microfono1)).toEqual({
    firstMs: 0,
    talkMs: 1800,
    turns: 2,
  });
  expect(parlanteStats(c, turns, sistema2)).toEqual({
    firstMs: 2000,
    talkMs: 900,
    turns: 1,
  });
});

test("Copia turno dà solo le Frasi del turno, senza la voce, con il Parziale com'è", () => {
  const [, , lucia] = turnsOf(withNome(diarized(), "sistema", 2, "Lucia"), t);
  expect(lucia?.label).toBe("Audio di sistema · Lucia");
  expect(turnText(lucia)).toBe("Anch'io.");

  let mix = [
    phrase(0, 0, "Buongiorno."),
    phrase(1, 1000, "Oggi parliamo."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  mix = withPartial(mix, phrase(2, 2000, "Di cas"));
  expect(turnsOf(mix, t).map(turnText)).toEqual([
    "Buongiorno. Oggi parliamo. Di cas",
  ]);
});
