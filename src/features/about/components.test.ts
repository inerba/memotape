import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { COMPONENTS } from "@/features/about/components";

const LICENSES = join(import.meta.dir, "../../../src-tauri/resources/licenses");

test("Informazioni elenca le licenze della spec, Parakeet per primo", () => {
  expect(COMPONENTS.map((c) => [c.name, c.license])).toEqual([
    ["Parakeet TDT v3 0.6B", "CC-BY-4.0"],
    ["Nemotron Streaming 3.5 0.6B", "OpenMDW-1.1"],
    ["Whisper Large v3 Turbo", "MIT"],
    ["Silero VAD v4", "MIT"],
    ["Symphonia", "MPL-2.0"],
    ["transcribe.cpp", "MIT"],
    ["ONNX Runtime", "MIT"],
    ["vad-rs", "MIT"],
  ]);
});

test("ogni testo di licenza è tra le risorse", () => {
  for (const c of COMPONENTS) {
    for (const file of [c.file, c.notices ?? c.file]) {
      expect([file, existsSync(join(LICENSES, file))]).toEqual([file, true]);
    }
  }
});
