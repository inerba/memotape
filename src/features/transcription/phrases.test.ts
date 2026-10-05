import { expect, test } from "bun:test";
import type { TFunction } from "i18next";
import type { Ingresso } from "@/bindings";
import {
  type Conversation,
  EMPTY_CONVERSATION,
  nomeTaken,
  parlanteStats,
  parlantiOf,
  turnsOf,
  turnText,
  VOICE_COLORS,
  voiceColors,
  withNome,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
  withTesto,
} from "@/features/transcription/phrases";

const t = ((key: string, options?: { n?: number }) =>
  ({
    "settings.recording.inputs.mic": "Microfono",
    "settings.recording.inputs.system": "Audio di sistema",
    "transcript.parlante": `Parlante ${options?.n}`,
  })[key] ?? key) as TFunction;

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

test("Copia turno dà il nome della voce e le Frasi del turno, con il Parziale com'è", () => {
  const [, , lucia] = turnsOf(withNome(diarized(), "sistema", 2, "Lucia"), t);
  expect(turnText(lucia)).toBe("Audio di sistema · Lucia: Anch'io.");

  let mix = [
    phrase(0, 0, "Buongiorno."),
    phrase(1, 1000, "Oggi parliamo."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  mix = withPartial(mix, phrase(2, 2000, "Di cas"));
  expect(turnsOf(mix, t).map(turnText)).toEqual([
    "Buongiorno. Oggi parliamo. Di cas",
  ]);
});
