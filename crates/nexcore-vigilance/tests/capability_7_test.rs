use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::{SovereignDomain, Verdict};

#[test]
fn test_capability_7_data_sovereignty_act() {
    let act = DataSovereigntyAct::new();
    assert!(act.conservation_active);

    let domain = SovereignDomain {
        id: "PV_Domain".into(),
        primary_axiom: "Detect".into(),
        laws: vec![],
        population_size: 100,
    };

    // 1. Grant a lease to a National Park (Immutable Module)
    let measured_lease = act.grant_lease(&domain, ResourceType::ImmutableModule, "AGENT_007");
    assert_eq!(
        measured_lease.value.resource_type,
        ResourceType::ImmutableModule
    );

    // 2. Verify integrity: Deny WRITE access to Immutable Module
    let verdict = act.verify_resource_integrity(&measured_lease.value, "WRITE");
    assert_eq!(verdict, Verdict::Rejected);

    // 3. Verify integrity: Permit READ access (implied)
    let permit_verdict = act.verify_resource_integrity(&measured_lease.value, "READ");
    assert_eq!(permit_verdict, Verdict::Permitted);
}
