//! Zipf's Law Deviation — log-log regression
//!
//! Tier: T2-C | Primitives: κ Comparison, N Quantity

use std::collections::HashMap;

/// Zipf analysis result.
#[derive(Debug, Clone)]
pub struct ZipfResult {
    pub alpha: f64,
    pub r_squared: f64,
    pub deviation: f64,
}

/// Analyze rank-frequency distribution against Zipf's law.
#[must_use]
pub fn zipf_analysis(frequencies: &HashMap<String, usize>) -> ZipfResult {
    if frequencies.len() < 2 {
        return ZipfResult {
            alpha: 0.0,
            r_squared: 0.0,
            deviation: 1.0,
        };
    }

    let mut freq_vec: Vec<usize> = frequencies.values().copied().collect();
    freq_vec.sort_unstable_by(|a, b| b.cmp(a));

    let n = freq_vec.len() as f64;
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut sum_xy = 0.0_f64;
    let mut sum_x2 = 0.0_f64;
    let mut sum_y2 = 0.0_f64;

    for (i, &freq) in freq_vec.iter().enumerate() {
        if freq == 0 {
            continue;
        }
        let x = ((i + 1) as f64).ln();
        let y = (freq as f64).ln();
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_x2 += x * x;
        sum_y2 += y * y;
    }

    #[allow(clippy::suspicious_operation_groupings)]
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        return ZipfResult {
            alpha: 0.0,
            r_squared: 0.0,
            deviation: 1.0,
        };
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let alpha = -slope;

    let ss_tot = sum_y2 - (sum_y * sum_y) / n;
    let r_squared = if ss_tot.abs() < 1e-15 {
        0.0
    } else {
        let intercept = (sum_y - slope * sum_x) / n;
        let mut ss_res = 0.0;
        for (i, &freq) in freq_vec.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            let x = ((i + 1) as f64).ln();
            let y = (freq as f64).ln();
            let predicted = intercept + slope * x;
            ss_res += (y - predicted).powi(2);
        }
        1.0 - (ss_res / ss_tot)
    };

    let deviation = (alpha - 1.0).abs();

    ZipfResult {
        alpha,
        r_squared: r_squared.clamp(0.0, 1.0),
        deviation,
    }
}
