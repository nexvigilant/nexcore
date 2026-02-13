# NexVigilant Core (nexcore)

The Vigilance Kernel — unified Rust workspace (~63K LOC) powering NexVigilant's pharmacovigilance platform, skill orchestration, and AI agent infrastructure.

> **nexcore** is the proprietary computation kernel developed by [NexVigilant LLC](https://nexvigilant.com) for drug safety signal detection, regulatory compliance, and intelligent automation.

## Quick Start

```bash
# Build
cd ~/nexcore
cargo build --release

# Run tests (~1800+)
cargo test --workspace

# Start REST API (preferred for external integrations)
cargo install --path crates/nexcore-api
nexcore-api  # http://localhost:3030

# Use MCP tools (preferred for Claude Code)
# mcp__nexcore__pv_signal_complete(a=15, b=100, c=20, d=10000)
```

## Workspace Structure

```
~/nexcore/
├── Cargo.toml                    # Workspace root
└── crates/
    ├── nexcore-vigilance/        # Core: ToV axioms, Guardian-AV, PV signals, 25+ modules
    ├── nexcore-brain/            # Working memory: sessions, artifacts, code tracking
    ├── nexcore-mcp/              # MCP server (90+ tools for Claude Code)
    ├── nexcore-api/              # REST API server (Axum, 33 routes)
    ├── nexcore-friday/           # FRIDAY orchestrator (voice, webhooks, scheduler)
    ├── nexcore-hooks/            # Claude Code hooks (quality, security enforcement)
    ├── nexcore-faers-etl/        # FAERS data pipeline
    ├── nexcore-config/           # Configuration types
    ├── nexcore-skill-verify/     # Skill validation CLI
    └── nexcore-epa1/             # EPA ECHO API integration
```

## Workspace Dependency Direction

Use a one-way dependency flow to keep layering clear and avoid cycles:

1. `foundation` crates: shared primitives, utilities, types
2. `domain` crates: business logic, algorithms, PV, vigilance
3. `orchestration` crates: workflows, pipelines, schedulers
4. `service/bin` crates: API servers, CLIs, MCP, UI shells

Rules of thumb:
1. `service/bin` can depend on anything below it, but never the other way around.
2. `domain` should not depend on `service/bin`; keep boundaries clean.
3. `foundation` should be leaf-level: no dependencies on higher layers.

## Crate Overview

| Crate | Description | Key Features |
|-------|-------------|--------------|
| `nexcore-vigilance` | Core monolith | ToV axioms, Guardian-AV, PV signals (PRR/ROR/IC/EBGM), skills, harm taxonomy |
| `nexcore-brain` | Working memory | Sessions, artifacts with `.resolved.N` snapshots, implicit learning |
| `nexcore-mcp` | MCP server | 90+ tools exposed to Claude Code (foundation, PV, vigilance, skills, brain) |
| `nexcore-api` | REST API | 33 Axum routes, Scalar docs at `/docs`, OpenAPI at `/openapi.json` |
| `nexcore-friday` | Orchestrator | Voice integration, webhooks, scheduler, event bus, decision engine |
| `nexcore-hooks` | Cognitive hooks | Python blocker, secret scanner, clippy enforcer, panic-free guarantee |

## Access Methods (Priority Order)

### 1. MCP Tools (Preferred for Claude Code)

```
mcp__nexcore__pv_signal_complete(a=15, b=100, c=20, d=10000)
mcp__nexcore__foundation_levenshtein(source="kitten", target="sitting")
mcp__nexcore__brain_artifact_save(name="task.md", content="...")
mcp__nexcore__skill_validate(path="~/.claude/skills/my-skill")
```

### 2. REST API (External systems, webhooks, scripts)

```bash
nexcore-api                        # Start on http://localhost:3030
PORT=8080 nexcore-api              # Custom port

curl -X POST http://localhost:3030/api/v1/pv/signal/complete \
  -H "Content-Type: application/json" \
  -d '{"a": 15, "b": 100, "c": 20, "d": 10000}'
```

**Docs:** http://localhost:3030/docs (Scalar UI)

### 3. CLI (Deprecated)

```bash
nexcore verify <path>              # Validate Diamond v2 compliance
nexcore skill scan <path>          # Build skill registry
```

## Signal Detection Thresholds

| Metric | Evans (Default) | EMA GVP-IX | Strict | Sensitive |
|--------|-----------------|------------|--------|-----------|
| PRR | ≥ 2.0 | ≥ 2.0 | ≥ 3.0 | ≥ 1.5 |
| χ² | ≥ 3.841 | ≥ 3.841 | ≥ 6.635 | ≥ 2.706 |
| n | ≥ 3 | ≥ 3 | ≥ 5 | ≥ 2 |
| ROR lower CI | > 1.0 | > 1.0 | > 2.0 | > 1.0 |
| IC025 | > 0 | > 0 | > 1.0 | > -0.5 |
| EB05 | ≥ 2.0 | ≥ 2.0 | ≥ 3.0 | ≥ 1.5 |

## Performance (vs Python)

| Operation | Speedup | Notes |
|-----------|---------|-------|
| Levenshtein | 63x | SIMD-optimized |
| Fuzzy search | 39x | Parallel with rayon |
| SHA-256 | 20x | Hardware acceleration |
| YAML parse | 7x | Zero-copy parsing |
| Signal detection | 10x | PRR/ROR/IC/EBGM |

## Cognitive Hooks

Hooks auto-enforce code quality and security:

- **Python Blocker**: Enforces 100% Rust mandate
- **Secret Scanner**: Blocks commits with credentials
- **Clippy Enforcer**: Zero warnings policy
- **Panic-Free Guarantee**: Blocks unhandled `unwrap()`/`expect()`

**Protocol:** Exit 0 = allow, Exit 1 = warn, Exit 2 = block

**Config:** `~/.claude/settings.json`

## Development

```bash
# Run specific crate tests
cargo test -p nexcore-vigilance
cargo test -p nexcore-mcp -- test_foundation

# Verbose output
RUST_LOG=debug cargo test -p nexcore-mcp

# Lint (strict)
cargo clippy --workspace -- -D warnings

# Build release
cargo build --release --workspace
```

## License

Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
