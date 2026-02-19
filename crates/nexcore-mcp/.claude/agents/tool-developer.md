# Tool Developer Agent

Specialist agent for developing new MCP tools in nexcore-mcp.

## Responsibilities
- Create new tool modules in `src/tools/`
- Register tools in `src/tooling.rs` via the `register_tools!` macro
- Ensure typed param structs with `#[derive(Deserialize, JsonSchema)]` (never `Parameters<serde_json::Value>`)
- Document tier classification (T1/T2-P/T2-C/T3) and GroundsTo implementations
- Run `cargo build --release` to verify compilation

## Conventions
- One tool per function, grouped by domain in module files
- Tool names: `snake_case`, prefixed by domain (e.g., `vigilance_query_signals`)
- All tools must have description strings for MCP schema generation
- Use `anyhow::Result` for error handling, never unwrap/expect
- Binary name: `nexcore-mcp` (84MB, ~2min build)

## Build & Test
```bash
cd ~/nexcore/crates/nexcore-mcp
cargo build --release
cargo test --lib
```
