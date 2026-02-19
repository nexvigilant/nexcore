//! # GroundsTo implementations for nexcore-cardiovascular types
//!
//! Connects cardiovascular types to the Lex Primitiva type system.
//!
//! ## → (Causality) Focus
//!
//! The cardiovascular system IS a causal transport network:
//! heart pumps → blood flows → organs receive → waste returns.
//! Every pump cycle causes data movement across the system.
//!
//! ## Dominant Primitives by Module
//!
//! | Module | Dominant | Why |
//! |--------|----------|-----|
//! | Heart | → Causality | Pump causes circulation |
//! | Vessels | ∂ Boundary | Vessel walls define channels |
//! | Blood | μ Mapping | Maps payloads to destinations |
//! | Pressure | κ Comparison | Compares current vs set point |
//! | Pathology | ∝ Irreversibility | Damage is one-way |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::blood::{BloodHealth, Plasma, Platelet, RedCell, WhiteCell};
use crate::heart::{Heart, HeartConfig, PumpResult};
use crate::pathology::{Diagnosis, Severity};
use crate::pressure::{Baroreceptor, PressureClass, PressureReading};
use crate::types::{
    BloodComponent, Circuit, FlowRate, HeartRate, Pathology, Pressure, Resistance, VesselType,
    Volume,
};
use crate::vessels::{CapillaryExchange, VascularBed, Vessel};

// ═══════════════════════════════════════════════════════════
// Newtypes — T2-P (single-primitive wrappers)
// ═══════════════════════════════════════════════════════════

/// Pressure: T2-P (N · κ), dominant N
///
/// A quantified measurement compared against thresholds.
impl GroundsTo for Pressure {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — numeric value
            LexPrimitiva::Comparison, // κ — threshold comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// FlowRate: T2-P (N · ν), dominant N
///
/// Quantity per unit time.
impl GroundsTo for FlowRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N — numeric rate
            LexPrimitiva::Frequency, // ν — per time step
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Volume: T2-P (N · ∂), dominant N
///
/// Bounded quantity.
impl GroundsTo for Volume {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — amount
            LexPrimitiva::Boundary, // ∂ — capacity bound
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Resistance: T2-P (N · ∂), dominant ∂
///
/// Boundary opposition to flow.
impl GroundsTo for Resistance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — opposition to flow
            LexPrimitiva::Quantity, // N — magnitude
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// HeartRate: T2-P (ν · N), dominant ν
///
/// Frequency of pump cycles.
impl GroundsTo for HeartRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν — beats per time
            LexPrimitiva::Quantity,  // N — numeric value
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.90)
    }
}

// ═══════════════════════════════════════════════════════════
// Enums — T2-P (sum types)
// ═══════════════════════════════════════════════════════════

/// Circuit: T2-P (Σ · →), dominant Σ
///
/// Sum type: Pulmonary | Systemic | Portal.
impl GroundsTo for Circuit {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ — variant alternation
            LexPrimitiva::Causality, // → — each circuit causes different processing
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// VesselType: T2-P (Σ · ∂), dominant Σ
///
/// Seven-variant vessel classification.
impl GroundsTo for VesselType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — variant alternation
            LexPrimitiva::Boundary, // ∂ — vessel wall properties
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// BloodComponent: T2-P (Σ · μ), dominant Σ
///
/// Four-variant component classification.
impl GroundsTo for BloodComponent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Σ — variant alternation
            LexPrimitiva::Mapping, // μ — each maps to a function
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// PressureClass: T2-P (Σ · κ), dominant κ
///
/// Classification by comparison against thresholds.
impl GroundsTo for PressureClass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — classified by comparison
            LexPrimitiva::Sum,        // Σ — three variants
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Severity: T2-P (κ · N), dominant κ
///
/// Ordered comparison: Mild < Moderate < Severe < Critical.
impl GroundsTo for Severity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — ordered severity
            LexPrimitiva::Quantity,   // N — ordinal level
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// BloodHealth: T2-P (Σ · κ), dominant Σ
///
/// Five-variant health classification.
impl GroundsTo for BloodHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — variant alternation
            LexPrimitiva::Comparison, // κ — classified by comparing counts
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// Composite types — T2-C and T3
// ═══════════════════════════════════════════════════════════

/// HeartConfig: T2-C (N · ν · κ · ∂), dominant N
///
/// Numeric configuration parameters.
impl GroundsTo for HeartConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — numeric thresholds
            LexPrimitiva::Frequency,  // ν — rate limits
            LexPrimitiva::Comparison, // κ — max comparisons
            LexPrimitiva::Boundary,   // ∂ — ceiling constraints
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Heart: T3 (→ · σ · ν · ∂ · ς · N), dominant →
///
/// The central pump — causality engine. Pump causes circulation.
/// Pipeline: collect → enrich → distribute (sequential causality).
impl GroundsTo for Heart {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → — pump causes flow
            LexPrimitiva::Sequence,  // σ — collect→enrich→route pipeline
            LexPrimitiva::Frequency, // ν — heartbeat rhythm
            LexPrimitiva::Boundary,  // ∂ — max capacity constraints
            LexPrimitiva::State,     // ς — beat count, active circuit
            LexPrimitiva::Quantity,  // N — stroke volume, cardiac output
        ])
        .with_dominant(LexPrimitiva::Causality, 0.95)
    }
}

/// PumpResult: T2-C (→ · N · Σ · ν), dominant →
///
/// The causal output of a pump cycle.
impl GroundsTo for PumpResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → — result of pump action
            LexPrimitiva::Quantity,  // N — volumes and rates
            LexPrimitiva::Sum,       // Σ — which circuit
            LexPrimitiva::Frequency, // ν — cardiac output rate
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// Vessel: T2-C (∂ · N · → · ς), dominant ∂
///
/// A bounded channel through which data flows.
impl GroundsTo for Vessel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — vessel walls define channel
            LexPrimitiva::Quantity,  // N — capacity, volume, resistance
            LexPrimitiva::Causality, // → — flow through vessel
            LexPrimitiva::State,     // ς — current fill, occlusion
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// VascularBed: T3 (∂ · σ · μ · → · N · ς), dominant ∂
///
/// A network of vessels forming a transport bed.
impl GroundsTo for VascularBed {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — composite vessel boundaries
            LexPrimitiva::Sequence,  // σ — ordered vessel chain
            LexPrimitiva::Mapping,   // μ — routes to destinations
            LexPrimitiva::Causality, // → — flow causes delivery
            LexPrimitiva::Quantity,  // N — total resistance, volume
            LexPrimitiva::State,     // ς — aggregate state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// CapillaryExchange: T2-C (μ · N · → · ∂), dominant μ
///
/// Exchange IS mapping: nutrients→tissue, waste→blood.
impl GroundsTo for CapillaryExchange {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // μ — bidirectional exchange
            LexPrimitiva::Quantity,  // N — amounts exchanged
            LexPrimitiva::Causality, // → — pressure causes flow
            LexPrimitiva::Boundary,  // ∂ — membrane boundary
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// RedCell: T2-C (μ · N · → · ∂), dominant μ
///
/// Maps payloads to destinations.
impl GroundsTo for RedCell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // μ — payload→destination
            LexPrimitiva::Quantity,  // N — capacity, load
            LexPrimitiva::Causality, // → — carries from source to dest
            LexPrimitiva::Boundary,  // ∂ — capacity limit
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// WhiteCell: T2-C (∂ · κ · → · ∃), dominant ∂
///
/// Boundary defense: detect and neutralize threats.
impl GroundsTo for WhiteCell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — defense at boundary
            LexPrimitiva::Comparison, // κ — threat matching
            LexPrimitiva::Causality,  // → — detection causes response
            LexPrimitiva::Existence,  // ∃ — threat existence check
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// Platelet: T2-C (∂ · ∝ · ς), dominant ∂
///
/// Repairs boundary breaches.
impl GroundsTo for Platelet {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,        // ∂ — repairs boundaries
            LexPrimitiva::Irreversibility, // ∝ — permanent seal
            LexPrimitiva::State,           // ς — activated/inactive
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// Plasma: T2-P (N · ∂), dominant N
///
/// Quantified transport medium.
impl GroundsTo for Plasma {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — volume
            LexPrimitiva::Boundary, // ∂ — containment
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Baroreceptor: T2-C (κ · ν · → · ∂), dominant κ
///
/// Pressure sensor using comparison against set point.
impl GroundsTo for Baroreceptor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — compare vs set point
            LexPrimitiva::Frequency,  // ν — continuous monitoring
            LexPrimitiva::Causality,  // → — deviation causes response
            LexPrimitiva::Boundary,   // ∂ — threshold boundaries
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// PressureReading: T2-P (N · κ), dominant N
///
/// A numeric reading with classification.
impl GroundsTo for PressureReading {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — pressure value
            LexPrimitiva::Comparison, // κ — classification
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Pathology: T2-C (Σ · ∝ · ∂ · N), dominant ∝
///
/// Irreversible disease — damage that cannot be simply undone.
impl GroundsTo for Pathology {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // ∝ — damage is one-way
            LexPrimitiva::Sum,             // Σ — variant pathology types
            LexPrimitiva::Boundary,        // ∂ — boundary affected
            LexPrimitiva::Quantity,        // N — severity measures
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.90)
    }
}

/// Diagnosis: T3 (∝ · κ · → · Σ · N · μ), dominant →
///
/// Causality-dominant: observation → classification → recommendation.
impl GroundsTo for Diagnosis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,       // → — leads to recommendation
            LexPrimitiva::Irreversibility, // ∝ — pathology irreversibility
            LexPrimitiva::Comparison,      // κ — severity classification
            LexPrimitiva::Sum,             // Σ — pathology variant
            LexPrimitiva::Quantity,        // N — severity measures
            LexPrimitiva::Mapping,         // μ — pathology→treatment
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_groundings_have_dominant() {
        // Verify every GroundsTo impl produces a valid composition
        let compositions: Vec<(&str, PrimitiveComposition)> = vec![
            ("Pressure", Pressure::primitive_composition()),
            ("FlowRate", FlowRate::primitive_composition()),
            ("Volume", Volume::primitive_composition()),
            ("Resistance", Resistance::primitive_composition()),
            ("HeartRate", HeartRate::primitive_composition()),
            ("Circuit", Circuit::primitive_composition()),
            ("VesselType", VesselType::primitive_composition()),
            ("BloodComponent", BloodComponent::primitive_composition()),
            ("PressureClass", PressureClass::primitive_composition()),
            ("Severity", Severity::primitive_composition()),
            ("BloodHealth", BloodHealth::primitive_composition()),
            ("HeartConfig", HeartConfig::primitive_composition()),
            ("Heart", Heart::primitive_composition()),
            ("PumpResult", PumpResult::primitive_composition()),
            ("Vessel", Vessel::primitive_composition()),
            ("VascularBed", VascularBed::primitive_composition()),
            (
                "CapillaryExchange",
                CapillaryExchange::primitive_composition(),
            ),
            ("RedCell", RedCell::primitive_composition()),
            ("WhiteCell", WhiteCell::primitive_composition()),
            ("Platelet", Platelet::primitive_composition()),
            ("Plasma", Plasma::primitive_composition()),
            ("Baroreceptor", Baroreceptor::primitive_composition()),
            ("PressureReading", PressureReading::primitive_composition()),
            ("Pathology", Pathology::primitive_composition()),
            ("Diagnosis", Diagnosis::primitive_composition()),
        ];

        for (name, comp) in &compositions {
            assert!(!comp.primitives.is_empty(), "{} has empty primitives", name);
            assert!(
                comp.dominant.is_some(),
                "{} has no dominant primitive",
                name
            );
            assert!(comp.confidence > 0.0, "{} has zero confidence", name);
        }

        assert!(
            compositions.len() >= 25,
            "Expected at least 25 GroundsTo impls"
        );
    }

    #[test]
    fn heart_grounds_to_causality() {
        let comp = Heart::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn vessel_grounds_to_boundary() {
        let comp = Vessel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn pathology_grounds_to_irreversibility() {
        let comp = Pathology::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
    }
}
