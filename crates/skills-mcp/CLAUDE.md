# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Skills MCP server — exposes vocabulary programs (LEARN, PROVE, VITALS) and T1 Lex Primitiva tools as MCP tools over stdio. Native Rust implementations replace shell scripts for the most critical programs; a generic runner shells out to remaining programs.

## Build & Test

```bash
cargo check                    # Fast type check
cargo build --release          # Release binary → target/release/skills-mcp
cargo test --lib               # Unit tests
cargo clippy -- -D warnings    # Lint

# Test via MCP protocol (stdin/stdout JSON-RPC)
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run 2>/dev/null
```

Binary is used as an MCP server in `~/.claude/settings.json` under `mcpServers`.

## Architecture

```
src/
├── main.rs          # Tokio entrypoint, stdio transport via rmcp
├── lib.rs           # SkillsMcpServer struct + all #[tool] method declarations
├── types.rs         # Param structs (EmptyParams, SkillRunParams, Primitive*Params)
└── tools/
    ├── mod.rs       # Module declarations
    ├── common.rs    # Shared helpers: brain_db(), json_result(), mcp_err(), etc.
    ├── vitals.rs    # VITALS [V,I,T,A,L,S] — 6 phases + pipeline (7 tools)
    ├── learn.rs     # LEARN [L,E,A,R,N] — 5 phases + pipeline (6 tools)
    ├── prove.rs     # PROVE [P,O,V,E] — 4 phases (4 tools, R=Run is shell-only)
    ├── primitives.rs# T1 Lex Primitiva — decompose, compose, list_all (3 tools)
    └── runner.rs    # Generic skill runner — shells out to any program (2 tools)
```

**Total: 22 tools** (17 native + 2 runner + 3 primitives)

## Adding a New Tool

1. **Add param struct** to `types.rs` with `#[derive(Debug, Deserialize, JsonSchema)]` and `#[serde(crate = "rmcp::serde")]`
2. **Create implementation** in `tools/<module>.rs` returning `Result<CallToolResult, McpError>`
3. **Add module** to `tools/mod.rs` if new file
4. **Wire the `#[tool]`** method in `lib.rs` under `#[tool_router] impl SkillsMcpServer`

Use `EmptyParams` for tools with no arguments. Use `json_result()` / `text_result()` from `common.rs` to build responses.

## Key Dependencies

| Crate | Role |
|-------|------|
| `rmcp` 0.14 | MCP protocol (server, stdio transport, `#[tool]` / `#[tool_router]` macros) |
| `rusqlite` | Direct read/write to `~/.claude/brain/brain.db` |
| `schemars` 1.2 | JSON Schema generation for tool params (rmcp requirement) |
| `chrono` | Timestamps in telemetry and history files |

## Data Paths (via `common.rs`)

| Helper | Path |
|--------|------|
| `brain_db_path()` | `~/.claude/brain/brain.db` |
| `telemetry_dir()` | `~/.claude/brain/telemetry/` |
| `implicit_dir()` | `~/.claude/implicit/` |
| `hormones_path()` | `~/.claude/hormones/state.json` |
| `skills_dir()` | `~/.claude/skills/` |
| `settings_path()` | `~/.claude/settings.json` |
| `memory_md_path()` | `~/.claude/projects/-home-matthew/memory/MEMORY.md` |

## Gotchas

- **`Parameters<serde_json::Value>`** generates broken MCP schema — always use typed param structs
- **`schemars` 1.x** — rmcp 0.14 uses schemars 1.2, not 0.8. `JsonSchema` derive syntax differs.
- **`#[serde(crate = "rmcp::serde")]`** required on all param structs — rmcp re-exports serde
- **No workspace root** — this crate has its own `[workspace]` in Cargo.toml, builds standalone
- **Shell runner** — `skill_run` executes scripts from `~/.claude/skills/<dir>/scripts/`. Scripts must be executable.
- **brain.db schema** — tables: sessions, artifacts, artifact_versions, beliefs, patterns, preferences, decisions_audit, antibodies, trust_accumulators, corrections, tracked_files, tasks_history, tool_usage, token_efficiency, schema_version, belief_implications
