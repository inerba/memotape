import { expect, test } from "bun:test";
import type { TFunction } from "i18next";
import type { Ingresso } from "@/bindings";
import {
  type Conversation,
  conversationText,
  EMPTY_CONVERSATION,
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

const render = (c: Conversation) => conversationText(c.phrases, c.partials, t);

test("le Frasi del mix finiscono una per riga, senza etichette", () => {
  const c = [
    phrase(0, 0, "Buongiorno a tutti."),
    phrase(1, 1500, "Oggi parliamo di trascrizione."),
  ].reduce(withPhrase, EMPTY_CONVERSATION);
  expect(render(c)).toBe("Buongiorno a tutti.\nOggi parliamo di trascrizione.");
  expect(conversationText([], [], t)).toBe("");
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
  expect(withoutPartials(c)).toEqual({ partials: [], phrases: c.phrases });
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
