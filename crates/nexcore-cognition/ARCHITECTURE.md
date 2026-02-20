# nexcore-cognition Architecture

## Structure → Function → Primitive Map

Every struct, its cognitive function, and the T1 primitives it embodies.

### Layer 0: Substrate

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `Tensor` | tensor.rs:32 | Numerical substrate | `Vec<f64>`, `Vec<usize>` shape | Dense row-major array | N, Σ, λ, ∅, ∂ | Every computation flows through this |
| `CognitionError` | error.rs:20 | Failure semantics | Violation context | Typed error variant | ∂, κ, ∅, → | Boundary violations become typed signals |

### Layer 1: Representation

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `Embedding` | embedding.rs:29 | Symbol→vector mapping | `token_id: usize` | `Tensor [embed_dim]` | μ, λ, ∂, ∅ | Discrete symbols enter continuous space |
| `PositionalEncoding` | embedding.rs:89 | Sequence position injection | `max_seq_len`, `embed_dim` | `Tensor [max_seq_len, embed_dim]` | λ, σ, N, ν, ∅ | Sinusoidal frequency signatures encode position |

### Layer 2: Attention

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `AttentionConfig` | attention.rs:31 | Head geometry | `model_dim`, `num_heads` | `head_dim` computed | ∂, N, κ | Validates divisibility constraint |
| `AttentionHead` | attention.rs:58 | Scaled dot-product attention | `Tensor [seq, model_dim]` | `AttentionOutput` | κ, →, N, μ, λ, ∅, ∂ | Q×K^T/√d_k → softmax → weights×V |
| `AttentionOutput` | attention.rs:125 | Attention result carrier | — | `output` + `weights` | π, N | Preserves weights for introspection |
| `MultiHeadAttention` | attention.rs:147 | Parallel aspect attention | `Tensor [seq, model_dim]` | `MultiHeadOutput` | κ, →, σ, μ, Σ, λ | Heads attend to different aspects, concat+project |
| `MultiHeadOutput` | attention.rs:218 | Multi-head result carrier | — | `output` + `head_weights` | π, Σ | Preserves per-head weights |

### Layer 3: Transformation

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `FeedForwardConfig` | feed_forward.rs:30 | FFN geometry | `model_dim`, `inner_dim` | Config struct | N, ∂ | 4× expansion ratio |
| `FeedForward` | feed_forward.rs:61 | Nonlinear transformation | `Tensor [seq, model_dim]` | `Tensor [seq, model_dim]` | μ, ς, N, ∅, Σ | GELU(x·W₁+b₁)·W₂+b₂ |
| `LayerNorm` | normalize.rs:34 | Signal stability | `Tensor` any shape | `Tensor` same shape | ∂, N, ∅, μ | γ·(x-μ)/√(σ²+ε)+β |

### Layer 4: Composition

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `TransformerConfig` | block.rs:59 | Full model geometry | 6 hyperparameters | Config struct | N, ∂ | `ffn_inner_dim` flows through Stack→Block→FFN |
| `TransformerBlock` | block.rs:37 | One layer of thought | `Tensor [seq, model_dim]` | `BlockOutput` | σ, ∃, π, Σ, ∂, μ, κ | Pre-norm residual attention + FFN |
| `BlockOutput` | block.rs:50 | Block result carrier | — | `hidden` + `attention_weights` | π, N | Preserves weights through stack |
| `TransformerStack` | block.rs:141 | N layers of thought | `Tensor [seq, model_dim]` | `StackOutput` | σ, ρ, π, Σ, ∂ | Sequential block application + final norm |
| `StackOutput` | block.rs:150 | Stack result carrier | — | `hidden` + `all_attention_weights` | π, σ | Full layer-wise weight preservation |

### Layer 5: Generation

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `SamplingConfig` | sample.rs:32 | Stochastic control | `temperature`, `top_k`, `top_p`, `repetition_penalty` | Config struct | N, ∂, ν | Creativity vs. determinism knob + frequency penalty |
| `GenerativeModel` | generator.rs:40 | Complete model | All layer configs | Forward + generate | σ, →, μ, ∂, ∝, ∅ | embed→position→transform→project |
| `ForwardOutput` | generator.rs:159 | Forward pass carrier | — | `logits` + `hidden` + `attention_weights` | π, N, σ | Full model state snapshot |
| `StopReason` | generator.rs:38 | Halt classification | — | MaxTokens, StopToken, LowConfidence, MaxSeqLen | ∂, κ, ∃ | Why generation terminated |
| `GenerationResult` | generator.rs:183 | Generation carrier | — | `tokens` + `prompt_len` + `generated_logits` + `stop_reason` | σ, ∝, π, N | Irreversible token sequence with halt reason |

### Layer 6: Pipeline + Measurement

| Struct | File:Line | Function | Inputs | Outputs | Actual Primitives | Role |
|--------|-----------|----------|--------|---------|-------------------|------|
| `CognitiveEngine` | pipeline.rs:31 | Top-level orchestrator | `TransformerConfig` | process / analyze | σ, →, μ, ∂ | Unified generate+measure interface |
| `CognitiveOutput` | pipeline.rs:41 | Full pipeline carrier | — | generation + profile + confidences + perplexity | π, N, κ, σ | Everything needed for introspection |
| `CognitiveProfile` | metrics.rs:37 | Attention introspection | `attention_weights` | entropy, utilization, sparsity, peak | κ, N, ν, ∂, ∅ | Self-measurement of attention patterns |

### Standalone Functions

| Function | File:Line | Function | Actual Primitives |
|----------|-----------|----------|-------------------|
| `causal_mask` | mask.rs:34 | Temporal boundary | ∂, →, ∅, λ |
| `apply_mask` | mask.rs:52 | Mask application | ∂, Σ |
| `padding_mask` | mask.rs:62 | Length boundary | ∂, ∅, λ |
| `residual_connection` | residual.rs:30 | Signal preservation | π, Σ |
| `pre_norm_residual` | residual.rs:45 | Stable residual | π, Σ, ∂ |
| `shannon_entropy` | metrics.rs:63 | Uncertainty measure | N, ν, ∅ |
| `analyze_attention` | metrics.rs:78 | Profile extraction | κ, N, ν, ∂, ∅ |
| `generation_confidence` | metrics.rs:191 | Step confidence | N, κ |
| `perplexity` | metrics.rs:202 | Sequence surprise | N, κ, ∅ |
| `sample_token` | sample.rs:89 | Stochastic selection | N, ∂, κ, ∅ |
| `sample_token_with_context` | sample.rs:98 | Selection with repetition penalty | N, ∂, κ, ν, ∅ |
| `make_rng` | lib.rs:66 | RNG factory | N |

---

## Data Flow Graph

```
                        token_ids: &[usize]
                              │
                    ┌─────────▼─────────┐
                    │    Embedding       │  μ: symbol→vector
                    │ [vocab, embed_dim] │
                    └─────────┬─────────┘
                              │ Tensor [seq, embed_dim]
                    ┌─────────▼─────────┐
                    │ PositionalEncoding │  λ: inject position
                    │ [max_seq, embed]   │  ν: frequency signatures
                    └─────────┬─────────┘
                              │ Tensor [seq, model_dim]
               ┌──────────────▼──────────────┐
               │     TransformerStack         │
               │  ┌───────────────────────┐   │
               │  │   TransformerBlock    │←──│── × num_layers (σ)
               │  │  ┌─────────────────┐  │   │
               │  │  │ LayerNorm (∂)   │  │   │
               │  │  │ MultiHeadAttn   │  │   │
               │  │  │  ├─ Head₁ (κ)   │  │   │
               │  │  │  ├─ Head₂ (κ)   │  │   │
               │  │  │  └─ Concat+Proj  │  │   │
               │  │  │ + Residual (π,Σ) │  │   │
               │  │  ├─────────────────┤  │   │
               │  │  │ LayerNorm (∂)   │  │   │
               │  │  │ FeedForward (μ) │  │   │
               │  │  │ GELU(x·W₁+b₁)  │  │   │
               │  │  │ ·W₂+b₂         │  │   │
               │  │  │ + Residual (π,Σ) │  │   │
               │  │  └─────────────────┘  │   │
               │  └───────────────────────┘   │
               │  LayerNorm (final) (∂)       │
               └──────────────┬──────────────┘
                              │ Tensor [seq, model_dim]
                    ┌─────────▼─────────┐
                    │  Output Projection │  μ: hidden→logits
                    │ [model_dim, vocab]  │
                    └─────────┬─────────┘
                              │ Tensor [seq, vocab_size]
                    ┌─────────▼─────────┐
                    │  sample_token      │  ∂: top-k/top-p boundary
                    │  temperature → softmax  │  N: probability selection
                    │  → categorical sample   │
                    └─────────┬─────────┘
                              │ usize (next token)
                              │
                    ┌─────────▼─────────┐
                    │  Generation Loop   │  ρ: feeds back (autoregressive)
                    │  append → re-run   │  ∝: tokens committed irreversibly
                    │  until stop/max    │  σ: sequential token output
                    └─────────┬─────────┘
                              │
                    ┌─────────▼─────────┐
                    │   Metrics          │  κ: compare distributions
                    │  entropy, ppl,     │  N: quantify attention
                    │  confidence, profile│  ν: frequency analysis
                    └────────────────────┘
```

---

## Primitive Coverage Matrix

Actual primitives present in code (not documentation claims):

| Primitive | tensor | embed | pos_enc | attn | mask | norm | ffn | residual | block | gen | sample | metrics | pipeline | Count |
|-----------|--------|-------|---------|------|------|------|-----|----------|-------|-----|--------|---------|----------|-------|
| N (Qty) | **X** | | | **X** | | **X** | **X** | | | | **X** | **X** | | 6 |
| Σ (Sum) | **X** | | | **X** | **X** | | **X** | **X** | **X** | | | **X** | | 7 |
| κ (Comp) | | | | **X** | | | | | | | **X** | **X** | | 3 |
| μ (Map) | | **X** | | **X** | | **X** | **X** | | **X** | **X** | | | **X** | 6 |
| σ (Seq) | | | | | | | | | **X** | **X** | | | **X** | 3 |
| → (Cause) | | | | **X** | **X** | | | | | **X** | | | **X** | 4 |
| ∂ (Bound) | **X** | **X** | | **X** | **X** | **X** | | **X** | **X** | **X** | **X** | **X** | **X** | 11 |
| λ (Loc) | **X** | **X** | **X** | **X** | **X** | | | | | **X** | | | | 6 |
| π (Pers) | | | | **X** | | | | **X** | **X** | **X** | | | **X** | 5 |
| ∅ (Void) | **X** | **X** | **X** | | **X** | **X** | **X** | | | **X** | **X** | **X** | | 8 |
| ∝ (Irrev) | | | | | | | | | | **X** | | | | 1 |
| ν (Freq) | | | **X** | | | | | | | | **X** | **X** | | 3 |
| ρ (Recur) | | | | | | | | | | | | | | 0 |
| ς (State) | | | | | | | **X** | | | | | | | 1 |
| ∃ (Exist) | | | | | | | | | **X** | | | | | 1 |

**Dominant:** ∂ (11/13), ∅ (8/13), Σ (7/13), N (6/13), μ (6/13), λ (6/13)
**Absent:** ρ (recursion — generation loop is iterative, not recursive)
**Weak:** ∃ (1), ∝ (1), ς (1), ν (2)

---

## Identified Defects

| ID | Module | Issue | Violated Primitive | Severity | Status |
|----|--------|-------|--------------------|----------|--------|
| D-001 | embedding.rs:119 | `unwrap_or_else` silently produces zeros on shape failure | ∂ (boundary) | HIGH | RESOLVED (cosmetic match) |
| D-002 | mask.rs:43 | `unwrap_or_else` silently produces zeros on shape failure | ∂ (boundary) | HIGH | OPEN (infallible by construction) |
| D-003 | mask.rs:71 | `unwrap_or_else` silently produces zeros on shape failure | ∂ (boundary) | HIGH | OPEN (infallible by construction) |
| D-004 | block.rs | `TransformerBlock::new` ignored `ffn_inner_dim` | μ (mapping broken) | MEDIUM | **RESOLVED** — `with_ffn_dim` added |
| D-005 | metrics.rs | Double iteration over attention rows | N (waste) | LOW | **RESOLVED** — single-pass accumulation |
| D-006 | pipeline.rs:88 | Double forward pass (generate already ran forward at each step) | N (waste) | MEDIUM | OPEN (architectural — analysis requires full-sequence forward) |
| D-007 | generator.rs | Phantom `ν` primitive claimed but no frequency tracking | κ (false claim) | LOW | **RESOLVED** — primitives corrected to →, ∂ |
| D-008 | Cargo.toml | `serde` dependency declared but unused | ∅ (dead dep) | LOW | **RESOLVED** — removed |

---

## Capability Gap → Primitive Root Cause

| Gap | Missing Primitive | Why | Status |
|-----|-------------------|-----|--------|
| No KV cache | ς (State) | No mutable runtime state between forward passes | OPEN |
| ~~No repetition penalty~~ | ~~ν (Frequency)~~ | ~~Cannot track token frequency~~ | **RESOLVED** — `sample_token_with_context` + `repetition_penalty` |
| No RoPE | ν (Frequency) | Position encoding doesn't use rotary frequency | OPEN |
| No save/load | π (Persistence) | Model weights have no persistence mechanism | OPEN |
| No batching | Σ (Sum) | Can't aggregate across batch dimension | OPEN |
| ~~No confidence gating~~ | ~~∝ (Irreversibility)~~ | ~~Can't gate generation on confidence~~ | **RESOLVED** — `generate_gated` + `StopReason::LowConfidence` |
| No token type system | ∃ (Existence) | Tokens are raw usize, no type distinction | OPEN |
| No recursive attention | ρ (Recursion) | Attention is flat, not tree-structured | OPEN |
| ~~No causal toggle~~ | ~~∂ (Boundary)~~ | ~~Causal masking hardcoded to true~~ | **RESOLVED** — `forward_with_mask` + `analyze_with_mask` |
