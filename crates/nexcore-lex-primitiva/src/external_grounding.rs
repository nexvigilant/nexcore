//! # External Grounding Authorities
//!
//! Addresses Red Team Attack F2 (Circular Roots) by grounding root primitives
//! to EXTERNAL authorities rather than internal graph structure.
//!
//! ## The Problem
//!
//! The original proof was circular:
//! - Step 1: We DEFINE derives_from relationships
//! - Step 2: We OBSERVE N and → have no dependencies
//! - Step 3: We CLAIM this proves they are fundamental
//!
//! This is petitio principii (begging the question).
//!
//! ## The Solution
//!
//! Ground each root to EXTERNAL peer-reviewed authorities:
//! - Quantity (N) → Peano axioms, IEEE 754, SI units
//! - Causality (→) → Pearl's do-calculus, Hume, Shannon
//!
//! ## Tier: T1-Universal

#![allow(
    dead_code,
    reason = "module contains proof infrastructure used in tests and doc examples"
)]

use crate::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An external authority that grounds a primitive independently of internal structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExternalAuthority {
    /// Name of the authority (e.g., "Peano Axioms")
    pub name: &'static str,
    /// Domain of the authority (e.g., "Mathematics", "Philosophy")
    pub domain: AuthorityDomain,
    /// Citation in Vancouver format
    pub citation: &'static str,
    /// Year of publication
    pub year: u16,
    /// Key claim that grounds the primitive
    pub grounding_claim: &'static str,
    /// DOI or persistent identifier if available
    pub doi: Option<&'static str>,
}

/// Domain classification for external authorities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AuthorityDomain {
    /// Pure mathematics (axioms, proofs)
    Mathematics,
    /// Philosophy (metaphysics, epistemology)
    Philosophy,
    /// Computer science (theory, standards)
    ComputerScience,
    /// Physics (measurement, units)
    Physics,
    /// Information theory
    InformationTheory,
    /// Engineering standards
    Engineering,
}

impl fmt::Display for AuthorityDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mathematics => write!(f, "Mathematics"),
            Self::Philosophy => write!(f, "Philosophy"),
            Self::ComputerScience => write!(f, "Computer Science"),
            Self::Physics => write!(f, "Physics"),
            Self::InformationTheory => write!(f, "Information Theory"),
            Self::Engineering => write!(f, "Engineering"),
        }
    }
}

/// External grounding for the Quantity (N) root primitive.
pub const QUANTITY_AUTHORITIES: [ExternalAuthority; 4] = [
    ExternalAuthority {
        name: "Peano Axioms",
        domain: AuthorityDomain::Mathematics,
        citation: "Peano G. Arithmetices principia: nova methodo exposita. Bocca; 1889.",
        year: 1889,
        grounding_claim: "Natural numbers exist with 0 as origin and successor function S(n)",
        doi: None,
    },
    ExternalAuthority {
        name: "IEEE 754 Floating-Point Standard",
        domain: AuthorityDomain::Engineering,
        citation: "IEEE. IEEE Standard for Floating-Point Arithmetic. IEEE Std 754-2019.",
        year: 2019,
        grounding_claim: "Numeric representation is fundamental to all computation",
        doi: Some("10.1109/IEEESTD.2019.8766229"),
    },
    ExternalAuthority {
        name: "SI Base Units",
        domain: AuthorityDomain::Physics,
        citation: "BIPM. The International System of Units (SI). 9th ed. 2019.",
        year: 2019,
        grounding_claim: "All physical measurement reduces to seven base quantities",
        doi: None,
    },
    ExternalAuthority {
        name: "Church Numerals",
        domain: AuthorityDomain::Mathematics,
        citation: "Church A. The calculi of lambda-conversion. Princeton University Press; 1941.",
        year: 1941,
        grounding_claim: "Numbers can be encoded in pure lambda calculus, proving their primitivity",
        doi: None,
    },
];

/// External grounding for the Causality (→) root primitive.
pub const CAUSALITY_AUTHORITIES: [ExternalAuthority; 4] = [
    ExternalAuthority {
        name: "Pearl's do-calculus",
        domain: AuthorityDomain::ComputerScience,
        citation: "Pearl J. Causality: Models, Reasoning, and Inference. 2nd ed. Cambridge University Press; 2009.",
        year: 2009,
        grounding_claim: "Causal relationships are irreducible to correlation; do(X) formalizes intervention",
        doi: Some("10.1017/CBO9780511803161"),
    },
    ExternalAuthority {
        name: "Hume's Constant Conjunction",
        domain: AuthorityDomain::Philosophy,
        citation: "Hume D. A Treatise of Human Nature. Clarendon Press; 1739 (reprinted 1978).",
        year: 1739,
        grounding_claim: "Causation is primitive to human understanding; we observe constant conjunction",
        doi: None,
    },
    ExternalAuthority {
        name: "Shannon's Channel Capacity",
        domain: AuthorityDomain::InformationTheory,
        citation: "Shannon CE. A mathematical theory of communication. Bell Syst Tech J. 1948;27(3):379-423.",
        year: 1948,
        grounding_claim: "Information transfer (cause → effect) is bounded by channel capacity",
        doi: Some("10.1002/j.1538-7305.1948.tb01338.x"),
    },
    ExternalAuthority {
        name: "Granger Causality",
        domain: AuthorityDomain::ComputerScience,
        citation: "Granger CWJ. Investigating causal relations by econometric models. Econometrica. 1969;37(3):424-438.",
        year: 1969,
        grounding_claim: "X causes Y if X helps predict Y beyond Y's own history",
        doi: Some("10.2307/1912791"),
    },
];

/// Returns external authorities for a root primitive.
///
/// Returns None for non-root primitives.
#[must_use]
pub fn external_authorities(primitive: LexPrimitiva) -> Option<&'static [ExternalAuthority]> {
    match primitive {
        LexPrimitiva::Quantity => Some(&QUANTITY_AUTHORITIES),
        LexPrimitiva::Causality => Some(&CAUSALITY_AUTHORITIES),
        LexPrimitiva::Sequence
        | LexPrimitiva::Mapping
        | LexPrimitiva::State
        | LexPrimitiva::Recursion
        | LexPrimitiva::Void
        | LexPrimitiva::Boundary
        | LexPrimitiva::Frequency
        | LexPrimitiva::Existence
        | LexPrimitiva::Persistence
        | LexPrimitiva::Comparison
        | LexPrimitiva::Location
        | LexPrimitiva::Irreversibility
        | LexPrimitiva::Sum
        | LexPrimitiva::Product => None,
    }
}

/// Validates that a root primitive has sufficient external grounding.
///
/// A root is considered externally grounded if it has authorities from
/// at least 2 different domains.
#[must_use]
pub fn is_externally_grounded(primitive: LexPrimitiva) -> bool {
    external_authorities(primitive)
        .map(|auths| {
            let domains: std::collections::BTreeSet<_> = auths.iter().map(|a| a.domain).collect();
            domains.len() >= 2
        })
        .unwrap_or(false)
}

/// Returns the minimum authority year (oldest citation) for a root.
#[must_use]
pub fn oldest_authority_year(primitive: LexPrimitiva) -> Option<u16> {
    external_authorities(primitive).map(|auths| auths.iter().map(|a| a.year).min().unwrap_or(0))
}

/// Grounding strength based on authority diversity and citation age.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GroundingStrength {
    /// Strong: 3+ domains, oldest citation >100 years
    Strong,
    /// Moderate: 2+ domains, oldest citation >50 years
    Moderate,
    /// Weak: <2 domains or recent citations only
    Weak,
    /// None: No external grounding (non-root)
    None,
}

/// Computes grounding strength for a primitive.
#[must_use]
pub fn grounding_strength(primitive: LexPrimitiva) -> GroundingStrength {
    let Some(auths) = external_authorities(primitive) else {
        return GroundingStrength::None;
    };

    let domains: std::collections::BTreeSet<_> = auths.iter().map(|a| a.domain).collect();
    let oldest = auths.iter().map(|a| a.year).min().unwrap_or(2025);
    let age = 2026_u16.saturating_sub(oldest);

    match (domains.len(), age) {
        (d, a) if d >= 3 && a >= 100 => GroundingStrength::Strong,
        (d, a) if d >= 2 && a >= 50 => GroundingStrength::Moderate,
        (d, _) if d >= 2 => GroundingStrength::Weak,
        _ => GroundingStrength::Weak,
    }
}

impl fmt::Display for GroundingStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Strong => write!(f, "Strong (3+ domains, >100yr)"),
            Self::Moderate => write!(f, "Moderate (2+ domains, >50yr)"),
            Self::Weak => write!(f, "Weak (<2 domains or recent)"),
            Self::None => write!(f, "None (non-root)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_has_external_authorities() {
        let auths = external_authorities(LexPrimitiva::Quantity);
        assert!(auths.is_some());
        assert_eq!(auths.map(|a| a.len()).unwrap_or(0), 4);
    }

    #[test]
    fn test_causality_has_external_authorities() {
        let auths = external_authorities(LexPrimitiva::Causality);
        assert!(auths.is_some());
        assert_eq!(auths.map(|a| a.len()).unwrap_or(0), 4);
    }

    #[test]
    fn test_non_roots_have_no_external_authorities() {
        assert!(external_authorities(LexPrimitiva::Sequence).is_none());
        assert!(external_authorities(LexPrimitiva::Void).is_none());
    }

    #[test]
    fn test_roots_externally_grounded() {
        assert!(is_externally_grounded(LexPrimitiva::Quantity));
        assert!(is_externally_grounded(LexPrimitiva::Causality));
    }

    #[test]
    fn test_non_roots_not_externally_grounded() {
        assert!(!is_externally_grounded(LexPrimitiva::Mapping));
    }

    #[test]
    fn test_oldest_authority_year() {
        // Hume 1739 is oldest for Causality
        assert_eq!(oldest_authority_year(LexPrimitiva::Causality), Some(1739));
        // Peano 1889 is oldest for Quantity
        assert_eq!(oldest_authority_year(LexPrimitiva::Quantity), Some(1889));
    }

    #[test]
    fn test_grounding_strength_quantity() {
        // Quantity: 4 domains (Math, Engineering, Physics, Math), oldest 1889 = 137 years
        let strength = grounding_strength(LexPrimitiva::Quantity);
        assert_eq!(strength, GroundingStrength::Strong);
    }

    #[test]
    fn test_grounding_strength_causality() {
        // Causality: 4 domains (CS, Philosophy, InfoTheory, CS), oldest 1739 = 287 years
        let strength = grounding_strength(LexPrimitiva::Causality);
        assert_eq!(strength, GroundingStrength::Strong);
    }

    #[test]
    fn test_grounding_strength_non_root() {
        assert_eq!(
            grounding_strength(LexPrimitiva::Sum),
            GroundingStrength::None
        );
    }

    #[test]
    fn test_authority_domains_diverse() {
        // Check both roots have multiple domains
        for root in LexPrimitiva::roots() {
            let auths = external_authorities(root).expect("root should have authorities");
            let domains: std::collections::BTreeSet<_> = auths.iter().map(|a| a.domain).collect();
            assert!(
                domains.len() >= 2,
                "{:?} should have authorities from 2+ domains",
                root
            );
        }
    }

    #[test]
    fn test_all_authorities_have_citations() {
        for root in LexPrimitiva::roots() {
            let auths = external_authorities(root).expect("root should have authorities");
            for auth in auths {
                assert!(!auth.citation.is_empty(), "{} missing citation", auth.name);
                assert!(
                    !auth.grounding_claim.is_empty(),
                    "{} missing claim",
                    auth.name
                );
            }
        }
    }
}
