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

/// Calculate confidence based on signal strength indicators.
///
/// Higher case count + more algorithms agreeing = higher confidence.
fn calculate_confidence(signal: &ActionableSignal) -> f64 {
    // Base confidence from algorithm agreement (0.25 per algorithm)
    let algo_confidence = (signal.algorithms_flagged as f64) * 0.25;

    // Case count factor: log2(n) / 10, capped at 0.3
    let case_factor = (signal.case_count as f64).log2() / 10.0;
    let case_confidence = case_factor.min(0.3);

    // Total confidence capped at 1.0
    (algo_confidence + case_confidence).min(1.0)
}

/// Batch convert all actionable signals to threat signals.
pub fn to_threat_signals(signals: &[ActionableSignal]) -> Vec<ThreatSignal<String>> {
    signals.iter().map(to_threat_signal).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signal(eb05: f64, algos: u32, cases: u64) -> ActionableSignal {
        ActionableSignal {
            drug: "TESTDRUG".to_string(),
            event: "TESTEVENT".to_string(),
            case_count: cases,
            prr: 3.0,
            prr_lower_ci: 2.0,
            ror: 3.5,
            ror_lower_ci: 2.1,
            ic: 1.5,
            ic025: 0.5,
            ebgm: eb05 + 1.0,
            eb05,
            algorithms_flagged: algos,
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
    fn test_confidence_calculation() {
        let sig = test_signal(5.0, 4, 1024);
        let confidence = calculate_confidence(&sig);
        // 4 algos * 0.25 = 1.0, case factor = log2(1024)/10 = 1.0 → capped at 1.0
        assert!((confidence - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_low_confidence() {
        let sig = test_signal(2.0, 1, 3);
        let confidence = calculate_confidence(&sig);
        // 1 algo * 0.25 = 0.25, case factor = log2(3)/10 ≈ 0.158
        assert!(confidence > 0.3);
        assert!(confidence < 0.5);
    }

    #[test]
    fn test_batch_conversion() {
        let signals = vec![test_signal(5.0, 4, 100), test_signal(2.5, 2, 10)];
        let threats = to_threat_signals(&signals);
        assert_eq!(threats.len(), 2);
    }
}
