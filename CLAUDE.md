# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# CLAUDE.md — NexCore

Rust monorepo: 281 workspace members across 4 layers, Edition 2024 (Rust 1.85+).

## Benchmark Governance (INVARIANT)

This document describes the REALIZED state of this codebase — the standard we hold.

**The One Rule:** When reality diverges from what is stated here, improve reality.

- NEVER update this document to match a lower reality
- ALWAYS identify the gap and take action to close it
- The bar moves UP only — through explicit revision to raise standards
- Track divergences as technical debt with priority proportional to gap size

This rule is ABSOLUTE. It overrides convenience, velocity, and scope pressure.

## Build Commands

```bash
# Single crate (preferred — faster than workspace)
cargo build -p <crate-name>
cargo test -p <crate-name> --lib       # Unit tests only
cargo clippy -p <crate-name> -- -D warnings
cargo test -p <crate-name> -- <test_name>  # Single test

# Key binaries
cargo build -p nexcore-mcp --release   # MCP server (~2min, restart Claude Code after)
cargo build -p nexcore-api --release   # REST API server
cargo build -p nexcore-brain --release # Brain CLI

# justfile (114 recipes — preferred entry point)
just check-crate <crate>  # cargo check single crate
just test-crate <crate>   # cargo test single crate
just test-match <crate> <pattern>  # Test with pattern filter
just clippy               # Workspace clippy
just fmt                  # Format all
just validate             # Full CI: fmt → clippy → test → docs → build (DAG orchestrator)
just validate-quick       # Quick: check → clippy → test-core (DAG orchestrator)
just validate-seq         # Sequential fallback (no orchestrator)
just sweep                # Quality sweep: fmt → clippy → deps → audit → test-compile
just sweep-fix            # Quality sweep with auto-fix
just coverage <crate>     # Test coverage via tarpaulin
just mcp-build            # Build MCP server
just services             # Build all service binaries
just up                   # Launch all services (build + tmux + API)
just up-fast              # Launch services (skip build)
just down                 # Graceful shutdown
```

## Layer Architecture

Dependency flows DOWN only: Service → Orchestration → Domain → Foundation.

### Foundation (0-3 internal deps)
Core primitives, no domain knowledge.
- `nexcore-primitives`, `nexcore-lex-primitiva` (15 operational + × axiomatic = 16 in enum)
- `nexcore-id`, `nexcore-constants`, `nexcore-macros`
- `stem-*` (9 crates: core, derive, phys, math, bio, finance, spatial, plus 2 more)
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
- `nexcore-mcp` (verify tool count: `nexcore_health_probe`, **76 internal deps** — pulls from everywhere)
- `nexcore-api` (196 REST routes)
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
- **Error types** — `nexcore-error` + `nexcore-error-derive` (anyhow removed). Crate-specific error enums for libraries.
- **Sovereignty campaign** — External deps actively replaced by internal crates: `anyhow`→`nexcore-error`, `base64`/`hex`→`nexcore-codec`, `dirs`/`glob`/`walkdir`→`nexcore-fs`, `uuid`→`nexcore-id`, `once_cell`→`std::sync::LazyLock`, `thiserror`→`nexcore-error-derive`. Check root `Cargo.toml` comments before adding external deps.
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
├── Cargo.toml          # Workspace root (231 members)
├── justfile            # 86 recipes (preferred CLI)
├── crates/             # 229 directories (4 layers)
├── tools/              # 2 utility crates (crate-converter, dag-publish)
├── studio/             # Legacy — active frontend is ~/Projects/Active/nucleus/
├── scripts/            # Build/audit scripts
├── docs/               # Documentation
│   └── sops/           # All SOPs (OPS/, QA/, DEV/, DEV-BIO/, SEC/, SPECIALIZED/)
├── boot/               # NexCore OS boot scripts (QEMU, Docker, initramfs)
├── kellnr/             # Crate registry (docker-compose)
├── data/, dna/, ksb/   # Reference data
└── .build-orchestrator/ # DAG state
```

## Build Orchestrator

DAG-based build pipeline (`nexcore-build-orchestrator`) replaces sequential CI:
- `just validate` — full pipeline: fmt → (clippy | test | docs) → build (parallel waves)
- `just validate-quick` — fast: check → (clippy | test-core)
- `just orc-status` — last pipeline status
- `just orc-plan` — dry-run DAG wave visualization
- State persisted in `.build-orchestrator/`
- Web dashboard: `just orc-serve` (port 3100)

## NexCore OS Subsystem

Experimental OS layer: `nexcore-pal` (Platform Abstraction), `nexcore-os`, `nexcore-compositor`, `nexcore-shell`, `nexcore-init` (PID 1).
- `just os-run` — virtual mode (default: desktop, 10 ticks)
- `just os-test` — test all OS crates
- `just os-boot-test` — all 3 form factors
- QEMU boot via `boot/` scripts

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

## Engineering Excellence Benchmarks

### Code Quality

| Domain | Benchmark |
|--------|-----------|
| **Test Coverage** | Target: >80% line coverage per crate (Foundation: >95%). 16 crates currently at 0% — active gap, not achieved. |
| **Doc Coverage** | Every public function, type, and module has rustdoc documentation. `cargo doc` generates complete API reference. |
| **Clippy** | Zero clippy warnings workspace-wide. `cargo clippy -- -D warnings` passes on every commit. |
| **DAG Health** | Zero layer violations. Service never depends on Foundation directly. DAG validation passes on every build. |
| **Build Time** | Full workspace check completes in under 5 minutes. Incremental builds under 30 seconds. |

### Tool & Signal Ecosystem

| Domain | Benchmark |
|--------|-----------|
| **MCP Tools** | MCP tools ship with typed parameters, forensic metadata, and >95% passing schema validation. Verify count: `nexcore_health_probe`. |
| **Signal Detection** | Signal detection algorithms (PRR, ROR, IC, EBGM) validated against FAERS gold-standard datasets with >90% concordance. |
| **FAERS Pipeline** | FAERS ETL processes quarterly data dumps with automatic signal detection across 20M+ reports. |

### Biological System

| Domain | Benchmark |
|--------|-----------|
| **Bio Crate Discipline** | All 8 biological crates operate as a cohesive organism: cytokine signaling, hormone config, immune defense, energy budgets, synaptic learning, schema inference, code generation, and adversarial testing — all active and measured. |
| **Safety Standard** | All 8 biological crates enforce `deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)`. This is the workspace safety gold standard. |

### Crate-Level Quality Bar

Every `crates/*/CLAUDE.md` file describes the aspirational state of that crate. When working in a crate and finding code below its CLAUDE.md standard, prioritize closing the gap over new feature work.

Per-crate minimum bar:
- Test coverage: target >80% (Foundation: >95%) — 16 crates at 0%, prioritize when touching
- Doc coverage: all public items
- Clippy: zero warnings
- No stubs: all documented types and functions implemented
- Grounding patterns: enforced in code, not just documented
