//! # The 16 Lex Primitiva Symbols
//!
//! The bedrock of all computation: 16 universal primitives from which all
//! higher constructs derive.
//!
//! ## Tier: T1-Universal
//!
//! These are the "quarks" of computation - everything else composes from them.

use crate::constants::MathConstant;
use crate::state_mode::StateMode;
use crate::symbols::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The 16 irreducible Lex Primitiva symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LexPrimitiva {
    /// σ (sigma) - Ordered succession. Rust: Iterator, method chains, `?`.
    #[serde(rename = "sequence")]
    Sequence,
    /// μ (mu) - Transformation A → B. Rust: `From`, `Into`, `map()`.
    #[serde(rename = "mapping")]
    Mapping,
    /// ς (varsigma) - Context at a point in time. Rust: `struct`, typestates.
    #[serde(rename = "state")]
    State,
    /// ρ (rho) - Self-reference via indirection. Rust: `enum { X(Box<Self>) }`.
    #[serde(rename = "recursion")]
    Recursion,
    /// ∅ (empty set) - Meaningful absence. Rust: `Option<T>`, `PhantomData`.
    #[serde(rename = "void")]
    Void,
    /// ∂ (partial) - Delimiters and limits. Rust: `Result`, bounds.
    #[serde(rename = "boundary")]
    Boundary,
    /// ν (nu) - Rate of occurrence. Rust: Loop counters, rate limiters.
    #[serde(rename = "frequency")]
    Frequency,
    /// ∃ (exists) - Instantiation of being. Rust: `new()`, constructors.
    #[serde(rename = "existence")]
    Existence,
    /// π (pi) - Continuity across time. Rust: Database, file storage.
    #[serde(rename = "persistence")]
    Persistence,
    /// → (arrow) - Cause and consequence. Rust: Function calls, events.
    #[serde(rename = "causality")]
    Causality,
    /// κ (kappa) - Predicate matching. Rust: `==`, `match`, `if let`.
    #[serde(rename = "comparison")]
    Comparison,
    /// N - Numerical magnitude. Rust: `u32`, `f64`, `usize`.
    #[serde(rename = "quantity")]
    Quantity,
    /// λ (lambda) - Positional context. Rust: `Path`, pointers, URLs.
    #[serde(rename = "location")]
    Location,
    /// ∝ (proportional) - One-way state transition. Rust: `Drop`, consuming methods.
    #[serde(rename = "irreversibility")]
    Irreversibility,
    /// Σ (sigma) - Exclusive disjunction. Rust: `enum`, `match`, `Either`.
    #[serde(rename = "sum")]
    Sum,
    /// × (times) - Conjunctive combination. Rust: `struct`, tuples `(A, B)`, `zip()`.
    #[serde(rename = "product")]
    Product,
}

impl LexPrimitiva {
    /// Returns the mathematical symbol for this primitive.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Sequence => SIGMA_SEQ,
            Self::Mapping => MU_MAP,
            Self::State => VARSIGMA_STATE,
            Self::Recursion => RHO_REC,
            Self::Void => EMPTYSET_VOID,
            Self::Boundary => PARTIAL_BOUND,
            Self::Frequency => NU_FREQ,
            Self::Existence => EXISTS_INST,
            Self::Persistence => PI_PERSIST,
            Self::Causality => ARROW_CAUSAL,
            Self::Comparison => KAPPA_COMP,
            Self::Quantity => N_QUANT,
            Self::Location => LAMBDA_LOC,
            Self::Irreversibility => PROPTORTO_IRREV,
            Self::Sum => SIGMA_SUM,
            Self::Product => CROSS_PROD,
        }
    }

    /// Returns the canonical name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Sequence => "Sequence",
            Self::Mapping => "Mapping",
            Self::State => "State",
            Self::Recursion => "Recursion",
            Self::Void => "Void",
            Self::Boundary => "Boundary",
            Self::Frequency => "Frequency",
            Self::Existence => "Existence",
            Self::Persistence => "Persistence",
            Self::Causality => "Causality",
            Self::Comparison => "Comparison",
            Self::Quantity => "Quantity",
            Self::Location => "Location",
            Self::Irreversibility => "Irreversibility",
            Self::Sum => "Sum",
            Self::Product => "Product",
        }
    }

    /// Returns a brief description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Sequence => "Ordered operations, iteration, chaining",
            Self::Mapping => "Transformation between domains",
            Self::State => "Mutable or immutable data containers",
            Self::Recursion => "Self-referential structures and algorithms",
            Self::Void => "Meaningful absence, null representation",
            Self::Boundary => "Limits, constraints, error handling",
            Self::Frequency => "Rate, repetition, time-based patterns",
            Self::Existence => "Creation, instantiation, presence",
            Self::Persistence => "Durable storage, logging, state retention",
            Self::Causality => "Cause-effect relationships, function calls",
            Self::Comparison => "Equality, ordering, pattern matching",
            Self::Quantity => "Numeric values, counts, measurements",
            Self::Location => "Addressing, references, paths",
            Self::Irreversibility => "Consuming operations, resource cleanup",
            Self::Sum => "Disjoint union, algebraic sum types",
            Self::Product => "Conjunctive combination, record types",
        }
    }

    /// Returns the primary Rust manifestation.
    #[must_use]
    pub const fn rust_manifestation(&self) -> &'static str {
        match self {
            Self::Sequence => "Iterator, method chains, ? operator",
            Self::Mapping => "From, Into, map(), and_then()",
            Self::State => "struct, PhantomData, Cell, Mutex",
            Self::Recursion => "enum { X(Box<Self>) }, recursive fn",
            Self::Void => "Option::None, PhantomData, (), !",
            Self::Boundary => "Result, bounds, max_iter, HALT",
            Self::Frequency => "loop counter, rate limiter, throttle",
            Self::Existence => "new(), constructors, fs::write()",
            Self::Persistence => "database, file storage, logs",
            Self::Causality => "fn call, event trigger, callback",
            Self::Comparison => "==, match, if let, Ord",
            Self::Quantity => "u32, f64, usize, numeric types",
            Self::Location => "Path, pointer, URL, index",
            Self::Irreversibility => "Drop, consuming methods, move",
            Self::Sum => "enum, match, Either, coproduct",
            Self::Product => "struct, tuples (A, B), zip(), product type",
        }
    }

    /// Returns all 16 primitives in canonical order.
    #[must_use]
    pub const fn all() -> [Self; 16] {
        [
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

    /// Returns the primitives this one derives from (circular dependency proof).
    #[must_use]
    pub fn derives_from(&self) -> Vec<Self> {
        match self {
            Self::Sequence => vec![Self::State, Self::Mapping, Self::Causality],
            Self::Mapping => vec![Self::Existence, Self::Causality],
            Self::State => vec![Self::Existence, Self::Location],
            Self::Recursion => vec![Self::State, Self::Mapping, Self::Sequence],
            Self::Void => vec![Self::Existence, Self::Comparison],
            Self::Boundary => vec![Self::Comparison, Self::Quantity],
            Self::Frequency => vec![Self::Sequence, Self::Quantity, Self::Boundary],
            Self::Existence => vec![Self::Causality], // Location removed: existence precedes location
            Self::Persistence => vec![Self::Existence, Self::State, Self::Sequence],
            Self::Causality => vec![], // Root primitive
            Self::Comparison => vec![Self::Quantity],
            Self::Quantity => vec![], // Root primitive
            Self::Location => vec![Self::Existence, Self::Quantity],
            Self::Irreversibility => vec![Self::Causality, Self::Boundary, Self::State],
            Self::Sum => vec![Self::Comparison, Self::Void],
            Self::Product => vec![Self::Existence, Self::State],
        }
    }

    /// Returns true if this is a root primitive (no derivations).
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.derives_from().is_empty()
    }

    /// Returns the root primitives (Quantity and Causality).
    #[must_use]
    pub const fn roots() -> [Self; 2] {
        [Self::Quantity, Self::Causality]
    }

    /// Returns the primary mathematical constant for this primitive.
    #[must_use]
    pub const fn primary_constant(&self) -> MathConstant {
        match self {
            Self::Quantity => MathConstant::ONE,
            Self::Causality => MathConstant::ONE,
            Self::Comparison => MathConstant::ZERO,
            Self::Sequence => MathConstant::ONE,
            Self::Mapping => MathConstant::ONE,
            Self::State => MathConstant::ZERO,
            Self::Recursion => MathConstant::PHI,
            Self::Void => MathConstant::ZERO,
            Self::Boundary => MathConstant::INFINITY,
            Self::Frequency => MathConstant::PI,
            Self::Existence => MathConstant::ONE,
            Self::Persistence => MathConstant::INFINITY,
            Self::Location => MathConstant::ZERO,
            Self::Irreversibility => MathConstant::LN_2,
            Self::Sum => MathConstant::ONE,
            Self::Product => MathConstant::ONE,
        }
    }

    /// Parse from symbol string.
    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        match s {
            SIGMA_SEQ | "sigma" => Some(Self::Sequence),
            MU_MAP | "mu" => Some(Self::Mapping),
            VARSIGMA_STATE | "varsigma" => Some(Self::State),
            RHO_REC | "rho" => Some(Self::Recursion),
            EMPTYSET_VOID | "emptyset" => Some(Self::Void),
            PARTIAL_BOUND | "partial" => Some(Self::Boundary),
            NU_FREQ | "nu" => Some(Self::Frequency),
            EXISTS_INST | "exists" => Some(Self::Existence),
            PI_PERSIST | "pi" => Some(Self::Persistence),
            ARROW_CAUSAL | "arrow" | "->" => Some(Self::Causality),
            KAPPA_COMP | "kappa" => Some(Self::Comparison),
            N_QUANT => Some(Self::Quantity),
            LAMBDA_LOC | "lambda" => Some(Self::Location),
            PROPTORTO_IRREV | "propto" => Some(Self::Irreversibility),
            SIGMA_SUM | "Sigma" => Some(Self::Sum),
            CROSS_PROD | "times" | "cross" => Some(Self::Product),
            _ => None,
        }
    }
}

impl std::fmt::Display for LexPrimitiva {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.symbol(), self.name())
    }
}

/// A composition of primitives that grounds a higher-tier type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PrimitiveComposition {
    /// The primitives that compose this type.
    pub primitives: Vec<LexPrimitiva>,
    /// The dominant primitive (primary characteristic).
    pub dominant: Option<LexPrimitiva>,
    /// Confidence in this grounding (0.0-1.0).
    pub confidence: f64,
    /// Disambiguated State (ς) mode, if this type involves state.
    ///
    /// `None` for types without state or where mode is not yet classified.
    /// Backward-compatible: old JSON without this field deserializes to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mode: Option<StateMode>,
}

impl PrimitiveComposition {
    /// Creates a new composition.
    #[must_use]
    pub fn new(primitives: Vec<LexPrimitiva>) -> Self {
        let dominant = primitives.first().copied();
        Self {
            primitives,
            dominant,
            confidence: 1.0,
            state_mode: None,
        }
    }

    /// Sets the state mode for this composition.
    #[must_use]
    pub fn with_state_mode(mut self, mode: StateMode) -> Self {
        self.state_mode = Some(mode);
        self
    }

    /// Sets the dominant primitive and confidence.
    #[must_use]
    pub fn with_dominant(mut self, dominant: LexPrimitiva, confidence: f64) -> Self {
        self.dominant = Some(dominant);
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Returns unique primitives as a set.
    #[must_use]
    pub fn unique(&self) -> BTreeSet<LexPrimitiva> {
        self.primitives.iter().copied().collect()
    }

    /// Returns true if this is a pure primitive (single element).
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.unique().len() == 1
    }
}

impl std::fmt::Display for PrimitiveComposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbols: Vec<&str> = self.primitives.iter().map(LexPrimitiva::symbol).collect();
        write!(f, "[{}]", symbols.join(" + "))?;
        if let Some(mode) = &self.state_mode {
            write!(f, " ({})", mode)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_16_primitives() {
        let all = LexPrimitiva::all();
        assert_eq!(all.len(), 16);
        let symbols: BTreeSet<_> = all.iter().map(|p| p.symbol()).collect();
        assert_eq!(symbols.len(), 16);
    }

    #[test]
    fn test_root_primitives() {
        assert!(LexPrimitiva::Quantity.is_root());
        assert!(LexPrimitiva::Causality.is_root());
        assert!(!LexPrimitiva::Sequence.is_root());
    }

    #[test]
    fn test_symbol_parsing() {
        assert_eq!(
            LexPrimitiva::from_symbol(SIGMA_SEQ),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            LexPrimitiva::from_symbol(N_QUANT),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(
            LexPrimitiva::from_symbol("->"),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(LexPrimitiva::from_symbol("invalid"), None);
    }

    #[test]
    fn test_composition() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ]);
        assert_eq!(comp.unique().len(), 3);
        assert!(!comp.is_pure());
        assert_eq!(
            format!("{comp}"),
            format!("[{} + {} + {}]", SIGMA_SEQ, MU_MAP, VARSIGMA_STATE)
        );
    }

    #[test]
    fn test_pure_composition() {
        let pure = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert!(pure.is_pure());
    }
}
