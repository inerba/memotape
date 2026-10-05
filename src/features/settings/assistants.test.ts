import { expect, test } from "bun:test";
import { assistantConfigs } from "@/features/settings/assistants";

const EXE = String.raw`C:\Users\O'Neil\AppData\Local\Memotape\memotape.exe`;

test("il comando di Claude Code registra Memotape per tutti i progetti", () => {
  expect(assistantConfigs(EXE).claudeCode).toBe(
    String.raw`claude mcp add --scope user memotape -- "C:\Users\O'Neil\AppData\Local\Memotape\memotape.exe" --mcp`
  );
});

test("la tabella di Codex è TOML valido anche con un apostrofo nel percorso", () => {
  expect(assistantConfigs(EXE).codex).toBe(
    [
      "[mcp_servers.memotape]",
      String.raw`command = "C:\\Users\\O'Neil\\AppData\\Local\\Memotape\\memotape.exe"`,
      'args = ["--mcp"]',
      'default_tools_approval_mode = "writes"',
    ].join("\n")
  );
});

test("il blocco di Claude Desktop è JSON con il percorso e --mcp", () => {
  expect(JSON.parse(assistantConfigs(EXE).claudeDesktop)).toEqual({
    mcpServers: { memotape: { args: ["--mcp"], command: EXE } },
  });
});
