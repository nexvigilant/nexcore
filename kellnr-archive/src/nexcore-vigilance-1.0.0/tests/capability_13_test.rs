use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::Verdict;

#[test]
fn test_capability_13_national_security_act() {
    let act = NationalSecurityAct::new();
    assert!(act.defense_active);

    // 1. Normal State (DARPA Analysis)
    let normal_measured = act.assess_threat(0.1, true);
    assert_eq!(normal_measured.value.level, ThreatLevel::Low);
    assert_eq!(
        act.authorize_defense(&normal_measured.value),
        Verdict::Permitted
    );

    // 2. Anomaly Detected (Guarded)
    let guarded_measured = act.assess_threat(0.5, true);
    assert_eq!(guarded_measured.value.level, ThreatLevel::Guarded);
    assert_eq!(
        act.authorize_defense(&guarded_measured.value),
        Verdict::Permitted
    );

    // 3. Adversarial Attempt (Elevated)
    let elevated_measured = act.assess_threat(0.8, true);
    assert_eq!(elevated_measured.value.level, ThreatLevel::Elevated);
    assert!(elevated_measured.value.isolation_active);
    assert_eq!(
        act.authorize_defense(&elevated_measured.value),
        Verdict::Flagged
    );

    // 4. Critical Breach (High)
    let high_measured = act.assess_threat(0.95, false); // Grounding failed!
    assert_eq!(high_measured.value.level, ThreatLevel::High);
    assert!(high_measured.value.isolation_active);
    assert_eq!(
        act.authorize_defense(&high_measured.value),
        Verdict::Rejected
    );
    // Confidence should be low due to failed grounding proof
    assert!(high_measured.confidence.value() < 0.6);
}
