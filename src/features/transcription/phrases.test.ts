import { expect, test } from "bun:test";
import type { TFunction } from "i18next";
import type { Ingresso } from "@/bindings";
import {
  type Conversation,
  conversationText,
  copyable,
  EMPTY_CONVERSATION,
  nomeTaken,
  parlanteAt,
  parlantiOf,
  phraseRange,
  relabeled,
  shownText,
  withNome,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
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

const render = (c: Conversation) => conversationText(c, t);

test("le Frasi del mix finiscono una per riga, senza etichette", () => {
  const c = [
    phrase(0, 0, "Buongiorno a tutti."),
    phrase(1, 1500, "Oggi parliamo di trascrizione."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  expect(render(c)).toBe("Buongiorno a tutti.\nOggi parliamo di trascrizione.");
  expect(conversationText(EMPTY_CONVERSATION, t)).toBe("");
});

test("durante la Trascrizione di un file l'area è vuota, dal vivo mostra la conversazione", () => {
  let c = withPhrase(EMPTY_CONVERSATION, phrase(0, 0, "Buongiorno a tutti."));
  c = withPartial(c, phrase(1, 1500, "Oggi parl"));
  expect(shownText(true, true, c, "Buongiorno a tutti.", t)).toBe("");
  expect(shownText(false, true, c, "", t)).toBe(
    "Buongiorno a tutti.\nOggi parl"
  );
  // Finita l'Attività, il testo dell'area, anche modificato a mano.
  expect(shownText(false, false, c, "testo mio", t)).toBe("testo mio");
  // Copia testo: niente durante la Trascrizione di un file, né con solo un Parziale dal vivo.
  expect(copyable(true, "Buongiorno a tutti.")).toBe(false);
  expect(copyable(false, "")).toBe(false);
  expect(copyable(false, "Buongiorno a tutti.")).toBe(true);
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

test("il clic su una riga di etichetta trova il suo Parlante, altrove nessuno", () => {
  const c = diarized();
  const text = render(c);
  const list = parlantiOf(c, t);
  const at = (line: string) => parlanteAt(text, text.indexOf(line) + 2, list);
  expect(at("Audio di sistema · Parlante 2:")).toEqual(list[2]);
  expect(parlanteAt(text, 0, list)).toEqual(list[0]);
  expect(at("Anch'io.")).toBeNull();
  // Fine dell'etichetta: il cursore dopo i due punti è ancora sulla sua riga.
  const end = text.indexOf("Audio di sistema · Parlante 1:") + 30;
  expect(parlanteAt(text, end, list)).toEqual(list[1]);
});

test("nel testo modificato a mano la rinomina cambia solo le righe dell'etichetta", () => {
  const voce = (label: string, nome: string) => ({
    ingresso: "sistema" as const,
    label,
    nome,
    parlante: 1,
  });
  const text =
    "Parlante 1:\nCiao Parlante 1:\n\nParlante 10:\nNo.\n\nParlante 1:\nSì.";
  expect(relabeled(text, voce("Parlante 1", "Parlante 1"), "Mario")).toBe(
    "Mario:\nCiao Parlante 1:\n\nParlante 10:\nNo.\n\nMario:\nSì."
  );
  // Con l'Ingresso nell'etichetta cambia solo il nome.
  expect(
    relabeled(
      "Audio di sistema · Mario:\nSì.",
      voce("Audio di sistema · Mario", "Mario"),
      "Lucia"
    )
  ).toBe("Audio di sistema · Lucia:\nSì.");
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

test("phraseRange trova la Frase nel testo dell'area", () => {
  const conversation = {
    ...EMPTY_CONVERSATION,
    phrases: [
      phrase(0, 0, "Ciao.", "microfono"),
      phrase(0, 500, "Salve.", "sistema"),
      phrase(1, 1000, "Come va?", "microfono"),
    ],
  };
  const text = render(conversation);
  const range = phraseRange(
    text,
    conversation,
    { ingresso: "microfono", phraseId: 1 },
    t
  );
  expect(range && text.slice(range.start, range.end)).toBe("Come va?");
  expect(
    phraseRange(text, conversation, { ingresso: "sistema", phraseId: 0 }, t)
  ).toEqual({
    end: text.indexOf("Salve.") + "Salve.".length,
    start: text.indexOf("Salve."),
  });
  // Testo modificato a mano: la Frase si cerca per il suo testo.
  const edited = `Premessa.\n${text}`;
  const moved = phraseRange(
    edited,
    conversation,
    { ingresso: "microfono", phraseId: 1 },
    t
  );
  expect(moved && edited.slice(moved.start, moved.end)).toBe("Come va?");
  expect(
    phraseRange(
      "altro",
      conversation,
      { ingresso: "microfono", phraseId: 1 },
      t
    )
  ).toBeNull();
  expect(
    phraseRange(text, conversation, { ingresso: "mix", phraseId: 7 }, t)
  ).toBeNull();
});
