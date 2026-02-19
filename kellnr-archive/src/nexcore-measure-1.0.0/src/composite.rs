//! Composite health scoring: CrateHealth and WorkspaceHealth.
//!
//! ## Scoring Formula
//!
//! ```text
//! Health = (entropy_norm * 0.25 + test_density_norm * 0.30
//!         + coupling_norm * 0.25 + freshness_norm * 0.20) * 10
//! ```
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | raw metrics → normalized [0,1] → composite |
//! | T1: Boundary (δ) | score clamped to [0, 10] |
//! | T1: Comparison (κ) | rating thresholds |

use crate::entropy;
use crate::types::*;

/// Weights for composite scoring.
/// Weights for composite scoring.
const W_ENTROPY: f64 = 0.2;
const W_TEST_DENSITY: f64 = 0.25;
const W_COUPLING: f64 = 0.2;
const W_FRESHNESS: f64 = 0.2;
const W_CDI: f64 = 0.15;

/// Compute crate health from measurement.
///
/// `last_modified_epoch`: Unix epoch of last source file modification.
/// `now_epoch`: current time as Unix epoch seconds.
pub fn crate_health(
    measurement: &CrateMeasurement,
    last_modified_epoch: i64,
    now_epoch: i64,
) -> CrateHealth {
    let components = compute_components(measurement, last_modified_epoch, now_epoch);
    let raw = components.entropy_norm * W_ENTROPY
        + components.test_density_norm * W_TEST_DENSITY
        + components.coupling_norm * W_COUPLING
        + components.freshness_norm * W_FRESHNESS
        + components.cdi_norm * W_CDI;
    let score = HealthScore::new(raw * 10.0);
    let rating = score.rating();

    CrateHealth {
        crate_id: measurement.crate_id.clone(),
        score,
        rating,
        components,
    }
}

/// Compute all normalized components.
fn compute_components(m: &CrateMeasurement, last_modified: i64, now: i64) -> HealthComponents {
    HealthComponents {
        entropy_norm: normalize_entropy(m),
        test_density_norm: normalize_test_density(m.test_density),
        coupling_norm: normalize_coupling(m.coupling_ratio),
        freshness_norm: normalize_freshness(last_modified, now),
        cdi_norm: normalize_cdi(m.cdi),
    }
}

/// Entropy normalization: optimal at H/H_max ∈ [0.6, 0.9].
///
/// Below 0.6: penalize god-modules.
/// Above 0.9: penalize over-fragmentation.
fn normalize_entropy(m: &CrateMeasurement) -> f64 {
    if m.module_count <= 1 {
        return 0.0; // single module → low diversity
    }
    let h_max = entropy::max_entropy(m.module_count)
        .map(|e| e.value())
        .unwrap_or(1.0);
    if h_max < f64::EPSILON {
        return 0.0;
    }
    let ratio = m.entropy.value() / h_max;
    // Triangular penalty: peak at [0.6, 0.9]
    (if ratio < 0.6 {
        ratio / 0.6
    } else if ratio <= 0.9 {
        1.0
    } else {
        // Slight penalty above 0.9, but not as severe
        1.0 - (ratio - 0.9) * 3.0 // reaches 0.7 at ratio=1.0
    })
    .clamp(0.0, 1.0)
}

/// Test density sigmoid: centered at 10 tests/KLOC.
///
/// f(x) = 1 / (1 + e^(-0.3*(x - 10)))
fn normalize_test_density(td: TestDensity) -> f64 {
    let x = td.value();
    1.0 / (1.0 + (-0.3 * (x - 10.0)).exp())
}

/// Coupling normalization: optimal at ~0.3 (balanced dependency ratio).
///
/// Pure consumers (1.0) and pure producers (0.0) are less healthy.
fn normalize_coupling(cr: CouplingRatio) -> f64 {
    let x = cr.value();
    // Bell curve centered at 0.3
    let optimal = 0.3;
    let sigma = 0.25;
    (-(x - optimal).powi(2) / (2.0 * sigma * sigma)).exp()
}

/// Freshness: half-life decay from last modification.
///
/// Half-life of 90 days (7,776,000 seconds).
fn normalize_freshness(last_modified_epoch: i64, now_epoch: i64) -> f64 {
    let age_secs = (now_epoch - last_modified_epoch).max(0) as f64;
    let half_life = 90.0 * 24.0 * 3600.0; // 90 days
    let lambda = (2.0_f64).ln() / half_life;
    (-lambda * age_secs).exp()
}

/// Code Density Index normalization.
///
/// Sigmoid centered at 0.15 (15% semantic tokens is typical well-structured Rust).
/// Range: [0, 1]
fn normalize_cdi(cdi: CodeDensityIndex) -> f64 {
    let x = cdi.value();
    // Centered at 0.15, steepness 20.0
    1.0 / (1.0 + (-20.0 * (x - 0.15)).exp())
}

/// Compute workspace health from individual measurements.
pub fn workspace_health(measurements: &[CrateMeasurement], now_epoch: i64) -> WorkspaceHealth {
    let crate_healths: Vec<CrateHealth> = measurements
        .iter()
        .map(|m| {
            // Use timestamp as proxy for last_modified
            crate_health(m, m.timestamp.epoch_secs(), now_epoch)
        })
        .collect();

    let mut dist = RatingDistribution::default();
    let mut score_sum = 0.0_f64;

    for ch in &crate_healths {
        score_sum += ch.score.value();
        dist.add(ch.rating);
    }

    let mean = if crate_healths.is_empty() {
        0.0
    } else {
        score_sum / (crate_healths.len() as f64)
    };
    let mean_score = HealthScore::new(mean);

    WorkspaceHealth {
        timestamp: MeasureTimestamp::now(),
        mean_score,
        mean_rating: mean_score.rating(),
        rating_distribution: dist,
        crate_healths,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_measurement(modules: usize, tests: usize, loc: usize) -> CrateMeasurement {
        let dist: Vec<usize> = if modules > 0 {
            vec![loc / modules; modules]
        } else {
            vec![]
        };
        let h = entropy::shannon_entropy(&dist).unwrap_or_else(|_| Entropy::new(0.0));
        let r = entropy::redundancy(&dist).unwrap_or_else(|_| Probability::new(0.0));
        let kloc = (loc as f64) / 1000.0;
        let td = if kloc > f64::EPSILON {
            (tests as f64) / kloc
        } else {
            0.0
        };
        CrateMeasurement {
            crate_id: CrateId::new("test-crate"),
            timestamp: MeasureTimestamp::now(),
            loc,
            test_count: tests,
            module_count: modules,
            module_loc_distribution: dist,
            entropy: h,
            redundancy: r,
            test_density: TestDensity::new(td),
            fan_in: 2,
            fan_out: 3,
            coupling_ratio: CouplingRatio::new(0.3),
            cdi: CodeDensityIndex::new(0.15),
        }
    }

    #[test]
    fn score_always_in_0_10() {
        let scenarios = vec![(1, 0, 100), (10, 50, 5000), (20, 200, 10000), (2, 1, 50)];
        let now = MeasureTimestamp::now().epoch_secs();
        for (modules, tests, loc) in scenarios {
            let m = sample_measurement(modules, tests, loc);
            let h = crate_health(&m, now, now);
            assert!(
                h.score.value() >= 0.0 && h.score.value() <= 10.0,
                "score {} out of [0,10] for m={}/t={}/l={}",
                h.score.value(),
                modules,
                tests,
                loc
            );
        }
    }

    #[test]
    fn zero_tests_gives_lower_score() {
        let m_zero = sample_measurement(5, 0, 1000);
        let m_good = sample_measurement(5, 50, 5000);
        let now = MeasureTimestamp::now().epoch_secs();
        let h_zero = crate_health(&m_zero, now, now);
        let h_good = crate_health(&m_good, now, now);
        assert!(
            h_zero.score.value() < h_good.score.value(),
            "zero tests {} should < good tests {}",
            h_zero.score.value(),
            h_good.score.value()
        );
    }

    #[test]
    fn good_crate_gives_higher_score() {
        // 10 modules, 100 tests, 10KLOC = 10 tests/KLOC
        let m = sample_measurement(10, 100, 10000);
        let now = MeasureTimestamp::now().epoch_secs();
        let h = crate_health(&m, now, now);
        assert!(h.score.value() > 4.0, "good crate = {}", h.score.value());
    }

    #[test]
    fn freshness_decays() {
        let now = 1_000_000_000;
        let recent = normalize_freshness(now - 3600, now); // 1 hour ago
        let old = normalize_freshness(now - 180 * 86400, now); // 180 days ago
        assert!(recent > old, "recent={} should > old={}", recent, old);
    }

    #[test]
    fn test_density_sigmoid_centered() {
        let low = normalize_test_density(TestDensity::new(0.0));
        let mid = normalize_test_density(TestDensity::new(10.0));
        let high = normalize_test_density(TestDensity::new(50.0));
        assert!(low < mid, "low={} < mid={}", low, mid);
        assert!(mid < high, "mid={} < high={}", mid, high);
        assert!((mid - 0.5).abs() < 0.01, "mid={} should be ~0.5", mid);
    }

    #[test]
    fn coupling_optimal_at_03() {
        let optimal = normalize_coupling(CouplingRatio::new(0.3));
        let extreme_low = normalize_coupling(CouplingRatio::new(0.0));
        let extreme_high = normalize_coupling(CouplingRatio::new(1.0));
        assert!(optimal > extreme_low);
        assert!(optimal > extreme_high);
        assert!((optimal - 1.0).abs() < 1e-10);
    }

    #[test]
    fn workspace_health_aggregates() {
        let now = MeasureTimestamp::now().epoch_secs();
        let m1 = sample_measurement(5, 25, 2500);
        let m2 = sample_measurement(10, 100, 10000);
        let wh = workspace_health(&[m1, m2], now);
        assert_eq!(wh.crate_healths.len(), 2);
        assert!(wh.mean_score.value() >= 0.0 && wh.mean_score.value() <= 10.0);
    }

    #[test]
    fn empty_workspace_health() {
        let wh = workspace_health(&[], MeasureTimestamp::now().epoch_secs());
        assert_eq!(wh.crate_healths.len(), 0);
        assert!(wh.mean_score.value().abs() < f64::EPSILON);
    }
}
