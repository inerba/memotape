import { expect, test } from "bun:test";
import { assistantConfigs } from "@/features/settings/assistants";

const EXE = String.raw`C:\Users\O'Neil\AppData\Local\Sbobino\sbobino.exe`;

test("il comando di Claude Code registra Sbobino per tutti i progetti", () => {
  expect(assistantConfigs(EXE).claudeCode).toBe(
    String.raw`claude mcp add --scope user sbobino -- "C:\Users\O'Neil\AppData\Local\Sbobino\sbobino.exe" --mcp`
  );
});

test("la tabella di Codex è TOML valido anche con un apostrofo nel percorso", () => {
  expect(assistantConfigs(EXE).codex).toBe(
    [
      "[mcp_servers.sbobino]",
      String.raw`command = "C:\\Users\\O'Neil\\AppData\\Local\\Sbobino\\sbobino.exe"`,
      'args = ["--mcp"]',
      'default_tools_approval_mode = "writes"',
    ].join("\n")
  );
});

test("il blocco di Claude Desktop è JSON con il percorso e --mcp", () => {
  expect(JSON.parse(assistantConfigs(EXE).claudeDesktop)).toEqual({
    mcpServers: { sbobino: { args: ["--mcp"], command: EXE } },
  });
});
