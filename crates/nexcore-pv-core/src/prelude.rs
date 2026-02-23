//! # Prelude
//!
//! Convenient re-exports for `use nexcore_pv_core::prelude::*`.
//!
//! Imports the most commonly needed types from pv-core in a single statement.
//! This is intentionally curated — not everything is re-exported, only the
//! essential types that form the everyday PV computation surface.
//!
//! ## Coverage
//!
//! | Category | Types |
//! |----------|-------|
//! | Signal detection | `ContingencyTable`, `SignalResult`, `CompleteSignalResult`, `SafeSignalDetector` |
//! | Thresholds | `SignalCriteria`, `ThresholdRegistry` |
//! | ICSR | `Icsr`, `IcsrBuilder`, `CaseId`, `Patient`, `Drug`, `Reaction`, `Seriousness` |
//! | Safety | `SafetyMargin` |
//! | Causality | `CausalityLevel`, `Pharmacovigilance`, `PvCycleResult`, `AssessmentResult`, `DetectionResult` |
//! | Classification | `SeverityAssessment`, `SeverityLevel` |
//! | Hierarchy | `SafetyLevel`, `ToVLevel` |
//! | Grounding | `GroundsTo`, `LexPrimitiva`, `PrimitiveComposition`, `Tier` |

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL DETECTION
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::signals::safe_detector::SafeSignalDetector;
pub use crate::types::{CompleteSignalResult, ContingencyTable, SignalResult};

// ═══════════════════════════════════════════════════════════════════════════
// THRESHOLDS
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::thresholds::{SignalCriteria, ThresholdRegistry};

// ═══════════════════════════════════════════════════════════════════════════
// ICSR
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::icsr::{CaseId, Drug, Icsr, IcsrBuilder, Patient, Reaction, Seriousness};

// ═══════════════════════════════════════════════════════════════════════════
// SAFETY MARGIN
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::SafetyMargin;

// ═══════════════════════════════════════════════════════════════════════════
// CAUSALITY & DEFINITION
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::definition::{
    AssessmentResult, CausalityLevel, DetectionResult, Pharmacovigilance, PvCycleResult,
};

// ═══════════════════════════════════════════════════════════════════════════
// CLASSIFICATION
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::classification::{SeverityAssessment, SeverityLevel};

// ═══════════════════════════════════════════════════════════════════════════
// HIERARCHY
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::hierarchy::{SafetyLevel, ToVLevel};

// ═══════════════════════════════════════════════════════════════════════════
// GROUNDING (T1 Primitives)
// ═══════════════════════════════════════════════════════════════════════════

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
pub use nexcore_lex_primitiva::tier::Tier;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_imports_signal_types() {
        // Verify ContingencyTable is accessible via prelude
        let table = ContingencyTable {
            a: 10,
            b: 20,
            c: 30,
            d: 1000,
        };
        assert_eq!(table.a, 10);
    }

    #[test]
    fn prelude_imports_case_id() {
        let id = CaseId::new("TEST-001");
        assert_eq!(id.as_str(), "TEST-001");
    }

    #[test]
    fn prelude_imports_primitives() {
        assert_eq!(LexPrimitiva::Causality.symbol(), "\u{2192}");
        assert!(matches!(Tier::T1Universal, Tier::T1Universal));
    }

    #[test]
    fn prelude_imports_safety_margin() {
        let margin = SafetyMargin::calculate(1.5, 0.8, -0.3, 1.2, 10);
        // weighted_distance=0.20 → "Safe (Low Margin)" (below 0.5 threshold for Robustly Safe)
        assert!(margin.distance > 0.0);
        assert_eq!(margin.interpretation, "Safe (Low Margin)");
    }

    #[test]
    fn prelude_imports_severity() {
        assert!(matches!(
            SeverityLevel::Mild1,
            SeverityLevel::Mild1
        ));
    }

    #[test]
    fn prelude_imports_hierarchy() {
        assert!(matches!(SafetyLevel::Molecular, SafetyLevel::Molecular));
        assert!(matches!(ToVLevel::Clinical, ToVLevel::Clinical));
    }
}
