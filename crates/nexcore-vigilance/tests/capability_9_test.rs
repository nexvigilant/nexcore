use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::{Treasury, Verdict};

#[test]
fn test_capability_9_compute_quota_act() {
    let act = ComputeQuotaAct::new();
    assert!(act.grid_stable);

    let treasury = Treasury {
        compute_quota: 5000,
        memory_quota: 5000,
    };

    // 1. EIA Stability Audit
    let status_measured = act.assess_grid(&treasury);
    assert!(status_measured.value.reserve_margin > 0.5);
    assert!(status_measured.confidence.value() > 0.9);

    // 2. NNSA Authorization: Standard Action
    let standard_cost = Treasury {
        compute_quota: 100,
        memory_quota: 50,
    };
    let verdict_std = act.authorize_surge(&status_measured.value, &standard_cost);
    assert_eq!(verdict_std, Verdict::Permitted);

    // 3. NNSA Authorization: High-Energy Action
    let expensive_cost = Treasury {
        compute_quota: 2000,
        memory_quota: 1000,
    };
    let verdict_exp = act.authorize_surge(&status_measured.value, &expensive_cost);
    assert_eq!(verdict_exp, Verdict::Flagged);

    // 4. Grid Criticality: Brownout Prevention
    let critical_status = GridStatus {
        compute_load: 0.95,
        memory_utilization: 0.90,
        reserve_margin: 0.05,
    };
    let verdict_crit = act.authorize_surge(&critical_status, &standard_cost);
    assert_eq!(verdict_crit, Verdict::Rejected);
}
