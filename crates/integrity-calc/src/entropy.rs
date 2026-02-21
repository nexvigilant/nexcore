//! Sliding window Shannon entropy
//!
//! Tier: T2-C | Primitives: Σ Sum, N Quantity

use std::collections::HashMap;

/// Entropy profile statistics.
#[derive(Debug, Clone)]
pub struct EntropyProfile {
    pub mean: f64,
    pub std_dev: f64,
    pub range: f64,
    pub window_count: usize,
    pub values: Vec<f64>,
}

/// Compute Shannon entropy of a token slice.
/// H = -Σ p(x) × log2(p(x))
#[must_use]
pub fn shannon_entropy(tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }

    let total = tokens.len() as f64;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for token in tokens {
        *counts.entry(token.as_str()).or_insert(0) += 1;
    }

    let mut entropy = 0.0;
    for &count in counts.values() {
        let p = count as f64 / total;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Compute entropy profile over sliding windows.
pub fn entropy_profile(tokens: &[String], window_size: usize, step: usize) -> EntropyProfile {
    let window_size = window_size.max(1);
    let step = step.max(1);

    if tokens.len() < window_size {
        let h = shannon_entropy(tokens);
        return EntropyProfile {
            mean: h,
            std_dev: 0.0,
            range: 0.0,
            window_count: usize::from(!tokens.is_empty()),
            values: if tokens.is_empty() { vec![] } else { vec![h] },
        };
    }

    let mut values = Vec::new();
    let mut start = 0;
    while start + window_size <= tokens.len() {
        let window = &tokens[start..start + window_size];
        values.push(shannon_entropy(window));
        start += step;
    }

    let window_count = values.len();
    if window_count == 0 {
        return EntropyProfile {
            mean: 0.0,
            std_dev: 0.0,
            range: 0.0,
            window_count: 0,
            values: vec![],
        };
    }

    let mean = values.iter().sum::<f64>() / window_count as f64;

    let variance = if window_count > 1 {
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (window_count - 1) as f64
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    EntropyProfile {
        mean,
        std_dev,
        range,
        window_count,
        values,
    }
}
