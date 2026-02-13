use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::Verdict;

#[test]
fn test_capability_8_agricultural_data_act() {
    let act = AgriculturalDataAct::new();
    assert!(act.harvest_active);

    // 1. Scenario: Healthy Harvest (High Purity)
    let measured_harvest = act.execute_harvest(1000, 0.95);
    assert_eq!(measured_harvest.value.quantity, 950);
    assert_eq!(
        act.inspect_yield(&measured_harvest.value),
        Verdict::Permitted
    );

    // 2. Scenario: Corrupted Harvest (Low Purity)
    let bad_harvest = act.execute_harvest(1000, 0.5);
    assert_eq!(bad_harvest.value.quantity, 500);
    assert_eq!(act.inspect_yield(&bad_harvest.value), Verdict::Rejected);
}
