import { useCallback, useEffect, useRef } from "react";
import { events } from "@/bindings";
import type { EventAction } from "@/features/activity/activity";

/**
 * L'Attività in corso: inoltra i suoi eventi a `dispatch`. Restituisce `ready`, che si risolve quando
 * tutti i listener sono registrati; la Registrazione lo aspetta prima di partire.
 * `onRecordingStarted` riceve la sessione partita nel listener stesso, prima del render: così un
 * errore di `record` subito dopo non la scambia per una Registrazione mai partita.
 */
export function useActivityEvents(
  dispatch: (action: EventAction) => void,
  onRecordingStarted: (sessionId: string) => void
) {
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
      events.recordingPhaseChanged.listen(({ payload }) => {
        dispatch({ payload, type: "recordingPhase" });
        if (payload.phase === "recording") {
          onRecordingStarted(payload.sessionId);
        }
      }),
      events.transcriptionProgress.listen(({ payload }) =>
        dispatch({ payload, type: "progress" })
      ),
      events.diarizationStarted.listen(({ payload }) =>
        dispatch({ payload, type: "diarizationStarted" })
      ),
      events.diarizationProgress.listen(({ payload }) =>
        dispatch({ payload, type: "diarizationProgress" })
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
  }, [dispatch, onRecordingStarted]);
  return useCallback(() => listeners.current, []);
}
