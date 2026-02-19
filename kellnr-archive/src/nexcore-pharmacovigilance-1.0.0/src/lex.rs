//! Lex Primitiva: the 16 irreducible T1 symbols.
//!
//! Every pharmacovigilance concept decomposes to combinations of these symbols.
//! Tier classification derived from unique symbol count in composition.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The 16 irreducible Lex Primitiva symbols.
///
/// Tier: T1 | Each symbol IS a primitive — single-symbol composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LexSymbol {
    /// σ — ordered succession of elements
    Sequence,
    /// μ — structure-preserving transformation
    Mapping,
    /// ς — snapshot of a system at time t
    State,
    /// ρ — self-referential definition
    Recursion,
    /// ∅ — absence, missing data, null
    Void,
    /// ∂ — threshold, limit, scope delimiter
    Boundary,
    /// ν — rate of occurrence per unit observation
    Frequency,
    /// ∃ — presence or absence of an entity
    Existence,
    /// π — retention across time
    Persistence,
    /// → — one event bringing about another
    Causality,
    /// κ — relation between two quantities
    Comparison,
    /// N — numerical magnitude
    Quantity,
    /// λ — position in space or context
    Location,
    /// ∝ — cannot be undone
    Irreversibility,
    /// Σ — coproduct, mutually exclusive categories
    Sum,
    /// × — conjunction, co-occurring factors
    Product,
}

impl LexSymbol {
    /// All 16 symbols.
    pub const ALL: &'static [Self] = &[
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
    ];

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

    /// All 16 symbols as a slice (alias for ALL).
    #[must_use]
    pub fn all() -> &'static [Self] {
        Self::ALL
    }
}

impl fmt::Display for LexSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.glyph(), self)
    }
}

/// A composition of Lex Primitiva symbols grounding a concept.
///
/// Tier classification: 1 unique = T1, 2-3 = T2-P, 4-5 = T2-C, 6+ = T3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveComposition {
    symbols: Vec<LexSymbol>,
}

impl PrimitiveComposition {
    /// Create a new composition from symbols (deduplicates).
    #[must_use]
    pub fn new(symbols: &[LexSymbol]) -> Self {
        let mut unique = Vec::new();
        for &s in symbols {
            if !unique.contains(&s) {
                unique.push(s);
            }
        }
        Self { symbols: unique }
    }

    /// Number of unique symbols in this composition.
    #[must_use]
    pub fn unique_count(&self) -> usize {
        self.symbols.len()
    }

    /// The symbols in this composition.
    #[must_use]
    pub fn symbols(&self) -> &[LexSymbol] {
        &self.symbols
    }

    /// Tier classification based on unique symbol count.
    #[must_use]
    pub fn tier(&self) -> Tier {
        match self.unique_count() {
            0 => Tier::T1,
            1 => Tier::T1,
            2 | 3 => Tier::T2P,
            4 | 5 => Tier::T2C,
            _ => Tier::T3,
        }
    }

    /// Merge two compositions (union of symbols).
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let mut all: Vec<LexSymbol> = self.symbols.clone();
        for &s in &other.symbols {
            if !all.contains(&s) {
                all.push(s);
            }
        }
        Self { symbols: all }
    }
}

/// Tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tier {
    /// 1 unique symbol — universal primitive
    T1,
    /// 2-3 unique symbols — cross-domain primitive
    T2P,
    /// 4-5 unique symbols — cross-domain composite
    T2C,
    /// 6+ unique symbols — domain-specific
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
        assert_eq!(LexSymbol::ALL.len(), 16);
    }

    #[test]
    fn composition_deduplicates() {
        let comp = PrimitiveComposition::new(&[
            LexSymbol::Causality,
            LexSymbol::Causality,
            LexSymbol::Boundary,
        ]);
        assert_eq!(comp.unique_count(), 2);
    }

    #[test]
    fn tier_from_count() {
        assert_eq!(
            PrimitiveComposition::new(&[LexSymbol::Boundary]).tier(),
            Tier::T1
        );
        assert_eq!(
            PrimitiveComposition::new(&[LexSymbol::Boundary, LexSymbol::Causality]).tier(),
            Tier::T2P
        );
        assert_eq!(
            PrimitiveComposition::new(&[
                LexSymbol::Boundary,
                LexSymbol::Causality,
                LexSymbol::Quantity,
                LexSymbol::Sum,
            ])
            .tier(),
            Tier::T2C
        );
        assert_eq!(
            PrimitiveComposition::new(&[
                LexSymbol::Boundary,
                LexSymbol::Causality,
                LexSymbol::Quantity,
                LexSymbol::Sum,
                LexSymbol::Mapping,
                LexSymbol::Sequence,
            ])
            .tier(),
            Tier::T3
        );
    }

    #[test]
    fn merge_unions() {
        let a = PrimitiveComposition::new(&[LexSymbol::Causality, LexSymbol::Boundary]);
        let b = PrimitiveComposition::new(&[LexSymbol::Boundary, LexSymbol::Quantity]);
        let merged = a.merge(&b);
        assert_eq!(merged.unique_count(), 3);
    }

    #[test]
    fn display_glyph() {
        assert_eq!(format!("{}", LexSymbol::Causality), "→ (Causality)");
    }
}
