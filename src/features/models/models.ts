import type { TFunction } from "i18next";
import type { ModelDownloadProgress, ModelInfo, ModelState } from "@/bindings";

/** Byte in MB binari, come li mostra Esplora file. */
export function mebibytes(bytes: number): number {
  return Math.round(bytes / 2 ** 20);
}

/** Applica `model-download-progress`, ignorando quelli arrivati dopo la fine del download. */
export function withDownloadProgress(
  models: ModelInfo[],
  { modelId, percent }: ModelDownloadProgress
): ModelInfo[] {
  return models.map((m) =>
    m.id === modelId && m.state.state === "downloading"
      ? { ...m, state: { percent, state: "downloading" } }
      : m
  );
}

export function stateText(state: ModelState, t: TFunction): string {
  switch (state.state) {
    case "notDownloaded":
      return t("models.state.notDownloaded");
    case "interrupted":
      return t("models.state.interrupted", { percent: state.percent });
    case "downloading":
      return t("models.state.downloading", { percent: state.percent });
    case "verifying":
      return t("models.state.verifying");
    case "downloaded":
      return t("models.state.downloaded");
    default:
      return state satisfies never;
  }
}
