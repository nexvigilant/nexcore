//! Unit tests for PV Signal Detection
//!
//! Phase 0 Preclinical: Signal metric isolation tests

use borrow_miner::game::{PRR, ROR, IC, EB05, CaseCount, SignalStrength, SignalResult, DrugEvent};

#[test]
fn prr_below_threshold_is_not_signal() {
    assert!(!PRR(1.5).is_signal());
    assert!(!PRR(1.9).is_signal());
}

#[test]
fn prr_at_threshold_is_signal() {
    assert!(PRR(2.0).is_signal());
    assert!(PRR(2.5).is_signal());
}

#[test]
fn prr_strength_classification() {
    assert_eq!(PRR(1.0).strength(), SignalStrength::None);
    assert_eq!(PRR(1.6).strength(), SignalStrength::Weak);
    assert_eq!(PRR(3.0).strength(), SignalStrength::Moderate);
    assert_eq!(PRR(6.0).strength(), SignalStrength::Strong);
}

#[test]
fn eb05_threshold_works() {
    assert!(!EB05(1.5).is_signal());
    assert!(EB05(2.0).is_signal());
    assert!(EB05(5.0).is_signal());
}

#[test]
fn case_count_minimum_required() {
    assert!(!CaseCount(0).sufficient());
    assert!(!CaseCount(2).sufficient());
    assert!(CaseCount(3).sufficient());
    assert!(CaseCount(100).sufficient());
}

#[test]
fn signal_strength_points_ordering() {
    let none = SignalStrength::None.points();
    let weak = SignalStrength::Weak.points();
    let moderate = SignalStrength::Moderate.points();
    let strong = SignalStrength::Strong.points();

    assert!(none < weak);
    assert!(weak < moderate);
    assert!(moderate < strong);
}

#[test]
fn signal_strength_has_symbols() {
    assert!(!SignalStrength::None.symbol().is_empty());
    assert!(!SignalStrength::Weak.symbol().is_empty());
    assert!(!SignalStrength::Moderate.symbol().is_empty());
    assert!(!SignalStrength::Strong.symbol().is_empty());
}

#[test]
fn signal_strength_has_colors() {
    for strength in [SignalStrength::None, SignalStrength::Weak, SignalStrength::Moderate, SignalStrength::Strong] {
        let color = strength.color();
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }
}

#[test]
fn signal_result_overall_strength_requires_cases() {
    let signal = SignalResult {
        prr: PRR(5.0),
        ror: ROR(5.0),
        ic: IC(2.0),
        eb05: EB05(4.0),
        case_count: CaseCount(2), // Below minimum!
        chi_square: 100.0,
    };

    assert_eq!(signal.overall_strength(), SignalStrength::None);
}

#[test]
fn signal_result_strong_when_all_criteria_met() {
    let signal = SignalResult {
        prr: PRR(6.0),
        ror: ROR(6.0),
        ic: IC(2.5),
        eb05: EB05(5.0),
        case_count: CaseCount(100),
        chi_square: 50.0,
    };

    assert_eq!(signal.overall_strength(), SignalStrength::Strong);
}

#[test]
fn drug_event_creation() {
    let de = DrugEvent::new("Aspirin", "Bleeding", 500);
    assert_eq!(de.drug_name, "Aspirin");
    assert_eq!(de.event_term, "Bleeding");
    assert_eq!(de.case_count.0, 500);
    assert!(de.signal.is_none());
}

#[test]
fn drug_event_with_signal() {
    let signal = SignalResult {
        prr: PRR(3.0),
        ror: ROR(3.0),
        ic: IC(1.5),
        eb05: EB05(2.5),
        case_count: CaseCount(100),
        chi_square: 25.0,
    };

    let de = DrugEvent::new("Test", "Event", 100).with_signal(signal);
    assert!(de.signal.is_some());
    assert_eq!(de.strength(), SignalStrength::Moderate);
}

#[test]
fn drug_event_display_name_format() {
    let de = DrugEvent::new("Warfarin", "Hemorrhage", 1000);
    assert_eq!(de.display_name(), "Warfarin→Hemorrhage");
}

#[test]
fn prr_display_format() {
    let prr = PRR(3.14159);
    let display = format!("{}", prr);
    assert!(display.contains("3.14"));
}

#[test]
fn sample_drug_events_not_empty() {
    let samples = borrow_miner::game::sample_drug_events();
    assert!(!samples.is_empty());
    assert!(samples.len() >= 3);
}
