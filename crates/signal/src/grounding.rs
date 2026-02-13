//! # GroundsTo implementations for signal crate types
//!
//! Connects the PV signal detection pipeline types to the Lex Primitiva type system.
//!
//! ## Pipeline Primitive Map
//!
//! The signal crate implements a 10-stage PV signal detection pipeline:
//! `ingest -> normalize -> detect -> threshold -> validate -> stats -> store -> alert -> report -> orchestrate`
//!
//! Each stage has a dominant T1 primitive reflecting its core operation:
//!
//! | Stage | Dominant | Rationale |
//! |-------|----------|-----------|
//! | Ingest | sigma (Sequence) | Ordered stream of raw reports |
//! | Normalize | mu (Mapping) | RawReport -> NormalizedEvent transformation |
//! | Detect | N (Quantity) | Contingency table arithmetic |
//! | Threshold | partial (Boundary) | Pass/fail boundary evaluation |
//! | Validate | kappa (Comparison) | Metric-vs-threshold comparison |
//! | Stats | mu (Mapping) | ContingencyTable -> SignalMetrics |
//! | Store | pi (Persistence) | Durable state for results/alerts |
//! | Alert | varsigma (State) | State machine lifecycle |
//! | Report | mu (Mapping) | Results -> formatted output |
//! | Orchestrate | sigma (Sequence) | Sequential pipeline coordination |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::alert::{AlertGenerator, AlertTransitions};
use crate::core::{
    Alert, AlertState, ChiSquare, ConfidenceInterval, ContingencyTable, DetectionResult,
    DrugEventPair, Ebgm, Ic, NormalizedEvent, Prr, RawReport, ReportSource, Ror, SignalError,
    SignalStrength, ThresholdConfig, ValidationCheck, ValidationReport,
};
use crate::detect::TableDetector;
use crate::ingest::{CsvIngestor, JsonIngestor};
use crate::normalize::{BasicNormalizer, SynonymNormalizer};
use crate::report::{JsonReporter, TableReporter};
use crate::stats::SignalMetrics;
use crate::store::{JsonFileStore, MemoryStore};
use crate::threshold::{CompositeThreshold, EvansThreshold};
use crate::validate::StandardValidator;

// ===========================================================================
// T1-Universal: Single primitive newtypes
// ===========================================================================

/// Prr: T1 (N), dominant N
///
/// Proportional Reporting Ratio -- a single numeric measurement.
/// Pure quantity: wraps f64 with no additional structure.
impl GroundsTo for Prr {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- ratio measurement
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// Ror: T1 (N), dominant N
///
/// Reporting Odds Ratio -- a single numeric measurement.
impl GroundsTo for Ror {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- odds ratio measurement
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// Ic: T1 (N), dominant N
///
/// Information Component -- a single Bayesian log2 measurement.
impl GroundsTo for Ic {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- information measure
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// Ebgm: T1 (N), dominant N
///
/// Empirical Bayesian Geometric Mean -- single numeric estimate.
impl GroundsTo for Ebgm {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- shrinkage estimate
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// ChiSquare: T1 (N), dominant N
///
/// Chi-square test statistic -- a single numeric result.
impl GroundsTo for ChiSquare {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- test statistic value
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

// ===========================================================================
// T2-Primitive: 2-3 unique primitives
// ===========================================================================

/// SignalStrength: T2-P (Sigma + kappa), dominant Sigma
///
/// Five-variant enum classifying signal intensity: None|Weak|Moderate|Strong|Critical.
/// Sum-dominant: the type IS a categorical alternation of strength levels.
/// Comparison is secondary: variants have ordinal ordering.
impl GroundsTo for SignalStrength {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- variant alternation
            LexPrimitiva::Comparison, // kappa -- ordinal ordering of strength
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// ReportSource: T2-P (Sigma + lambda), dominant Sigma
///
/// Seven-variant enum classifying data origin: Faers|Eudravigilance|Vigibase|...
/// Sum-dominant: the type IS a categorical alternation of sources.
/// Location is secondary: each variant represents a distinct data source location.
impl GroundsTo for ReportSource {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- variant alternation
            LexPrimitiva::Location, // lambda -- data source origin
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// AlertState: T2-P (varsigma + Sigma), dominant varsigma
///
/// Six-variant lifecycle enum: New|UnderReview|Confirmed|Escalated|Closed|FalsePositive.
/// State-dominant: the type IS a finite state in a lifecycle machine with
/// defined transitions. Sum is secondary: the enum form represents alternation.
impl GroundsTo for AlertState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- lifecycle state
            LexPrimitiva::Sum,   // Sigma -- variant alternation
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// SignalError: T2-P (Sigma + partial), dominant Sigma
///
/// Eight-variant error enum classifying pipeline failures by stage.
/// Sum-dominant: the type IS a categorical alternation of error kinds.
/// Boundary is secondary: each variant marks a failure boundary.
impl GroundsTo for SignalError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- variant alternation
            LexPrimitiva::Boundary, // partial -- failure boundary
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// ConfidenceInterval: T2-P (N + partial), dominant N
///
/// Lower/upper bounds with confidence level. Quantity-dominant: the type
/// IS a numeric range. Boundary is secondary: the interval defines where
/// the true value is expected to lie.
impl GroundsTo for ConfidenceInterval {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric bounds and level
            LexPrimitiva::Boundary, // partial -- interval bounds
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// DrugEventPair: T2-P (x + exists), dominant x
///
/// A (drug, event) tuple identifier. Product-dominant: the type IS a
/// conjunction of two independent identifiers. Existence is secondary:
/// the pair asserts co-occurrence of drug exposure and adverse event.
impl GroundsTo for DrugEventPair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,   // x -- conjunction of drug and event
            LexPrimitiva::Existence, // exists -- asserts co-occurrence
        ])
        .with_dominant(LexPrimitiva::Product, 0.90)
    }
}

/// ContingencyTable: T2-P (N + x), dominant N
///
/// 2x2 table with cells a, b, c, d for disproportionality analysis.
/// Quantity-dominant: the type IS four numeric counts.
/// Product is secondary: the table is a product of two binary dimensions
/// (drug+/- x event+/-).
impl GroundsTo for ContingencyTable {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- four cell counts
            LexPrimitiva::Product,  // x -- two binary dimensions
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// ValidationCheck: T2-P (kappa + partial), dominant kappa
///
/// A single pass/fail check with name and message.
/// Comparison-dominant: the check IS a predicate evaluation.
/// Boundary is secondary: pass/fail is a binary boundary.
impl GroundsTo for ValidationCheck {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- predicate evaluation
            LexPrimitiva::Boundary,   // partial -- pass/fail boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// ThresholdConfig: T2-P (partial + N), dominant partial
///
/// Configurable Evans/strict/sensitive threshold values.
/// Boundary-dominant: the config IS a set of pass/fail boundaries.
/// Quantity is secondary: the boundaries are expressed as numeric values.
impl GroundsTo for ThresholdConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- threshold boundaries
            LexPrimitiva::Quantity, // N -- numeric threshold values
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ===========================================================================
// T2-Composite: 4-5 unique primitives
// ===========================================================================

/// NormalizedEvent: T2-C (mu + x + exists + sigma + lambda), dominant mu
///
/// A standardized event after drug/term normalization. Mapping-dominant:
/// the type IS the result of a raw-to-canonical transformation.
/// Product: drug x event conjunction. Existence: asserts the event occurred.
/// Sequence: position in the pipeline stream. Location: data source origin.
impl GroundsTo for NormalizedEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- normalized transformation result
            LexPrimitiva::Product,   // x -- drug x event conjunction
            LexPrimitiva::Existence, // exists -- event occurrence assertion
            LexPrimitiva::Sequence,  // sigma -- pipeline stream position
            LexPrimitiva::Location,  // lambda -- data source origin
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// ValidationReport: T2-C (kappa + sigma + partial + x), dominant kappa
///
/// Aggregate validation result with individual checks.
/// Comparison-dominant: the report IS a compound predicate evaluation.
/// Sequence: ordered list of checks. Boundary: overall pass/fail.
/// Product: pair identity x check results.
impl GroundsTo for ValidationReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- compound predicate
            LexPrimitiva::Sequence,   // sigma -- ordered checks
            LexPrimitiva::Boundary,   // partial -- overall pass/fail
            LexPrimitiva::Product,    // x -- pair x checks conjunction
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// SignalMetrics: T2-C (N + mu + Sigma + kappa), dominant N
///
/// Aggregated disproportionality metrics (PRR, ROR, IC, EBGM, chi-sq, strength).
/// Quantity-dominant: the type IS a collection of numeric measurements.
/// Mapping: computed from ContingencyTable. Sum: strength classification.
/// Comparison: strength ordering.
impl GroundsTo for SignalMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric metric values
            LexPrimitiva::Mapping,    // mu -- computed from table
            LexPrimitiva::Sum,        // Sigma -- strength classification
            LexPrimitiva::Comparison, // kappa -- strength ordering
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ===========================================================================
// T3-DomainSpecific: 6+ unique primitives
// ===========================================================================

/// RawReport: T3 (sigma + x + exists + lambda + nu + varsigma), dominant sigma
///
/// A raw adverse event report before normalization.
/// Sequence-dominant: a report IS an ordered entry in a data stream.
/// Product: drug x event combination. Existence: report asserts occurrence.
/// Location: source database. Frequency: report_date temporal marker.
/// State: carries metadata payload.
impl GroundsTo for RawReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- stream entry ordering
            LexPrimitiva::Product,   // x -- drug x event combination
            LexPrimitiva::Existence, // exists -- occurrence assertion
            LexPrimitiva::Location,  // lambda -- data source
            LexPrimitiva::Frequency, // nu -- temporal marker (report_date)
            LexPrimitiva::State,     // varsigma -- metadata payload
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// DetectionResult: T3 (N + x + kappa + Sigma + nu + mu), dominant N
///
/// Full detection output for a drug-event pair. Quantity-dominant: the type
/// IS a collection of computed numeric metrics plus the source table.
/// Product: pair identity. Comparison: strength classification.
/// Sum: strength enum. Frequency: detection timestamp.
/// Mapping: computed from contingency table.
impl GroundsTo for DetectionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- metrics + table counts
            LexPrimitiva::Product,    // x -- drug x event pair
            LexPrimitiva::Comparison, // kappa -- strength classification
            LexPrimitiva::Sum,        // Sigma -- strength enum
            LexPrimitiva::Frequency,  // nu -- detected_at timestamp
            LexPrimitiva::Mapping,    // mu -- computed from table
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

/// Alert: T3 (varsigma + N + x + nu + pi + sigma), dominant varsigma
///
/// A signal alert with lifecycle state, detection data, and notes.
/// State-dominant: an alert IS a stateful entity in a lifecycle machine.
/// Quantity: embedded detection metrics. Product: pair identity.
/// Frequency: created_at/updated_at timestamps. Persistence: stored entity.
/// Sequence: ordered notes history.
impl GroundsTo for Alert {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- lifecycle state machine
            LexPrimitiva::Quantity,    // N -- embedded detection metrics
            LexPrimitiva::Product,     // x -- drug x event identity
            LexPrimitiva::Frequency,   // nu -- timestamps
            LexPrimitiva::Persistence, // pi -- stored/retrievable entity
            LexPrimitiva::Sequence,    // sigma -- ordered notes
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ===========================================================================
// Stage implementation types
// ===========================================================================

/// JsonIngestor: T2-P (sigma + mu), dominant sigma
///
/// Ingests reports from JSON strings. Sequence-dominant: ingestion IS
/// ordered stream production. Mapping: JSON -> RawReport transformation.
impl GroundsTo for JsonIngestor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered stream production
            LexPrimitiva::Mapping,  // mu -- JSON -> RawReport
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// CsvIngestor: T2-P (sigma + mu), dominant sigma
///
/// Ingests reports from CSV text. Same primitive composition as JsonIngestor:
/// sequence-dominant stream production with format mapping.
impl GroundsTo for CsvIngestor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered stream production
            LexPrimitiva::Mapping,  // mu -- CSV -> RawReport
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// BasicNormalizer: T2-P (mu + sigma), dominant mu
///
/// Lowercases and trims drug/event names. Mapping-dominant: normalization
/// IS a pure transformation. Sequence: iterates drug x event combinations.
impl GroundsTo for BasicNormalizer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- name standardization
            LexPrimitiva::Sequence, // sigma -- iterates combinations
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// SynonymNormalizer: T2-P (mu + varsigma + sigma), dominant mu
///
/// Normalizes drug names via a synonym dictionary. Mapping-dominant:
/// normalization IS a transformation. State: encapsulated synonym map.
/// Sequence: iterates drug x event combinations.
impl GroundsTo for SynonymNormalizer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- synonym lookup transformation
            LexPrimitiva::State,    // varsigma -- synonym dictionary
            LexPrimitiva::Sequence, // sigma -- iterates combinations
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// TableDetector: T2-P (N + mu), dominant N
///
/// Builds contingency tables and computes metrics from normalized events.
/// Quantity-dominant: detection IS counting and arithmetic.
/// Mapping: events -> contingency tables transformation.
impl GroundsTo for TableDetector {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- counting and arithmetic
            LexPrimitiva::Mapping,  // mu -- events -> tables
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// EvansThreshold: T2-P (partial + kappa + varsigma), dominant partial
///
/// Evans-criteria threshold evaluator. Boundary-dominant: threshold
/// evaluation IS pass/fail boundary logic. Comparison: metric vs config.
/// State: encapsulated threshold configuration.
impl GroundsTo for EvansThreshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- pass/fail boundary
            LexPrimitiva::Comparison, // kappa -- metric comparison
            LexPrimitiva::State,      // varsigma -- config state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// CompositeThreshold: T2-P (partial + rho + sigma), dominant partial
///
/// Multi-criteria threshold requiring ALL sub-thresholds to pass.
/// Boundary-dominant: composite IS a compound pass/fail gate.
/// Recursion: delegates to inner thresholds. Sequence: ordered evaluation.
impl GroundsTo for CompositeThreshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // partial -- compound pass/fail gate
            LexPrimitiva::Recursion, // rho -- delegates to sub-thresholds
            LexPrimitiva::Sequence,  // sigma -- ordered evaluation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// StandardValidator: T2-P (kappa + partial + varsigma), dominant kappa
///
/// Validates detection results against configurable checks.
/// Comparison-dominant: validation IS predicate evaluation.
/// Boundary: pass/fail outcomes. State: threshold configuration.
impl GroundsTo for StandardValidator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- predicate evaluation
            LexPrimitiva::Boundary,   // partial -- pass/fail outcomes
            LexPrimitiva::State,      // varsigma -- config state
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// MemoryStore: T2-P (pi + varsigma), dominant pi
///
/// In-memory store backed by HashMap. Persistence-dominant: the store
/// IS a persistence layer. State: mutable accumulated results.
impl GroundsTo for MemoryStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- persistence layer
            LexPrimitiva::State,       // varsigma -- mutable accumulation
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// JsonFileStore: T2-P (pi + varsigma + mu), dominant pi
///
/// JSON file-based store. Persistence-dominant: the store IS a durable
/// persistence layer. State: in-memory buffer. Mapping: serialize to JSON.
impl GroundsTo for JsonFileStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- durable file persistence
            LexPrimitiva::State,       // varsigma -- in-memory buffer
            LexPrimitiva::Mapping,     // mu -- serialize to JSON
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// AlertTransitions: T2-P (varsigma + partial), dominant varsigma
///
/// Manages alert state transitions. State-dominant: the manager IS a
/// state transition controller. Boundary: validates legal transitions.
impl GroundsTo for AlertTransitions {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- state transition controller
            LexPrimitiva::Boundary, // partial -- validates transitions
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// JsonReporter: T2-P (mu + sigma), dominant mu
///
/// Generates JSON summary reports. Mapping-dominant: reporting IS a
/// transformation from internal results to JSON output. Sequence: ordered
/// result serialization.
impl GroundsTo for JsonReporter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- results -> JSON output
            LexPrimitiva::Sequence, // sigma -- ordered serialization
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// TableReporter: T2-P (mu + sigma), dominant mu
///
/// Generates plain-text tabular reports. Same primitive composition as
/// JsonReporter: mapping-dominant transformation with sequenced output.
impl GroundsTo for TableReporter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- results -> text table output
            LexPrimitiva::Sequence, // sigma -- ordered formatting
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // -----------------------------------------------------------------------
    // T1 Universal newtypes -- single primitive
    // -----------------------------------------------------------------------

    #[test]
    fn prr_is_t1_quantity_dominant() {
        let comp = Prr::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(Prr::tier(), Tier::T1Universal);
    }

    #[test]
    fn ror_is_t1_quantity_dominant() {
        let comp = Ror::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(Ror::tier(), Tier::T1Universal);
    }

    #[test]
    fn ic_is_t1_quantity_dominant() {
        assert_eq!(Ic::tier(), Tier::T1Universal);
        assert_eq!(
            Ic::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn ebgm_is_t1_quantity_dominant() {
        assert_eq!(Ebgm::tier(), Tier::T1Universal);
        assert_eq!(
            Ebgm::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn chi_square_is_t1_quantity_dominant() {
        assert_eq!(ChiSquare::tier(), Tier::T1Universal);
        assert_eq!(
            ChiSquare::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    // -----------------------------------------------------------------------
    // T2-Primitive: 2-3 unique primitives
    // -----------------------------------------------------------------------

    #[test]
    fn signal_strength_is_t2p_sum_dominant() {
        let comp = SignalStrength::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(SignalStrength::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn report_source_is_t2p_sum_dominant() {
        let comp = ReportSource::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(ReportSource::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
    }

    #[test]
    fn alert_state_is_t2p_state_dominant() {
        let comp = AlertState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(AlertState::tier(), Tier::T2Primitive);
    }

    #[test]
    fn signal_error_is_t2p_sum_dominant() {
        let comp = SignalError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(SignalError::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn confidence_interval_is_t2p_quantity_dominant() {
        let comp = ConfidenceInterval::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(ConfidenceInterval::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn drug_event_pair_is_t2p_product_dominant() {
        let comp = DrugEventPair::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(DrugEventPair::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn contingency_table_is_t2p_quantity_dominant() {
        let comp = ContingencyTable::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(ContingencyTable::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
    }

    #[test]
    fn validation_check_is_t2p_comparison_dominant() {
        let comp = ValidationCheck::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(ValidationCheck::tier(), Tier::T2Primitive);
    }

    #[test]
    fn threshold_config_is_t2p_boundary_dominant() {
        let comp = ThresholdConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(ThresholdConfig::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    // -----------------------------------------------------------------------
    // T2-Composite: 4-5 unique primitives
    // -----------------------------------------------------------------------

    #[test]
    fn normalized_event_is_t2c_mapping_dominant() {
        let comp = NormalizedEvent::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(NormalizedEvent::tier(), Tier::T2Composite);
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn validation_report_is_t2c_comparison_dominant() {
        let comp = ValidationReport::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(ValidationReport::tier(), Tier::T2Composite);
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn signal_metrics_is_t2c_quantity_dominant() {
        let comp = SignalMetrics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(SignalMetrics::tier(), Tier::T2Composite);
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
    }

    // -----------------------------------------------------------------------
    // T3-DomainSpecific: 6+ unique primitives
    // -----------------------------------------------------------------------

    #[test]
    fn raw_report_is_t3_sequence_dominant() {
        let comp = RawReport::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(RawReport::tier(), Tier::T3DomainSpecific);
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    #[test]
    fn detection_result_is_t3_quantity_dominant() {
        let comp = DetectionResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(DetectionResult::tier(), Tier::T3DomainSpecific);
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
    }

    #[test]
    fn alert_is_t3_state_dominant() {
        let comp = Alert::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(Alert::tier(), Tier::T3DomainSpecific);
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
    }

    // -----------------------------------------------------------------------
    // Stage implementation types
    // -----------------------------------------------------------------------

    #[test]
    fn json_ingestor_is_t2p_sequence_dominant() {
        let comp = JsonIngestor::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(JsonIngestor::tier(), Tier::T2Primitive);
    }

    #[test]
    fn csv_ingestor_is_t2p_sequence_dominant() {
        assert_eq!(CsvIngestor::tier(), Tier::T2Primitive);
        assert_eq!(
            CsvIngestor::primitive_composition().dominant,
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn basic_normalizer_is_t2p_mapping_dominant() {
        let comp = BasicNormalizer::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(BasicNormalizer::tier(), Tier::T2Primitive);
    }

    #[test]
    fn synonym_normalizer_is_t2p_mapping_dominant() {
        let comp = SynonymNormalizer::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(SynonymNormalizer::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    #[test]
    fn table_detector_is_t2p_quantity_dominant() {
        let comp = TableDetector::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(TableDetector::tier(), Tier::T2Primitive);
    }

    #[test]
    fn evans_threshold_is_t2p_boundary_dominant() {
        let comp = EvansThreshold::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(EvansThreshold::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn composite_threshold_is_t2p_boundary_dominant() {
        let comp = CompositeThreshold::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(CompositeThreshold::tier(), Tier::T2Primitive);
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    #[test]
    fn standard_validator_is_t2p_comparison_dominant() {
        let comp = StandardValidator::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(StandardValidator::tier(), Tier::T2Primitive);
    }

    #[test]
    fn memory_store_is_t2p_persistence_dominant() {
        let comp = MemoryStore::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(MemoryStore::tier(), Tier::T2Primitive);
    }

    #[test]
    fn json_file_store_is_t2p_persistence_dominant() {
        let comp = JsonFileStore::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(JsonFileStore::tier(), Tier::T2Primitive);
    }

    #[test]
    fn alert_manager_is_t2p_state_dominant() {
        let comp = AlertTransitions::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(AlertTransitions::tier(), Tier::T2Primitive);
    }

    #[test]
    fn json_reporter_is_t2p_mapping_dominant() {
        let comp = JsonReporter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(JsonReporter::tier(), Tier::T2Primitive);
    }

    #[test]
    fn table_reporter_is_t2p_mapping_dominant() {
        let comp = TableReporter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(TableReporter::tier(), Tier::T2Primitive);
    }

    // -----------------------------------------------------------------------
    // Tier distribution coverage
    // -----------------------------------------------------------------------

    #[test]
    fn tier_distribution_is_balanced() {
        // 5 T1 types (metric newtypes)
        assert_eq!(Prr::tier(), Tier::T1Universal);
        assert_eq!(Ror::tier(), Tier::T1Universal);
        assert_eq!(Ic::tier(), Tier::T1Universal);
        assert_eq!(Ebgm::tier(), Tier::T1Universal);
        assert_eq!(ChiSquare::tier(), Tier::T1Universal);

        // 3 T2-C types
        assert_eq!(NormalizedEvent::tier(), Tier::T2Composite);
        assert_eq!(ValidationReport::tier(), Tier::T2Composite);
        assert_eq!(SignalMetrics::tier(), Tier::T2Composite);

        // 3 T3 types
        assert_eq!(RawReport::tier(), Tier::T3DomainSpecific);
        assert_eq!(DetectionResult::tier(), Tier::T3DomainSpecific);
        assert_eq!(Alert::tier(), Tier::T3DomainSpecific);
    }
}
