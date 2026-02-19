//! # Chemistry Primitives - Universal Computational Patterns
//!
//! Chemistry equations decomposed to T1 primitives, providing cross-domain
//! computational patterns for signal detection, throughput analysis, and
//! feasibility assessment.
//!
//! ## T1 Foundation
//!
//! All primitives compose from these T1 universals:
//! - **Causation**: cause, effect, mechanism
//! - **Temporal**: frequency, duration, sequence
//! - **Quantitative**: quantity, threshold, maximum, ratio, proportion
//! - **State**: state, changes, persists, transition
//! - **Relational**: dependency, connection
//! - **Information**: signal, noise, pattern
//!
//! ## Primitive Catalog
//!
//! | Equation | Primitive | T1 Components | PV Application |
//! |----------|-----------|---------------|----------------|
//! | Arrhenius | `threshold_gating` | threshold × quantity × frequency | Signal activation |
//! | Michaelis-Menten | `saturation` | maximum × cause × effect × ratio | Throughput limits |
//! | Gibbs | `feasibility` | quantity × comparison × state | Causality likelihood |
//! | Rate Laws | `dependency` | dependency × frequency × quantity | Propagation |
//! | Henderson-Hasselbalch | `buffer` | state × ratio × persists | Baseline stability |
//! | Beer-Lambert | `signal_intensity` | signal × proportion × quantity | Dose-response |
//! | Half-Life | `decay` | duration × quantity × ratio | Signal persistence |
//! | Equilibrium | `equilibrium` | state × persists × ratio | Steady state |
//!
//! ## Integration
//!
//! These primitives wrap existing `pv::thermodynamic` and `pv::pk` functions
//! with universal semantics, enabling cross-domain transfer.
//!
//! ```ignore
//! use nexcore_vigilance::primitives::chemistry::{
//!     threshold_gating, saturation, feasibility, decay_kinetics,
//! };
//!
//! // Signal detection: does observed rate exceed activation threshold?
//! let rate = threshold_gating::arrhenius_rate(1e13, 50.0, 298.15)?;
//!
//! // Throughput: system capacity at given load
//! let throughput = saturation::michaelis_menten_rate(500.0, 1000.0, 200.0)?;
//!
//! // Feasibility: is this action spontaneous (favorable)?
//! let is_favorable = feasibility::is_favorable(-50.0, 10.0, 2.0);
//!
//! // Decay: signal persistence over time
//! let remaining = decay_kinetics::remaining_after_time(100.0, 0.023, 90.0);
//! ```

pub mod adsorption;
pub mod aggregation_pipeline;
pub mod buffer_stability;
pub mod cooperativity;
pub mod decay_kinetics;
pub mod dependency;
pub mod electrochemical;
pub mod equilibrium;
pub mod feasibility;
pub mod inhibition;
pub mod saturation;
pub mod signal_intensity;
pub mod threshold_gating;
pub mod transition_state;

// Re-export for convenient access
pub use aggregation_pipeline::AggregationPipeline;
pub use buffer_stability::{BufferError, BufferSystem, buffer_capacity, ionization_ratio};
pub use decay_kinetics::{
    DecayError, DecayKinetics, decay_constant_from_half_life, first_order_decay,
    half_life_from_decay_constant, remaining_after_time, time_to_fraction,
};
pub use dependency::{DependencyError, RateLaw, calculate_rate_law, rate_limiting_factor};
pub use equilibrium::{
    EquilibriumError, EquilibriumSystem, equilibrium_constant, steady_state_fractions,
    time_to_equilibrium,
};
pub use feasibility::{
    Favorability, FeasibilityAssessment, FeasibilityError, classify_favorability,
    gibbs_free_energy, is_favorable,
};
pub use saturation::{
    SaturationError, SaturationKinetics, michaelis_menten_rate, saturation_fraction,
    utilization_at_load,
};
pub use signal_intensity::{
    SignalDetector, SignalError, beer_lambert_absorbance, detection_limit, infer_concentration,
};
pub use threshold_gating::{
    ThresholdError, ThresholdGate, activation_probability, arrhenius_rate, threshold_exceeded,
};

// New cooperative/electrochemical/inhibition exports
pub use cooperativity::{
    CooperativeBinding, CooperativityError, CooperativityType, classify_cooperativity,
    hill_response, infer_hill_coefficient,
};
pub use electrochemical::{
    ElectrochemicalCell, ElectrochemicalError, FARADAY, GAS_CONSTANT, PotentialState,
    dynamic_threshold, millivolts_per_decade, nernst_factor_standard, nernst_potential,
};
pub use inhibition::{
    CompetitiveInhibition, InhibitionError, InhibitionStrength, apparent_km, classify_inhibition,
    inhibited_rate, inhibitor_for_fractional, throughput_reduction,
};

// Transition state theory (Eyring) exports
pub use transition_state::{
    ActivationParameters, BOLTZMANN, PLANCK, TransitionState, TransitionStateError,
    activation_from_rates, compare_to_arrhenius, eyring_rate, frequency_factor, gibbs_activation,
};

// Langmuir adsorption exports
pub use adsorption::{
    AdsorptionError, CompetitiveLangmuir, CoverageState, LangmuirIsotherm, adsorption_free_energy,
    amount_from_coverage, classify_coverage, equilibrium_from_coverage,
    half_coverage_concentration, langmuir_coverage,
};

use serde::{Deserialize, Serialize};

/// PV Context mapping for chemistry primitives.
///
/// Maps chemistry concepts to their PV equivalents for cross-domain transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PvMapping {
    /// Chemistry term
    pub chemistry_term: &'static str,
    /// PV equivalent
    pub pv_equivalent: &'static str,
    /// Transfer confidence (0.0-1.0)
    pub confidence: f64,
    /// Mapping rationale
    pub rationale: &'static str,
}

/// Get all chemistry → PV mappings.
#[must_use]
pub fn pv_mappings() -> Vec<PvMapping> {
    vec![
        PvMapping {
            chemistry_term: "activation_energy",
            pv_equivalent: "signal_detection_threshold",
            confidence: 0.92,
            rationale: "Both gate action on exceeding energy barrier",
        },
        PvMapping {
            chemistry_term: "saturation_kinetics",
            pv_equivalent: "case_processing_capacity",
            confidence: 0.88,
            rationale: "Both exhibit hyperbolic throughput curves",
        },
        PvMapping {
            chemistry_term: "gibbs_free_energy",
            pv_equivalent: "causality_likelihood",
            confidence: 0.85,
            rationale: "Spontaneous reaction ↔ likely causal relationship",
        },
        PvMapping {
            chemistry_term: "rate_law_order",
            pv_equivalent: "signal_dependency",
            confidence: 0.82,
            rationale: "Both describe how inputs affect output rate",
        },
        PvMapping {
            chemistry_term: "buffer_capacity",
            pv_equivalent: "baseline_stability",
            confidence: 0.78,
            rationale: "Both resist perturbation around setpoint",
        },
        PvMapping {
            chemistry_term: "beer_lambert",
            pv_equivalent: "dose_response_linearity",
            confidence: 0.75,
            rationale: "Linear relationship between concentration and signal",
        },
        PvMapping {
            chemistry_term: "half_life",
            pv_equivalent: "signal_persistence",
            confidence: 0.90,
            rationale: "Both describe exponential decay over time",
        },
        PvMapping {
            chemistry_term: "equilibrium_constant",
            pv_equivalent: "reporting_baseline",
            confidence: 0.72,
            rationale: "Both represent steady-state balance point",
        },
        // New mappings (v2)
        PvMapping {
            chemistry_term: "hill_cooperativity",
            pv_equivalent: "signal_cascade_amplification",
            confidence: 0.85,
            rationale: "nH > 1 amplifies weak signals; nH < 1 dampens noise",
        },
        PvMapping {
            chemistry_term: "nernst_potential",
            pv_equivalent: "dynamic_decision_threshold",
            confidence: 0.80,
            rationale: "Threshold shifts with background concentration",
        },
        PvMapping {
            chemistry_term: "competitive_inhibition",
            pv_equivalent: "signal_interference_factor",
            confidence: 0.78,
            rationale: "Competing signals raise apparent detection threshold",
        },
        // New mappings (v3) - Eyring and Langmuir
        PvMapping {
            chemistry_term: "eyring_transition_state",
            pv_equivalent: "signal_escalation_rate",
            confidence: 0.82,
            rationale: "Accounts for both threshold (ΔH‡) and process complexity (ΔS‡)",
        },
        PvMapping {
            chemistry_term: "langmuir_adsorption",
            pv_equivalent: "case_slot_occupancy",
            confidence: 0.88,
            rationale: "Finite reviewer slots compete for cases; saturation behavior",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_mappings_count() {
        let mappings = pv_mappings();
        assert_eq!(mappings.len(), 13); // 8 original + 3 v2 (Hill, Nernst, Inhibition) + 2 v3 (Eyring, Langmuir)
    }

    #[test]
    fn test_pv_mappings_confidence_range() {
        for mapping in pv_mappings() {
            assert!(mapping.confidence >= 0.0 && mapping.confidence <= 1.0);
        }
    }

    #[test]
    fn test_highest_confidence_mappings() {
        let mappings = pv_mappings();
        let high_conf: Vec<_> = mappings.iter().filter(|m| m.confidence >= 0.85).collect();
        assert!(
            high_conf.len() >= 4,
            "Should have at least 4 high-confidence mappings"
        );
    }
}
