use crate::types::HillActivation;
use std::f64::consts::PI;

/// Volume integral of a bicone: π × Σ(w²).
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::volume_integral;
/// let v = volume_integral(&[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0]);
/// assert!((v - std::f64::consts::PI * 76.0).abs() < 1e-9);
/// ```
pub fn volume_integral(widths: &[f64]) -> f64 {
    PI * widths.iter().map(|w| w * w).sum::<f64>()
}

/// Shannon entropy over the width distribution.
///
/// Returns `(H, H_normalized)` where `H_normalized = H / log₂(n)`.
/// When all widths are zero or the slice is empty, returns `(0.0, 0.0)`.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::information_entropy;
/// let (h, hn) = information_entropy(&[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0]);
/// assert!(hn > 0.90 && hn < 0.93);
/// ```
pub fn information_entropy(widths: &[f64]) -> (f64, f64) {
    let total: f64 = widths.iter().sum();
    if total == 0.0 || widths.is_empty() {
        return (0.0, 0.0);
    }
    let h: f64 = widths
        .iter()
        .map(|&w| {
            let p = w / total;
            if p == 0.0 { 0.0 } else { -p * p.log2() }
        })
        .sum();
    let h_max = (widths.len() as f64).log2();
    let h_normalized = if h_max == 0.0 { 0.0 } else { h / h_max };
    (h, h_normalized)
}

/// Asymmetry ratio: upper-cone mass / lower-cone mass.
///
/// `singularity_idx` is the level index of the bicone throat.
///
/// # Errors
/// Returns an error if `lower == 0.0` (division by zero) or if
/// `singularity_idx` is out of bounds.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::cone_asymmetry;
/// let r = cone_asymmetry(&[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0], 4).unwrap();
/// assert!(r > 0.0);
/// ```
pub fn cone_asymmetry(widths: &[f64], singularity_idx: usize) -> anyhow::Result<f64> {
    anyhow::ensure!(
        singularity_idx < widths.len(),
        "singularity_idx {singularity_idx} out of bounds (len={})",
        widths.len()
    );
    let upper: f64 = widths[..singularity_idx].iter().sum();
    let lower: f64 = widths[singularity_idx + 1..].iter().sum();
    anyhow::ensure!(
        lower != 0.0,
        "lower-cone mass is zero — asymmetry undefined"
    );
    Ok(upper / lower)
}

/// Width decrease rate from `peak_idx` to `singularity_idx` (width/level).
///
/// # Errors
/// Returns an error if either index is out of bounds or `peak_idx >= singularity_idx`.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::convergence_rate;
/// let rate = convergence_rate(&[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0], 1, 4).unwrap();
/// assert!(rate > 0.0);
/// ```
pub fn convergence_rate(
    widths: &[f64],
    peak_idx: usize,
    singularity_idx: usize,
) -> anyhow::Result<f64> {
    anyhow::ensure!(
        peak_idx < widths.len(),
        "peak_idx {peak_idx} out of bounds (len={})",
        widths.len()
    );
    anyhow::ensure!(
        singularity_idx < widths.len(),
        "singularity_idx {singularity_idx} out of bounds (len={})",
        widths.len()
    );
    anyhow::ensure!(
        peak_idx < singularity_idx,
        "peak_idx ({peak_idx}) must be less than singularity_idx ({singularity_idx})"
    );
    let delta_w = widths[peak_idx] - widths[singularity_idx];
    let delta_l = (singularity_idx - peak_idx) as f64;
    Ok(delta_w / delta_l)
}

/// Hill-function activation profile across all levels.
///
/// `response = w^n / (k^n + w^n)`, `is_bottleneck` when `response < 0.10`.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::hill_profile;
/// let acts = hill_profile(&[1.0], 3.0, 2.5);
/// assert!((acts[0].response - 0.060).abs() < 0.001);
/// ```
pub fn hill_profile(widths: &[f64], k_half: f64, n_hill: f64) -> Vec<HillActivation> {
    widths
        .iter()
        .enumerate()
        .map(|(i, &w)| {
            let w_n = w.powf(n_hill);
            let k_n = k_half.powf(n_hill);
            let response = w_n / (k_n + w_n);
            HillActivation {
                level: i,
                width: w,
                response,
                is_bottleneck: response < 0.10,
            }
        })
        .collect()
}

/// Cosine similarity between two width vectors.
///
/// Both slices must have equal length and non-zero L2 norms.
///
/// # Errors
/// Returns an error if lengths differ or either norm is zero.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::spectral_overlap;
/// let s = spectral_overlap(&[1.0, 0.0], &[1.0, 0.0]).unwrap();
/// assert!((s - 1.0).abs() < 1e-9);
/// ```
pub fn spectral_overlap(a: &[f64], b: &[f64]) -> anyhow::Result<f64> {
    anyhow::ensure!(
        a.len() == b.len(),
        "slice length mismatch: {} vs {}",
        a.len(),
        b.len()
    );
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    anyhow::ensure!(
        norm_a != 0.0 && norm_b != 0.0,
        "zero-norm vector — overlap undefined"
    );
    Ok(dot / (norm_a * norm_b))
}

/// Per-level tool density: `tools[i] / nodes[i]`.
///
/// # Errors
/// Returns an error if lengths differ or any `nodes[i] == 0.0`.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::tool_density;
/// let d = tool_density(&[6.0, 3.0], &[3.0, 1.0]).unwrap();
/// assert_eq!(d, vec![2.0, 3.0]);
/// ```
pub fn tool_density(tools: &[f64], nodes: &[f64]) -> anyhow::Result<Vec<f64>> {
    anyhow::ensure!(
        tools.len() == nodes.len(),
        "slice length mismatch: {} vs {}",
        tools.len(),
        nodes.len()
    );
    tools
        .iter()
        .zip(nodes.iter())
        .enumerate()
        .map(|(i, (&t, &n))| {
            anyhow::ensure!(n != 0.0, "nodes[{i}] is zero — density undefined");
            Ok(t / n)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const WIDTHS: &[f64] = &[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0];

    #[test]
    fn test_volume_integral() {
        // Σ(w²) = 9+36+9+4+1+16+1 = 76
        let v = volume_integral(WIDTHS);
        assert!((v - PI * 76.0).abs() < 1e-9, "volume={v}");
    }

    #[test]
    fn test_entropy() {
        // total=20, H≈2.571 bits, H_max=log₂(7)≈2.807 → H_norm≈0.916
        let (_h, hn) = information_entropy(WIDTHS);
        assert!(
            (0.90..=0.93).contains(&hn),
            "H_normalized={hn} not in [0.90, 0.93]"
        );
    }

    #[test]
    fn test_hill_profile() {
        // w=1, k=3, n=2.5 → 1^2.5 / (3^2.5 + 1^2.5) = 1/(15.588+1) ≈ 0.0604
        let acts = hill_profile(&[1.0], 3.0, 2.5);
        assert_eq!(acts.len(), 1);
        assert!(
            (acts[0].response - 0.060).abs() < 0.001,
            "response={}",
            acts[0].response
        );
        assert!(acts[0].is_bottleneck);
    }

    #[test]
    fn test_spectral_overlap_identical() {
        let s = spectral_overlap(&[1.0, 0.0], &[1.0, 0.0]).unwrap();
        assert!((s - 1.0).abs() < 1e-9, "overlap={s}");
    }

    #[test]
    fn test_spectral_overlap_orthogonal() {
        let s = spectral_overlap(&[1.0, 0.0], &[0.0, 1.0]).unwrap();
        assert!((s - 0.0).abs() < 1e-9, "overlap={s}");
    }

    #[test]
    fn test_cone_asymmetry() {
        // upper=[3,6,3,2], lower=[4,1] → 14/5=2.8
        let r = cone_asymmetry(WIDTHS, 4).unwrap();
        assert!((r - 2.8).abs() < 1e-9, "asymmetry={r}");
    }

    #[test]
    fn test_convergence_rate() {
        // peak=1 (w=6), singularity=4 (w=1) → (6-1)/3 = 5/3
        let rate = convergence_rate(WIDTHS, 1, 4).unwrap();
        assert!((rate - 5.0 / 3.0).abs() < 1e-9, "rate={rate}");
    }

    #[test]
    fn test_tool_density() {
        let d = tool_density(&[6.0, 3.0], &[3.0, 1.0]).unwrap();
        assert_eq!(d, vec![2.0, 3.0]);
    }

    #[test]
    fn test_volume_empty() {
        assert_eq!(volume_integral(&[]), 0.0);
    }

    #[test]
    fn test_entropy_empty() {
        assert_eq!(information_entropy(&[]), (0.0, 0.0));
    }

    #[test]
    fn test_spectral_overlap_length_mismatch() {
        assert!(spectral_overlap(&[1.0], &[1.0, 2.0]).is_err());
    }

    #[test]
    fn test_convergence_rate_invalid_order() {
        assert!(convergence_rate(WIDTHS, 4, 1).is_err());
    }
}
