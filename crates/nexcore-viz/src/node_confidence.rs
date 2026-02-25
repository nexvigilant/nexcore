//! Node Confidence Computation
//!
//! Unifies 5 Observatory frontend confidence derivation formulas into a single
//! Rust computation returning `Measured<f64>`. Replaces client-side heuristics
//! with server-computed, calibrated confidence scores.
//!
//! # Confidence Sources
//!
//! | Variant | Explorer | Formula |
//! |---------|----------|---------|
//! | `ChiSquared` | Causality | `1 - chi_squared_pvalue(χ²)`, FDR reduction ×0.6 |
//! | `SignalStrength` | Graph/Signal | PRR-scaled: `min(0.95, 0.5 + prr/15)` or 0.3 |
//! | `Severity` | Timeline | `max(0.15, severity)` |
//! | `RelevanceScore` | Regulatory | Direct `score` mapping |
//! | `StructuralCertainty` | Molecule | 0.95 (preset) vs 0.60 (API-derived) |
//!
//! # Primitive Grounding
//!
//! μ(Mapping): metric → confidence. N(Quantity): numeric thresholds.
//! κ(Comparison): threshold checks. ∂(Boundary): [0.0, 1.0] clamping.

use nexcore_constants::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// Source of confidence for an Observatory graph node.
///
/// Each variant corresponds to one of the 5 Observatory explorer confidence
/// derivation strategies. The `compute_confidence` function maps any source
/// to a `Measured<f64>` with calibrated confidence metadata.
// CALIBRATION: Formulas match the TypeScript implementations in:
//   - use-causality-data.ts (ChiSquared)
//   - live-signal-adapter.ts (SignalStrength)
//   - use-timeline-data.ts (Severity)
//   - use-regulatory-data.ts (RelevanceScore)
//   - use-molecule-data.ts (StructuralCertainty)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ConfidenceSource {
    /// Causality explorer: chi-squared statistic → p-value → confidence.
    /// FDR correction reduces confidence by 40% for non-rejected signals.
    ChiSquared {
        /// Chi-squared test statistic (df=1).
        chi_squared: f64,
        /// Whether the signal was rejected by Benjamini-Hochberg FDR correction.
        /// `false` = not significant after FDR → confidence reduced by 0.6.
        #[serde(default)]
        fdr_rejected: bool,
    },

    /// Graph/Signal explorer: PRR-scaled confidence for FAERS adverse events.
    /// Signal-detected events get `min(0.95, 0.5 + prr/15)`, others get 0.3.
    SignalStrength {
        /// Proportional Reporting Ratio.
        prr: f64,
        /// Whether disproportionality analysis detected a signal.
        signal_detected: bool,
    },

    /// Timeline explorer: event severity maps to confidence.
    /// Floor at 0.15 ensures even low-severity events remain visible.
    Severity {
        /// Event severity score in [0.0, 1.0].
        severity: f64,
    },

    /// Regulatory explorer: guideline relevance score maps directly.
    RelevanceScore {
        /// Relevance score from guidelines_search, typically in [0.0, 1.0].
        score: f64,
    },

    /// Molecule explorer: structural certainty based on data provenance.
    /// Known crystal structures (presets) get 0.95, API-derived get 0.60.
    StructuralCertainty {
        /// Whether the molecular data was derived from API (lower certainty)
        /// vs a known preset/crystal structure (higher certainty).
        api_derived: bool,
    },
}

/// Compute confidence for an Observatory graph node.
///
/// Returns `Measured<f64>` where:
/// - `value`: the confidence score in [0.0, 1.0]
/// - `confidence`: mirrors value (confidence in the confidence score itself)
///
/// # Examples
///
/// ```
/// use nexcore_viz::node_confidence::{ConfidenceSource, compute_confidence};
///
/// // Strong chi-squared signal, FDR-confirmed
/// let src = ConfidenceSource::ChiSquared {
///     chi_squared: 10.0,
///     fdr_rejected: true,
/// };
/// let result = compute_confidence(&src);
/// assert!(result.value > 0.9);
///
/// // Weak signal, not FDR-rejected → reduced confidence
/// let src = ConfidenceSource::ChiSquared {
///     chi_squared: 4.0,
///     fdr_rejected: false,
/// };
/// let result = compute_confidence(&src);
/// assert!(result.value < 0.6);
/// ```
// CALIBRATION: Each branch matches its TypeScript counterpart exactly.
// Observatory frontend Measured<T>→Opacity: opacity = 0.15 + confidence × 0.85
#[must_use]
pub fn compute_confidence(source: &ConfidenceSource) -> Measured<f64> {
    let raw = match source {
        ConfidenceSource::ChiSquared {
            chi_squared,
            fdr_rejected,
        } => {
            // 1 - p_value from chi-squared (df=1)
            // Abramowitz & Stegun erfc approximation
            let p = chi_squared_p_value(*chi_squared);
            let base = f64_max(0.15, 1.0 - p);
            if *fdr_rejected {
                base
            } else {
                // FDR correction: non-rejected signals get 60% of base confidence
                base * 0.6
            }
        }

        ConfidenceSource::SignalStrength {
            prr,
            signal_detected,
        } => {
            if *signal_detected {
                // PRR-scaled: 0.5 base + prr/15, capped at 0.95
                f64_min(0.95, 0.5 + prr / 15.0)
            } else {
                0.3
            }
        }

        ConfidenceSource::Severity { severity } => {
            // Floor at 0.15 for minimum visibility
            f64_max(0.15, *severity)
        }

        ConfidenceSource::RelevanceScore { score } => {
            // Direct mapping, clamped to valid range
            score.clamp(0.0, 1.0)
        }

        ConfidenceSource::StructuralCertainty { api_derived } => {
            if *api_derived {
                0.60
            } else {
                0.95
            }
        }
    };

    // Clamp to valid confidence range
    let clamped = raw.clamp(0.0, 1.0);
    Measured::new(clamped, Confidence::new(clamped))
}

/// Compute confidence for a batch of sources.
///
/// Returns one `Measured<f64>` per source, in the same order.
#[must_use]
pub fn compute_confidence_batch(sources: &[ConfidenceSource]) -> Vec<Measured<f64>> {
    sources.iter().map(compute_confidence).collect()
}

// ─── Chi-Squared P-Value (df=1) ─────────────────────────────────────────────

/// Compute p-value for chi-squared statistic with df=1.
///
/// Uses the relationship: p = erfc(sqrt(χ²/2)) for df=1.
/// erfc approximated via Abramowitz & Stegun (max error < 1.5e-7).
// CALIBRATION: Matches TypeScript `chiSquaredPValue()` in use-causality-data.ts.
// Reference: Abramowitz & Stegun, Handbook of Mathematical Functions, 7.1.26
fn chi_squared_p_value(chi_sq: f64) -> f64 {
    if chi_sq <= 0.0 {
        return 1.0; // No evidence → p = 1
    }

    let x = (chi_sq / 2.0).sqrt();
    erfc_approx(x)
}

/// Complementary error function approximation.
///
/// Abramowitz & Stegun formula 7.1.26 — rational approximation.
/// Maximum absolute error: 1.5 × 10⁻⁷.
fn erfc_approx(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc_approx(-x);
    }

    // A&S 7.1.26 constants
    let p = 0.327_591_1;
    let a1 = 0.254_829_592;
    let a2 = -0.284_496_736;
    let a3 = 1.421_413_741;
    let a4 = -1.453_152_027;
    let a5 = 1.061_405_429;

    let t = 1.0 / (1.0 + p * x);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;

    let poly = a1 * t + a2 * t2 + a3 * t3 + a4 * t4 + a5 * t5;

    // erfc(x) = poly * exp(-x²)
    // Use checked arithmetic to avoid overflow for large x
    let exp_neg_x2 = (-x * x).exp();
    poly * exp_neg_x2
}

// ─── f64 min/max without method calls (clippy-safe) ─────────────────────────

/// Returns the larger of two f64 values.
/// Avoids `f64::max()` which can trigger `clippy::float_cmp` in some contexts.
fn f64_max(a: f64, b: f64) -> f64 {
    if a >= b { a } else { b }
}

/// Returns the smaller of two f64 values.
fn f64_min(a: f64, b: f64) -> f64 {
    if a <= b { a } else { b }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ChiSquared ──────────────────────────────────────────────────────

    #[test]
    fn chi_squared_strong_signal() {
        // χ² = 10.0 with FDR-confirmed → high confidence
        let src = ConfidenceSource::ChiSquared {
            chi_squared: 10.0,
            fdr_rejected: true,
        };
        let m = compute_confidence(&src);
        assert!(m.value > 0.9, "Expected >0.9, got {}", m.value);
        assert!(m.confidence.value() > 0.9);
    }

    #[test]
    fn chi_squared_threshold() {
        // χ² = 3.841 is the 0.05 significance threshold (df=1)
        let src = ConfidenceSource::ChiSquared {
            chi_squared: 3.841,
            fdr_rejected: true,
        };
        let m = compute_confidence(&src);
        // p ≈ 0.05, so confidence ≈ 0.95
        assert!(m.value > 0.9, "Expected >0.9 at threshold, got {}", m.value);
    }

    #[test]
    fn chi_squared_weak_not_fdr() {
        // χ² = 4.0 but NOT FDR-rejected → confidence reduced by 0.6
        let src = ConfidenceSource::ChiSquared {
            chi_squared: 4.0,
            fdr_rejected: false,
        };
        let m = compute_confidence(&src);
        // Base ≈ 0.954, × 0.6 ≈ 0.572
        assert!(m.value < 0.7, "Expected <0.7 without FDR, got {}", m.value);
        assert!(m.value > 0.4, "Expected >0.4, got {}", m.value);
    }

    #[test]
    fn chi_squared_zero() {
        let src = ConfidenceSource::ChiSquared {
            chi_squared: 0.0,
            fdr_rejected: true,
        };
        let m = compute_confidence(&src);
        // p=1.0 → 1-1=0 → floor at 0.15
        assert!(
            (m.value - 0.15).abs() < 0.01,
            "Expected ~0.15, got {}",
            m.value
        );
    }

    #[test]
    fn chi_squared_negative() {
        let src = ConfidenceSource::ChiSquared {
            chi_squared: -5.0,
            fdr_rejected: true,
        };
        let m = compute_confidence(&src);
        // Negative → p=1.0 → floor at 0.15
        assert!(
            (m.value - 0.15).abs() < 0.01,
            "Expected ~0.15, got {}",
            m.value
        );
    }

    // ── SignalStrength ───────────────────────────────────────────────────

    #[test]
    fn signal_detected_high_prr() {
        let src = ConfidenceSource::SignalStrength {
            prr: 10.0,
            signal_detected: true,
        };
        let m = compute_confidence(&src);
        // 0.5 + 10/15 = 0.5 + 0.667 = 1.167 → capped at 0.95
        assert!(
            (m.value - 0.95).abs() < 0.01,
            "Expected ~0.95, got {}",
            m.value
        );
    }

    #[test]
    fn signal_detected_low_prr() {
        let src = ConfidenceSource::SignalStrength {
            prr: 2.0,
            signal_detected: true,
        };
        let m = compute_confidence(&src);
        // 0.5 + 2/15 = 0.633
        assert!(
            (m.value - 0.633).abs() < 0.01,
            "Expected ~0.633, got {}",
            m.value
        );
    }

    #[test]
    fn signal_not_detected() {
        let src = ConfidenceSource::SignalStrength {
            prr: 5.0,
            signal_detected: false,
        };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.3).abs() < f64::EPSILON,
            "Expected 0.3, got {}",
            m.value
        );
    }

    #[test]
    fn signal_prr_zero() {
        let src = ConfidenceSource::SignalStrength {
            prr: 0.0,
            signal_detected: true,
        };
        let m = compute_confidence(&src);
        // 0.5 + 0/15 = 0.5
        assert!(
            (m.value - 0.5).abs() < f64::EPSILON,
            "Expected 0.5, got {}",
            m.value
        );
    }

    // ── Severity ────────────────────────────────────────────────────────

    #[test]
    fn severity_high() {
        let src = ConfidenceSource::Severity { severity: 0.9 };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.9).abs() < f64::EPSILON,
            "Expected 0.9, got {}",
            m.value
        );
    }

    #[test]
    fn severity_low_floors() {
        let src = ConfidenceSource::Severity { severity: 0.05 };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.15).abs() < f64::EPSILON,
            "Expected 0.15 (floor), got {}",
            m.value
        );
    }

    #[test]
    fn severity_zero() {
        let src = ConfidenceSource::Severity { severity: 0.0 };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.15).abs() < f64::EPSILON,
            "Expected 0.15 (floor), got {}",
            m.value
        );
    }

    // ── RelevanceScore ──────────────────────────────────────────────────

    #[test]
    fn relevance_direct() {
        let src = ConfidenceSource::RelevanceScore { score: 0.75 };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.75).abs() < f64::EPSILON,
            "Expected 0.75, got {}",
            m.value
        );
    }

    #[test]
    fn relevance_clamped_above() {
        let src = ConfidenceSource::RelevanceScore { score: 1.5 };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 1.0).abs() < f64::EPSILON,
            "Expected 1.0 (clamped), got {}",
            m.value
        );
    }

    #[test]
    fn relevance_clamped_below() {
        let src = ConfidenceSource::RelevanceScore { score: -0.3 };
        let m = compute_confidence(&src);
        assert!(
            m.value.abs() < f64::EPSILON,
            "Expected 0.0 (clamped), got {}",
            m.value
        );
    }

    // ── StructuralCertainty ─────────────────────────────────────────────

    #[test]
    fn preset_structure() {
        let src = ConfidenceSource::StructuralCertainty { api_derived: false };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.95).abs() < f64::EPSILON,
            "Expected 0.95, got {}",
            m.value
        );
    }

    #[test]
    fn api_derived_structure() {
        let src = ConfidenceSource::StructuralCertainty { api_derived: true };
        let m = compute_confidence(&src);
        assert!(
            (m.value - 0.60).abs() < f64::EPSILON,
            "Expected 0.60, got {}",
            m.value
        );
    }

    // ── Batch ───────────────────────────────────────────────────────────

    #[test]
    fn batch_processes_all() {
        let sources = vec![
            ConfidenceSource::Severity { severity: 0.8 },
            ConfidenceSource::RelevanceScore { score: 0.5 },
            ConfidenceSource::StructuralCertainty { api_derived: true },
        ];
        let results = compute_confidence_batch(&sources);
        assert_eq!(results.len(), 3);
        assert!((results[0].value - 0.8).abs() < f64::EPSILON);
        assert!((results[1].value - 0.5).abs() < f64::EPSILON);
        assert!((results[2].value - 0.6).abs() < f64::EPSILON);
    }

    // ── erfc approximation accuracy ─────────────────────────────────────

    #[test]
    fn erfc_at_zero() {
        let result = erfc_approx(0.0);
        assert!(
            (result - 1.0).abs() < 0.001,
            "erfc(0) should be ~1.0, got {}",
            result
        );
    }

    #[test]
    fn erfc_at_one() {
        // erfc(1) ≈ 0.15730
        let result = erfc_approx(1.0);
        assert!(
            (result - 0.1573).abs() < 0.001,
            "erfc(1) should be ~0.157, got {}",
            result
        );
    }

    #[test]
    fn erfc_large_value() {
        // erfc(4) ≈ 1.54e-8 → essentially zero
        let result = erfc_approx(4.0);
        assert!(result < 0.001, "erfc(4) should be ~0, got {}", result);
    }

    // ── Serde round-trip ────────────────────────────────────────────────

    #[test]
    fn serde_chi_squared() {
        let src = ConfidenceSource::ChiSquared {
            chi_squared: 5.0,
            fdr_rejected: true,
        };
        let json = serde_json::to_string(&src).unwrap_or_default();
        assert!(json.contains("chi_squared"));
        assert!(json.contains("\"source\":\"chi_squared\""));
    }

    #[test]
    fn serde_all_variants() {
        let sources = vec![
            ConfidenceSource::ChiSquared {
                chi_squared: 3.841,
                fdr_rejected: false,
            },
            ConfidenceSource::SignalStrength {
                prr: 2.5,
                signal_detected: true,
            },
            ConfidenceSource::Severity { severity: 0.7 },
            ConfidenceSource::RelevanceScore { score: 0.85 },
            ConfidenceSource::StructuralCertainty { api_derived: false },
        ];
        for src in &sources {
            let json = serde_json::to_string(src).unwrap_or_default();
            assert!(!json.is_empty(), "Serialization should succeed");
        }
    }
}
