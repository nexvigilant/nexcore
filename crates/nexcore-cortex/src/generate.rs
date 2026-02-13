//! # Text Generation
//!
//! Sampling and text generation loop for autoregressive models.
//!
//! ## T1 Grounding
//! - N (Quantity): Token counts, temperature, top-p
//! - ∂ (Boundary): Sampling bounds, repeat penalty
//! - ν (Frequency): Token frequency tracking for repeat penalty

use candle_core::Tensor;
use serde::{Deserialize, Serialize};

/// Parameters controlling text generation.
///
/// Tier: T2-P (N + ∂ + ν — Quantity + Boundary + Frequency)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateParams {
    /// Maximum number of tokens to generate.
    pub max_tokens: usize,
    /// Sampling temperature (0.0 = greedy, 1.0 = full random).
    pub temperature: f64,
    /// Nucleus sampling: cumulative probability cutoff.
    pub top_p: f64,
    /// Penalty applied to tokens that have already appeared.
    pub repeat_penalty: f32,
    /// Optional random seed for reproducibility.
    pub seed: Option<u64>,
}

impl Default for GenerateParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            seed: None,
        }
    }
}

impl GenerateParams {
    /// Create params for greedy (deterministic) generation.
    pub fn greedy(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            temperature: 0.0,
            top_p: 1.0,
            repeat_penalty: 1.0,
            seed: Some(0),
        }
    }

    /// Create params for creative generation.
    pub fn creative(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            temperature: 0.9,
            top_p: 0.95,
            repeat_penalty: 1.2,
            seed: None,
        }
    }
}

/// Apply temperature scaling to logits.
///
/// Divides logits by temperature, making the distribution sharper (low temp)
/// or flatter (high temp). Temperature of 0.0 triggers greedy decoding.
pub fn apply_temperature(logits: &Tensor, temperature: f64) -> Result<Tensor, candle_core::Error> {
    if temperature <= 0.0 || (temperature - 1.0).abs() < f64::EPSILON {
        return Ok(logits.clone());
    }
    logits.broadcast_div(&Tensor::new(&[temperature], logits.device())?)
}

/// Apply repeat penalty to logits for tokens that have already been generated.
///
/// Tokens in `generated_tokens` have their logits divided by `penalty` (if positive)
/// or multiplied by `penalty` (if negative).
pub fn apply_repeat_penalty(
    logits: &Tensor,
    generated_tokens: &[u32],
    penalty: f32,
) -> Result<Tensor, candle_core::Error> {
    if (penalty - 1.0).abs() < f32::EPSILON || generated_tokens.is_empty() {
        return Ok(logits.clone());
    }

    let device = logits.device();
    let mut logits_vec: Vec<f32> = logits.to_vec1()?;

    for &token_id in generated_tokens {
        let idx = token_id as usize;
        if idx < logits_vec.len() {
            let logit = logits_vec[idx];
            logits_vec[idx] = if logit > 0.0 {
                logit / penalty
            } else {
                logit * penalty
            };
        }
    }

    Tensor::new(logits_vec, device)
}

/// Sample a token from logits using top-p (nucleus) sampling.
///
/// Returns the selected token index.
pub fn sample_top_p(logits: &Tensor, top_p: f64, rng_seed: u64) -> Result<u32, candle_core::Error> {
    let logits_vec: Vec<f32> = logits.to_vec1()?;

    // Softmax
    let max_logit = logits_vec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_logits: Vec<f32> = logits_vec.iter().map(|&x| (x - max_logit).exp()).collect();
    let sum_exp: f32 = exp_logits.iter().sum();
    let probs: Vec<f32> = exp_logits.iter().map(|&x| x / sum_exp).collect();

    // Sort by probability descending
    let mut indexed_probs: Vec<(usize, f32)> = probs.iter().copied().enumerate().collect();
    indexed_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Nucleus sampling: accumulate probability mass up to top_p
    let mut cumulative = 0.0_f32;
    let mut candidates: Vec<(usize, f32)> = Vec::new();
    for (idx, prob) in &indexed_probs {
        candidates.push((*idx, *prob));
        cumulative += prob;
        if cumulative >= top_p as f32 {
            break;
        }
    }

    // Renormalize candidates
    let total: f32 = candidates.iter().map(|(_, p)| p).sum();
    let normalized: Vec<(usize, f32)> = candidates
        .iter()
        .map(|(idx, p)| (*idx, p / total))
        .collect();

    // Simple seeded random sampling
    let random_value = simple_random(rng_seed);
    let mut cumulative = 0.0_f32;
    for (idx, prob) in &normalized {
        cumulative += prob;
        if random_value <= cumulative as f64 {
            return Ok(*idx as u32);
        }
    }

    // Fallback: return highest probability token
    Ok(indexed_probs
        .first()
        .map(|(idx, _)| *idx as u32)
        .unwrap_or(0))
}

/// Simple deterministic random number generator (xorshift64).
/// Returns a value in [0.0, 1.0).
fn simple_random(seed: u64) -> f64 {
    let mut x = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x as f64) / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_params_default() {
        let params = GenerateParams::default();
        assert_eq!(params.max_tokens, 512);
        assert!((params.temperature - 0.7).abs() < f64::EPSILON);
        assert!((params.top_p - 0.9).abs() < f64::EPSILON);
        assert!((params.repeat_penalty - 1.1).abs() < f32::EPSILON);
        assert!(params.seed.is_none());
    }

    #[test]
    fn test_greedy_params() {
        let params = GenerateParams::greedy(100);
        assert_eq!(params.max_tokens, 100);
        assert!((params.temperature - 0.0).abs() < f64::EPSILON);
        assert_eq!(params.seed, Some(0));
    }

    #[test]
    fn test_creative_params() {
        let params = GenerateParams::creative(256);
        assert_eq!(params.max_tokens, 256);
        assert!((params.temperature - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simple_random_deterministic() {
        let v1 = simple_random(42);
        let v2 = simple_random(42);
        assert!((v1 - v2).abs() < f64::EPSILON);
        assert!((0.0..1.0).contains(&v1));
    }

    #[test]
    fn test_simple_random_different_seeds() {
        let v1 = simple_random(42);
        let v2 = simple_random(99);
        assert!((v1 - v2).abs() > f64::EPSILON);
    }
}
