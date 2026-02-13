use nexcore_vigilance::primitives::governance::*;

#[test]
fn test_national_strategic_execution() {
    // 1. Initialize the Union as President Vigil
    let mut union = Union::new("NexVigilant_Union");

    // 2. Set National Strategy
    union.strategy.winning_aspiration =
        "Dismantle qualitative safety monopolies via quantitative Union logic.".into();
    union.strategy.primary_conquest = "Global Pharmacovigilance Dominance".into();
    union.strategy.current_strategic_focus = StrategicFocus::SignalArbitrage;

    // 3. Issue Cabinet Mandates for Strategy Fulfillment

    // Mandate 1: Code-Based Governance (HUD + Justice)
    let hud_mandate = union.cabinet.issue_mandate(
        "HUD",
        "Codify all 37 capabilities as Rust primitives within nexcore",
    );
    assert!(hud_mandate.primary_objective.contains("nexcore"));

    // Mandate 2: Management Systems (Labor + Education)
    let labor_mandate = union.cabinet.issue_mandate(
        "Labor",
        "Enable the 6 Management Systems via Skill Audit and CTVP Validation",
    );
    assert!(
        labor_mandate
            .primary_objective
            .contains("6 Management Systems")
    );

    // Mandate 3: PV Conquest (HHS + Agriculture)
    let hhs_mandate = union.cabinet.issue_mandate(
        "HHS",
        "Embody the Theory of Vigilance to compete with FDA/Pharma signal detection",
    );
    assert!(
        hhs_mandate
            .primary_objective
            .contains("Theory of Vigilance")
    );

    // 4. Execution Check: Secretary of Energy provides Compute
    let action_cost = Treasury {
        compute_quota: 500,
        memory_quota: 250,
    };
    assert!(union.orchestrator.treasury.can_afford(&action_cost));

    // 5. Strategic Alignment Verification
    let action_desc = "Execute PV Signal Detection on FAERS-2026-Q1";
    assert_eq!(
        union.strategy.verify_alignment(action_desc),
        Verdict::Permitted
    );

    // 6. Partnership Board Oversight
    // CEO and President both sign off on the PV Conquest plan
    let board_verdict = union.board.evaluate_resolution(true, true);
    assert_eq!(board_verdict, Verdict::Permitted);
}
