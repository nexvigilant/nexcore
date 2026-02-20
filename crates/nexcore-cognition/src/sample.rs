//! Sampling strategies for token generation.
//!
//! # Meta-cognitive observation
//!
//! After my forward pass produces logits (raw scores for each possible next token),
//! I must choose ONE token. This choice is where creativity meets precision.
//!
//! - **Temperature** controls randomness: low temperature → confident/deterministic,
//!   high temperature → creative/diverse. At T=0, I always pick the highest-scoring
//!   token (greedy). At T=1, I sample from the raw probability distribution.
//!
//! - **Top-k** limits choices to the k most probable tokens, preventing the model
//!   from ever choosing very unlikely tokens (which tend to be nonsensical).
//!
//! - **Top-p (nucleus)** dynamically limits choices to the smallest set of tokens
//!   whose cumulative probability exceeds p. Adaptive — in confident contexts
//!   (peaked distribution), few tokens qualify. In ambiguous contexts (flat
//!   distribution), more tokens are considered.
//!
//! # T1 Primitive grounding
//!
//! - `N` (Quantity): probabilities are quantities being compared
//! - `∂` (Boundary): top-k and top-p draw boundaries on the candidate set
//! - `ν` (Frequency): repetition penalty tracks token frequency in context
//! - `κ` (Comparison): temperature and top-k compare logit magnitudes

use crate::error::{CognitionError, Result};
use crate::tensor::Tensor;

/// Sampling configuration.
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Temperature: scales logits before softmax. Lower = more deterministic.
    pub temperature: f64,
    /// Top-k: keep only the k highest-probability tokens. 0 = disabled.
    pub top_k: usize,
    /// Top-p (nucleus): keep tokens until cumulative probability exceeds p. 1.0 = disabled.
    pub top_p: f64,
    /// Repetition penalty: penalize logits for tokens already in context.
    /// 1.0 = disabled. >1.0 = penalize repeats. <1.0 = encourage repeats.
    pub repetition_penalty: f64,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            temperature: 1.0,
            top_k: 0,
            top_p: 1.0,
            repetition_penalty: 1.0,
        }
    }
}

impl SamplingConfig {
    /// Greedy sampling: always pick the most probable token.
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_k: 1,
            top_p: 1.0,
            repetition_penalty: 1.0,
        }
    }

    /// Standard creative sampling.
    pub fn creative() -> Self {
        Self {
            temperature: 0.8,
            top_k: 40,
            top_p: 0.95,
            repetition_penalty: 1.2,
        }
    }
}

/// Sample a token index from a logits vector.
///
/// Steps:
/// 1. Apply repetition penalty
/// 2. Apply temperature scaling
/// 3. Apply top-k filtering
/// 4. Apply top-p (nucleus) filtering
/// 5. Softmax → probabilities
/// 6. Sample from the distribution
///
/// `context`: tokens already generated — used for repetition penalty.
/// Pass an empty slice to disable repetition penalty regardless of config.
pub fn sample_token(
    logits: &Tensor,
    config: &SamplingConfig,
    rng: &mut impl rand::Rng,
) -> Result<usize> {
    sample_token_with_context(logits, config, &[], rng)
}

/// Sample with repetition penalty applied to tokens in `context`.
pub fn sample_token_with_context(
    logits: &Tensor,
    config: &SamplingConfig,
    context: &[usize],
    rng: &mut impl rand::Rng,
) -> Result<usize> {
    if logits.ndim() != 1 {
        return Err(CognitionError::DimensionOutOfRange {
            dim: 1,
            ndim: logits.ndim(),
            operation: "sample_token",
        });
    }

    let n = logits.numel();
    if n == 0 {
        return Err(CognitionError::EmptyTensor {
            operation: "sample_token",
            reason: "empty logits vector",
        });
    }

    // Step 0: Apply repetition penalty (ν primitive — frequency tracking)
    let penalized = if config.repetition_penalty != 1.0 && !context.is_empty() {
        let mut data = logits.data().to_vec();
        for &tok in context {
            if tok < n {
                // Penalize: divide positive logits, multiply negative logits
                if data[tok] > 0.0 {
                    data[tok] /= config.repetition_penalty;
                } else {
                    data[tok] *= config.repetition_penalty;
                }
            }
        }
        Tensor::new(data, vec![n])?
    } else {
        logits.clone()
    };

    // Greedy: return argmax
    if config.temperature <= 0.0 || config.top_k == 1 {
        return penalized.argmax();
    }

    // Step 1: Temperature scaling
    let scaled = penalized.scale(1.0 / config.temperature);

    // Create (index, logit) pairs and sort descending
    let mut indexed: Vec<(usize, f64)> = scaled
        .data()
        .iter()
        .enumerate()
        .map(|(i, &v)| (i, v))
        .collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Step 2: Top-k filtering
    if config.top_k > 0 && config.top_k < n {
        indexed.truncate(config.top_k);
    }

    // Convert to probabilities (softmax on filtered set)
    let max_logit = indexed[0].1;
    let exps: Vec<f64> = indexed.iter().map(|(_, l)| (l - max_logit).exp()).collect();
    let sum: f64 = exps.iter().sum();
    if sum == 0.0 {
        return Err(CognitionError::NumericalInstability {
            operation: "sample_token",
            detail: "softmax sum is zero after filtering".into(),
        });
    }
    let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

    // Step 3: Top-p (nucleus) filtering
    let mut cumulative = 0.0;
    let mut cutoff = probs.len();
    if config.top_p < 1.0 {
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if cumulative >= config.top_p {
                cutoff = i + 1;
                break;
            }
        }
    }

    // Renormalize after top-p cut
    let filtered_probs = &probs[..cutoff];
    let filtered_sum: f64 = filtered_probs.iter().sum();
    let normed: Vec<f64> = filtered_probs.iter().map(|p| p / filtered_sum).collect();

    // Step 4: Sample from categorical distribution
    let u: f64 = rng.random();
    let mut acc = 0.0;
    for (i, &p) in normed.iter().enumerate() {
        acc += p;
        if u < acc {
            return Ok(indexed[i].0);
        }
    }

    // Fallback: return the last token in the filtered set
    Ok(indexed[cutoff - 1].0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy_picks_max() {
        let mut rng = rand::rng();
        let logits = Tensor::new(vec![1.0, 5.0, 2.0, 0.5], vec![4]).unwrap();
        let config = SamplingConfig::greedy();
        let token = sample_token(&logits, &config, &mut rng).unwrap();
        assert_eq!(token, 1); // index of max (5.0)
    }

    #[test]
    fn test_sampling_returns_valid_index() {
        let mut rng = rand::rng();
        let logits = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
        let config = SamplingConfig::creative();
        for _ in 0..100 {
            let token = sample_token(&logits, &config, &mut rng).unwrap();
            assert!(token < 4);
        }
    }

    #[test]
    fn test_repetition_penalty() {
        let mut rng = rand::rng();
        // Logits: index 1 is highest
        let logits = Tensor::new(vec![1.0, 10.0, 2.0], vec![3]).unwrap();
        let config = SamplingConfig {
            temperature: 0.01,
            top_k: 0,
            top_p: 1.0,
            repetition_penalty: 100.0, // very strong penalty
        };
        // With token 1 already in context and heavy penalty, it should be suppressed
        let token = sample_token_with_context(&logits, &config, &[1], &mut rng).unwrap();
        // Token 1's logit (10.0) / 100.0 = 0.1, so token 2 (logit 2.0) should win
        assert_eq!(token, 2, "repetition penalty should suppress token 1");
    }

    #[test]
    fn test_low_temperature_is_deterministic() {
        let mut rng = rand::rng();
        let logits = Tensor::new(vec![1.0, 10.0, 2.0], vec![3]).unwrap();
        let config = SamplingConfig {
            temperature: 0.01,
            top_k: 0,
            top_p: 1.0,
            repetition_penalty: 1.0,
        };
        // Very low temperature should almost always pick the max
        let mut count_max = 0;
        for _ in 0..100 {
            if sample_token(&logits, &config, &mut rng).unwrap() == 1 {
                count_max += 1;
            }
        }
        assert!(
            count_max > 95,
            "low temperature picked max {count_max}/100 times"
        );
    }
}
