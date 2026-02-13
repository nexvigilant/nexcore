use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::Verdict;

#[test]
fn test_capability_10_module_tenancy_act() {
    let act = SystemHousingAct::new();
    assert!(act.community_stable);

    // 1. Appraise a Secure Compound
    let measured_residence = act.appraise_residence("nexcore-security", TenancyTier::Secure);
    assert_eq!(measured_residence.value.tier, TenancyTier::Secure);
    assert_eq!(measured_residence.value.floor_space, 1000);
    assert!(measured_residence.confidence.value() > 0.95);

    // 2. Verify Fair Tenancy: High load with guarantee
    let verdict = act.verify_fair_tenancy(&measured_residence.value, 0.9);
    assert_eq!(verdict, Verdict::Permitted);

    // 3. Scenario: Unfair Tenancy (No guarantee)
    let unfair_residence = ModuleResidence {
        crate_id: "marginalized-module".into(),
        tier: TenancyTier::Public,
        floor_space: 50,
        fair_access_guaranteed: false,
    };
    let fail_verdict = act.verify_fair_tenancy(&unfair_residence, 0.85);
    assert_eq!(fail_verdict, Verdict::Rejected);
}
