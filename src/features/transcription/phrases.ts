import type { TFunction } from "i18next";
import type { Ingresso, TranscriptPartial, TranscriptPhrase } from "@/bindings";

/**
 * Il testo di una Trascrizione mentre arriva: le Frasi in ordine di inizio e il Parziale in corso di
 * ogni Ingresso (uno solo, `mix`, senza Ingressi separati).
 */
export interface Conversation {
  partials: TranscriptPartial[];
  phrases: TranscriptPhrase[];
}

export const EMPTY_CONVERSATION: Conversation = { partials: [], phrases: [] };

/** `item` dopo gli elementi che iniziano prima o insieme a lui. */
function byStart<T extends { inizioMs: number }>(list: T[], item: T): T[] {
  const at = list.findIndex((other) => other.inizioMs > item.inizioMs);
  return at === -1
    ? [...list, item]
    : [...list.slice(0, at), item, ...list.slice(at)];
}

/**
 * Aggiunge una Frase in ordine di inizio: con gli Ingressi separati può arrivare dopo una Frase
 * iniziata più tardi. Sostituisce il Parziale del suo Ingresso con lo stesso id.
 */
export function withPhrase(
  conversation: Conversation,
  phrase: TranscriptPhrase
): Conversation {
  return {
    partials: conversation.partials.filter(
      (p) => p.ingresso !== phrase.ingresso || p.phraseId !== phrase.phraseId
    ),
    phrases: byStart(conversation.phrases, phrase),
  };
}

/** Il Parziale sostituisce il precedente del suo Ingresso; se è vuoto lo toglie e basta. */
export function withPartial(
  conversation: Conversation,
  partial: TranscriptPartial
): Conversation {
  const others = conversation.partials.filter(
    (p) => p.ingresso !== partial.ingresso
  );
  return {
    ...conversation,
    partials: partial.text ? [...others, partial] : others,
  };
}

/** La conversazione senza Parziali: a fine Trascrizione, o se quella dal vivo si ferma. */
export function withoutPartials(conversation: Conversation): Conversation {
  return { ...conversation, partials: [] };
}

const INGRESSO_LABELS = {
  microfono: "settings.recording.inputs.mic",
  sistema: "settings.recording.inputs.system",
} as const;

/**
 * Il testo dell'area: una Frase per riga in ordine di inizio, con i Parziali al loro posto. Con gli
 * Ingressi separati è una conversazione: ogni turno di un Ingresso comincia con la sua etichetta
 * (`Microfono:`) su una riga, dopo una riga vuota.
 */
export function conversationText(
  phrases: TranscriptPhrase[],
  partials: TranscriptPartial[],
  t: TFunction
): string {
  const lines: string[] = [];
  let previous: Ingresso = "mix";
  for (const { ingresso, text } of partials.reduce(byStart, phrases)) {
    if (ingresso !== "mix" && ingresso !== previous) {
      if (lines.length > 0) {
        lines.push("");
      }
      lines.push(`${t(INGRESSO_LABELS[ingresso])}:`);
    }
    previous = ingresso;
    lines.push(text);
  }
  return lines.join("\n");
}
