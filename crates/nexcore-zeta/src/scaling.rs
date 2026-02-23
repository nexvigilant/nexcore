//! # Scaling Law Analysis
//!
//! Fits power-law models to how telescope confidence scales with the number
//! of zeros N. The hypothesis: `C(N) = 1 - a·N^(-b)` where b > 0 indicates
//! that confidence improves with more data.
//!
//! ## Method
//!
//! Transform: `log(1 - C) = log(a) - b·log(N)`
//!
//! Ordinary least squares on the log-log data yields (a, b) and R².

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;

// ── Types ────────────────────────────────────────────────────────────────────

/// A single (N, confidence) measurement point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScalingPoint {
    /// Number of zeros used.
    pub n: usize,
    /// RH confidence from the telescope.
    pub confidence: f64,
}

/// Fitted power-law scaling model.
///
/// `C(N) = 1 - a·N^(-b)`
///
/// Equivalently: `1 - C(N) = a·N^(-b)`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingLaw {
    /// Amplitude parameter a (positive).
    pub a: f64,
    /// Decay exponent b (positive = confidence increases with N).
    pub b: f64,
    /// R² goodness of fit on the log-log regression.
    pub r_squared: f64,
    /// Number of data points used.
    pub n_points: usize,
    /// Residuals in log space.
    pub residuals: Vec<f64>,
}

/// Comparative scaling analysis across multiple arms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingComparison {
    /// Per-arm scaling laws.
    pub arms: Vec<ScalingArm>,
    /// Best-fitting arm (highest R²).
    pub best_arm: String,
    /// Whether any arm shows non-improving confidence (b ≤ 0).
    pub has_pathological_arm: bool,
}

/// Scaling law result for a named arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingArm {
    /// Arm label.
    pub label: String,
    /// Fitted scaling law.
    pub law: ScalingLaw,
    /// Extrapolated confidence at N=10_000.
    pub confidence_at_10k: f64,
    /// Extrapolated confidence at N=100_000.
    pub confidence_at_100k: f64,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Fit a power-law scaling model to (N, confidence) data.
///
/// Model: `C(N) = 1 - a·N^(-b)`
///
/// Uses log-log OLS regression on `log(1-C)` vs `log(N)`.
///
/// # Errors
///
/// Returns error if fewer than 3 points, or if any confidence is exactly
/// 0.0 or 1.0 (log undefined).
pub fn fit_scaling_law(points: &[ScalingPoint]) -> Result<ScalingLaw, ZetaError> {
    if points.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 scaling points for power-law fit".to_string(),
        ));
    }

    // Transform to log-log space: y = log(1-C), x = log(N)
    let mut xs = Vec::with_capacity(points.len());
    let mut ys = Vec::with_capacity(points.len());

    for p in points {
        if p.confidence <= 0.0 || p.confidence >= 1.0 {
            return Err(ZetaError::InvalidParameter(format!(
                "confidence {} at N={} must be in (0, 1) for log transform",
                p.confidence, p.n
            )));
        }
        if p.n == 0 {
            return Err(ZetaError::InvalidParameter(
                "N must be positive".to_string(),
            ));
        }
        xs.push((p.n as f64).ln());
        ys.push((1.0 - p.confidence).ln());
    }

    // OLS: y = intercept + slope * x
    // where intercept = log(a), slope = -b
    let (intercept, slope, r_squared, residuals) = ols_regression(&xs, &ys);

    let a = intercept.exp();
    let b = -slope;

    Ok(ScalingLaw {
        a,
        b,
        r_squared,
        n_points: points.len(),
        residuals,
    })
}

/// Predict confidence at a given N using a fitted scaling law.
///
/// Returns `1 - a·N^(-b)`, clamped to [0, 1].
#[must_use]
pub fn predict_confidence(law: &ScalingLaw, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let c = 1.0 - law.a * (n as f64).powf(-law.b);
    c.clamp(0.0, 1.0)
}

/// Compare scaling laws across multiple named arms.
///
/// Each arm is a `(label, points)` pair.
pub fn compare_scaling_laws(
    arms: &[(&str, &[ScalingPoint])],
) -> Result<ScalingComparison, ZetaError> {
    if arms.is_empty() {
        return Err(ZetaError::InvalidParameter(
            "need at least 1 arm for comparison".to_string(),
        ));
    }

    let mut results = Vec::with_capacity(arms.len());
    let mut best_r2 = f64::NEG_INFINITY;
    let mut best_label = String::new();
    let mut has_pathological = false;

    for &(label, points) in arms {
        let law = fit_scaling_law(points)?;

        if law.b <= 0.0 {
            has_pathological = true;
        }
        if law.r_squared > best_r2 {
            best_r2 = law.r_squared;
            best_label = label.to_string();
        }

        let c_10k = predict_confidence(&law, 10_000);
        let c_100k = predict_confidence(&law, 100_000);

        results.push(ScalingArm {
            label: label.to_string(),
            law,
            confidence_at_10k: c_10k,
            confidence_at_100k: c_100k,
        });
    }

    Ok(ScalingComparison {
        arms: results,
        best_arm: best_label,
        has_pathological_arm: has_pathological,
    })
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Simple OLS: y = intercept + slope * x.
/// Returns (intercept, slope, R², residuals).
fn ols_regression(xs: &[f64], ys: &[f64]) -> (f64, f64, f64, Vec<f64>) {
    let n = xs.len() as f64;
    let sum_x: f64 = xs.iter().sum();
    let sum_y: f64 = ys.iter().sum();
    let sum_xy: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum();
    let sum_x2: f64 = xs.iter().map(|x| x * x).sum();

    let denom = n * sum_x2 - sum_x.powi(2);
    let (slope, intercept) = if denom.abs() < 1e-15 {
        (0.0, sum_y / n)
    } else {
        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        (slope, intercept)
    };

    // Residuals and R²
    let mean_y = sum_y / n;
    let mut ss_res = 0.0_f64;
    let mut ss_tot = 0.0_f64;
    let mut residuals = Vec::with_capacity(xs.len());

    for (x, y) in xs.iter().zip(ys.iter()) {
        let predicted = intercept + slope * x;
        let residual = y - predicted;
        ss_res += residual * residual;
        ss_tot += (y - mean_y) * (y - mean_y);
        residuals.push(residual);
    }

    let r_squared = if ss_tot.abs() < 1e-15 {
        1.0
    } else {
        1.0 - ss_res / ss_tot
    };

    (intercept, slope, r_squared, residuals)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_perfect_power_law() {
        // C(N) = 1 - 2·N^(-0.5)
        let points: Vec<ScalingPoint> = [100, 400, 900, 1600, 2500]
            .iter()
            .map(|&n| ScalingPoint {
                n,
                confidence: 1.0 - 2.0 * (n as f64).powf(-0.5),
            })
            .collect();

        let law = fit_scaling_law(&points);
        assert!(law.is_ok());
        let l = law.unwrap_or_else(|_| unreachable!());
        assert!((l.a - 2.0).abs() < 0.1, "a={} != 2.0", l.a);
        assert!((l.b - 0.5).abs() < 0.01, "b={} != 0.5", l.b);
        assert!(l.r_squared > 0.999, "R²={} too low", l.r_squared);
    }

    #[test]
    fn predict_extrapolation() {
        let law = ScalingLaw {
            a: 10.0,
            b: 0.5,
            r_squared: 1.0,
            n_points: 5,
            residuals: vec![],
        };
        let c = predict_confidence(&law, 10_000);
        // 1 - 10 * 10000^(-0.5) = 1 - 10/100 = 0.9
        assert!((c - 0.9).abs() < 1e-10);
    }

    #[test]
    fn predict_clamps_to_zero_one() {
        let law = ScalingLaw {
            a: 100.0,
            b: 0.1,
            r_squared: 1.0,
            n_points: 5,
            residuals: vec![],
        };
        // For small N, 1 - 100*N^(-0.1) could be negative
        let c = predict_confidence(&law, 1);
        assert!((c - 0.0).abs() < 1e-10);
    }

    #[test]
    fn too_few_points() {
        let points = vec![
            ScalingPoint {
                n: 100,
                confidence: 0.5,
            },
            ScalingPoint {
                n: 200,
                confidence: 0.6,
            },
        ];
        assert!(fit_scaling_law(&points).is_err());
    }

    #[test]
    fn confidence_out_of_range() {
        let points = vec![
            ScalingPoint {
                n: 100,
                confidence: 0.0,
            },
            ScalingPoint {
                n: 200,
                confidence: 0.5,
            },
            ScalingPoint {
                n: 300,
                confidence: 0.6,
            },
        ];
        assert!(fit_scaling_law(&points).is_err());
    }

    #[test]
    fn compare_two_arms() {
        // Arm A: fast scaling (b=0.5), a=2 avoids boundary
        let arm_a: Vec<ScalingPoint> = [100, 400, 900, 1600]
            .iter()
            .map(|&n| ScalingPoint {
                n,
                confidence: 1.0 - 2.0 * (n as f64).powf(-0.5),
            })
            .collect();

        // Arm B: slow scaling (b=0.2), a=2
        let arm_b: Vec<ScalingPoint> = [100, 400, 900, 1600]
            .iter()
            .map(|&n| ScalingPoint {
                n,
                confidence: 1.0 - 2.0 * (n as f64).powf(-0.2),
            })
            .collect();

        let cmp = compare_scaling_laws(&[("fast", &arm_a), ("slow", &arm_b)]);
        assert!(cmp.is_ok());
        let c = cmp.unwrap_or_else(|_| unreachable!());
        assert_eq!(c.arms.len(), 2);
        assert!(!c.has_pathological_arm);
        // Both should have R² ≈ 1.0 since data is exact
        assert!(c.arms[0].law.r_squared > 0.99);
        assert!(c.arms[1].law.r_squared > 0.99);
    }

    #[test]
    fn noisy_data_still_fits() {
        // C(N) = 1 - 2·N^(-0.4) + noise (a=2 avoids boundary issues at small N)
        let ns = [100, 200, 500, 1000, 2000, 5000, 10000];
        let noise = [0.005, -0.008, 0.006, -0.003, 0.004, -0.005, 0.002];
        let points: Vec<ScalingPoint> = ns
            .iter()
            .zip(noise.iter())
            .map(|(&n, &eps)| {
                let c = (1.0 - 2.0 * (n as f64).powf(-0.4) + eps).clamp(0.001, 0.999);
                ScalingPoint { n, confidence: c }
            })
            .collect();

        let law = fit_scaling_law(&points);
        assert!(law.is_ok());
        let l = law.unwrap_or_else(|_| unreachable!());
        assert!((l.a - 2.0).abs() < 1.0, "a={} too far from 2.0", l.a);
        assert!((l.b - 0.4).abs() < 0.15, "b={} too far from 0.4", l.b);
        assert!(
            l.r_squared > 0.90,
            "R²={} too low for noisy data",
            l.r_squared
        );
    }

    #[test]
    fn telescope_scaling_integration() {
        // Run telescope at multiple N values and fit scaling law
        use crate::pipeline::{TelescopeConfig, run_telescope};
        use crate::zeros::find_zeros_bracket;

        let ranges: Vec<(f64, f64)> = vec![(10.0, 80.0), (10.0, 150.0), (10.0, 300.0)];

        let mut points = Vec::new();
        let config = TelescopeConfig::default();

        for (t_min, t_max) in &ranges {
            let zeros = find_zeros_bracket(*t_min, *t_max, 0.1).unwrap_or_default();
            if zeros.len() < 20 {
                continue;
            }
            if let Ok(report) = run_telescope(&zeros, &config) {
                if report.overall_rh_confidence > 0.0 && report.overall_rh_confidence < 1.0 {
                    points.push(ScalingPoint {
                        n: zeros.len(),
                        confidence: report.overall_rh_confidence,
                    });
                }
            }
        }

        if points.len() >= 3 {
            let law = fit_scaling_law(&points);
            assert!(law.is_ok(), "scaling law fit failed: {:?}", law.err());
            let l = law.unwrap_or_else(|_| unreachable!());
            eprintln!("Scaling: a={:.4}, b={:.4}, R²={:.4}", l.a, l.b, l.r_squared);
        }
    }
}
