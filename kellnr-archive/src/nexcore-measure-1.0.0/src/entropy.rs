//! Information Theory: Shannon entropy, mutual information, NCD, redundancy.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | counts → probability distribution → entropy |
//! | T1: Boundary (δ) | Entropy ≥ 0, Redundancy ∈ [0,1] |
//! | T1: Comparison (κ) | NCD as Kolmogorov distance approximation |

use crate::error::{MeasureError, MeasureResult};
use crate::types::{Entropy, Probability};

/// Compute Shannon entropy H = -Σ p_i * log2(p_i) from count data.
///
/// Returns entropy in bits. Empty or all-zero counts return 0.
pub fn shannon_entropy(counts: &[usize]) -> MeasureResult<Entropy> {
    let total: usize = counts.iter().sum();
    if total == 0 {
        return Ok(Entropy::new(0.0));
    }

    let total_f = total as f64;
    let mut h = 0.0_f64;
    for &c in counts {
        if c > 0 {
            let p = c as f64 / total_f;
            h -= p * p.log2();
        }
    }
    Ok(Entropy::new(h))
}

/// Maximum entropy for n categories: H_max = log2(n).
pub fn max_entropy(n: usize) -> MeasureResult<Entropy> {
    if n == 0 {
        return Err(MeasureError::EmptyInput {
            context: "max_entropy requires n > 0".into(),
        });
    }
    Ok(Entropy::new((n as f64).log2()))
}

/// Redundancy R = 1 - H/H_max.
///
/// Measures how far the distribution is from uniform.
/// R = 0 means perfectly uniform; R = 1 means all mass on one category.
pub fn redundancy(counts: &[usize]) -> MeasureResult<Probability> {
    let n = counts.iter().filter(|&&c| c > 0).count();
    if n <= 1 {
        return Ok(Probability::new(1.0));
    }
    let h = shannon_entropy(counts)?;
    let h_max = max_entropy(n)?;
    if h_max.value() < f64::EPSILON {
        return Ok(Probability::new(1.0));
    }
    Ok(Probability::new(1.0 - h.value() / h_max.value()))
}

/// Joint entropy H(X,Y) from co-occurrence matrix.
///
/// `co_occurrence` is a flat vector of size |X|*|Y| (row-major).
pub fn joint_entropy(co_occurrence: &[usize]) -> MeasureResult<Entropy> {
    shannon_entropy(co_occurrence)
}

/// Mutual information I(X;Y) = H(X) + H(Y) - H(X,Y).
///
/// `marginal_x`, `marginal_y`: marginal count distributions.
/// `co_occurrence`: joint distribution (flat row-major, len = |X|*|Y|).
pub fn mutual_information(
    marginal_x: &[usize],
    marginal_y: &[usize],
    co_occurrence: &[usize],
) -> MeasureResult<Entropy> {
    let h_x = shannon_entropy(marginal_x)?;
    let h_y = shannon_entropy(marginal_y)?;
    let h_xy = joint_entropy(co_occurrence)?;
    // MI can be slightly negative due to floating point; clamp to 0.
    let mi = (h_x.value() + h_y.value() - h_xy.value()).max(0.0);
    Ok(Entropy::new(mi))
}

/// Normalized Compression Distance: approximates Kolmogorov complexity.
///
/// NCD(x,y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))
///
/// Uses flate2 (DEFLATE) as the compressor.
pub fn ncd(data_x: &[u8], data_y: &[u8]) -> MeasureResult<Probability> {
    use flate2::Compression;
    use flate2::write::DeflateEncoder;
    use std::io::Write;

    let compress = |data: &[u8]| -> MeasureResult<usize> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data).map_err(MeasureError::Io)?;
        let compressed = encoder.finish().map_err(MeasureError::Io)?;
        Ok(compressed.len())
    };

    let c_x = compress(data_x)?;
    let c_y = compress(data_y)?;

    // Concatenate x + y
    let mut combined = Vec::with_capacity(data_x.len() + data_y.len());
    combined.extend_from_slice(data_x);
    combined.extend_from_slice(data_y);
    let c_xy = compress(&combined)?;

    let min_c = c_x.min(c_y) as f64;
    let max_c = c_x.max(c_y) as f64;

    if max_c < f64::EPSILON {
        return Ok(Probability::new(0.0));
    }

    let ncd_val = (c_xy as f64 - min_c) / max_c;
    Ok(Probability::new(ncd_val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shannon_uniform_distribution() {
        // 4 categories with equal counts → log2(4) = 2.0 bits
        let counts = vec![100, 100, 100, 100];
        let h = shannon_entropy(&counts).unwrap_or_else(|_| Entropy::new(0.0));
        assert!((h.value() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn shannon_degenerate_distribution() {
        // All mass on one category → 0 bits
        let counts = vec![100, 0, 0, 0];
        let h = shannon_entropy(&counts).unwrap_or_else(|_| Entropy::new(0.0));
        assert!((h.value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn shannon_empty_input() {
        let counts: Vec<usize> = vec![];
        let h = shannon_entropy(&counts).unwrap_or_else(|_| Entropy::new(0.0));
        assert!((h.value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn shannon_all_zeros() {
        let counts = vec![0, 0, 0];
        let h = shannon_entropy(&counts).unwrap_or_else(|_| Entropy::new(0.0));
        assert!((h.value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn max_entropy_power_of_two() {
        let h = max_entropy(8).unwrap_or_else(|_| Entropy::new(0.0));
        assert!((h.value() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn max_entropy_zero_errors() {
        let result = max_entropy(0);
        assert!(result.is_err());
    }

    #[test]
    fn redundancy_uniform_is_zero() {
        let counts = vec![50, 50, 50, 50];
        let r = redundancy(&counts).unwrap_or_else(|_| Probability::new(0.0));
        assert!(r.value().abs() < 1e-10);
    }

    #[test]
    fn redundancy_degenerate_is_one() {
        let counts = vec![100, 0, 0, 0];
        let r = redundancy(&counts).unwrap_or_else(|_| Probability::new(0.0));
        assert!((r.value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mutual_information_self_equals_entropy() {
        // I(X;X) = H(X)
        let counts = vec![30, 70, 50, 100];
        let h = shannon_entropy(&counts).unwrap_or_else(|_| Entropy::new(0.0));
        // For I(X;X), marginals are both `counts`, joint is diagonal.
        let n = counts.len();
        let mut co = vec![0usize; n * n];
        for (i, &c) in counts.iter().enumerate() {
            co[i * n + i] = c;
        }
        let mi = mutual_information(&counts, &counts, &co).unwrap_or_else(|_| Entropy::new(0.0));
        assert!((mi.value() - h.value()).abs() < 1e-10);
    }

    #[test]
    fn ncd_identical_is_near_zero() {
        let data = b"hello world, this is a test of the compression distance";
        let result = ncd(data, data).unwrap_or_else(|_| Probability::new(1.0));
        // NCD of identical strings should be small (not exactly 0 due to compressor overhead)
        assert!(
            result.value() < 0.3,
            "NCD of identical = {}",
            result.value()
        );
    }

    #[test]
    fn ncd_different_is_higher() {
        // Use longer, more varied data to overcome compressor overhead
        let data_a: Vec<u8> = (0..500).map(|i| b'a' + (i % 5) as u8).collect();
        let data_b: Vec<u8> = (0..500).map(|i| b'z' - (i % 7) as u8).collect();
        let ncd_same = ncd(&data_a, &data_a).unwrap_or_else(|_| Probability::new(0.0));
        let ncd_diff = ncd(&data_a, &data_b).unwrap_or_else(|_| Probability::new(0.0));
        assert!(
            ncd_diff.value() > ncd_same.value(),
            "ncd_diff={} should > ncd_same={}",
            ncd_diff.value(),
            ncd_same.value()
        );
    }

    #[test]
    fn ncd_empty_data() {
        let result = ncd(b"", b"").unwrap_or_else(|_| Probability::new(1.0));
        // Both empty → compressed to small header → ratio near 0
        assert!(result.value() < 1.0);
    }
}
