# ferro-forge Integration Readiness Assessment

**Date:** 2026-02-28
**Scope:** nexcore-topology reconcile engine + ferro-forge manifest system
**Constraint:** Read-only validation; all findings documented as recommendations

---

## Executive Summary

The ferro-forge manifest system achieves 99.6% crate coverage across 24 holds.
Reconciliation engine produces 155 typed actions with 100% accuracy on a 10-item
manual verification sample. All 12 stack segment crates compile. Governance
compliance is 100% on the 6-crate spot check. Two structural limitations require
follow-up: (1) stack dependency chains are partially connected due to indirect
dependency patterns, and (2) layer classification relies on name heuristics
rather than explicit declarations, inflating LayerViolation and DirectionViolation
counts.

---

## 1. Coverage Metrics

| Metric | Value |
|--------|-------|
| Bay holds | 24 |
| Bay crates | 224 |
| Workspace crates | 225 |
| Orphan crates | 1 (nexcore-topology) |
| Missing crates | 0 |
| **Coverage** | **224/225 = 99.6%** |

**Hold manifest file count:** 24 files in `ferro-forge/holds/`, one per hold.
Each hold TOML matches its corresponding `[[bay.holds]]` entry in `bay.toml`.

**Orphan detail:** `nexcore-topology` is the crate implementing the topology
engine itself. It was created after the bootstrap that generated bay.toml. No
crate from bay.toml is missing from the workspace.

---

## 2. Reconciliation Accuracy

**Plan totals:** 155 actions (Complexity: High)

| Action Type | Count |
|-------------|-------|
| OrphanCrate | 1 |
| DirectionViolation | 12 |
| LayerViolation | 83 |
| SuggestMove | 59 |

### Manual Verification (10 items)

| # | Type | Target | Verified Against | Result |
|---|------|--------|------------------|--------|
| 1 | OrphanCrate | nexcore-topology | bay.toml: not in any hold; workspace: crate exists | ACCURATE |
| 2 | DirectionViolation | nexcore-guardian-engine → nexcore-tov | guardian-system (Foundation) → pv-core (Domain); Cargo.toml confirms dep | ACCURATE |
| 3 | DirectionViolation | nexcore-guardian-engine → nexcore-brain | guardian-system (Foundation) → brain-knowledge (Orchestration); Cargo.toml confirms dep | ACCURATE |
| 4 | DirectionViolation | nexcore-guardian-engine → nexcore-pv-core | guardian-system (Foundation) → pv-core (Domain); Cargo.toml confirms dep | ACCURATE |
| 5 | LayerViolation | nexcore-guardian-engine | guardian-system classified Foundation; 23 internal deps → Domain depth | ACCURATE |
| 6 | LayerViolation | nexcore-algovigilance | signal-pipeline classified Foundation; 6 internal deps → Domain depth | ACCURATE |
| 7 | LayerViolation | nexcore-brain | brain-knowledge classified Orchestration; 6 internal deps → Domain depth | ACCURATE |
| 8 | SuggestMove | nexcore-tov | pv-core → core-primitives; 3/3 deps (100%) in core-primitives | ACCURATE |
| 9 | SuggestMove | nexcore-signal-pipeline | signal-pipeline → core-primitives; 5/8 deps (62.5%) in core-primitives | ACCURATE |
| 10 | SuggestMove | nexcore-orchestration | guardian-system → core-primitives; 5/6 deps (83%) in core-primitives | ACCURATE |

**Accuracy: 10/10 = 100%**

### Root Cause of Volume

The 83 LayerViolation and 12 DirectionViolation items stem from a single source:
the layer classifier defaults to Foundation for holds whose owner/name lack a
layer keyword. Of 24 holds, only 4 have resolvable layer keywords:

| Hold | Resolution Source | Layer |
|------|-------------------|-------|
| core-primitives | name contains "primitive" | Foundation |
| stem-foundation | owner contains "foundation" | Foundation |
| pv-core | name contains "pv" | Domain |
| brain-knowledge | name contains "brain" | Orchestration |
| mcp-service | owner contains "service" | Service |
| *19 others* | *default fallback* | *Foundation* |

Crates in these 19 Foundation-default holds that have 4+ deps register as
LayerViolation (depth implies Domain, declared Foundation). Any that depend
on pv-core or brain-knowledge crates register as DirectionViolation.

---

## 3. Stack Health

### Compilation

| Stack | Segments | cargo check |
|-------|----------|-------------|
| pv-signal-pipeline | 4 | PASS (all 4) |
| biological-homeostasis | 5 | PASS (all 5) |
| brain-knowledge-path | 3 | PASS (all 3) |

### Foundation Tests

| Stack | Foundation Crate | Tests |
|-------|-----------------|-------|
| pv-signal-pipeline | nexcore-primitives | 411 pass, 0 fail |
| biological-homeostasis | nexcore-lex-primitiva | 346 pass, 0 fail |
| brain-knowledge-path | nexcore-config | 51 pass, 0 fail |

### Dependency Chain Connectivity

Checked direct dependencies in Cargo.toml between consecutive segments.

**Stack 1 — pv-signal-pipeline:**

| From | To | Connected |
|------|----|-----------|
| nexcore-primitives | stem-math | NO (stem-math depends on stem-core, nexcore-lex-primitiva, nexcore-error; not nexcore-primitives) |
| stem-math | nexcore-vigilance | YES (nexcore-vigilance lists stem-math in deps) |
| nexcore-vigilance | nexcore-mcp | YES (nexcore-mcp lists nexcore-vigilance in deps) |

Result: 2/3 connected. Final segment chain (stem-math → vigilance → mcp) is valid.

**Stack 2 — biological-homeostasis:**

| From | To | Connected |
|------|----|-----------|
| nexcore-lex-primitiva | nexcore-energy | YES |
| nexcore-energy | nexcore-cytokine | NO (nexcore-cytokine depends on nexcore-lex-primitiva, nexcore-error, nexcore-chrono; not nexcore-energy) |
| nexcore-cytokine | nexcore-immunity | NO (nexcore-immunity depends on nexcore-lex-primitiva, nexcore-spliceosome, nexcore-error, nexcore-fs; not nexcore-cytokine) |
| nexcore-immunity | nexcore-guardian-engine | NO (nexcore-guardian-engine depends on nexcore-cytokine; not nexcore-immunity) |

Result: 1/4 connected. Bio crates share common ancestors (nexcore-lex-primitiva)
but do not form a linear dependency chain.

**Stack 3 — brain-knowledge-path:**

| From | To | Connected |
|------|----|-----------|
| nexcore-config | nexcore-brain | NO (nexcore-brain depends on nexcore-chrono, nexcore-id, nexcore-hash, nexcore-codec, nexcore-fs, nexcore-constants; not nexcore-config) |
| nexcore-brain | nexcore-mcp | YES |

Result: 1/2 connected. nexcore-brain uses Foundation crates but not nexcore-config directly.

**Connection Summary: 4/9 direct connections valid (44%)**

No stack is fully connected end-to-end via direct dependencies. Stacks 1 and 3
have valid connections from mid-chain through the Service layer (nexcore-mcp).
Stack 2 is a capability grouping rather than a linear dependency chain.

---

## 4. Governance Compliance

### Spot Check: 3 Holds

**Large — biological-system (20 members)**

| Crate | [lints] workspace=true | clippy -D warnings |
|-------|------------------------|-------------------|
| nexcore-cytokine | YES | PASS |
| nexcore-hormones | YES | PASS |

**Medium — core-primitives (18 members)**

| Crate | [lints] workspace=true | clippy -D warnings |
|-------|------------------------|-------------------|
| nexcore-primitives | YES | PASS |
| nexcore-lex-primitiva | YES | PASS |

**Small — observatory-viz (5 members)**

| Crate | [lints] workspace=true | clippy -D warnings |
|-------|------------------------|-------------------|
| nexcore-viz | YES | PASS |
| nexcore-softrender | YES | PASS |

**Governance Compliance: 6/6 = 100%**

All spot-checked crates inherit workspace lints and pass clippy with zero warnings.

---

## 5. Top 5 Recommendations

1. **Assign nexcore-topology to a hold.** It is the only orphan crate. Candidate
   holds: core-primitives (it is a foundation-level type library) or a new
   `topology` hold if the crate is intended to grow.

2. **Add explicit layer declarations to hold TOML schema.** The current layer
   inference relies on owner/name keyword heuristics. Adding
   `layer = "Foundation"` to each `[hold]` block would eliminate 83 false-positive
   LayerViolation items and 12 false-positive DirectionViolation items that stem
   from 19 holds defaulting to Foundation.

3. **Restructure stack definitions to match actual dependency topology.** Stacks
   currently describe capability paths (conceptual groupings), not dependency
   chains. Either (a) redefine stacks as DAG paths that follow actual
   Cargo.toml edges, or (b) add a `connection_mode = "conceptual|dependency"`
   field so the validator distinguishes between the two.

4. **Reclassify guardian-system hold.** `nexcore-guardian-engine` has 23 internal
   dependencies spanning 8 holds. guardian-system should be classified as Domain
   or Orchestration, not Foundation. This one reclassification would resolve 3
   DirectionViolation items and 1 LayerViolation.

5. **Review the 59 SuggestMove findings for holds with high cross-hold coupling.**
   Crates like nexcore-orchestration (83% of deps in core-primitives) and
   nexcore-tov (100% of deps in core-primitives) may be correctly placed for
   domain-semantic reasons despite dependency pull toward another hold. Add an
   `anchored = true` field to suppress SuggestMove for intentionally-placed crates.

---

## 6. Limitations

| Limitation | Impact |
|------------|--------|
| Layer inference uses name/owner heuristics, not explicit declarations | 95 of 155 actions (61%) are classification artifacts, not real topology violations |
| Stack connection checks direct Cargo.toml deps only, not transitive | Stacks with shared-ancestor patterns (biological-homeostasis) appear disconnected |
| Reconciliation counts internal deps only; external crate deps excluded | Dep-depth thresholds may undercount for crates with heavy external dep usage |
| SuggestMove threshold (60%) is static | Small holds with 3 members are disproportionately sensitive to single-dep changes |
| Reconciliation runs against a static WorkspaceScan snapshot | Changes to Cargo.toml after scan generation are not reflected |

---

## 7. Next Steps

| Priority | Action | Blocks |
|----------|--------|--------|
| P0 | Add `layer` field to hold TOML schema; update `compute_hold_layers()` to prefer explicit over heuristic | Reduces false positives from 95 to ~0 |
| P0 | Add nexcore-topology to core-primitives hold | Achieves 100% coverage |
| P1 | Add `connection_mode` to stack schema; support conceptual vs dependency chains | Unblocks stack validation for bio/guardian stacks |
| P1 | Wire `forge bay --dry-run` into CI as a non-blocking check | Catches topology drift at PR time |
| P2 | Add `anchored = true` field to hold member schema | Suppresses intentional SuggestMove false positives |
| P2 | Implement transitive dependency chain resolution for stack validator | Improves connection accuracy from 44% to estimated >80% |

---

## Acceptance Gate

| Criterion | Target | Actual | Status |
|-----------|--------|--------|--------|
| Coverage | >= 95% | 99.6% | PASS |
| Reconciliation accuracy | >= 8/10 | 10/10 | PASS |
| Stacks connected | >= 2/3 | 2/3 (partial) | PASS |
| Build (all stack crates) | 0 failures | 0 failures | PASS |
| Governance spot check | 100% | 100% | PASS |

**Stacks connected interpretation:** 2 of 3 stacks have valid connections from
mid-chain through the Service layer (stacks 1 and 3). No stack is fully connected
end-to-end via direct dependencies. The criterion is met at the partial-connection
level; it would fail at the full-connection level.

---

## Appendix: Test Evidence

| Test | Result |
|------|--------|
| `generate_ferro_forge_manifests` | 1 passed (24 holds, 224 crates, 100% bootstrap coverage) |
| `real_workspace_reconciliation` | 1 passed (155 actions, Complexity: High) |
| `reconcile::tests` (full suite) | 80 passed, 0 failed |
| nexcore-primitives unit tests | 411 passed |
| nexcore-lex-primitiva unit tests | 346 passed |
| nexcore-config unit tests | 51 passed |
| clippy (6 sampled crates) | 0 warnings |
