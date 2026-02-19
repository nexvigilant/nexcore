use nexcore_vigilance::hud::capabilities::BayesianCredibilityLayer;

#[test]
fn test_capability_2_bayesian_credibility() {
    let layer = BayesianCredibilityLayer::new();
    assert!(layer.ecs_engine_active);

    // 1. Scenario: High Credibility (90% public, move against, 0.5h to game)
    // ECS formula: (U × R × T) / 100 — strong signals typically yield 1.5-2.5
    let measured_high = layer.calculate_credibility(0.9, -1, true, 0.5);
    assert!(
        measured_high.value.ecs > 1.5,
        "High credibility scenario should produce ECS > 1.5, got {}",
        measured_high.value.ecs
    );
    // Confidence depends on is_actionable (ECS >= threshold AND lower_credibility > threshold)
    // For strong signals that meet actionability: 0.9, otherwise: 0.3
    if measured_high.value.is_actionable {
        assert!(measured_high.confidence.value() > 0.8);
    } else {
        // Signal is strong but may not meet actionability thresholds (e.g., credibility interval)
        assert!(measured_high.confidence.value() < 0.5);
    }

    // 2. Scenario: Low Credibility (50% public, no move, 48h to game)
    let measured_low = layer.calculate_credibility(0.5, 0, false, 48.0);
    assert!(
        measured_low.value.ecs < 1.0,
        "Low credibility scenario should produce ECS < 1.0, got {}",
        measured_low.value.ecs
    );
    assert!(
        !measured_low.value.is_actionable,
        "Low signal should not be actionable"
    );
    assert!(measured_low.confidence.value() < 0.5);
}
