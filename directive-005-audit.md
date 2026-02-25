# Directive 005 — MCP Tool Visibility Audit

**Phase:** E (Examine)
**Date:** 2026-02-24
**Target:** `~/nexcore/crates/nexcore-mcp/src/unified.rs`
**Primitives:** ∃+N+κ+∂+μ

---

## Executive Summary

The MCP tool ecosystem has a **dual-catalog desync** problem. Two independent hardcoded JSON catalogs (`help_catalog()` and `help_catalog_json()`) serve different discovery surfaces but are not synchronized. The toolbox search — the primary discovery mechanism for AI agents — covers only **38% of categories** visible in the help catalog. An estimated 500+ tools are invisible to keyword search.

---

## Quantified Findings

### F1: Dual-Catalog Architecture (ROOT CAUSE)

| Surface | Function | Source Line | Categories | Purpose |
|---------|----------|-------------|------------|---------|
| `help` command | `help_catalog()` | unified.rs:2928 | **184** | User-facing full catalog |
| `toolbox` search | `help_catalog_json()` → `toolbox_catalog()` | unified.rs:2851 | **70** | AI agent keyword search |

**These are separate inline JSON objects.** `toolbox_catalog()` delegates to `help_catalog_json()` (line 2825), which is a DIFFERENT dataset than `help_catalog()`. Adding a tool to one does not add it to the other.

### F2: Toolbox Blindspot — 114 Missing Categories

**114 categories** (62%) exist in `help_catalog` but are invisible to `toolbox` search:

| Domain | Missing Categories | Est. Tools |
|--------|-------------------|------------|
| Bio systems | nervous, cardiovascular, lymphatic, respiratory, urinary, integumentary, reproductive, anatomy, nmd | ~37 |
| Visualization | viz, visual | ~33 |
| PV extended | pv_pipeline, pv_axioms, pv_embeddings, preemptive_pv, signal_pipeline, signal_theory, signal_fence, pharmacovigilance, retrocasting, fhir | ~58 |
| Brain/Learning | brain_db, learning, oracle, lessons, engram, cognition | ~49 |
| Forge/Dev | forge, academy_forge, validify, ctvp, code_inspect, primitive_coverage, compounding, polymer, prompt_kinetics, model_delegation | ~48 |
| Infrastructure | monitoring, clearance, secure_boot, sentinel, mcp_telemetry, mcp_lock, user, claude_fs, cortex | ~40 |
| Domain-specific | lex_primitiva, laboratory, cep, education, highway, domain_primitives, fda_credibility, registry, skills_engine | ~73 |
| Newer additions | zeta, combinatorics, word, harm_taxonomy, antibodies, jeopardy, audio, compilation_space, dna, statemind, ghost, insight, etc. | ~160 |

**Total estimated invisible tools: ~500+**

### F3: Reverse Blindspot — 7 Toolbox-Only Categories

7 categories exist in `toolbox` but are **missing from help**:

| Category | Tools | Impact |
|----------|-------|--------|
| `cargo` | 6 (cargo_check, cargo_build, etc.) | Searchable but unlisted |
| `rust_dev` | 12 | Searchable but unlisted |
| `chemivigilance` | 19 | Searchable but unlisted |
| `pk` | 6 | Searchable but unlisted |
| `causality` | 2 | Searchable but unlisted |
| `temporal` | 3 | Searchable but unlisted |
| `knowledge_engine` | 6 | Searchable but unlisted |

### F4: Within-Category Drift

Even categories present in BOTH catalogs have different tool counts:

| Category | help_catalog | toolbox | Delta | Missing from toolbox |
|----------|-------------|---------|-------|---------------------|
| guardian | 17 | 11 | -6 | guardian_subscribe, guardian_adversarial_input, adversarial_decision_probe, pv_control_loop_tick, fda_bridge_evaluate, fda_bridge_batch |
| wolfram | 19 | 16 | -3 | wolfram_query_with_assumption, wolfram_query_filtered, wolfram_image_result |
| brain | 24 | 23 | -1 | brain_verify_engrams |
| chemistry | 17 | 16 | -1 | chemistry_first_law_closed or chemistry_first_law_open |
| regulatory | 4 | 3 | -1 | regulatory_effectiveness_assess |
| stem | 25 | 24 | -1 | (1 tool missing) |

### F5: Stale Hardcoded Total

`help_catalog()` declares `"total": 496` (line 2930). The actual tool count across 184 categories is significantly higher. This value is manually maintained and has drifted.

### F6: Dispatch-to-Catalog Gap

| Metric | Count |
|--------|-------|
| Dispatch match arms (`dispatch_inner`) | **1,289** |
| help_catalog claimed total | 496 |
| toolbox categories | 70 |
| help categories | 184 |
| Tool source files (`src/tools/`) | 100+ |
| `pub fn` across tool files | 1,799 |

Not all 1,289 match arms are unique tools (some handle system commands like `help`, `toolbox`, aliases), but the gap between dispatch capacity and catalog coverage is substantial.

### F7: No Automated Catalog Validation

- No test verifies that every dispatch arm appears in a catalog
- No test verifies that catalog entries have matching dispatch arms
- No test verifies `help_catalog` and `help_catalog_json` are synchronized
- The `"total"` field is manually maintained, not computed

---

## Gap Inventory

| Gap ID | Description | Severity | Primitive Violated |
|--------|-------------|----------|--------------------|
| G1 | Dual catalog not synchronized | **Critical** | ∂ (Boundary) — two sources of truth |
| G2 | 114 categories invisible to toolbox search | **Critical** | ∃ (Existence) — tools exist but can't be found |
| G3 | 7 categories in toolbox but not in help | **High** | κ (Comparison) — asymmetric visibility |
| G4 | Within-category tool count drift | **High** | N (Quantity) — counts diverge |
| G5 | Stale hardcoded total | **Medium** | N (Quantity) — wrong count |
| G6 | No test coverage for catalog integrity | **High** | ∝ (Irreversibility) — drift accelerates without gates |
| G7 | Manual catalog maintenance does not scale | **Medium** | σ (Sequence) — process gap |

---

## Test Coverage Assessment

| Test Type | Coverage | Evidence |
|-----------|---------|---------|
| Dispatch ↔ Catalog sync | **0%** | No test exists |
| help ↔ toolbox sync | **0%** | No test exists |
| Total field accuracy | **0%** | Hardcoded, never computed |
| Category completeness | **0%** | No test exists |
| Tool name validity | **Unknown** | May exist in integration tests |

---

## Recommended Design Direction

### Single Source of Truth

Replace dual inline JSON with a single generated catalog:

```
dispatch_inner match arms → build.rs or macro → catalog.rs → both help() and toolbox() read from it
```

Alternatively, a `#[catalog("category")]` attribute on dispatch handlers that gets collected at compile time.

### Automated Validation

Add a test that:
1. Collects all dispatch match arm strings
2. Collects all help_catalog tool names
3. Collects all toolbox_catalog tool names
4. Asserts symmetric coverage (dispatch ⊆ catalog ∧ catalog ⊆ dispatch)
5. Asserts help = toolbox (or toolbox ⊇ help)
6. Computes and validates the total count

---

## Score

| Criterion | Points | Score | Evidence |
|-----------|--------|-------|---------|
| Findings count > 0 | 30 | **30** | 7 gaps identified |
| Quantified metrics present | 30 | **30** | Tool counts, category counts, percentages |
| Gap inventory complete | 20 | **20** | 7 gaps with severity and primitive |
| Test coverage measured | 20 | **20** | 0% across all catalog validation dimensions |

**E Phase Score: 100/100**

---

## GATE-E Decision

**PASS** — Score 100 >= 50 threshold. Proceed to D (Design) phase.
