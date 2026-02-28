# SuggestMove Triage Report (FF2-CaC-001 P7/P8)

**Date:** 2026-02-28
**Directive:** Crates-as-Code SuggestMove Triage + Directional Filter
**Reconciliation paths:** Bootstrap (workspace-topology.json) and Manifest (ferro-forge/holds/*.toml)

---

## Before/After Summary

| Metric | Before (P7 baseline) | After P7 (orphan fix) | After P8 (directional filter) |
|--------|---------------------|-----------------------|-------------------------------|
| **Bootstrap Path** | | | |
| Total actions | 79 | 78 | 24 |
| OrphanCrate | 1 | 1 | 1 |
| DirectionViolation | 19 | 19 | 17 |
| LayerViolation | 0 | 0 | 6 |
| SuggestMove | 59 | 58 | 0 |
| **Manifest Path** | | | |
| Total actions | 78 | 19 | 18 |
| OrphanCrate | 0 | 0 | 0 |
| DirectionViolation | 19 | 19 | 12 |
| LayerViolation | 0 | 0 | 6 |
| SuggestMove | 59 | 0 | 0 |

### P7 Changes
- Orphan fix: `nexcore-topology` added to `build-tooling` hold
- Manual triage: all 59 SuggestMoves classified as false positives (foundation gravity)

### P8 Changes (Directional Filter)
- Added layer-aware directional filter in `suggest_moves()`: suppresses downward moves (higher→lower layer) and moves into foundation-keyword holds
- Fixed `is_foundation_hold_name()` substring false positive: `"system"` was matching `"stem"`, clamping `guardian-system`, `biological-system`, and `system-utilities` to Foundation
- Corrected hold layer assignments in TOML: `core-primitives`→Foundation, `stem-foundation`→Foundation, `chemistry`→Domain (was Foundation), `observatory-viz`→Domain (was Foundation)
- Changed `infer_layer_from_name()` default from Foundation to Domain (most holds are domain-level)
- Segment-based keyword matching (split on `-`) replaces naive `contains()` to prevent substring collisions

### P8 Net Impact
- SuggestMove: 59 → **0** (all foundation gravity false positives eliminated)
- DirectionViolation: 19 → **17** (bootstrap) / **12** (manifest) — corrected layer assignments resolved some
- LayerViolation: 0 → **6** — surfaced pre-existing issues in `core-primitives` and `stem-foundation` where member crates have more deps than the Foundation threshold (≤3)
- Total: 79 → **24** (bootstrap) / **18** (manifest)

---

## Key Finding: Foundation Gravity

58 of 59 SuggestMove actions suggest moving crates to `core-primitives`. This is a structural artifact, not a misplacement signal. Every domain crate depends on foundation primitives (`nexcore-primitives`, `nexcore-id`, `nexcore-config`, etc.), so naive dependency-ratio analysis always finds that the majority of a crate's deps are in `core-primitives`. Moving domain crates to a foundation hold would destroy the layer architecture.

The 1 remaining SuggestMove (`nexcore-chemivigilance` to `chemistry`) is a legitimate dual-domain candidate but classified as SPLIT rather than ACCEPT because the crate bridges pharmacovigilance and chemistry domains.

---

## Executed Moves

| Crate | From | To | Rationale |
|-------|------|----|-----------|
| `nexcore-topology` | (unassigned) | `build-tooling` | Orphan fix: topology crate belongs with build tooling |

**Total ACCEPT moves: 0** (all SuggestMoves are false positives)

---

## Full Classification Table (59 SuggestMove Actions)

| # | Crate | Current Hold | Suggested Hold | Dep Ratio | Classification | Rule | Rationale |
|---|-------|-------------|----------------|-----------|---------------|------|-----------|
| 1 | claude-fs-mcp | mcp-service | core-primitives | 2/3 | REJECT | 3 | Foundation gravity; MCP crate stays in service layer |
| 2 | nexcloud | observatory-viz | core-primitives | 3/3 | REJECT | 3 | Foundation gravity; observatory crate correctly placed |
| 3 | nexcore-antibodies | quality-assurance | core-primitives | 4/4 | REJECT | 3 | Foundation gravity; QA crate semantically correct |
| 4 | nexcore-browser | observatory-viz | core-primitives | 3/3 | REJECT | 3 | Foundation gravity; observatory crate correctly placed |
| 5 | nexcore-build-gate | build-tooling | core-primitives | 3/4 | REJECT | 3+2 | Foundation gravity; name aligns with current hold |
| 6 | nexcore-capa | quality-assurance | core-primitives | 5/5 | REJECT | 3 | Foundation gravity; CAPA is a QA concept |
| 7 | nexcore-chemivigilance | pv-core | chemistry | 4/6 | SPLIT | 4 | Bridges PV and chemistry domains; 4/6 chemistry deps but core PV function |
| 8 | nexcore-compliance | regulatory-compliance | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; name aligns with current hold |
| 9 | nexcore-cortex | brain-knowledge | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; cortex is a brain concept |
| 10 | nexcore-cytokine | biological-system | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; biological crate correctly placed |
| 11 | nexcore-dtree | data-computation | core-primitives | 3/4 | REJECT | 3+2 | Foundation gravity; decision tree is data computation |
| 12 | nexcore-foundry | build-tooling | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; name aligns with current hold |
| 13 | nexcore-ghost | linguistics | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; ghost is a linguistics concept |
| 14 | nexcore-grammar-lab | linguistics | stem-foundation | 3/4 | REJECT | 2 | Name aligns with linguistics; stem deps are structural |
| 15 | nexcore-homeostasis-memory | guardian-system | core-primitives | 3/4 | REJECT | 3+2 | Foundation gravity; homeostasis is a guardian concept |
| 16 | nexcore-hormones | biological-system | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; biological crate correctly placed |
| 17 | nexcore-insight | analysis-tools | core-primitives | 4/4 | REJECT | 3+2 | Foundation gravity; insight is an analysis concept |
| 18 | nexcore-jeopardy | experimentation | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; jeopardy is experimentation |
| 19 | nexcore-knowledge-engine | brain-knowledge | core-primitives | 4/5 | REJECT | 3+2 | Foundation gravity; knowledge engine is brain domain |
| 20 | nexcore-labs | experimentation | core-primitives | 5/6 | REJECT | 3+2 | Foundation gravity; labs is experimentation |
| 21 | nexcore-measure | data-computation | core-primitives | 4/5 | REJECT | 3 | Foundation gravity; measure belongs in data-computation |
| 22 | nexcore-mesh | regulatory-compliance | core-primitives | 5/7 | REJECT | 3+2 | Foundation gravity; MeSH is regulatory domain |
| 23 | nexcore-notebooklm | mcp-service | core-primitives | 3/4 | REJECT | 3 | Foundation gravity; MCP crate stays in service layer |
| 24 | nexcore-openfda | regulatory-compliance | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; OpenFDA is regulatory domain |
| 25 | nexcore-orchestration | guardian-system | core-primitives | 5/6 | REJECT | 3+2 | Foundation gravity; orchestration is guardian domain |
| 26 | nexcore-organize | system-utilities | core-primitives | 5/6 | REJECT | 3 | Foundation gravity; utility crate correctly placed |
| 27 | nexcore-prima | prima-language | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; Prima language crate correctly placed |
| 28 | nexcore-proof-of-meaning | business-strategy | core-primitives | 4/4 | REJECT | 3+2 | Foundation gravity; PoM is business strategy |
| 29 | nexcore-qbr | pv-core | core-primitives | 4/5 | REJECT | 3+2 | Foundation gravity; QBR is a PV concept |
| 30 | nexcore-reason | analysis-tools | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; reasoning is analysis domain |
| 31 | nexcore-registry | system-utilities | core-primitives | 2/3 | REJECT | 3 | Foundation gravity; registry is system utility |
| 32 | nexcore-renderer | observatory-viz | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; renderer is observatory domain |
| 33 | nexcore-retrocasting | analysis-tools | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; retrocasting is analysis |
| 34 | nexcore-ribosome | biological-system | core-primitives | 3/4 | REJECT | 3+2 | Foundation gravity; biological crate correctly placed |
| 35 | nexcore-sentinel | guardian-system | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; sentinel is guardian domain |
| 36 | nexcore-signal-pipeline | signal-pipeline | core-primitives | 5/8 | REJECT | 3+2 | Foundation gravity; name aligns with current hold |
| 37 | nexcore-skill-exec | skills-engine | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; skill crate correctly placed |
| 38 | nexcore-social | mcp-service | core-primitives | 3/3 | REJECT | 3 | Foundation gravity; MCP service crate |
| 39 | nexcore-stoichiometry | chemistry | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; stoichiometry is chemistry |
| 40 | nexcore-synapse | biological-system | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; biological crate correctly placed |
| 41 | nexcore-telemetry-core | observability | core-primitives | 4/5 | REJECT | 3+2 | Foundation gravity; telemetry is observability |
| 42 | nexcore-terminal | os-runtime | core-primitives | 3/4 | REJECT | 3 | Foundation gravity; terminal is OS runtime |
| 43 | nexcore-tov | pv-core | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; Theory of Vigilance is PV core |
| 44 | nexcore-trial | experimentation | core-primitives | 5/5 | REJECT | 3+2 | Foundation gravity; trial is experimentation |
| 45 | nexcore-value-mining | business-strategy | core-primitives | 3/4 | REJECT | 3+2 | Foundation gravity; value mining is business strategy |
| 46 | nexcore-vault | system-utilities | core-primitives | 4/5 | REJECT | 3 | Foundation gravity; vault is system utility |
| 47 | signal | experimental-prototypes | core-primitives | 3/4 | REJECT | 3 | Foundation gravity; experimental prototype |
| 48 | skill-hunter | skills-engine | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; skill crate correctly placed |
| 49 | stem-complex | stem-foundation | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; stem crate correctly placed |
| 50 | stem-core | stem-foundation | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; stem crate correctly placed |
| 51 | stem-math | stem-foundation | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; stem crate correctly placed |
| 52 | stem-number-theory | stem-foundation | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; stem crate correctly placed |
| 53 | transformer-primitives | experimental-prototypes | core-primitives | 2/3 | REJECT | 3 | Foundation gravity; experimental prototype |
| 54 | vr-billing | business-strategy | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; VR billing is business strategy |
| 55 | vr-compliance | business-strategy | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; VR compliance is business strategy |
| 56 | vr-core | business-strategy | core-primitives | 3/3 | REJECT | 3+2 | Foundation gravity; VR core is business strategy |
| 57 | vr-marketplace | business-strategy | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; VR marketplace is business strategy |
| 58 | vr-platform-ml | business-strategy | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; VR platform ML is business strategy |
| 59 | vr-tenant | business-strategy | core-primitives | 2/3 | REJECT | 3+2 | Foundation gravity; VR tenant is business strategy |

### Classification Rules Applied

| Rule | Description | Count |
|------|-------------|-------|
| Rule 2 | Crate name aligns with current hold (semantic match) | 0 solo, 44 combined |
| Rule 3 | Suggested hold is foundation-layer (foundation gravity false positive) | 58 (44 with Rule 2) |
| Rule 4 | Deps split across two domain holds (SPLIT candidate) | 1 |
| Rule 5 | Dep ratio > 80% toward non-foundation hold | 0 |

---

## SPLIT Candidates (Deferred)

| Crate | Current | Suggested | Dep Ratio | Analysis |
|-------|---------|-----------|-----------|----------|
| nexcore-chemivigilance | pv-core | chemistry | 4/6 to chemistry | Bridges pharmacovigilance and chemistry domains. 67% of deps in chemistry hold, but the crate's primary function is PV signal detection on chemical entities. Candidate for future refactoring into a bridge crate or explicit dual-hold membership. |

---

## Remaining Issues (Post-P8) — SUPERSEDED

> **Note:** This section captures the post-P8 baseline before bio-remediation and Phases 1-3. For the current DV inventory (8 remaining), see **"Phase 1-3 Remediation Results → Current DV Inventory"** below.

### DirectionViolations (17 bootstrap / 12 manifest)

Post-P8 direction violations on the manifest path (the authoritative source).
Items 11-12 (stem/stem-bio → biological-system) were resolved by the bio-remediation arc (nexcore-hormone-types extraction, commit 555f4bbb). Items 1, 5, 9 removed by dead dep cleanup (commit 03103855). Items 4, 6, 11 resolved by hold reclassifications (same commit).

| # | Crate | Source Hold | Target Hold | Direction | Status |
|---|-------|-----------|------------|-----------|--------|
| 1 | nexcore-compositor | build-tooling (Domain) | os-runtime (Orchestration) | Domain → Orchestration | Open |
| 2 | nexcore-constants | core-primitives (Foundation) | system-utilities (Domain) | Foundation → Domain | Open |
| 3 | nexcore-core | core-primitives (Foundation) | brain-knowledge (Domain) | Foundation → Domain | Open |
| 4 | nexcore-faers-etl | regulatory-compliance (Domain) | pv-core (Orchestration) | Domain → Orchestration | Open |
| 5 | nexcore-init | core-primitives (Foundation) | build-tooling (Domain) | Foundation → Domain | Open |
| 6 | nexcore-model-checker | analysis-tools (Domain) | os-runtime (Orchestration) | Domain → Orchestration | Open |
| 7 | nexcore-pharos | observability (Domain) | guardian-system (Orchestration) | Domain → Orchestration | Open |
| 8 | nexcore-rh-proofs | core-primitives (Foundation) | pv-core (Orchestration) | Foundation → Orchestration | Open |
| 9 | nexcore-value-mining | business-strategy (Domain) | mcp-service (Service) | Domain → Service | Open |
| 10 | nexcore-watch-core | observability (Domain) | pv-core (Orchestration) | Domain → Orchestration | Open |
| 11 | ~~stem~~ | ~~stem-foundation~~ | ~~biological-system~~ | ~~Foundation → Domain~~ | **Fixed** (555f4bbb) |
| 12 | ~~stem-bio~~ | ~~stem-foundation~~ | ~~biological-system~~ | ~~Foundation → Domain~~ | **Fixed** (555f4bbb) |

### LayerViolations (6 both paths)

Crates in foundation holds whose dep count exceeds the Foundation threshold (≤3):

| # | Crate | Hold | Declared Layer | Actual Deps |
|---|-------|------|---------------|-------------|
| 1 | nexcore-core | core-primitives | Foundation | 4 |
| 2 | nexcore-init | core-primitives | Foundation | 6 |
| 3 | nexcore-primitives | core-primitives | Foundation | 4 |
| 4 | nexcore-rh-proofs | core-primitives | Foundation | 5 |
| 5 | stem | stem-foundation | Foundation | 5 |
| 6 | stem-phys | stem-foundation | Foundation | 4 |

### Violation Patterns

| Pattern | Count | Root Cause | Potential Fix |
|---------|-------|------------|---------------|
| Foundation crate → Domain/Orchestration hold | 3 DV | Foundation crates (`nexcore-core`, `nexcore-init`, `nexcore-constants`, `nexcore-rh-proofs`) depend on higher-layer crates | Move high-dep Foundation members to Domain hold, or reclassify target holds |
| Domain → Orchestration | 5 DV | `pv-core` and `os-runtime` assigned Orchestration but consumed by Domain crates | Consider downgrading `pv-core`/`os-runtime` to Domain |
| Foundation members with >3 deps | 6 LV | `core-primitives` and `stem-foundation` contain crates that have grown beyond Foundation threshold | Move `nexcore-core`, `nexcore-init`, `nexcore-primitives`, `nexcore-rh-proofs` to a domain hold, or accept as known violations |
| Domain → Service | 1 DV | `nexcore-value-mining` depends on `nexcore-social` (MCP service) | Move `nexcore-social` to a domain hold, or restructure dependency |

---

## Post Bio-Remediation Baseline (2026-02-28)

**Context:** Biological-system hold split into bio-molecular (9 crates) + bio-anatomical (11 crates). `nexcore-hormone-types` extracted from `nexcore-hormones` and placed in stem-foundation hold to resolve stem→bio-molecular DirectionViolations.

### Action Counts

| Metric | P8 Baseline | Post Bio-Remediation | Delta |
|--------|-------------|---------------------|-------|
| **Bootstrap Path** | | | |
| Total actions | 24 | 24 | 0 |
| OrphanCrate | 1 | 3 | +2 |
| DirectionViolation | 17 | 15 | -2 |
| LayerViolation | 6 | 6 | 0 |
| SuggestMove | 0 | 0 | 0 |
| **Manifest Path** | | | |
| Total actions | 18 | 23 | +5 |
| OrphanCrate | 0 | 2 | +2 |
| DirectionViolation | 12 | 15 | +3 |
| LayerViolation | 6 | 6 | 0 |
| SuggestMove | 0 | 0 | 0 |

### Bio Remediation Impact

**Fixed (2 DVs eliminated):**
- `stem` (stem-foundation) → `nexcore-hormones` (biological-system) — resolved by switching to `nexcore-hormone-types` (stem-foundation)
- `stem-bio` (stem-foundation) → `nexcore-hormones` (biological-system) — resolved by switching to `nexcore-hormone-types` (stem-foundation)

**Newly Surfaced (5 DVs):**

The biological-system split and hold restructuring exposed 5 chemistry→prima-language direction violations that were previously masked:

| # | Crate | Source Hold (Layer) | Target Hold (Layer) | Direction |
|---|-------|-------------------|-------------------|-----------|
| 1 | nexcore-metabolite | chemistry (Foundation) | prima-language (Domain) | Foundation → Domain |
| 2 | nexcore-molcore | chemistry (Foundation) | prima-language (Domain) | Foundation → Domain |
| 3 | nexcore-qsar | chemistry (Foundation) | prima-language (Domain) | Foundation → Domain |
| 4 | nexcore-renderer | observatory-viz (Foundation) | prima-language (Domain) | Foundation → Domain |
| 5 | nexcore-structural-alerts | chemistry (Foundation) | prima-language (Domain) | Foundation → Domain |

**Root cause:** Chemistry hold was declared Foundation-layer, but 4 chemistry crates depend on `prima-chem` (prima-language hold, Domain-layer). Similarly, `nexcore-renderer` depends on `prima` (prima-language hold). These were pre-existing layer mismatches exposed by the hold topology changes.

**Resolution (commit e74d6db5):** Chemistry hold reclassified from Foundation to Domain. This resolved the 4 chemistry→prima-language DVs. The nexcore-renderer→prima DV remains (observatory-viz is Foundation-layer).

### Orphan Changes

| Crate | P8 Status | Current Status | Cause |
|-------|-----------|---------------|-------|
| nexcore-pty | Not in workspace | Orphan (both paths) | New crate, not assigned to any hold |
| nexcore-topology | In build-tooling | Orphan (both paths) | Hold assignment lost during bay.toml regeneration |
| nexcore-hormone-types | N/A | Orphan (bootstrap only) | Added to stem-foundation.toml manifest but bootstrap topology not regenerated |

### Net Assessment

The bio-remediation arc achieved its primary objective: stem-foundation → bio-molecular DVs eliminated. The manifest total rose from 18→23 due to 5 previously-masked chemistry→prima-language DVs surfacing — these are pre-existing architectural issues, not regressions. Two orphan crates need hold assignment (nexcore-pty, nexcore-topology re-assignment).

---

## DV Classification (2026-02-28)

**Method:** For each of the 11 remaining DVs, the violating Cargo.toml dependency was traced to actual `use` imports in source code. Dependencies with zero imports are dead code (FIXABLE-REMOVE). Used dependencies were classified by whether the fix is a hold layer change (FIXABLE-RECLASSIFY), a type extraction (FIXABLE-EXTRACT), or an intentional architectural edge (ACCEPT).

### Classification Table

| # | Source Crate | Source Hold (Layer) | Target Crate | Target Hold (Layer) | Used? | Classification | Fix |
|---|-------------|-------------------|-------------|-------------------|-------|---------------|-----|
| 1 | nexcore-compositor | build-tooling (Domain) | nexcore-os | os-runtime (Orchestration) | No | FIXABLE-REMOVE | Remove unused dep from Cargo.toml |
| 2 | nexcore-constants | core-primitives (Foundation) | nexcore-fs | system-utilities (Domain) | Yes | ACCEPT | See rationale below |
| 3 | nexcore-core | core-primitives (Foundation) | nexcore-brain | brain-knowledge (Domain) | Yes | LIVE-MISCLASSIFIED | Used via `nexcore_brain::BrainSession` (lib.rs:82,87) |
| 4 | nexcore-faers-etl | regulatory-compliance (Domain) | nexcore-vigilance | pv-core (Orchestration) | Yes | FIXABLE-RECLASSIFY | Reclassify pv-core to Domain |
| 5 | nexcore-init | core-primitives (Foundation) | nexcore-compositor | build-tooling (Domain) | No | FIXABLE-REMOVE | Remove unused dep from Cargo.toml |
| 6 | nexcore-model-checker | analysis-tools (Domain) | nexcore-state-theory | os-runtime (Orchestration) | Yes | FIXABLE-RECLASSIFY | Move nexcore-state-theory to analysis-tools |
| 7 | nexcore-pharos | observability (Domain) | nexcore-guardian-engine | guardian-system (Orchestration) | Yes | ACCEPT | See rationale below |
| 8 | nexcore-renderer | observatory-viz (Foundation) | prima | prima-language (Domain) | Yes | LIVE-MISCLASSIFIED | Used via `prima::eval` (adventure.rs:60,67,69) |
| 9 | nexcore-rh-proofs | core-primitives (Foundation) | nexcore-tov-proofs | pv-core (Orchestration) | No | FIXABLE-REMOVE | Remove unused dep from Cargo.toml |
| 10 | nexcore-value-mining | business-strategy (Domain) | nexcore-social | mcp-service (Service) | Yes | FIXABLE-EXTRACT | Extract `Post` type to domain-layer types crate |
| 11 | nexcore-watch-core | observability (Domain) | nexcore-pvos | pv-core (Orchestration) | Yes | FIXABLE-RECLASSIFY | Reclassify pv-core to Domain |

### Summary by Classification

| Classification | Count | DVs | Projected DV Reduction |
|---------------|-------|-----|----------------------|
| FIXABLE-REMOVE | 3 | 1, 5, 9 | -3 (dead dep removal) |
| FIXABLE-RECLASSIFY | 3 | 4, 6, 11 | -3 (hold layer changes) |
| FIXABLE-EXTRACT | 1 | 10 | -1 (type extraction) |
| ACCEPT | 2 | 2, 7 | 0 (documented exceptions) |
| LIVE-MISCLASSIFIED | 2 | 3, 8 | 0 (still DVs — require reclassification or accept) |
| **Total** | **11** | | **-7 fixable, 4 permanent or pending** |

### ACCEPT Rationale

**DV2 — nexcore-constants → nexcore-fs (Foundation → Domain):**
`nexcore-constants` uses `nexcore_fs::dirs` in `bathroom_lock.rs` for platform-aware directory resolution (`$HOME`, XDG paths). A Foundation crate needing to locate the filesystem root for lock files is a single-function boundary crossing. The alternative — extracting `dirs` into its own foundation crate — creates a single-function crate for negligible layer-purity gain. The dependency is narrow (one import, one call site) and stable.

**DV7 — nexcore-pharos → nexcore-guardian-engine (Domain → Orchestration):**
PHAROS (Pharmacovigilance Autonomous Reconnaissance and Observation System) imports `SignalSource`, `ThreatLevel`, and `ThreatSignal` from the guardian's `sensing` module. PHAROS exists to observe system health signals — consuming guardian threat classifications is its primary purpose, not a leaked abstraction. The guardian exports these sensing types for exactly this use case. Extracting them into a separate types crate would fracture the guardian's cohesive threat API for minimal benefit. The dependency is narrow (3 types from one submodule) and architecturally intentional.

### FIXABLE-REMOVE Detail (Dead Dependencies — Removed in Phase 1)

3 DVs were caused by Cargo.toml dependencies with zero source references. All 3 were removed in Phase 1 (commit 03103855). The table below shows the pre-removal state.

| DV | Source Crate | Dead Dependency | Cargo.toml Line |
|----|-------------|----------------|-----------------|
| 1 | nexcore-compositor | `nexcore-os = { version = "0.1.0", path = "../nexcore-os" }` | deps section |
| 5 | nexcore-init | `nexcore-compositor = { version = "0.1.0", path = "../nexcore-compositor" }` | deps section |
| 9 | nexcore-rh-proofs | `nexcore-tov-proofs = { workspace = true }` | deps section |

**Misclassification note:** DV3 (`nexcore-core` → `nexcore-brain`) and DV8 (`nexcore-renderer` → `prima`) were originally classified as dead dependencies by the audit agent. Both are live — they use fully-qualified paths (`nexcore_brain::BrainSession`, `prima::eval`) rather than top-level `use` imports, which the agent's grep pattern did not detect.

### FIXABLE-RECLASSIFY Detail

**pv-core: Orchestration → Domain (fixes DV4 + DV11):**
pv-core holds 13 crates including `nexcore-vigilance` (the 57-module domain monolith with 25 deps, described in CLAUDE.md as "the domain monolith"). `nexcore-pvos`, `nexcore-pv-core`, `nexcore-tov`, `nexcore-qbr` are PV domain types and algorithms, not workflow orchestration. The Orchestration classification reflects the high dep count of nexcore-vigilance, not the semantic purpose of the hold. Domain crates (`nexcore-faers-etl`, `nexcore-watch-core`) consuming PV signal types is architecturally correct — the direction violation is a classification error, not a dependency error.

**nexcore-state-theory: os-runtime → analysis-tools (fixes DV6):**
`nexcore-state-theory` provides abstract temporal logic types (`AtomicProp`, `CtlFormula`, `LtlFormula`) used by `nexcore-model-checker` for formal verification. These are mathematical constructs, not OS runtime features. The crate's placement in os-runtime appears historical (co-located with other state-related crates). Moving it to analysis-tools aligns the hold assignment with actual usage. No other os-runtime members depend on nexcore-state-theory.

### FIXABLE-EXTRACT Detail

**DV10 — Extract `Post` type from nexcore-social (fixes DV10):**
`nexcore-value-mining` imports `nexcore_social::Post` across 6 signal detection modules (engagement, trend, controversy, sentiment, virality, signals). The `Post` type is a domain-layer data structure (social media post representation) that happens to live in nexcore-social (mcp-service/Service layer). Following the nexcore-hormone-types pattern: extract `Post` and related domain types into `nexcore-social-types` (placed in business-strategy or a shared domain hold), then have nexcore-social re-export via `pub use nexcore_social_types::*`.

### Recommended Remediation Sequence

| Phase | Action | DVs Fixed | Side Effects | Effort | Resulting DV Count |
|-------|--------|-----------|-------------|--------|-------------------|
| 1 | Remove 3 dead deps from Cargo.toml | 1, 5, 9 | Unmasked 2 hidden DVs (init→os, rh-proofs→zeta) | Trivial (3 line deletions) | 8 (net 0: -3 removed, +1 unmasked init→os, +1 unmasked rh-proofs→zeta, +1 reclassified core→brain) |
| 2 | Reclassify pv-core: Orchestration → Domain | 4, 11 | +1 DV (vigilance→guardian-engine), +1 LV | Low (1 TOML edit) | 7 |
| 3 | Move nexcore-state-theory: os-runtime → analysis-tools | 6 | None | Low (2 TOML edits) | 6 |
| 4 | Extract nexcore-social-types | 10 | TBD | Medium (new crate + re-export wrapper) | 5 |
| — | Documented exceptions (ACCEPT) | 2, 7 | — | — | — |
| — | Pending analysis (LIVE-MISCLASSIFIED + unmasked + side-effect) | 3, 8 + init→os, rh-proofs→zeta + vigilance→guardian | — | TBD | **≤5 after Phase 4** |

### Governance Exception Register

| Exception ID | DV | Source → Target | Layer Pair | Approved | Rationale |
|-------------|-----|----------------|-----------|----------|-----------|
| EX-DV-002 | 2 | nexcore-constants → nexcore-fs | Foundation → Domain | 2026-02-28 | Single-function boundary crossing for platform directory resolution. Narrow, stable, no extraction benefit. |
| EX-DV-007 | 7 | nexcore-pharos → nexcore-guardian-engine | Domain → Orchestration | 2026-02-28 | Architecturally intentional: PHAROS observes guardian threat signals by design. Narrow (3 types, 1 submodule). |

---

## Phase 1-3 Remediation Results (2026-02-28)

**Context:** Executed dead dep removal + hold reclassifications. Phase 4 (type extraction) deferred.

### Phase 1 — Dead Dependency Removal

Of 5 DVs classified as FIXABLE-REMOVE, only **3 were actually dead**. 2 were misclassified by the audit agent:

| DV | Source Crate | Dep | Classified | Actual | Action |
|----|-------------|-----|-----------|--------|--------|
| 1 | nexcore-compositor | nexcore-os | Dead | **Dead** | Removed |
| 3 | nexcore-core | nexcore-brain | Dead | **Live** (lib.rs:82,87) | Restored |
| 5 | nexcore-init | nexcore-compositor | Dead | **Dead** | Removed |
| 8 | nexcore-renderer | prima | Dead | **Live** (adventure.rs:60,67,69) | Restored |
| 9 | nexcore-rh-proofs | nexcore-tov-proofs | Dead | **Dead** | Removed |

**Root cause of misclassification:** The Explore agent's grep did not find `use nexcore_brain` / `use prima` imports because these crates are referenced via fully-qualified paths (`nexcore_brain::BrainSession`, `prima::eval`) rather than top-level `use` imports.

### Phase 2 — pv-core Reclassification

Changed `pv-core.toml` layer from Orchestration to Domain.

**Resolved:** DV4 (faers-etl→vigilance), DV11 (watch-core→pvos) — now Domain→Domain (no violation).
**Created:** 1 new DV: nexcore-vigilance (pv-core/Domain) → nexcore-guardian-engine (guardian-system/Orchestration). Also +1 LV: nexcore-vigilance depth 29 exceeds Domain layer threshold.

### Phase 3 — nexcore-state-theory Move

Moved nexcore-state-theory from os-runtime to analysis-tools.

**Resolved:** DV6 (model-checker→state-theory) — now same hold (no violation).

### Stale Hold Cleanup

Deleted `ferro-forge/holds/biological-system.toml` — stale pre-split file. The `regenerate_bay_from_holds` test failed with "crate 'nexcore-cytokine' in both 'bio-molecular' and 'biological-system'" because the old hold file conflicted with the new bio-molecular/bio-anatomical split. The `generate_ferro_forge` test (bootstrap path) had masked this orphan by overwriting all hold files from topology JSON. Reactive deletion after test failure.

### Reconciliation Comparison

| Metric | Pre-Phase 1-3 | Post-Phase 1-3 | Delta |
|--------|--------------|---------------|-------|
| Total actions | 17 | 16 | -1 |
| DirectionViolation | 11 | 8 | -3 |
| LayerViolation | 6 | 7 | +1 |
| OrphanCrate | 0 | 0 | 0 |
| SuggestMove | 0 | 0 | 0 |

### Current DV Inventory (8 remaining)

| # | Source Crate | Source Hold (Layer) | Target Crate | Target Hold (Layer) | Classification |
|---|-------------|-------------------|-------------|-------------------|---------------|
| 1 | nexcore-constants | core-primitives (Foundation) | nexcore-fs | system-utilities (Domain) | ACCEPT (EX-DV-002) |
| 2 | nexcore-core | core-primitives (Foundation) | nexcore-brain | brain-knowledge (Domain) | FIXABLE-RECLASSIFY |
| 3 | nexcore-init | core-primitives (Foundation) | nexcore-os | os-runtime (Orchestration) | FIXABLE-RECLASSIFY |
| 4 | nexcore-pharos | observability (Domain) | nexcore-guardian-engine | guardian-system (Orchestration) | ACCEPT (EX-DV-007) → upgrades to non-DV if DV8 applied |
| 5 | nexcore-renderer | observatory-viz (Foundation) | prima | prima-language (Domain) | FIXABLE-RECLASSIFY |
| 6 | nexcore-rh-proofs | core-primitives (Foundation) | nexcore-zeta | analysis-tools (Domain) | FIXABLE-RECLASSIFY |
| 7 | nexcore-value-mining | business-strategy (Domain) | nexcore-social | mcp-service (Service) | FIXABLE-EXTRACT |
| 8 | nexcore-vigilance | pv-core (Domain) | nexcore-guardian-engine | guardian-system (Orchestration) | FIXABLE-RECLASSIFY |

### DV Reclassification Impact Models (2026-02-28)

All 6 non-ACCEPT DVs verified LIVE by `cargo check` (remove dep → compilation failure → restore). Zero FIXABLE-REMOVE candidates.

#### DV2: nexcore-core → nexcore-brain — FIXABLE-RECLASSIFY

**Action:** Move `nexcore-core` from `core-primitives` to `pv-core` (Domain).

**Source usage:** `src/lib.rs` — 1 function (`persist_as_artifact`), 2 call sites: `nexcore_brain::BrainSession` (line 82), `nexcore_brain::Artifact::new` (line 87). Highly concentrated.

**Consumer analysis:** 0 downstream consumers (leaf crate).

**Impact model:**
- core→brain becomes Domain→Domain (same layer, not a DV) ✓
- nexcore-core also depends on: nexcore-vigilance (pv-core/Domain), nexcore-lex-primitiva (core-primitives/Foundation), nexcore-id (core-primitives/Foundation), duckdb (external). All Domain→Domain or Domain→Foundation = downward. ✓
- **Predicted new DVs: 0**
- **Side benefit:** Also resolves LayerViolation (nexcore-core has >3 internal deps, violates Foundation threshold).

#### DV3: nexcore-init → nexcore-os — FIXABLE-RECLASSIFY

**Action:** Move `nexcore-init` from `core-primitives` to `os-runtime` (Orchestration).

**Source usage:** `src/main.rs` — 3 call sites: `NexCoreOs::boot()` (line 158), `.tick()` (line 194), `.shutdown()` (lines 212-214). OS lifecycle management.

**Consumer analysis:** 0 downstream consumers (leaf binary crate).

**Impact model:**
- init→os becomes Orchestration→Orchestration (same layer) ✓
- nexcore-init also depends on: nexcore-pal + nexcore-pal-linux (system-utilities/Domain), nexcore-shell (os-runtime/Orchestration), nexcore-lex-primitiva (core-primitives/Foundation). All Orchestration→Domain, Orchestration→Orchestration, or Orchestration→Foundation = downward. ✓
- **Predicted new DVs: 0**
- **Side benefit:** Also resolves LayerViolation (nexcore-init has >3 internal deps, violates Foundation threshold).

#### DV5: nexcore-renderer → prima — FIXABLE-RECLASSIFY

**Action:** Reclassify `observatory-viz` hold from Foundation to Domain.

**Source usage:** `src/adventure.rs` — 2 operational sites: `prima::eval` (line 60), `prima::Value`/`prima::value::ValueData` (lines 67, 69). Template evaluation for nex:// pages.

**Consumer analysis:** observatory-viz has 5 members: nexcloud, nexcore-browser, nexcore-renderer, nexcore-softrender, nexcore-viz. Reclassifying the hold affects all 5.

**Impact model:**
- renderer→prima becomes Domain→Domain (same layer) ✓
- Other observatory-viz members: nexcore-browser depends on nexcore-lex-primitiva (Foundation), nexcore-softrender has minimal deps, nexcore-viz depends on foundation types. All Domain→Foundation = downward. ✓
- observatory-viz members' consumers: nexcore-vigilance (pv-core/Domain) depends on nexcore-browser — Domain→Domain = OK. ✓
- **Predicted new DVs: 0**
- **Rationale:** observatory-viz contains rendering, visualization, and browser infrastructure — Domain-layer work, not Foundation primitives. The "Foundation" classification was an artifact of name inference, not architectural intent.

#### DV6: nexcore-rh-proofs → nexcore-zeta — FIXABLE-RECLASSIFY

**Action:** Move `nexcore-rh-proofs` from `core-primitives` to `analysis-tools` (Domain).

**Source usage:** `src/bridge.rs` — 3-4 call sites: `convert_rh_verification()`, `convert_zeros()`, `build_certificate_from_zeta()`, `RhVerification`/`ZetaZero` types. Highly concentrated bridge pattern.

**Consumer analysis:** 0 downstream consumers (leaf crate).

**Impact model:**
- rh-proofs→zeta becomes Domain→Domain (both in analysis-tools, same hold) ✓
- nexcore-rh-proofs also depends on: nexcore-error (workspace), stem-number-theory (stem-foundation/Foundation), nexcore-lex-primitiva (core-primitives/Foundation), serde (external). All Domain→Foundation = downward. ✓
- **Predicted new DVs: 0**
- **Side benefit:** Also resolves LayerViolation (nexcore-rh-proofs in Foundation hold but depends on analysis-tools/Domain crate).

#### DV7: nexcore-value-mining → nexcore-social — FIXABLE-EXTRACT

**Action:** Extract `Post` + `SocialError` types from `nexcore-social` into new `nexcore-social-types` crate. Place `nexcore-social-types` in `business-strategy` hold (Domain).

**Source usage:** 7 call sites across 5 detector modules (`engagement.rs`, `trend.rs`, `virality.rs`, `controversy.rs`, `sentiment.rs`) + `error.rs` + `types.rs`. The `Post` type is pervasive — used in every detector's analysis function signature. Highly dispersed.

**Types to extract:**
- `Post` (core social media data structure)
- `SocialError` (error type used in `From` impl)
- Associated enums/constants referenced by the above

**Precedent:** Follows the `nexcore-hormone-types` extraction pattern from the bio-remediation arc.

**Impact model:**
- value-mining→social-types becomes Domain→Domain (same hold) ✓
- nexcore-social (mcp-service/Service) retains `Post` implementation but re-exports from social-types
- **Predicted new DVs: 0**
- **Consumer count:** 1 (nexcore-value-mining is the only non-service consumer of `Post`)

#### DV8: nexcore-vigilance → nexcore-guardian-engine — FIXABLE-RECLASSIFY

**Action:** Move `nexcore-guardian-engine` from `guardian-system` (Orchestration) to `pv-core` (Domain).

**Source usage:** `src/guardian/mod.rs` line 6 — single facade re-export: `pub use nexcore_guardian_engine::*;`. Minimal/facade-only integration.

**Consumer analysis:** nexcore-guardian-engine has 4 consumers:
1. nexcore-vigilance (pv-core/Domain) — the DV8 source
2. nexcore-pharos (observability/Domain) — currently DV4 (ACCEPT EX-DV-007)
3. nexcore-mcp (mcp-service/Service)
4. nexcore-guardian-cli (guardian-system/Orchestration)

**Impact model:**
- vigilance→guardian-engine becomes Domain→Domain (both in pv-core) ✓
- pharos→guardian-engine becomes Domain→Domain — **DV4 resolves automatically** ✓
- mcp→guardian-engine becomes Service→Domain = downward ✓
- guardian-cli→guardian-engine: guardian-cli is in guardian-system (Orchestration), guardian-engine moves to pv-core (Domain). Orchestration→Domain = downward ✓
- **Predicted new DVs: 0**
- **Side benefit:** DV4 (currently ACCEPT EX-DV-007) upgrades to non-DV. Net DV reduction: 2 (DV8 + DV4).

### Projected DV Inventory After All Fixes

| # | DV | Fix | Status |
|---|-----|-----|--------|
| 1 | nexcore-constants → nexcore-fs | ACCEPT (EX-DV-002) | Remains (1 of 2 final DVs) |
| 2 | nexcore-core → nexcore-brain | Move core to pv-core | **Resolved** |
| 3 | nexcore-init → nexcore-os | Move init to os-runtime | **Resolved** |
| 4 | nexcore-pharos → nexcore-guardian-engine | Cascade from DV8 fix | **Resolved** |
| 5 | nexcore-renderer → prima | Reclassify observatory-viz to Domain | **Resolved** |
| 6 | nexcore-rh-proofs → nexcore-zeta | Move rh-proofs to analysis-tools | **Resolved** |
| 7 | nexcore-value-mining → nexcore-social | Extract nexcore-social-types | **Resolved** |
| 8 | nexcore-vigilance → nexcore-guardian-engine | Move guardian-engine to pv-core | **Resolved** |

**Projected final state:** 1 DV (down from 8). The sole remaining DV (nexcore-constants → nexcore-fs) has an accepted exemption (EX-DV-002).

### Lessons

1. **Audit accuracy**: Agent-based grep audits for "dead deps" must search for fully-qualified paths (`crate::Type`), not just `use crate` imports. 2 of 5 classified dead deps were live.
2. **Layer reclassification creates new DVs**: Downgrading pv-core from Orchestration to Domain made nexcore-vigilance→guardian-engine visible as a DV. The dep was always there — the previous same-layer classification masked it.
3. **Bootstrap vs manifest path divergence**: The `generate_ferro_forge` test (bootstrap path) overwrites ALL hold TOMLs from workspace-topology.json. Must use `regenerate_bay_from_holds` (manifest path) when hold TOMLs are the source of truth.
4. **Stale hold files**: The biological-system.toml was never deleted after the bio-remediation split. Bootstrap path masked the issue; manifest path correctly detected it.

---

## Recommendations

1. **Foundation gravity filter — DONE (P8)**: Directional filter eliminates all 59 false positives. SuggestMove count: 0.

2. **Dead dependency cleanup (Phase 1)**: 5 of 11 DVs are caused by unused Cargo.toml dependencies with zero source imports. Removing them is the highest-ROI topology fix: 5 DVs eliminated with 5 line deletions, zero code changes.

3. **Hold reclassification (Phases 2-3)**: pv-core is classified Orchestration but contains domain logic (PV signal detection algorithms, types, parsers). Reclassifying to Domain fixes 2 DVs. Moving nexcore-state-theory from os-runtime to analysis-tools fixes 1 more.

4. **Type extraction (Phase 4)**: Extract `Post` domain type from nexcore-social into nexcore-social-types, following the nexcore-hormone-types pattern. Fixes the Domain → Service DV.

5. **LayerViolation remediation**: 6 crates in `core-primitives`/`stem-foundation` exceed the Foundation dep threshold (≤3). These are real violations surfaced by correcting layer assignments. Fix: either move high-dep members to domain holds, or increase the Foundation threshold.

6. **nexcore-chemivigilance SPLIT**: Consider introducing a bridge-crate pattern or dual-hold membership for crates that legitimately span two domains.
