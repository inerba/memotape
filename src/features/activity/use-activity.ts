import { useCallback, useEffect, useReducer, useRef } from "react";
import { events } from "@/bindings";
import { activity, INITIAL_ACTIVITY } from "@/features/activity/activity";
import type { Status } from "@/features/status/status";

/**
 * L'Attività in corso: inoltra i suoi eventi al modulo dell'Attività. `ready` si risolve quando
 * tutti i listener sono registrati; la Registrazione lo aspetta prima di partire.
 */
export function useActivity(status: Status) {
  const [state, dispatch] = useReducer(activity, status, (initial) => ({
    ...INITIAL_ACTIVITY,
    status: initial,
  }));
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
      events.liveTranscriptionFailed.listen(({ payload }) =>
        dispatch({ payload, type: "liveFailed" })
      ),
      events.recordingPhaseChanged.listen(({ payload }) =>
        dispatch({ payload, type: "recordingPhase" })
      ),
      events.transcriptionProgress.listen(({ payload }) =>
        dispatch({ payload, type: "progress" })
      ),
      events.diarizationStarted.listen(({ payload }) =>
        dispatch({ payload, type: "diarizationStarted" })
      ),
      events.recordingTick.listen(({ payload }) =>
        dispatch({ payload, type: "tick" })
      ),
      events.recordingCleaningPreparing.listen(({ payload }) =>
        dispatch({ payload, type: "cleaningPreparing" })
      ),
      events.recordingCleaningFailed.listen(({ payload }) =>
        dispatch({ payload, type: "cleaningFailed" })
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
