import { useReducer } from "react";
import type { OpenedTape } from "@/bindings";
import {
  type ActivityAction,
  type ActivityState,
  activity,
  INITIAL_ACTIVITY,
} from "@/features/activity/activity";
import { useActivityEvents } from "@/features/activity/use-activity";
import { isTape } from "@/features/source/file-name";
import {
  INITIAL_VIEW,
  type SourceView,
  type SourceViewAction,
  sourceView,
  type TapeView,
} from "@/features/source/view";
import { isBusy, type Status } from "@/features/status/status";

/** La finestra principale: l'Attività e la vista della Sorgente in un solo stato. */
export interface HomeState {
  activity: ActivityState;
  view: SourceView;
}

export const INITIAL_HOME: HomeState = {
  activity: INITIAL_ACTIVITY,
  view: INITIAL_VIEW,
};

/** Apertura, modifica e spostamento di un Tape passano dalla vista, che li applica anche qui. */
type OwnedByView = "sourceLoaded" | "sourceChanged" | "moved";

export type HomeAction =
  | Exclude<ActivityAction, { type: OwnedByView }>
  | SourceViewAction;

/** Dopo ogni azione, le reazioni della vista alle fasi dell'Attività. */
export function home(state: HomeState, action: HomeAction): HomeState {
  return reacted(state, step(state, action));
}

function step(state: HomeState, action: HomeAction): HomeState {
  const view = (a: SourceViewAction) => sourceView(state.view, a);
  switch (action.type) {
    case "sourceOpened":
      return { activity: opened(state.activity, action), view: view(action) };
    case "tapeConsulted":
    case "openRequested":
    case "activityShown":
    case "libraryShown":
    case "homeShown":
    case "tapeRequested":
      return { ...state, view: view(action) };
    // Correzioni, nomi e informazioni valgono per ogni vista di quel Tape.
    case "tapeChanged":
      return {
        activity:
          state.view.source === action.path
            ? activity(state.activity, { ...action, type: "sourceChanged" })
            : state.activity,
        view: view(action),
      };
    case "tapeMoved":
      return {
        activity: activity(state.activity, { ...action, type: "moved" }),
        view: view(action),
      };
    // La Sorgente nel Cestino è come nessuna Sorgente aperta.
    case "tapeTrashed":
      return {
        activity:
          state.view.source === action.path
            ? opened(state.activity, { path: null })
            : state.activity,
        view: view(action),
      };
    default:
      return { ...state, activity: activity(state.activity, action) };
  }
}

/**
 * All'avvio della Registrazione la sua vista prende il posto di Home, Libreria e Tape consultato;
 * Trascrivi toglie la Frase evidenziata; a fine Attività si chiude il Tape consultato.
 */
function reacted(before: HomeState, after: HomeState): HomeState {
  const was = before.activity.status;
  const is = after.activity.status;
  let { view } = after;
  if (was.phase !== "recording" && is.phase === "recording") {
    view = {
      ...view,
      consulted: null,
      highlight: null,
      homeOpen: false,
      libraryOpen: false,
    };
  }
  if (was.phase !== "transcribing" && is.phase === "transcribing") {
    view = { ...view, highlight: null };
  }
  if (isBusy(was) && !isBusy(is)) {
    view = { ...view, consulted: null };
  }
  return view === after.view ? after : { ...after, view };
}

/** La vista al centro della finestra; `isSource`: il Tape è la Sorgente, che Trascrivi trascrive. */
export type CenterView =
  | { kind: "library" | "home" | "live" }
  | {
      kind: "preparation";
      stage: Extract<Status, { phase: "preparingRecording" }>["stage"];
    }
  | { kind: "tape"; tape: TapeView; isSource: boolean }
  | { kind: "file"; path: string };

export function centerView(state: HomeState): CenterView {
  const { conversation, info, status } = state.activity;
  const { view } = state;
  if (status.phase === "preparingRecording") {
    return { kind: "preparation", stage: status.stage };
  }
  if (view.libraryOpen) {
    return { kind: "library" };
  }
  if (view.homeOpen) {
    return { kind: "home" };
  }
  if (view.consulted) {
    return { isSource: false, kind: "tape", tape: view.consulted };
  }
  if (isLive(status)) {
    return { kind: "live" };
  }
  if (view.source && isTape(view.source)) {
    const tape = { conversation, info, path: view.source };
    return { isSource: true, kind: "tape", tape };
  }
  return view.source ? { kind: "file", path: view.source } : { kind: "home" };
}

/**
 * Cosa evidenzia la barra laterale: quello che mostra `centerView`, Home, la Libreria o il Tape (o
 * il file); niente durante la preparazione e la Registrazione.
 */
export function selection(state: HomeState): {
  home: boolean;
  library: boolean;
  tape: string | null;
} {
  const center = centerView(state);
  let tape: string | null = null;
  if (center.kind === "tape") {
    tape = center.tape.path;
  } else if (center.kind === "file") {
    tape = center.path;
  }
  return {
    home: center.kind === "home",
    library: center.kind === "library",
    tape,
  };
}

/** La Registrazione, anche mentre completa la Trascrizione dal vivo. */
function isLive({ phase }: Status) {
  return phase === "recording" || phase === "completing";
}

/**
 * La Sorgente aperta senza un'Attività in corso: testo, informazioni e Status "pronto". Durante
 * un'Attività solo il Tape del suo esito ne prende il testo; lo Status lo dà l'esito.
 */
function opened(
  state: ActivityState,
  { path, tape }: { path: string | null; tape?: OpenedTape }
): ActivityState {
  if (state.session.state === "open") {
    return tape ? activity(state, { tape, type: "sourceLoaded" }) : state;
  }
  return activity(activity(state, { tape, type: "sourceLoaded" }), {
    status: { phase: "idle", source: path },
    type: "status",
  });
}

/**
 * Lo stato della finestra principale e i listener dell'Attività. `ready` e `onRecordingStarted`
 * come in `useActivityEvents`.
 */
export function useHomeState(
  initialStatus: Status,
  onRecordingStarted: (sessionId: string) => void
) {
  const [state, dispatch] = useReducer(home, initialStatus, (status) => ({
    ...INITIAL_HOME,
    activity: { ...INITIAL_ACTIVITY, status },
  }));
  const ready = useActivityEvents(dispatch, onRecordingStarted);
  return { dispatch, ready, state };
}
