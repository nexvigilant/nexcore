//! Endocrine → Output Style Pipeline Test
//!
//! Demonstrates the full biology→behavior loop:
//! Stimulus → Sense → Classify → Infer → Experiment → Normalize → Codify → Extend → ToneProfile

use stem::bio::endocrine::{BehaviorModulation, EndocrineSystem, StimulusCategory, ToneProfile};
use stem::core::{Classify, Codify, Confidence, Extend, Infer, Sense};

use nexcore_hormones::Stimulus;

#[test]
fn full_endocrine_pipeline_stress() {
    let system = EndocrineSystem::new();

    // 1. SENSE: Detect error stimulus
    let stimulus = Stimulus::ErrorEncountered { severity: 0.8 };
    let signal = system.sense(&stimulus);
    assert_eq!(signal.stimulus, StimulusCategory::Stress);
    assert!((signal.intensity - 0.8).abs() < f64::EPSILON);

    // 2. CLASSIFY: Categorize as stress
    let category = system.classify(&signal);
    assert_eq!(category, StimulusCategory::Stress);

    // 3. INFER: Predict cortisol + adrenaline spike
    let predictions = system.infer(&category, &signal.intensity);
    assert!(predictions.len() >= 2);

    // 4. CODIFY: Export to behavioral modifiers
    let modulation = system.codify(&nexcore_hormones::EndocrineState::default());

    // 5. EXTEND: Map to output style tone profile
    let tone = system.extend(&modulation);
    validate_tone_profile(&tone);
}

#[test]
fn full_endocrine_pipeline_reward() {
    let system = EndocrineSystem::new();

    let stimulus = Stimulus::TaskCompleted { complexity: 0.9 };
    let signal = system.sense(&stimulus);
    assert_eq!(signal.stimulus, StimulusCategory::Reward);

    let category = system.classify(&signal);
    let predictions = system.infer(&category, &signal.intensity);
    assert!(
        predictions
            .iter()
            .any(|(h, _)| *h == nexcore_hormones::HormoneType::Dopamine)
    );

    let modulation = system.codify(&nexcore_hormones::EndocrineState::default());
    let tone = system.extend(&modulation);
    validate_tone_profile(&tone);
}

#[test]
fn full_endocrine_pipeline_social() {
    let system = EndocrineSystem::new();

    let stimulus = Stimulus::MutualSuccess { shared_win: true };
    let signal = system.sense(&stimulus);
    assert_eq!(signal.stimulus, StimulusCategory::Social);

    let predictions = system.infer(&signal.stimulus, &signal.intensity);
    assert!(
        predictions
            .iter()
            .any(|(h, _)| *h == nexcore_hormones::HormoneType::Oxytocin)
    );
}

#[test]
fn tone_profile_responds_to_modulation_extremes() {
    let system = EndocrineSystem::new();

    // High stress → conservative, precise, less direct
    let stressed = BehaviorModulation {
        risk_tolerance: 0.2,
        validation_depth: 0.95,
        exploration_rate: 0.1,
        warmth: 0.4,
        urgency: 0.8,
    };
    let stressed_tone = system.extend(&stressed);
    assert!(stressed_tone.hedging > 0.2); // More hedging under stress
    assert!(stressed_tone.precision > 0.9); // High validation → high precision
    assert!(stressed_tone.verbosity < 0.5); // Urgency → less verbose

    // High reward → bold, warm, exploratory
    let rewarded = BehaviorModulation {
        risk_tolerance: 0.9,
        validation_depth: 0.5,
        exploration_rate: 0.9,
        warmth: 0.9,
        urgency: 0.1,
    };
    let rewarded_tone = system.extend(&rewarded);
    assert!(rewarded_tone.directness > 0.7); // Bold → direct
    assert!(rewarded_tone.warmth > 0.8); // Social reward → warm
    assert!(rewarded_tone.verbosity > 0.5); // Low urgency → more verbose
}

#[test]
fn confidence_propagates_through_pipeline() {
    let system = EndocrineSystem::new();
    let stimulus = Stimulus::PositiveFeedback { intensity: 0.7 };
    let signal = system.sense(&stimulus);

    // Signal carries confidence from sensing
    assert!((signal.confidence.value() - 0.9).abs() < f64::EPSILON);

    // Confidence would combine multiplicatively if we chain Measured<T>
    let combined = signal.confidence.combine(Confidence::new(0.85));
    assert!(combined.value() < signal.confidence.value());
}

/// Validate tone profile values are in [0, 1] range
fn validate_tone_profile(tone: &ToneProfile) {
    assert!(
        tone.directness >= 0.0 && tone.directness <= 1.0,
        "directness out of range: {}",
        tone.directness
    );
    assert!(
        tone.hedging >= 0.0 && tone.hedging <= 1.0,
        "hedging out of range: {}",
        tone.hedging
    );
    assert!(
        tone.warmth >= 0.0 && tone.warmth <= 1.0,
        "warmth out of range: {}",
        tone.warmth
    );
    assert!(
        tone.precision >= 0.0 && tone.precision <= 1.0,
        "precision out of range: {}",
        tone.precision
    );
    assert!(
        tone.verbosity >= 0.0 && tone.verbosity <= 1.0,
        "verbosity out of range: {}",
        tone.verbosity
    );
}
