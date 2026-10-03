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
      return t(`errors.codes.${status.error.code}`, {
        detail: "detail" in status.error ? status.error.detail : "",
      });
    default:
      return status satisfies never;
  }
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
