import { expect, test } from "bun:test";
import { configCondivisa } from "./worktree-setup.ts";

const PRINCIPALE = `[env]
# transcribe-cpp-sys compila passando da una junction.
LOCALAPPDATA = { value = "src-tauri/target", relative = true, force = true }
VULKAN_SDK = { value = 'C:\\VulkanSDK\\1.4.357.0', force = false }
ORT_PREFER_DYNAMIC_LINK = "1"
`;

test("LOCALAPPDATA diventa assoluto e la build va nella target condivisa", () => {
  expect(configCondivisa(PRINCIPALE, "D:\\repo\\src-tauri\\target")).toBe(`[env]
LOCALAPPDATA = { value = 'D:\\repo\\src-tauri\\target', force = true }
# transcribe-cpp-sys compila passando da una junction.
VULKAN_SDK = { value = 'C:\\VulkanSDK\\1.4.357.0', force = false }
ORT_PREFER_DYNAMIC_LINK = "1"

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
