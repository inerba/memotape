import type { TFunction } from "i18next";
import type {
  Ingresso,
  SpeakerAssignment,
  TranscriptPartial,
  TranscriptPhrase,
} from "@/bindings";

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

/** Applica `speakers-assigned`: il Parlante di ogni Frase dopo la Diarizzazione. */
export function withParlanti(
  conversation: Conversation,
  assignments: SpeakerAssignment[]
): Conversation {
  return {
    ...conversation,
    phrases: conversation.phrases.map((phrase) => {
      const assigned = assignments.find(
        (s) => s.ingresso === phrase.ingresso && s.phraseId === phrase.phraseId
      );
      return assigned ? { ...phrase, parlante: assigned.parlante } : phrase;
    }),
  };
}

const INGRESSO_LABELS = {
  microfono: "settings.recording.inputs.mic",
  sistema: "settings.recording.inputs.system",
} as const;

/** `Microfono · Parlante 2`, `Microfono`, `Parlante 2` o nessuna etichetta, come nel Markdown. */
function voiceLabel(
  ingresso: Ingresso,
  parlante: number | null | undefined,
  t: TFunction
): string | null {
  const parts = [
    ingresso === "mix" ? null : t(INGRESSO_LABELS[ingresso]),
    parlante ? t("transcript.parlante", { n: parlante }) : null,
  ].filter((part) => part !== null);
  return parts.length > 0 ? parts.join(" · ") : null;
}

/**
 * Il testo dell'area: una Frase per riga in ordine di inizio, con i Parziali al loro posto. Con gli
 * Ingressi separati o i Parlanti è una conversazione: ogni turno di una voce comincia con la sua
 * etichetta (`Microfono:`, `Parlante 1:`) su una riga, dopo una riga vuota.
 */
export function conversationText(
  phrases: TranscriptPhrase[],
  partials: TranscriptPartial[],
  t: TFunction
): string {
  const lines: string[] = [];
  let previous: string | null = null;
  for (const item of partials.reduce<(TranscriptPhrase | TranscriptPartial)[]>(
    byStart,
    phrases
  )) {
    const label = voiceLabel(
      item.ingresso,
      "parlante" in item ? item.parlante : null,
      t
    );
    if (label !== previous) {
      if (lines.length > 0) {
        lines.push("");
      }
      if (label) {
        lines.push(`${label}:`);
      }
    }
    previous = label;
    lines.push(item.text);
  }
  return lines.join("\n");
}
