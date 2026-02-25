# Directive 005 — MCP Tool Visibility — Results

**Pipeline:** E-D-I-C-T (all phases complete)
**Date:** 2026-02-24
**Status:** COMPLETE

---

## Phase Scores

| Phase | Name | Score | Gate |
|-------|------|-------|------|
| E | Examine | 100/100 | PASS |
| D | Design | 100/100 | PASS |
| I | Implement | 85/100 | PASS |
| C | Connect | 90/100 | PASS |
| T | Test | 95/100 | PASS |

**Composite:** E(15%) + D(20%) + I(30%) + C(20%) + T(15%) = 15 + 20 + 25.5 + 18 + 14.25 = **92.75/100**

---

## What Changed

### Root Cause Fixed
Two independent inline JSON catalogs (`help_catalog()` and `help_catalog_json()`) served different discovery surfaces without synchronization. Toolbox search only saw 70/184 categories (38%).

### Solution Implemented
Single source of truth: `unified_catalog_data()` function containing ALL categories, consumed by both `help_catalog()` and `toolbox_catalog()`.

### Files Modified

| File | Change |
|------|--------|
| `crates/nexcore-mcp/src/unified.rs` | Renamed `help_catalog_json()` → `unified_catalog_data()`, expanded from 70 → 192 categories, fixed within-category drift (guardian +6, wolfram +3, brain +1, regulatory +1, chemistry +2). Rewrote `help_catalog()` to delegate with computed total. Added `catalog_tests` module with 5 validation tests (C5+C6). |

### Metrics Before/After

| Metric | Before | After |
|--------|--------|-------|
| Toolbox categories | 70 | 192 |
| Help categories | 184 | 192 |
| Toolbox blindspot | 114 (62%) | 0 (0%) |
| Hardcoded total | "496" (stale) | Computed dynamically |
| Catalog data sources | 2 (desynced) | 1 (unified) |
| guardian tools in toolbox | 11 | 17 |
| wolfram tools in toolbox | 16 | 19 |

### Build Verification

| Check | Result |
|-------|--------|
| `cargo check -p nexcore-mcp` | PASS (0 errors) |
| `cargo build -p nexcore-mcp --release` | PASS (69MB binary) |
| `cargo test -p nexcore-mcp --lib` | 461/462 (1 pre-existing failure in lex_primitiva) + 5/5 new catalog_tests |
| `cargo clippy --no-deps` | 171 pre-existing warnings, 0 new |

---

## C5+C6: Catalog Validation Tests (COMPLETE)

5 tests added in `unified::catalog_tests`:

| Test | What It Validates | Result |
|------|-------------------|--------|
| `catalog_has_expected_categories` | >= 190 categories in unified catalog | PASS |
| `catalog_total_above_threshold` | > 400 total tools computed from data | PASS |
| `no_empty_categories` | Every category has at least one tool | PASS |
| `toolbox_and_help_share_source` | Both surfaces return identical categories | PASS |
| `dispatch_commands_in_catalog` | Dispatch match arms appear in catalog (>50% threshold) | PASS |

`dispatch_commands_from_source()` uses `include_str!` to parse match arm patterns from the source file — self-maintaining, no manual list.

---

## Remaining Work

1. ~~**Dispatch-to-catalog alignment test** (C5+C6 from design)~~ — DONE (5 tests, all pass).
2. **MCP server restart** — required for changes to take effect in live sessions. Release binary rebuilt.
3. **Pre-existing test fix** — `test_known_types_resolution_rate` fails at 95% (35 unresolved Cloud types). Unrelated to this directive.

---

## Deliverables

| File | Purpose |
|------|---------|
| `directive-005-audit.md` | E phase findings (7 gaps, quantified) |
| `directive-005-design.md` | D phase solution design (6 changes, dep map) |
| `directive-005-results.md` | This file — final summary |
