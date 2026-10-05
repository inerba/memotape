import type { TFunction } from "i18next";
import type { AppError, commands } from "@/bindings";
import { movedPath } from "@/features/source/file-name";

/** Cosa mostra la status bar: la fase dell'Attività, o l'errore. */
export type Status =
  | { phase: "idle"; source: string | null }
  | { phase: "recording"; paused: boolean; liveError?: AppError }
  | { phase: "completing"; percent: number | null; diarizing?: boolean }
  | { phase: "recorded"; path: string }
  | { phase: "transcribing"; percent: number | null; diarizing?: boolean }
  | { phase: "finished"; path: string }
  | { phase: "noSpeech" }
  | { phase: "cancelled" }
  | { phase: "failed"; error: AppError };

export function statusText(status: Status, t: TFunction): string {
  switch (status.phase) {
    case "idle":
      return status.source ?? t("status.idle");
    case "recording":
      if (status.liveError) {
        return errorText(status.liveError, t);
      }
      return status.paused
        ? t("status.recordingPaused")
        : t("status.recording");
    case "completing":
      if (status.diarizing) {
        return t("status.diarizing");
      }
      return status.percent === null
        ? t("status.completing")
        : t("status.completingPercent", { percent: status.percent });
    case "recorded":
      return t("status.recorded", { path: status.path });
    case "transcribing":
      if (status.diarizing) {
        return t("status.diarizing");
      }
      return status.percent === null
        ? t("status.transcribing")
        : t("status.transcribingPercent", { percent: status.percent });
    case "finished":
      return t("status.finished", { path: status.path });
    case "noSpeech":
      return t("status.noSpeech");
    case "cancelled":
      return t("status.cancelled");
    case "failed":
      return errorText(status.error, t);
    default:
      return status satisfies never;
  }
}

/** Il messaggio tradotto di un errore applicativo. */
export function errorText(error: AppError, t: TFunction): string {
  return t(`errors.codes.${error.code}`, {
    detail: "detail" in error ? error.detail : "",
  });
}

/**
 * Applica `transcription-progress`, ignorando gli eventi arrivati dopo la fine. Durante una
 * Registrazione arriva solo dopo Stop, quando si smaltisce la coda della Trascrizione dal vivo.
 */
export function withProgress(status: Status, percent: number | null): Status {
  switch (status.phase) {
    case "transcribing":
      return { ...status, percent };
    case "recording":
      return { percent, phase: "completing" };
    case "completing":
      // Un avanzamento in ritardo non toglie "Riconoscimento dei parlanti…".
      return status.diarizing ? status : { percent, phase: "completing" };
    default:
      return status;
  }
}

/**
 * Applica `diarization-started`: la Trascrizione è finita (o, dopo Stop, la coda della Trascrizione
 * dal vivo), ora si riconoscono i parlanti.
 */
export function withDiarizing(status: Status): Status {
  return status.phase === "transcribing" || status.phase === "completing"
    ? { diarizing: true, percent: null, phase: status.phase }
    : status;
}

/** Applica `live-transcription-failed`: la Registrazione continua e la status bar lo dice. */
export function withLiveError(status: Status, liveError: AppError): Status {
  return status.phase === "recording" ? { ...status, liveError } : status;
}

/**
 * La status bar dopo che il Tape o la Raccolta `from`, forse con la Sorgente che mostra, è diventato
 * `to`.
 */
export function withMovedSource(
  status: Status,
  from: string,
  to: string
): Status {
  return status.phase === "idle"
    ? { ...status, source: status.source && movedPath(status.source, from, to) }
    : status;
}

/** Se accanto al messaggio serve il link alle Impostazioni, per scaricare o cambiare modello. */
export function needsSettings(status: Status): boolean {
  const code = shownError(status)?.code;
  return (
    code === "modelMissing" ||
    code === "diarizerMissing" ||
    code === "liveTranscriptionUnavailable"
  );
}

function shownError(status: Status): AppError | undefined {
  if (status.phase === "failed") {
    return status.error;
  }
  return status.phase === "recording" ? status.liveError : undefined;
}

type TranscribeResult = Awaited<ReturnType<typeof commands.transcribe>>;

/** La status bar alla fine di `transcribe`: Annulla e "nessun parlato" non sono errori. */
export function afterTranscription(result: TranscribeResult): Status {
  if (result.status === "error") {
    return failedStatus(result.error);
  }
  return result.data.outcome === "saved"
    ? { path: result.data.path, phase: "finished" }
    : { phase: "noSpeech" };
}

/** La status bar per una Trascrizione fallita: Annulla non è un errore. */
export function failedStatus(error: AppError): Status {
  return error.code === "cancelled"
    ? { phase: "cancelled" }
    : { error, phase: "failed" };
}

/** L'avviso in cima al pannello centrale per la fase: errori ed esiti, non l'avanzamento. */
export interface Banner {
  /** Serve il link alle Impostazioni, per scaricare o cambiare modello. */
  settings: boolean;
  text: string;
  tone: "error" | "info";
}

/**
 * L'avviso della fase `status`: l'errore (anche quello della Trascrizione dal vivo durante la
 * Registrazione) o l'esito di un'Attività finita. `null` a riposo e durante un'Attività, la cui
 * fase sta nella barra laterale.
 */
export function bannerOf(status: Status, t: TFunction): Banner | null {
  switch (status.phase) {
    case "failed":
      return {
        settings: needsSettings(status),
        text: statusText(status, t),
        tone: "error",
      };
    case "recording":
      return status.liveError
        ? {
            settings: needsSettings(status),
            text: statusText(status, t),
            tone: "error",
          }
        : null;
    case "finished":
    case "recorded":
    case "noSpeech":
    case "cancelled":
      return { settings: false, text: statusText(status, t), tone: "info" };
    default:
      return null;
  }
}

/** La percentuale di una Trascrizione o del completamento dopo Stop; `null` se non è nota. */
export function progressPercent(status: Status): number | null {
  return (status.phase === "transcribing" || status.phase === "completing") &&
    !status.diarizing
    ? status.percent
    : null;
}
