# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Primary instructions:** See `~/CLAUDE.md` (root) for structural architecture. This file is the operational reasoning substrate — validated learnings, crate details, PVOS layers, and gotchas that prevent repeat mistakes.

## How This File Grounds Reasoning

Every validated learning below was extracted from a real development session where something broke. Future instances inherit these as ∂ Boundary primitives — constraints that prevent re-discovery of known failure modes. When uncertain about a pattern, check here first.

## Quick Development Commands

```bash
# Using just (preferred — see justfile for all recipes)
just                    # default: check + clippy
just test-core          # 3300+ vigilance tests (fastest feedback)
just test-fast          # Full workspace via nextest (parallel)
just validate           # Full CI: fmt → clippy → test
just validate-quick     # Quick: check → clippy → core tests
just test-crate <name>  # Single crate test
just browser-run        # Launch NexBrowser (DISPLAY=:0)
just hooks-build        # Build hook binaries
just waste-audit        # Show disk usage across all systems
just --list             # Show all recipes
```

## Access Methods

**MCP (preferred for Claude Code):**
```
mcp__nexcore__pv_signal_complete(a=15, b=100, c=20, d=10000)
mcp__nexcore__foundation_levenshtein(source="kitten", target="sitting")
mcp__nexcore__skill_list()
```

**REST:** `curl -X POST localhost:3030/api/v1/pv/signal/complete -d '{"a":15,"b":100,"c":20,"d":10000}'`

## MCP Servers

| Server | Binary | Purpose |
|--------|--------|---------|
| `nexcore` | `nexcore-mcp` | Primary: 260+ tools (PV, skills, brain, lex primitiva, foundation) |
| `claude-fs` | `claude-fs-mcp` | Filesystem ops on `~/.claude/` + session backup |
| `compendious` | `compendious-machine` | Text compression scoring (Cs = I/E × C × R) |

Additional: `nexcore-docs-mcp`, `gsheets-mcp` (Sheets API v4), `reddit-mcp`.

## Workspace Members Beyond `crates/*`

Explicit members: `nexcore-cli/`, `claude-repl-mcp`, `claude-fs-mcp`, `mcp-stdio-client`, `claude-mcp-config`, `codex-mcp-config`, `stem-derive-core`, `skill-macros-core`, `nexcore-macros-core`.

## PVOS: 15-Layer Pharmacovigilance Operating System

Standalone `nexcore-pvos` crate (82 `.rs` files, 424 GroundsTo impls). The `nexcore-vigilance/src/pvos/mod.rs` facade re-exports via `pub use nexcore_pvos::*`. Each layer has a unique dominant T1 primitive (100% Quindecet coverage).

| Layer | Prefix | Dominant | Purpose |
|-------|--------|----------|---------|
| AVC | `avc_*` | κ Comparison | Adverse event classification |
| PVSC | `stream_*` | μ Mapping | Signal collection streams |
| PVSP | `pipeline_*` | σ Sequence | Signal processing pipelines |
| PVSD | `detection_*` | ∂ Boundary | Signal detection thresholds |
| PVTF | `temporal_*` | ν Frequency | Temporal frequency analysis |
| PVRC | `recursive_*` | ρ Recursion | Recursive case analysis |
| PVGL | `geo_*` | λ Location | Geographic localization |
| PVAG | `aggregate_*` | Σ Sum | Multi-source aggregation |
| PVIR | `irreversibility_*` | ∝ Irreversibility | Outcome irreversibility scoring |
| PVNL | `null_*` | ∅ Void | Null/missing data handling |
| PVCL | `causal_*` | → Causality | Causal inference (Bradford Hill) |
| PVST | `state_*` | ς State | Case lifecycle state machines |
| PVPS | `persist_*` | π Persistence | Signal persistence & storage |
| PVEX | `existence_*` | ∃ Existence | Signal existence validation |
| PVNM | `quantity/units/arithmetic/range/statistics` | N Quantity | Numeric measurement |

**Adding a PVOS module:** Create files under `crates/nexcore-pvos/src/`, add `pub mod` in the crate's `lib.rs`, ensure T3 engine type has `GroundsTo` impl. The vigilance facade re-exports automatically via `pub use nexcore_pvos::*`.

## Signal Pipeline

The `signal` crate is a **single monolithic crate** (not 12 separate crates) with internal modules:

`core` → `ingest` → `normalize` → `validate` → `detect` → `threshold` → `stats` → `store` → `alert` → `report` → `api` → `orchestrate`

## KSB Framework

1,462 KSBs across 15 PV domains (D01-D15) in `nexcore-vigilance/src/primitives/ksb.rs`. Knowledge (π, 630), Skill (σ, 344), Behavior (ς, 255), AI Integration (μ, 233). Cross-domain universals: ∂ in 11/15 domains, σ in 9/15, μ in 8/15.

## Primitives Architecture

**`nexcore-primitives`** (standalone, re-exported by vigilance): `chemistry` (ThresholdGate, SaturationKinetics, 8 types), `quantum` (Amplitude, Phase, Superposition, 13 types), `transfer` (Homeostasis, CircuitBreaker, 13 types), `bathroom_lock` (ς+∂), `measurement` (Confidence, Measured<T>).

**Quantum primitives** use `impl_float_total_ord!` for zero-dep Eq/Hash/Ord on f64 via `to_bits()`/`total_cmp()`.

## Bi-Cameral Cloud Architecture

**Body** (`nexvigilant-digital-clubhouse`): `guardian-gateway` (SENSE), `guardian-analytics-service` (COMPARE), `guardian-epa*` (ACT).
**Mind** (`pv-education-machine-prod`): `pvdsl-signal-api-correct`, `pvdsl-orchestrator`, `content-orchestrator`.
**Bridge:** `guardian-gateway` → `pvdsl-signal-api-correct` (Service-to-Service IAM).

```bash
gcloud run deploy guardian-epa[N] --source crates/guardian/guardian-epa[N]
```

## Prima Language Integration

**Prima crates** are workspace members under `crates/prima*` (7 crates, 556+ tests). Prima's standalone `lex-primitiva` was merged into `nexcore-lex-primitiva` (16 primitives, adds Product/×). Source uses `lex-primitiva = { workspace = true }` which resolves to `nexcore-lex-primitiva` via package alias.

| Crate | Purpose |
|-------|---------|
| `prima` | Core language: lexer, parser, VM, REPL (484 tests) |
| `prima-codegen` | Universal code generator: Rust/TS/Python/Go/C (72 tests) |
| `prima-chem` | Molecular primitives (SMILES, 3D geometry) |
| `prima-academy` | Academic course classification |
| `prima-mcp` | Prima-to-MCP compiler |
| `prima-mcp-server` | Standalone MCP server for Prima functions |
| `prima-pipeline` | Universal concept translator |

**Consumers:** `nexcore-mcp` (prima + prima-codegen), `nexcore-renderer` (prima).

**Docs:** `~/nexcore/docs/prima/` — README, OPERATING_PROCEDURE, LEXICON, DAG proofs.

**Brain Knowledge Repository**: Prima documentation compiled to Brain session `edfa355a-163c-470d-983c-e69ba45bbff1` (Git: e598be2):
- `prima-core-knowledge.md` — 16 Lex Primitiva, tier system, homoiconicity
- `prima-operating-procedure.md` — MINE→GENERATE→VALIDATE→REFINE loop
- `lex-primitiva-dag-proof.md` — Mathematical DAG foundation (dual-root: N, →)

## Skill Scripts

94 skills in `~/nexcore/skills/`, each with a `scripts/` directory containing bash automation.

**Shared library:** `skills/_shared/script-lib.sh` — `pass()`, `fail()`, `warn()`, `header()`, `phase()`, `summary()`, `run_cargo_test()`, `count_pattern()`.

**Conventions:**
- All scripts: `#!/usr/bin/env bash` + `set -euo pipefail` + source `script-lib.sh`
- 100% bash — no Python/Ruby/Node calls
- Single-quote grep patterns with backticks to avoid command substitution
- Exit 0=pass, 1=fail (via `summary` function)

**STEM crate names:** `stem-bio`, `stem-phys`, `stem-chem`, `stem-math` (not stem-biology/stem-physics).

**Adding a skill script:** `mkdir -p skills/<name>/scripts/`, write script sourcing `../../_shared/script-lib.sh`, `chmod +x`.

## Cognitive Hooks

Hooks in `~/.claude/hooks/` (separate Cargo workspace, 35 members). Exit 0=allow, 1=warn, 2=block.

- **Python Blocker**: 100% Rust mandate
- **Secret Scanner**: Blocks credentials in commits
- **Unwrap Guardian**: Blocks `unwrap()`/`expect()` in production
- **Complexity Gate**: Nesting > 5 levels blocked (`tokio::select!` counts as nesting)
- **Incremental Verifier**: After ~15 edits, requires `cargo check`
- **Timeout Enforcer**: Bash commands capped at 120s
- **Compound Growth Miner**: Stop hook — saves mining report to Brain

Hook infra crates (in NexCore workspace): `nexcore-hook-lib`, `nexcore-hook-metrics`, `nexcore-claude-hooks`.

## Validated Learnings (Session-Extracted)

### Grammar-State Algebra (Grand Equation)
Grammar IS math. Lex Primitiva has 5 generators {σ,Σ,ρ,κ,∃} producing all 16 symbols. Chomsky hierarchy = filtration: Type-3=⟨σ,Σ⟩, Type-2=+ρ, Type-1=+κ, Type-0=+∃. Adding one generator = one Chomsky level = one qualitative capability. Use this to select minimal architecture: count generators needed → choose automaton class. Overengineering = |generators_used| - |generators_needed|. STEM traits are grammar operators (Transit↔σ, Classify↔Σ, Associate↔ρ, Membership↔κ, Normalize↔∃). Proved by `nexcore-grammar-lab` (35 tests). Full theory: `crates/nexcore-grammar-lab/THEORY.md`.

### StateMode (ς Disambiguation)
State (ς) conflated 3 algebraic objects (F2 equivocation fallacy). `StateMode` in `nexcore-lex-primitiva/src/state_mode.rs` resolves this:

| Mode | Symbol | Reversible | Example |
|------|--------|------------|---------|
| `Mutable` | `ς-mut` | Yes | `String`, `Vec<T>`, `HashMap`, `Mutex<T>` |
| `Modal` | `ς-mod` | Constrained | `CircuitBreaker`, `enum Phase { A, B }` |
| `Accumulated` | `ς-acc` | No | `Vec<Event>` (append-only), audit trails |

**Key insight (F4):** Rust has no pure T1-ς type — `mut` is a binding annotation, not a type. Every mutable type adds ∂ (Boundary).

**API:** `GroundsTo::state_mode() -> Option<StateMode>` (default `None`, 1,705 impls unchanged). `PrimitiveComposition::state_mode: Option<StateMode>` with `#[serde(default, skip_serializing_if)]` for backward compat.

**MCP:** `mcp__nexcore__lex_primitiva_state_mode(type_name="String")` returns mode, symbol, reversibility.

**Annotate when:** Type has State (ς) in its composition AND clear mutation semantics. Currently ~170 types annotated across 19 crates (9 stdlib + ~161 domain types: pv-core, signal, guardian-engine, brain, energy, algovigilance, primitives, constants, cloud, clearance, compliance, measure, cortex, hormones, pvdsl, aggregate, cytokine, vigilance).

**Theory:** Brain session `8e3987ea` — 4 artifacts (theory, Rust manifestations, grounding inventory, fallacy audit). Full crate: `nexcore-state-theory` (4,577 LOC, 10 theorems).

### Crate Creation Sequence (CRITICAL)
Write ALL source files BEFORE updating `lib.rs` module declarations. `cargo fmt` strips `mod` declarations when the referenced file doesn't exist. Sequence: create files → add `pub mod` → run `cargo check`.

### Forward-Reference Pattern
Define functions BEFORE their first call site in the same file. The compile-verifier hook catches this but costs a round-trip.

### wgpu 27 API Breaking Changes
- `wgpu::Maintain::Wait` → `wgpu::PollType::Wait { submission_index: None, timeout: None }`
- `TextureUsages` must include `COPY_SRC` for GPU readback

### tokenizers Crate
MUST keep default features — stripping them breaks SysRegex/onig backend.

### hf-hub Sync Downloads
Use `hf-hub` with `features=[ureq]` for sync downloads in non-async crates. Avoids tokio dep conflict.

### rmcp Request Names
`request.name.as_ref()` is ambiguous — use `request.name.to_string()`.

### map_err Closures
Closures crossing error types need explicit annotation: `|e: Box<dyn Error + Send + Sync>|`

### HealthCheck Threshold Semantics
`HealthCheck::new(name, threshold)` where `is_healthy() = failures < threshold`. Threshold=1 = single failure flips unhealthy. Threshold=3 = up to 2 failures tolerated.

## Known Gotchas

- `nexcore-vigilance/src/pv/landscape.rs:98,106` — pre-existing `unwrap()` on `partial_cmp` (not ours)
- Temporal-safe testing: Never parse log lines for time-dependent records — use builder structs with `Utc::now()` offsets
- `IpAddr::parse()` expects bare IPs — use `parse_ip_or_cidr()` for iptables output
- `Path::new("test").parent()` returns `Some("")` not `None` — handle empty parent
- `#[instrument]` on async fns can cause `PhantomNotSend` — use `tracing::Instrument` trait
- **Prima crates** are workspace members (`crates/prima*`). Prima's `lex-primitiva` merged into `nexcore-lex-primitiva` (16 primitives). `nexcore-mcp` and `nexcore-renderer` depend on `prima` via workspace refs.
- `nexcore-foundation` is a **package alias** for `nexcore-vigilance` (root `Cargo.toml:146`). Do not create a `crates/nexcore-foundation/` directory.
- **Flaky test:** `routes::guardian::tests::test_pause_resume_loop` — skip with `--skip test_pause_resume_loop` or use nextest retry

## Test Configuration

- **nextest config:** `.config/nextest.toml` — parallel, flaky retry, slow-test thresholds
- **Profiles:** `default` (retries=1, fail-fast=false), `ci` (retries=0, fail-fast=true)

## Debug Infrastructure

- **NexBrowser debug server:** Port 9333 — `/screenshot.png` (raw PNG), `/` (HTML auto-refresh 3s), `/health` (JSON)
- **Frame capture:** Vello → target_texture (COPY_SRC) → staging buffer → PNG via `image` crate
