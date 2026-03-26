//! Guardian integration — convert PHAROS signals to Guardian ThreatSignals.
//!
//! Primitive composition: μ(Mapping) + →(Causality) + ∂(Boundary)

use nexcore_guardian_engine::sensing::{SignalSource, ThreatLevel, ThreatSignal};
use nexcore_primitives::measurement::Measured;

use crate::pipeline::ActionableSignal;

/// Convert a PHAROS ActionableSignal into a Guardian ThreatSignal.
///
/// Maps threat levels:
/// - Critical (EB05 >= 5.0, 4+ algorithms) → ThreatLevel::Critical
/// - High (EB05 >= 3.0, 3+ algorithms) → ThreatLevel::High
/// - Medium (EB05 >= 2.0, 2+ algorithms) → ThreatLevel::Medium
/// - Low → ThreatLevel::Low
pub fn to_threat_signal(signal: &ActionableSignal) -> ThreatSignal<String> {
    let level = match signal.threat_level() {
        "Critical" => ThreatLevel::Critical,
        "High" => ThreatLevel::High,
        "Medium" => ThreatLevel::Medium,
        _ => ThreatLevel::Low,
    };

    let pattern = format!(
        "PHAROS signal: {} + {} (n={}, PRR={:.2}, EB05={:.2}, algos={})",
        signal.drug,
        signal.event,
        signal.case_count,
        signal.prr,
        signal.eb05,
        signal.algorithms_flagged,
    );

    let source = SignalSource::Pamp {
        source_id: "pharos".to_string(),
        vector: format!(
            "PRR={:.2} ROR_LCI={:.2} IC025={:.2} EB05={:.2} N={}",
            signal.prr, signal.ror_lower_ci, signal.ic025, signal.eb05, signal.case_count
        ),
    };

    let confidence = Measured::certain(calculate_confidence(signal));

    ThreatSignal::new(pattern, level, source)
        .with_confidence(confidence)
        .with_metadata("drug", &signal.drug)
        .with_metadata("event", &signal.event)
        .with_metadata("case_count", signal.case_count.to_string())
        .with_metadata("algorithms_flagged", signal.algorithms_flagged.to_string())
}

/// Derive confidence from the unified boundary sharpness score.
///
/// The ∂-score already encodes algorithm agreement, dimensional excess,
/// and case resolution. Confidence is a saturating map from ∂-score to
/// [0, 1]: a ∂-score of 2.0 (Critical boundary) yields full confidence.
fn calculate_confidence(signal: &ActionableSignal) -> f64 {
    // Confidence saturates at 1.0 when ∂-score reaches the Critical
    // threshold (1.5). If the boundary is blazing, you're fully confident.
    (signal.boundary_score / 1.5).min(1.0).max(0.0)
}

/// Batch convert all actionable signals to threat signals.
pub fn to_threat_signals(signals: &[ActionableSignal]) -> Vec<ThreatSignal<String>> {
    signals.iter().map(to_threat_signal).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signal(eb05: f64, algos: u32, cases: u64) -> ActionableSignal {
        let thresholds = crate::thresholds::SignalThresholds::default();
        // Scale all metrics proportionally to eb05 — the Rosetta encoding
        // says all four algorithms measure the same boundary, so test
        // signals should reflect consistent strength across all angles.
        let ratio = eb05 / thresholds.min_eb05;
        let prr = thresholds.min_prr * ratio;
        let ror_lower_ci = thresholds.min_ror_lower_ci * ratio;
        let ic025 = thresholds.min_ic025 + (ratio - 1.0).max(0.0);
        let boundary_score =
            thresholds.boundary_score(prr, ror_lower_ci, ic025, eb05, cases, algos);
        ActionableSignal {
            drug: "TESTDRUG".to_string(),
            event: "TESTEVENT".to_string(),
            case_count: cases,
            prr,
            prr_lower_ci: prr * 0.7,
            ror: prr * 1.1,
            ror_lower_ci,
            ic: ic025 + 0.5,
            ic025,
            ebgm: eb05 + 1.0,
            eb05,
            algorithms_flagged: algos,
            boundary_score,
        }
    }

    #[test]
    fn test_critical_mapping() {
        let sig = test_signal(5.5, 4, 100);
        let threat = to_threat_signal(&sig);
        assert!(matches!(threat.severity, ThreatLevel::Critical));
    }

    #[test]
    fn test_high_mapping() {
        let sig = test_signal(3.5, 3, 50);
        let threat = to_threat_signal(&sig);
        assert!(matches!(threat.severity, ThreatLevel::High));
    }

    #[test]
    fn test_medium_mapping() {
        let sig = test_signal(2.5, 2, 20);
        let threat = to_threat_signal(&sig);
        assert!(matches!(threat.severity, ThreatLevel::Medium));
    }

    #[test]
    fn test_confidence_from_strong_boundary() {
        // Strong signal: high eb05, all 4 algorithms, many cases
        let sig = test_signal(5.0, 4, 1024);
        let confidence = calculate_confidence(&sig);
        // ∂-score should be well above 2.0 → confidence saturates at 1.0
        assert!(
            confidence > 0.9,
            "Strong boundary should yield high confidence, got {confidence}"
        );
    }

    #[test]
    fn test_confidence_from_weak_boundary() {
        // Weak signal: low eb05, 1 algorithm, few cases
        let sig = test_signal(2.0, 1, 3);
        let confidence = calculate_confidence(&sig);
        // Low ∂-score → low confidence
        assert!(
            confidence < 0.5,
            "Weak boundary should yield low confidence, got {confidence}"
        );
    }

    #[test]
    fn test_batch_conversion() {
        let signals = vec![test_signal(5.0, 4, 100), test_signal(2.5, 2, 10)];
        let threats = to_threat_signals(&signals);
        assert_eq!(threats.len(), 2);
    }
}
