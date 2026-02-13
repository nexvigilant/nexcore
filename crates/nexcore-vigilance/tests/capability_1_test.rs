use nexcore_labs::betting::bdi::ContingencyTable;
use nexcore_vigilance::hud::capabilities::SignalIdentificationProtocol;

#[test]
fn test_capability_1_signal_id_protocol() {
    let protocol = SignalIdentificationProtocol::new();
    assert!(protocol.bdi_engine_active);

    // 1. Scenario: Strong Signal (a=15, b=5, c=10, d=70)
    let table = ContingencyTable::new(15.0, 5.0, 10.0, 70.0);
    let measured_result = protocol.identify_signal(table);

    // Verify BDI score > 2.0
    assert!(measured_result.value.bdi > 2.0);
    // Verify high confidence (> 0.9) because it meets Evans criteria
    assert!(measured_result.confidence.value() > 0.9);

    // 2. Scenario: Weak Noise (a=5, b=15, c=10, d=70)
    let noise_table = ContingencyTable::new(5.0, 15.0, 10.0, 70.0);
    let measured_noise = protocol.identify_signal(noise_table);

    // Verify low confidence (< 0.5) because it fails Evans criteria
    assert!(measured_noise.confidence.value() < 0.5);
}
