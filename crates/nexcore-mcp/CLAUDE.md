# AI Guidance — nexcore-mcp

Tool provider and dispatcher for Claude Code.

## Use When
- Exposing new Rust functionality to AI agents.
- Modifying the unified tool schema or help catalog.
- Debugging tool execution failures or confidence scoring.
- Managing tool gating or telemetry collection.

## Grounding Patterns
- **First-Class Tools**: New tools should be added as `#[tool]` methods on `NexCoreMcpServer` in `lib.rs` with typed param structs in `src/params/`. All tools are individually discoverable via MCP `list_tools()`. The `nexcore` unified dispatcher in `unified.rs` remains for backward compatibility.
- **Forensic Attachment**: Always call `attach_forensic_meta` for detection tools to enable automated downstream reasoning.
- **T1 Primitives**:
  - `μ + →`: Root primitives for dispatch and execution flow.
  - `∂ + N`: Root primitives for gating and scale management.

## Maintenance SOPs
- **Typed Params**: Never use `serde_json::Value` in a `#[tool]` signature. Always define a named struct in `src/params/`.
- **Result Wrapping**: Use `wrap_result` to ensure the Grammar Controller is applied to all text outputs.
- **Schema Validation**: Run `cargo check` after adding a tool to verify the `schemars` derivation.

## Key Entry Points
- `src/unified.rs`: The core match-table for the `nexcore` command.
- `src/tooling.rs`: Gating, limiting, and forensic attachment logic.
- `src/tools/`: Domain-specific tool implementations.
