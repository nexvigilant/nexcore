# CLAUDE.md — NexCore

Rust monorepo: 195 crates across 4 layers, Edition 2024 (Rust 1.85+).

## Build Commands

```bash
# Workspace builds (from ~/nexcore/)
cargo build -p <crate-name>            # Single crate
cargo test -p <crate-name> --lib       # Unit tests
cargo clippy -p <crate-name> -- -D warnings

# Or from crate directory
cd crates/<crate> && cargo build --release

# Key binaries
cargo build -p nexcore-mcp --release   # MCP server (84MB, ~2min)
cargo build -p nexcore-brain --release # Brain CLI

# Rebuild MCP tools (restart Claude Code to pick up new binary)
cargo build -p nexcore-mcp --release

# Justfile (530 recipes — preferred entry point)
just check <crate>     # cargo check
just clippy <crate>    # cargo clippy
just test <crate>      # cargo test
just crate-count       # inventory
just validate          # DAG validation pipeline
```

## Layer Architecture

Dependency flows DOWN only: Service → Orchestration → Domain → Foundation.

### Foundation (0-3 internal deps)
Core primitives, no domain knowledge.
- `nexcore-primitives`, `nexcore-lex-primitiva` (15 operational + × axiomatic = 16 in enum)
- `nexcore-id`, `nexcore-constants`, `nexcore-macros`
- `stem-*` (6 crates: core, derive, phys, math, plus 2 more)
- `nexcore-config`, `nexcore-traits`
- `nexcore-prima` (Lex Primitiva language interpreter)
- `nucli` (bijective byte↔DNA nucleotide codec, zero deps, 10 exhaustive proofs)

### Domain (2-25 internal deps)
Business logic, uses foundation types.
- `nexcore-vigilance` (57 modules, 76 PVOS — the domain monolith, 25 deps)
- `nexcore-tov` (Theory of Vigilance axioms)
- `nexcore-faers-etl`, `nexcore-pvos`, `nexcore-dtree`
- `nexcore-energy`, `nexcore-cytokine`, `nexcore-hormones`
- `nexcore-immunity`, `nexcore-synapse`, `nexcore-phenotype`
- `nexcore-statemind` (DNA pipeline), `nexcore-value-mining` (economic signal detection)
- `prima-*` (7 crates — Prima language ecosystem)

### Orchestration (3-5 internal deps)
Workflow coordination.
- `nexcore-friday` (event bus / Vigil orchestrator)
- `nexcore-brain` (sessions, artifacts, working memory)
- `nexcore-build-gate`
- `nexcore-skill-*`, `nexcore-signal-*`

### Service (5-76 internal deps)
External interfaces. Only layer with binary targets.
- `nexcore-mcp` (458 MCP tools, **76 internal deps** — pulls from everywhere)
- `nexcore-api` (84+ REST routes)
- `nexcore-cli`, `nexcore-guardian-cli`

### MCP Architecture (Direct stdio)
```
Claude Code ←stdio→ nexcore-mcp (direct binary)
```
- **Config**: `~/.claude.json` → nexcore entry points to `target/release/nexcore-mcp`
- **Reload**: `cargo build -p nexcore-mcp --release` then restart Claude Code

## Conventions

- **No unwrap/expect** — `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
- **No unsafe** — `#![forbid(unsafe_code)]` workspace-wide
- **No Python** — Enforced by hook. Shell glue in zsh only.
- **Workspace deps** — Define in root `Cargo.toml`, crates reference via `{ workspace = true }`
- **Error types** — `anyhow::Error` for binaries, crate-specific error enums for libraries
- **Write source files BEFORE `lib.rs`** — `cargo fmt` strips `mod` declarations for missing files
- **Edition 2024 patterns** — `|&(_, c)| *c` not `|(_, &c)| c`

## Key Gotchas

- `nexcore-foundation` is a **package alias** for `nexcore-vigilance` (root Cargo.toml:134). No directory exists for it.
- STEM crate names: `stem-bio` (not stem-biology), `stem-phys` (not stem-physics)
- `Parameters<serde_json::Value>` in `#[tool]` signatures generates broken schema — use typed param structs
- `nexcore-api` flaky test: `test_pause_resume_loop` — skip with `--skip test_pause_resume_loop`
- Binary name mismatches: `nexcore-build-gate` → `build-gate`, `nexcore-guardian-cli` → `nvrepl`
- **MCP `CallToolResult` error classification** — JSON payload `"success": false` MUST align with `CallToolResult::error()`. Never return `CallToolResult::success()` wrapping a failure payload. Pattern: `if success { Ok(CallToolResult::success(content)) } else { Ok(CallToolResult::error(content)) }`
- **Lex Primitiva count** — 15 operational + × (Product, axiomatic) = 16 in code. `LexPrimitiva::all()` returns `[Self; 16]`. × is excluded from operational tracing (trivially everywhere). Use `.len()` for code, "15" is correct in theoretical docs.
- **rmcp `Content` text extraction in tests** — Use `c.raw` field: `match &c.raw { RawContent::Text(t) => t.text.clone(), _ => String::new() }`. NOT `Content::Text(t)`.

## Directory Map

```
~/nexcore/
├── Cargo.toml          # Workspace root (174 members)
├── Justfile            # 530 recipes (preferred CLI)
├── crates/             # 172 Rust crates (4 layers)
├── tools/              # 2 utility crates (crate-converter, dag-publish)
├── studio/             # Separate: Next.js portal (see studio/CLAUDE.md)
├── scripts/            # Build/audit scripts
├── docs/               # Documentation
├── kellnr/             # Crate registry (docker-compose)
├── data/, dna/, ksb/   # Reference data
└── .build-orchestrator/ # DAG state
```

## AI Engineering Bible Tools (19 tools, 5 modules)

Source: "The AI Engineering Bible" — synthesized from 37 sections across 3 sessions.

| Module | Tools | Primitives | Use When |
|--------|-------|------------|----------|
| `drift_detection` | `drift_ks_test`, `drift_psi`, `drift_jsd`, `drift_detect` | ν+κ+∂+N | Comparing two data distributions for statistical shift |
| `rate_limiter` | `rate_limit_token_bucket`, `rate_limit_sliding_window`, `rate_limit_status` | ν+∂+ς+N | Throttling API/MCP calls, preventing overload |
| `rank_fusion` | `rank_fusion_rrf`, `rank_fusion_hybrid`, `rank_fusion_borda` | σ+μ+κ+N | Merging multiple ranked result lists |
| `security_posture` | `security_posture_assess`, `security_threat_readiness`, `security_compliance_gap` | ∂+∝+κ+π | Compliance scorecards (SOC2/HIPAA/GDPR/EU AI Act/NIST/ISO27001), AI threat readiness |
| `observability` | `observability_record_latency`, `observability_query`, `observability_freshness` | ν+N+∂+π | Inference latency percentiles (p50/p95/p99), data freshness SLAs, throughput tracking |

Key thresholds: PSI >0.25 = significant drift, KS alpha=0.05, RRF k=60, hybrid alpha=0.6, p99 <100ms=excellent.

## Biological Crate System

| Crate | Analog | Purpose |
|-------|--------|---------|
| `nexcore-cytokine` | Cytokines | Inter-crate event signaling |
| `nexcore-hormones` | Hormones | System-wide config propagation |
| `nexcore-immunity` | Immune system | Antipattern detection |
| `nexcore-energy` | ATP/ADP | Token budget management |
| `nexcore-synapse` | Synapses | Learning curves |
| `nexcore-transcriptase` | Reverse transcriptase | Schema inference |
| `nexcore-ribosome` | Ribosome | Schema-to-code generation |
| `nexcore-phenotype` | Phenotype | Adversarial test generation |
