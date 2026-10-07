import type { RecordingCleaningFailed } from "@/bindings";
import { inEventSession } from "@/lib/event-session";

/** Gli avvisi recuperati restano nella sessione fino a chiusura esplicita. */
export function withCleaningFailure(
  current: RecordingCleaningFailed[],
  event: RecordingCleaningFailed,
  sessionId: string | null
): RecordingCleaningFailed[] {
  if (!(sessionId && inEventSession(event, sessionId))) {
    return current;
  }
  return [...current.filter((item) => item.ingresso !== event.ingresso), event];
}
