# mcp-stdio-client

Simple Rust CLI for MCP stdio testing.

Examples:
```bash
mcp-stdio-client --command /home/matthew/nexcore/target/release/claude-fs-mcp --list-tools
mcp-stdio-client --command /home/matthew/nexcore/target/release/claude-fs-mcp --call claude_fs_list --args-json '{"path":"."}'
```
