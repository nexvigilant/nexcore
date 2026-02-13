use nexcore_vigilance::algorithmovigilance::scoring::{
    AcaCausalityCategory, AcaLemma, AcaScoringInput, GroundTruthStandard, LemmaResponse,
};
use nexcore_vigilance::hud::capabilities::CausalAttributionEngine;

#[test]
fn test_capability_3_causal_attribution() {
    let engine = CausalAttributionEngine::new();
    assert!(engine.aca_logic_active);

    // 1. Scenario: Definite Causality
    let input_definite = AcaScoringInput::new()
        .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Cognition, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Harm, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Rechallenge, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Validation, LemmaResponse::Yes)
        .with_ground_truth_standard(GroundTruthStandard::Gold);

    let measured_definite = engine.attribute_causality(&input_definite);
    assert_eq!(
        measured_definite.value.category,
        AcaCausalityCategory::Definite
    );
    assert!(measured_definite.confidence.value() > 0.9);

    // 2. Scenario: Unassessable (Missing required lemma L1)
    let input_unassessable = AcaScoringInput::new()
        .with_lemma(AcaLemma::Temporal, LemmaResponse::No)
        .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
        .with_lemma(AcaLemma::Harm, LemmaResponse::Yes);

    let measured_unassessable = engine.attribute_causality(&input_unassessable);
    assert_eq!(
        measured_unassessable.value.category,
        AcaCausalityCategory::Unassessable
    );
    assert!(measured_unassessable.confidence.value() < 0.2);
}
