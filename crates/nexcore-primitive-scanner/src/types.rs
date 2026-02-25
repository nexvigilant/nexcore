//! Core types for primitive extraction.
//!
//! Tier: T2-C (Cross-domain composite - used across extraction, testing, graphing)
//! Grounds to T1 Concepts (Quantity N, Sequence σ, Void ∅) via scalar and container types.

use serde::{Deserialize, Serialize};

/// Primitive tier classification.
///
/// Tier: T2-P (Cross-domain atomic classification)
/// Ord: T1 > T2P > T2C > T3 (higher = more universal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrimitiveTier {
    /// T3: Domain-specific (1 domain only)
    #[serde(rename = "T3")]
    T3DomainSpecific = 0,

    /// T2-C: Cross-domain composite (built from primitives, 2+ domains)
    #[serde(rename = "T2-C")]
    T2Composite = 1,

    /// T2-P: Cross-domain primitive (atomic, 2+ domains)
    #[serde(rename = "T2-P")]
    T2Primitive = 2,

    /// T1: Universal (atomic, ALL domains)
    #[serde(rename = "T1")]
    T1Universal = 3,
}

impl PrimitiveTier {
    /// Base confidence for this tier.
    #[must_use]
    pub const fn base_confidence(&self) -> f64 {
        match self {
            Self::T1Universal => 1.0,
            Self::T2Primitive => 0.9,
            Self::T2Composite => 0.7,
            Self::T3DomainSpecific => 0.5,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::T1Universal => "T1 Universal",
            Self::T2Primitive => "T2-P Cross-Domain Primitive",
            Self::T2Composite => "T2-C Cross-Domain Composite",
            Self::T3DomainSpecific => "T3 Domain-Specific",
        }
    }

    /// Minimum domain coverage for this tier.
    #[must_use]
    pub const fn min_domains(&self) -> usize {
        match self {
            Self::T1Universal => 10,
            Self::T2Primitive | Self::T2Composite => 2,
            Self::T3DomainSpecific => 1,
        }
    }
}

impl std::fmt::Display for PrimitiveTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A term with its definition for testing.
///
/// Tier: T2-C (Cross-domain - NLP, documentation, testing)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TermDefinition {
    /// The term being analyzed.
    pub term: String,

    /// Natural language definition.
    pub definition: String,

    /// Domain context (optional).
    #[serde(default)]
    pub domain: Option<String>,

    /// Known domain terms in definition (for Test 1).
    #[serde(default)]
    pub domain_terms: Vec<String>,

    /// External grounding concepts (for Test 2).
    #[serde(default)]
    pub external_grounding: Vec<String>,
}

impl TermDefinition {
    /// Create a new term definition.
    #[must_use]
    pub fn new(term: impl Into<String>, definition: impl Into<String>) -> Self {
        Self {
            term: term.into(),
            definition: definition.into(),
            domain: None,
            domain_terms: Vec::new(),
            external_grounding: Vec::new(),
        }
    }

    /// Set domain context.
    #[must_use]
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Add domain terms found in definition.
    #[must_use]
    pub fn with_domain_terms(mut self, terms: Vec<String>) -> Self {
        self.domain_terms = terms;
        self
    }

    /// Add external grounding.
    #[must_use]
    pub fn with_grounding(mut self, grounding: Vec<String>) -> Self {
        self.external_grounding = grounding;
        self
    }
}

/// An extracted primitive.
///
/// Tier: T2-C (Cross-domain composite)
/// Grounds to: T1 (String, f64, Vec) + T2-P (classification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Primitive {
    /// Unique identifier (e.g., "PRIM-UNIV-0001").
    pub id: String,

    /// Primitive name.
    pub name: String,

    /// Natural language definition.
    pub definition: String,

    /// Tier classification.
    pub tier: PrimitiveTier,

    /// Confidence score [0.0, 1.0].
    pub confidence: f64,

    /// Dependencies (names of other primitives).
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Domains where this primitive appears.
    #[serde(default)]
    pub domains: Vec<String>,

    /// External grounding concepts.
    #[serde(default)]
    pub grounds_to: Vec<String>,

    /// Rust manifestation (how it appears in code).
    #[serde(default)]
    pub rust_form: Option<String>,
}

impl Primitive {
    /// Create a new primitive.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        definition: impl Into<String>,
        tier: PrimitiveTier,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            definition: definition.into(),
            tier,
            confidence: tier.base_confidence(),
            depends_on: Vec::new(),
            domains: Vec::new(),
            grounds_to: Vec::new(),
            rust_form: None,
        }
    }

    /// Set dependencies.
    #[must_use]
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.depends_on = deps;
        self
    }

    /// Set domains.
    #[must_use]
    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.domains = domains;
        self
    }

    /// Set external grounding.
    #[must_use]
    pub fn with_grounding(mut self, grounding: Vec<String>) -> Self {
        self.grounds_to = grounding;
        self
    }

    /// Set Rust manifestation.
    #[must_use]
    pub fn with_rust_form(mut self, form: impl Into<String>) -> Self {
        self.rust_form = Some(form.into());
        self
    }

    /// Check if this is a root primitive (no dependencies).
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.depends_on.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_ordering() {
        assert!(PrimitiveTier::T1Universal > PrimitiveTier::T2Primitive);
        assert!(PrimitiveTier::T2Primitive > PrimitiveTier::T2Composite);
        assert!(PrimitiveTier::T2Composite > PrimitiveTier::T3DomainSpecific);
    }

    #[test]
    fn test_tier_confidence() {
        assert!((PrimitiveTier::T1Universal.base_confidence() - 1.0).abs() < f64::EPSILON);
        assert!((PrimitiveTier::T2Primitive.base_confidence() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_primitive_creation() {
        let p = Primitive::new(
            "PRIM-UNIV-0001",
            "sequence",
            "Ordered operations",
            PrimitiveTier::T1Universal,
        )
        .with_rust_form("Iterator, for");

        assert_eq!(p.name, "sequence");
        assert!(p.is_root());
        assert_eq!(p.rust_form, Some("Iterator, for".to_string()));
    }
}
