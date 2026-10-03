import type { TFunction } from "i18next";
import type { AppError, commands, TranscriptionOutcome } from "@/bindings";

/** Cosa mostra la status bar: la fase dell'Attività, o l'errore. */
export type Status =
  | { phase: "idle"; source: string | null }
  | { phase: "recording"; paused: boolean; liveError?: AppError }
  | { phase: "completing"; percent: number | null }
  | { phase: "recorded"; path: string }
  | { phase: "transcribing"; percent: number | null }
  | { phase: "finished"; txtPath: string; chars: number }
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
      return status.percent === null
        ? t("status.completing")
        : t("status.completingPercent", { percent: status.percent });
    case "recorded":
      return t("status.recorded", { path: status.path });
    case "transcribing":
      return status.percent === null
        ? t("status.transcribing")
        : t("status.transcribingPercent", { percent: status.percent });
    case "finished":
      return t("status.finished", {
        count: status.chars,
        path: status.txtPath,
      });
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
    case "completing":
      return { percent, phase: "completing" };
    default:
      return status;
  }
}

/** Applica `live-transcription-failed`: la Registrazione continua e la status bar lo dice. */
export function withLiveError(status: Status, liveError: AppError): Status {
  return status.phase === "recording" ? { ...status, liveError } : status;
}

/** Se accanto al messaggio serve il link alle Impostazioni, per scaricare o cambiare modello. */
export function needsSettings(status: Status): boolean {
  const code = shownError(status)?.code;
  return code === "modelMissing" || code === "liveTranscriptionUnavailable";
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
  return result.status === "error"
    ? failedStatus(result.error)
    : outcomeStatus(result.data);
}

/** La status bar per l'esito di una Trascrizione arrivata alla fine, anche dal vivo. */
export function outcomeStatus(outcome: TranscriptionOutcome): Status {
  if (outcome.outcome === "noSpeech") {
    return { phase: "noSpeech" };
  }
  const { chars, txtPath } = outcome;
  return { chars, phase: "finished", txtPath };
}

/** La status bar per una Trascrizione fallita: Annulla non è un errore. */
export function failedStatus(error: AppError): Status {
  return error.code === "cancelled"
    ? { phase: "cancelled" }
    : { error, phase: "failed" };
}
