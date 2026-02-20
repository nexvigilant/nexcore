//! Cognition MCP tools — transformer algorithm as strict Rust.
//!
//! # T1 Grounding
//! - κ (Comparison): attention compares query-key pairs
//! - σ (Sequence): generation produces token sequences
//! - ρ (Recursion): autoregressive loop feeds output back as input
//! - N (Quantity): entropy, perplexity, confidence — all quantities
//! - ∂ (Boundary): causal mask enforces temporal boundary
//! - ν (Frequency): repetition penalty tracks token frequency

use nexcore_cognition::block::TransformerConfig;
use nexcore_cognition::generator::StopReason;
use nexcore_cognition::metrics;
use nexcore_cognition::pipeline::CognitiveEngine;
use nexcore_cognition::sample::SamplingConfig;
use nexcore_cognition::tensor::Tensor;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    CognitionAnalyzeParams, CognitionConfidenceParams, CognitionEmbedParams,
    CognitionEntropyParams, CognitionForwardParams, CognitionPerplexityParams,
    CognitionProcessParams, CognitionSampleParams,
};

/// Helper: build TransformerConfig with optional ffn_inner_dim.
fn build_config(
    model_dim: usize,
    num_heads: usize,
    num_layers: usize,
    vocab_size: usize,
    max_seq_len: usize,
    ffn_inner_dim: Option<usize>,
) -> TransformerConfig {
    let mut config =
        TransformerConfig::new(model_dim, num_heads, num_layers, vocab_size, max_seq_len);
    if let Some(dim) = ffn_inner_dim {
        config.ffn_inner_dim = dim;
    }
    config
}

// ============================================================================
// Full Pipeline: Generate + Measure
// ============================================================================

/// Run the full cognitive pipeline: embed → attend → generate → measure.
///
/// Returns generated tokens, perplexity, per-step confidence, generated logits
/// (for composability with cognition_perplexity), and attention profile.
pub fn cognition_process(params: CognitionProcessParams) -> Result<CallToolResult, McpError> {
    let config = build_config(
        params.model_dim,
        params.num_heads,
        params.num_layers,
        params.vocab_size,
        params.max_seq_len,
        params.ffn_inner_dim,
    );

    let mut rng = nexcore_cognition::make_rng(params.seed);

    let engine = CognitiveEngine::new(config, &mut rng)
        .map_err(|e| McpError::invalid_params(format!("Failed to create engine: {e}"), None))?;

    let sampling = SamplingConfig {
        temperature: params.temperature,
        top_k: params.top_k,
        top_p: params.top_p,
        repetition_penalty: params.repetition_penalty,
    };

    let output = engine
        .process_gated(
            &params.prompt,
            params.max_new_tokens,
            &sampling,
            params.stop_token,
            params.min_confidence,
            &mut rng,
        )
        .map_err(|e| McpError::invalid_params(format!("Generation failed: {e}"), None))?;

    // Extract generated logits for composability (NODE_013)
    let generated_logits: Vec<Vec<f64>> = output
        .generation
        .generated_logits
        .iter()
        .map(|t| t.data().to_vec())
        .collect();

    let stop_reason_str = match &output.generation.stop_reason {
        StopReason::MaxTokens => "max_tokens".to_string(),
        StopReason::StopToken(id) => format!("stop_token({id})"),
        StopReason::LowConfidence {
            step,
            confidence,
            threshold,
        } => format!("low_confidence(step={step}, conf={confidence:.4}, threshold={threshold})"),
        StopReason::MaxSeqLen => "max_seq_len".to_string(),
    };

    let response = serde_json::json!({
        "status": "success",
        "stop_reason": stop_reason_str,
        "prompt_tokens": output.generation.prompt_len,
        "generated_tokens": output.generation.num_generated(),
        "total_tokens": output.generation.tokens.len(),
        "generated_token_ids": output.generation.generated_tokens(),
        "all_token_ids": output.generation.tokens,
        "generated_logits": generated_logits,
        "perplexity": format!("{:.4}", output.perplexity),
        "step_confidences": output.step_confidences.iter()
            .map(|c| format!("{c:.4}"))
            .collect::<Vec<_>>(),
        "mean_confidence": format!("{:.4}",
            if output.step_confidences.is_empty() { 0.0 }
            else { output.step_confidences.iter().sum::<f64>() / output.step_confidences.len() as f64 }),
        "profile": {
            "num_layers": output.profile.num_layers,
            "num_heads": output.profile.num_heads,
            "mean_attention_entropy": format!("{:.4}", output.profile.mean_attention_entropy),
            "context_utilization": format!("{:.1}%", output.profile.context_utilization * 100.0),
            "peak_attention": format!("{:.4}", output.profile.peak_attention),
            "attention_sparsity": format!("{:.1}%", output.profile.attention_sparsity * 100.0),
        },
        "config": {
            "vocab_size": params.vocab_size,
            "model_dim": params.model_dim,
            "num_heads": params.num_heads,
            "num_layers": params.num_layers,
            "ffn_inner_dim": params.ffn_inner_dim.unwrap_or(4 * params.model_dim),
            "max_seq_len": params.max_seq_len,
            "temperature": params.temperature,
            "top_k": params.top_k,
            "top_p": params.top_p,
            "repetition_penalty": params.repetition_penalty,
        },
        "grounding": {
            "tier": "T2-C",
            "dominant": "σ+ρ (Sequence + Recursion)",
            "primitives": ["κ", "σ", "ρ", "N", "∂", "μ", "ν"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Analyze: Attention Profile Only (No Generation)
// ============================================================================

/// Analyze attention patterns for an existing token sequence.
///
/// Returns the cognitive profile: entropy, sparsity, utilization, peak attention.
pub fn cognition_analyze(params: CognitionAnalyzeParams) -> Result<CallToolResult, McpError> {
    let config = build_config(
        params.model_dim,
        params.num_heads,
        params.num_layers,
        params.vocab_size,
        params.max_seq_len,
        params.ffn_inner_dim,
    );

    let mut rng = nexcore_cognition::make_rng(params.seed);

    let engine = CognitiveEngine::new(config, &mut rng)
        .map_err(|e| McpError::invalid_params(format!("Failed to create engine: {e}"), None))?;

    let profile = engine
        .analyze_with_mask(&params.tokens, params.causal)
        .map_err(|e| McpError::invalid_params(format!("Analysis failed: {e}"), None))?;

    let response = serde_json::json!({
        "status": "success",
        "tokens_analyzed": params.tokens.len(),
        "causal": params.causal,
        "profile": {
            "num_layers": profile.num_layers,
            "num_heads": profile.num_heads,
            "mean_attention_entropy": format!("{:.4}", profile.mean_attention_entropy),
            "context_utilization": format!("{:.1}%", profile.context_utilization * 100.0),
            "peak_attention": format!("{:.4}", profile.peak_attention),
            "attention_sparsity": format!("{:.1}%", profile.attention_sparsity * 100.0),
            "per_layer_entropy": profile.attention_entropy.iter()
                .enumerate()
                .map(|(l, heads)| {
                    serde_json::json!({
                        "layer": l,
                        "head_entropies": heads.iter().map(|h| format!("{h:.4}")).collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>(),
        },
        "grounding": {
            "tier": "T2-P",
            "dominant": "κ (Comparison)",
            "primitives": ["κ", "N", "ν"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Forward Pass (Rich Data — logits, hidden stats, attention entropy)
// ============================================================================

/// Run a single forward pass and return rich data.
///
/// Returns actual logit values (last position), hidden state statistics,
/// per-layer attention entropy, and predicted token with confidence.
pub fn cognition_forward(params: CognitionForwardParams) -> Result<CallToolResult, McpError> {
    let config = build_config(
        params.model_dim,
        params.num_heads,
        params.num_layers,
        params.vocab_size,
        params.max_seq_len,
        params.ffn_inner_dim,
    );

    let mut rng = nexcore_cognition::make_rng(params.seed);

    let engine = CognitiveEngine::new(config, &mut rng)
        .map_err(|e| McpError::invalid_params(format!("Failed to create engine: {e}"), None))?;

    let fwd = engine
        .model
        .forward_with_mask(&params.tokens, params.causal)
        .map_err(|e| McpError::invalid_params(format!("Forward pass failed: {e}"), None))?;

    // Extract last position logits (actual values, not just shape)
    let seq_len = params.tokens.len();
    let last_logits = fwd.logits.row(seq_len - 1).map_err(|e| {
        McpError::invalid_params(format!("Failed to extract last logits: {e}"), None)
    })?;
    let confidence = metrics::generation_confidence(&last_logits).unwrap_or(0.0);
    let argmax = last_logits.argmax().unwrap_or(0);

    // Hidden state statistics
    let hidden_mean = fwd.hidden.mean().unwrap_or(0.0);
    let hidden_std = fwd.hidden.std_dev().unwrap_or(0.0);

    // Per-layer attention entropy
    let profile = metrics::analyze_attention(&fwd.attention_weights).ok();

    let response = serde_json::json!({
        "status": "success",
        "input_tokens": params.tokens.len(),
        "causal": params.causal,
        "logits_shape": fwd.logits.shape(),
        "hidden_shape": fwd.hidden.shape(),
        "last_position": {
            "predicted_token": argmax,
            "confidence": format!("{confidence:.4}"),
            "logit_values": last_logits.data(),
        },
        "hidden_stats": {
            "mean": format!("{hidden_mean:.6}"),
            "std_dev": format!("{hidden_std:.6}"),
        },
        "attention": {
            "num_layers": fwd.attention_weights.len(),
            "heads_per_layer": if fwd.attention_weights.is_empty() { 0 } else { fwd.attention_weights[0].len() },
            "profile": profile.map(|p| serde_json::json!({
                "mean_entropy": format!("{:.4}", p.mean_attention_entropy),
                "peak_attention": format!("{:.4}", p.peak_attention),
                "sparsity": format!("{:.1}%", p.attention_sparsity * 100.0),
                "utilization": format!("{:.1}%", p.context_utilization * 100.0),
            })),
        },
        "grounding": {
            "tier": "T2-P",
            "dominant": "μ (Mapping)",
            "primitives": ["μ", "σ", "→", "N", "κ"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Shannon Entropy
// ============================================================================

/// Compute Shannon entropy of a probability distribution.
///
/// H(p) = -Σ pᵢ · log₂(pᵢ)
pub fn cognition_entropy(params: CognitionEntropyParams) -> Result<CallToolResult, McpError> {
    if params.probabilities.is_empty() {
        return Err(McpError::invalid_params(
            "Probabilities must not be empty.".to_string(),
            None,
        ));
    }

    let sum: f64 = params.probabilities.iter().sum();
    let entropy = metrics::shannon_entropy(&params.probabilities);
    let max_entropy = (params.probabilities.len() as f64).log2();
    let normalized = if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    };

    let response = serde_json::json!({
        "status": "success",
        "entropy_bits": format!("{entropy:.6}"),
        "max_entropy_bits": format!("{max_entropy:.6}"),
        "normalized_entropy": format!("{normalized:.4}"),
        "distribution_sum": format!("{sum:.6}"),
        "num_categories": params.probabilities.len(),
        "interpretation": if normalized < 0.3 {
            "Concentrated (high certainty)"
        } else if normalized < 0.7 {
            "Moderate spread"
        } else {
            "Diffuse (high uncertainty)"
        },
        "grounding": {
            "tier": "T1",
            "dominant": "N (Quantity)",
            "primitives": ["N", "ν"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Perplexity
// ============================================================================

/// Compute perplexity of a token sequence given per-step logits.
///
/// Perplexity = exp(-1/N · Σ log(p(tokenᵢ)))
pub fn cognition_perplexity(params: CognitionPerplexityParams) -> Result<CallToolResult, McpError> {
    if params.token_ids.is_empty() || params.logits_per_step.is_empty() {
        return Err(McpError::invalid_params(
            "Both token_ids and logits_per_step must be non-empty.".to_string(),
            None,
        ));
    }

    let tensors: Vec<Tensor> = params
        .logits_per_step
        .into_iter()
        .map(|logits| {
            let len = logits.len();
            Tensor::new(logits, vec![len])
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| McpError::invalid_params(format!("Invalid logits: {e}"), None))?;

    let ppl = metrics::perplexity(&params.token_ids, &tensors).map_err(|e| {
        McpError::invalid_params(format!("Perplexity computation failed: {e}"), None)
    })?;

    let response = serde_json::json!({
        "status": "success",
        "perplexity": format!("{ppl:.4}"),
        "num_tokens": params.token_ids.len(),
        "num_steps": tensors.len(),
        "interpretation": if ppl < 5.0 {
            "Very low perplexity (highly predictable)"
        } else if ppl < 20.0 {
            "Low perplexity (well-predicted)"
        } else if ppl < 100.0 {
            "Moderate perplexity"
        } else {
            "High perplexity (poorly predicted / surprised)"
        },
        "grounding": {
            "tier": "T1",
            "dominant": "N (Quantity)",
            "primitives": ["N", "κ", "ν"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Embed: Token IDs → Embedding Vectors
// ============================================================================

/// Embed token IDs into continuous vectors.
///
/// Returns the embedding vectors for each token in the input.
pub fn cognition_embed(params: CognitionEmbedParams) -> Result<CallToolResult, McpError> {
    if params.tokens.is_empty() {
        return Err(McpError::invalid_params(
            "Tokens must not be empty.".to_string(),
            None,
        ));
    }

    let mut rng = nexcore_cognition::make_rng(params.seed);
    let embedding =
        nexcore_cognition::embedding::Embedding::new(params.vocab_size, params.model_dim, &mut rng);

    let embedded = embedding
        .forward_batch(&params.tokens)
        .map_err(|e| McpError::invalid_params(format!("Embedding failed: {e}"), None))?;

    // Return first few values per token (full vectors can be huge)
    let preview_dim = params.model_dim.min(8);
    let token_previews: Vec<serde_json::Value> = params
        .tokens
        .iter()
        .enumerate()
        .map(|(i, &tok)| {
            let row = embedded.row(i).ok();
            serde_json::json!({
                "token_id": tok,
                "embedding_preview": row.map(|r| r.data()[..preview_dim].to_vec()),
            })
        })
        .collect();

    let response = serde_json::json!({
        "status": "success",
        "num_tokens": params.tokens.len(),
        "embedding_dim": params.model_dim,
        "shape": embedded.shape(),
        "embeddings": token_previews,
        "grounding": {
            "tier": "T1",
            "dominant": "μ (Mapping)",
            "primitives": ["μ", "λ"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Sample: Logits → Token (with full sampling config)
// ============================================================================

/// Sample a token from a logit vector with full sampling controls.
///
/// Supports temperature, top-k, top-p, and repetition penalty.
pub fn cognition_sample(params: CognitionSampleParams) -> Result<CallToolResult, McpError> {
    if params.logits.is_empty() {
        return Err(McpError::invalid_params(
            "Logits must not be empty.".to_string(),
            None,
        ));
    }

    let len = params.logits.len();
    let logits = Tensor::new(params.logits, vec![len])
        .map_err(|e| McpError::invalid_params(format!("Invalid logits: {e}"), None))?;

    let config = SamplingConfig {
        temperature: params.temperature,
        top_k: params.top_k,
        top_p: params.top_p,
        repetition_penalty: params.repetition_penalty,
    };

    let mut rng = nexcore_cognition::make_rng(params.seed);

    let token = nexcore_cognition::sample::sample_token_with_context(
        &logits,
        &config,
        &params.context,
        &mut rng,
    )
    .map_err(|e| McpError::invalid_params(format!("Sampling failed: {e}"), None))?;

    let confidence = metrics::generation_confidence(&logits).unwrap_or(0.0);

    let response = serde_json::json!({
        "status": "success",
        "sampled_token": token,
        "confidence": format!("{confidence:.4}"),
        "vocab_size": len,
        "config": {
            "temperature": params.temperature,
            "top_k": params.top_k,
            "top_p": params.top_p,
            "repetition_penalty": params.repetition_penalty,
            "context_length": params.context.len(),
        },
        "grounding": {
            "tier": "T1",
            "dominant": "N+∂ (Quantity + Boundary)",
            "primitives": ["N", "∂", "ν", "κ"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Confidence: Logits → Generation Confidence
// ============================================================================

/// Compute generation confidence from logits.
///
/// Confidence = max(softmax(logits)) — probability mass on top choice.
pub fn cognition_confidence(params: CognitionConfidenceParams) -> Result<CallToolResult, McpError> {
    if params.logits.is_empty() {
        return Err(McpError::invalid_params(
            "Logits must not be empty.".to_string(),
            None,
        ));
    }

    let len = params.logits.len();
    let logits = Tensor::new(params.logits, vec![len])
        .map_err(|e| McpError::invalid_params(format!("Invalid logits: {e}"), None))?;

    let confidence = metrics::generation_confidence(&logits).map_err(|e| {
        McpError::invalid_params(format!("Confidence computation failed: {e}"), None)
    })?;

    let probs = logits
        .softmax()
        .map_err(|e| McpError::invalid_params(format!("Softmax failed: {e}"), None))?;
    let entropy = metrics::shannon_entropy(probs.data());
    let max_entropy = (len as f64).log2();
    let normalized_entropy = if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    };

    let argmax = logits.argmax().unwrap_or(0);

    let response = serde_json::json!({
        "status": "success",
        "confidence": format!("{confidence:.4}"),
        "predicted_token": argmax,
        "entropy_bits": format!("{entropy:.4}"),
        "normalized_entropy": format!("{normalized_entropy:.4}"),
        "vocab_size": len,
        "interpretation": if confidence > 0.9 {
            "Very high confidence (near-certain)"
        } else if confidence > 0.5 {
            "High confidence"
        } else if confidence > 0.3 {
            "Moderate confidence"
        } else {
            "Low confidence (uncertain)"
        },
        "grounding": {
            "tier": "T1",
            "dominant": "N+κ (Quantity + Comparison)",
            "primitives": ["N", "κ"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
