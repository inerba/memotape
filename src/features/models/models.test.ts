import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import type { ModelInfo } from "@/bindings";
import {
  mebibytes,
  stateText,
  withDownloadProgress,
} from "@/features/models/models";

const t = i18n.t.bind(i18n);

const nemotron: ModelInfo = {
  acceptsLanguage: true,
  error: null,
  id: "nemotron",
  inUse: false,
  kind: "trascrizione",
  languages: null,
  license: "OpenMDW-1.1",
  mode: "stream",
  name: "Nemotron",
  recommended: true,
  size: 559_647_200,
  state: { state: "notDownloaded" },
};
const whisper: ModelInfo = {
  ...nemotron,
  id: "whisper",
  mode: "frase",
  name: "Whisper",
  recommended: false,
};

test("la dimensione del download è in MB come in Esplora file", () => {
  expect(mebibytes(559_647_200)).toBe(534);
  expect(mebibytes(619_628_128)).toBe(591);
});

test("la percentuale si applica solo al download in corso di quel modello", () => {
  const downloading: ModelInfo[] = [
    { ...nemotron, state: { percent: 0, state: "downloading" } },
    { ...whisper, state: { percent: 10, state: "downloading" } },
  ];
  const updated = withDownloadProgress(downloading, {
    modelId: "nemotron",
    percent: 42,
  });
  expect(updated[0]?.state).toEqual({ percent: 42, state: "downloading" });
  expect(updated[1]).toBe(downloading[1]);
  // Un avanzamento arrivato dopo la fine non riporta il modello in download.
  const verifying: ModelInfo[] = [
    { ...nemotron, state: { state: "verifying" } },
  ];
  expect(
    withDownloadProgress(verifying, { modelId: "nemotron", percent: 99 })
  ).toEqual(verifying);
});

test("lo stato del modello si legge in italiano", () => {
  expect(stateText({ state: "notDownloaded" }, t)).toBe("Non scaricato");
  expect(stateText({ percent: 40, state: "interrupted" }, t)).toBe(
    "Download interrotto al 40%"
  );
  expect(stateText({ percent: 7, state: "downloading" }, t)).toBe(
    "Download in corso… 7%"
  );
  expect(stateText({ state: "verifying" }, t)).toBe("Verifica dell'integrità…");
  expect(stateText({ state: "downloaded" }, t)).toBe("Scaricato");
});
