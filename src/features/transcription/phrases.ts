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

/** La chiave del Parlante `n` di un Ingresso tra i nomi, come nel Tape: `sistema:2`. */
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

/** Una Frase di un Tape, come nei `TranscriptPhrase`. */
export interface PhraseRef {
  ingresso: Ingresso;
  phraseId: number;
}

/** Ingresso e id: identifica una Frase tra quelle di tutti gli Ingressi. */
export function phraseKey(p: PhraseRef): string {
  return `${p.ingresso}:${p.phraseId}`;
}

/** La correzione del testo di una Frase; tempi e Parlante restano. Una Frase svuotata resta. */
export function withTesto(
  conversation: Conversation,
  { ingresso, phraseId }: PhraseRef,
  text: string
): Conversation {
  return {
    ...conversation,
    phrases: conversation.phrases.map((p) =>
      p.ingresso === ingresso && p.phraseId === phraseId ? { ...p, text } : p
    ),
  };
}

/** Un turno di una voce: le sue Frasi (e Parziali) consecutive, con l'etichetta della voce. */
export interface Turn {
  ingresso: Ingresso;
  items: (TranscriptPhrase | TranscriptPartial)[];
  /** Ingresso e id del primo elemento: resta lo stesso mentre il turno cresce. */
  key: string;
  /** `Microfono · Parlante 2`, `Microfono`, `Parlante 2`; `null` senza etichetta (il mix). */
  label: string | null;
  parlante: number | null;
}

/** Oltre questo silenzio, senza etichette, comincia un paragrafo nuovo: come `transcript::render`. */
const PARAGRAPH_PAUSE_MS = 2000;
/** Hangover e prefill del segmentatore, compresi nei tempi delle due Frasi ai lati di un silenzio. */
const HANGOVER_PREFILL_MS = 1000;

/** Il silenzio reale tra due Frasi, come `transcript::silence_ms`; contigue (tagliate a 18 s) è 0. */
function silenceMs(
  previous: { fineMs: number },
  next: { inizioMs: number }
): number {
  const gap = Math.max(0, next.inizioMs - previous.fineMs);
  return gap === 0 ? 0 : gap + HANGOVER_PREFILL_MS;
}

/**
 * La conversazione a turni, in ordine di inizio con i Parziali al loro posto: un turno nuovo a ogni
 * cambio di etichetta (Ingresso o Parlante) o, senza etichette, dopo una pausa: come nel Markdown.
 */
export function turnsOf(conversation: Conversation, t: TFunction): Turn[] {
  // Senza etichette i turni sono i paragrafi del Markdown, separati dalle pause.
  const labeled = conversation.phrases.some(
    (p) => voiceLabel(conversation, p.ingresso, p.parlante, t) !== null
  );
  const turns: Turn[] = [];
  let last: Turn | undefined;
  for (const item of conversation.partials.reduce<
    (TranscriptPhrase | TranscriptPartial)[]
  >(byStart, conversation.phrases)) {
    const partial = conversation.partials.includes(item as TranscriptPartial);
    const parlante = partial
      ? null
      : ((item as TranscriptPhrase).parlante ?? null);
    const label = voiceLabel(conversation, item.ingresso, parlante, t);
    const previous = last?.items[last.items.length - 1];
    if (
      last &&
      previous &&
      last.label === label &&
      (labeled || silenceMs(previous, item) <= PARAGRAPH_PAUSE_MS)
    ) {
      last.items.push(item);
    } else {
      last = {
        ingresso: item.ingresso,
        items: [item],
        key: `${phraseKey(item)}${partial ? ":parziale" : ""}`,
        label,
        parlante,
      };
      turns.push(last);
    }
  }
  return turns;
}

/** Il testo di Copia turno, sempre semplice: `Nome: Frasi…`; un turno senza etichetta dà solo le Frasi. */
export function turnText(turn: Turn): string {
  const text = turn.items.map((item) => item.text).join(" ");
  return turn.label ? `${turn.label}: ${text}` : text;
}

/** Quanti colori hanno le voci: oltre, si ricomincia dal primo. */
export const VOICE_COLORS = 6;

/**
 * Il colore (da 0) di ogni etichetta dei turni, per ordine di comparsa: una voce rinominata resta
 * nello stesso posto, quindi tiene il suo colore.
 */
export function voiceColors(turns: Turn[]): Map<string, number> {
  const colors = new Map<string, number>();
  for (const { label } of turns) {
    if (label !== null && !colors.has(label)) {
      colors.set(label, colors.size % VOICE_COLORS);
    }
  }
  return colors;
}

/** Quanto ha parlato un Parlante: i suoi turni, la somma delle sue Frasi e quando compare. */
export interface ParlanteStats {
  firstMs: number;
  talkMs: number;
  turns: number;
}

export function parlanteStats(
  conversation: Conversation,
  turns: Turn[],
  { ingresso, parlante }: Parlante
): ParlanteStats {
  const own = conversation.phrases.filter(
    (p) => p.ingresso === ingresso && p.parlante === parlante
  );
  return {
    firstMs: Math.min(...own.map((p) => p.inizioMs)),
    talkMs: own.reduce((sum, p) => sum + p.fineMs - p.inizioMs, 0),
    turns: turns.filter(
      (t) => t.ingresso === ingresso && t.parlante === parlante
    ).length,
  };
}
