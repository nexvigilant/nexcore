use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::Verdict;

#[test]
fn test_capability_5_ferrostack_bridge() {
    let mut bridge = FerrostackBridge::new();
    assert!(!bridge.rd_team_deployed);

    // 1. Dispatch R&D Team
    let result = bridge.dispatch_rd_team();
    assert_eq!(result, Verdict::Permitted);
    assert!(bridge.rd_team_deployed);

    // 2. Naturalize a pattern (Signal)
    let measured_pattern = bridge.naturalize_pattern(WebPattern::Signal, 0.95);
    assert!(matches!(measured_pattern.value, WebPattern::Signal));
    assert!(measured_pattern.confidence.value() > 0.9);
    assert_eq!(bridge.active_patterns.len(), 1);

    // 3. Generate Bridge Component
    let component_code = bridge.bridge_component("GovernanceDashboard");
    assert!(component_code.contains("#[component]"));
    assert!(component_code.contains("GovernanceDashboard"));
}
