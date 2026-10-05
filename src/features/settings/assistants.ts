/**
 * Come si collega Sbobino a un Assistente (ADR-0012): il server MCP è l'exe stesso con `--mcp`.
 * `JSON.stringify` dà una stringa valida sia in JSON sia in TOML (una stringa "basic" con gli
 * escape), anche con un apostrofo nel percorso.
 */
export function assistantConfigs(exe: string) {
  const command = JSON.stringify(exe);
  return {
    claudeCode: `claude mcp add --scope user sbobino -- "${exe}" --mcp`,
    claudeDesktop: [
      "{",
      '  "mcpServers": {',
      '    "sbobino": {',
      `      "command": ${command},`,
      '      "args": ["--mcp"]',
      "    }",
      "  }",
      "}",
    ].join("\n"),
    codex: [
      "[mcp_servers.sbobino]",
      `command = ${command}`,
      'args = ["--mcp"]',
      'default_tools_approval_mode = "writes"',
    ].join("\n"),
  };
}

export type AssistantConfig = keyof ReturnType<typeof assistantConfigs>;
