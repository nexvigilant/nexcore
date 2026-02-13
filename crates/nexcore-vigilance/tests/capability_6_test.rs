use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::measurement::{Confidence, Measured};

#[test]
fn test_capability_6_signal_relay_act() {
    let relay = SovereignSignalRelay::new();
    assert!(relay.pathways_active);

    // 1. Scenario: High-Confidence FAA Relay
    let signal = Measured::uncertain("Safe_Signal", Confidence::new(0.95));
    let manifest_measured = relay.relay_signal(
        &signal,
        "HHS",
        "Transportation",
        RelayMode::FastPathAviation,
    );

    assert_eq!(manifest_measured.value.mode, RelayMode::FastPathAviation);
    // Confidence should remain high
    assert!(manifest_measured.confidence.value() > 0.9);

    // 2. Scenario: Low-Confidence FAA Relay (Safety Violation)
    let low_conf_signal = Measured::uncertain("Risky_Signal", Confidence::new(0.6));
    let viol_manifest = relay.relay_signal(
        &low_conf_signal,
        "HHS",
        "Transportation",
        RelayMode::FastPathAviation,
    );

    // Confidence should be adjusted down by NHTSA safety logic
    assert!(viol_manifest.confidence.value() < 0.6);
}
