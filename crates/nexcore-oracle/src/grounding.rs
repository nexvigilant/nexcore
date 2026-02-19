//! # Grounding — T1 primitive declarations for Oracle types
//!
//! Every type in nexcore-oracle grounds to Lex Primitiva.

use nexcore_lex_primitiva::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::accuracy::AccuracyTracker;
use crate::event::{Event, EventSequence};
use crate::matrix::TransitionMatrix;
use crate::predictor::{Prediction, Predictor};

// Event: ∃(Existence) + ν(Frequency) + λ(Location)
// An event is something that exists at a point in time
impl GroundsTo for Event {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Frequency,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.90)
    }
}

// EventSequence: σ(Sequence) + ∃(Existence) + ν(Frequency)
// An ordered series of events
impl GroundsTo for EventSequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

// TransitionMatrix: →(Causality) + N(Quantity) + μ(Mapping) + ν(Frequency)
// Maps cause-effect relationships with quantified frequencies
impl GroundsTo for TransitionMatrix {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// Predictor: σ(Sequence) + →(Causality) + ν(Frequency) + κ(Comparison) + N(Quantity) + π(Persistence)
// The full Oracle: learns sequences, compares predictions, persists knowledge
impl GroundsTo for Predictor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Causality,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// Prediction: →(Causality) + N(Quantity) + κ(Comparison)
// A causal prediction with quantified confidence
impl GroundsTo for Prediction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

// AccuracyTracker: ν(Frequency) + N(Quantity) + κ(Comparison) + π(Persistence)
// Tracks prediction frequency and accuracy over time
impl GroundsTo for AccuracyTracker {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_grounding() {
        let comp = Event::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
    }

    #[test]
    fn predictor_grounding() {
        let comp = Predictor::primitive_composition();
        assert_eq!(comp.primitives.len(), 6);
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn prediction_grounding() {
        let comp = Prediction::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn matrix_grounding() {
        let comp = TransitionMatrix::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn accuracy_grounding() {
        let comp = AccuracyTracker::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }
}
