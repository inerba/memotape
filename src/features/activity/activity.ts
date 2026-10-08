import type {
  Ingresso,
  LiveTranscriptUpdated,
  SpeakersAssigned,
  TranscriptPartial,
  TranscriptPhrase,
} from "@/bindings";
import {
  type Conversation,
  EMPTY_CONVERSATION,
  withLiveTranscript,
  withNome,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
} from "@/features/transcription/phrases";

export type ActivityKind = "recording" | "transcription" | "diarization";

/**
 * La sessione dell'Attività che scrive il testo (ADR-0028). Aperta accetta gli eventi con il suo
 * identificatore (nessuno per Trascrivi e Riconosci i parlanti); in chiusura, dopo l'esito di una
 * Registrazione, solo il suo snapshot finale; chiusa, o nessuna, scarta tutto.
 */
export type Session =
  | { state: "none" | "closed" }
  | {
      state: "open" | "closing";
      kind: ActivityKind;
      id: string | null;
      /** Trascrivi dal vivo all'avvio della Registrazione. */
      partials: boolean;
      /** Gli Ingressi che hanno già ricevuto lo snapshot finale. */
      finali: Partial<Record<Ingresso, true>>;
    };

export interface ActivityState {
  conversation: Conversation;
  /** Sessione e testo prima dell'avvio, per il ripristino. */
  previous: { conversation: Conversation; session: Session } | null;
  session: Session;
}

export const INITIAL_ACTIVITY: ActivityState = {
  conversation: EMPTY_CONVERSATION,
  previous: null,
  session: { state: "none" },
};

export type ActivityAction =
  | {
      type: "start";
      kind: ActivityKind;
      sessionId: string | null;
      partials: boolean;
      nomeMicrofono: string | null;
    }
  | { type: "outcome" }
  | { type: "restore" }
  | {
      type: "sourceOpened";
      /** Frasi e nomi del Tape aperto; senza, un file. */
      tape?: Pick<Conversation, "parlanti" | "phrases">;
    }
  | { type: "sourceChanged"; change: (c: Conversation) => Conversation }
  | { type: "phrase"; payload: TranscriptPhrase }
  | { type: "partial"; payload: TranscriptPartial }
  | { type: "liveTranscript"; payload: LiveTranscriptUpdated }
  | { type: "speakers"; payload: SpeakersAssigned }
  | { type: "liveFailed"; sessionId: string };

export function activity(
  state: ActivityState,
  action: ActivityAction
): ActivityState {
  switch (action.type) {
    case "start":
      return {
        conversation: startText(state.conversation, action),
        previous: { conversation: state.conversation, session: state.session },
        session: {
          finali: {},
          id: action.sessionId,
          kind: action.kind,
          partials: action.partials,
          state: "open",
        },
      };
    // Una Trascrizione finisce qui; una Registrazione aspetta ancora il suo snapshot finale.
    case "outcome":
      if (state.session.state !== "open") {
        return state;
      }
      return {
        conversation: withoutPartials(state.conversation),
        previous: null,
        session:
          state.session.kind === "recording"
            ? { ...state.session, state: "closing" }
            : { state: "closed" },
      };
    // La Registrazione annullata in Preparazione, o non partita: tornano testo e sessione di prima.
    case "restore":
      return state.previous ? { ...state.previous, previous: null } : state;
    case "sourceOpened":
      return {
        conversation: { ...EMPTY_CONVERSATION, ...action.tape },
        previous: null,
        session:
          state.session.state === "none" ? state.session : { state: "closed" },
      };
    // Correzioni e nomi valgono solo quando nessuna Attività scrive il testo.
    case "sourceChanged":
      return writing(state.session) ? state : text(state, action.change);
    default:
      return withEvent(state, action);
  }
}

type EventAction = Exclude<
  ActivityAction,
  { type: "start" | "outcome" | "restore" | "sourceOpened" | "sourceChanged" }
>;

function withEvent(state: ActivityState, action: EventAction): ActivityState {
  const { session } = state;
  switch (action.type) {
    case "phrase":
      return accepts(session, action.payload)
        ? text(state, (c) => withPhrase(c, action.payload))
        : state;
    case "partial":
      return accepts(session, action.payload) && session.partials
        ? text(state, (c) => withPartial(c, action.payload))
        : state;
    case "speakers":
      return inSession(session, action.payload.sessionId)
        ? text(state, (c) => withParlanti(c, action.payload.speakers))
        : state;
    case "liveFailed":
      return inSession(session, action.sessionId)
        ? text(state, withoutPartials)
        : state;
    // In chiusura entra ancora: nel fallback Ogg arriva dopo la risposta a Stop.
    case "liveTranscript": {
      const { ingresso, sessionId } = action.payload;
      if (
        !writing(session) ||
        session.id !== sessionId ||
        session.finali[ingresso]
      ) {
        return state;
      }
      return {
        ...text(state, (c) => withLiveTranscript(c, action.payload)),
        session: {
          ...session,
          finali: { ...session.finali, [ingresso]: true },
        },
      };
    }
    default:
      return state;
  }
}

type Live = Extract<Session, { state: "open" | "closing" }>;

/** Un'Attività sta scrivendo il testo, o una Registrazione aspetta il suo snapshot finale. */
function writing(session: Session): session is Live {
  return session.state === "open" || session.state === "closing";
}

function inSession(
  session: Session,
  sessionId: string | null | undefined
): session is Live {
  return session.state === "open" && (sessionId ?? null) === session.id;
}

/** Un evento della sessione aperta, per un Ingresso senza snapshot finale. */
function accepts(
  session: Session,
  event: { sessionId?: string | null; ingresso: Ingresso }
): session is Live {
  return inSession(session, event.sessionId) && !session.finali[event.ingresso];
}

function text(
  state: ActivityState,
  change: (c: Conversation) => Conversation
): ActivityState {
  return { ...state, conversation: change(state.conversation) };
}

/** Trascrivi riparte da zero; Riconosci i parlanti tiene il testo; la Registrazione ha il Microfono. */
function startText(
  conversation: Conversation,
  { kind, nomeMicrofono }: { kind: ActivityKind; nomeMicrofono: string | null }
): Conversation {
  if (kind === "diarization") {
    return conversation;
  }
  return kind === "recording" && nomeMicrofono
    ? withNome(EMPTY_CONVERSATION, "microfono", null, nomeMicrofono)
    : EMPTY_CONVERSATION;
}
