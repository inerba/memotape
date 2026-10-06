import type { TFunction } from "i18next";
import type {
  AppError,
  commands,
  Levels,
  LiveTranscription,
  RecordingSource,
} from "@/bindings";
import {
  failedStatus,
  progressPercent,
  type Status,
  statusText,
} from "@/features/status/status";

/** Il livello più basso mostrato, in dBFS: sotto è silenzio. */
const FLOOR_DB = -60;

const two = (n: number) => String(n).padStart(2, "0");

/** Il timer della Registrazione: `m:ss`, oppure `h:mm:ss` dalla prima ora. */
export function elapsedText(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const h = Math.floor(seconds / 3600);
  const m = Math.floor(seconds / 60) % 60;
  const s = two(seconds % 60);
  return h > 0 ? `${h}:${two(m)}:${s}` : `${m}:${s}`;
}

/** Il picco (0–1) di `recording-tick` come percentuale dell'indicatore, in scala di decibel. */
export function levelPercent(peak: number | null): number {
  if (!peak || peak <= 0) {
    return 0;
  }
  const db = 20 * Math.log10(Math.min(peak, 1));
  return Math.round(Math.max(0, 1 - db / FLOOR_DB) * 100);
}

/** Un indicatore di livello: l'ingresso e la percentuale. */
export interface Meter {
  input: keyof Levels;
  percent: number;
}

/** Gli indicatori di `recording-tick`: uno per ingresso registrato, prima il microfono. */
export function meters(levels: Levels): Meter[] {
  return (["microphone", "system"] as const).flatMap((input) => {
    const peak = levels[input];
    return peak === null ? [] : [{ input, percent: levelPercent(peak) }];
  });
}

/**
 * I livelli a zero degli ingressi di `source` (la sorgente di registrazione delle impostazioni),
 * prima del primo `recording-tick`.
 */
export function silentLevels(source: RecordingSource): Levels {
  return {
    microphone: source === "system" ? null : 0,
    system: source === "mic" ? null : 0,
  };
}

type RecordResult = Awaited<ReturnType<typeof commands.record>>;

/**
 * La Sorgente e la status bar alla fine di `record`. Il file salvato diventa la Sorgente anche se
 * il dispositivo si è scollegato; se la Registrazione non è partita, `source` è assente. Con la
 * Trascrizione dal vivo la status bar dice com'è finita, dopo l'eventuale errore del dispositivo.
 */
export function afterRecording(result: RecordResult): {
  source?: string;
  status: Status;
} {
  if (result.status === "error") {
    return { status: { error: result.error, phase: "failed" } };
  }
  const { error, path, transcription } = result.data;
  const status = recordedStatus(path, error, transcription);
  if (status.phase === "finished" && result.data.diarizzazione) {
    status.diarizzazione = result.data.diarizzazione;
  }
  if (
    status.phase === "finished" &&
    result.data.diarizzazione &&
    result.data.diarizzazione.esito !== "completata"
  ) {
    status.diarizzazioneNonCompletata = true;
  }
  return { source: path, status };
}

function recordedStatus(
  path: string,
  error: AppError | null,
  transcription: LiveTranscription | null
): Status {
  if (error) {
    return { error, phase: "failed" };
  }
  if (!transcription) {
    return { path, phase: "recorded" };
  }
  switch (transcription.outcome) {
    case "failed":
      return failedStatus(transcription.error);
    case "noSpeech":
      return { phase: "noSpeech" };
    default:
      return { path, phase: "finished" };
  }
}

/**
 * Lo stato in breve dell'Attività in corso, per la barra laterale: il timer della Registrazione
 * (`elapsedMs` dall'ultimo `recording-tick`) o la fase con la percentuale. `null` senza Attività.
 */
export function activityText(
  status: Status,
  elapsedMs: number,
  t: TFunction
): string | null {
  switch (status.phase) {
    case "recording":
      return t("library.activityRecording", {
        elapsed: elapsedText(elapsedMs),
      });
    case "transcribing":
    case "completing":
      return statusText(status, t);
    default:
      return null;
  }
}

/**
 * L'Attività in corso per la barra laterale: il testo di `activityText`, l'avanzamento (`null` se
 * non è noto) e se è una Registrazione. `null` senza Attività.
 */
export function activitySummary(
  status: Status,
  elapsedMs: number,
  t: TFunction
): { percent: number | null; recording: boolean; text: string } | null {
  const text = activityText(status, elapsedMs, t);
  return text
    ? {
        percent: progressPercent(status),
        recording: status.phase === "recording",
        text,
      }
    : null;
}
