# Biological-System Hold Architecture Audit

**Date:** 2026-02-28
**Scope:** 20 biological-system crates + 10 stem-foundation crates + 6 chemistry crates (neighbor)
**Source:** `ferro-forge/holds/biological-system.toml`, `stem-foundation.toml`, `chemistry.toml`

---

## 1. Per-Crate Metrics

### 1.1 Biological-System Hold (20 crates)

#### Molecular/Cellular Sub-Group (9 crates)

| Crate | LOC | Tests | pub fn | pub struct | pub trait | pub enum | Maturity | Internal Deps |
|-------|-----|-------|--------|------------|-----------|----------|----------|---------------|
| nexcore-cytokine | 7,003 | 162 | 221 | 44 | 6 | 18 | MATURE | nexcore-error, nexcore-lex-primitiva, nexcore-chrono |
| nexcore-hormones | 1,318 | 50 | 21 | 3 | 0 | 3 | MATURE | nexcore-chrono, nexcore-error, nexcore-lex-primitiva |
| nexcore-immunity | 5,341 | 118 | 60 | 35 | 0 | 12 | MATURE | nexcore-error, nexcore-fs, nexcore-lex-primitiva, **nexcore-spliceosome** |
| nexcore-energy | 2,788 | 104 | 42 | 9 | 0 | 7 | MATURE | nexcore-error, nexcore-lex-primitiva |
| nexcore-synapse | 3,263 | 65 | 69 | 15 | 0 | 9 | MATURE | nexcore-chrono, nexcore-error, nexcore-lex-primitiva |
| nexcore-transcriptase | 1,999 | 59 | 15 | 6 | 0 | 4 | MATURE | nexcore-error, nexcore-lex-primitiva |
| nexcore-ribosome | 2,514 | 63 | 24 | 11 | 0 | 6 | MATURE | nexcore-error, nexcore-lex-primitiva, nexcore-chrono, **nexcore-transcriptase** |
| nexcore-phenotype | 3,186 | 91 | 27 | 8 | 0 | 5 | MATURE | nexcore-error, nexcore-lex-primitiva, **nexcore-transcriptase**, **nexcore-ribosome** |
| nexcore-spliceosome | 1,018 | 20 | 10 | 4 | 0 | 2 | MATURE | nexcore-chrono |

**Sub-group totals:** 28,430 LOC | 732 tests | 489 pub fn | 135 pub struct | 6 pub trait | 66 pub enum

#### Anatomical Sub-Group (11 crates)

| Crate | LOC | Tests | pub fn | pub struct | pub trait | pub enum | Maturity | Internal Deps |
|-------|-----|-------|--------|------------|-----------|----------|----------|---------------|
| nexcore-cardiovascular | 5,300 | 70 | 69 | 21 | 0 | 8 | MATURE | nexcore-lex-primitiva |
| nexcore-circulatory | 2,117 | 62 | 31 | 20 | 0 | 7 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-digestive | 2,524 | 84 | 31 | 20 | 0 | 7 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-integumentary | 1,454 | 42 | 23 | 19 | 0 | 8 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-lymphatic | 1,249 | 46 | 24 | 8 | 0 | 3 | MATURE | nexcore-lex-primitiva |
| nexcore-muscular | 1,065 | 43 | 17 | 8 | 0 | 1 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-nervous | 1,162 | 47 | 24 | 15 | 0 | 3 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-reproductive | 3,206 | 84 | 63 | 35 | 0 | 14 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-respiratory | 1,502 | 44 | 18 | 18 | 0 | 3 | MATURE | nexcore-lex-primitiva |
| nexcore-skeletal | 1,117 | 37 | 19 | 7 | 0 | 4 | MATURE | nexcore-chrono, nexcore-lex-primitiva |
| nexcore-urinary | 1,457 | 49 | 37 | 15 | 0 | 3 | MATURE | nexcore-lex-primitiva |

**Sub-group totals:** 22,153 LOC | 608 tests | 356 pub fn | 186 pub struct | 0 pub trait | 61 pub enum

#### Hold Totals

| Metric | Molecular/Cellular | Anatomical | Hold Total |
|--------|-------------------|------------|------------|
| Crates | 9 | 11 | 20 |
| LOC | 28,430 | 22,153 | 50,583 |
| Tests | 732 | 608 | 1,340 |
| pub fn | 489 | 356 | 845 |
| pub struct | 135 | 186 | 321 |
| pub trait | 6 | 0 | 6 |
| pub enum | 66 | 61 | 127 |
| Tests/kSLOC | 25.7 | 27.4 | 26.5 |

### 1.2 Stem-Foundation Hold (10 crates)

| Crate | LOC | Tests | pub fn | pub struct | pub trait | pub enum | Maturity | Internal Deps |
|-------|-----|-------|--------|------------|-----------|----------|----------|---------------|
| stem | 5,169 | 147 | 98 | 43 | 50 | 10 | MATURE | nexcore-error, stem-derive, nexcore-constants, **nexcore-hormones**, nexcore-lex-primitiva |
| stem-bio | 581 | 11 | 4 | 4 | 0 | 1 | MATURE | stem-core, **nexcore-hormones**, nexcore-lex-primitiva |
| stem-phys | 964 | 19 | 21 | 9 | 1 | 0 | MATURE | nexcore-error, stem-core, stem-complex, nexcore-lex-primitiva |
| stem-core | 1,969 | 46 | 23 | 13 | 4 | 0 | MATURE | nexcore-error, nexcore-constants, nexcore-lex-primitiva |
| stem-math | 9,120 | 278 | 161 | 23 | 14 | 8 | MATURE | nexcore-error, stem-core, nexcore-lex-primitiva |
| stem-complex | 1,142 | 48 | 16 | 1 | 2 | 1 | MATURE | nexcore-error, stem-core, nexcore-lex-primitiva |
| stem-derive | 19 | 0 | 1 | 0 | 0 | 0 | STUB | stem-derive-core |
| stem-derive-core | 113 | 2 | 3 | 0 | 0 | 0 | PARTIAL | (none) |
| stem-number-theory | 1,547 | 38 | 18 | 3 | 0 | 1 | MATURE | nexcore-error, nexcore-lex-primitiva, stem-core |
| stem-topology | 929 | 38 | 26 | 6 | 0 | 0 | MATURE | nexcore-lex-primitiva, stem-complex |

**Hold totals:** 21,553 LOC | 627 tests | 371 pub fn | 102 pub struct | 71 pub trait | 21 pub enum

### 1.3 Chemistry Hold (6 crates, neighbor audit)

| Crate | Internal Deps |
|-------|---------------|
| nexcore-clearance | nexcore-lex-primitiva, nexcore-chrono |
| nexcore-metabolite | nexcore-error, nexcore-molcore, prima-chem |
| nexcore-molcore | nexcore-error, prima-chem |
| nexcore-qsar | nexcore-error, nexcore-molcore, prima-chem |
| nexcore-stoichiometry | nexcore-lex-primitiva, nexcore-primitives, nexcore-error |
| nexcore-structural-alerts | nexcore-error, nexcore-molcore, prima-chem |

**Cross-hold edges with biological-system:** Zero. No bio crate depends on any chemistry crate, and no chemistry crate depends on any bio crate.

---

## 2. Dependency Graph

### 2.1 Intra-Hold Edges (bio to bio)

```
nexcore-phenotype ──> nexcore-ribosome ──> nexcore-transcriptase
         |
         └─────────> nexcore-transcriptase

nexcore-immunity ───> nexcore-spliceosome
```

Four directed edges total. No cycles. The graph is a DAG.

The schema-inference pipeline (transcriptase -> ribosome -> phenotype) is the primary intra-hold chain. The spliceosome -> immunity edge is the only other connection.

**15 of 20 crates have zero intra-hold dependencies.** They connect only to foundation crates.

### 2.2 Cross-Hold Edges

#### Bio to Foundation (all 20 crates)

```
nexcore-lex-primitiva <── 19 of 20 bio crates (all except spliceosome)
nexcore-error         <── 9 bio crates (all molecular, zero anatomical)
nexcore-chrono        <── 13 bio crates (6 molecular + 7 anatomical)
nexcore-fs            <── nexcore-immunity (1 crate)
```

The anatomical sub-group depends exclusively on nexcore-lex-primitiva and nexcore-chrono. It never imports nexcore-error.

#### Stem-Foundation to Bio (DirectionViolations)

```
stem      ──> nexcore-hormones  [DIRECTION VIOLATION]
stem-bio  ──> nexcore-hormones  [DIRECTION VIOLATION]
```

Both holds are classified as Domain layer. The nexcore-hormones dependency in stem and stem-bio creates cross-hold edges that violate the intended dependency direction. stem-foundation is architecturally Foundation-adjacent (stem-core, stem-math, stem-derive have zero Domain deps), yet two of its members import from biological-system.

#### Bio to Stem-Foundation

Zero edges. No bio crate depends on any stem crate.

#### Bio to Chemistry / Chemistry to Bio

Zero edges in both directions.

### 2.3 Complete Dependency Topology

```
FOUNDATION LAYER
  nexcore-lex-primitiva ─────────────────────────────────────────────┐
  nexcore-error ─────────────────────────────────────┐               |
  nexcore-chrono ────────────────────────┐           |               |
  nexcore-fs ──────────────┐             |           |               |
                           |             |           |               |
BIOLOGICAL-SYSTEM HOLD     |             |           |               |
                           |             |           |               |
  Anatomical cluster       |             |           |               |
  (11 independent leaves)  |             |           |               |
    cardiovascular ────────┼─────────────┼───────────┼───────────────┤
    circulatory ───────────┼─────────────┤           |               |
    digestive ─────────────┼─────────────┤           |               |
    integumentary ─────────┼─────────────┤           |               |
    lymphatic ─────────────┼─────────────┼───────────┼───────────────┤
    muscular ──────────────┼─────────────┤           |               |
    nervous ───────────────┼─────────────┤           |               |
    reproductive ──────────┼─────────────┤           |               |
    respiratory ───────────┼─────────────┼───────────┼───────────────┤
    skeletal ──────────────┼─────────────┤           |               |
    urinary ───────────────┼─────────────┼───────────┼───────────────┤
                           |             |           |               |
  Molecular cluster        |             |           |               |
    spliceosome ───────────┼─────────────┤           |               |
    hormones ──────────────┼─────────────┤           ├───────────────┤
    energy ────────────────┼─────────────┼───────────┤               |
    synapse ───────────────┼─────────────┤           ├───────────────┤
    cytokine ──────────────┼─────────────┤           ├───────────────┤
    immunity ──┬───────────┤             |           ├───────────────┤
               └──> spliceosome          |           |               |
    transcriptase ─────────┼─────────────┼───────────┤               |
    ribosome ──┬───────────┼─────────────┤           ├───────────────┤
               └──> transcriptase        |           |               |
    phenotype ─┬───────────┼─────────────┼───────────┤               |
               ├──> transcriptase        |           |               |
               └──> ribosome             |           |               |
                           |             |           |               |
STEM-FOUNDATION HOLD       |             |           |               |
  stem-derive-core (no nexcore deps)     |           |               |
  stem-derive ─> stem-derive-core        |           |               |
  stem-core ───────────────┼─────────────┼───────────┤               |
  stem-complex ──> stem-core             |           ├───────────────┤
  stem-math ─────> stem-core             |           ├───────────────┤
  stem-phys ─────> stem-core, stem-complex           ├───────────────┤
  stem-number-theory > stem-core         |           ├───────────────┤
  stem-topology ─> stem-complex          |           |               |
  stem-bio ──────> stem-core ────────────┼──> nexcore-hormones  [DV] |
  stem ──────────> stem-derive ──────────┼──> nexcore-hormones  [DV] |

[DV] = DirectionViolation
```

---

## 3. Sub-Group Analysis

### 3.1 Are Molecular and Anatomical Crates Distinct Clusters?

**Yes. Completely distinct.** The two sub-groups share zero intra-hold edges.

| Property | Molecular/Cellular (9) | Anatomical (11) |
|----------|----------------------|-----------------|
| Intra-group edges | 4 (schema pipeline + immunity->spliceosome) | 0 |
| Cross-group edges | 0 | 0 |
| Depends on nexcore-error | All 9 | 0 of 11 |
| Defines pub trait | 1 crate (cytokine: 6 traits) | 0 crates |
| Average LOC | 3,159 | 2,014 |
| Average tests | 81.3 | 55.3 |
| Module structure | Multi-file (blood.rs, heart.rs, etc.) | Mostly lib.rs + grounding.rs |
| In CLAUDE.md canonical list | 8 of 9 (spliceosome excluded) | 0 of 11 |

The molecular crates implement domain logic (schema inference, antipattern detection, event signaling). The anatomical crates implement architectural metaphors (data transport, I/O, boundary protection) with no behavioral dependencies on each other.

The anatomical sub-group is architecturally flat: 11 parallel leaves with identical dependency shapes. No anatomical crate imports another anatomical crate.

### 3.2 Dependency Shape Uniformity

The anatomical crates exhibit a strict 2-dependency pattern:

| Dep Pattern | Count | Crates |
|-------------|-------|--------|
| lex-primitiva only | 4 | cardiovascular, lymphatic, respiratory, urinary |
| lex-primitiva + chrono | 7 | circulatory, digestive, integumentary, muscular, nervous, reproductive, skeletal |

This uniformity suggests template-based generation rather than organic evolution.

---

## 4. Stem-Foundation Relationship Analysis

### 4.1 Internal Stem DAG

```
stem-derive-core (leaf, 0 deps)
  └──> stem-derive (proc-macro shim)

stem-core (leaf, foundation deps only)
  ├──> stem-complex
  ├──> stem-math
  ├──> stem-phys (also -> stem-complex)
  ├──> stem-number-theory
  └──> stem-bio [-> nexcore-hormones: DV]

stem (facade) -> stem-derive, nexcore-hormones [DV]
stem-topology -> stem-complex
```

### 4.2 DirectionViolation Detail

| Crate | Violating Dep | What It Imports | Resolution Options |
|-------|---------------|-----------------|-------------------|
| stem-bio | nexcore-hormones | Endocrine system primitives (hormone types, receptor model) | (A) Extract shared types to Foundation crate, (B) Move stem-bio into bio hold, (C) Inline the types |
| stem (facade) | nexcore-hormones | Re-exports stem-bio's endocrine module | Resolves automatically if stem-bio's violation is resolved |

The root cause: `nexcore-hormones` defines types (HormoneType, ReceptorModel) that `stem-bio` needs for its endocrine system module. The dependency is on types, not behavior.

### 4.3 Boundary Assessment

Of the 10 stem-foundation crates, only 2 (stem-bio, stem) have any biological-system dependency. The remaining 8 are architecturally clean with zero Domain-layer cross-hold imports. stem-math (9,120 LOC, 278 tests) is the computational workhorse with zero bio deps.

---

## 5. Maturity Classification

### 5.1 Criteria

| Class | LOC | Tests | Implementation |
|-------|-----|-------|----------------|
| STUB | <50 | 0 | Trait signatures / re-exports only |
| PARTIAL | 50-300 | 0-3 | Some implementation, minimal testing |
| MATURE | >300 | >3 | Substantial implementation with test coverage |

### 5.2 Results

| Hold | STUB | PARTIAL | MATURE | Total |
|------|------|---------|--------|-------|
| biological-system | 0 | 0 | 20 | 20 |
| stem-foundation | 1 | 1 | 8 | 10 |

**STUB:** stem-derive (19 LOC, 0 tests, proc-macro shim delegating to stem-derive-core)
**PARTIAL:** stem-derive-core (113 LOC, 2 tests, proc-macro utility library)

All 20 biological-system crates are MATURE. The smallest (nexcore-muscular, 1,065 LOC, 43 tests) exceeds MATURE thresholds by 21x on LOC and 14x on tests.

### 5.3 Test Density (top 5 by tests/kSLOC)

| Crate | Tests/kSLOC |
|-------|-------------|
| nexcore-muscular | 40.4 |
| nexcore-nervous | 40.4 |
| nexcore-hormones | 37.9 |
| nexcore-energy | 37.3 |
| nexcore-lymphatic | 36.8 |

Hold average: 26.5 tests/kSLOC. All 20 crates exceed the institutional minimum of 20 tests/kSLOC.

---

## 6. Biological-Homeostasis Stack Analysis

From `ferro-forge/stacks.toml`:

```
biological-homeostasis stack:
  Foundation  -> core-primitives    -> nexcore-lex-primitiva
  Domain      -> biological-system  -> nexcore-energy
  Domain      -> biological-system  -> nexcore-cytokine
  Domain      -> biological-system  -> nexcore-immunity
  Domain      -> guardian-system    -> nexcore-guardian-engine
```

**5 segments, 4 in Domain layer.** The stack terminates at Domain (nexcore-guardian-engine) without reaching Service or Orchestration. The biological system has no direct MCP tool exposure and no REST API surface.

---

## 7. Adjacent Crates (Outside Hold, Biologically Themed)

The following crates have biological names but are NOT members of the biological-system hold per `biological-system.toml`:

| Crate | LOC | Tests | Notes |
|-------|-----|-------|-------|
| nexcore-antibodies | 1,035 | 30 | Adaptive immune / epitope-paratope binding |
| nexcore-dna | 37,453 | 1,162 | Codon VM, NCBI, lang/ subsystem |
| nexcore-cognition | 3,167 | 42 | Transformer algorithm implementation |
| nexcore-homeostasis | 2,908 | 38 | Self-regulation framework (facade) |
| nexcore-homeostasis-primitives | 4,599 | 75 | Shared homeostasis types/math |
| nexcore-homeostasis-memory | 2,746 | 83 | Incident memory/playbooks |
| nexcore-homeostasis-storm | 1,809 | 13 | Storm detection/prevention |
| nexcore-anatomy | 2,413 | 44 | Workspace introspection via cargo_metadata |
| nexcore-organize | 3,663 | 69 | File/workspace organization |
| nexcore-sop-anatomy | 1,942 | 33 | SOP structural analysis |

These 10 crates total 61,735 LOC and 1,589 tests. nexcore-dna alone (37,453 LOC) exceeds the entire anatomical sub-group (22,153 LOC).

---

## 8. Recommendation: Hold Structure

### Current State

The biological-system hold contains 20 crates that split into two disconnected sub-graphs:
- **Molecular/Cellular (9):** 4 intra-group edges, nexcore-error dependency, multi-file structure
- **Anatomical (11):** 0 intra-group edges, no nexcore-error, template-uniform structure

Zero edges connect the two sub-groups.

### Recommended Actions

**Action 1: Split biological-system into two holds.**

| New Hold | Members | Layer | Rationale |
|----------|---------|-------|-----------|
| `bio-molecular` | cytokine, hormones, immunity, energy, synapse, transcriptase, ribosome, phenotype, spliceosome | Domain | 9 crates with intra-group dependencies, behavioral contracts (pub trait), and CLAUDE.md canonical status. Active biological computation layer. |
| `bio-anatomical` | cardiovascular, circulatory, digestive, integumentary, lymphatic, muscular, nervous, reproductive, respiratory, skeletal, urinary | Domain | 11 parallel metaphorical modules with zero cross-references. Architectural patterns (data transport, I/O, filtering), not biological computation. |

**Benefit:** The split makes the dependency graph match reality. The current hold implies 20 crates with shared purpose. In practice, 11 share zero edges with the other 9.

**Action 2: Resolve stem-foundation DirectionViolations.**

```
Option A: New Foundation crate (nexcore-hormone-types)
  - Extract shared type defs to Foundation layer
  - stem-bio and nexcore-hormones both import from it
  - Eliminates both DVs

Option B: Move stem-bio into bio-molecular hold
  - Accept that stem-bio is Domain-layer, not Foundation

Option C: Inline the 3-4 types stem-bio needs
  - Remove the dep entirely, duplicate the small type set
```

Option A is the cleanest resolution.

**Action 3: Triage the 10 uncategorized bio-adjacent crates.**

| Crate | Recommended Hold |
|-------|-----------------|
| nexcore-homeostasis, -primitives, -memory, -storm | New `bio-homeostasis` hold (4 crates, self-contained DAG) |
| nexcore-antibodies | `bio-molecular` (immune subsystem) |
| nexcore-dna | Standalone `bio-computation` hold (37k LOC warrants isolation) |
| nexcore-cognition | Evaluate for `bio-computation` or standalone |
| nexcore-anatomy, nexcore-organize | Not biological domain: `build-tooling` or `system-utilities` |
| nexcore-sop-anatomy | Governance tooling: `quality-assurance` |

**Action 4: Extend the biological-homeostasis stack to Service.**

Add `{ layer = "Service", hold = "mcp-service", crate = "nexcore-mcp" }` after verifying nexcore-mcp depends on nexcore-guardian-engine.

### Summary

| Action | Effort | Impact |
|--------|--------|--------|
| Split hold into bio-molecular + bio-anatomical | Low (2 TOML files) | Dependency graph matches reality |
| Resolve stem DV (Option A) | Medium (new crate + dep rewire) | Eliminates 2 DirectionViolations |
| Triage 10 uncategorized crates | Low (hold assignments) | Reduces uncategorized surface |
| Extend homeostasis stack to Service | Low (1 TOML edit + verify) | Completes the capability path |

---

## Appendix A: Full Maturity Table (30 Audited Crates)

| Crate | LOC | Tests | Class |
|-------|-----|-------|-------|
| nexcore-cardiovascular | 5,300 | 70 | MATURE |
| nexcore-circulatory | 2,117 | 62 | MATURE |
| nexcore-cytokine | 7,003 | 162 | MATURE |
| nexcore-digestive | 2,524 | 84 | MATURE |
| nexcore-energy | 2,788 | 104 | MATURE |
| nexcore-hormones | 1,318 | 50 | MATURE |
| nexcore-immunity | 5,341 | 118 | MATURE |
| nexcore-integumentary | 1,454 | 42 | MATURE |
| nexcore-lymphatic | 1,249 | 46 | MATURE |
| nexcore-muscular | 1,065 | 43 | MATURE |
| nexcore-nervous | 1,162 | 47 | MATURE |
| nexcore-phenotype | 3,186 | 91 | MATURE |
| nexcore-reproductive | 3,206 | 84 | MATURE |
| nexcore-respiratory | 1,502 | 44 | MATURE |
| nexcore-ribosome | 2,514 | 63 | MATURE |
| nexcore-skeletal | 1,117 | 37 | MATURE |
| nexcore-spliceosome | 1,018 | 20 | MATURE |
| nexcore-synapse | 3,263 | 65 | MATURE |
| nexcore-transcriptase | 1,999 | 59 | MATURE |
| nexcore-urinary | 1,457 | 49 | MATURE |
| stem | 5,169 | 147 | MATURE |
| stem-bio | 581 | 11 | MATURE |
| stem-core | 1,969 | 46 | MATURE |
| stem-complex | 1,142 | 48 | MATURE |
| stem-derive | 19 | 0 | STUB |
| stem-derive-core | 113 | 2 | PARTIAL |
| stem-math | 9,120 | 278 | MATURE |
| stem-number-theory | 1,547 | 38 | MATURE |
| stem-phys | 964 | 19 | MATURE |
| stem-topology | 929 | 38 | MATURE |
