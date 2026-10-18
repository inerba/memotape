import { expect, test } from "bun:test";
import { configCondivisa } from "./worktree-setup.ts";

const PRINCIPALE = `[env]
# transcribe-cpp-sys compila passando da una junction.
LOCALAPPDATA = { value = "src-tauri/target", relative = true, force = true }
VULKAN_SDK = { value = 'C:\\VulkanSDK\\1.4.357.0', force = false }
`;

test("LOCALAPPDATA diventa assoluto e la build va nella target condivisa", () => {
  expect(configCondivisa(PRINCIPALE, "D:\\repo\\src-tauri\\target")).toBe(`[env]
LOCALAPPDATA = { value = 'D:\\repo\\src-tauri\\target', force = true }
# transcribe-cpp-sys compila passando da una junction.
VULKAN_SDK = { value = 'C:\\VulkanSDK\\1.4.357.0', force = false }

[build]
target-dir = 'D:\\repo\\src-tauri\\target'
`);
});

test("una sezione [build] che c'era si sostituisce", () => {
  const config = configCondivisa(
    `${PRINCIPALE}\n[build]\ntarget-dir = 'altrove'\n`,
    "T"
  );
  expect(config.match(/\[build\]/g)).toHaveLength(1);
  expect(config).toContain("target-dir = 'T'");
  expect(config).not.toContain("altrove");
});

test("le variabili di ONNX Runtime della config principale non passano al worktree", () => {
  const config = configCondivisa(
    `${PRINCIPALE}ORT_LIB_LOCATION = 'D:\\deps\\onnxruntime\\lib'\nORT_PREFER_DYNAMIC_LINK = "1"\n`,
    "T"
  );
  expect(config).not.toContain("ORT_");
  expect(config).toContain("VULKAN_SDK");
});
