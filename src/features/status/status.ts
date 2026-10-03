import type { TFunction } from "i18next";
import type { AppError, commands } from "@/bindings";

/** Cosa mostra la status bar: la fase dell'Attività, o l'errore. */
export type Status =
  | { phase: "idle"; source: string | null }
  | { phase: "transcribing"; percent: number | null }
  | { phase: "finished"; txtPath: string; chars: number }
  | { phase: "noSpeech" }
  | { phase: "cancelled" }
  | { phase: "failed"; error: AppError };

export function statusText(status: Status, t: TFunction): string {
  switch (status.phase) {
    case "idle":
      return status.source ?? t("status.idle");
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

/** Applica `transcription-progress`, ignorando gli eventi arrivati dopo la fine. */
export function withProgress(status: Status, percent: number | null): Status {
  return status.phase === "transcribing" ? { ...status, percent } : status;
}

type TranscribeResult = Awaited<ReturnType<typeof commands.transcribe>>;

/** La status bar alla fine di `transcribe`: Annulla e "nessun parlato" non sono errori. */
export function afterTranscription(result: TranscribeResult): Status {
  if (result.status === "error") {
    return result.error.code === "cancelled"
      ? { phase: "cancelled" }
      : { error: result.error, phase: "failed" };
  }
  if (result.data.outcome === "noSpeech") {
    return { phase: "noSpeech" };
  }
  const { chars, txtPath } = result.data;
  return { chars, phase: "finished", txtPath };
}
