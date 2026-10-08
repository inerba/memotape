import { useCallback, useEffect, useReducer, useRef } from "react";
import { events } from "@/bindings";
import { activity, INITIAL_ACTIVITY } from "@/features/activity/activity";

/**
 * Il testo dell'Attività in corso: inoltra gli eventi al modulo dell'Attività. `ready` si risolve
 * quando i listener sono registrati; la Registrazione lo aspetta prima di partire.
 */
export function useActivity() {
  const [state, dispatch] = useReducer(activity, INITIAL_ACTIVITY);
  const listeners = useRef<Promise<unknown>>(Promise.resolve());
  useEffect(() => {
    const all = [
      events.transcriptPhrase.listen(({ payload }) =>
        dispatch({ payload, type: "phrase" })
      ),
      events.transcriptPartial.listen(({ payload }) =>
        dispatch({ payload, type: "partial" })
      ),
      events.liveTranscriptUpdated.listen(({ payload }) =>
        dispatch({ payload, type: "liveTranscript" })
      ),
      events.speakersAssigned.listen(({ payload }) =>
        dispatch({ payload, type: "speakers" })
      ),
    ];
    listeners.current = Promise.all(all);
    return () => {
      for (const listener of all) {
        listener.then((stop) => stop());
      }
    };
  }, []);
  const ready = useCallback(() => listeners.current, []);
  return { ...state, dispatch, ready };
}
