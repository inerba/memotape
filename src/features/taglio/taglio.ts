import type { TranscriptPhrase } from "@/bindings";
import type { Turn } from "@/features/transcription/phrases";

/** Un tratto da tagliare, in ms sulla linea del tempo del Tape. */
export interface Taglio {
  fineMs: number;
  inizioMs: number;
}

/** Da quale parte del Turno si taglia: tutto quello che viene prima o tutto quello che viene dopo. */
export type Dove = "prima" | "dopo";

/** Un Taglio chiesto da un Turno, con quello che serve alla conferma. */
export interface RichiestaTaglio {
  dove: Dove;
  taglio: Taglio;
  turn: Turn;
  /** I Turni che spariscono tutti. */
  turni: number;
}

/** Il silenzio che resta fra il bordo del Taglio e il parlato che si tiene. */
const MARGINE_MS = 1000;

/**
 * Il Taglio di tutto quello che viene prima (o dopo) di `turn`, che resta: dall'inizio del Tape a
 * 1 s prima del suo parlato, o da 1 s dopo fino alla fine. Il bordo non entra mai in una Frase che
 * resta: una voce di un altro Ingresso che si accavalla al Turno resta intera, e il bordo si sposta
 * prima (o dopo) di lei. `null` se non c'è niente da togliere.
 */
export function taglioDelTurno(
  phrases: TranscriptPhrase[],
  turn: Turn,
  dove: Dove,
  durataMs: number
): Taglio | null {
  if (dove === "prima") {
    // Si tiene il Turno e tutto ciò che finisce dopo il suo inizio, anche a catena.
    let tenuto = Math.min(...turn.items.map((i) => i.inizioMs));
    let cresciuto = true;
    while (cresciuto) {
      const prima = tenuto;
      tenuto = Math.min(
        tenuto,
        ...phrases.filter((p) => p.fineMs > tenuto).map((p) => p.inizioMs)
      );
      cresciuto = tenuto < prima;
    }
    const tolte = phrases
      .filter((p) => p.fineMs <= tenuto)
      .map((p) => p.fineMs);
    const fineMs = Math.max(0, ...tolte, tenuto - MARGINE_MS);
    return fineMs > 0 ? { fineMs, inizioMs: 0 } : null;
  }
  let tenuto = Math.max(...turn.items.map((i) => i.fineMs));
  let cresciuto = true;
  while (cresciuto) {
    const prima = tenuto;
    tenuto = Math.max(
      tenuto,
      ...phrases.filter((p) => p.inizioMs < tenuto).map((p) => p.fineMs)
    );
    cresciuto = tenuto > prima;
  }
  const tolte = phrases
    .filter((p) => p.inizioMs >= tenuto)
    .map((p) => p.inizioMs);
  const inizioMs = Math.min(durataMs, ...tolte, tenuto + MARGINE_MS);
  return inizioMs < durataMs ? { fineMs: durataMs, inizioMs } : null;
}

/** Se il tratto `[inizioMs, fineMs]` sta tutto dentro il Taglio: allora sparisce con lui. */
export function tagliato(
  taglio: Taglio,
  { fineMs, inizioMs }: { fineMs: number; inizioMs: number }
): boolean {
  return taglio.inizioMs <= inizioMs && fineMs <= taglio.fineMs;
}
