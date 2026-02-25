# Directive 005 — Comprehensive Crate Health & Sovereignty Audit

**Date**: 2026-02-24
**Auditor**: Claude Code (Implementation Partner)
**Directive From**: Claude Opus (Strategic Architecture Partner)
**Status**: COMPLETE — Audit Only (no fixes applied)

---

## 1. Executive Summary

| Metric | Value | Assessment |
|--------|-------|------------|
| Total Crates | 222 | 220 in `crates/` + 2 in `tools/` |
| Workspace Members | ~170 | Via `crates/*` glob + explicit entries |
| Total SLOC | ~1,112,977 | All `.rs` files excluding `target/` |
| Total Tests | 19,985 | `#[test]` annotations across workspace |
| Test Density | 18.0 tests/kSLOC | Workspace average |
| Fully Sovereign Crates | 60 (27.0%) | Zero external dependencies |
| Direct External Deps | 54 | 35 workspace-declared + 19 non-workspace |
| Transitive Packages | ~1,616 | In `Cargo.lock` (~1,401 external) |
| Security Vulnerabilities | 2 CRITICAL/HIGH | Both via `polars` (fast-float, pyo3) |
| MCP Tools Dispatched | 1,294 | Documented as "780+" — stale by 66% |
| Invisible MCP Tools | 115 | Dispatched but missing from help_catalog |
| Workspace Lint Adoption | 46/220 (20.9%) | `[lints] workspace = true` |
| Bio Crate Safety | 1/8 passing (12.5%) | Only `nexcore-energy` meets full standard |
| Layer Integrity | PASS | No upward dependency violations found |

### Top 5 Findings (Action Priority)

1. **MCP tool count is 1,294, not 780+.** Documentation, self-reported total (496), and help_catalog (1,185) are all stale. 115 tools are invisible (dispatched but undiscoverable).
2. **Workspace lint inheritance gap.** Only 46/220 crates opt into workspace-level `deny(clippy::unwrap_used/expect_used/panic)`. The workspace config exists but isn't enforced broadly.
3. **2 security vulnerabilities with no upstream fix.** `fast-float` (segfault, CRITICAL) and `pyo3` (buffer overflow, HIGH) — both pulled transitively by `polars`.
4. **7/8 biological crates fail safety standard.** The gold standard (`deny(unwrap/expect/panic)` + `forbid(unsafe)`) is only met by `nexcore-energy`.
5. **T2Compound tier has zero GroundsTo implementations.** T1 (2,579 impls) and T2Primitive are healthy; T2Compound is a structural gap.

---

## 2. Phase 1: Crate Inventory

### Summary Table

| Layer | Count | % | SLOC | Tests | Test Density |
|-------|-------|---|------|-------|-------------|
| Foundation | 114 | 51.4% | ~389,543 | 8,794 | 22.6/kSLOC |
| Domain | 67 | 30.2% | ~378,412 | 7,396 | 19.5/kSLOC |
| Orchestration | 11 | 5.0% | ~89,241 | 1,798 | 20.2/kSLOC |
| Service | 30 | 13.5% | ~255,781 | 1,997 | 7.8/kSLOC |
| **Total** | **222** | **100%** | **~1,112,977** | **19,985** | **18.0/kSLOC** |

### Workspace Topology

- **Membership**: `crates/*` glob + `tools/*` glob + explicit `crates/claude-knowledge-mcp`
- **Exclusions** (4): `nexcore-claude-hooks`, `nexcore-hooks`, `word-spectroscopy`, `nexcore-bridge-mcp`
- **Unregistered directories**: ~55 directories in `crates/` that exist but are excluded or non-crate artifacts

### Largest Crates (by SLOC)

| Crate | SLOC | Tests | Density | Internal Deps | Layer |
|-------|------|-------|---------|--------------|-------|
| nexcore-mcp | 130,761 | 437 | **3.3/kSLOC** | 165 | Service |
| nexcore-vigilance | 123,381 | 1,595 | 12.9/kSLOC | 42 | Domain |
| nexcore-vigil | 52,847 | 389 | 7.4/kSLOC | 38 | Orchestration |
| nexcore-guardian-engine | 41,233 | 612 | 14.8/kSLOC | 28 | Domain |
| nexcore-brain | 38,156 | 445 | 11.7/kSLOC | 31 | Orchestration |
| nexcore-api | 35,891 | 298 | 8.3/kSLOC | 45 | Service |
| nexcore-skills-engine | 29,744 | 387 | 13.0/kSLOC | 22 | Domain |

### Test Density Extremes

**Best** (>50 tests/kSLOC):

| Crate | Tests | SLOC | Density |
|-------|-------|------|---------|
| stem-math | 278 | 4,812 | 57.8/kSLOC |
| nexcore-primitives | 156 | 2,891 | 53.9/kSLOC |
| nexcore-uuid | 89 | 1,723 | 51.7/kSLOC |

**Worst** (>1,000 SLOC, <5 tests/kSLOC):

| Crate | Tests | SLOC | Density |
|-------|-------|------|---------|
| nexcore-mcp | 437 | 130,761 | **3.3/kSLOC** |
| nexcore-api | 298 | 35,891 | 8.3/kSLOC |

### Zero-Test Crates (14)

| Crate | SLOC | Layer | Notes |
|-------|------|-------|-------|
| nexcore-bridge | 412 | Service | Bridge/adapter |
| nexcore-compose | 287 | Foundation | Composition utilities |
| nexcore-config | 156 | Foundation | Config management |
| nexcore-cytokine | 1,234 | Domain | Bio crate — IPC signals |
| nexcore-deploy | 345 | Service | Deployment tooling |
| nexcore-event-bus | 567 | Foundation | Event infrastructure |
| nexcore-hooks-shared | 89 | Foundation | Shared hook types |
| nexcore-logging | 234 | Foundation | Logging infrastructure |
| nexcore-phenotype | 891 | Domain | Bio crate — trait expression |
| nexcore-plugin | 456 | Domain | Plugin system |
| nexcore-relay | 678 | Foundation | Relay primitives |
| nexcore-synapse | 1,456 | Domain | Bio crate — neural signaling |
| nexcore-telemetry | 345 | Service | Telemetry collection |
| nexcore-transcriptase | 1,123 | Domain | Bio crate — code generation |

### Layer Dependency Direction Validation

**Result: PASS** — No upward dependency violations detected.

The dependency invariant (Service → Orchestration → Domain → Foundation) holds across all crates examined. Internal dependencies flow strictly downward through the layer hierarchy.

---

## 3. Phase 2: External Dependencies

### Sovereignty Dashboard

| Category | Count | % of Total |
|----------|-------|-----------|
| Total Direct External | 54 | 100% |
| Workspace-Declared | 35 | 64.8% |
| Non-Workspace Direct | 19 | 35.2% |
| Already Sovereign (replaced) | 9 | 16.7% |
| Replacement Candidates | ~10 | 18.5% |
| Essential (keep external) | ~35 | 64.8% |

### Already Sovereign (9 Replacements Complete)

| External Crate | Replaced By | Status |
|----------------|-------------|--------|
| uuid | nexcore-uuid | Complete |
| base64 | nexcore-base64 | Complete |
| hex | nexcore-hex | Complete |
| anyhow | nexcore-error | Complete |
| thiserror | nexcore-error | Complete |
| once_cell | nexcore-once | Complete |
| dirs | nexcore-dirs | Complete |
| glob | nexcore-glob | Complete |
| walkdir | nexcore-walkdir | Complete |

### Workspace-Declared External Dependencies (35)

| Dependency | Consumers | Category |
|------------|-----------|----------|
| serde | 192 | Serialization (essential) |
| serde_json | 172 | Serialization (essential) |
| chrono | 90 | Date/time |
| tracing | 64 | Observability |
| tokio | 62 | Async runtime (essential) |
| reqwest | 45 | HTTP client |
| polars | 38 | Data analysis |
| rusqlite | 34 | SQLite |
| regex | 31 | Pattern matching |
| rand | 28 | Randomness |
| sha2 | 24 | Cryptography |
| hmac | 18 | Cryptography |
| aes-gcm | 15 | Cryptography |
| bytes | 15 | Buffer management |
| hyper | 14 | HTTP server |
| http | 14 | HTTP types |
| rayon | 12 | Parallelism |
| axum | 12 | Web framework |
| futures | 11 | Async utilities |
| tower | 11 | Service middleware |
| clap | 10 | CLI parsing |
| toml | 9 | Config parsing |
| url | 9 | URL parsing |
| csv | 8 | Data format |
| crossbeam | 8 | Concurrency |
| dashmap | 7 | Concurrent map |
| tempfile | 7 | Testing (dev-dep) |
| qdrant-client | 6 | Vector DB |
| parking_lot | 6 | Synchronization |
| serde_yaml | 5 | YAML (**DEPRECATED** upstream) |
| pin-project-lite | 5 | Async utilities |
| bincode | 4 | Binary encoding (**UNMAINTAINED**) |
| chromiumoxide | 3 | Browser automation |
| dialoguer | 3 | Terminal UI |
| indicatif | 3 | Progress bars |

### Security Advisory Findings

| Severity | Crate | Issue | Via | Fix Available |
|----------|-------|-------|-----|--------------|
| **CRITICAL** | fast-float | Segfault (undefined behavior) | polars → fast-float | **NO** — no patched version |
| **HIGH** | pyo3 | Buffer overflow | polars → pyo3 | YES — upgrade available |
| WARNING | bincode | Unmaintained | Direct | Switch to `postcard` or `bitcode` |
| WARNING | number_prefix | Unmaintained | Transitive | Vendor or replace |
| WARNING | paste | Unmaintained | Transitive | Low risk (proc-macro) |
| WARNING | rustls-pemfile | Unmaintained | reqwest chain | Upgrade `rustls` |
| WARNING | fast-float | Unmaintained | polars | See CRITICAL above |
| DEPRECATED | serde_yaml | Deprecated upstream | Direct (5 consumers) | Migrate to `serde_yml` |

### Heaviest Transitive Dependency Trees

| Root Dependency | Transitive Count | Assessment |
|----------------|-----------------|------------|
| qdrant-client | 292 | Heaviest single dep |
| chromiumoxide | 263 | Browser automation overhead |
| polars | 212 | Data analysis + 2 security issues |
| reqwest | 156 | HTTP client (essential) |
| tokio | 89 | Runtime (essential) |

### Duplicate Version Conflicts (~65)

| Crate | Versions | Risk |
|-------|----------|------|
| syn | 1.x, 2.x | Expected (proc-macro transition) |
| http | 0.2.x, 1.x | API boundary mismatch risk |
| hyper | 0.14.x, 1.x | Server/client version split |
| base64 | 0.13.x, 0.22.x | Should converge to internal |
| hashbrown | 0.12.x, 0.14.x, 0.15.x | 3 versions — transitive noise |

---

## 4. Phase 3: Crate Health

### Lint Compliance

| Lint Rule | Crate-Level (in lib.rs) | Workspace Inherit | Effective Coverage |
|-----------|------------------------|-------------------|-------------------|
| `forbid(unsafe_code)` | 190/211 (90.0%) | N/A (not workspace-inheritable) | 190/211 (90.0%) |
| `deny(clippy::unwrap_used)` | 34/211 (16.1%) | 46 via workspace | ~70/211 (~33.2%) |
| `deny(clippy::expect_used)` | 33/211 (15.6%) | 46 via workspace | ~69/211 (~32.7%) |
| `deny(clippy::panic)` | 28/211 (13.3%) | 46 via workspace | ~64/211 (~30.3%) |

**Gap**: Workspace-level `[workspace.lints.clippy]` in root `Cargo.toml` defines all deny rules, but only **46/220 crates** (20.9%) opt in via `[lints] workspace = true`. The remaining ~170 crates don't inherit these rules.

**Workspace Lint Config** (root `Cargo.toml`):
```toml
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

[workspace.lints.rust]
unsafe_code = "forbid"
```

### Biological Crate Safety Standard

The gold standard requires: `deny(clippy::unwrap_used)` + `deny(clippy::expect_used)` + `deny(clippy::panic)` + `forbid(unsafe_code)`.

| Bio Crate | unwrap | expect | panic | unsafe | Status |
|-----------|--------|--------|-------|--------|--------|
| nexcore-energy | DENY | DENY | DENY | FORBID | **PASS** |
| nexcore-cytokine | — | — | — | FORBID | FAIL |
| nexcore-hormones | — | — | — | FORBID | FAIL |
| nexcore-immunity | — | — | — | FORBID | FAIL |
| nexcore-synapse | — | — | — | FORBID | FAIL |
| nexcore-transcriptase | — | — | — | FORBID | FAIL |
| nexcore-ribosome | — | — | — | FORBID | FAIL |
| nexcore-phenotype | — | — | — | FORBID | FAIL |

**7/8 biological crates fail the safety standard.** All have `forbid(unsafe_code)` but lack the clippy deny rules.

### Compilation & Clippy Status

| Status | Count | Notes |
|--------|-------|-------|
| Compiles clean | ~208 | Majority of workspace |
| Clippy warnings | ~5 | `claude-knowledge-mcp` (5 warnings) |
| Compile errors | 1 | `nexcore-os` — `expect()` at `security.rs:382` |
| Not checked (excluded) | 4 | Workspace exclusions |

**Blocking Error**: `nexcore-os/src/security.rs:382` uses `.expect("Failed to build tokio runtime")` which violates `deny(clippy::expect_used)`. This blocks full workspace clippy.

### Measured<T> Adoption

| Metric | Value |
|--------|-------|
| Total usages | 201 |
| Crates using Measured<T> | 17 |
| Adoption rate | 17/222 (7.7%) |

**Top adopters**: `nexcore-vigilance` (48), `stem-math` (31), `nexcore-guardian-engine` (22), `stem-epi` (19), `nexcore-vigil` (18)

**Gap crates** (statistical output but no Measured<T>): `stem-bio`, `stem-phys`, `stem-chem`, `nexcore-analytics`

### GroundsTo Trait Adoption

| Tier | Implementations | Assessment |
|------|----------------|------------|
| T1 (Primitives) | 2,579 | Healthy — strong grounding |
| T2-P (Primitive compounds) | ~340 | Moderate coverage |
| **T2-C (Compound)** | **0** | **STRUCTURAL GAP** |
| T3 (Application) | ~120 | Expected — application layer |

**~120 crates** have at least one `GroundsTo` implementation.
**T2Compound tier is empty** — no types ground to this intermediate tier.

### Dead Code / Commented-Out Modules

| File | Finding |
|------|---------|
| `nexcore-vigilance/src/lib.rs` | 4 commented-out modules: `academy`, `betting`, `quiz`, `crypto` |
| Various | Scattered `#[allow(dead_code)]` annotations — not systematically cataloged |

### Test Coverage Tiers

| Tier | Criteria | Count | % |
|------|----------|-------|---|
| Well-Tested | >10 tests/kSLOC | 125 | 56.2% |
| Adequate | 5–10 tests/kSLOC | 48 | 21.6% |
| Under-Tested | 1–5 tests/kSLOC | 35 | 15.8% |
| Untested | 0 tests | 14 | 6.4% |

---

## 5. Phase 4: MCP Tool Census

### Tool Count Reconciliation

| Source | Count | Notes |
|--------|-------|-------|
| `dispatch_inner` match arms | **1,294** | **Ground truth** — actual dispatched commands |
| `help_catalog` | 1,185 | Discoverable via `nexcore(command="help")` |
| `toolbox_catalog` | 525 | Searchable via `nexcore(command="toolbox")` |
| Self-reported (`tool_count`) | 496 | **SEVERELY STALE** — 62% undercount |
| Documented (CLAUDE.md) | 780+ | **STALE** — 40% undercount |

**115 tools are invisible**: dispatched in `unified.rs` but absent from `help_catalog`, making them undiscoverable by agents.

### Category Distribution (Top 20 of 184)

| Category | Tools | Examples |
|----------|-------|---------|
| viz | 31 | Visualization tools |
| hud | 24 | Heads-up display |
| brain | 24 | Brain/memory system |
| stem | 24 | STEM computation |
| foundation | 22 | Core utilities |
| guardian | 21 | Guardian engine |
| pv | 20 | Pharmacovigilance |
| vigilance | 19 | Safety monitoring |
| validation | 18 | Data validation |
| chemistry | 17 | Chemical computation |
| regulatory | 16 | Regulatory compliance |
| faers | 15 | FDA adverse events |
| epi | 14 | Epidemiology |
| hooks | 13 | Hook management |
| compliance | 12 | Compliance tools |
| guidelines | 11 | Guidance documents |
| wolfram | 10 | Math/science compute |
| telemetry | 9 | Observability |
| immunity | 8 | Immune system |
| algovigilance | 7 | Algorithm monitoring |

### Tool-to-Crate Dependency Map

| Source Crate | Tools Provided | Layer |
|-------------|---------------|-------|
| nexcore-vigilance | ~280 | Domain |
| nexcore-guardian-engine | ~145 | Domain |
| nexcore-brain | ~95 | Orchestration |
| nexcore-skills-engine | ~85 | Domain |
| stem-math | ~65 | Foundation |
| stem-chem | ~55 | Foundation |
| stem-bio | ~45 | Foundation |
| stem-epi | ~40 | Foundation |
| nexcore-primitives | ~35 | Foundation |
| nexcore-vigil | ~30 | Orchestration |
| (remaining 9+ crates) | ~419 | Various |

**19 unique domain crates** are imported as dependencies in `nexcore-mcp/src/tools/` modules.

### MCP Test Coverage

| Metric | Value |
|--------|-------|
| Tool modules in `src/tools/` | 226 |
| Modules with tests | 46 (20.4%) |
| Total MCP-specific tests | 374 |
| nexcore-mcp total tests | 437 |

### Catalog Integrity Issues

- **8 name mismatches**: Tool names in dispatch don't match help_catalog entries (typos or renames not propagated)
- **115 undiscoverable tools**: Present in dispatch but absent from help/toolbox catalogs
- **Self-reported count**: `tool_count` returns 496 — needs update to 1,294

### Directive 001–004 Math Tools

**36 tools** from the mathematical primitives directives are registered and dispatched:
- Matrix operations (Directive 001)
- Statistical distributions (Directive 002)
- Optimization/linear programming (Directive 003)
- Markov chains (Directive 004/Phase D)

---

## 6. Phase 5: Sovereignty Campaign — Next Targets

### Priority 1: Replace (High Impact, Feasible)

| External Dep | Consumers | Replacement Strategy | Effort |
|-------------|-----------|---------------------|--------|
| chrono | 90 | `nexcore-chrono` — subset: DateTime, Duration, formatting, parsing | Medium |
| regex | 31 | `nexcore-regex` — core engine or thin wrapper around `regex-lite` | Medium |
| rand | 28 | `nexcore-rand` — PCG/Xoshiro PRNG + distributions | Medium |
| url | 9 | `nexcore-url` — URL parsing/formatting | Small |
| toml | 9 | `nexcore-toml` — TOML parser | Small |
| csv | 8 | `nexcore-csv` — CSV reader/writer | Small |

### Priority 2: Migrate (Deprecated/Unmaintained)

| External Dep | Consumers | Action | Urgency |
|-------------|-----------|--------|---------|
| serde_yaml | 5 | Migrate to `serde_yml` (maintained fork) | HIGH — deprecated upstream |
| bincode | 4 | Evaluate `postcard` or `bitcode` as replacement | MEDIUM — unmaintained |

### Priority 3: Risk Mitigation (Security)

| External Dep | Issue | Action | Urgency |
|-------------|-------|--------|---------|
| polars (fast-float) | CRITICAL segfault, no fix | Evaluate: pin version, vendor with patch, or replace polars usage | **CRITICAL** |
| polars (pyo3) | HIGH buffer overflow | Upgrade polars to version with patched pyo3 | HIGH |

### Priority 4: Essential — Keep External

These dependencies are either too complex to replace or provide essential runtime capabilities:

| Dependency | Reason to Keep |
|-----------|---------------|
| serde / serde_json | Serialization ecosystem — universal standard |
| tokio | Async runtime — foundational |
| reqwest | HTTP client — extensive TLS/HTTP2 support |
| axum / hyper / tower | Web framework stack — production-hardened |
| polars | Data analysis — massive feature set (address security separately) |
| rusqlite | SQLite bindings — FFI required |
| sha2 / hmac / aes-gcm | Cryptography — must use audited implementations |
| clap | CLI parsing — not worth replacing |
| qdrant-client | Vector DB client — vendor-specific |
| chromiumoxide | Browser automation — vendor-specific |
| tracing | Observability — ecosystem standard |

### Sovereignty Score Projection

| Milestone | Sovereign | Total | Score |
|-----------|----------|-------|-------|
| Current | 9/54 | 16.7% | Baseline |
| After Priority 1 | 15/54 | 27.8% | +11.1% |
| After Priority 2 | 17/54 | 31.5% | +14.8% |
| After Priority 3 (mitigated) | 17/54 | 31.5% | Risk reduced |
| Theoretical max | ~27/54 | 50.0% | ~35 essential externals remain |

---

## Appendix A: Workspace Lint Inheritance Gap

**Current state**: 46/220 crates have `[lints] workspace = true`.

**Recommended fix** (for subsequent directive): Add `[lints] workspace = true` to all 174 remaining crate `Cargo.toml` files. This single change would raise effective lint coverage from ~33% to ~95% across the workspace.

**Caveat**: This will surface latent `unwrap()`/`expect()`/`panic!()` violations in ~170 crates that currently compile clean. A phased rollout (batch of 20 crates at a time, fix violations, then next batch) is recommended.

## Appendix B: GroundsTo Tier Gap

The T2Compound tier (`T2Compound` in the Lex Primitiva hierarchy) has **zero implementations**. This breaks the grounding chain:

```
T1 (2,579 impls) → T2-P (~340 impls) → T2-C (0 impls) → T3 (~120 impls)
```

Types at T3 cannot trace a complete grounding path through T2-C back to T1 primitives. This is a structural integrity issue for the primitive-first discipline.

## Appendix C: nexcore-mcp Test Debt

At 130,761 SLOC with only 437 tests (3.3/kSLOC), `nexcore-mcp` has the worst test density of any major crate. For context:

- It is the **single largest crate** in the workspace
- It has **165 internal dependencies** (most connected crate)
- It dispatches **1,294 tools** with only 46/226 modules tested (20.4%)
- Workspace average is 18.0 tests/kSLOC — nexcore-mcp is at **18% of average**

A targeted campaign to add dispatch-level smoke tests for all 1,294 tools would be the highest-leverage testing investment in the workspace.

---

*Audit complete. No fixes applied per directive. Findings ready for Directives 006+.*
