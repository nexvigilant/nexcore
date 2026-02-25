//! # Compound Growth Detection Engine
//!
//! Detects growth phases, bottlenecks, and component contributions from
//! a time-series of [`BasisSnapshot`]s tracked by [`CompoundTracker`].
//!
//! ## Phase Detection
//!
//! Classifies the growth trajectory into one of six phases:
//! Dormant -> Ignition -> Acceleration -> Sustain -> Plateau -> Decline
//!
//! ## Bottleneck Detection
//!
//! Identifies which component of V = B x eta x r is limiting growth.

use crate::compound::{BasisSnapshot, CompoundTracker};
use serde::{Deserialize, Serialize};

// ============================================================================
// GrowthPhase — lifecycle classification
// ============================================================================

/// Compound growth lifecycle phase.
///
/// Tier: T2-P (kappa -- Comparison/Classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GrowthPhase {
    /// No snapshots or only one — insufficient data.
    Dormant,
    /// Early stage (2-3 snapshots), velocity rising.
    Ignition,
    /// Growth rate increasing (second derivative > 0).
    Acceleration,
    /// Growth rate stable and positive (~1.0-1.05).
    Sustain,
    /// Growth rate approximately 1.0 (within +/-0.05).
    Plateau,
    /// Growth rate consistently below 0.95.
    Decline,
}

impl GrowthPhase {
    /// Human-readable label for this phase.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Dormant => "Dormant",
            Self::Ignition => "Ignition",
            Self::Acceleration => "Acceleration",
            Self::Sustain => "Sustain",
            Self::Plateau => "Plateau",
            Self::Decline => "Decline",
        }
    }
}

// ============================================================================
// Bottleneck — limiting factor classification
// ============================================================================

/// Which component of V = B x eta x r is most limiting.
///
/// Tier: T2-P (kappa -- Comparison/Classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Bottleneck {
    /// Basis is small while eta and r are healthy.
    BasisLimited,
    /// Transfer efficiency is low (poor tier mix).
    EfficiencyLimited,
    /// Reuse rate is low (not leveraging existing primitives).
    ReuseLimited,
    /// No single dominant bottleneck.
    Balanced,
}

impl Bottleneck {
    /// Human-readable label for this bottleneck.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BasisLimited => "basis",
            Self::EfficiencyLimited => "efficiency",
            Self::ReuseLimited => "reuse",
            Self::Balanced => "balanced",
        }
    }
}

// ============================================================================
// ComponentAnalysis — log-decomposition of velocity
// ============================================================================

/// Decomposition of velocity into component contributions.
///
/// Tier: T2-C (varsigma + kappa + proportional -- State + Comparison + Proportion)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComponentAnalysis {
    /// Raw basis size as f64.
    pub basis_value: f64,
    /// Raw transfer efficiency [0, 1].
    pub efficiency_value: f64,
    /// Raw reuse rate [0, 1].
    pub reuse_value: f64,
    /// Percentage of log-velocity attributed to basis.
    pub basis_contribution_pct: f64,
    /// Percentage of log-velocity attributed to efficiency.
    pub efficiency_contribution_pct: f64,
    /// Percentage of log-velocity attributed to reuse.
    pub reuse_contribution_pct: f64,
    /// Which component is weakest: "basis", "efficiency", or "reuse".
    pub weakest_component: String,
}

// ============================================================================
// DetectionResult — full analysis output
// ============================================================================

/// Complete detection result combining phase, bottleneck, and analysis.
///
/// Tier: T3 (Full domain detection result)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DetectionResult {
    /// Detected growth phase.
    pub phase: GrowthPhase,
    /// Detected bottleneck.
    pub bottleneck: Bottleneck,
    /// Current velocity (latest snapshot).
    pub current_velocity: f64,
    /// Most recent growth rate, if available.
    pub latest_growth_rate: Option<f64>,
    /// Geometric mean of all growth rates.
    pub avg_growth_rate: f64,
    /// Component analysis (None if tracker is empty).
    pub component_analysis: Option<ComponentAnalysis>,
    /// Actionable recommendation string.
    pub recommendation: String,
    /// Number of snapshots analyzed.
    pub snapshot_count: usize,
}

// ============================================================================
// CompoundDetector — stateless detection engine
// ============================================================================

/// Stateless compound growth detector.
///
/// Tier: T3 (Domain-specific detector)
#[non_exhaustive]
pub struct CompoundDetector;

/// Normalization ceiling for basis size (50 = excellent basis).
const BASIS_NORM_CEILING: f64 = 50.0;

/// Threshold for "within 10% of second-lowest" in bottleneck detection.
const BALANCED_THRESHOLD: f64 = 0.10;

impl CompoundDetector {
    /// Run full detection on a [`CompoundTracker`].
    #[must_use]
    pub fn detect(tracker: &CompoundTracker) -> DetectionResult {
        let velocities = tracker.velocity_history();
        let phase = Self::detect_phase(&velocities);
        let growth_rates = tracker.growth_rates();
        let latest_growth_rate = growth_rates.last().copied();

        let (bottleneck, component_analysis) = match tracker.latest() {
            Some(snap) => (
                Self::detect_bottleneck(snap),
                Some(Self::analyze_components(snap)),
            ),
            None => (Bottleneck::Balanced, None),
        };

        let recommendation = Self::recommend(phase, bottleneck);

        DetectionResult {
            phase,
            bottleneck,
            current_velocity: tracker.current_velocity(),
            latest_growth_rate,
            avg_growth_rate: tracker.avg_growth_rate(),
            component_analysis,
            recommendation,
            snapshot_count: tracker.len(),
        }
    }

    /// Detect growth phase from a velocity history.
    #[must_use]
    pub fn detect_phase(velocities: &[f64]) -> GrowthPhase {
        match velocities.len() {
            0 | 1 => GrowthPhase::Dormant,
            2 | 3 => {
                let first = velocities.first().copied().unwrap_or(0.0);
                let last = velocities.last().copied().unwrap_or(0.0);
                if last > first {
                    GrowthPhase::Ignition
                } else {
                    GrowthPhase::Decline
                }
            }
            _ => {
                // Compute pairwise growth rates
                let rates: Vec<f64> = velocities
                    .windows(2)
                    .map(|w| {
                        let prev = w.first().copied().unwrap_or(0.0);
                        let next = w.get(1).copied().unwrap_or(0.0);
                        if prev == 0.0 { 0.0 } else { next / prev }
                    })
                    .collect();

                // Take last 3 growth rates
                let n = rates.len();
                let tail: Vec<f64> = if n >= 3 {
                    rates.get(n.saturating_sub(3)..).unwrap_or(&rates).to_vec()
                } else {
                    rates
                };

                // Check if increasing (for acceleration detection)
                let increasing = tail
                    .windows(2)
                    .all(|w| w.get(1).copied().unwrap_or(0.0) > w.first().copied().unwrap_or(0.0));

                if tail.iter().all(|r| *r > 1.05) && increasing {
                    GrowthPhase::Acceleration
                } else if tail.iter().all(|r| *r > 1.0 && *r <= 1.05) {
                    GrowthPhase::Sustain
                } else if tail.iter().all(|r| *r >= 0.95 && *r <= 1.05) {
                    GrowthPhase::Plateau
                } else if tail.iter().all(|r| *r < 0.95) {
                    GrowthPhase::Decline
                } else {
                    // Mixed — use median of last 3
                    let mut sorted = tail;
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);

                    if median > 1.05 {
                        GrowthPhase::Acceleration
                    } else if median > 1.0 {
                        GrowthPhase::Sustain
                    } else if median > 0.95 {
                        GrowthPhase::Plateau
                    } else {
                        GrowthPhase::Decline
                    }
                }
            }
        }
    }

    /// Detect the primary bottleneck from a single snapshot.
    #[must_use]
    pub fn detect_bottleneck(snapshot: &BasisSnapshot) -> Bottleneck {
        let (b, eta, r) = snapshot.component_triple();

        // Normalize to [0, 1]
        let b_norm = (b / BASIS_NORM_CEILING).min(1.0);
        let eta_norm = eta; // already [0, 1]
        let r_norm = r; // already [0, 1]

        let mut components = [
            (b_norm, Bottleneck::BasisLimited),
            (eta_norm, Bottleneck::EfficiencyLimited),
            (r_norm, Bottleneck::ReuseLimited),
        ];

        // Sort by normalized value ascending
        components.sort_by(|lhs, rhs| {
            lhs.0
                .partial_cmp(&rhs.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let lowest = components.first().map(|c| c.0).unwrap_or(0.0);
        let second_lowest = components.get(1).map(|c| c.0).unwrap_or(0.0);

        // If within 10% of second-lowest, or both are zero, balanced
        let is_balanced = (lowest == 0.0 && second_lowest == 0.0)
            || (second_lowest > 0.0
                && (second_lowest - lowest) / second_lowest <= BALANCED_THRESHOLD);
        if is_balanced {
            Bottleneck::Balanced
        } else {
            components
                .first()
                .map(|c| c.1)
                .unwrap_or(Bottleneck::Balanced)
        }
    }

    /// Analyze component contributions using log decomposition.
    ///
    /// Since V = B x eta x r, we have ln(V) = ln(B) + ln(eta) + ln(r).
    /// Each component's contribution is its share of the log-sum.
    #[must_use]
    pub fn analyze_components(snapshot: &BasisSnapshot) -> ComponentAnalysis {
        let (b, eta, r) = snapshot.component_triple();

        // Normalize for weakest-component detection
        let b_norm = (b / BASIS_NORM_CEILING).min(1.0);

        let weakest = if b_norm <= eta && b_norm <= r {
            "basis"
        } else if eta <= b_norm && eta <= r {
            "efficiency"
        } else {
            "reuse"
        };

        // Log decomposition (only valid when all components > 0)
        let (basis_pct, efficiency_pct, reuse_pct) = if b > 0.0 && eta > 0.0 && r > 0.0 {
            let ln_b = b.ln();
            let ln_eta = eta.ln();
            let ln_r = r.ln();
            let ln_v = ln_b + ln_eta + ln_r;

            if ln_v.abs() > f64::EPSILON {
                (
                    (ln_b / ln_v) * 100.0,
                    (ln_eta / ln_v) * 100.0,
                    (ln_r / ln_v) * 100.0,
                )
            } else {
                (33.3, 33.3, 33.3)
            }
        } else {
            (0.0, 0.0, 0.0)
        };

        ComponentAnalysis {
            basis_value: b,
            efficiency_value: eta,
            reuse_value: r,
            basis_contribution_pct: basis_pct,
            efficiency_contribution_pct: efficiency_pct,
            reuse_contribution_pct: reuse_pct,
            weakest_component: weakest.to_string(),
        }
    }

    /// Generate a recommendation string based on phase and bottleneck.
    #[must_use]
    pub fn recommend(phase: GrowthPhase, bottleneck: Bottleneck) -> String {
        match phase {
            GrowthPhase::Dormant => {
                "Record more sessions to enable detection. Need >= 2 snapshots.".to_string()
            }
            GrowthPhase::Ignition => match bottleneck {
                Bottleneck::BasisLimited => {
                    "Early growth. Expand primitive basis -- focus on T1 and T2-P extraction."
                        .to_string()
                }
                Bottleneck::EfficiencyLimited => {
                    "Early growth. Shift investment toward higher-confidence tiers (T1 > T2-P > T2-C).".to_string()
                }
                Bottleneck::ReuseLimited => {
                    "Early growth. Increase primitive reuse -- catalog and reference existing primitives.".to_string()
                }
                Bottleneck::Balanced => {
                    "Early growth. All components balanced -- continue expanding across tiers."
                        .to_string()
                }
            },
            GrowthPhase::Acceleration => {
                format!(
                    "Strong compound growth. Maintain current strategy. {} is limiting ceiling.",
                    bottleneck.label()
                )
            }
            GrowthPhase::Sustain => {
                format!(
                    "Healthy sustained growth. Address {} to unlock next acceleration phase.",
                    bottleneck.label()
                )
            }
            GrowthPhase::Plateau => match bottleneck {
                Bottleneck::BasisLimited => {
                    "Growth stalled -- basis exhausted. Extract new T2-P primitives to reignite."
                        .to_string()
                }
                Bottleneck::EfficiencyLimited => {
                    "Growth stalled -- tier mix suboptimal. Replace T3 with T2-P equivalents."
                        .to_string()
                }
                Bottleneck::ReuseLimited => {
                    "Growth stalled -- low reuse. Systematize primitive cataloging for higher reuse."
                        .to_string()
                }
                Bottleneck::Balanced => {
                    "Growth stalled -- all components equally constrained. Step-change needed across all dimensions.".to_string()
                }
            },
            GrowthPhase::Decline => {
                format!(
                    "Compound growth declining. Urgent: address {}. Consider basis refresh.",
                    bottleneck.label()
                )
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compound::BasisSnapshot;

    fn snap(
        session: &str,
        t1: u32,
        t2p: u32,
        t2c: u32,
        t3: u32,
        reused: u32,
        total: u32,
    ) -> BasisSnapshot {
        BasisSnapshot {
            session: session.to_string(),
            t1_count: t1,
            t2_p_count: t2p,
            t2_c_count: t2c,
            t3_count: t3,
            reused,
            total_needed: total,
        }
    }

    // ============================
    // Phase Detection Tests
    // ============================

    #[test]
    fn test_phase_dormant_empty() {
        assert_eq!(CompoundDetector::detect_phase(&[]), GrowthPhase::Dormant);
    }

    #[test]
    fn test_phase_dormant_single() {
        assert_eq!(
            CompoundDetector::detect_phase(&[10.0]),
            GrowthPhase::Dormant
        );
    }

    #[test]
    fn test_phase_ignition() {
        // 2 snapshots, rising
        assert_eq!(
            CompoundDetector::detect_phase(&[5.0, 10.0]),
            GrowthPhase::Ignition
        );
        // 3 snapshots, rising
        assert_eq!(
            CompoundDetector::detect_phase(&[5.0, 8.0, 12.0]),
            GrowthPhase::Ignition
        );
    }

    #[test]
    fn test_phase_acceleration() {
        // 5 snapshots with accelerating rates (each ratio > 1.05 and increasing)
        // Rates: 1.20, 1.25, 1.30, 1.35 — all > 1.05 and increasing
        let velocities = vec![10.0, 12.0, 15.0, 19.5, 26.325];
        let phase = CompoundDetector::detect_phase(&velocities);
        assert_eq!(phase, GrowthPhase::Acceleration);
    }

    #[test]
    fn test_phase_sustain() {
        // Stable growth rates between 1.0 and 1.05
        // Rates: 1.02, 1.03, 1.02
        let velocities = vec![100.0, 102.0, 105.06, 107.1612];
        let phase = CompoundDetector::detect_phase(&velocities);
        assert_eq!(phase, GrowthPhase::Sustain);
    }

    #[test]
    fn test_phase_plateau() {
        // Growth rates very close to 1.0 (within 0.95 - 1.05)
        // Rates: 1.00, 0.99, 1.01
        let velocities = vec![100.0, 100.0, 99.0, 99.99];
        let phase = CompoundDetector::detect_phase(&velocities);
        assert_eq!(phase, GrowthPhase::Plateau);
    }

    #[test]
    fn test_phase_decline() {
        // Growth rates all < 0.95
        // Rates: 0.90, 0.85, 0.80
        let velocities = vec![100.0, 90.0, 76.5, 61.2];
        let phase = CompoundDetector::detect_phase(&velocities);
        assert_eq!(phase, GrowthPhase::Decline);
    }

    // ============================
    // Bottleneck Detection Tests
    // ============================

    #[test]
    fn test_bottleneck_basis() {
        // Small basis (5), high efficiency and reuse
        let s = snap("b", 3, 1, 1, 0, 5, 5);
        // B=5, B_norm=5/50=0.10, eta=high, r=1.0
        assert_eq!(
            CompoundDetector::detect_bottleneck(&s),
            Bottleneck::BasisLimited
        );
    }

    #[test]
    fn test_bottleneck_efficiency() {
        // Large basis, low efficiency (all T3), high reuse
        let s = snap("e", 0, 0, 0, 50, 50, 50);
        // B=50, B_norm=1.0, eta=0.4, r=1.0
        assert_eq!(
            CompoundDetector::detect_bottleneck(&s),
            Bottleneck::EfficiencyLimited
        );
    }

    #[test]
    fn test_bottleneck_reuse() {
        // Large basis, high efficiency, low reuse
        let s = snap("r", 40, 5, 5, 0, 5, 50);
        // B=50, B_norm=1.0, eta~high, r=0.1
        assert_eq!(
            CompoundDetector::detect_bottleneck(&s),
            Bottleneck::ReuseLimited
        );
    }

    #[test]
    fn test_bottleneck_balanced() {
        // All components roughly similar (within 10%)
        // B=50 => B_norm=1.0, all T1 => eta=1.0, reuse=50/50 => r=1.0
        let s = snap("bal", 50, 0, 0, 0, 50, 50);
        assert_eq!(
            CompoundDetector::detect_bottleneck(&s),
            Bottleneck::Balanced
        );
    }

    // ============================
    // Full Detection Tests
    // ============================

    #[test]
    fn test_full_detection() {
        let mut tracker = CompoundTracker::new();
        tracker.record(snap("s1", 15, 2, 13, 0, 28, 30));
        tracker.record(snap("s2", 15, 4, 15, 0, 32, 34));
        tracker.record(snap("s3", 15, 6, 17, 0, 36, 38));

        let result = CompoundDetector::detect(&tracker);
        assert_eq!(result.snapshot_count, 3);
        assert!(result.current_velocity > 0.0);
        assert!(result.component_analysis.is_some());
        assert!(!result.recommendation.is_empty());
        assert!(result.latest_growth_rate.is_some());
    }

    #[test]
    fn test_full_detection_empty() {
        let tracker = CompoundTracker::new();
        let result = CompoundDetector::detect(&tracker);
        assert_eq!(result.phase, GrowthPhase::Dormant);
        assert_eq!(result.bottleneck, Bottleneck::Balanced);
        assert_eq!(result.snapshot_count, 0);
        assert!(result.component_analysis.is_none());
        assert!(result.latest_growth_rate.is_none());
    }

    // ============================
    // Recommendation Tests
    // ============================

    #[test]
    fn test_recommendation_not_empty() {
        let phases = [
            GrowthPhase::Dormant,
            GrowthPhase::Ignition,
            GrowthPhase::Acceleration,
            GrowthPhase::Sustain,
            GrowthPhase::Plateau,
            GrowthPhase::Decline,
        ];
        let bottlenecks = [
            Bottleneck::BasisLimited,
            Bottleneck::EfficiencyLimited,
            Bottleneck::ReuseLimited,
            Bottleneck::Balanced,
        ];

        for phase in &phases {
            for bottleneck in &bottlenecks {
                let rec = CompoundDetector::recommend(*phase, *bottleneck);
                assert!(
                    !rec.is_empty(),
                    "Empty recommendation for {:?} x {:?}",
                    phase,
                    bottleneck
                );
            }
        }
    }

    #[test]
    fn test_component_analysis() {
        let s = snap("a", 15, 2, 13, 0, 28, 30);
        let analysis = CompoundDetector::analyze_components(&s);
        assert!(analysis.basis_value > 0.0);
        assert!(analysis.efficiency_value > 0.0);
        assert!(analysis.reuse_value > 0.0);
        assert!(!analysis.weakest_component.is_empty());
    }

    #[test]
    fn test_component_analysis_zero() {
        let s = snap("z", 0, 0, 0, 0, 0, 0);
        let analysis = CompoundDetector::analyze_components(&s);
        assert!((analysis.basis_value - 0.0).abs() < f64::EPSILON);
        assert!((analysis.basis_contribution_pct - 0.0).abs() < f64::EPSILON);
    }
}
