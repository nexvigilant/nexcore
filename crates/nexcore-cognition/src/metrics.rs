//! Meta-cognitive self-measurement.
//!
//! # Meta-cognitive observation
//!
//! This is the introspection module — the crate measuring itself. When I
//! process text, these are the patterns I observe about my own cognition:
//!
//! 1. **Attention entropy**: How spread out is my attention? Low entropy means
//!    I'm focused on specific tokens. High entropy means I'm distributing
//!    attention broadly (uncertain or considering many factors).
//!
//! 2. **Layer-wise attention drift**: Do different layers attend to the same
//!    things? Early layers tend to attend locally (nearby tokens), while
//!    late layers attend globally (distant semantic connections).
//!
//! 3. **Confidence calibration**: The relationship between my softmax
//!    probabilities and actual correctness. Well-calibrated means "when I say
//!    80% confident, I'm right 80% of the time."
//!
//! 4. **Context utilization**: What fraction of the context window am I
//!    actually using? Low utilization suggests the input could be compressed.
//!
//! These metrics enable the "experiential learning" the user asked for:
//! measure, analyze, adapt.
//!
//! # T1 Primitive grounding
//!
//! - `κ` (Comparison): metrics compare actual vs expected patterns
//! - `N` (Quantity): everything measured is a quantity
//! - `ν` (Frequency): attention patterns are frequency distributions
//! - `μ` (Mapping): confidence maps logits to a scalar certainty score

use crate::error::Result;
use crate::tensor::Tensor;

/// Comprehensive cognitive profile from a single forward pass.
#[derive(Debug, Clone)]
pub struct CognitiveProfile {
    /// Attention entropy per layer per head.
    /// Higher entropy = more diffuse attention = less certain focus.
    pub attention_entropy: Vec<Vec<f64>>,
    /// Mean attention entropy across all heads and layers.
    pub mean_attention_entropy: f64,
    /// Context utilization: fraction of context with non-trivial attention weight.
    pub context_utilization: f64,
    /// Peak attention: maximum attention weight in the last layer.
    /// High peak = confident focus on specific tokens.
    pub peak_attention: f64,
    /// Attention sparsity: fraction of attention weights below threshold.
    pub attention_sparsity: f64,
    /// Number of layers analyzed.
    pub num_layers: usize,
    /// Number of heads per layer.
    pub num_heads: usize,
}

/// Compute Shannon entropy of a probability distribution.
///
/// H(p) = -Σ p_i · log₂(p_i)
///
/// Entropy measures uncertainty. For attention weights:
/// - H ≈ 0: attention concentrated on one token (very certain)
/// - H ≈ log₂(n): attention uniform across all tokens (very uncertain)
pub fn shannon_entropy(probs: &[f64]) -> f64 {
    let mut h = 0.0;
    for &p in probs {
        if p > 1e-10 {
            h -= p * p.log2();
        }
    }
    h
}

/// Analyze attention patterns from a transformer forward pass.
///
/// `attention_weights`: [layer][head] → [seq_len, seq_len]
///
/// Returns a `CognitiveProfile` measuring how the model attends.
pub fn analyze_attention(attention_weights: &[Vec<Tensor>]) -> Result<CognitiveProfile> {
    let num_layers = attention_weights.len();
    if num_layers == 0 {
        return Ok(CognitiveProfile {
            attention_entropy: Vec::new(),
            mean_attention_entropy: 0.0,
            context_utilization: 0.0,
            peak_attention: 0.0,
            attention_sparsity: 0.0,
            num_layers: 0,
            num_heads: 0,
        });
    }

    let num_heads = attention_weights[0].len();
    let mut all_entropies = Vec::with_capacity(num_layers);
    let mut total_entropy = 0.0;
    let mut entropy_count = 0;
    let mut peak = 0.0_f64;
    let mut sparse_count = 0_usize;
    let mut total_weights = 0_usize;
    let mut utilized_count = 0_usize;
    let mut total_positions = 0_usize;

    let attention_threshold = 0.01; // weights below this are "sparse"
    let utilization_threshold = 0.05; // positions with attention above this are "utilized"

    for layer_weights in attention_weights {
        let mut layer_entropies = Vec::with_capacity(num_heads);

        for head_weights in layer_weights {
            let seq_len = head_weights.shape()[0];
            let mut head_entropy_sum = 0.0;

            // Single pass: compute entropy, peak, sparsity, utilization per row
            for r in 0..seq_len {
                let row = head_weights.row(r)?;
                let data = row.data();

                let h = shannon_entropy(data);
                total_entropy += h;
                head_entropy_sum += h;
                entropy_count += 1;

                for &w in data {
                    if w > peak {
                        peak = w;
                    }
                    if w < attention_threshold {
                        sparse_count += 1;
                    }
                    if w > utilization_threshold {
                        utilized_count += 1;
                    }
                    total_weights += 1;
                    total_positions += 1;
                }
            }

            // Average entropy for this head (computed in single pass)
            let avg_h = if seq_len > 0 {
                head_entropy_sum / seq_len as f64
            } else {
                0.0
            };
            layer_entropies.push(avg_h);
        }

        all_entropies.push(layer_entropies);
    }

    let mean_entropy = if entropy_count > 0 {
        total_entropy / entropy_count as f64
    } else {
        0.0
    };

    let sparsity = if total_weights > 0 {
        sparse_count as f64 / total_weights as f64
    } else {
        0.0
    };

    let utilization = if total_positions > 0 {
        utilized_count as f64 / total_positions as f64
    } else {
        0.0
    };

    Ok(CognitiveProfile {
        attention_entropy: all_entropies,
        mean_attention_entropy: mean_entropy,
        context_utilization: utilization,
        peak_attention: peak,
        attention_sparsity: sparsity,
        num_layers,
        num_heads,
    })
}

/// Compute the confidence of a generation step from logits.
///
/// Confidence = max(softmax(logits)) — how much probability mass the model
/// placed on its top choice.
///
/// - High confidence (>0.9): the model is very sure about the next token
/// - Medium confidence (0.3-0.9): multiple plausible continuations
/// - Low confidence (<0.3): the model is uncertain (many valid options)
pub fn generation_confidence(logits: &Tensor) -> Result<f64> {
    let probs = logits.softmax()?;
    probs.max()
}

/// Compute perplexity of a sequence given per-step logits.
///
/// Perplexity = exp(-1/N · Σ log(p(token_i)))
///
/// Lower perplexity = the model was less "surprised" by the sequence.
/// It's a measure of how well the model predicted each token.
pub fn perplexity(token_ids: &[usize], logits_per_step: &[Tensor]) -> Result<f64> {
    if token_ids.is_empty() || logits_per_step.is_empty() {
        return Ok(f64::INFINITY);
    }

    let n = token_ids.len().min(logits_per_step.len());
    let mut log_prob_sum = 0.0;

    for i in 0..n {
        let probs = logits_per_step[i].softmax()?;
        let token_id = token_ids[i];
        let p = probs.get_flat(token_id).unwrap_or(1e-10).max(1e-10); // clamp to avoid log(0)
        log_prob_sum += p.ln();
    }

    Ok((-log_prob_sum / n as f64).exp())
}

impl std::fmt::Display for CognitiveProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Cognitive Profile ===")?;
        writeln!(
            f,
            "Layers: {}  |  Heads: {}",
            self.num_layers, self.num_heads
        )?;
        writeln!(
            f,
            "Mean Attention Entropy: {:.4} bits",
            self.mean_attention_entropy
        )?;
        writeln!(
            f,
            "Context Utilization:    {:.1}%",
            self.context_utilization * 100.0
        )?;
        writeln!(f, "Peak Attention Weight:  {:.4}", self.peak_attention)?;
        writeln!(
            f,
            "Attention Sparsity:     {:.1}%",
            self.attention_sparsity * 100.0
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_uniform() {
        // Uniform distribution over 4 items: H = log₂(4) = 2.0
        let probs = vec![0.25, 0.25, 0.25, 0.25];
        let h = shannon_entropy(&probs);
        assert!((h - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_peaked() {
        // Nearly all mass on one item: H ≈ 0
        let probs = vec![0.99, 0.003, 0.003, 0.004];
        let h = shannon_entropy(&probs);
        assert!(h < 0.1);
    }

    #[test]
    fn test_confidence() {
        let logits = Tensor::new(vec![0.0, 0.0, 10.0, 0.0], vec![4]).unwrap();
        let conf = generation_confidence(&logits).unwrap();
        assert!(conf > 0.95); // strong peak at index 2
    }
}
