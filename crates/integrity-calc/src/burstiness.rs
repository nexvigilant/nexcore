//! Burstiness coefficient — inter-arrival time analysis
//!
//! Tier: T2-C | Primitives: ν Frequency, ∂ Boundary

use std::collections::HashMap;

/// Burstiness analysis result.
#[derive(Debug, Clone)]
pub struct BurstinessResult {
    pub coefficient: f64,
    pub tokens_analyzed: usize,
    pub per_token: Vec<(String, f64)>,
}

/// Compute inter-arrival times for a token in a sequence.
fn inter_arrival_times(tokens: &[String], target: &str) -> Vec<usize> {
    let positions: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.as_str() == target)
        .map(|(i, _)| i)
        .collect();

    if positions.len() < 2 {
        return vec![];
    }

    positions.windows(2).map(|w| w[1] - w[0]).collect()
}

/// Compute burstiness coefficient for a single token.
/// B = (σ - μ) / (σ + μ)
fn single_burstiness(inter_arrivals: &[usize]) -> Option<f64> {
    if inter_arrivals.is_empty() {
        return None;
    }

    let n = inter_arrivals.len() as f64;
    let mean = inter_arrivals.iter().sum::<usize>() as f64 / n;

    if mean.abs() < 1e-15 {
        return Some(0.0);
    }

    let variance = if inter_arrivals.len() > 1 {
        inter_arrivals
            .iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>()
            / (n - 1.0)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let denom = std_dev + mean;
    if denom.abs() < 1e-15 {
        return Some(0.0);
    }

    Some((std_dev - mean) / denom)
}

/// Analyze burstiness of the full token sequence.
#[must_use]
pub fn burstiness_analysis(
    tokens: &[String],
    frequencies: &HashMap<String, usize>,
) -> BurstinessResult {
    let mut per_token = Vec::new();

    for (token, &count) in frequencies {
        if count < 2 {
            continue;
        }
        let arrivals = inter_arrival_times(tokens, token);
        if let Some(b) = single_burstiness(&arrivals) {
            per_token.push((token.clone(), b));
        }
    }

    let tokens_analyzed = per_token.len();
    let coefficient = if tokens_analyzed > 0 {
        per_token.iter().map(|(_, b)| b).sum::<f64>() / tokens_analyzed as f64
    } else {
        0.0
    };

    BurstinessResult {
        coefficient,
        tokens_analyzed,
        per_token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make(words: &[&str]) -> (Vec<String>, HashMap<String, usize>) {
        let tokens: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        let mut freq = HashMap::new();
        for t in &tokens {
            *freq.entry(t.clone()).or_insert(0) += 1;
        }
        (tokens, freq)
    }

    #[test]
    fn inter_arrival_no_match() {
        let tokens: Vec<String> = vec!["a", "b"].into_iter().map(|s| s.into()).collect();
        assert!(inter_arrival_times(&tokens, "z").is_empty());
    }

    #[test]
    fn inter_arrival_single() {
        let tokens: Vec<String> = vec!["a", "b"].into_iter().map(|s| s.into()).collect();
        assert!(inter_arrival_times(&tokens, "a").is_empty());
    }

    #[test]
    fn inter_arrival_regular() {
        let tokens: Vec<String> = vec!["a", "b", "a", "b", "a"]
            .into_iter()
            .map(|s| s.into())
            .collect();
        assert_eq!(inter_arrival_times(&tokens, "a"), vec![2, 2]);
    }

    #[test]
    fn single_burstiness_regular() {
        let b = single_burstiness(&[5, 5, 5, 5]);
        assert!(b.is_some());
        assert!(b.unwrap_or(1.0) <= 0.0);
    }

    #[test]
    fn single_burstiness_empty() {
        assert!(single_burstiness(&[]).is_none());
    }

    #[test]
    fn analysis_empty() {
        let result = burstiness_analysis(&[], &HashMap::new());
        assert_eq!(result.tokens_analyzed, 0);
        assert_eq!(result.coefficient, 0.0);
    }

    #[test]
    fn analysis_all_unique() {
        let (tokens, freq) = make(&["a", "b", "c", "d"]);
        let result = burstiness_analysis(&tokens, &freq);
        assert_eq!(result.tokens_analyzed, 0);
    }

    #[test]
    fn analysis_repeated() {
        let (tokens, freq) = make(&["a", "b", "a", "c", "a", "b", "a"]);
        let result = burstiness_analysis(&tokens, &freq);
        assert!(result.tokens_analyzed > 0);
        assert!(result.coefficient >= -1.0 && result.coefficient <= 1.0);
    }
}
