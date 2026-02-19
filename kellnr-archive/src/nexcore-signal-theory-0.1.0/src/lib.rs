// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Universal Theory of Signals
//!
//! Domain-agnostic foundations for signal detection, extracted from
//! pharmacovigilance and generalized for universal application.
//!
//! ## Core Thesis
//!
//! **"All detection is boundary drawing."**
//!
//! Every signal detection system can be modeled as:
//! - An **observation space** (the data)
//! - A **noise model** (what we expect under null)
//! - A **boundary** (the threshold that separates signal from noise)
//! - A **comparison** (observed vs expected)
//! - A **decision** (detected / not detected)
//! - A **causal inference** (accumulating evidence toward causality)
//!
//! ## Primitive Grounding
//!
//! The theory grounds to the Lex Primitiva:
//!
//! | Symbol | Name | Role in Signal Theory |
//! |--------|------|----------------------|
//! | ∂ | Boundary | **Dominant** — thresholds, detection boundaries |
//! | κ | Comparison | Observed vs expected, decision outcomes |
//! | N | Quantity | Counts, rates, measures |
//! | ν | Frequency | Data generation rates |
//! | ∅ | Void | Noise, missing signals |
//! | ∃ | Existence | Signal existence, validation |
//! | → | Causality | Causal inference, evidence accumulation |
//! | Σ | Sum | Aggregation, conservation totals |
//! | σ | Sequence | Detection pipelines, cascades |
//! | π | Persistence | Evidence persistence, monitoring |
//!
//! ## Axiom System (6 axioms)
//!
//! | Axiom | Name | Dominant | Statement |
//! |-------|------|----------|-----------|
//! | **A1** | Data Generation | ν | Any observable system generates data |
//! | **A2** | Noise Dominance | ∅ | Noise dominates signal (ratio > 0.5) |
//! | **A3** | Signal Existence | ∃ | At least one true signal exists |
//! | **A4** | Boundary Requirement | ∂ | Detection requires a threshold |
//! | **A5** | Disproportionality | κ | Signal = observed > expected |
//! | **A6** | Causal Inference | → | Evidence must accumulate toward causality |
//!
//! ## Module Structure
//!
//! ```text
//! nexcore-signal-theory/
//! ├── axioms/        # A1-A6 formal definitions and witnesses
//! ├── detection/     # ObservationSpace, Baseline, Ratio, DetectionOutcome
//! ├── threshold/     # Fixed/Adaptive/Composite boundaries, presets
//! ├── algebra/       # Parallel, sequential, cascaded detection
//! ├── decision/      # SDT 2×2 matrix, ROC, d', response bias
//! ├── conservation/  # 4 conservation laws (L1-L4)
//! ├── theorems/      # 5 theorems (T1-T5) + registry
//! └── grounding/     # 34+ GroundsTo implementations
//! ```
//!
//! ## Example: Basic Signal Detection
//!
//! ```rust
//! use nexcore_signal_theory::prelude::*;
//!
//! // 1. Draw a boundary (A4)
//! let boundary = FixedBoundary::above(2.0, "PRR");
//!
//! // 2. Check disproportionality (A5)
//! let ratio = Ratio::from_counts(15.0, 5.0);
//!
//! // 3. Evaluate detection
//! if let Some(r) = ratio {
//!     let outcome = if boundary.evaluate(r.0) {
//!         DetectionOutcome::Detected
//!     } else {
//!         DetectionOutcome::NotDetected
//!     };
//!     assert!(outcome.is_detected());
//!     assert_eq!(SignalStrengthLevel::from_ratio(r.0), SignalStrengthLevel::Moderate);
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

extern crate alloc;

// Core modules
pub mod axioms;
pub mod detection;
pub mod threshold;

// Composition modules
pub mod algebra;
pub mod decision;

// Verification modules
pub mod conservation;
pub mod theorems;

// Grounding
pub mod grounding;

/// Prelude for convenient imports.
pub mod prelude {
    // Axioms
    pub use crate::axioms::{
        A1DataGeneration, A2NoiseDominance, A3SignalExistence, A4BoundaryRequirement,
        A5Disproportionality, A6CausalInference, Axiom, EvidenceKind, SignalTheoryProof,
    };

    // Core traits and types
    pub use crate::SignalPrimitive;

    // Detection
    pub use crate::detection::{
        Baseline, DetectionInterval, DetectionOutcome, Difference, ObservationSpace, Ratio,
        SignalStrengthLevel, SignalVerificationReport,
    };

    // Threshold
    pub use crate::threshold::{
        AdaptiveBoundary, BoundaryKind, CompositeBoundary, ConjunctionMode, DetectionPhase,
        FixedBoundary, ThresholdPreset,
    };

    // Algebra
    pub use crate::algebra::{
        CascadedThreshold, DetectionIteration, DetectionPipeline, Detector, ParallelDetection,
        SequentialDetection, ThresholdDetector,
    };

    // Decision
    pub use crate::decision::{
        DPrime, DecisionMatrix, DecisionOutcome, ResponseBias, RocCurve, RocPoint,
    };

    // Conservation
    pub use crate::conservation::{
        ConservationLaw, ConservationReport, L1TotalCountConservation, L2BaseRateInvariance,
        L3SensitivitySpecificityTradeoff, L4InformationConservation, LawVerification,
    };

    // Theorems
    pub use crate::theorems::{
        T1NeymanPearson, T2ParallelSpecificity, T3SequentialFPReduction, T4ThresholdMonotonicity,
        T5CausalAccumulation, Theorem, TheoremRegistry, TheoremSummary,
    };
}

// ═══════════════════════════════════════════════════════════
// PRIMITIVE SYMBOLS
// ═══════════════════════════════════════════════════════════

/// The 6 primitives central to signal theory.
///
/// Extracted from the full 16-symbol Lex Primitiva.
/// These correspond 1:1 to the 6 axioms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SignalPrimitive {
    /// ν — Frequency (A1: Data Generation)
    Frequency,
    /// ∅ — Void (A2: Noise Dominance)
    Void,
    /// ∃ — Existence (A3: Signal Existence)
    Existence,
    /// ∂ — Boundary (A4: Boundary Requirement) **[DOMINANT]**
    Boundary,
    /// κ — Comparison (A5: Disproportionality)
    Comparison,
    /// → — Causality (A6: Causal Inference)
    Causality,
}

impl SignalPrimitive {
    /// Unicode symbol for this primitive.
    #[must_use]
    pub const fn symbol(&self) -> char {
        match self {
            Self::Frequency => 'ν',
            Self::Void => '∅',
            Self::Existence => '∃',
            Self::Boundary => '∂',
            Self::Comparison => 'κ',
            Self::Causality => '→',
        }
    }

    /// All primitives in axiom order (A1-A6).
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Frequency,
            Self::Void,
            Self::Existence,
            Self::Boundary,
            Self::Comparison,
            Self::Causality,
        ]
    }

    /// The corresponding axiom ID.
    #[must_use]
    pub const fn axiom_id(&self) -> &'static str {
        match self {
            Self::Frequency => "A1",
            Self::Void => "A2",
            Self::Existence => "A3",
            Self::Boundary => "A4",
            Self::Comparison => "A5",
            Self::Causality => "A6",
        }
    }

    /// Whether this is the dominant primitive (∂ Boundary).
    #[must_use]
    pub const fn is_dominant(&self) -> bool {
        matches!(self, Self::Boundary)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_primitive_symbols() {
        assert_eq!(SignalPrimitive::Frequency.symbol(), 'ν');
        assert_eq!(SignalPrimitive::Void.symbol(), '∅');
        assert_eq!(SignalPrimitive::Existence.symbol(), '∃');
        assert_eq!(SignalPrimitive::Boundary.symbol(), '∂');
        assert_eq!(SignalPrimitive::Comparison.symbol(), 'κ');
        assert_eq!(SignalPrimitive::Causality.symbol(), '→');
    }

    #[test]
    fn test_signal_primitive_all() {
        assert_eq!(SignalPrimitive::all().len(), 6);
    }

    #[test]
    fn test_signal_primitive_axiom_ids() {
        assert_eq!(SignalPrimitive::Frequency.axiom_id(), "A1");
        assert_eq!(SignalPrimitive::Boundary.axiom_id(), "A4");
        assert_eq!(SignalPrimitive::Causality.axiom_id(), "A6");
    }

    #[test]
    fn test_signal_primitive_dominant() {
        assert!(SignalPrimitive::Boundary.is_dominant());
        assert!(!SignalPrimitive::Frequency.is_dominant());
    }

    #[test]
    fn test_prelude_imports() {
        // Verify prelude provides access to key types
        use crate::prelude::*;

        let _boundary = FixedBoundary::above(2.0, "test");
        let _outcome = DetectionOutcome::Detected;
        let _strength = SignalStrengthLevel::from_ratio(3.0);
        let _preset = ThresholdPreset::Default;
    }
}
