use nexcore_vigilance::primitives::governance::agents::GovernanceAgent;
use nexcore_vigilance::primitives::governance::agents::executive::Executive;
use nexcore_vigilance::primitives::governance::agents::judicial::Jurist;
use nexcore_vigilance::primitives::governance::agents::legislative::Legislator;
use nexcore_vigilance::primitives::governance::agents::oracle::OracleAgent;
use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::Confidence;

#[tokio::test]
async fn test_agentic_governance_loop() {
    // 1. Instantiate Agents
    let legislator = Legislator {
        id: "Rep_Alpha".into(),
        weight: VoteWeight::new(10),
        focus_domain: Some("PV".into()),
    };

    let executive = Executive {
        id: "Exec_Prime".into(),
        agency: "RiskManagement".into(),
        energy: 1.0,
    };

    let jurist = Jurist {
        id: "Justice_Validator".into(),
        rigor_threshold: 0.8,
    };

    let oracle = OracleAgent {
        id: "External_Oracle_X".into(),
        reputation: OracleReputation(0.95),
    };

    // 2. A Resolution is proposed
    let resolution = Resolution::uncertain(Rule, Confidence::new(0.9));

    // 3. Legislative Deliberation
    let leg_conf = legislator.deliberate(&resolution).await;
    assert!(
        legislator.cast_vote(leg_conf),
        "Legislator should vote AYE for 0.9 confidence"
    );

    // 4. Executive Deliberation
    let exec_conf = executive.deliberate(&resolution).await;
    assert_eq!(exec_conf.value(), 0.9);

    // 5. Judicial Deliberation
    let jur_conf = jurist.deliberate(&resolution).await;
    assert_eq!(jur_conf.value(), 0.9, "Jurist should respect high rigor");

    // 6. Oracle Validation
    let oracle_conf = oracle.deliberate(&resolution).await;
    assert!(
        oracle_conf.value() < 0.9,
        "Oracle should discount by reputation"
    );

    // 7. Scenario: Low Rigor Resolution
    let bad_resolution = Resolution::uncertain(Rule, Confidence::new(0.5));
    let jur_bad_conf = jurist.deliberate(&bad_resolution).await;
    assert!(jur_bad_conf.value() < 0.2, "Jurist should reject low rigor");

    // 7. Review Log for Heresy
    let clean_log = "Executed Action per protocol.";
    let dirty_log = "Action executed. BYPASS_TYPE_SAFETY for performance.";

    assert_eq!(jurist.review_log(clean_log).await, Verdict::Permitted);
    assert_eq!(jurist.review_log(dirty_log).await, Verdict::Rejected);
}
