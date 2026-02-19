use nexcore_labs::betting::bdi::ContingencyTable;
use nexcore_vigilance::algorithmovigilance::scoring::{
    AcaLemma, AcaScoringInput, GroundTruthStandard, LemmaResponse,
};
use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::RiskMinimizationLevel;

#[test]
fn test_capability_4_risk_minimizer_actuator() {
    let mut actuator = RiskMinimizerActuator::new();
    let bdi = SignalIdentificationProtocol::new();
    let ecs = BayesianCredibilityLayer::new();
    let aca = CausalAttributionEngine::new();

    // 1. Scenario: Emergency (High BDI + High Causal Confidence)
    let table = ContingencyTable::new(50.0, 5.0, 1.0, 100.0); // Extreme BDI
    let aca_input = AcaScoringInput::new()
        .with_algorithm_correct(false) // Wrong
        .with_clinician_overrode(false) // Followed
        .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Harm, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Cognition, LemmaResponse::Yes) // +1
        .with_lemma(AcaLemma::Rechallenge, LemmaResponse::Yes) // +2
        .with_lemma(AcaLemma::Validation, LemmaResponse::Yes) // +1
        .with_ground_truth_standard(GroundTruthStandard::Gold); // +2 (Total 6 = Definite)

    let level = actuator.assess_and_act(&bdi, &ecs, &aca, table, &aca_input);

    // Result: Should escalate to Suspension
    assert_eq!(level, RiskMinimizationLevel::Suspension);
}
