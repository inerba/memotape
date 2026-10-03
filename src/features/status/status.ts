import type { TFunction } from "i18next";
import type { AppError } from "@/bindings";

/** Cosa mostra la status bar: la fase dell'Attività, o l'errore. */
export type Status =
  | { phase: "idle"; source: string | null }
  | { phase: "transcribing"; percent: number | null }
  | { phase: "finished"; txtPath: string; chars: number }
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
    case "failed":
      return t(`errors.codes.${status.error.code}`, {
        detail: status.error.detail,
      });
    default:
      return status satisfies never;
  }
}

/** Applica `transcription-progress`, ignorando gli eventi arrivati dopo la fine. */
export function withProgress(status: Status, percent: number | null): Status {
  return status.phase === "transcribing" ? { ...status, percent } : status;
}
