import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import type { ModelInfo } from "@/bindings";
import {
  mebibytes,
  stateText,
  withDownloadProgress,
  withState,
} from "@/features/models/models";

const t = i18n.t.bind(i18n);

const nemotron: ModelInfo = {
  error: null,
  id: "nemotron",
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

test("model-state-changed aggiorna solo il modello indicato, errore compreso", () => {
  const error = { code: "verificationFailed", detail: "SHA" } as const;
  const models = withState([nemotron, whisper], {
    error,
    modelId: "whisper",
    state: { state: "notDownloaded" },
  });
  expect(models[0]).toBe(nemotron);
  expect(models[1]).toEqual({ ...whisper, error });
});

test("la percentuale si applica solo a un download in corso", () => {
  const downloading = withState([nemotron], {
    error: null,
    modelId: "nemotron",
    state: { percent: 0, state: "downloading" },
  });
  expect(
    withDownloadProgress(downloading, { modelId: "nemotron", percent: 42 })[0]
      ?.state
  ).toEqual({ percent: 42, state: "downloading" });
  // Un avanzamento arrivato dopo la fine non riporta il modello in download.
  const verifying = withState([nemotron], {
    error: null,
    modelId: "nemotron",
    state: { state: "verifying" },
  });
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
