//! Endocrine system implementing SCIENCE traits from stem-core.
//!
//! Maps biological hormone signaling to behavioral modulation.
//!
//! ## SCIENCE Trait Implementations
//!
//! | Trait | Endocrine Mapping |
//! |-------|-------------------|
//! | `Sense` | Detect stimuli (errors, completions, social) |
//! | `Classify` | Categorize stimulus type |
//! | `Infer` | Predict behavioral impact |
//! | `Experiment` | Apply stimulus, observe state change |
//! | `Normalize` | Update hormone levels (decay toward baseline) |
//! | `Codify` | Export to BehavioralModifiers |
//! | `Extend` | Apply modifiers to AI behavior parameters |

use crate::core::{
    Classify, Codify, Confidence, Experiment, Extend, Infer, Measured, Normalize, Sense,
};
use nexcore_hormones::{BehavioralModifiers, EndocrineState, HormoneType, Stimulus};
use serde::{Deserialize, Serialize};

/// Signal detected from environment (T2-P grounded in T1 Mapping μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormoneSignal {
    /// The stimulus that triggered this signal
    pub stimulus: StimulusCategory,
    /// Intensity of the signal (0.0-1.0)
    pub intensity: f64,
    /// Confidence in signal detection
    pub confidence: Confidence,
}

/// Stimulus categories (T2-P classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StimulusCategory {
    /// Error or failure occurred
    Stress,
    /// Task completed successfully
    Reward,
    /// Social interaction detected
    Social,
    /// Time passage (decay trigger)
    Temporal,
    /// High-urgency situation
    Urgency,
}

/// Behavioral modulation output (T2-C grounded in T1 Mapping μ)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BehaviorModulation {
    /// Risk tolerance (0.0 = conservative, 1.0 = aggressive)
    pub risk_tolerance: f64,
    /// Validation depth (0.0 = minimal, 1.0 = exhaustive)
    pub validation_depth: f64,
    /// Exploration rate (0.0 = exploit, 1.0 = explore)
    pub exploration_rate: f64,
    /// Response warmth (0.0 = clinical, 1.0 = warm)
    pub warmth: f64,
    /// Urgency level (0.0 = relaxed, 1.0 = urgent)
    pub urgency: f64,
}

impl From<BehavioralModifiers> for BehaviorModulation {
    fn from(m: BehavioralModifiers) -> Self {
        Self {
            risk_tolerance: m.risk_tolerance,
            validation_depth: m.validation_depth,
            exploration_rate: m.exploration_rate,
            warmth: (m.risk_tolerance + 0.5).min(1.0), // Derived
            urgency: 1.0 - m.validation_depth,         // Inverse relationship
        }
    }
}

/// The endocrine system implementing all SCIENCE traits
#[derive(Debug, Clone)]
pub struct EndocrineSystem {
    state: EndocrineState,
}

impl Default for EndocrineSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl EndocrineSystem {
    /// Create a new endocrine system at baseline
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: EndocrineState::default(),
        }
    }

    /// Load state from persistent storage
    #[must_use]
    pub fn load() -> Self {
        Self {
            state: EndocrineState::load(),
        }
    }

    /// Get current hormone level
    #[must_use]
    pub fn get_hormone(&self, hormone: HormoneType) -> f64 {
        self.state.get(hormone).value()
    }

    /// Save state to persistent storage
    pub fn save(&self) -> Result<(), nexcore_hormones::EndocrineError> {
        self.state.save()
    }
}

// ============================================================================
// SCIENCE Trait Implementations
// ============================================================================

/// SENSE: Detect stimuli from environment
impl Sense for EndocrineSystem {
    type Environment = Stimulus;
    type Signal = HormoneSignal;

    fn sense(&self, env: &Self::Environment) -> Self::Signal {
        let (category, intensity) = match env {
            // Stress stimuli
            Stimulus::ErrorEncountered { severity } => (StimulusCategory::Stress, *severity),
            Stimulus::DeadlinePressure { urgency } => (StimulusCategory::Urgency, *urgency),
            Stimulus::UncertaintyDetected { confidence_gap } => {
                (StimulusCategory::Stress, *confidence_gap)
            }
            // Reward stimuli
            Stimulus::TaskCompleted { complexity } => (StimulusCategory::Reward, *complexity),
            Stimulus::PositiveFeedback { intensity } => (StimulusCategory::Reward, *intensity),
            Stimulus::PatternSuccess { reuse_count } => (
                StimulusCategory::Reward,
                (*reuse_count as f64 / 10.0).min(1.0),
            ),
            // Stability stimuli
            Stimulus::ConsistentSession { variance } => {
                (StimulusCategory::Temporal, 1.0 - *variance)
            }
            Stimulus::PredictableOutcome { accuracy } => (StimulusCategory::Reward, *accuracy),
            // Crisis stimuli
            Stimulus::CriticalError { recoverable } => (
                StimulusCategory::Urgency,
                if *recoverable { 0.6 } else { 1.0 },
            ),
            Stimulus::TimeConstraint { remaining_pct } => {
                (StimulusCategory::Urgency, 1.0 - *remaining_pct)
            }
            Stimulus::HighStakesDecision { impact } => (StimulusCategory::Urgency, *impact),
            // Social stimuli
            Stimulus::PartnershipReinforced { signal } => (StimulusCategory::Social, *signal),
            Stimulus::MutualSuccess { shared_win } => (
                StimulusCategory::Social,
                if *shared_win { 1.0 } else { 0.5 },
            ),
            Stimulus::TransparentCommunication { clarity } => (StimulusCategory::Social, *clarity),
            // Temporal stimuli
            Stimulus::SessionDuration { minutes } => (
                StimulusCategory::Temporal,
                (*minutes as f64 / 120.0).min(1.0),
            ),
            Stimulus::ContextUtilization { pct } => (StimulusCategory::Temporal, *pct),
            Stimulus::CompletionSignal { tasks_done } => (
                StimulusCategory::Temporal,
                (*tasks_done as f64 / 10.0).min(1.0),
            ),
            Stimulus::PlanetaryAlignment { distance_au, .. } => {
                // Normalize AU: 0.37 (close) to 2.67 (far)
                let proximity = (2.67 - distance_au.clamp(0.37, 2.67)) / (2.67 - 0.37);
                (StimulusCategory::Temporal, proximity)
            }
        };

        HormoneSignal {
            stimulus: category,
            intensity,
            confidence: Confidence::new(0.9), // High confidence in stimulus detection
        }
    }
}

/// CLASSIFY: Categorize signal by stimulus type
impl Classify for EndocrineSystem {
    type Signal = HormoneSignal;
    type Category = StimulusCategory;

    fn classify(&self, signal: &Self::Signal) -> Self::Category {
        signal.stimulus
    }
}

/// INFER: Predict which hormones will be affected
impl Infer for EndocrineSystem {
    type Pattern = StimulusCategory;
    type Data = f64; // intensity
    type Prediction = Vec<(HormoneType, f64)>; // hormone → delta

    fn infer(&self, pattern: &Self::Pattern, data: &Self::Data) -> Self::Prediction {
        let intensity = *data;
        match pattern {
            StimulusCategory::Stress => vec![
                (HormoneType::Cortisol, intensity * 0.3),
                (HormoneType::Adrenaline, intensity * 0.2),
            ],
            StimulusCategory::Reward => vec![
                (HormoneType::Dopamine, intensity * 0.2),
                (HormoneType::Serotonin, intensity * 0.1),
            ],
            StimulusCategory::Social => vec![(HormoneType::Oxytocin, intensity * 0.25)],
            StimulusCategory::Temporal => vec![(HormoneType::Melatonin, intensity * 0.15)],
            StimulusCategory::Urgency => vec![
                (HormoneType::Adrenaline, intensity * 0.4),
                (HormoneType::Cortisol, intensity * 0.2),
            ],
        }
    }
}

/// EXPERIMENT: Apply stimulus, observe state change
impl Experiment for EndocrineSystem {
    type Action = Stimulus;
    type Outcome = Measured<EndocrineState>;

    fn experiment(&mut self, action: Self::Action) -> Self::Outcome {
        // Apply the stimulus
        action.apply(&mut self.state);

        Measured::new(
            self.state.clone(),
            Confidence::new(0.95), // High confidence in state measurement
        )
    }
}

/// NORMALIZE: Decay toward baseline (Bayesian belief update analog)
impl Normalize for EndocrineSystem {
    type Prior = EndocrineState;
    type Evidence = f64; // decay_rate (unused - apply_decay uses internal rates)
    type Posterior = EndocrineState;

    fn normalize(&self, mut prior: Self::Prior, _evidence: &Self::Evidence) -> Self::Posterior {
        prior.apply_decay();
        prior
    }
}

/// CODIFY: Export state to behavioral modifiers
impl Codify for EndocrineSystem {
    type Belief = EndocrineState;
    type Representation = BehaviorModulation;

    fn codify(&self, belief: &Self::Belief) -> Self::Representation {
        BehaviorModulation::from(BehavioralModifiers::from(belief))
    }
}

/// EXTEND: Apply modifiers to external system (AI behavior parameters)
impl Extend for EndocrineSystem {
    type Source = BehaviorModulation;
    type Target = ToneProfile;

    fn extend(&self, source: &Self::Source) -> Self::Target {
        ToneProfile {
            directness: 0.5 + (source.risk_tolerance * 0.3),
            hedging: (1.0 - source.risk_tolerance) * 0.4,
            warmth: source.warmth,
            precision: source.validation_depth,
            verbosity: 1.0 - source.urgency,
        }
    }
}

/// Tone profile for AI output style (T3 domain-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneProfile {
    pub directness: f64,
    pub hedging: f64,
    pub warmth: f64,
    pub precision: f64,
    pub verbosity: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sense_detects_stimulus() {
        let system = EndocrineSystem::new();
        let stimulus = Stimulus::TaskCompleted { complexity: 0.8 };
        let signal = system.sense(&stimulus);

        assert_eq!(signal.stimulus, StimulusCategory::Reward);
        assert!((signal.intensity - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn classify_returns_category() {
        let system = EndocrineSystem::new();
        let signal = HormoneSignal {
            stimulus: StimulusCategory::Stress,
            intensity: 0.5,
            confidence: Confidence::new(0.9),
        };

        assert_eq!(system.classify(&signal), StimulusCategory::Stress);
    }

    #[test]
    fn infer_predicts_hormone_changes() {
        let system = EndocrineSystem::new();
        let predictions = system.infer(&StimulusCategory::Reward, &0.8);

        assert!(!predictions.is_empty());
        assert!(predictions.iter().any(|(h, _)| *h == HormoneType::Dopamine));
    }

    #[test]
    fn codify_produces_modulation() {
        let system = EndocrineSystem::new();
        let modulation = system.codify(&system.state);

        // Default state should have balanced modulation
        assert!(modulation.risk_tolerance >= 0.0 && modulation.risk_tolerance <= 1.0);
    }

    #[test]
    fn extend_produces_tone_profile() {
        let system = EndocrineSystem::new();
        let modulation = BehaviorModulation {
            risk_tolerance: 0.7,
            validation_depth: 0.8,
            exploration_rate: 0.6,
            warmth: 0.75,
            urgency: 0.3,
        };

        let tone = system.extend(&modulation);
        assert!(tone.directness > 0.5); // Higher risk tolerance → more direct
        assert!(tone.precision > 0.5); // Higher validation → more precise
    }
}
