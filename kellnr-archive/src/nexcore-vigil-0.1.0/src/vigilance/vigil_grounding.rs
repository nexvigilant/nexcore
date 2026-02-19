//! # GroundsTo implementations for vigilance subsystem types
//!
//! Every public type grounds to the Lex Primitiva via its composition
//! of the 16 T1 symbols. The vigilance formula π(∂·ν)|∝ maps directly:
//!
//! - π = Persistence (Ledger)
//! - ∂ = Boundary (Gate)
//! - ν = Frequency (Watcher)
//! - ∝ = Irreversibility (Consequences)

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::vigilance::boundary::{BoundaryGate, BoundarySpec, BoundaryViolation, ThresholdCheck};
use crate::vigilance::consequence::{
    ConsequenceOutcome, ConsequencePipeline, ConsequenceReceipt, EscalationLevel,
    NotifyConsequence, ShellConsequence, WebhookConsequence,
};
use crate::vigilance::daemon::{ShutdownHandle, VigilDaemon, VigilHealth, VigilStats};
use crate::vigilance::error::VigilError;
use crate::vigilance::event::{EventId, EventKind, EventSeverity, WatchEvent};
use crate::vigilance::ledger::{LedgerEntry, LedgerEntryType, VigilanceLedger};

// ---------------------------------------------------------------------------
// T1/T2-P: Primitive types
// ---------------------------------------------------------------------------

/// EventId: T1 (N)
impl GroundsTo for EventId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

/// EventKind: T2-P (Σ)
impl GroundsTo for EventKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// EventSeverity: T2-P (κ + N)
impl GroundsTo for EventSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// EscalationLevel: T2-P (κ + ∝)
impl GroundsTo for EscalationLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// LedgerEntryType: T2-P (Σ + π)
impl GroundsTo for LedgerEntryType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Persistence])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// ConsequenceOutcome: T2-P (Σ + ∝)
impl GroundsTo for ConsequenceOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Irreversibility])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// ShellConsequence: T2-C (∝ + → + ∂)
impl GroundsTo for ShellConsequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // ∝ — shell commands are irreversible
            LexPrimitiva::Causality,       // → — violation causes command
            LexPrimitiva::Boundary,        // ∂ — timeout boundary
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.85)
    }
}

/// WebhookConsequence: T2-C (∝ + → + λ)
impl GroundsTo for WebhookConsequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // ∝ — sent HTTP cannot be unsent
            LexPrimitiva::Causality,       // → — violation causes webhook
            LexPrimitiva::Location,        // λ — URL endpoint
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.85)
    }
}

/// NotifyConsequence: T2-P (∝ + ∃)
impl GroundsTo for NotifyConsequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // ∝ — notification sent
            LexPrimitiva::Existence,       // ∃ — notification file exists
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T2-C: Composed types
// ---------------------------------------------------------------------------

/// WatchEvent: T2-C (ν + σ + κ + λ)
impl GroundsTo for WatchEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // ν — observation frequency
            LexPrimitiva::Sequence,   // σ — temporal ordering
            LexPrimitiva::Comparison, // κ — severity
            LexPrimitiva::Location,   // λ — source identity
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

/// ThresholdCheck: T2-C (∂ + κ + ν + N)
impl GroundsTo for ThresholdCheck {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — boundary condition
            LexPrimitiva::Comparison, // κ — threshold comparison
            LexPrimitiva::Frequency,  // ν — window frequency
            LexPrimitiva::Quantity,   // N — count threshold
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// BoundarySpec: T2-C (∂ + κ + ν + λ)
impl GroundsTo for BoundarySpec {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — the boundary itself
            LexPrimitiva::Comparison, // κ — threshold
            LexPrimitiva::Frequency,  // ν — cooldown
            LexPrimitiva::Location,   // λ — source filter
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// BoundaryViolation: T2-C (∂ + κ + ν + N)
impl GroundsTo for BoundaryViolation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — violated boundary
            LexPrimitiva::Comparison, // κ — severity assessment
            LexPrimitiva::Frequency,  // ν — violation frequency
            LexPrimitiva::Quantity,   // N — violation count
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// ConsequenceReceipt: T2-C (∃ + ∝ + π + →)
impl GroundsTo for ConsequenceReceipt {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,       // ∃ — proof of execution
            LexPrimitiva::Irreversibility, // ∝ — consequence applied
            LexPrimitiva::Persistence,     // π — ledger-linked
            LexPrimitiva::Causality,       // → — violation caused consequence
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
    }
}

/// LedgerEntry: T2-C (π + σ + ∝ + ∂)
impl GroundsTo for LedgerEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,     // π — immutable record
            LexPrimitiva::Sequence,        // σ — ordered chain
            LexPrimitiva::Irreversibility, // ∝ — hash chain
            LexPrimitiva::Boundary,        // ∂ — integrity boundary
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Domain types
// ---------------------------------------------------------------------------

/// BoundaryGate: T3 (∂ + κ + ν + σ + ς + N)
impl GroundsTo for BoundaryGate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — boundary evaluation
            LexPrimitiva::Comparison, // κ — threshold comparison
            LexPrimitiva::Frequency,  // ν — sliding windows
            LexPrimitiva::Sequence,   // σ — evaluation ordering
            LexPrimitiva::State,      // ς — window state
            LexPrimitiva::Quantity,   // N — event counts
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// VigilanceLedger: T3 (π + ∝ + σ + ∂ + N + ∃)
impl GroundsTo for VigilanceLedger {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,     // π — append-only store
            LexPrimitiva::Irreversibility, // ∝ — hash chain = irreversible
            LexPrimitiva::Sequence,        // σ — sequence numbers
            LexPrimitiva::Boundary,        // ∂ — integrity boundary
            LexPrimitiva::Quantity,        // N — sequence counter
            LexPrimitiva::Existence,       // ∃ — existence proof via hash
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

/// ConsequencePipeline: T3 (∝ + σ + ∂ + → + π + ∃)
impl GroundsTo for ConsequencePipeline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // ∝ — consequences are irreversible
            LexPrimitiva::Sequence,        // σ — escalation order
            LexPrimitiva::Boundary,        // ∂ — escalation thresholds
            LexPrimitiva::Causality,       // → — violation causes consequence
            LexPrimitiva::Persistence,     // π — ledger recording
            LexPrimitiva::Existence,       // ∃ — receipt existence proof
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.80)
    }
}

/// VigilDaemon: T3 (π + ∂ + ν + ∝ + σ + ς + → + ∃)
///
/// The daemon IS the vigilance formula: π(∂·ν)|∝
impl GroundsTo for VigilDaemon {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,     // π — ledger
            LexPrimitiva::Boundary,        // ∂ — gate
            LexPrimitiva::Frequency,       // ν — watcher
            LexPrimitiva::Irreversibility, // ∝ — consequences
            LexPrimitiva::Sequence,        // σ — event pipeline
            LexPrimitiva::State,           // ς — daemon state
            LexPrimitiva::Causality,       // → — event → action
            LexPrimitiva::Existence,       // ∃ — health proof
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.75)
    }
}

/// ShutdownHandle: T2-P (∂ + ς)
impl GroundsTo for ShutdownHandle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// VigilHealth: T2-C (∃ + N + ς + ∂)
impl GroundsTo for VigilHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ — liveness proof
            LexPrimitiva::Quantity,  // N — counters
            LexPrimitiva::State,     // ς — running/stopped
            LexPrimitiva::Boundary,  // ∂ — chain verification
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
    }
}

/// VigilStats: T2-C (N + σ + π)
impl GroundsTo for VigilStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,    // N — counters
            LexPrimitiva::Sequence,    // σ — temporal
            LexPrimitiva::Persistence, // π — head hash
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// VigilError: T2-C (∂ + → + ∅ + Σ)
impl GroundsTo for VigilError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — constraint violations
            LexPrimitiva::Causality, // → — operation failures
            LexPrimitiva::Void,      // ∅ — missing/unknown
            LexPrimitiva::Sum,       // Σ — error variants
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn event_id_is_t1() {
        assert_eq!(EventId::tier(), Tier::T1Universal);
    }

    #[test]
    fn event_severity_is_comparison_dominant() {
        assert_eq!(
            EventSeverity::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn watch_event_is_frequency_dominant() {
        assert_eq!(
            WatchEvent::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn boundary_spec_is_boundary_dominant() {
        assert_eq!(
            BoundarySpec::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn boundary_gate_is_t3() {
        assert_eq!(BoundaryGate::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn vigilance_ledger_is_persistence_dominant() {
        assert_eq!(
            VigilanceLedger::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn consequence_pipeline_is_irreversibility_dominant() {
        assert_eq!(
            ConsequencePipeline::dominant_primitive(),
            Some(LexPrimitiva::Irreversibility)
        );
    }

    #[test]
    fn vigil_daemon_is_t3() {
        assert_eq!(VigilDaemon::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            VigilDaemon::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn consequence_receipt_is_existence_dominant() {
        assert_eq!(
            ConsequenceReceipt::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn vigil_error_is_boundary_dominant() {
        assert_eq!(
            VigilError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn ledger_entry_is_persistence_dominant() {
        assert_eq!(
            LedgerEntry::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn escalation_level_is_comparison_dominant() {
        assert_eq!(
            EscalationLevel::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn vigil_health_is_existence_dominant() {
        assert_eq!(
            VigilHealth::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn vigil_stats_is_quantity_dominant() {
        assert_eq!(
            VigilStats::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }
}
