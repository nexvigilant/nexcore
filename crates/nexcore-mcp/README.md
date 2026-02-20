# nexcore-mcp

The primary Model Context Protocol (MCP) server for Claude Code integration. This crate exposes NexCore's high-performance Rust kernel as a massive suite of 450+ tools, organized by domain.

## Intent
To provide AI agents with direct, in-process access to pharmacovigilance algorithms, workspace management tools, and the Lex Primitiva foundation. It bridges the gap between high-level AI reasoning and low-level axiomatic computation.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **μ (Mapping)**: The core primitive for mapping Claude's tool calls to internal Rust handlers.
- **→ (Causality)**: Manages the sequence of tool execution and data flow.
- **∂ (Boundary)**: Enforces tool gating (allow/deny lists) and scan limits.
- **N (Quantity)**: Manages the sheer scale of the tool catalog (305+ unified commands).

## Core Architecture
- **Unified Dispatcher**: Single `nexcore` tool for efficient discovery and execution.
- **Tool Gating**: Environment-driven allow/deny lists (`NEXCORE_MCP_TOOL_DENYLIST`).
- **Forensic Metadata**: Automatic attachment of confidence scores and signal categories to results.
- **Telemetry**: Instrumented tracking of tool latency, success rates, and token efficiency.

## Tool Categories (Sample)
| Category | Tools | Description |
| :--- | :---: | :--- |
| **Foundation** | 9 | Edit distance, fuzzy search, graph topsort, FSRS. |
| **PV** | 8 | PRR, ROR, IC, EBGM, causality (Naranjo/WHO). |
| **Vigilance** | 4 | Safety margins, harm types, manifold boundaries. |
| **Skills** | 22 | Registry scanning, validation, and execution. |
| **Brain** | 25 | Session creation, artifact versioning, engram verify. |

## SOPs for Use
### Running the Server
```bash
./target/release/nexcore-mcp
```
Connect via stdio in your MCP configuration.

### Adding a new Tool
1. Define the tool function in a domain module under `src/tools/`.
2. Register the tool in `src/lib.rs` (for individual mode).
3. Register the tool in `src/unified.rs` (for unified dispatcher mode).
4. Update the `help_catalog` in `unified.rs`.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
