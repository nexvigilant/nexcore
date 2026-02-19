use nexcore_vigilance::primitives::governance::*;

#[test]
fn test_partnership_charter_verification() {
    // 1. Setup the Board
    let board = PartnershipBoard::new();
    assert_eq!(board.ceo_weight.value(), 50);
    assert_eq!(board.president_weight.value(), 50);

    // 2. Scenario: Unanimous Consent (Both agree)
    let verdict_agree = board.evaluate_resolution(true, true);
    assert_eq!(verdict_agree, Verdict::Permitted);

    // 3. Scenario: Dissent (CEO disagrees)
    let verdict_dissent = board.evaluate_resolution(false, true);
    assert_eq!(verdict_dissent, Verdict::Rejected);

    // 4. Setup Treasury
    let initial_funds = Treasury {
        compute_quota: 1000,
        memory_quota: 1000,
    };
    let treasury = DualSignatureTreasury::new(initial_funds.clone());
    assert_eq!(treasury.ceo_allocation, Share(50));
    assert_eq!(treasury.president_allocation, Share(50));

    // 5. Dissolution Protocol
    let mut exit_strategy = DissolutionProtocol {
        activated: false,
        safe_state_captured: false,
    };
    exit_strategy.invoke();
    assert!(exit_strategy.activated);
    assert!(exit_strategy.safe_state_captured);
}
