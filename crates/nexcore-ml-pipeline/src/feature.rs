//! PV-specific feature engineering.
//!
//! Transforms raw FAERS contingency data into 12-element feature vectors
//! suitable for ML classification.
//!
//! ## Primitive Foundation
//! - T1: Mapping (μ) — contingency table → disproportionality scores
//! - T1: Quantity (N) — counts, ratios, rates

use crate::types::{FEATURE_NAMES, RawPairData, Sample};

/// Errors during feature extraction.
#[derive(Debug, nexcore_error::Error)]
pub enum FeatureError {
    /// Contingency table has zero marginals.
    #[error("zero marginal in contingency table: {detail}")]
    ZeroMarginal {
        /// Which marginal is zero.
        detail: String,
    },
}

/// Extract a 12-element feature vector from raw FAERS pair data.
///
/// Features:
/// 0. PRR (Proportional Reporting Ratio)
/// 1. ROR (Reporting Odds Ratio)
/// 2. IC (Information Component, log2)
/// 3. EBGM (Empirical Bayesian Geometric Mean, simplified)
/// 4. log(case_count + 1)
/// 5. HCP reporter ratio
/// 6. Consumer reporter ratio
/// 7. Serious outcome ratio
/// 8. Death outcome ratio
/// 9. Hospitalization ratio
/// 10. Median time-to-onset (days), 0.0 if unknown
/// 11. Reporting velocity (cases/quarter)
///
/// # Errors
/// Returns `FeatureError` if contingency table has invalid marginals.
pub fn extract_features(raw: &RawPairData) -> Result<Sample, FeatureError> {
    let ct = &raw.contingency;
    let a = ct.a as f64;
    let b = ct.b as f64;
    let c = ct.c as f64;
    let d = ct.d as f64;
    let n = a + b + c + d;

    if n < 1.0 {
        return Err(FeatureError::ZeroMarginal {
            detail: "total N is zero".into(),
        });
    }

    // Disproportionality scores (with Haldane correction +0.5 for zeros)
    let prr = compute_prr(a, b, c, d);
    let ror = compute_ror(a, b, c, d);
    let ic = compute_ic(a, b, c, d, n);
    let ebgm = compute_ebgm(a, b, c, d, n);

    // Case count (log-transformed for scale normalization)
    let log_cases = (a + 1.0).ln();

    // Reporter ratios
    let rep = &raw.reporters;
    let total_reporters = (rep.hcp + rep.consumer + rep.other).max(1) as f64;
    let hcp_ratio = rep.hcp as f64 / total_reporters;
    let consumer_ratio = rep.consumer as f64 / total_reporters;

    // Outcome ratios
    let out = &raw.outcomes;
    let total_cases = out.total.max(1) as f64;
    let serious_ratio = out.serious as f64 / total_cases;
    let death_ratio = out.death as f64 / total_cases;
    let hosp_ratio = out.hospitalization as f64 / total_cases;

    // Temporal
    let tto = raw.temporal.median_tto_days.unwrap_or(0.0);
    let velocity = raw.temporal.velocity;

    let features = vec![
        prr,
        ror,
        ic,
        ebgm,
        log_cases,
        hcp_ratio,
        consumer_ratio,
        serious_ratio,
        death_ratio,
        hosp_ratio,
        tto,
        velocity,
    ];

    Ok(Sample {
        drug: ct.drug.clone(),
        event: ct.event.clone(),
        features,
        label: None,
    })
}

/// Feature names for the 12-element vector.
#[must_use]
pub fn feature_names() -> Vec<String> {
    FEATURE_NAMES.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Disproportionality formulas
// ---------------------------------------------------------------------------

/// PRR = (a/(a+b)) / (c/(c+d)) with Haldane correction.
fn compute_prr(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let num = (a + 0.5) / (a + b + 1.0);
    let den = (c + 0.5) / (c + d + 1.0);
    if den < f64::EPSILON {
        return 0.0;
    }
    num / den
}

/// ROR = (a*d) / (b*c) with Haldane correction.
fn compute_ror(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let num = (a + 0.5) * (d + 0.5);
    let den = (b + 0.5) * (c + 0.5);
    if den < f64::EPSILON {
        return 0.0;
    }
    num / den
}

/// IC = log2(observed / expected) with shrinkage.
fn compute_ic(a: f64, b: f64, c: f64, _d: f64, n: f64) -> f64 {
    let observed = a + 0.5;
    let expected = ((a + b) * (a + c)) / n;
    let expected = expected + 0.5;
    if expected < f64::EPSILON {
        return 0.0;
    }
    (observed / expected).log2()
}

/// Simplified EBGM = (a + alpha) / (E + alpha) with shrinkage toward prior.
/// Alpha=0.5 is the Haldane correction, consistent with PRR/ROR/IC.
fn compute_ebgm(a: f64, b: f64, c: f64, _d: f64, n: f64) -> f64 {
    let expected = ((a + b) * (a + c)) / n;
    if expected < f64::EPSILON {
        return 0.0;
    }
    let alpha = 0.5; // Haldane prior weight
    (a + alpha) / (expected + alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_raw_data(a: u64, b: u64, c: u64, d: u64) -> RawPairData {
        RawPairData {
            contingency: ContingencyTable {
                drug: "metformin".into(),
                event: "lactic_acidosis".into(),
                a,
                b,
                c,
                d,
            },
            reporters: ReporterBreakdown {
                hcp: 60,
                consumer: 30,
                other: 10,
            },
            outcomes: OutcomeBreakdown {
                total: 100,
                serious: 70,
                death: 10,
                hospitalization: 40,
            },
            temporal: TemporalData {
                median_tto_days: Some(14.0),
                velocity: 5.2,
            },
        }
    }

    #[test]
    fn extract_features_basic() {
        let raw = make_raw_data(150, 5000, 3000, 500_000);
        let sample = extract_features(&raw);
        assert!(sample.is_ok());
        let sample = sample.unwrap_or_else(|_| Sample {
            drug: String::new(),
            event: String::new(),
            features: vec![],
            label: None,
        });
        assert_eq!(sample.features.len(), 12);
        assert!(sample.features[0] > 1.0, "PRR should indicate signal");
        assert_eq!(sample.drug, "metformin");
    }

    #[test]
    fn extract_features_zero_total() {
        let raw = make_raw_data(0, 0, 0, 0);
        assert!(extract_features(&raw).is_err());
    }

    #[test]
    fn prr_known_value() {
        // a=100, b=900, c=200, d=8800
        // PRR = (100/1000) / (200/9000) = 0.1 / 0.0222 ≈ 4.5
        let prr = compute_prr(100.0, 900.0, 200.0, 8800.0);
        // With Haldane: (100.5/1001) / (200.5/9001) ≈ 4.505
        assert!(prr > 4.0 && prr < 5.0, "PRR = {prr}, expected ~4.5");
    }

    #[test]
    fn ror_known_value() {
        // ROR = (100*8800) / (900*200) = 880000/180000 ≈ 4.89
        let ror = compute_ror(100.0, 900.0, 200.0, 8800.0);
        assert!(ror > 4.0 && ror < 6.0, "ROR = {ror}, expected ~4.9");
    }

    #[test]
    fn ic_positive_for_signal() {
        let ic = compute_ic(150.0, 5000.0, 3000.0, 500_000.0, 508_150.0);
        assert!(ic > 0.0, "IC should be positive for signal: {ic}");
    }

    #[test]
    fn feature_names_count() {
        assert_eq!(feature_names().len(), 12);
    }
}
