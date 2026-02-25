# Directive 006A — Phase 2: nexcore-dataframe Design

**Date**: 2026-02-24
**Status**: DESIGN — Awaiting review before implementation
**Predecessor**: Phase 1 audit complete — 4 consumers, 841 LOC, 20 operations

---

## Design Thesis

This is NOT a general-purpose DataFrame library. It is a purpose-built columnar data container covering the 20 operations used across 4 crates. The Phase 1 audit eliminated joins, pivots, melts, window functions, CSV I/O, categorical columns, SIMD, streaming, complex expressions, and LazyFrame from scope.

**What polars actually does in this workspace**: constructs DataFrames from column vecs, filters rows, groups by columns and aggregates (count/sum/mean/min/max/first), sorts, writes JSON. That's it. Everything else was compiled but never called.

---

## Crate Metadata

```
Name:         nexcore-dataframe
Layer:        Foundation
Location:     crates/nexcore-dataframe/
Dependencies: nexcore-error (workspace), serde + serde_json (workspace, for JSON I/O)
External:     ZERO
Safety:       forbid(unsafe_code), deny(unwrap_used, expect_used, panic)
Lints:        [lints] workspace = true
```

---

## 1. Core Types

### 1.1 — ColumnData (Storage Layer)

```rust
/// Type-safe column storage. Each variant holds a Vec<Option<T>> for null support.
#[derive(Debug, Clone)]
pub enum ColumnData {
    Bool(Vec<Option<bool>>),
    Int64(Vec<Option<i64>>),
    UInt64(Vec<Option<u64>>),
    Float64(Vec<Option<f64>>),
    String(Vec<Option<String>>),
}
```

**Why these 5 types**: The audit found Bool, i64, u64, f64, String used across all 4 crates. No Date/DateTime/Categorical usage found anywhere.

**Why `Vec<Option<T>>`**: Null handling is used in faers-etl (demographics columns). Option per-element is simpler than bitmap-based null tracking and sufficient for the data volumes in this workspace.

### 1.2 — DataType (Type Enumeration)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Bool,
    Int64,
    UInt64,
    Float64,
    Utf8,
}
```

### 1.3 — Column

```rust
#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    data: ColumnData,
}
```

### 1.4 — Scalar (Heterogeneous Value)

```rust
/// A single typed value, used for aggregation results and literals.
#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    Null,
    Bool(bool),
    Int64(i64),
    UInt64(u64),
    Float64(f64),
    String(String),
}
```

### 1.5 — DataFrame

```rust
/// A columnar data structure. Each column is a named, typed array.
/// All columns must have the same length.
#[derive(Debug, Clone)]
pub struct DataFrame {
    columns: Vec<Column>,
}
```

No separate `height` field — derived from first column's length (or 0 if empty). Enforced at construction: all columns same length.

### 1.6 — Schema

```rust
/// Column name → DataType mapping.
#[derive(Debug, Clone)]
pub struct Schema {
    fields: Vec<(String, DataType)>,
}
```

### 1.7 — Error Type

```rust
#[derive(Debug, nexcore_error::Error)]
pub enum DataFrameError {
    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("column length mismatch: expected {expected}, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },

    #[error("type mismatch: column '{column}' is {actual}, expected {expected}")]
    TypeMismatch {
        column: String,
        expected: DataType,
        actual: DataType,
    },

    #[error("empty dataframe")]
    Empty,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
```

---

## 2. Operations (Priority-Ordered)

### P0 — MUST HAVE (Used by all 4 crates)

| Operation | Signature | Used By |
|-----------|-----------|---------|
| `new` | `fn new(columns: Vec<Column>) -> Result<Self>` | faers-etl |
| `empty` | `fn empty() -> Self` | faers-etl |
| `height` | `fn height(&self) -> usize` | all 4 |
| `width` | `fn width(&self) -> usize` | faers-etl |
| `column` | `fn column(&self, name: &str) -> Result<&Column>` | faers-etl |
| `schema` | `fn schema(&self) -> Schema` | — |
| `filter` | `fn filter(&self, mask: &[bool]) -> Result<DataFrame>` | all 4 |
| `group_by` | `fn group_by(&self, keys: &[&str]) -> Result<GroupBy>` | all 4 |
| `GroupBy::agg` | `fn agg(&self, aggs: &[Agg]) -> Result<DataFrame>` | all 4 |
| `Agg::Count` | Count non-null values | all 4 |
| `Agg::Sum` | Sum numeric column | disney, transformer |
| `Agg::Mean` | Mean of numeric column | pvos |
| `Agg::Min` | Minimum value | disney |
| `Agg::First` | First value in group | disney |

### P1 — NEEDED (Used by 1-2 crates)

| Operation | Signature | Used By |
|-----------|-----------|---------|
| `sort` | `fn sort(&self, by: &str, descending: bool) -> Result<DataFrame>` | faers-etl |
| `head` | `fn head(&self, n: usize) -> DataFrame` | faers-etl |
| `Agg::Max` | Maximum value | faers-etl |
| `Agg::Std` | Standard deviation | faers-etl |
| `Agg::NUnique` | Count distinct | faers-etl |
| `quantile` | `fn quantile(&self, col: &str, q: f64) -> Result<f64>` | faers-etl |
| `select` | `fn select(&self, names: &[&str]) -> Result<DataFrame>` | faers-etl |
| `with_column` | `fn with_column(&self, col: Column) -> Result<DataFrame>` | — |

### P2 — DEFERRED (Not used, but natural extensions)

| Operation | Rationale |
|-----------|-----------|
| `join` | Not used anywhere — defer |
| `pivot` / `melt` | Not used anywhere — defer |
| `concat` | Not used anywhere — defer |
| `window functions` | Not used anywhere — defer |
| `CSV I/O` | Feature enabled but never called — defer |
| `Parquet I/O` | See section 4 below |
| `LazyFrame` | See section 3 below |

---

## 3. No LazyFrame

The original directive considered LazyFrame. The audit eliminates it.

**Every pipeline in this workspace is linear**: ingest → transform₁ → transform₂ → ... → sink. There are no branching query plans, no predicate pushdown opportunities, no join reordering. A LazyFrame query optimizer adds complexity for zero benefit on linear chains.

**Replacement strategy**: Method chaining on eager DataFrames. Each operation returns a new `DataFrame` (or `Result<DataFrame>`). This covers every use case found in the audit:

```rust
// Current polars pattern:
let result = df.lazy()
    .group_by([col("drug"), col("event")])
    .agg([col("case_id").count().alias("n")])
    .filter(col("n").gt_eq(lit(3)))
    .collect()?;

// nexcore-dataframe equivalent:
let result = df
    .group_by(&["drug", "event"])?
    .agg(&[Agg::Count("case_id", "n")])?
    .filter_by("n", |v| v.as_i64().is_some_and(|n| n >= 3))?;
```

No `.lazy()`, no `.collect()`, no `col()`, no `lit()`. Direct method calls.

---

## 4. Parquet Decision: Defer, Switch to JSON

Parquet is used only in `nexcore-faers-etl` for writing signal detection results and drug-event counts. Building a sovereign Parquet writer is non-trivial (Thrift encoding, Snappy compression, row groups, column chunks, page headers).

**Decision**: Switch faers-etl output from Parquet to JSON (via serde_json). Rationale:
- serde_json is already an essential workspace dependency (zero new deps)
- Signal detection results are 50K-500K rows × 16 columns — JSON handles this fine
- No downstream consumer currently reads the Parquet files programmatically
- If Parquet is needed later, add it as a standalone feature behind a feature flag

**If Parquet becomes required**: Add `parquet` crate as a direct dep of `nexcore-faers-etl` (not of `nexcore-dataframe`). This isolates the complex format dependency to the single crate that needs it, rather than polluting the foundation layer.

---

## 5. JSON I/O

### 5.1 — JSON Read (replaces polars JsonReader)

```rust
impl DataFrame {
    /// Deserialize a JSON array of objects into a DataFrame.
    /// Each object becomes a row; keys become column names.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, DataFrameError>;

    /// Read from a JSON string.
    pub fn from_json_str(s: &str) -> Result<Self, DataFrameError>;

    /// Read from a reader (replaces JsonReader::new(cursor).finish()).
    pub fn from_json_reader<R: std::io::Read>(reader: R) -> Result<Self, DataFrameError>;
}
```

Used by: disney-loop (stdin), pvos-primitive-expansion (HTTP response), transformer-primitives (stdin).

### 5.2 — JSON Write (replaces polars JsonWriter)

```rust
impl DataFrame {
    /// Serialize to a JSON array of objects.
    pub fn to_json(&self) -> Result<serde_json::Value, DataFrameError>;

    /// Write JSON to a writer (replaces JsonWriter::new(file).finish()).
    pub fn to_json_writer<W: std::io::Write>(&self, writer: W) -> Result<(), DataFrameError>;

    /// Write JSON to a file path.
    pub fn to_json_file(&self, path: &std::path::Path) -> Result<(), DataFrameError>;
}
```

Used by: disney-loop (JSON file), pvos-primitive-expansion (JSON file).

---

## 6. Column Construction (replaces polars Series::new)

```rust
impl Column {
    pub fn new_bool(name: impl Into<String>, data: Vec<Option<bool>>) -> Self;
    pub fn new_i64(name: impl Into<String>, data: Vec<Option<i64>>) -> Self;
    pub fn new_u64(name: impl Into<String>, data: Vec<Option<u64>>) -> Self;
    pub fn new_f64(name: impl Into<String>, data: Vec<Option<f64>>) -> Self;
    pub fn new_string(name: impl Into<String>, data: Vec<Option<String>>) -> Self;

    // Convenience: construct from non-optional vecs (no nulls)
    pub fn from_bools(name: impl Into<String>, data: Vec<bool>) -> Self;
    pub fn from_i64s(name: impl Into<String>, data: Vec<i64>) -> Self;
    pub fn from_u64s(name: impl Into<String>, data: Vec<u64>) -> Self;
    pub fn from_f64s(name: impl Into<String>, data: Vec<f64>) -> Self;
    pub fn from_strings(name: impl Into<String>, data: Vec<String>) -> Self;
    pub fn from_strs(name: impl Into<String>, data: Vec<&str>) -> Self;
}
```

This replaces `Series::new(name.into(), vec).into()`. The convenience constructors (`from_*`) avoid wrapping every element in `Some(...)` for the common case of no nulls.

---

## 7. Column Access (replaces polars .str(), .u32(), .u64(), .i64())

```rust
impl Column {
    pub fn name(&self) -> &str;
    pub fn dtype(&self) -> DataType;
    pub fn len(&self) -> usize;

    // Type-specific accessors (replace polars .str(), .u32(), etc.)
    pub fn as_str_iter(&self) -> Result<impl Iterator<Item = Option<&str>>, DataFrameError>;
    pub fn as_i64_iter(&self) -> Result<impl Iterator<Item = Option<i64>>, DataFrameError>;
    pub fn as_u64_iter(&self) -> Result<impl Iterator<Item = Option<u64>>, DataFrameError>;
    pub fn as_f64_iter(&self) -> Result<impl Iterator<Item = Option<f64>>, DataFrameError>;
    pub fn as_bool_iter(&self) -> Result<impl Iterator<Item = Option<bool>>, DataFrameError>;

    // Indexed access (replaces polars .get(i))
    pub fn get(&self, index: usize) -> Option<Scalar>;
    pub fn get_str(&self, index: usize) -> Result<Option<&str>, DataFrameError>;
    pub fn get_i64(&self, index: usize) -> Option<Option<i64>>;
    pub fn get_u64(&self, index: usize) -> Option<Option<u64>>;
    pub fn get_f64(&self, index: usize) -> Option<Option<f64>>;
}
```

---

## 8. Aggregation API

```rust
/// Aggregation specification for GroupBy.
pub enum Agg {
    /// Count non-null values. Args: (source_column, output_alias)
    Count(&'static str, &'static str),
    /// Sum numeric column.
    Sum(&'static str, &'static str),
    /// Mean of numeric column.
    Mean(&'static str, &'static str),
    /// Minimum value.
    Min(&'static str, &'static str),
    /// Maximum value.
    Max(&'static str, &'static str),
    /// First value in group.
    First(&'static str, &'static str),
    /// Standard deviation.
    Std(&'static str, &'static str),
    /// Count distinct values.
    NUnique(&'static str, &'static str),
}

/// Intermediate GroupBy state.
pub struct GroupBy<'a> {
    df: &'a DataFrame,
    keys: Vec<String>,
}

impl<'a> GroupBy<'a> {
    pub fn agg(&self, aggs: &[Agg]) -> Result<DataFrame, DataFrameError>;
}
```

**GroupBy implementation**: Hash-based. Build `HashMap<Vec<Scalar>, Vec<usize>>` mapping key tuples to row indices. For each group, compute aggregations over the collected indices. This handles the FAERS scale (100K-500K groups from 20-50M input rows) efficiently.

---

## 9. Filter API

```rust
impl DataFrame {
    /// Filter rows by a boolean mask.
    pub fn filter(&self, mask: &[bool]) -> Result<DataFrame, DataFrameError>;

    /// Filter rows by a predicate on a named column.
    pub fn filter_by(
        &self,
        column: &str,
        predicate: impl Fn(&Scalar) -> bool,
    ) -> Result<DataFrame, DataFrameError>;
}
```

This replaces polars' `col("x").gt_eq(lit(3))` expression system with a closure. Simpler, no expression AST, covers every filter pattern found in the audit.

---

## 10. FAERS Optimization: Direct HashMap Path

The audit identified that faers-etl's heaviest operation is `group_by([drug, event]).agg([count(case_id)])` on 20-50M rows. This is semantically `HashMap<(String, String), u64>`.

**Design decision**: Provide BOTH paths:
1. **DataFrame path** (general): `df.group_by(&["drug", "event"])?.agg(&[Agg::Count("case_id", "n")])?` — works for any DataFrame
2. **Direct counter** (optimized): A standalone `Counter` type for the common "count by key" pattern

```rust
/// High-performance counter for the "group by string keys + count" pattern.
/// Avoids constructing an intermediate DataFrame for the most common aggregation.
pub struct Counter {
    counts: HashMap<Vec<String>, u64>,
    key_names: Vec<String>,
}

impl Counter {
    pub fn new(key_names: Vec<String>) -> Self;
    pub fn increment(&mut self, keys: Vec<String>);
    pub fn into_dataframe(self, count_column: &str) -> Result<DataFrame, DataFrameError>;
}
```

faers-etl can bypass the 20-50M row DataFrame entirely:

```rust
// Instead of: build 20M-row DataFrame → group_by → count
// Do: accumulate directly into Counter during flatten
let mut counter = Counter::new(vec!["drug".into(), "event".into()]);
for (drug, event) in pairs {
    counter.increment(vec![drug, event]);
}
let counts_df = counter.into_dataframe("n")?;
// counts_df has ~500K rows directly — no 20M intermediate
```

This is the single highest-impact optimization in the migration. It eliminates the need for a 20-50M row in-memory DataFrame for the FAERS pipeline.

---

## 11. Measured<T> Integration

```rust
use stem_math::measured::Measured;

impl DataFrame {
    /// Aggregate with confidence derived from null ratio.
    /// Confidence = valid_count / total_count, clamped to [0.05, 0.99].
    pub fn sum_measured(&self, col: &str) -> Result<Measured<f64>, DataFrameError>;
    pub fn mean_measured(&self, col: &str) -> Result<Measured<f64>, DataFrameError>;
    pub fn std_measured(&self, col: &str) -> Result<Measured<f64>, DataFrameError>;
}
```

---

## 12. GroundsTo Compliance

```rust
impl GroundsTo for DataFrame {
    // T2-C: Compound type — composed of Columns (T2-P) which hold primitives (T1)
    // Dominant: Σ(Sum) 0.85 — DataFrame exists to aggregate and summarize
    // Supporting: N(Quantity), μ(Mapping), ∂(Boundary)
}

impl GroundsTo for Column {
    // T2-P: Primitive compound — single named typed array
    // Dominant: N(Quantity) 0.80
    // Supporting: κ(Comparison) — columns enable comparison
}

impl GroundsTo for DataType {
    // T1: Pure primitive — type classification
    // Dominant: κ(Comparison) 1.0
}
```

This addresses the T2Compound gap from Directive 005.

---

## 13. File Structure

```
crates/nexcore-dataframe/
├── Cargo.toml
├── src/
│   ├── lib.rs            # Public API re-exports, safety attrs
│   ├── column.rs          # Column, ColumnData constructors/accessors
│   ├── dataframe.rs       # DataFrame core: new, empty, height, width, column, schema
│   ├── scalar.rs          # Scalar enum + conversions
│   ├── schema.rs          # Schema type
│   ├── error.rs           # DataFrameError
│   ├── counter.rs         # Counter (optimized group-count)
│   ├── filter.rs          # filter, filter_by
│   ├── sort.rs            # sort, sort_by
│   ├── agg.rs             # Scalar aggregations: sum, mean, min, max, std, quantile
│   ├── group.rs           # GroupBy + Agg
│   ├── select.rs          # select, head, with_column
│   ├── measured.rs        # Measured<T> wrappers
│   ├── grounding.rs       # GroundsTo implementations
│   └── io/
│       ├── mod.rs
│       └── json.rs        # from_json, to_json, reader/writer
```

---

## 14. Estimated LOC

| Module | Est. LOC | Notes |
|--------|----------|-------|
| column.rs | 250 | 5 data types × constructors, accessors, iterators |
| dataframe.rs | 150 | Core struct, new, empty, column lookup |
| scalar.rs | 100 | Enum + From impls + comparison |
| schema.rs | 50 | Simple name→type vec |
| error.rs | 40 | Error enum |
| counter.rs | 80 | HashMap-based counter |
| filter.rs | 100 | Mask filter + predicate filter |
| sort.rs | 120 | Sort by column(s) with direction |
| agg.rs | 200 | 8 aggregation functions on columns |
| group.rs | 250 | Hash-based GroupBy + multi-agg |
| select.rs | 80 | Column selection, head, with_column |
| measured.rs | 60 | 3 Measured<T> wrappers |
| grounding.rs | 40 | GroundsTo impls |
| io/json.rs | 200 | JSON array↔DataFrame round-trip |
| **Total** | **~1,720** | |

Tests estimated at ~1,000 LOC additional.

**Total: ~2,700 LOC** (vs polars' ~200,000+ LOC). We're building <1.5% of polars.

---

## 15. Test Strategy

### 15.1 — Unit Tests (Per Module)

| Module | Tests | Coverage |
|--------|-------|----------|
| column.rs | Construction, null handling, type access, iteration | Every constructor + accessor |
| dataframe.rs | new (valid, empty, mismatched lengths), column lookup | Error paths |
| scalar.rs | Comparison, ordering, conversions | All type variants |
| filter.rs | Mask filter, predicate filter, empty result | Edge: all-false mask |
| sort.rs | Ascending, descending, null ordering, multi-column | Stability |
| agg.rs | Each of 8 aggregation functions | Null handling, empty column |
| group.rs | Single key, multi key, all aggregation types | Edge: single group |
| counter.rs | Increment, into_dataframe | Large key sets |
| io/json.rs | Round-trip, nested objects, null values | Malformed input |
| measured.rs | Confidence from null ratio | All-null, no-null |

### 15.2 — Integration Tests

| Test | Purpose |
|------|---------|
| FAERS pattern | Build 18-col DataFrame, group_by(drug,event).agg(count), filter(n>=3), extract columns, verify contingency table math |
| Disney pattern | from_json, filter(direction != backward), group_by(domain).agg(sum+count), to_json_file |
| PVOS pattern | from_json, group_by(tier).agg(count+mean), to_json |
| Counter vs GroupBy | Verify Counter produces identical results to DataFrame group_by for same input |
| Round-trip | to_json → from_json produces identical DataFrame |

### 15.3 — Minimum Targets

- 200+ tests total
- Every public method has at least one test
- Null handling tested for every operation
- Edge cases: empty DataFrame, single-row, single-column, all-null column
- Type mismatch errors tested

---

## 16. Migration Plan Overview

### Order (leaf-first)

1. **`transformer-primitives`** — Simplest. 6/7 stubs, 1 group_by. ~56 LOC to change.
2. **`pvos-primitive-expansion`** — Small. HTTP ingest → filter → 2 group_by → JSON sink. ~110 LOC.
3. **`disney-loop`** — Small. JSON stdin → filter → group_by → JSON file. ~160 LOC.
4. **`nexcore-faers-etl`** — Largest. Parquet→JSON switch + Counter optimization. ~515 LOC.
5. **`nexcore-mcp`** — Transitive. Update Cargo.toml deps only.
6. **`nexcore-pharos`** — Transitive. Update Cargo.toml deps only.

### Migration Pattern Per Crate

```rust
// Before: use polars::prelude::*;
// After:  use nexcore_dataframe::prelude::*;

// Before: Series::new("name".into(), vec).into()
// After:  Column::from_strings("name", vec)

// Before: df.lazy().group_by([col("x")]).agg([col("y").count().alias("n")]).collect()?
// After:  df.group_by(&["x"])?.agg(&[Agg::Count("y", "n")])?

// Before: df.filter(col("n").gt_eq(lit(3)))
// After:  df.filter_by("n", |v| v.as_i64().is_some_and(|n| n >= 3))?

// Before: JsonReader::new(cursor).finish()?
// After:  DataFrame::from_json_reader(cursor)?

// Before: JsonWriter::new(&mut file).with_json_format(JsonFormat::Json).finish(&mut df)?
// After:  df.to_json_writer(&mut file)?
```

### Atomic Commit Per Crate

Each crate migration is a single commit:
1. Replace `use polars::prelude::*` with `use nexcore_dataframe::prelude::*`
2. Translate API calls (mechanical — patterns above)
3. Remove `polars` from crate's `Cargo.toml`
4. Run crate tests — must pass
5. Commit: `sovereignty: migrate {crate} from polars to nexcore-dataframe`

---

## 17. What We Do NOT Build

| Capability | Rationale |
|------------|-----------|
| LazyFrame | Linear pipelines don't benefit from query optimization |
| Expression DSL (`col()`, `lit()`) | Replaced by closures and direct method calls |
| CSV reader/writer | Feature enabled in all 4 crates but never called |
| Parquet reader/writer | Deferred — switch to JSON output |
| Joins | Not used anywhere |
| Pivot / Melt / Reshape | Not used anywhere |
| Window functions | Not used anywhere |
| String operations | Not used anywhere |
| Concat | Not used anywhere |
| Apply / Map UDFs | Not used anywhere |
| Categorical columns | Not used anywhere |
| Date/DateTime columns | Not used anywhere |
| SIMD vectorization | Batch pipelines — correctness over speed |
| Multi-threaded execution | Single-threaded sufficient |
| Apache Arrow interop | No external Arrow consumers |
| Streaming / chunked processing | Not used anywhere |

---

## 18. Prelude Module

```rust
/// nexcore_dataframe::prelude — common imports for consumers.
pub mod prelude {
    pub use crate::{
        Agg, Column, ColumnData, Counter, DataFrame, DataFrameError, DataType, Scalar, Schema,
    };
}
```

---

*Design complete. Ready for review before Phase 3 implementation.*
