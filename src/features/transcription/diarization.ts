import type { TapeInfo, TranscriptPhrase } from "@/bindings";
import type { Conversation } from "@/features/transcription/phrases";

/** Anche i Tape precedenti ai metadati della Diarizzazione possono avere Parlanti. */
export function wasDiarized(
  info: TapeInfo | null,
  phrases: TranscriptPhrase[]
): boolean {
  return (
    !!info?.diarizzazione ||
    phrases.some(
      (p) =>
        p.parlante !== null || p.parlanteNonDeterminato || p.parlanteProvvisorio
    )
  );
}

/** Una nuova analisi avvisa delle correzioni salvate, compresi i nomi dei Tape precedenti. */
export function diarizationNeedsConfirmation(
  info: Pick<TapeInfo, "correttoAMano"> | null,
  parlanti: Conversation["parlanti"]
): boolean {
  return !!info?.correttoAMano || Object.keys(parlanti).length > 0;
}
