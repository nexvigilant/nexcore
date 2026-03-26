# AI Guidance — nexcore-knowledge-engine

Knowledge compression, compilation, and query engine for structured knowledge packs.

## Use When
- Ingesting raw text (brain distillations, artifacts, implicit knowledge, free text) into scored fragments.
- Compressing verbose text through pattern replacement, dedup, and hierarchy flattening.
- Compiling fragments into versioned, immutable knowledge packs with concept graphs.
- Querying packs by keyword, concept, or domain with relevance ranking.
- Extracting T1-T3 primitives and domain-classified concepts from text.
- Computing Compendious Score (Cs = I/E x C x R) for information density measurement.

## Layer
**Domain** — depends on foundation crates (`nexcore-lex-primitiva`, `nexcore-error`, `nexcore-chrono`, `nexcore-id`, `nexcore-fs`). Consumed by `nexcore-mcp` (10 MCP tools).

## Architecture

```
RawKnowledge → ingest() → KnowledgeFragment
                              ↓
Fragments → KnowledgeCompiler::compile() → KnowledgePack
                              ↓
KnowledgePack → KnowledgeStore::save_pack() → disk (atomic writes)
                              ↓
QueryEngine::query() → QueryResponse (keyword/concept/domain modes)
```

### Compression Pipeline (3 stages)
1. **Pattern** — 50+ verbose phrase replacements (nominalizations, redundancies, throat-clearing)
2. **Dedup** — Token Jaccard similarity collapses near-duplicate sentences (threshold: 0.8)
3. **Hierarchy** — Flattens single-child markdown heading nesting (multi-child preserved)

### Key Invariants
- `apply_hierarchy()` only flattens when `direct_child_count == 1` — multi-child structures are preserved
- Readability uses sentence-length penalty, NOT Flesch formula (which penalizes technical vocabulary)
- Store writes use atomic tmp+rename pattern for crash safety
- All collections use `BTreeMap`/`BTreeSet` for deterministic iteration

## Grounding Patterns

All primary types implement `GroundsTo`:

| Type | Composition | Dominant |
|------|-------------|----------|
| `KnowledgePack` | `π + μ + σ + N` | `π` (0.80) |
| `KnowledgeFragment` | `π + μ + N` | `π` (0.80) |
| `ConceptGraph` | `μ + σ + ρ` | `μ` (0.80) |
| `StructuralCompressor` | `μ + σ + ∂` | `μ` (0.80) |
| `CompendiousScorer` | `μ + N` | `μ` (0.85) |
| `QueryEngine` | `μ + κ + N` | `μ` (0.85) |
| `KnowledgeStore` | `π + λ` | `π` (0.85) |
| `CompileOptions` | `σ + μ + →` | `σ` (0.80) |
| `DomainClassifier` | `μ + κ` | `μ` (0.85) |
| `KnowledgeEngineError` | `∂ + Σ` | `∂` (0.85) |

## MCP Tools (10)

| Tool | Purpose |
|------|---------|
| `knowledge_ingest` | Ingest single text into scored fragment, persisted to staging area |
| `knowledge_compress` | Compress text through 3 active stages (Pattern, Dedup, Hierarchy) |
| `knowledge_compile` | Full pipeline: sources + staged fragments → compress → graph → pack |
| `knowledge_query` | Query packs by keyword/concept/domain (returns `packs_searched` + `packs_matched`) |
| `knowledge_stats` | Engine-wide statistics across all packs |
| `knowledge_score` | Compute Compendious Score for text |
| `knowledge_extract_primitives` | Extract T1-T3 primitives from text |
| `knowledge_extract_concepts` | Extract domain-classified concepts from text |
| `knowledge_delete` | Remove a pack (all versions) or a specific version |
| `knowledge_prune` | Keep N newest versions, remove the rest |

## MCP Tool Naming — Two Surfaces

The 10 `#[tool]` methods in `nexcore-mcp/src/lib.rs` are exposed directly as `knowledge_*`
(e.g., `knowledge_ingest`). The `nexcore(command=...)` unified dispatcher uses the
`knowledge_engine_*` prefix (e.g., `knowledge_engine_ingest`). Both call the same backend
functions. The CLAUDE.md table above shows the `#[tool]` names (direct surface).

`query_merged()` in `src/query.rs` is a cross-pack merge helper used internally. It is
NOT exposed as a separate MCP tool — `knowledge_query` (without `pack_name`) already
queries all packs and flattens results. `query_merged()` may be promoted to an MCP tool
in a future iteration if callers need single-ranked cross-pack results without pack metadata.

## Key Entry Points
- `src/lib.rs` — Module declarations and public re-exports
- `src/ingest.rs` — `ingest()` function, `KnowledgeFragment`, `RawKnowledge`
- `src/compiler.rs` — `KnowledgeCompiler`, `CompileOptions`, `CompressTextResult`, brain source loading
- `src/compression.rs` — `StructuralCompressor`, 3-stage pipeline, `token_similarity()`
- `src/scoring.rs` — `CompendiousScorer`, Cs formula
- `src/concept_graph.rs` — `ConceptGraph`, Kahn's topsort, adjacency list
- `src/extraction.rs` — `ConceptExtractor`, `DomainClassifier`, primitive/concept keyword heuristics
- `src/query.rs` — `QueryEngine`, 3 query modes with relevance ranking, `query_merged()` for cross-pack results
- `src/store.rs` — `KnowledgeStore`, atomic file persistence, versioned packs, lifecycle (delete/prune), `find_fragment()` cross-pack search
- `src/stats.rs` — `compute_stats()`, `pack_stats()`
- `src/grounding.rs` — `GroundsTo` implementations for all primary types
- `src/knowledge_pack.rs` — `KnowledgePack`, `PackIndex`, `PackStats`, `get_fragment()` lookup
- `src/error.rs` — `KnowledgeEngineError` enum

## Maintenance SOPs
- **Readability formula**: Do NOT replace with Flesch. Flesch penalizes polysyllabic technical vocabulary. Sentence-length penalty is intentional.
- **Hierarchy flattening**: Always verify `direct_child_count == 1` before promoting. Regression test: `hierarchy_multi_child_is_preserved`.
- **Store writes**: Always use tmp+rename. Never sequential `fs::write()`.
- **Collections**: `BTreeMap`/`BTreeSet` only. `HashMap`/`HashSet` banned by workspace policy.
- **PackStats.compression_ratio**: Initialized to `0.0` in `KnowledgePack::new()`, set by compiler after compression. If constructing packs outside the compiler, this will be wrong.
- **KnowledgeId**: Type alias for `String` in `lib.rs`. Used by `KnowledgeFragment.id`, `KnowledgePack.id`, and `QueryResult.fragment_id`. If migrating to a newtype, only `lib.rs` changes.
- **DomainClassifier**: Default covers 6 domains (pv, rust, claude-code, chemistry, physics, regulatory). Custom domains via `add_domain()`. `ConceptExtractor::extract_concepts()` uses the default; `extract_concepts_with()` accepts a custom classifier.
