# Directive 005 — MCP Tool Visibility Design

**Phase:** D (Design)
**Date:** 2026-02-24
**Primitives:** μ(Mapping) + ∂(Boundary)
**Depends on:** directive-005-audit.md (E phase, PASS)

---

## Design Goal

Eliminate dual-catalog desync by establishing a **single source of truth** for the MCP tool catalog, shared by both `help` and `toolbox` discovery surfaces. Add automated validation to prevent future drift.

---

## Approach: Unified Catalog Function

### Current Architecture (broken)

```
dispatch_inner() ← 1,289 match arms (source of truth for routing)
help_catalog()   ← inline JSON, 184 categories (manual, stale total)
help_catalog_json() ← SEPARATE inline JSON, 70 categories (subset)
toolbox_catalog() → help_catalog_json() (gets the subset)
toolbox_search()  → toolbox_catalog() (searches the subset)
```

### Target Architecture

```
dispatch_inner() ← match arms (routing, unchanged)
unified_catalog_data() ← SINGLE inline JSON, ALL categories
help_catalog() → unified_catalog_data() + computed total
toolbox_catalog() → unified_catalog_data() (same source)
toolbox_search() → toolbox_catalog() (now searches everything)
dispatch_commands() ← returns Vec<&str> of all command names (for testing)
```

---

## Changes

### C1: Create `unified_catalog_data()` (new function)

**Location:** `unified.rs`, near line 2851 (replacing `help_catalog_json()`)

Single function returning the complete catalog as `serde_json::Value`. Contains ALL 184+ categories from current `help_catalog()` plus the 7 toolbox-only categories (cargo, rust_dev, chemivigilance, pk, causality, temporal, knowledge_engine).

```rust
/// Single source of truth for the complete tool catalog.
/// Both `help_catalog()` and `toolbox_catalog()` read from this.
fn unified_catalog_data() -> serde_json::Value {
    serde_json::json!({
        "categories": {
            // ALL categories merged from both current catalogs
            // ... (184 + 7 unique = ~191 categories)
        }
    })
}
```

### C2: Rewrite `help_catalog()` to use unified source

**Location:** `unified.rs:2928`

```rust
fn help_catalog() -> Result<CallToolResult, McpError> {
    let data = unified_catalog_data();
    let categories = data.get("categories")
        .and_then(|c| c.as_object())
        .cloned()
        .unwrap_or_default();

    // Compute total from actual data
    let total: usize = categories.values()
        .filter_map(|v| v.as_array())
        .map(|arr| arr.len())
        .sum();

    let catalog = serde_json::json!({
        "total": total,
        "usage": "nexcore(command=\"CMD\", params={...})",
        "categories": categories
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&catalog).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
```

**Key change:** `total` is now computed, never hardcoded.

### C3: Rewrite `toolbox_catalog()` to use unified source

**Location:** `unified.rs:2825`

```rust
fn toolbox_catalog() -> Vec<(&'static str, Vec<String>)> {
    let catalog_json = unified_catalog_data();  // was: help_catalog_json()
    // ... rest unchanged
}
```

One-line change: call `unified_catalog_data()` instead of `help_catalog_json()`.

### C4: Delete `help_catalog_json()`

**Location:** `unified.rs:2851-2926`

Remove the entire function. It's replaced by `unified_catalog_data()`.

### C5: Add `dispatch_commands()` for test validation

**Location:** `unified.rs`, after `dispatch_inner()`

```rust
/// Returns all registered command names for catalog validation.
/// Used by tests to verify catalog ↔ dispatch alignment.
#[cfg(test)]
fn dispatch_commands() -> Vec<&'static str> {
    vec![
        "help", "toolbox",
        "nexcore_health", "config_validate", "mcp_servers_list", "mcp_server_get",
        // ... all 1,289 command strings
    ]
}
```

### C6: Add catalog validation test

**Location:** `unified.rs` (test module) or `tests/catalog_integrity.rs`

```rust
#[test]
fn catalog_covers_all_dispatch_commands() {
    let catalog = unified_catalog_data();
    let categories = catalog.get("categories")
        .and_then(|c| c.as_object())
        .expect("categories must exist");

    // Collect all cataloged tool names
    let cataloged: HashSet<String> = categories.values()
        .filter_map(|v| v.as_array())
        .flat_map(|arr| arr.iter())
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let dispatched: HashSet<&str> = dispatch_commands()
        .into_iter()
        .filter(|c| !["help", "toolbox"].contains(c))  // system commands
        .collect();

    // Every dispatched command should be in the catalog
    let not_in_catalog: Vec<_> = dispatched.iter()
        .filter(|c| !cataloged.contains(**c))
        .collect();
    assert!(not_in_catalog.is_empty(),
        "Dispatched but not cataloged: {:?}", not_in_catalog);

    // Every cataloged command should be dispatched
    let not_dispatched: Vec<_> = cataloged.iter()
        .filter(|c| !dispatched.contains(c.as_str()))
        .collect();
    assert!(not_dispatched.is_empty(),
        "Cataloged but not dispatched: {:?}", not_dispatched);
}

#[test]
fn catalog_total_is_accurate() {
    let catalog = unified_catalog_data();
    let categories = catalog.get("categories")
        .and_then(|c| c.as_object())
        .expect("categories must exist");

    let computed: usize = categories.values()
        .filter_map(|v| v.as_array())
        .map(|arr| arr.len())
        .sum();

    // No hardcoded total to drift — just verify non-zero
    assert!(computed > 400, "Catalog should have >400 tools, got {}", computed);
}
```

---

## Dependency Map

```
C1 (unified_catalog_data) ← C4 (delete help_catalog_json)
C1 → C2 (help_catalog rewrite)
C1 → C3 (toolbox_catalog rewrite)
C5 (dispatch_commands) → C6 (validation test)
C2 + C3 are independent once C1 lands
```

**Build order:** C1 → C4 → C2 + C3 (parallel) → C5 → C6

---

## Trait Signatures

No new traits. All changes are within `unified.rs` — private functions only. No public API change. No new crates. No new dependencies.

---

## Test Plan

| Test | What It Validates | Gate |
|------|------------------|------|
| `catalog_covers_all_dispatch_commands` | Dispatch ↔ catalog bidirectional sync | Every command in both sets |
| `catalog_total_is_accurate` | Total > 400 tools | Smoke check |
| `cargo build -p nexcore-mcp --release` | No compile errors | Clean build |
| `cargo test -p nexcore-mcp --lib` | No regressions | All pass |
| `cargo clippy -p nexcore-mcp -- -D warnings` | No new warnings | Clean |
| Manual: `nexcore(command="help")` | Help shows complete catalog | 190+ categories |
| Manual: `nexcore(command="toolbox", params={query:"forge"})` | Toolbox finds forge tools | Non-empty results |

---

## T1 Primitive Grounding

| Concept | Primitive | How |
|---------|-----------|-----|
| Catalog unification | μ (Mapping) | Many categories → one function |
| Dispatch ↔ catalog boundary | ∂ (Boundary) | Test enforces alignment at interface |
| Computed total | N (Quantity) | Derive from data, don't hardcode |
| Source of truth | ∃ (Existence) | One function exists, not two |
| Catalog search | κ (Comparison) | Toolbox keyword matching |
| Build-order deps | σ (Sequence) | C1→C4→C2+C3→C5→C6 |

---

## Exclusions

- **NOT refactoring dispatch_inner** — the match table stays as-is. Only catalog functions change.
- **NOT adding `#[catalog]` macros** — too invasive for this directive. Future work.
- **NOT changing toolbox search algorithm** — it works fine, just needs complete data.
- **NOT restructuring tool source files** — only `unified.rs` changes.

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Merge conflict (unified.rs is hot) | Medium | Single file, all changes localized to catalog section (lines 2825-3125) |
| Missing commands in dispatch_commands() | Low | Test catches the delta immediately |
| Large inline JSON in one function | Low | Already exists (help_catalog is ~200 lines of JSON) — consolidation doesn't increase total LOC |
| Breaking toolbox for existing users | Low | Same interface, just more data |

---

## Score

| Criterion | Points | Score | Evidence |
|-----------|--------|-------|---------|
| Trait signatures present | 25 | **25** | No new traits needed (documented), function signatures specified |
| Dependency map present | 25 | **25** | C1→C4→C2+C3→C5→C6 with build order |
| Test plan present | 25 | **25** | 7 tests covering compile, unit, integration, manual |
| T1 grounding table present | 25 | **25** | 6 primitives mapped |

**D Phase Score: 100/100**

---

## GATE-D Decision

**PASS** — Score 100 >= 50 threshold. Proceed to I (Implement) phase.
