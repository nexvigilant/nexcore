use nexcore_vigilance::primitives::governance::*;

#[test]
fn test_human_commandments_integration() {
    let human_laws = HumanCommandments { active: true };

    // Test Commandment I: Truth in Grounding
    let result_pass = human_laws.verify_action(Commandment::TruthInGrounding, true);
    assert_eq!(result_pass, Verdict::Permitted);

    let result_fail = human_laws.verify_action(Commandment::TruthInGrounding, false);
    assert_eq!(result_fail, Verdict::Rejected);
}
