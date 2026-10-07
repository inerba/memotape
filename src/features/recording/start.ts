import type { AppError, commands } from "@/bindings";

type RecordingResult = Awaited<ReturnType<typeof commands.record>>;

/** Annulla torna subito alla vista precedente; i salvataggi già richiesti proseguono. */
export async function recordAfterSettings(
  flush: () => Promise<AppError | null>,
  record: () => Promise<RecordingResult>,
  signal: AbortSignal,
  listeners: Promise<unknown> = Promise.resolve()
): Promise<RecordingResult> {
  const cancelled = { code: "cancelled" } as const;
  if (signal.aborted) {
    return { error: cancelled, status: "error" };
  }
  let abort: () => void = () => undefined;
  const cancellation = new Promise<AppError>((resolve) => {
    abort = () => resolve(cancelled);
    signal.addEventListener("abort", abort, { once: true });
  });
  try {
    const preparation = (async () => {
      const error = await flush();
      if (!error) {
        await listeners;
      }
      return error;
    })();
    const error = await Promise.race([preparation, cancellation]);
    if (signal.aborted) {
      return { error: cancelled, status: "error" };
    }
    return error ? { error, status: "error" } : record();
  } finally {
    signal.removeEventListener("abort", abort);
  }
}
