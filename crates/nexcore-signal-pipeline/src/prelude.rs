//! # Prelude
//!
//! Convenient re-exports for `use nexcore_signal_pipeline::prelude::*`.
//!
//! ## T1 Primitive: Mapping (mu)
//!
//! A prelude is a pure mapping from scattered paths to a single import surface.
//! It transforms `N` individual `use` statements into one glob import.

// ---- Core pipeline types ----

/// Domain types: the building blocks of every signal detection workflow.
pub use crate::core::{
    Alert, AlertState, ContingencyTable, DetectionResult, DrugEventPair, NormalizedEvent,
    RawReport, ReportSource, SignalStrength, ThresholdConfig, ValidationCheck, ValidationReport,
};

/// Signal metric newtypes: no naked f64.
pub use crate::core::{ChiSquare, ConfidenceInterval, Ebgm, Ic, Prr, Ror};

/// Error type.
pub use crate::core::SignalError;

// ---- Pipeline orchestration ----

/// Full pipeline coordinator.
pub use crate::orchestrate::Pipeline;

/// All-in-one signal metrics and compute function.
pub use crate::stats::{SignalMetrics, compute_all};

// ---- Relay fidelity ----

/// Pre-built relay chains for pipeline fidelity measurement.
pub use crate::relay::{core_detection_chain, pv_pipeline_chain};

// ---- Grounding (Lex Primitiva) ----

/// `GroundsTo` trait for primitive composition declarations.
pub use nexcore_lex_primitiva::grounding::GroundsTo;

/// The 16-variant primitive enum and its composition type.
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

/// Tier classification (T1, T2-P, T2-C, T3).
pub use nexcore_lex_primitiva::tier::Tier;

// ---- Structural modules ----

/// Cross-domain transfer mappings.
pub use crate::transfer::{
    TransferMapping, best_transfer, transfer_confidence, transfer_mappings, transfers_for_domain,
    transfers_for_type,
};

/// T1 primitive inventory for this crate.
pub use crate::primitives::{CratePrimitiveManifest, PipelineStagePrimitive, crate_manifest};

/// T2/T3 composite inventory for this crate.
pub use crate::composites::{CompositeDescriptor, composite_inventory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_imports_core_types() {
        // Verify key types are accessible through the prelude.
        let _pair = DrugEventPair::new("aspirin", "bleeding");
        let _table = ContingencyTable {
            a: 10,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let _strength = SignalStrength::None;
        let _state = AlertState::New;
        let _source = ReportSource::Faers;
    }

    #[test]
    fn prelude_imports_metric_newtypes() {
        let _prr = Prr(2.5);
        let _ror = Ror(3.0);
        let _ic = Ic(1.2);
        let _ebgm = Ebgm(2.0);
        let _chi = ChiSquare(4.0);
        let _ci = ConfidenceInterval {
            lower: 1.0,
            upper: 5.0,
            level: 0.95,
        };
    }

    #[test]
    fn prelude_imports_grounding_types() {
        // LexPrimitiva and Tier are accessible.
        let _q = LexPrimitiva::Quantity;
        let _t = Tier::T1Universal;
        let _comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence]);
    }

    #[test]
    fn prelude_imports_relay_chains() {
        let pv = pv_pipeline_chain();
        assert_eq!(pv.hop_count(), 7);
        let core = core_detection_chain();
        assert_eq!(core.hop_count(), 4);
    }

    #[test]
    fn prelude_imports_compute_all() {
        let table = ContingencyTable {
            a: 15,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let metrics = compute_all(&table);
        assert!(metrics.prr.is_some());
    }

    #[test]
    fn prelude_imports_transfer_module() {
        let mappings = transfer_mappings();
        assert!(!mappings.is_empty());
    }

    #[test]
    fn prelude_imports_primitives_module() {
        let manifest = crate_manifest();
        assert_eq!(manifest.crate_name, "nexcore-signal-pipeline");
    }

    #[test]
    fn prelude_imports_composites_module() {
        let inventory = composite_inventory();
        assert!(!inventory.is_empty());
    }
}
