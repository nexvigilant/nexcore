//! Agreement Checking (×-Product Validation)
//!
//! In Latin, adjectives must agree with their noun in case, number, AND gender —
//! a 3-dimensional Cartesian product check. Failure to agree is a grammatical error
//! caught at "compile time" (by the reader/speaker).
//!
//! We enforce agreement between components across multiple dimensions:
//! declension compatibility, case role consistency, and primitive grounding.
//!
//! ## Primitive Grounding
//! × Product (dominant) + κ Comparison + ∂ Boundary = T2-C

use crate::case::ComponentCase;
use crate::declension::Declension;
use serde::{Deserialize, Serialize};

/// Tier: T2-P — × Product
///
/// Dimensions across which agreement must be checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgreementDimension {
    /// Declension compatibility (dependency direction).
    Declension,
    /// Case role consistency (actor↔target relationship).
    Case,
    /// Primitive grounding compatibility.
    Grounding,
    /// Async boundary agreement (async↔sync mixing).
    AsyncBoundary,
}

impl AgreementDimension {
    /// Latin grammatical analog.
    pub fn latin_analog(&self) -> &'static str {
        match self {
            Self::Declension => "casus (case agreement)",
            Self::Case => "numerus (number agreement)",
            Self::Grounding => "genus (gender agreement)",
            Self::AsyncBoundary => "tempus (tense agreement)",
        }
    }

    /// All dimensions.
    pub fn all() -> &'static [AgreementDimension] {
        &[
            Self::Declension,
            Self::Case,
            Self::Grounding,
            Self::AsyncBoundary,
        ]
    }
}

/// Tier: T2-C — × + κ + ∂
///
/// Result of checking agreement between two components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgreementResult {
    /// The two components being checked.
    pub components: (String, String),
    /// Per-dimension results.
    pub dimensions: Vec<DimensionCheck>,
    /// Overall agreement (all dimensions pass).
    pub agrees: bool,
    /// Agreement ratio (passed / total).
    pub agreement_ratio: f64,
}

/// Result of checking one dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionCheck {
    /// Which dimension.
    pub dimension: AgreementDimension,
    /// Whether agreement holds.
    pub passes: bool,
    /// Explanation.
    pub reason: String,
}

/// Check declension agreement: can `from` depend on `to`?
pub fn check_declension_agreement(
    from_name: &str,
    from_decl: Declension,
    to_name: &str,
    to_decl: Declension,
) -> DimensionCheck {
    let passes = crate::declension::is_valid_dep(from_decl, to_decl);
    let reason = if passes {
        format!(
            "{} ({:?}) may depend on {} ({:?})",
            from_name, from_decl, to_name, to_decl
        )
    } else {
        format!(
            "VIOLATION: {} ({:?}) cannot depend on {} ({:?})",
            from_name, from_decl, to_name, to_decl
        )
    };

    DimensionCheck {
        dimension: AgreementDimension::Declension,
        passes,
        reason,
    }
}

/// Check case agreement: is the relationship between two cases valid?
///
/// Valid relationships:
/// - Nominative → Accusative (actor acts on target)
/// - Nominative → Dative (actor sends to beneficiary)
/// - Genitive → any (owner provides to anyone)
/// - Ablative → Nominative (instrument enables actor)
/// - Vocative → any (addressed component can interact with all)
pub fn check_case_agreement(
    from_name: &str,
    from_case: ComponentCase,
    to_name: &str,
    to_case: ComponentCase,
) -> DimensionCheck {
    let passes = match (from_case, to_case) {
        // Actor can act on targets and beneficiaries
        (ComponentCase::Nominative, ComponentCase::Accusative) => true,
        (ComponentCase::Nominative, ComponentCase::Dative) => true,
        // Owner provides to anyone
        (ComponentCase::Genitive, _) => true,
        // Instrument enables actors
        (ComponentCase::Ablative, ComponentCase::Nominative) => true,
        (ComponentCase::Ablative, ComponentCase::Accusative) => true,
        // Addressed can interact with anything
        (ComponentCase::Vocative, _) => true,
        // Location is accessed by anyone
        (_, ComponentCase::Locative) => true,
        // Same case = peer relationship (valid)
        (a, b) if a == b => true,
        // Everything else needs justification
        _ => false,
    };

    let reason = if passes {
        format!(
            "{} ({:?}) -> {} ({:?}): valid relationship",
            from_name, from_case, to_name, to_case
        )
    } else {
        format!(
            "DISAGREEMENT: {} ({:?}) -> {} ({:?}): unusual relationship",
            from_name, from_case, to_name, to_case
        )
    };

    DimensionCheck {
        dimension: AgreementDimension::Case,
        passes,
        reason,
    }
}

/// Check async boundary agreement.
///
/// Async code calling sync is fine. Sync code calling async is a violation
/// (requires runtime, potential deadlock).
pub fn check_async_agreement(
    from_name: &str,
    from_decl: Declension,
    to_name: &str,
    to_decl: Declension,
) -> DimensionCheck {
    let from_async = from_decl.async_permitted();
    let to_async = to_decl.async_permitted();

    let passes = match (from_async, to_async) {
        (true, true) => true,   // async → async: fine
        (true, false) => true,  // async → sync: fine
        (false, false) => true, // sync → sync: fine
        (false, true) => false, // sync → async: VIOLATION
    };

    let reason = if passes {
        format!(
            "{} (async={}) -> {} (async={}): compatible",
            from_name, from_async, to_name, to_async
        )
    } else {
        format!(
            "VIOLATION: {} (sync) depends on {} (async) — sync calling async",
            from_name, to_name
        )
    };

    DimensionCheck {
        dimension: AgreementDimension::AsyncBoundary,
        passes,
        reason,
    }
}

/// Full multi-dimensional agreement check between two components.
pub fn check_agreement(
    from_name: &str,
    from_decl: Declension,
    from_case: ComponentCase,
    to_name: &str,
    to_decl: Declension,
    to_case: ComponentCase,
) -> AgreementResult {
    let dimensions = vec![
        check_declension_agreement(from_name, from_decl, to_name, to_decl),
        check_case_agreement(from_name, from_case, to_name, to_case),
        check_async_agreement(from_name, from_decl, to_name, to_decl),
    ];

    let passed = dimensions.iter().filter(|d| d.passes).count();
    let total = dimensions.len();
    let agrees = dimensions.iter().all(|d| d.passes);

    AgreementResult {
        components: (from_name.to_string(), to_name.to_string()),
        dimensions,
        agrees,
        agreement_ratio: passed as f64 / total as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_declension_agreement() {
        let check = check_declension_agreement(
            "nexcore-vigilance",
            Declension::Second,
            "nexcore-primitives",
            Declension::First,
        );
        assert!(check.passes);
    }

    #[test]
    fn test_invalid_declension_agreement() {
        let check = check_declension_agreement(
            "nexcore-primitives",
            Declension::First,
            "nexcore-vigilance",
            Declension::Second,
        );
        assert!(!check.passes);
    }

    #[test]
    fn test_nominative_accusative_agreement() {
        let check = check_case_agreement(
            "nexcore-friday",
            ComponentCase::Nominative,
            "nexcore-brain",
            ComponentCase::Accusative,
        );
        assert!(check.passes);
    }

    #[test]
    fn test_genitive_provides_to_any() {
        let check = check_case_agreement(
            "nexcore-config",
            ComponentCase::Genitive,
            "nexcore-mcp",
            ComponentCase::Vocative,
        );
        assert!(check.passes);
    }

    #[test]
    fn test_async_sync_violation() {
        let check = check_async_agreement(
            "nexcore-primitives",
            Declension::First, // sync
            "nexcore-friday",
            Declension::Third, // async
        );
        assert!(!check.passes);
    }

    #[test]
    fn test_async_calling_sync_ok() {
        let check = check_async_agreement(
            "nexcore-friday",
            Declension::Third, // async
            "nexcore-primitives",
            Declension::First, // sync
        );
        assert!(check.passes);
    }

    #[test]
    fn test_full_agreement_pass() {
        let result = check_agreement(
            "nexcore-mcp",
            Declension::Fourth,
            ComponentCase::Vocative,
            "nexcore-vigilance",
            Declension::Second,
            ComponentCase::Accusative,
        );
        assert!(result.agrees);
        assert!((result.agreement_ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_full_agreement_fail() {
        let result = check_agreement(
            "nexcore-primitives",
            Declension::First,
            ComponentCase::Genitive,
            "nexcore-brain",
            Declension::Third,
            ComponentCase::Accusative,
        );
        // Declension: First cannot depend on Third — FAIL
        assert!(!result.agrees);
    }

    #[test]
    fn test_peer_case_agreement() {
        let check = check_case_agreement(
            "crate-a",
            ComponentCase::Accusative,
            "crate-b",
            ComponentCase::Accusative,
        );
        assert!(check.passes);
    }

    #[test]
    fn test_locative_accessible_by_all() {
        let check = check_case_agreement(
            "crate-a",
            ComponentCase::Nominative,
            "nexcore-cloud",
            ComponentCase::Locative,
        );
        assert!(check.passes);
    }
}
