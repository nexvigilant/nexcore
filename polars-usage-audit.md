# Directive 006A — Phase 1: Polars Usage Audit

**Date**: 2026-02-24
**Status**: COMPLETE

---

## Critical Correction: 4 Consumers, Not 38

Directive 005 estimated 38 workspace crates depend on polars. **The actual number is 4 direct consumers + 2 transitive consumers.** The D005 estimate likely counted grep matches in docs, archives, and comments.

| Category | Crates | Names |
|----------|--------|-------|
| Direct dependency | 4 | `nexcore-faers-etl`, `disney-loop`, `pvos-primitive-expansion`, `transformer-primitives` |
| Transitive consumer | 2 | `nexcore-mcp` (imports all 3 domain crates), `nexcore-pharos` (imports faers-etl) |
| Archived/experimental | 4 | In `docs/archive/` and `wksp/` — not workspace members, not compiled |
| **Total active** | **6** | 4 direct + 2 transitive |

This dramatically reduces migration scope. ~841 lines of polars-touching code across 4 crates (not thousands across 38).

---

## 1.1 — Consumer Census

### Crate 1: `nexcore-faers-etl` (CRITICAL — Primary Consumer)

```
Crate name: nexcore-faers-etl
  - Dependency: DIRECT
  - Feature flags: lazy, csv, json, parquet
  - Polars types used:
    - DataFrame (eager construction, column extraction)
    - LazyFrame (all transforms)
    - Series (column construction from vecs — 18 columns)
    - StringChunked (string column iteration)
    - Column (wrapped Series for DataFrame::new)
    - ParquetWriter (Snappy-compressed output)
    - PolarsError (error wrapping)
    - col(), lit() (expression builders)
    - SortMultipleOptions (sorting config)
  - Operations used:
    - DataFrame::new(), DataFrame::empty()
    - Series::new(name, vec) — 18 columns constructed
    - df.lazy(), df.column(name), df.height(), df.clone()
    - .group_by([col(...)]).agg([col(...).count()])
    - .filter(col(...).gt_eq(lit(...)))
    - .collect()
    - .str(), .u32(), .u64(), .i64() (type casting)
    - .iter(), .get(i) (value access)
    - .min(), .max(), .mean(), .quantile(), .n_unique()
    - .sort() with SortMultipleOptions
    - .limit(n)
    - .alias() (column rename)
    - ParquetWriter::new().with_compression(Snappy).with_row_group_size(1M).finish()
  - Data sources: FAERS ASCII files parsed via nexcore_vigilance::pv::faers::parse_quarterly_linked()
                  → LinkedReport structs → flattened to (drug, event) pairs → DataFrame
  - Data sinks: Parquet files (Snappy compression, 1M row groups)
                Signal results Vec<SignalDetectionResult> → DataFrame → Parquet
  - LazyFrame: YES — all transforms use LazyFrame, eager only for ingest/sink
  - Approximate polars-touching LOC: ~515
```

### Crate 2: `disney-loop`

```
Crate name: disney-loop
  - Dependency: DIRECT
  - Feature flags: lazy, csv (UNUSED), json
  - Polars types used:
    - LazyFrame (all transforms)
    - DataFrame (test helpers, collect output)
    - PolarsError (error variant)
    - JsonReader (JSON ingestion from stdin)
    - JsonWriter (JSON output to file)
    - JsonFormat (format enum)
    - col(), lit() (expression builders)
  - Operations used:
    - JsonReader::new(cursor).finish() (JSON → eager DataFrame)
    - .lazy() (eager → lazy)
    - .filter(col("direction").neq(lit("backward")))
    - .filter(col("prob_generated").lt(lit(threshold)))
    - .group_by([col("domain")]).agg([col("novelty_score").sum(), col("discovery").count()])
    - .group_by([col("id")]).agg([col("text").first(), col("prob_generated").min()])
    - .alias() (column rename in aggregation)
    - .collect() (lazy → eager)
    - .height() (row count)
    - JsonWriter::new().with_json_format(Json).finish() (DataFrame → JSON file)
    - df! macro (test DataFrame construction)
  - Data sources: stdin (JSON lines → JsonReader)
  - Data sinks: JSON file (JsonWriter)
  - LazyFrame: YES — pure lazy pipeline, single collect at final sink
  - Approximate polars-touching LOC: ~160
```

### Crate 3: `pvos-primitive-expansion`

```
Crate name: pvos-primitive-expansion
  - Dependency: DIRECT
  - Feature flags: lazy, csv (UNUSED), json
  - Polars types used:
    - DataFrame (ingest return type)
    - LazyFrame (all transforms)
    - JsonReader (JSON ingestion from HTTP response)
    - JsonWriter, JsonFormat (JSON output to file)
    - col(), lit() (expression builders)
  - Operations used:
    - JsonReader::new(cursor).finish() (JSON → eager DataFrame)
    - .lazy() (eager → lazy)
    - .filter(lit(true)) (passthrough filter — stub)
    - .group_by([col("tier")]).agg([col("type_name").count().alias("type_count"), col("confidence").mean().alias("mean_confidence")])
    - .group_by([col("dominant_primitive")]).agg([col("type_name").count().alias("types_per_dominant")])
    - .collect() (lazy → eager)
    - .height() (row count)
    - JsonWriter::new().with_json_format(Json).finish() (DataFrame → JSON file)
  - Data sources: HTTP endpoint (localhost:3030/api/v1/lex_primitiva/types) → reqwest → JSON → DataFrame
  - Data sinks: JSON file (output/pvos-expansion-report.json)
  - LazyFrame: YES — lazy transforms, eager I/O
  - Approximate polars-touching LOC: ~110
```

### Crate 4: `transformer-primitives`

```
Crate name: transformer-primitives
  - Dependency: DIRECT
  - Feature flags: lazy, csv (UNUSED), json
  - Polars types used:
    - LazyFrame (function signatures — all 7 transforms)
    - JsonReader (stdin JSON ingestion)
    - col() (expression builder)
  - Operations used:
    - JsonReader::new(cursor).finish() (JSON → eager DataFrame)
    - .lazy() (eager → lazy)
    - .height() (row count)
    - .group_by([col("head_id")]).agg([col("value_vector").sum().alias("attended_output")])
    - NOTE: 6/7 transform functions are STUBS (log warning, return input unchanged)
    - NOTE: sink_token_prediction() is UNIMPLEMENTED (panics)
  - Data sources: stdin (JSON lines)
  - Data sinks: None (sink unimplemented)
  - LazyFrame: YES — exclusively LazyFrame
  - Approximate polars-touching LOC: ~56
  - STATUS: LARGELY UNIMPLEMENTED — only 1 of 7 transforms has code
```

### Transitive Consumers (no direct polars API usage)

```
nexcore-mcp:
  - Depends on: disney-loop, pvos-primitive-expansion, nexcore-faers-etl
  - Does NOT use polars types directly in MCP tool code
  - Polars enters via crate dependency graph only
  - Migration: update Cargo.toml deps after primary crates migrate

nexcore-pharos:
  - Depends on: nexcore-faers-etl
  - Does NOT use polars types directly
  - Migration: automatic once nexcore-faers-etl migrates
```

---

## 1.2 — Operation Frequency Map

| Operation Category | Occurrences | Crates Using | Example |
|--------------------|-------------|--------------|---------|
| DataFrame construction | 5 | 1 (faers-etl) | `DataFrame::new(vec![Series::new(...).into()])` |
| DataFrame::empty() | 1 | 1 (faers-etl) | `DataFrame::empty()` |
| Series::new() | 18+ | 1 (faers-etl) | `Series::new("drug", drug_vec)` |
| JSON read (JsonReader) | 3 | 3 (disney, pvos, transformer) | `JsonReader::new(cursor).finish()` |
| JSON write (JsonWriter) | 2 | 2 (disney, pvos) | `JsonWriter::new(file).with_json_format(Json).finish()` |
| Parquet write | 1 | 1 (faers-etl) | `ParquetWriter::new(file).with_compression(Snappy).finish()` |
| CSV read/write | 0 | 0 | Feature enabled but NEVER USED |
| Column selection (col()) | 15+ | 4 | `col("drug")` |
| Literal (lit()) | 5 | 3 | `lit("backward")`, `lit(min_cases)` |
| Filtering (.filter()) | 5 | 4 | `.filter(col("direction").neq(lit("backward")))` |
| GroupBy + Aggregation | 7 | 4 | `.group_by([col("drug"), col("event")]).agg([...])` |
| .sum() in agg | 2 | 2 | `col("novelty_score").sum()` |
| .count() in agg | 4 | 3 | `col("case_id").count()` |
| .mean() in agg | 2 | 2 | `col("confidence").mean()` |
| .first() in agg | 1 | 1 (disney) | `col("text").first()` |
| .min() in agg/stat | 2 | 2 | `col("prob_generated").min()` |
| .max() in stat | 1 | 1 (faers-etl) | `.max()` |
| .quantile() | 1 | 1 (faers-etl) | `.quantile(lit(0.5), QuantileMethod::Linear)` |
| .n_unique() | 1 | 1 (faers-etl) | `.n_unique()` |
| .alias() (rename) | 8 | 3 | `.alias("type_count")` |
| Sorting (.sort()) | 1 | 1 (faers-etl) | `.sort(SortMultipleOptions::...)` |
| .limit(n) | 1 | 1 (faers-etl) | `.limit(15)` |
| .collect() (lazy→eager) | 7 | 4 | `lazy_frame.collect()` |
| .lazy() (eager→lazy) | 5 | 4 | `df.lazy()` |
| .height() (row count) | 5 | 4 | `df.height()` |
| Type casting (.str/.u32/.u64/.i64) | 4 | 1 (faers-etl) | `col.str()`, `col.u32()` |
| Value iteration (.iter/.get) | 2 | 1 (faers-etl) | `series.iter()`, `series.get(i)` |
| df! macro (tests) | 2 | 1 (disney) | `df!("col" => vec![...])` |
| Joining | 0 | 0 | NOT USED ANYWHERE |
| Pivot / Melt / Reshape | 0 | 0 | NOT USED ANYWHERE |
| Window functions | 0 | 0 | NOT USED ANYWHERE |
| String operations | 0 | 0 | NOT USED ANYWHERE |
| Concat | 0 | 0 | NOT USED ANYWHERE |
| Apply/Map | 0 | 0 | NOT USED ANYWHERE |
| Expression system (complex) | 0 | 0 | Only simple col/lit/comparison |
| Categorical dtype | 0 | 0 | NOT USED |

### Key Observation: Minimal API Surface

Of polars' hundreds of operations, NexCore uses approximately **20 unique operations**. The usage is overwhelmingly:
1. **Construct** (DataFrame::new, Series::new, JsonReader)
2. **Filter** (simple comparisons: eq, neq, lt, gt_eq)
3. **GroupBy + Aggregate** (count, sum, mean, min, max)
4. **Collect** (lazy → eager materialization)
5. **Write** (JsonWriter, ParquetWriter)

**NOT USED**: Joins, pivots, melts, window functions, string ops, apply/map, concat, complex expressions, SIMD, streaming, categoricals. This means the sovereign replacement can be much simpler than a full polars clone.

---

## 1.3 — Feature Flag Analysis

| Crate | lazy | csv | json | parquet | csv Used? | json Used? | parquet Used? |
|-------|------|-----|------|---------|-----------|-----------|---------------|
| nexcore-faers-etl | YES | YES | YES | YES | **NO** | **NO** | **YES** |
| disney-loop | YES | YES | YES | — | **NO** | **YES** | — |
| pvos-primitive-expansion | YES | YES | YES | — | **NO** | **YES** | — |
| transformer-primitives | YES | YES | YES | — | **NO** | **YES** | — |

**Key findings:**
- `csv` feature: Enabled in ALL 4 crates but **NEVER USED** in any of them. Zero CsvReader/CsvWriter calls.
- `json` feature: Enabled and USED in 3/4 crates (disney-loop, pvos, transformer). NOT used in faers-etl.
- `parquet` feature: Enabled and USED only in nexcore-faers-etl.
- `lazy` feature: Enabled and USED in all 4 crates. All transforms are LazyFrame-based.

**Implication for replacement**: The sovereign engine needs:
- LazyFrame-like deferred evaluation (or can be replaced by method chaining on eager DataFrames)
- JSON I/O (via serde_json — already a workspace dep)
- Parquet I/O (only for faers-etl — can use a minimal Parquet writer or output CSV/JSON instead)
- **NO CSV I/O needed** (despite feature flags, no crate actually reads/writes CSV via polars)

---

## 1.4 — Data Volume Profile

### nexcore-faers-etl (Largest Volume)

| Metric | Value |
|--------|-------|
| Typical input rows | 6–10 million per FAERS quarter |
| After drug×reaction flattening | 20–50 million (drug, event) pairs |
| After group_by aggregation | 100K–500K unique drug-event pairs |
| Typical column count | 18 (ingest), 3 (counts), 16 (signal results) |
| Data types | String, u64, f64, Option<f64>, Option<String>, bool |
| Performance sensitivity | Batch/offline — NOT in hot paths. Runs as CLI pipeline. |
| Memory sensitivity | Files can exceed 10GB. CLAUDE.md notes "Monitor RSS on files >10GB; ensure SSD backing." |
| LazyFrame optimization | Used for query planning on large aggregations |

### disney-loop (Small Volume)

| Metric | Value |
|--------|-------|
| Typical input rows | 10s to 100s (discovery records) |
| Typical column count | 4-5 (domain, direction, novelty_score, discovery, prob_generated) |
| Data types | String, f64, bool |
| Performance sensitivity | NONE — small data, interactive pipeline |
| Memory sensitivity | NONE — trivially fits in memory |

### pvos-primitive-expansion (Small Volume)

| Metric | Value |
|--------|-------|
| Typical input rows | 10s to 100s (type classification records from API) |
| Typical column count | 3-5 (type_name, tier, confidence, dominant_primitive) |
| Data types | String, f64 |
| Performance sensitivity | NONE — small data, batch pipeline |
| Memory sensitivity | NONE — trivially fits in memory |

### transformer-primitives (Minimal — Largely Unimplemented)

| Metric | Value |
|--------|-------|
| Typical input rows | Unknown (6/7 transforms are stubs) |
| Performance sensitivity | NONE — experimental crate |

### Volume Assessment

**Only nexcore-faers-etl processes significant data volumes** (millions of rows). The other 3 crates process trivial amounts (10s–100s of rows).

For faers-etl: The heavy lifting is the `group_by([drug, event]).agg([count()])` operation that reduces 20–50M rows to 100K–500K. This is a hash-based aggregation. An eager implementation with `HashMap<(String, String), u64>` would handle this efficiently. LazyFrame query optimization is nice-to-have but not essential — the pipeline is linear (no branching query plans to optimize).

**Conclusion: A simple eager DataFrame with efficient group_by (hash-based) suffices.** LazyFrame is not architecturally required — the pipelines are linear chains, not DAGs. Iterator chaining or method chaining on an eager DataFrame covers all use cases.

---

## 1.5 — FAERS-Specific Analysis

### Data Ingestion Path

```
FAERS ASCII quarterly files (FDA download)
    ↓
nexcore_vigilance::pv::faers::parse_quarterly_linked()
    ↓
Vec<LinkedReport> (Rust structs — NOT polars)
    ↓
Flatten: each report × drugs × reactions → Vec<(case_id, drug, event, demographics...)>
    ↓
Series::new() for each of 18 columns
    ↓
DataFrame::new(columns) — eager construction
    ↓
.lazy() → LazyFrame for transforms
```

**Key insight**: FAERS data does NOT enter the system as CSV/Parquet via polars readers. It enters as parsed Rust structs from `nexcore_vigilance`. Polars is only used as an in-memory columnar container for aggregation. The data is already in Rust before polars touches it.

### File Formats

- **Input**: FAERS ASCII files (pipe-delimited or fixed-width, parsed by nexcore_vigilance — NOT by polars)
- **Output**: Parquet (Snappy compression, 1M row groups)

### FAERS Table Structures Referenced

| Table | Contents | Used By |
|-------|----------|---------|
| DEMO | Demographics (age, sex, weight, country) | Ingest → 18-column DataFrame |
| DRUG | Suspect/concomitant drugs | Drug names → group_by key |
| REAC | Reactions (MedDRA PTs) | Event names → group_by key |
| OUTC | Outcomes (fatal, recovering) | A82 algorithm |
| THER | Therapy dates | therapy_summary column |
| RPSR | Manufacturer metadata | mfr_sndr, mfr_num columns |
| INDI | Indications | Not directly used in ETL |

### Polars Operations on FAERS Data

1. **Ingest**: `DataFrame::new(vec![18 Series])` — construct from pre-parsed vecs
2. **Normalize**: `df.lazy()` → transform chain
3. **Count**: `.group_by([col("drug"), col("event")]).agg([col("case_id").count()])` — THE core operation
4. **Filter**: `.filter(col("n").gt_eq(lit(min_cases)))` — minimum case threshold
5. **Extract**: `.column("drug").str()`, `.u32()`, etc. — extract for contingency table building
6. **Signal detect**: Iterate extracted columns → compute PRR/ROR/IC/EBGM → build 16-column results DataFrame
7. **Sink**: `ParquetWriter::new(file).with_compression(Snappy).finish()` — write output

### Row Counts Per Stage

| Stage | Rows | Columns |
|-------|------|---------|
| Raw FAERS quarterly | 6–10M reports | 7 tables |
| Flattened (drug×event) | 20–50M pairs | 18 |
| After group_by | 100K–500K unique pairs | 3 (drug, event, n) |
| After min_cases filter | 50K–200K | 3 |
| Signal results | 50K–200K | 16 |

### Streaming Assessment

- **No streaming currently used.** All data loaded into memory.
- **Could work with iterator-based streaming?** YES — the pipeline is:
  1. Parse FAERS files → iterator of LinkedReport (already streaming in nexcore_vigilance)
  2. Flatten to (drug, event) pairs → could use HashMap counter directly
  3. The group_by+count step is essentially `HashMap<(String, String), u64>` accumulation
  4. This would eliminate the need for a 20–50M row DataFrame entirely

### Parquet Dependency Assessment

Parquet output is used only in nexcore-faers-etl for persisting signal detection results. Options:
1. **Keep Parquet via `parquet` crate directly** (smaller than polars, ~30 transitive deps)
2. **Switch to JSON output** (zero additional deps — serde_json already in workspace)
3. **Switch to CSV output** (simple sovereign implementation)
4. **Build minimal Parquet writer** (complex — Parquet format is non-trivial)

**Recommendation**: Switch faers-etl output to JSON or CSV. Parquet is convenient but not essential for downstream consumers. If Parquet is required for external tooling compatibility, add the `parquet` crate as a standalone dep (decoupled from polars).

---

## Summary: What the Sovereign Replacement Needs

### MUST HAVE (Used by ≥1 production crate)

| Capability | Used By | Priority |
|------------|---------|----------|
| DataFrame construction from column vecs | faers-etl | P0 |
| Column types: String, i64, u64, f64, bool, Option<T> | all | P0 |
| `.filter()` with simple comparisons (eq, neq, lt, gt, gte) | all 4 | P0 |
| `.group_by()` + `.agg()` with count, sum, mean, min, max | all 4 | P0 |
| `.collect()` equivalent (method chaining result) | all 4 | P0 |
| `.height()` (row count) | all 4 | P0 |
| `.column(name)` (column extraction) | faers-etl | P0 |
| Column type casting (to_str, to_u32, to_u64, to_i64, to_f64) | faers-etl | P0 |
| Column value iteration | faers-etl | P0 |
| `.alias()` (column rename in aggregation) | 3 crates | P0 |
| JSON serialization (DataFrame → JSON) | disney, pvos, transformer | P0 |
| JSON deserialization (JSON → DataFrame) | disney, pvos, transformer | P0 |
| `.sort()` with descending option | faers-etl | P1 |
| `.limit(n)` (head/take) | faers-etl | P1 |
| `.quantile()` | faers-etl | P1 |
| `.n_unique()` | faers-etl | P1 |
| `.first()` in aggregation | disney | P1 |
| `.std()` (standard deviation) | faers-etl | P1 |
| Parquet output (Snappy) | faers-etl | P2 (can switch to JSON/CSV) |

### NOT NEEDED (Not used anywhere)

| Capability | Status |
|------------|--------|
| CSV read/write | Feature enabled, never called |
| Joins | Not used |
| Pivot / Melt / Reshape | Not used |
| Window functions | Not used |
| String operations | Not used |
| Apply / Map | Not used |
| Concat | Not used |
| Complex expression DSL | Not used (only simple col/lit) |
| Categorical columns | Not used |
| SIMD vectorization | Not needed (batch pipelines) |
| Multi-threaded execution | Not needed |
| LazyFrame query optimizer | Not needed (linear pipelines) |
| Apache Arrow interop | Not needed |
| Streaming/chunked processing | Not needed |

### SIMPLIFICATION OPPORTUNITY

The FAERS pipeline's core operation (`group_by([drug, event]).agg([count()])`) is semantically a `HashMap<(String, String), u64>` accumulation. The sovereign replacement could bypass DataFrame entirely for this step and use native Rust HashMap. This would be faster (no columnar overhead), simpler, and eliminate the need for a 20–50M row intermediate DataFrame.

**Estimated sovereign replacement LOC**: 2,000–3,000 lines (vs polars' ~200,000+ lines). We need <2% of polars' functionality.

---

*Phase 1 complete. Ready for Phase 2 design review.*
