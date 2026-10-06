import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import "@/lib/i18n";
import type { Conversation } from "./phrases";
import { TranscriptView } from "./transcript-view";

for (const parlante of [null, 1]) {
  test(`le Frasi restano in paragrafi distinti nello stesso turno ${parlante === null ? "senza voce" : "con voce"}, anche dal vivo`, () => {
    const conversation: Conversation = {
      parlanti: {},
      partials: [
        {
          fineMs: 2500,
          ingresso: "mix",
          inizioMs: 2000,
          parlante,
          phraseId: 2,
          text: "Terza frase in corso",
        },
      ],
      phrases: [
        {
          fineMs: 1000,
          ingresso: "mix",
          inizioMs: 0,
          parlante,
          phraseId: 0,
          text: "Prima frase conclusa.",
        },
        {
          fineMs: 2000,
          ingresso: "mix",
          inizioMs: 1000,
          parlante,
          phraseId: 1,
          text: "Seconda frase conclusa.",
        },
      ],
    };
    const html = renderToStaticMarkup(
      <TranscriptView conversation={conversation} parlanti={[]} />
    );
    const paragraphs = [...html.matchAll(/<p\b[^>]*>[\s\S]*?<\/p>/g)].map(
      ([markup]) => markup
    );

    expect([...html.matchAll(/<article\b/g)]).toHaveLength(1);
    expect(paragraphs).toHaveLength(3);
    for (const [index, text] of [
      "Prima frase conclusa.",
      "Seconda frase conclusa.",
      "Terza frase in corso",
    ].entries()) {
      expect(paragraphs[index]).toContain(text);
      expect(paragraphs[index]).toContain(`data-phrase="mix:${index}"`);
      expect([...paragraphs[index].matchAll(/data-phrase=/g)]).toHaveLength(1);
    }
  });
}
