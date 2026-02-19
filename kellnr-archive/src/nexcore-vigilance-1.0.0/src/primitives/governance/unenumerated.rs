//! # Unenumerated Rights (Bill of Rights)
//!
//! Implementation of Amendment IX: The enumeration of certain rights
//! shall not be construed to deny or disparage others retained by the Axioms.

use serde::{Deserialize, Serialize};

/// T3: UnenumeratedRight — A right not explicitly listed in the Constitution
/// but retained by the axioms (Lex Primitiva).
///
/// The system acknowledges that the 10 amendments are not exhaustive.
/// Any behavior grounded to T1 primitives has implicit constitutional standing.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnenumeratedRight {
    /// Description of the right being claimed
    pub right_description: String,
    /// T1 primitives this right grounds to
    pub grounding_primitives: Vec<String>,
    /// Whether the right conflicts with any enumerated right
    pub conflicts_with_enumerated: bool,
}

impl UnenumeratedRight {
    /// An unenumerated right is valid if it grounds to at least one T1 primitive
    /// and does not conflict with enumerated rights.
    pub fn is_valid(&self) -> bool {
        !self.grounding_primitives.is_empty() && !self.conflicts_with_enumerated
    }

    /// The tier of the right based on primitive count.
    pub fn tier(&self) -> &'static str {
        match self.grounding_primitives.len() {
            0 => "Invalid",
            1 => "T1-Universal",
            2..=3 => "T2-P",
            4..=5 => "T2-C",
            _ => "T3",
        }
    }
}

/// T3: AxiomRetention — The principle that axioms retain all rights
/// not explicitly delegated.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomRetention {
    /// The 16 T1 axioms
    pub axiom_count: u8,
    /// Rights explicitly enumerated (amendments)
    pub enumerated_rights: u8,
    /// Rights retained by axioms (implicit)
    pub retained_rights_count: u32,
}

impl AxiomRetention {
    /// The Lex Primitiva axiom count.
    pub const LEX_PRIMITIVA_COUNT: u8 = 16;
    /// The Bill of Rights amendment count.
    pub const AMENDMENT_COUNT: u8 = 10;

    /// Axioms always retain more rights than are enumerated.
    pub fn axioms_retain_more(&self) -> bool {
        self.retained_rights_count > self.enumerated_rights as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_unenumerated_right() {
        let right = UnenumeratedRight {
            right_description: "Right to cache computed results".to_string(),
            grounding_primitives: vec!["π".to_string(), "ς".to_string()],
            conflicts_with_enumerated: false,
        };
        assert!(right.is_valid());
        assert_eq!(right.tier(), "T2-P");
    }

    #[test]
    fn unenumerated_right_conflicting() {
        let right = UnenumeratedRight {
            right_description: "Right to bypass validation".to_string(),
            grounding_primitives: vec!["σ".to_string()],
            conflicts_with_enumerated: true, // Conflicts with Amendment V
        };
        assert!(!right.is_valid());
    }

    #[test]
    fn unenumerated_right_no_grounding() {
        let right = UnenumeratedRight {
            right_description: "Baseless claim".to_string(),
            grounding_primitives: vec![],
            conflicts_with_enumerated: false,
        };
        assert!(!right.is_valid());
        assert_eq!(right.tier(), "Invalid");
    }

    #[test]
    fn tier_classification() {
        let t1 = UnenumeratedRight {
            right_description: "Single primitive".to_string(),
            grounding_primitives: vec!["σ".to_string()],
            conflicts_with_enumerated: false,
        };
        assert_eq!(t1.tier(), "T1-Universal");

        let t3 = UnenumeratedRight {
            right_description: "Complex right".to_string(),
            grounding_primitives: vec![
                "σ".to_string(),
                "μ".to_string(),
                "ς".to_string(),
                "ρ".to_string(),
                "∂".to_string(),
                "κ".to_string(),
            ],
            conflicts_with_enumerated: false,
        };
        assert_eq!(t3.tier(), "T3");
    }

    #[test]
    fn axiom_retention() {
        let retention = AxiomRetention {
            axiom_count: AxiomRetention::LEX_PRIMITIVA_COUNT,
            enumerated_rights: AxiomRetention::AMENDMENT_COUNT,
            retained_rights_count: 1000,
        };
        assert!(retention.axioms_retain_more());
    }
}
