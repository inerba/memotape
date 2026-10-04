import type { TranscriptPhrase } from "@/bindings";

/**
 * Le Frasi da evidenziare alla posizione `ms` del player, tra `phrases` in ordine di inizio: quelle
 * che la contengono (`fineMs` esclusa), più di una se si sovrappongono (Ingressi separati); nel
 * silenzio, la Frase finita per ultima; prima della prima Frase nessuna. La prima è quella che lo
 * scorrimento segue.
 */
export function playingAt(
  phrases: TranscriptPhrase[],
  ms: number
): TranscriptPhrase[] {
  const playing = phrases.filter((p) => p.inizioMs <= ms && ms < p.fineMs);
  if (playing.length > 0) {
    return playing;
  }
  let previous: TranscriptPhrase | undefined;
  for (const p of phrases) {
    if (p.fineMs <= ms && (!previous || p.fineMs > previous.fineMs)) {
      previous = p;
    }
  }
  return previous ? [previous] : [];
}

/** Se il testo segue l'audio (`following`) o l'utente l'ha scorso da solo (`free`). */
export type Follow = "following" | "free";

/**
 * `scroll`: lo scorrimento a mano; `seek`: la barra del player; `jump`: il pulsante del tempo di una
 * Frase o un risultato della ricerca; `follow`: il comando "Segui l'audio".
 */
export type FollowEvent = "scroll" | "seek" | "jump" | "follow";

export function nextFollow(event: FollowEvent): Follow {
  return event === "scroll" ? "free" : "following";
}

/** Se il testo scorre da solo fino alla Frase in riproduzione: non mentre se ne corregge una. */
export function autoScroll(follow: Follow, editing: boolean): boolean {
  return follow === "following" && !editing;
}
