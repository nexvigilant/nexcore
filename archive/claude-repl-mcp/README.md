# claude-repl-mcp

MCP bridge to Claude Code CLI. Exposes a single tool `claude_repl` that forwards prompts to the local `claude` binary.

Defaults:
- CLI path: `/home/matthew/.local/bin/claude` (override with `CLAUDE_CLI_PATH`)
- Non-interactive mode: `--print`
- Max output: 1,000,000 bytes (override with `max_output_bytes`)

Example MCP params:
```json
{"prompt":"Hello","model":"sonnet","persist_session":false,"max_output_bytes":100000}
```
