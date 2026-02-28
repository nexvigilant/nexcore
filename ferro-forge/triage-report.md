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

## Remaining Issues (Post-P8)

### DirectionViolations (17 bootstrap / 12 manifest)

Post-P8 direction violations on the manifest path (the authoritative source).
Items 11-12 (stem/stem-bio → biological-system) were resolved by the bio-remediation arc (nexcore-hormone-types extraction, commit 555f4bbb). See "Post Bio-Remediation Baseline" section for current counts.

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

**Root cause:** Chemistry hold is declared Foundation-layer, but 4 chemistry crates depend on `prima-chem` (prima-language hold, Domain-layer). Similarly, `nexcore-renderer` depends on `prima` (prima-language hold). These are pre-existing layer mismatches exposed by the hold topology changes.

**Potential fixes:**
- Reclassify chemistry hold from Foundation to Domain (aligns with actual dependency depth)
- Or reclassify prima-language hold from Domain to Foundation (if prima-chem has few deps)

### Orphan Changes

| Crate | P8 Status | Current Status | Cause |
|-------|-----------|---------------|-------|
| nexcore-pty | Not in workspace | Orphan (both paths) | New crate, not assigned to any hold |
| nexcore-topology | In build-tooling | Orphan (both paths) | Hold assignment lost during bay.toml regeneration |
| nexcore-hormone-types | N/A | Orphan (bootstrap only) | Added to stem-foundation.toml manifest but bootstrap topology not regenerated |

### Net Assessment

The bio-remediation arc achieved its primary objective: stem-foundation → bio-molecular DVs eliminated. The manifest total rose from 18→23 due to 5 previously-masked chemistry→prima-language DVs surfacing — these are pre-existing architectural issues, not regressions. Two orphan crates need hold assignment (nexcore-pty, nexcore-topology re-assignment).

---

## Recommendations

1. **Foundation gravity filter — DONE (P8)**: Directional filter eliminates all 59 false positives. SuggestMove count: 0.

2. **DirectionViolation clustering**: The 12 manifest-path violations cluster around two root causes:
   - **Foundation crates with upward deps** (5): `nexcore-core`, `nexcore-init`, `nexcore-constants`, `nexcore-rh-proofs`, `stem`/`stem-bio` depend on higher-layer crates. Fix: move these to domain holds or restructure their dependencies.
   - **Orchestration holds consumed by Domain** (5): `pv-core` and `os-runtime` are Orchestration but consumed by Domain crates. Fix: downgrade to Domain layer.

3. **LayerViolation remediation**: 6 crates in `core-primitives`/`stem-foundation` exceed the Foundation dep threshold (≤3). These are real violations surfaced by correcting layer assignments. Fix: either move high-dep members to domain holds, or increase the Foundation threshold.

4. **nexcore-chemivigilance SPLIT**: Consider introducing a bridge-crate pattern or dual-hold membership for crates that legitimately span two domains.
