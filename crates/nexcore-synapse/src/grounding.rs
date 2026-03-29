//! # GroundsTo implementations for nexcore-synapse types
//!
//! Connects amplitude growth learning model types to the Lex Primitiva type system.
//!
//! ## Biological Analogy
//!
//! A synapse strengthens through repeated activation. This crate models
//! learning as amplitude growth with exponential decay and saturation kinetics.
//!
//! ## Key Primitive Mapping
//!
//! - Amplitude: N (Quantity) -- numeric learning strength
//! - Decay: proportional (Irreversibility) -- exponential time decay
//! - Saturation: partial (Boundary) -- Michaelis-Menten bounds
//! - Observation frequency: nu (Frequency) -- observation count
//! - Consolidation: varsigma (State) -- learning state transitions
//! - Persistence: pi (Persistence) -- cross-session durability

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    Amplitude, AmplitudeConfig, ConsolidationStatus, LearningSignal, SaturationKinetics, Synapse,
    SynapseBank,
};

// ---------------------------------------------------------------------------
// Primitive types -- N (Quantity) dominant
// ---------------------------------------------------------------------------

/// Amplitude: T2-P (N + partial), dominant N
///
/// Learning strength value clamped to [0.0, 1.0].
/// Quantity-dominant: it IS a numeric measurement.
/// Boundary is secondary (clamped range).
impl GroundsTo for Amplitude {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric learning strength
            LexPrimitiva::Boundary, // partial -- [0, 1] clamping
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// LearningSignal: T2-P (exists + N + nu), dominant exists
///
/// A single observation contributing to amplitude growth.
/// Existence-dominant: the signal IS an existential observation event.
/// Quantity is secondary (confidence, relevance are numeric).
/// Frequency is tertiary (observation timestamp for rate tracking).
impl GroundsTo for LearningSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- observation event
            LexPrimitiva::Quantity,  // N -- confidence, relevance
            LexPrimitiva::Frequency, // nu -- observation timing
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Kinetics types -- partial (Boundary) dominant
// ---------------------------------------------------------------------------

/// SaturationKinetics: T2-P (partial + N + rho), dominant partial
///
/// Michaelis-Menten saturation parameters (Vmax, Km).
/// Boundary-dominant: it defines the asymptotic growth boundary.
/// Quantity is secondary (Vmax, Km are numeric).
/// Recursion is tertiary (iterative saturation application).
impl GroundsTo for SaturationKinetics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // partial -- saturation boundary
            LexPrimitiva::Quantity,  // N -- Vmax, Km parameters
            LexPrimitiva::Recursion, // rho -- iterative application
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

/// AmplitudeConfig: T2-C (partial + N + proportional + pi), dominant partial
///
/// Configuration for amplitude growth: learning rate, half-life, threshold.
/// Boundary-dominant: it defines all the boundary parameters.
/// Quantity is secondary (numeric config values).
/// Irreversibility is tertiary (decay half-life).
/// Persistence is quaternary (cross-session flag).
impl GroundsTo for AmplitudeConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,        // partial -- threshold/limit boundaries
            LexPrimitiva::Quantity,        // N -- numeric config values
            LexPrimitiva::Irreversibility, // proportional -- decay half-life
            LexPrimitiva::Persistence,     // pi -- cross-session persistence
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// State types -- varsigma (State) dominant
// ---------------------------------------------------------------------------

/// ConsolidationStatus: T2-P (varsigma + partial), dominant varsigma
///
/// Accumulating, Consolidated, or Decayed learning state.
/// State-dominant: it IS a state classification.
/// Boundary is secondary (threshold crossing determines state).
impl GroundsTo for ConsolidationStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- learning state
            LexPrimitiva::Boundary, // partial -- threshold boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Domain types -- T2-C / T3
// ---------------------------------------------------------------------------

/// Synapse: T3 (nu + N + varsigma + partial + proportional + pi), dominant nu
///
/// A learning synapse that grows amplitude through observations.
/// Frequency-dominant: the synapse IS driven by observation frequency.
/// Quantity is secondary (amplitude value).
/// State is tertiary (consolidation state).
/// Boundary is quaternary (saturation, threshold).
/// Irreversibility is quinary (exponential decay).
/// Persistence is senary (cross-session durability).
impl GroundsTo for Synapse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,       // nu -- observation frequency
            LexPrimitiva::Quantity,        // N -- amplitude value
            LexPrimitiva::State,           // varsigma -- consolidation state
            LexPrimitiva::Boundary,        // partial -- saturation, threshold
            LexPrimitiva::Irreversibility, // proportional -- exponential decay
            LexPrimitiva::Persistence,     // pi -- cross-session durability
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

/// SynapseBank: T3 (pi + mu + varsigma + sigma + nu + N), dominant pi
///
/// Collection of synapses for managing multiple learning targets.
/// Persistence-dominant: the bank IS a persistent store of learning.
/// Mapping is secondary (ID -> synapse lookup).
/// State is tertiary (collection state).
/// Sequence is quaternary (iteration order).
/// Frequency is quinary (per-synapse observation rates).
/// Quantity is senary (synapse count).
impl GroundsTo for SynapseBank {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- persistent learning store
            LexPrimitiva::Mapping,     // mu -- ID -> synapse lookup
            LexPrimitiva::State,       // varsigma -- bank state
            LexPrimitiva::Sequence,    // sigma -- iteration order
            LexPrimitiva::Frequency,   // nu -- observation rates
            LexPrimitiva::Quantity,    // N -- synapse count
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn amplitude_is_t2p() {
        assert_eq!(Amplitude::tier(), Tier::T2Primitive);
        assert_eq!(
            Amplitude::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn learning_signal_is_t2p() {
        assert_eq!(LearningSignal::tier(), Tier::T2Primitive);
        assert_eq!(
            LearningSignal::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn saturation_kinetics_is_t2p() {
        assert_eq!(SaturationKinetics::tier(), Tier::T2Primitive);
        assert_eq!(
            SaturationKinetics::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn consolidation_status_is_t2p() {
        assert_eq!(ConsolidationStatus::tier(), Tier::T2Primitive);
        assert_eq!(
            ConsolidationStatus::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn synapse_is_t3() {
        assert_eq!(Synapse::tier(), Tier::T3DomainSpecific);
        assert_eq!(Synapse::dominant_primitive(), Some(LexPrimitiva::Frequency));
    }

    #[test]
    fn synapse_bank_is_t3() {
        assert_eq!(SynapseBank::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            SynapseBank::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn synapse_contains_decay() {
        let comp = Synapse::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
    }

    #[test]
    fn synapse_bank_contains_mapping() {
        let comp = SynapseBank::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            Amplitude::primitive_composition(),
            LearningSignal::primitive_composition(),
            SaturationKinetics::primitive_composition(),
            AmplitudeConfig::primitive_composition(),
            ConsolidationStatus::primitive_composition(),
            Synapse::primitive_composition(),
            SynapseBank::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
