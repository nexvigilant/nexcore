# VDAG: nexcore-cognition Capability Development

## SMART Goal

**Specific:** Fix 8 identified defects, add 5 missing capabilities, expand MCP tool surface from 5→12, update primitive documentation across all 13 modules.
**Measurable:** 0 silent errors, 0 dead config, 0 phantom primitives, 8 defects→0, 37→55+ tests passing, 12 MCP tools operational.
**Achievable:** All work is within existing crate boundary, no external dependencies needed.
**Relevant:** Enables the cognitive engine to be composable, measurable, and learning-capable from any Claude Code session.
**Time-bound:** Single execution session.

## Reality Gradient Targets

| Phase | Weight | Target Evidence |
|-------|--------|-----------------|
| 0 (Preclinical) | 0.05 | SMART goal validated, DAG acyclic |
| 1 (Safety) | 0.15 | All silent errors fixed, cargo test passes |
| 2 (Efficacy) | 0.30 | Core capabilities added and tested |
| 3 (Scale) | 0.30 | MCP tools expanded and composable |
| 4 (Surveillance) | 0.20 | Docs updated, primitive coverage verified |

---

## DAG Definition

### Parallel Group 0: Foundation Fixes (no dependencies)

```
NODE_001: Fix silent error in embedding.rs
  file: src/embedding.rs:119
  action: Replace unwrap_or_else with ? propagation
  test: embedding table construction never silently produces zeros
  primitives_fixed: ∂

NODE_002: Fix silent error in mask.rs (causal)
  file: src/mask.rs:43
  action: Replace unwrap_or_else with ? propagation (change return type to Result)
  test: causal_mask returns Result, callers handle
  primitives_fixed: ∂

NODE_003: Fix silent error in mask.rs (padding)
  file: src/mask.rs:71
  action: Replace unwrap_or_else with ? propagation
  test: padding_mask returns Result
  primitives_fixed: ∂

NODE_004: Fix ffn_inner_dim dead config
  file: src/block.rs:98
  action: Pass ffn_inner_dim from TransformerConfig through to FeedForwardConfig
  test: custom ffn_inner_dim flows through to FeedForward::w1 shape
  primitives_fixed: μ

NODE_005: Remove phantom ν from generator.rs
  file: src/generator.rs:29
  action: Remove ν from T1 grounding section (no frequency tracking exists)
  primitives_fixed: κ (documentation accuracy)

NODE_006: Remove unused serde dependency
  file: Cargo.toml
  action: Delete serde = { workspace = true }
  test: cargo build succeeds without serde
  primitives_fixed: ∅

NODE_007: Fix double iteration in metrics.rs
  file: src/metrics.rs:112-148
  action: Compute per-head entropy in first loop, don't re-iterate
  test: same results, single pass
  primitives_fixed: N (efficiency)
```

### Parallel Group 1: Core Capabilities (depends on Group 0)

```
NODE_008: Add repetition penalty to sampling
  file: src/sample.rs
  depends_on: []
  action: Add repetition_penalty field to SamplingConfig,
          penalize logits for tokens already in context
  test: repeated tokens get lower probability
  primitives_added: ν (frequency tracking)

NODE_009: Wire ffn_inner_dim through generation
  file: src/generator.rs, src/block.rs
  depends_on: [NODE_004]
  action: GenerativeModel::new accepts ffn_inner_dim,
          passes to TransformerStack which passes to each TransformerBlock
  test: model with custom ffn_inner_dim builds and runs
  primitives_added: μ (complete config mapping)

NODE_010: Add confidence-gated generation
  file: src/generator.rs
  depends_on: []
  action: Add min_confidence to generation config,
          halt if generation_confidence drops below threshold
  test: generation stops early on low confidence
  primitives_added: ∝ (irreversibility gate), ∂ (threshold boundary)

NODE_011: Update all T1 primitive documentation
  files: all 13 source files
  depends_on: [NODE_005, NODE_007]
  action: Match documented primitives to actual primitives per ARCHITECTURE.md
  test: grep-verify each module's doc section
  primitives_fixed: κ (documentation accuracy)
```

### Parallel Group 2: MCP Tool Expansion (depends on Group 1)

```
NODE_012: Fix cognition_forward to return rich data
  file: nexcore-mcp/src/tools/cognition.rs
  depends_on: [NODE_009]
  action: Return actual logit values (last position), hidden state stats,
          per-layer attention entropy (not just shapes)
  test: cognition_forward response includes logit_values, hidden_stats
  primitives_fixed: π (data preservation)

NODE_013: Fix cognition_process composability
  file: nexcore-mcp/src/tools/cognition.rs
  depends_on: []
  action: Include generated_logits in response so cognition_perplexity
          can compose with cognition_process output
  test: logits_per_step in response can feed into cognition_perplexity
  primitives_fixed: → (causal chain), σ (pipeline composability)

NODE_014: Add cognition_sample tool
  file: nexcore-mcp/src/tools/cognition.rs, params/cognition.rs
  depends_on: [NODE_008]
  action: MCP tool that takes logits + SamplingConfig, returns sampled token
  test: tool returns valid token from logit distribution
  primitives_added: N, ∂

NODE_015: Add cognition_embed tool
  file: nexcore-mcp/src/tools/cognition.rs, params/cognition.rs
  depends_on: []
  action: MCP tool that takes token IDs, returns embedding vectors
  test: returns tensor shape [seq_len, model_dim]
  primitives_added: μ, λ

NODE_016: Add cognition_attention_weights tool
  file: nexcore-mcp/src/tools/cognition.rs, params/cognition.rs
  depends_on: []
  action: MCP tool that takes tokens, returns raw attention weight matrices
  test: returns [layer][head] weight matrices
  primitives_added: κ, π

NODE_017: Add cognition_confidence tool
  file: nexcore-mcp/src/tools/cognition.rs, params/cognition.rs
  depends_on: []
  action: MCP tool that takes logits, returns confidence + entropy
  test: returns confidence and entropy values
  primitives_added: N, κ

NODE_018: Expose causal toggle in MCP tools
  file: nexcore-mcp/src/params/cognition.rs
  depends_on: []
  action: Add causal: bool param to cognition_forward and cognition_analyze
  test: causal=false produces bidirectional attention
  primitives_exposed: ∂, →

NODE_019: Add sampling presets to MCP
  file: nexcore-mcp/src/params/cognition.rs
  depends_on: [NODE_008]
  action: Add preset field (greedy/creative/custom) + repetition_penalty
  test: preset="creative" uses temperature=0.8, top_k=40, top_p=0.95
  primitives_exposed: N, ∂
```

### Parallel Group 3: Integration + Dispatch (depends on Group 2)

```
NODE_020: Wire new tools into unified.rs dispatch
  file: nexcore-mcp/src/unified.rs
  depends_on: [NODE_014, NODE_015, NODE_016, NODE_017]
  action: Add 4 new dispatch entries + update help catalog
  test: all 12 cognition_* commands dispatched

NODE_021: Update help catalog
  file: nexcore-mcp/src/unified.rs
  depends_on: [NODE_020]
  action: Update cognition category in help to list all 12 tools
  test: help command shows 12 cognition tools
```

### Sequential: Verification (depends on all)

```
NODE_022: Full test suite
  depends_on: [NODE_001..NODE_021]
  action: cargo test -p nexcore-cognition --lib
  gate: all tests pass, zero warnings

NODE_023: Release build
  depends_on: [NODE_022]
  action: cargo build -p nexcore-mcp --release
  gate: binary builds, 12 cognition tools callable

NODE_024: Primitive coverage audit
  depends_on: [NODE_011, NODE_022]
  action: Verify ARCHITECTURE.md primitive matrix matches code
  gate: zero false claims, zero missing primitives
```

---

## DAG Visualization

```
Group 0 (parallel):
  [001]─┐
  [002]─┤
  [003]─┤
  [004]─┼──────────────────────┐
  [005]─┤                      │
  [006]─┤                      │
  [007]─┘                      │
    │                          │
    ▼                          ▼
Group 1 (parallel):        Group 1:
  [008]──────────┐         [009]←(004)
  [010]          │           │
  [011]←(005,007)│           │
    │            │           │
    ▼            ▼           ▼
Group 2 (parallel):
  [012]←(009)
  [013]
  [014]←(008)
  [015]
  [016]
  [017]
  [018]
  [019]←(008)
    │
    ▼
Group 3:
  [020]←(014,015,016,017)
  [021]←(020)
    │
    ▼
Verify:
  [022]←(all)
  [023]←(022)
  [024]←(011,022)
```

---

## Interaction Sequence (Module Call Graph)

The cognitive pipeline's actual runtime call sequence:

```
CognitiveEngine::process()
  │
  ├─▶ GenerativeModel::generate()          [σ: sequence loop]
  │     │
  │     └─▶ for step in 0..max_new_tokens  [ρ: autoregressive recursion]
  │           │
  │           ├─▶ GenerativeModel::forward()
  │           │     │
  │           │     ├─▶ Embedding::forward_batch()     [μ: token→vector]
  │           │     │     └─▶ Embedding::forward() × N  [λ: lookup by position]
  │           │     │
  │           │     ├─▶ PositionalEncoding::forward()   [λ: inject position]
  │           │     │     └─▶ Tensor::add()              [Σ: embed + position]
  │           │     │
  │           │     ├─▶ TransformerStack::forward()     [σ: layer sequence]
  │           │     │     │
  │           │     │     └─▶ for block in blocks        [σ: iterate layers]
  │           │     │           │
  │           │     │           ├─▶ pre_norm_residual()   [π+Σ+∂]
  │           │     │           │     ├─▶ LayerNorm::forward()   [∂: normalize]
  │           │     │           │     └─▶ MultiHeadAttention::forward()  [κ+→]
  │           │     │           │           │
  │           │     │           │           ├─▶ AttentionHead::forward() × H
  │           │     │           │           │     ├─▶ Tensor::matmul() × 4    [N: compute]
  │           │     │           │           │     ├─▶ causal_mask()           [∂+→]
  │           │     │           │           │     ├─▶ apply_mask()            [∂]
  │           │     │           │           │     └─▶ Tensor::softmax()       [N+∅]
  │           │     │           │           │
  │           │     │           │           └─▶ concat + matmul(w_output)  [Σ+μ]
  │           │     │           │
  │           │     │           └─▶ pre_norm_residual()   [π+Σ+∂]
  │           │     │                 ├─▶ LayerNorm::forward()   [∂]
  │           │     │                 └─▶ FeedForward::forward()  [μ+ς]
  │           │     │                       ├─▶ matmul(w1) + bias  [μ]
  │           │     │                       ├─▶ Tensor::gelu()     [ς: state change]
  │           │     │                       └─▶ matmul(w2) + bias  [μ]
  │           │     │
  │           │     └─▶ Tensor::matmul(output_proj)     [μ: hidden→logits]
  │           │
  │           ├─▶ Tensor::row(seq_len - 1)              [λ: last position]
  │           └─▶ sample_token()                        [N+∂: stochastic selection]
  │                 ├─▶ temperature scaling              [N]
  │                 ├─▶ top_k truncation                 [∂: boundary]
  │                 ├─▶ softmax + top_p                  [∂: nucleus boundary]
  │                 └─▶ categorical sample               [N: probability]
  │
  ├─▶ GenerativeModel::forward()                       [σ: analysis pass]
  │     (DEFECT D-006: redundant full forward pass)
  │
  ├─▶ metrics::analyze_attention()                     [κ+N+ν]
  │     ├─▶ shannon_entropy() per row per head          [N+ν]
  │     ├─▶ peak attention tracking                     [N+κ]
  │     ├─▶ sparsity counting                           [N+∂]
  │     └─▶ utilization counting                        [N+∂]
  │
  ├─▶ metrics::generation_confidence() per step        [N+κ]
  │     └─▶ Tensor::softmax() → Tensor::max()
  │
  └─▶ metrics::perplexity()                            [N+κ+∅]
        └─▶ softmax → log(p(token)) → exp(-mean)
```

### MCP Tool Composability Graph (Current)

```
cognition_process ──────► generates tokens + profile + perplexity
                          (MISSING: does NOT return logits)
                              │
                              ✗ BROKEN: cannot feed into cognition_perplexity

cognition_analyze ──────► attention profile only
                          (no generation, no logits)

cognition_forward ──────► shape info only
                          (DISCARDS: actual logits, hidden, weights)
                              │
                              ✗ BROKEN: returns shapes, not data

cognition_entropy ──────► standalone (works)

cognition_perplexity ───► ORPHANED: requires logits no tool produces
```

### MCP Tool Composability Graph (After VDAG)

```
cognition_process ──────► tokens + profile + perplexity + logits
                              │
                              ├──► cognition_perplexity (logits fed in)
                              ├──► cognition_confidence (logits fed in)
                              └──► cognition_sample (logits fed in, re-sample)

cognition_forward ──────► full data: logits, hidden stats, weight entropy
                              │
                              ├──► cognition_perplexity
                              ├──► cognition_confidence
                              └──► cognition_attention_weights (deeper detail)

cognition_embed ────────► embedding vectors
                              └──► standalone analysis

cognition_analyze ──────► attention profile
                              └──► standalone analysis

cognition_entropy ──────► standalone (works, unchanged)

cognition_sample ───────► sample from logits with full config
                              └──► top-k/top-p/temperature/repetition penalty
```

---

## Five Problems Analysis

| # | Category | Finding | Mitigation |
|---|----------|---------|------------|
| 1 | SAFETY | 3 silent `unwrap_or_else` can produce corrupt zeros | Nodes 001-003: propagate errors |
| 2 | EFFICACY | `ffn_inner_dim` dead config, double forward pass | Nodes 004, 009, fix D-006 |
| 3 | CONFIRMATION | MCP tools not composable (orphaned perplexity) | Nodes 012-013: return logits |
| 4 | STRUCTURAL | Primitive documentation 60% inaccurate | Node 011: full update |
| 5 | FUNCTIONAL | No repetition penalty, no confidence gating | Nodes 008, 010: add capabilities |

---

## Execution Order (Topological Sort)

```
Phase 1: [001, 002, 003, 004, 005, 006, 007]  ← parallel, no deps
Phase 2: [008, 009, 010, 011]                   ← parallel, deps on Phase 1
Phase 3: [012, 013, 014, 015, 016, 017, 018, 019]  ← parallel, deps on Phase 2
Phase 4: [020, 021]                             ← sequential, deps on Phase 3
Phase 5: [022, 023, 024]                        ← sequential, verification
```

Total: 24 nodes, 5 phases, 4 parallel groups.
