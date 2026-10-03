import type { TFunction } from "i18next";
import type {
  Ingresso,
  SpeakerAssignment,
  TranscriptPartial,
  TranscriptPhrase,
} from "@/bindings";

/**
 * Il testo di una Trascrizione mentre arriva: le Frasi in ordine di inizio, il Parziale in corso di
 * ogni Ingresso (uno solo, `mix`, senza Ingressi separati) e i nomi dati ai Parlanti, per chiave
 * `<ingresso>:<n>` (`parlanteKey`).
 */
export interface Conversation {
  parlanti: Partial<Record<string, string>>;
  partials: TranscriptPartial[];
  phrases: TranscriptPhrase[];
}

export const EMPTY_CONVERSATION: Conversation = {
  parlanti: {},
  partials: [],
  phrases: [],
};

/** La chiave del Parlante `n` di un Ingresso tra i nomi, come nel Bino: `sistema:2`. */
function parlanteKey(ingresso: Ingresso, n: number): string {
  return `${ingresso}:${n}`;
}

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
    ...conversation,
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

/** Il nome dato al Parlante `parlante` di `ingresso`, per tutte le sue Frasi. */
export function withNome(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number,
  nome: string
): Conversation {
  return {
    ...conversation,
    parlanti: {
      ...conversation.parlanti,
      [parlanteKey(ingresso, parlante)]: nome,
    },
  };
}

const INGRESSO_LABELS = {
  microfono: "settings.recording.inputs.mic",
  sistema: "settings.recording.inputs.system",
} as const;

/** Il nome del Parlante: quello dato con la rinomina, o `Parlante 2`. */
function nomeOf(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number,
  t: TFunction
): string {
  return (
    conversation.parlanti[parlanteKey(ingresso, parlante)] ??
    t("transcript.parlante", { n: parlante })
  );
}

/** `Microfono · Parlante 2`, `Microfono`, `Parlante 2` o nessuna etichetta, come nel Markdown. */
function voiceLabel(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number | null | undefined,
  t: TFunction
): string | null {
  const parts = [
    ingresso === "mix" ? null : t(INGRESSO_LABELS[ingresso]),
    parlante ? nomeOf(conversation, ingresso, parlante, t) : null,
  ].filter((part) => part !== null);
  return parts.length > 0 ? parts.join(" · ") : null;
}

/** Un Parlante del testo, con l'etichetta dei suoi turni e il nome da cui parte la rinomina. */
export interface Parlante {
  ingresso: Ingresso;
  label: string;
  nome: string;
  parlante: number;
}

/** I Parlanti delle Frasi, in ordine di comparsa. */
export function parlantiOf(
  conversation: Conversation,
  t: TFunction
): Parlante[] {
  const found: Parlante[] = [];
  for (const { ingresso, parlante } of conversation.phrases) {
    if (
      parlante &&
      !found.some((s) => s.ingresso === ingresso && s.parlante === parlante)
    ) {
      found.push({
        ingresso,
        label: voiceLabel(conversation, ingresso, parlante, t) ?? "",
        nome: nomeOf(conversation, ingresso, parlante, t),
        parlante,
      });
    }
  }
  return found;
}

/** La riga di `text` in cui cade la posizione `at`. */
function lineAt(text: string, at: number): string {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const end = text.indexOf("\n", at);
  return text.slice(start, end === -1 ? undefined : end);
}

/** Il Parlante della riga di etichetta (`Parlante 1:`) su cui si è cliccato, se lo è. */
export function parlanteAt(
  text: string,
  at: number,
  list: Parlante[]
): Parlante | null {
  const line = lineAt(text, at);
  return list.find((s) => line === `${s.label}:`) ?? null;
}

/**
 * Se `nome` è già il nome di un altro Parlante dello stesso Ingresso: i loro turni si unirebbero e le
 * etichette non si distinguerebbero più.
 */
export function nomeTaken(
  list: Parlante[],
  voce: Parlante,
  nome: string
): boolean {
  return list.some(
    (other) =>
      other.ingresso === voce.ingresso &&
      other.parlante !== voce.parlante &&
      other.nome === nome
  );
}

/**
 * Il testo modificato a mano con le righe di etichetta di `voce` che hanno il nome `nome`. Il nome è
 * sempre in fondo all'etichetta (`voiceLabel`), dopo l'Ingresso.
 */
export function relabeled(text: string, voce: Parlante, nome: string): string {
  const from = voce.label;
  const to = from.slice(0, from.length - voce.nome.length) + nome;
  return text
    .split("\n")
    .map((line) => (line === `${from}:` ? `${to}:` : line))
    .join("\n");
}

/**
 * Il testo dell'area: una Frase per riga in ordine di inizio, con i Parziali al loro posto. Con gli
 * Ingressi separati o i Parlanti è una conversazione: ogni turno di una voce comincia con la sua
 * etichetta (`Microfono:`, `Parlante 1:`) su una riga, dopo una riga vuota.
 */
export function conversationText(
  conversation: Conversation,
  t: TFunction
): string {
  const lines: string[] = [];
  let previous: string | null = null;
  for (const item of conversation.partials.reduce<
    (TranscriptPhrase | TranscriptPartial)[]
  >(byStart, conversation.phrases)) {
    const label = voiceLabel(
      conversation,
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
