import type { TFunction } from "i18next";
import type {
  Ingresso,
  LiveTranscriptUpdated,
  SpeakerAssignment,
  TranscriptPartial,
  TranscriptPhrase,
} from "@/bindings";

/**
 * Il Parziale ASR. Dal vivo non ha Parlante: i campi facoltativi servono a leggere Frasi e Parziali
 * allo stesso modo nei turni.
 */
export type ConversationPartial = TranscriptPartial &
  Partial<
    Pick<
      TranscriptPhrase,
      "parlante" | "parlanteNonDeterminato" | "parlanteProvvisorio"
    >
  >;

/**
 * Il testo di una Trascrizione mentre arriva: le Frasi in ordine di inizio, il Parziale di ogni
 * Ingresso (`mix`, senza Ingressi separati) e i nomi dati ai Parlanti, per chiave
 * `<ingresso>:<n>`, o `<ingresso>` per un Ingresso senza Parlanti (`parlanteKey`).
 */
export interface Conversation {
  parlanti: Partial<Record<string, string>>;
  partials: ConversationPartial[];
  phrases: TranscriptPhrase[];
}

export const EMPTY_CONVERSATION: Conversation = {
  parlanti: {},
  partials: [],
  phrases: [],
};

/** Il testo in vista: Frasi e Parziali in ordine di inizio. */
export function visiblePhrases(conversation: Conversation): TranscriptPhrase[] {
  return [...conversation.phrases, ...conversation.partials]
    .sort((a, b) => a.inizioMs - b.inizioMs)
    .map((p) => ({
      ...p,
      parlante: p.parlante ?? null,
      parlanteNonDeterminato: p.parlanteNonDeterminato ?? false,
      parlanteProvvisorio: p.parlanteProvvisorio ?? false,
    }));
}

/**
 * La chiave del Parlante `n` di un Ingresso tra i nomi, come nel Tape: `sistema:2`; senza Parlante
 * quella dell'Ingresso, `microfono` (il Microfono non diarizzato è una persona sola).
 */
function parlanteKey(ingresso: Ingresso, n: number | null): string {
  return n === null ? ingresso : `${ingresso}:${n}`;
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
 * iniziata più tardi. Sostituisce la Frase e il Parziale dello stesso Ingresso con lo stesso id,
 * anche quando l'analisi finale ripubblica il risultato diviso.
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
    phrases: byStart(
      conversation.phrases.filter(
        (p) => p.ingresso !== phrase.ingresso || p.phraseId !== phrase.phraseId
      ),
      phrase
    ),
  };
}

/** Lo snapshot finale di un Ingresso della Registrazione sostituisce le sue Frasi e il suo Parziale. */
export function withLiveTranscript(
  conversation: Conversation,
  { ingresso, phrases }: LiveTranscriptUpdated
): Conversation {
  return {
    ...conversation,
    partials: conversation.partials.filter((p) => p.ingresso !== ingresso),
    phrases: [
      ...conversation.phrases.filter((p) => p.ingresso !== ingresso),
      ...phrases,
    ].sort((a, b) => a.inizioMs - b.inizioMs),
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
      return assigned
        ? {
            ...phrase,
            parlante: assigned.parlante,
            parlanteNonDeterminato: assigned.parlanteNonDeterminato,
            parlanteProvvisorio: assigned.parlanteProvvisorio,
          }
        : phrase;
    }),
  };
}

/** Il nome dato al Parlante `parlante` di `ingresso` (o all'Ingresso), per tutte le sue Frasi. */
export function withNome(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number | null,
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

/**
 * Il nome del Parlante: quello dato con la rinomina, o `Parlante 2`. Senza Parlante, il nome dato
 * all'Ingresso, se c'è.
 */
function nomeOf(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number | null,
  t: TFunction
): string | null {
  return (
    conversation.parlanti[parlanteKey(ingresso, parlante)] ??
    (parlante === null ? null : t("transcript.parlante", { n: parlante }))
  );
}

/**
 * `Microfono · Parlante 2`, `Microfono`, `Microfono · Mario` (l'Ingresso rinominato), `Parlante 2` o
 * nessuna etichetta, come nel Markdown.
 */
function voiceLabel(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number | null | undefined,
  t: TFunction,
  nonDeterminato = false,
  provvisorio = false
): string | null {
  const name = voiceName(
    conversation,
    ingresso,
    parlante,
    t,
    nonDeterminato,
    provvisorio
  );
  const parts = [
    ingresso === "mix" ? null : t(INGRESSO_LABELS[ingresso]),
    name,
  ].filter((part) => part !== null);
  return parts.length > 0 ? parts.join(" · ") : null;
}

/** Il nome visibile accanto all'icona dell'Ingresso; l'etichetta testuale resta per Copia turno. */
function voiceName(
  conversation: Conversation,
  ingresso: Ingresso,
  parlante: number | null | undefined,
  t: TFunction,
  nonDeterminato = false,
  provvisorio = false
): string | null {
  const name = nonDeterminato
    ? t("transcript.unknownSpeaker")
    : nomeOf(conversation, ingresso, parlante ?? null, t);
  return provvisorio && name && !nonDeterminato
    ? t("transcript.provisionalSpeaker", { name })
    : name;
}

/**
 * Un Parlante del testo, con l'etichetta dei suoi turni e il nome da cui parte la rinomina. Senza
 * numero (`parlante` `null`) è un Ingresso non diarizzato, una persona sola: il Microfono.
 */
export interface Parlante {
  ingresso: Ingresso;
  label: string;
  nome: string;
  parlante: number | null;
  provvisorio?: boolean;
}

/**
 * I Parlanti delle Frasi, in ordine di comparsa, e gli Ingressi con Frasi senza Parlante (si
 * rinominano come una persona sola). Il mix senza Parlanti non ne ha.
 */
export function parlantiOf(
  conversation: Conversation,
  t: TFunction
): Parlante[] {
  const found: Parlante[] = [];
  for (const phrase of conversation.phrases) {
    if (phrase.parlanteNonDeterminato) {
      continue;
    }
    const { ingresso } = phrase;
    const parlante = phrase.parlante ?? null;
    if (
      (parlante !== null || ingresso !== "mix") &&
      !found.some((s) => s.ingresso === ingresso && s.parlante === parlante)
    ) {
      const provvisorio = conversation.phrases.some(
        (p) =>
          p.ingresso === ingresso &&
          p.parlante === parlante &&
          p.parlanteProvvisorio
      );
      found.push({
        ingresso,
        label:
          voiceLabel(conversation, ingresso, parlante, t, false, provvisorio) ??
          "",
        nome:
          nomeOf(conversation, ingresso, parlante, t) ??
          (ingresso === "mix" ? "" : t(INGRESSO_LABELS[ingresso])),
        parlante,
        ...(provvisorio ? { provvisorio: true } : {}),
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
  items: (TranscriptPhrase | ConversationPartial)[];
  /** Ingresso e id del primo elemento: resta lo stesso mentre il turno cresce. */
  key: string;
  /** `Microfono · Parlante 2`, `Microfono`, `Parlante 2`; `null` senza etichetta (il mix). */
  label: string | null;
  /** Il nome del Parlante, senza il prefisso dell'Ingresso. */
  name: string | null;
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

/** Da questo silenzio stimato in su, Copione e Nastro mostrano il separatore della pausa. */
const SHOWN_PAUSE_MS = 5000;

/**
 * Il silenzio stimato prima di ogni turno, se merita il separatore (almeno 5 s), altrimenti `null`.
 * Si misura dalla fine più tarda delle Frasi precedenti: con gli Ingressi separati una voce può
 * parlare sopra il turno di prima.
 */
export function pausesOf(turns: Turn[]): (number | null)[] {
  let fineMs: number | null = null;
  return turns.map(({ items }) => {
    const [first] = items;
    const silence =
      fineMs === null || !first ? 0 : silenceMs({ fineMs }, first);
    for (const item of items) {
      fineMs = Math.max(fineMs ?? 0, item.fineMs);
    }
    return silence >= SHOWN_PAUSE_MS ? silence : null;
  });
}

/**
 * La conversazione a turni, in ordine di inizio con i Parziali al loro posto: un turno nuovo a ogni
 * cambio di etichetta (Ingresso o Parlante) o, senza etichette, dopo una pausa: come nel Markdown.
 */
export function turnsOf(conversation: Conversation, t: TFunction): Turn[] {
  // Senza etichette i turni sono i paragrafi del Markdown, separati dalle pause.
  const labeled = [...conversation.phrases, ...conversation.partials].some(
    (p) =>
      voiceLabel(
        conversation,
        p.ingresso,
        p.parlante,
        t,
        p.parlanteNonDeterminato
      ) !== null
  );
  const turns: Turn[] = [];
  let last: Turn | undefined;
  for (const item of conversation.partials.reduce<
    (TranscriptPhrase | ConversationPartial)[]
  >(byStart, conversation.phrases)) {
    const partial = conversation.partials.includes(item as TranscriptPartial);
    const parlante = item.parlanteNonDeterminato
      ? null
      : (item.parlante ?? null);
    const label = voiceLabel(
      conversation,
      item.ingresso,
      parlante,
      t,
      item.parlanteNonDeterminato,
      item.parlanteProvvisorio
    );
    const previous = last?.items[last.items.length - 1];
    if (
      last &&
      previous &&
      last.label === label &&
      last.ingresso === item.ingresso &&
      last.parlante === parlante &&
      Boolean(previous.parlanteNonDeterminato) ===
        Boolean(item.parlanteNonDeterminato) &&
      Boolean(previous.parlanteProvvisorio) ===
        Boolean(item.parlanteProvvisorio) &&
      (labeled || silenceMs(previous, item) <= PARAGRAPH_PAUSE_MS)
    ) {
      last.items.push(item);
    } else {
      last = {
        ingresso: item.ingresso,
        items: [item],
        key: `${phraseKey(item)}${partial ? ":parziale" : ""}`,
        label,
        name: voiceName(
          conversation,
          item.ingresso,
          parlante,
          t,
          item.parlanteNonDeterminato,
          item.parlanteProvvisorio
        ),
        parlante,
      };
      turns.push(last);
    }
  }
  return turns;
}

/** Il testo di Copia turno, sempre semplice: solo le Frasi, senza Ingresso né nome della voce. */
export function turnText(turn: Turn): string {
  return turnBody(
    turn,
    turn.items.some((item) => "testoCorretto" in item && item.testoCorretto)
      ? "\n"
      : " "
  );
}

/** I separatori sono testo modificabile; le altre Frasi della stessa correzione non si duplicano. */
export function turnBody(turn: Turn, separator = "\n"): string {
  return textItemsOf(turn)
    .map((item) => item.text)
    .filter((text) => text.length > 0)
    .join(separator);
}

/** Le unità di testo salvate; ogni correzione del Turno ha un solo portatore. */
export function textItemsOf(turn: Turn): Turn["items"] {
  return turn.items.filter(
    (item) =>
      !("testoTurno" in item) ||
      item.testoTurno === null ||
      item.testoTurno === undefined ||
      item.testoTurno === item.phraseId
  );
}

/** Il Turno adiacente è una destinazione soltanto se ha una voce nota dello stesso Ingresso. */
export function mergeDestination(
  source: Turn,
  target: Turn | undefined,
  partials: Conversation["partials"]
): PhraseRef | null {
  if (
    !target ||
    source.ingresso !== target.ingresso ||
    !target.label ||
    [...source.items, ...target.items].some((item) =>
      partials.includes(item as ConversationPartial)
    ) ||
    target.items.some(
      (item) => item.parlanteNonDeterminato || item.parlanteProvvisorio
    )
  ) {
    return null;
  }
  return target.items[0] ?? null;
}

/** Quanti colori hanno le voci: oltre, si ricomincia dal primo. */
export const VOICE_COLORS = 6;

/**
 * Il colore (da 0) di ogni etichetta dei turni, per ordine di comparsa: una voce rinominata resta
 * nello stesso posto, quindi tiene il suo colore. Il Parlante non determinato non è una voce: niente
 * colore.
 */
export function voiceColors(turns: Turn[]): Map<string, number> {
  const colors = new Map<string, number>();
  for (const { label, items } of turns) {
    if (
      label !== null &&
      !items[0]?.parlanteNonDeterminato &&
      !colors.has(label)
    ) {
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
