/** I file non hanno sessione; ogni Registrazione ha il proprio UUID. */
export function inEventSession(
  event: { sessionId?: string | null },
  sessionId?: string | null
): boolean {
  return (event.sessionId ?? null) === (sessionId ?? null);
}
