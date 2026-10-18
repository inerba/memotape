import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { CREDITS } from "@/features/about/credits";
import catalog from "../../../src-tauri/src/managers/models.json";

const LICENSES = join(import.meta.dir, "../../../src-tauri/resources/licenses");

test("Informazioni elenca le licenze della spec, prima i modelli CC BY 4.0", () => {
  expect(CREDITS.map((c) => [c.name, c.license, c.quantization])).toEqual([
    ["Parakeet TDT v3 0.6B", "CC-BY-4.0", "Q5_K_M"],
    ["Parakeet TDT primeLine 0.6B", "CC-BY-4.0", "Q8_0"],
    ["Canary 1B v2", "CC-BY-4.0", "Q5_K_M"],
    ["Nemotron Streaming 3.5 0.6B", "OpenMDW-1.1", "Q5_K_M"],
    ["Cohere Transcribe", "Apache-2.0", "Q5_K_M"],
    ["Qwen3-ASR 0.6B", "Apache-2.0", "Q8_0"],
    ["Qwen3-ASR 1.7B", "Apache-2.0", "Q5_K_M"],
    ["Voxtral Mini 3B", "Apache-2.0", "Q5_K_M"],
    ["Voxtral Mini 4B Realtime", "Apache-2.0", "Q5_K_M"],
    ["Voxtral Small 24B", "Apache-2.0", "Q5_K_M"],
    ["Whisper Large v3 Turbo", "MIT", "Q5_K_M"],
    ["Whisper Tiny", "MIT", "Q8_0"],
    ["Whisper Base", "MIT", "Q8_0"],
    ["Whisper Small", "MIT", "Q8_0"],
    ["Whisper Medium", "MIT", "Q8_0"],
    ["Whisper Large", "MIT", "Q5_K_M"],
    ["Whisper Large v2", "MIT", "Q5_K_M"],
    ["Whisper Large v3", "MIT", "Q5_K_M"],
    ["Sortformer 4spk v2.1", "NVIDIA Open Model License", "Q8_0"],
    ["RNNoise 0.2", "BSD-3-Clause", undefined],
    ["Silero VAD v4", "MIT", undefined],
    ["Symphonia", "MPL-2.0", undefined],
    ["transcribe.cpp", "MIT", undefined],
    ["ONNX Runtime", "MIT", undefined],
    ["vad-rs", "MIT", undefined],
    ["Inter", "OFL-1.1", undefined],
    ["Commissioner", "OFL-1.1", undefined],
  ]);
});

test("ogni modello del catalogo ha i suoi crediti", () => {
  const credited = new Set(CREDITS.map((c) => c.name));
  for (const m of catalog.modelli) {
    expect([m.nome, credited.has(m.nome)]).toEqual([m.nome, true]);
  }
});

test("ogni testo di licenza è tra le risorse", () => {
  for (const c of CREDITS) {
    for (const file of [c.file, c.notices ?? c.file]) {
      expect([file, existsSync(join(LICENSES, file))]).toEqual([file, true]);
    }
  }
});
