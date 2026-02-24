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
pub fn cone_asymmetry(widths: &[f64], singularity_idx: usize) -> nexcore_error::Result<f64> {
    nexcore_error::ensure!(
        singularity_idx < widths.len(),
        "singularity_idx {singularity_idx} out of bounds (len={})",
        widths.len()
    );
    let upper: f64 = widths[..singularity_idx].iter().sum();
    let lower: f64 = widths[singularity_idx + 1..].iter().sum();
    nexcore_error::ensure!(
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
) -> nexcore_error::Result<f64> {
    nexcore_error::ensure!(
        peak_idx < widths.len(),
        "peak_idx {peak_idx} out of bounds (len={})",
        widths.len()
    );
    nexcore_error::ensure!(
        singularity_idx < widths.len(),
        "singularity_idx {singularity_idx} out of bounds (len={})",
        widths.len()
    );
    nexcore_error::ensure!(
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
pub fn spectral_overlap(a: &[f64], b: &[f64]) -> nexcore_error::Result<f64> {
    nexcore_error::ensure!(
        a.len() == b.len(),
        "slice length mismatch: {} vs {}",
        a.len(),
        b.len()
    );
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    nexcore_error::ensure!(
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
pub fn tool_density(tools: &[f64], nodes: &[f64]) -> nexcore_error::Result<Vec<f64>> {
    nexcore_error::ensure!(
        tools.len() == nodes.len(),
        "slice length mismatch: {} vs {}",
        tools.len(),
        nodes.len()
    );
    tools
        .iter()
        .zip(nodes.iter())
        .map(|(&t, &n)| {
            nexcore_error::ensure!(n != 0.0, "node count is zero — density undefined");
            Ok(t / n)
        })
        .collect()
}

/// Index of the minimum width (singularity / throat).
///
/// For an empty slice, returns 0.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::find_singularity;
/// assert_eq!(find_singularity(&[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0]), 4);
/// ```
pub fn find_singularity(widths: &[f64]) -> usize {
    widths
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Index of the maximum width (peak).
///
/// For an empty slice, returns 0.
///
/// # Example
/// ```
/// # use nexcore_bicone::metrics::find_peak;
/// assert_eq!(find_peak(&[3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0]), 1);
/// ```
pub fn find_peak(widths: &[f64]) -> usize {
    widths
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Compute all metrics for a bicone profile in a single call.
///
/// # Errors
/// Returns an error if the width sequence is empty or if any sub-metric fails.
///
/// # Example
/// ```
/// # use nexcore_bicone::types::BiconeProfile;
/// # use nexcore_bicone::metrics::compute_metrics;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let profile = BiconeProfile {
///     width_sequence: vec![3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0],
///     level_labels: None,
/// };
/// let m = compute_metrics(&profile)?;
/// assert_eq!(m.level_count, 7);
/// assert_eq!(m.singularity_index, 4);
/// # Ok(())
/// # }
/// ```
pub fn compute_metrics(
    profile: &crate::types::BiconeProfile,
) -> nexcore_error::Result<crate::types::BiconeMetrics> {
    let w = &profile.width_sequence;
    nexcore_error::ensure!(!w.is_empty(), "empty width sequence");

    let volume = volume_integral(w);
    let (entropy, entropy_normalized) = information_entropy(w);
    let singularity_index = find_singularity(w);
    let peak_index = find_peak(w);
    let total_nodes: f64 = w.iter().sum();

    let asymmetry_ratio = if singularity_index > 0 && singularity_index < w.len() - 1 {
        cone_asymmetry(w, singularity_index).unwrap_or(0.0)
    } else {
        0.0
    };

    let conv_rate = if peak_index < singularity_index {
        convergence_rate(w, peak_index, singularity_index).unwrap_or(0.0)
    } else {
        0.0
    };

    Ok(crate::types::BiconeMetrics {
        volume,
        entropy,
        entropy_normalized,
        asymmetry_ratio,
        convergence_rate: conv_rate,
        singularity_index,
        total_nodes,
        level_count: w.len(),
    })
}

/// Compare two bicone profiles for shape similarity.
///
/// Returns cosine overlap, a qualitative classification, and the indices
/// where the two profiles diverge most.
///
/// # Errors
/// Returns an error if the profiles have different lengths.
///
/// # Example
/// ```
/// # use nexcore_bicone::types::BiconeProfile;
/// # use nexcore_bicone::metrics::compare_profiles;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let a = BiconeProfile { width_sequence: vec![3.0, 6.0, 1.0], level_labels: None };
/// let b = BiconeProfile { width_sequence: vec![3.0, 6.0, 1.0], level_labels: None };
/// let cmp = compare_profiles(&a, &b)?;
/// assert_eq!(cmp.classification, "identical");
/// # Ok(())
/// # }
/// ```
pub fn compare_profiles(
    a: &crate::types::BiconeProfile,
    b: &crate::types::BiconeProfile,
) -> nexcore_error::Result<crate::types::ShapeComparison> {
    let wa = &a.width_sequence;
    let wb = &b.width_sequence;
    let overlap = spectral_overlap(wa, wb)?;

    // Identify divergent levels: where relative difference > 30%
    let divergent_levels: Vec<usize> = wa
        .iter()
        .zip(wb.iter())
        .enumerate()
        .filter_map(|(i, (&va, &vb))| {
            let max_val = va.abs().max(vb.abs());
            if max_val == 0.0 {
                return None;
            }
            let rel_diff = (va - vb).abs() / max_val;
            if rel_diff > 0.30 { Some(i) } else { None }
        })
        .collect();

    let classification = if (overlap - 1.0).abs() < 1e-9 && divergent_levels.is_empty() {
        "identical"
    } else if overlap >= 0.95 {
        "similar"
    } else if overlap >= 0.70 {
        "moderate"
    } else {
        "divergent"
    }
    .to_string();

    Ok(crate::types::ShapeComparison {
        overlap,
        classification,
        divergent_levels,
    })
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

    #[test]
    fn test_find_singularity() {
        // min width is 1.0 at index 4 (first occurrence)
        assert_eq!(find_singularity(WIDTHS), 4);
    }

    #[test]
    fn test_find_peak() {
        // max width is 6.0 at index 1
        assert_eq!(find_peak(WIDTHS), 1);
    }

    #[test]
    fn test_find_singularity_empty() {
        assert_eq!(find_singularity(&[]), 0);
    }

    #[test]
    fn test_compute_metrics() {
        let profile = crate::types::BiconeProfile {
            width_sequence: vec![3.0, 6.0, 3.0, 2.0, 1.0, 4.0, 1.0],
            level_labels: None,
        };
        let m = compute_metrics(&profile).unwrap();
        assert_eq!(m.level_count, 7);
        assert_eq!(m.singularity_index, 4);
        assert!((m.volume - PI * 76.0).abs() < 1e-9);
        assert!((m.total_nodes - 20.0).abs() < 1e-9);
        assert!(m.asymmetry_ratio > 0.0);
        assert!(m.convergence_rate > 0.0);
        assert!(m.entropy_normalized > 0.90);
    }

    #[test]
    fn test_compute_metrics_empty() {
        let profile = crate::types::BiconeProfile {
            width_sequence: vec![],
            level_labels: None,
        };
        assert!(compute_metrics(&profile).is_err());
    }

    #[test]
    fn test_compare_profiles_identical() {
        let a = crate::types::BiconeProfile {
            width_sequence: vec![3.0, 6.0, 1.0],
            level_labels: None,
        };
        let b = crate::types::BiconeProfile {
            width_sequence: vec![3.0, 6.0, 1.0],
            level_labels: None,
        };
        let cmp = compare_profiles(&a, &b).unwrap();
        assert_eq!(cmp.classification, "identical");
        assert!((cmp.overlap - 1.0).abs() < 1e-9);
        assert!(cmp.divergent_levels.is_empty());
    }

    #[test]
    fn test_compare_profiles_divergent() {
        let a = crate::types::BiconeProfile {
            width_sequence: vec![10.0, 1.0, 10.0],
            level_labels: None,
        };
        let b = crate::types::BiconeProfile {
            width_sequence: vec![1.0, 10.0, 1.0],
            level_labels: None,
        };
        let cmp = compare_profiles(&a, &b).unwrap();
        assert!(cmp.overlap < 0.70, "overlap={}", cmp.overlap);
        assert_eq!(cmp.classification, "divergent");
        assert!(!cmp.divergent_levels.is_empty());
    }

    #[test]
    fn test_compare_profiles_length_mismatch() {
        let a = crate::types::BiconeProfile {
            width_sequence: vec![1.0, 2.0],
            level_labels: None,
        };
        let b = crate::types::BiconeProfile {
            width_sequence: vec![1.0, 2.0, 3.0],
            level_labels: None,
        };
        assert!(compare_profiles(&a, &b).is_err());
    }
}
