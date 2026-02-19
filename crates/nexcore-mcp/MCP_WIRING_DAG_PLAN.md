# MCP Wiring DAG Plan (2026-02-17)

## Goal

Close all actionable `nexcore-mcp` wiring gaps (`domain-gap` crates) with validated, incremental patches.

## Current Baseline

- Set A (workspace crates): 185
- Set B (`nexcore-mcp` internal deps): 83
- Set C (`src/tools/*.rs`): 163
- Wired coverage: 44.9%
- Domain-gap crates: 40

## Acceptance Gates Per Wave

1. `cargo check -p nexcore-mcp` passes.
2. Wiring audit snapshot count improves or remains stable with explicit rationale.
3. New wiring is either:
   - Directly used by a tool, or
   - Added with a scheduled follow-up task in the next wave.

## DAG Overview

```text
N0 Baseline audit snapshot
  -> N1 Mismatch closure (ctvp/pvdsl/pv_axioms)
  -> N2 PV/ToV domain core wiring
  -> N3 Platform/orchestration/security wiring
  -> N4 Prima pipeline wiring
  -> N5 Ancillary domain wiring
  -> N6 Final audit + cleanup
```

## Wave Plan

### Wave 1 (N1): High-Signal Mismatch Closure

Scope:
- `nexcore-ctvp`
- `nexcore-pvdsl`
- `nexcore-pvos`

Tool alignment:
- `src/tools/ctvp.rs` -> `nexcore-ctvp` (wire now, deep integration follow-up)
- `src/tools/pvdsl.rs` -> switch to `nexcore-pvdsl`
- `src/tools/pv_axioms.rs` -> add explicit `nexcore-pvos` boundary touchpoint

### Wave 2 (N2): PV/ToV Core

Scope:
- `nexcore-pharmacovigilance`
- `nexcore-preemptive-pv`
- `nexcore-harm-taxonomy`
- `nexcore-tov-grounded`
- `nexcore-tov-proofs`
- `pvos-primitive-expansion`

### Wave 3 (N3): Platform, Orchestration, Security

Scope:
- `nexcore-orchestration`
- `nexcore-vault`
- `nexcore-statemind`
- `nexcore-model-checker`
- `nexcore-combinatorics`
- `nexcore-api`
- `nexcore-cloud`
- `nexcloud`
- `mcp-relay`

### Wave 4 (N4): Prima and Language Pipeline

Scope:
- `nexcore-prima`
- `prima-chem`
- `prima-pipeline`
- `prima-academy`
- `prima-mcp-server`
- `nexcore-word`
- `nexcore-grammar-lab`
- `primitive-innovation`

### Wave 5 (N5): Remaining Domain Crates

Scope:
- `core-true`
- `nexcore-antibodies`
- `nexcore-audio`
- `nexcore-compilation-space`
- `nexcore-compositor`
- `nexcore-downloads-scanner`
- `nexcore-engram`
- `nexcore-ghost`
- `nexcore-jeopardy`
- `nexcore-mcp-hot`
- `nexcore-nvrepl`
- `nexcore-pharma-rd`
- `nexcore-softrender`
- `nexcore-state-theory`

### Wave 6 (N6): Finalization

Scope:
- Re-run full audit snapshot.
- Confirm `domain-gap` count progression.
- Identify any crates still intentionally deferred and mark with rationale.

## Risk Controls

- Do not wire all 40 crates in one commit.
- Keep each wave small enough to diagnose compile regressions.
- Prioritize crates with existing tool files first.

## Execution Log

- 2026-02-17: Plan created.
- 2026-02-17 (N1 complete): wired `nexcore-ctvp`, `nexcore-pvdsl`, `nexcore-pvos`; switched `pvdsl.rs` to `nexcore_pvdsl`; added PVOS marker in `pv_axioms.rs`. Snapshot: Set B 86, coverage 46.5%, domain-gap 37.
- 2026-02-17 (N2 complete): wired PV/ToV core batch (`nexcore-pharmacovigilance`, `nexcore-preemptive-pv`, `nexcore-harm-taxonomy`, `nexcore-tov-grounded`, `nexcore-tov-proofs`, `pvos-primitive-expansion`). Snapshot: Set B 92, coverage 49.7%, domain-gap 31.
- 2026-02-17 (N3 complete): wired platform/orchestration/security batch except `nexcore-api` (initial compile blocker). Snapshot: Set B 100, coverage 54.1%, domain-gap 23.
- 2026-02-17 (N4 complete): wired Prima/language batch (`nexcore-prima`, `prima-chem`, `prima-pipeline`, `prima-academy`, `prima-mcp-server`, `nexcore-word`, `nexcore-grammar-lab`, `primitive-innovation`). Snapshot: Set B 108, coverage 58.4%, domain-gap 15.
- 2026-02-17 (N5 complete): wired remaining domain crates (except `nexcore-api`) including `core-true`, `nexcore-antibodies`, `nexcore-audio`, `nexcore-compilation-space`, `nexcore-compositor`, `nexcore-downloads-scanner`, `nexcore-engram`, `nexcore-ghost`, `nexcore-jeopardy`, `nexcore-mcp-hot`, `nexcore-nvrepl`, `nexcore-pharma-rd`, `nexcore-softrender`, `nexcore-state-theory`. Snapshot: Set B 122, coverage 65.9%, domain-gap 1.
- 2026-02-17 (N6 complete): patched `nexcore-api` tenant integration to current `vr_core` interfaces, then wired `nexcore-api` and `nexcore-primitive-scanner`. Final snapshot: Set B 124, coverage 67.0%, domain-gap 0, mismatches 0.
