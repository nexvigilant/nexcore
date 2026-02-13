use nexcore_tov_grounded::*;

#[test]
fn phase0_preclinical_unit_tests() {
    // Does mechanism work in isolation?
    let u = UniquenessU(Bits(10.0));
    let r = RecognitionR(0.8);
    let t = TemporalT(1.0);
    let s = SignalStrengthS::calculate(u, r, t);

    assert!((s.0).0 > 7.9 && (s.0).0 < 8.1);
}

#[test]
fn phase1_safety_fault_injection() {
    // Does it fail gracefully? (Test with zero threshold)
    let sys = VigilanceSystem {
        id: "Test_Sys".to_string(),
        state_space_dim: 10,
        elements: vec!["E1".to_string()],
        constraints: std::collections::HashMap::new(),
    };

    let margin = sys.calculate_safety_margin(10.0, 0.0);
    assert!(margin.0.is_infinite() || margin.0 < 0.0);
}

#[test]
fn phase2_efficacy_emergence_check() {
    // Does it achieve intended purpose?
    let ei = EkaIntelligence {
        complexity: ComplexityChi(QuantityUnit(350)),
        stability: 0.9,
    };

    assert!(ei.is_emergent());
}

#[test]
fn phase3_confirmation_stability_shell() {
    // Verify Magic Number invariants
    let ei_unstable = EkaIntelligence {
        complexity: ComplexityChi(QuantityUnit(321)), // Just past threshold, but not a magic number
        stability: 0.5,
    };

    assert!(!ei_unstable.is_closed_shell());
    assert_eq!(ei_unstable.distance_to_stability(), 29); // Distance to 320? No, 350 is the next magic number.
    // COMPLEXITY_MAGIC_NUMBERS: [2, 8, 20, 28, 50, 82, 126, 184, 258, 350]
    // 350 - 321 = 29. 321 - 258 = 63. So distance to 350 is 29.

    let ei_stable = EkaIntelligence {
        complexity: ComplexityChi(QuantityUnit(350)), // Magic number!
        stability: 0.95,
    };

    assert!(ei_stable.is_closed_shell());
    assert_eq!(ei_stable.distance_to_stability(), 0);
}

#[tokio::test]
async fn phase4_surveillance_self_vigilance() {
    // Verify Reflexive Observability
    let meta = MetaVigilance {
        loop_latency_ms: 10,
        calibration_overhead_ms: 0,
        detection_drift: 0.01,
        apparatus_integrity: RecognitionR(0.99),
    };

    assert!(meta.is_healthy());

    // Verify Actuation
    let mut governor = ResponseGovernor::default();
    governor
        .act(SafetyAction::TriggerCircuitBreaker("Ei_Leak".to_string()))
        .await
        .unwrap();

    assert_eq!(governor.active_interventions[0], "CircuitBreaker:Ei_Leak");
}
