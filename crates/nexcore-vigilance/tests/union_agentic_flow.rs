use nexcore_vigilance::primitives::governance::agents::legislative::Legislator;
use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::Confidence;

#[tokio::test]
async fn test_union_agentic_pipeline() {
    // 1. Create Union
    let mut union = Union::new("NexVigilant_Test_Union");

    // 2. Appoint Agents
    union.appoint_legislator(Legislator {
        id: "Rep_1".into(),
        weight: VoteWeight::new(10),
        focus_domain: None,
    });

    union.appoint_legislator(Legislator {
        id: "Rep_2".into(),
        weight: VoteWeight::new(10),
        focus_domain: None,
    });

    // 3. Propose Resolution
    let resolution = Resolution::uncertain(Rule, Confidence::new(0.85));

    // 4. Process Resolution (Requires quorum and executive review)
    // Note: Jurist list is empty, so judicial review passes by default in this simulation logic
    let verdict = union
        .process_resolution(resolution)
        .await
        .expect("Union process failed");

    // Legislative deliberation: 0.85 * 0.9 = 0.765 (> 0.5 quorum)
    // Executive review: 0.85 (> 0.7 threshold)
    assert_eq!(verdict, Verdict::Permitted);
    assert_eq!(union.orchestrator.current_cycle, 1);
}
