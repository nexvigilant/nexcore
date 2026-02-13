# claude-mcp-config

Rust CLI to manage `.claude/settings.json` and global `.mcp.json` MCP server entries.

Examples:
```bash
claude-mcp-config add-server --scope claude --name claude-repl --command /home/matthew/nexcore/target/release/claude-repl-mcp
claude-mcp-config allow-tool --pattern mcp__claude-repl__*
```
