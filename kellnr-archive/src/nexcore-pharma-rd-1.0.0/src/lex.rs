//! Lex Primitiva symbols and grounding infrastructure.
//!
//! Self-contained copy of the 16 T1 symbols for pharma R&D domain modeling.
//! Production systems should use `nexcore-lex-primitiva` directly.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The 16 irreducible T1 symbols from Lex Primitiva.
///
/// Tier: T1 | These ARE the primitives — everything else composes from them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum LexSymbol {
    /// σ — Ordered progression, temporal flow
    Sequence,
    /// μ — Input-to-output correspondence
    Mapping,
    /// ς — Mutable condition, lifecycle phase
    State,
    /// ρ — Self-reference, iterative deepening
    Recursion,
    /// ∅ — Absence, negation, null
    Void,
    /// ∂ — Threshold, limit, demarcation
    Boundary,
    /// ν — Rate of occurrence, periodicity
    Frequency,
    /// ∃ — Presence, detection, verification
    Existence,
    /// π — Durability, storage, retention
    Persistence,
    /// → — If-then, mechanism, pathway
    Causality,
    /// κ — Same/different, ranking, threshold gate
    Comparison,
    /// N — Measurable magnitude, count, dose
    Quantity,
    /// λ — Spatial position, compartment, tissue
    Location,
    /// ∝ — Permanent change, no undo
    Irreversibility,
    /// Σ — Aggregation, ensemble, accumulation
    Sum,
    /// × — Structured conjunction, record, type
    Product,
}

impl LexSymbol {
    /// Unicode glyph for this symbol.
    #[must_use]
    pub const fn glyph(&self) -> &'static str {
        match self {
            Self::Sequence => "σ",
            Self::Mapping => "μ",
            Self::State => "ς",
            Self::Recursion => "ρ",
            Self::Void => "∅",
            Self::Boundary => "∂",
            Self::Frequency => "ν",
            Self::Existence => "∃",
            Self::Persistence => "π",
            Self::Causality => "→",
            Self::Comparison => "κ",
            Self::Quantity => "N",
            Self::Location => "λ",
            Self::Irreversibility => "∝",
            Self::Sum => "Σ",
            Self::Product => "×",
        }
    }

    /// All 16 symbols in canonical order.
    #[must_use]
    pub const fn all() -> &'static [LexSymbol; 16] {
        &[
            Self::Sequence,
            Self::Mapping,
            Self::State,
            Self::Recursion,
            Self::Void,
            Self::Boundary,
            Self::Frequency,
            Self::Existence,
            Self::Persistence,
            Self::Causality,
            Self::Comparison,
            Self::Quantity,
            Self::Location,
            Self::Irreversibility,
            Self::Sum,
            Self::Product,
        ]
    }
}

impl fmt::Display for LexSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.glyph())
    }
}

/// A composition of Lex Primitiva symbols grounding a concept.
///
/// Tier classification follows from symbol count:
/// - 1 unique symbol → T1
/// - 2-3 unique symbols → T2-P (cross-domain primitive)
/// - 4-5 unique symbols → T2-C (cross-domain composite)
/// - 6+ unique symbols → T3 (domain-specific)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveComposition {
    symbols: Vec<LexSymbol>,
}

impl PrimitiveComposition {
    /// Create a composition from symbols. Deduplicates internally.
    #[must_use]
    pub fn new(symbols: &[LexSymbol]) -> Self {
        let mut unique: Vec<LexSymbol> = symbols.to_vec();
        unique.sort();
        unique.dedup();
        Self { symbols: unique }
    }

    /// Number of unique symbols in this composition.
    #[must_use]
    pub fn unique_count(&self) -> usize {
        self.symbols.len()
    }

    /// The unique symbols.
    #[must_use]
    pub fn symbols(&self) -> &[LexSymbol] {
        &self.symbols
    }

    /// Classify tier based on unique symbol count.
    #[must_use]
    pub fn tier(&self) -> Tier {
        match self.unique_count() {
            0 => Tier::T1, // degenerate
            1 => Tier::T1,
            2..=3 => Tier::T2P,
            4..=5 => Tier::T2C,
            _ => Tier::T3,
        }
    }

    /// Whether this composition contains a specific symbol.
    #[must_use]
    pub fn contains(&self, symbol: LexSymbol) -> bool {
        self.symbols.contains(&symbol)
    }

    /// Format as glyph string: "κ + ∂ + N"
    #[must_use]
    pub fn glyph_string(&self) -> String {
        self.symbols
            .iter()
            .map(|s| s.glyph())
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

impl fmt::Display for PrimitiveComposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.glyph_string())
    }
}

/// Primitive tier classification.
///
/// Tier: T2-P | Dominant: Σ (Sum) — four-variant alternation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tier {
    /// Universal, domain-independent (1 unique symbol).
    T1,
    /// Cross-domain primitive (2-3 unique symbols).
    T2P,
    /// Cross-domain composite (4-5 unique symbols).
    T2C,
    /// Domain-specific (6+ unique symbols).
    T3,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::T1 => write!(f, "T1"),
            Self::T2P => write!(f, "T2-P"),
            Self::T2C => write!(f, "T2-C"),
            Self::T3 => write!(f, "T3"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_symbols_has_16() {
        assert_eq!(LexSymbol::all().len(), 16);
    }

    #[test]
    fn composition_deduplicates() {
        let c = PrimitiveComposition::new(&[
            LexSymbol::Quantity,
            LexSymbol::Comparison,
            LexSymbol::Quantity, // duplicate
        ]);
        assert_eq!(c.unique_count(), 2);
    }

    #[test]
    fn tier_from_count() {
        let t1 = PrimitiveComposition::new(&[LexSymbol::Quantity]);
        assert_eq!(t1.tier(), Tier::T1);

        let t2p = PrimitiveComposition::new(&[LexSymbol::Quantity, LexSymbol::Comparison]);
        assert_eq!(t2p.tier(), Tier::T2P);

        let t2c = PrimitiveComposition::new(&[
            LexSymbol::Boundary,
            LexSymbol::Location,
            LexSymbol::State,
            LexSymbol::Causality,
        ]);
        assert_eq!(t2c.tier(), Tier::T2C);

        let t3 = PrimitiveComposition::new(&[
            LexSymbol::Sequence,
            LexSymbol::Mapping,
            LexSymbol::State,
            LexSymbol::Recursion,
            LexSymbol::Boundary,
            LexSymbol::Causality,
        ]);
        assert_eq!(t3.tier(), Tier::T3);
    }

    #[test]
    fn glyph_string_format() {
        let c = PrimitiveComposition::new(&[LexSymbol::Quantity, LexSymbol::Causality]);
        let s = c.glyph_string();
        assert!(s.contains("N"));
        assert!(s.contains("→"));
    }

    #[test]
    fn display_roundtrip() {
        assert_eq!(format!("{}", LexSymbol::Boundary), "∂");
        assert_eq!(format!("{}", Tier::T2P), "T2-P");
    }
}
