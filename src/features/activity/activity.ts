import type {
  DiarizationStarted,
  Ingresso,
  LiveTranscriptionFailed,
  LiveTranscriptUpdated,
  RecordingCleaningFailed,
  RecordingCleaningPreparing,
  RecordingPhaseChanged,
  RecordingTick,
  SpeakersAssigned,
  TapeInfo,
  TranscriptionProgress,
  TranscriptPartial,
  TranscriptPhrase,
} from "@/bindings";
import type { TapeChange } from "@/features/source/view";
import {
  type Status,
  withDiarizing,
  withLiveError,
  withMovedSource,
  withProgress,
  withRecordingPhase,
} from "@/features/status/status";
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
      finals: Partial<Record<Ingresso, true>>;
    };

export interface ActivityState {
  /** I guasti della pulizia della Registrazione, uno per Ingresso, e se l'avviso è stato chiuso. */
  cleaningDismissed: boolean;
  cleaningFailures: RecordingCleaningFailed[];
  cleaningPreparing: RecordingCleaningPreparing[];
  conversation: Conversation;
  /** Il timer della Registrazione. */
  elapsedMs: number;
  /** Le informazioni del Tape Sorgente; nessuna per un file o durante una Registrazione. */
  info: TapeInfo | null;
  /** Lo stato prima dell'avvio, per il ripristino. */
  previous: Omit<ActivityState, "previous"> | null;
  session: Session;
  status: Status;
}

export const INITIAL_ACTIVITY: ActivityState = {
  cleaningDismissed: false,
  cleaningFailures: [],
  cleaningPreparing: [],
  conversation: EMPTY_CONVERSATION,
  elapsedMs: 0,
  info: null,
  previous: null,
  session: { state: "none" },
  status: { phase: "idle", source: null },
};

/** I comandi della finestra; gli eventi del backend sono `EventAction`. */
type CommandAction =
  | {
      type: "start";
      kind: "recording";
      sessionId: string;
      partials: boolean;
      nomeMicrofono: string | null;
    }
  | { type: "start"; kind: "transcription" | "diarization" }
  /** Salvate le impostazioni, la richiesta di Registrazione è partita. */
  | { type: "recordingRequested" }
  /** Lo Status dell'esito, già calcolato dalle funzioni di esito. */
  | { type: "outcome"; status: Status }
  | { type: "restore" }
  | { type: "pause"; paused: boolean }
  /** Uno Status fuori dalle Attività: la Sorgente aperta, o l'errore di un comando. */
  | { type: "status"; status: Status }
  | { type: "moved"; from: string; to: string }
  | { type: "cleaningOff"; ingresso: Ingresso }
  | { type: "cleaningDismissed" }
  | {
      type: "sourceLoaded";
      /** Frasi, nomi e informazioni del Tape aperto; senza, un file. */
      tape?: Pick<Conversation, "parlanti" | "phrases"> & { info: TapeInfo };
    }
  | ({ type: "sourceChanged" } & TapeChange);

export type EventAction =
  | { type: "phrase"; payload: TranscriptPhrase }
  | { type: "partial"; payload: TranscriptPartial }
  | { type: "liveTranscript"; payload: LiveTranscriptUpdated }
  | { type: "speakers"; payload: SpeakersAssigned }
  | { type: "liveFailed"; payload: LiveTranscriptionFailed }
  | { type: "recordingPhase"; payload: RecordingPhaseChanged }
  | { type: "progress"; payload: TranscriptionProgress }
  | { type: "diarizationStarted"; payload: DiarizationStarted }
  | { type: "tick"; payload: RecordingTick }
  | { type: "cleaningPreparing"; payload: RecordingCleaningPreparing }
  | { type: "cleaningFailed"; payload: RecordingCleaningFailed };

export type ActivityAction = CommandAction | EventAction;

export function activity(
  state: ActivityState,
  action: ActivityAction
): ActivityState {
  switch (action.type) {
    case "start":
      return started(state, action);
    case "recordingRequested":
      return {
        ...state,
        status: withRecordingPhase(state.status, "preparing"),
      };
    // Una Trascrizione finisce qui; una Registrazione aspetta ancora il suo snapshot finale.
    case "outcome":
      if (state.session.state !== "open") {
        return { ...state, status: action.status };
      }
      return {
        ...state,
        conversation: withoutPartials(state.conversation),
        previous: null,
        session:
          state.session.kind === "recording"
            ? { ...state.session, state: "closing" }
            : { state: "closed" },
        status: action.status,
      };
    // La Registrazione annullata in Preparazione, o non partita: torna lo stato di prima.
    case "restore":
      return state.previous ? { ...state.previous, previous: null } : state;
    case "sourceLoaded":
      return {
        ...state,
        conversation: {
          ...EMPTY_CONVERSATION,
          parlanti: action.tape?.parlanti ?? {},
          phrases: action.tape?.phrases ?? [],
        },
        info: action.tape?.info ?? null,
        previous: null,
        session:
          state.session.state === "none" ? state.session : { state: "closed" },
      };
    // Correzioni, nomi e informazioni valgono solo quando nessuna Attività scrive il testo.
    case "sourceChanged":
      return writing(state.session) ? state : withSourceChange(state, action);
    case "pause":
      return state.status.phase === "recording"
        ? { ...state, status: { ...state.status, paused: action.paused } }
        : state;
    case "status":
      return { ...state, status: action.status };
    case "moved":
      return {
        ...state,
        status: withMovedSource(state.status, action.from, action.to),
      };
    case "cleaningOff":
      return {
        ...state,
        cleaningPreparing: state.cleaningPreparing.filter(
          (item) => item.ingresso !== action.ingresso
        ),
      };
    case "cleaningDismissed":
      return { ...state, cleaningDismissed: true };
    default:
      return withEvent(state, action);
  }
}

type StartAction = Extract<CommandAction, { type: "start" }>;

function started(state: ActivityState, action: StartAction): ActivityState {
  const { previous: _, ...before } = state;
  const recording = action.kind === "recording";
  return {
    ...state,
    conversation: startText(state.conversation, action),
    previous: before,
    session: {
      finals: {},
      id: recording ? action.sessionId : null,
      kind: action.kind,
      partials: recording && action.partials,
      state: "open",
    },
    status: STARTED[action.kind],
    // Ogni Registrazione parte da zero, senza preparazioni né guasti della pulizia.
    ...(recording ? RECORDING_START : {}),
  };
}

const STARTED: Record<ActivityKind, Status> = {
  diarization: { phase: "diarizing" },
  recording: { phase: "preparingRecording", stage: "saving" },
  transcription: { percent: null, phase: "transcribing" },
};

const RECORDING_START = {
  cleaningDismissed: false,
  cleaningFailures: [],
  cleaningPreparing: [],
  elapsedMs: 0,
} satisfies Partial<ActivityState>;

function withEvent(state: ActivityState, action: EventAction): ActivityState {
  const { session } = state;
  if (action.type === "liveTranscript") {
    return withFinal(state, action.payload);
  }
  if (!inSession(session, action.payload.sessionId)) {
    return state;
  }
  switch (action.type) {
    case "phrase":
      return session.finals[action.payload.ingresso]
        ? state
        : text(state, (c) => withPhrase(c, action.payload));
    case "partial":
      return session.partials && !session.finals[action.payload.ingresso]
        ? text(state, (c) => withPartial(c, action.payload))
        : state;
    case "speakers":
      return text(state, (c) => withParlanti(c, action.payload.speakers));
    // La Registrazione continua: via i Parziali, la fase dice il guasto.
    case "liveFailed":
      return {
        ...text(state, withoutPartials),
        status: withLiveError(state.status, action.payload.error),
      };
    // Partita la Registrazione, la sua vista prende il posto della Sorgente di prima.
    case "recordingPhase": {
      const status = withRecordingPhase(state.status, action.payload.phase);
      return {
        ...state,
        info: status.phase === "recording" ? null : state.info,
        status,
      };
    }
    case "progress":
      return {
        ...state,
        status: withProgress(state.status, action.payload.percent),
      };
    case "diarizationStarted":
      return { ...state, status: withDiarizing(state.status) };
    case "tick":
      return { ...state, elapsedMs: action.payload.elapsedMs };
    case "cleaningPreparing":
      return {
        ...state,
        cleaningPreparing: [
          ...state.cleaningPreparing.filter(
            (item) => item.ingresso !== action.payload.ingresso
          ),
          action.payload,
        ],
      };
    // L'avviso torna a ogni guasto e resta fino alla chiusura; uno per Ingresso.
    case "cleaningFailed":
      return {
        ...state,
        cleaningDismissed: false,
        cleaningFailures: [
          ...state.cleaningFailures.filter(
            (item) => item.ingresso !== action.payload.ingresso
          ),
          action.payload,
        ],
      };
    default:
      return action satisfies never;
  }
}

/** In chiusura entra ancora: nel fallback Ogg arriva dopo la risposta a Stop. */
function withFinal(
  state: ActivityState,
  payload: LiveTranscriptUpdated
): ActivityState {
  const { session } = state;
  const { ingresso, sessionId } = payload;
  if (
    !writing(session) ||
    session.id !== sessionId ||
    session.finals[ingresso]
  ) {
    return state;
  }
  return {
    ...text(state, (c) => withLiveTranscript(c, payload)),
    session: { ...session, finals: { ...session.finals, [ingresso]: true } },
  };
}

function withSourceChange(
  state: ActivityState,
  { change, info }: TapeChange
): ActivityState {
  return {
    ...state,
    conversation: change ? change(state.conversation) : state.conversation,
    info: info ? info(state.info) : state.info,
  };
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

function text(
  state: ActivityState,
  change: (c: Conversation) => Conversation
): ActivityState {
  return { ...state, conversation: change(state.conversation) };
}

/** Trascrivi riparte da zero; Riconosci i parlanti tiene il testo; la Registrazione ha il Microfono. */
function startText(
  conversation: Conversation,
  action: StartAction
): Conversation {
  if (action.kind === "diarization") {
    return conversation;
  }
  return action.kind === "recording" && action.nomeMicrofono
    ? withNome(EMPTY_CONVERSATION, "microfono", null, action.nomeMicrofono)
    : EMPTY_CONVERSATION;
}
