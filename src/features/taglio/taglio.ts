import type { TranscriptPhrase } from "@/bindings";
import {
  phraseKey,
  SHOWN_PAUSE_MS,
  type Turn,
} from "@/features/transcription/phrases";

/** Un silenzio lungo tra `a` (fine del parlato prima) e `b` (inizio di quello dopo), in modalità Taglio. */
export interface Silenzio {
  a: number;
  b: number;
  dove: "inizio" | "pausa" | "fine";
  /** Il silenzio stimato, come il separatore della pausa. */
  ms: number;
}

/**
 * I silenzi lunghi del testo in modalità Taglio: prima del primo turno, prima di ogni turno con una
 * pausa (`pauses`, da `pausesOf`) e dopo l'ultimo fino alla fine del Tape. Gli Assistenti usano la
 * stessa regola in Rust (`Document::silenzi` in `tape.rs`): si cambiano insieme.
 */
export function silenziDi(
  turns: Turn[],
  pauses: (number | null)[],
  durataMs: number
): {
  fine: Silenzio | null;
  inizio: Silenzio | null;
  prima: (Silenzio | null)[];
} {
  let fine = 0;
  const primaDeiTurni = turns.map((turn, index) => {
    const [first] = turn.items;
    const pausa = pauses[index];
    const silenzio: Silenzio | null =
      index > 0 && first && pausa
        ? { a: fine, b: first.inizioMs, dove: "pausa", ms: pausa }
        : null;
    for (const item of turn.items) {
      fine = Math.max(fine, item.fineMs);
    }
    return silenzio;
  });
  const primo = turns[0]?.items[0]?.inizioMs ?? 0;
  return {
    fine:
      turns.length > 0 && durataMs - fine >= SHOWN_PAUSE_MS
        ? { a: fine, b: durataMs, dove: "fine", ms: durataMs - fine }
        : null,
    inizio:
      primo >= SHOWN_PAUSE_MS
        ? { a: 0, b: primo, dove: "inizio", ms: primo }
        : null,
    prima: primaDeiTurni,
  };
}

/** Un tratto da tagliare, in ms sulla linea del tempo del Tape: un Taglio in sospeso. */
export interface Taglio {
  fineMs: number;
  inizioMs: number;
}

/** Il silenzio che resta da ogni lato di una giunzione. */
const MARGINE_MS = 1000;
/**
 * Oltre questo accavallamento due Frasi sono parlato insieme; sotto, sono i margini dei tempi
 * (prefill, hangover, tempi ASR delle parti divise al cambio di Parlante).
 */
const SOVRAPPOSIZIONE_MS = 500;

/** Il bordo d'inizio di un Taglio nel silenzio `[a, b]`: 1 s dopo il parlato, a metà se è corto. */
function dopo(a: number, b: number): number {
  return b - a > 2 * MARGINE_MS ? a + MARGINE_MS : (a + b) / 2;
}

/** Il bordo di fine di un Taglio nel silenzio `[a, b]`: 1 s prima del parlato, a metà se è corto. */
function prima(a: number, b: number): number {
  return b - a > 2 * MARGINE_MS ? b - MARGINE_MS : (a + b) / 2;
}

/**
 * Le chiavi delle Frasi scelte, con le altre Frasi della stessa correzione del Turno: il suo testo
 * sta solo nella prima, e scegliere quella vuol dire scegliere il Turno corretto.
 */
function conCorrezioni(
  phrases: TranscriptPhrase[],
  scelte: Set<string>
): Set<string> {
  const tutte = new Set(scelte);
  for (const p of phrases) {
    const portatore =
      p.testoTurno === null || p.testoTurno === undefined
        ? null
        : phraseKey({ ingresso: p.ingresso, phraseId: p.testoTurno });
    if (portatore && scelte.has(portatore)) {
      tutte.add(phraseKey(p));
    }
  }
  return tutte;
}

/**
 * Il Taglio delle Frasi `scelte` (chiavi di `phraseKey`): dalle Frasi intere si allarga ai silenzi
 * intorno, lasciando 1 s di pausa per lato; senza Frasi prima parte dall'inizio, senza Frasi dopo
 * arriva alla fine. `null` se non c'è nessuna Frase scelta.
 */
export function taglioDelleFrasi(
  phrases: TranscriptPhrase[],
  scelte: Set<string>,
  durataMs: number
): Taglio | null {
  const chiavi = conCorrezioni(phrases, scelte);
  if (!phrases.some((p) => chiavi.has(phraseKey(p)))) {
    return null;
  }
  // Il tratto si toglie a tutti gli Ingressi: chi parla sopra la scelta si taglia con lei (e si
  // vede barrato); sotto mezzo secondo è il margine dei tempi, non parlato insieme.
  let inizio = 0;
  let fine = 0;
  let cresciuta = true;
  while (cresciuta) {
    const scelteFrasi = phrases.filter((p) => chiavi.has(phraseKey(p)));
    inizio = Math.min(...scelteFrasi.map((p) => p.inizioMs));
    fine = Math.max(...scelteFrasi.map((p) => p.fineMs));
    const sopra = phrases.filter(
      (p) =>
        !chiavi.has(phraseKey(p)) &&
        Math.min(fine, p.fineMs) - Math.max(inizio, p.inizioMs) >
          SOVRAPPOSIZIONE_MS
    );
    for (const p of sopra) {
      chiavi.add(phraseKey(p));
    }
    cresciuta = sopra.length > 0;
  }
  const altre = phrases.filter((p) => !chiavi.has(phraseKey(p)));
  // Ogni altra Frase comincia prima o dopo l'inizio della scelta: nessuna resta fuori dal conto.
  const precedenti = altre
    .filter((p) => p.inizioMs < inizio)
    .map((p) => p.fineMs);
  const seguenti = altre
    .filter((p) => p.inizioMs >= inizio)
    .map((p) => p.inizioMs);
  return {
    // Se la Frase dopo si accavalla alla scelta, il bordo resta alla fine della scelta.
    fineMs:
      seguenti.length === 0
        ? durataMs
        : prima(fine, Math.max(fine, Math.min(...seguenti))),
    inizioMs:
      precedenti.length === 0
        ? 0
        : dopo(Math.min(inizio, Math.max(...precedenti)), inizio),
  };
}

/** Il Taglio del silenzio tra `a` e `b`: 1 s per lato, tutto se è all'inizio o alla fine del Tape. */
export function taglioDelSilenzio(
  a: number,
  b: number,
  durataMs: number
): Taglio {
  return {
    fineMs: b >= durataMs ? durataMs : prima(a, b),
    inizioMs: a <= 0 ? 0 : dopo(a, b),
  };
}

/** I Tagli con `nuovo`, in ordine e uniti dove si toccano. */
export function conTaglio(tagli: Taglio[], nuovo: Taglio): Taglio[] {
  const uniti: Taglio[] = [];
  for (const t of [...tagli, nuovo].sort((x, y) => x.inizioMs - y.inizioMs)) {
    const [last] = uniti.slice(-1);
    if (last && t.inizioMs <= last.fineMs) {
      last.fineMs = Math.max(last.fineMs, t.fineMs);
    } else {
      uniti.push({ ...t });
    }
  }
  return uniti;
}

/** Se il tratto `[inizioMs, fineMs]` sta tutto dentro un Taglio: allora sparisce con lui. */
export function tagliato(
  tagli: Taglio[],
  { fineMs, inizioMs }: { fineMs: number; inizioMs: number }
): boolean {
  return tagli.some((t) => t.inizioMs <= inizioMs && fineMs <= t.fineMs);
}

/** Quanto audio tolgono i Tagli. */
export function durataTagliata(tagli: Taglio[]): number {
  return tagli.reduce((n, t) => n + t.fineMs - t.inizioMs, 0);
}

/**
 * Se un Taglio toglie solo una parte di un Turno corretto a mano: il testo corretto resta sulle
 * Frasi rimaste, e la conferma lo dice.
 */
export function correzioniDivise(
  phrases: TranscriptPhrase[],
  tagli: Taglio[]
): boolean {
  const gruppi = new Map<string, boolean[]>();
  for (const p of phrases) {
    if (p.testoTurno !== null && p.testoTurno !== undefined) {
      const key = `${p.ingresso}:${p.testoTurno}`;
      gruppi.set(key, [...(gruppi.get(key) ?? []), tagliato(tagli, p)]);
    }
  }
  return [...gruppi.values()].some(
    (stati) => stati.includes(true) && stati.includes(false)
  );
}
