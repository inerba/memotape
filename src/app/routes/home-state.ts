import { useReducer } from "react";
import type { OpenedTape } from "@/bindings";
import {
  type ActivityAction,
  type ActivityState,
  activity,
  INITIAL_ACTIVITY,
} from "@/features/activity/activity";
import { useActivityEvents } from "@/features/activity/use-activity";
import {
  INITIAL_VIEW,
  type SourceView,
  type SourceViewAction,
  sourceView,
} from "@/features/source/view";
import type { Status } from "@/features/status/status";

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
type OwnedByView = "sourceOpened" | "sourceChanged" | "moved";

export type HomeAction =
  | Exclude<ActivityAction, { type: OwnedByView }>
  | SourceViewAction;

export function home(state: HomeState, action: HomeAction): HomeState {
  switch (action.type) {
    case "sourceOpened":
      return {
        activity: opened(state.activity, action),
        view: sourceView(state.view, action),
      };
    case "browsingClosed":
    case "tapeBrowsed":
      return { ...state, view: sourceView(state.view, action) };
    // Correzioni, nomi e informazioni valgono per ogni vista di quel Tape.
    case "tapeChanged":
      return {
        activity:
          state.view.source === action.path
            ? activity(state.activity, { ...action, type: "sourceChanged" })
            : state.activity,
        view: sourceView(state.view, action),
      };
    case "tapeMoved":
      return {
        activity: activity(state.activity, { ...action, type: "moved" }),
        view: sourceView(state.view, action),
      };
    // La Sorgente nel Cestino è come nessuna Sorgente aperta.
    case "tapeTrashed":
      return {
        activity:
          state.view.source === action.path
            ? opened(state.activity, { path: null })
            : state.activity,
        view: sourceView(state.view, action),
      };
    default:
      return { ...state, activity: activity(state.activity, action) };
  }
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
    return tape ? activity(state, { tape, type: "sourceOpened" }) : state;
  }
  return activity(activity(state, { tape, type: "sourceOpened" }), {
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
  return { ...state.activity, ...state.view, dispatch, ready };
}
