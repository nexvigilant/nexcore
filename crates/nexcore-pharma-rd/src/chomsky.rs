//! Chomsky grammar classification and generator algebra.
//!
//! Maps the Grand Equation: `Power(S) = Chomsky(min{G ⊆ {σ,Σ,ρ,κ,∃} : S ⊆ Lang(⟨G⟩)})`.
//! Overengineering = `|generators_used| - |generators_needed|`.

use crate::lex::LexSymbol;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 5 generators that produce all 16 Lex Primitiva symbols.
///
/// `Lex = ⟨σ, Σ, ρ, κ, ∃ | ∅ = ¬∃⟩`
///
/// Tier: T2-P | Dominant: Σ (Sum) — five-variant alternation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Generator {
    /// σ (Sequence) — ordered progression
    Sigma,
    /// Σ (Sum) — alternation, aggregation
    BigSigma,
    /// ρ (Recursion) — self-reference
    Rho,
    /// κ (Comparison) — context-sensitivity
    Kappa,
    /// ∃ (Existence) — universal computation
    Exists,
}

impl Generator {
    /// All 5 generators in Chomsky-ascending order.
    pub const ALL: &'static [Self] = &[
        Self::Sigma,
        Self::BigSigma,
        Self::Rho,
        Self::Kappa,
        Self::Exists,
    ];

    /// Which Lex Primitiva symbols this generator produces.
    #[must_use]
    pub fn produces(&self) -> &'static [LexSymbol] {
        match self {
            Self::Sigma => &[
                LexSymbol::Sequence,
                LexSymbol::Frequency,
                LexSymbol::Persistence,
            ],
            Self::BigSigma => &[
                LexSymbol::Sum,
                LexSymbol::Product,
                LexSymbol::Mapping,
                LexSymbol::Void,
            ],
            Self::Rho => &[LexSymbol::Recursion, LexSymbol::State],
            Self::Kappa => &[
                LexSymbol::Comparison,
                LexSymbol::Boundary,
                LexSymbol::Quantity,
                LexSymbol::Irreversibility,
            ],
            Self::Exists => &[
                LexSymbol::Existence,
                LexSymbol::Causality,
                LexSymbol::Location,
            ],
        }
    }
}

impl fmt::Display for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sigma => write!(f, "σ"),
            Self::BigSigma => write!(f, "Σ"),
            Self::Rho => write!(f, "ρ"),
            Self::Kappa => write!(f, "κ"),
            Self::Exists => write!(f, "∃"),
        }
    }
}

/// Chomsky hierarchy level — grammar classification.
///
/// Each level adds one generator to the previous:
/// - Type-3: σ + Σ → finite automata
/// - Type-2: + ρ → pushdown automata
/// - Type-1: + κ → linear bounded automata
/// - Type-0: + ∃ → Turing machines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChomskyLevel {
    /// Regular grammar. Finite automaton. Pattern matching, flat pipelines.
    Type3Regular,
    /// Context-free grammar. Pushdown automaton. Recursive parsers, tree walkers.
    Type2ContextFree,
    /// Context-sensitive grammar. Linear bounded automaton. Validators, type checkers.
    Type1ContextSensitive,
    /// Unrestricted grammar. Turing machine. Interpreters, Bayesian engines.
    Type0Unrestricted,
}

impl ChomskyLevel {
    /// Minimum generators required for this level.
    #[must_use]
    pub fn required_generators(&self) -> &'static [Generator] {
        match self {
            Self::Type3Regular => &[Generator::Sigma, Generator::BigSigma],
            Self::Type2ContextFree => &[Generator::Sigma, Generator::BigSigma, Generator::Rho],
            Self::Type1ContextSensitive => &[
                Generator::Sigma,
                Generator::BigSigma,
                Generator::Rho,
                Generator::Kappa,
            ],
            Self::Type0Unrestricted => Generator::ALL,
        }
    }

    /// Number of generators at this level.
    #[must_use]
    pub fn generator_count(&self) -> usize {
        self.required_generators().len()
    }

    /// Computational model name.
    #[must_use]
    pub const fn automaton(&self) -> &'static str {
        match self {
            Self::Type3Regular => "Finite Automaton",
            Self::Type2ContextFree => "Pushdown Automaton",
            Self::Type1ContextSensitive => "Linear Bounded Automaton",
            Self::Type0Unrestricted => "Turing Machine",
        }
    }

    /// Typical software architecture at this level.
    #[must_use]
    pub const fn architecture(&self) -> &'static str {
        match self {
            Self::Type3Regular => "State machine, flat pipeline, threshold gate",
            Self::Type2ContextFree => "Recursive parser, tree walker, nested protocol",
            Self::Type1ContextSensitive => "Context-aware validator, type checker, expert system",
            Self::Type0Unrestricted => "Interpreter, Bayesian engine, neural network",
        }
    }
}

impl fmt::Display for ChomskyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type3Regular => write!(f, "Type-3 (Regular)"),
            Self::Type2ContextFree => write!(f, "Type-2 (Context-Free)"),
            Self::Type1ContextSensitive => write!(f, "Type-1 (Context-Sensitive)"),
            Self::Type0Unrestricted => write!(f, "Type-0 (Unrestricted)"),
        }
    }
}

/// Determine the minimum Chomsky level needed for a set of generators.
///
/// `Chomsky(G) = min level where G ⊆ required_generators(level)`
///
/// Grounding: κ (Comparison) — subset check across levels.
#[must_use]
pub fn classify_generators(generators: &[Generator]) -> ChomskyLevel {
    let has = |g: Generator| generators.contains(&g);

    if has(Generator::Exists) {
        ChomskyLevel::Type0Unrestricted
    } else if has(Generator::Kappa) {
        ChomskyLevel::Type1ContextSensitive
    } else if has(Generator::Rho) {
        ChomskyLevel::Type2ContextFree
    } else {
        ChomskyLevel::Type3Regular
    }
}

/// Compute overengineering: `|generators_used| - |generators_needed|`.
///
/// Returns 0 if perfectly matched, positive if over-engineered.
///
/// Grounding: N (Quantity) + κ (Comparison).
#[must_use]
pub fn overengineering(used: &[Generator], needed: ChomskyLevel) -> usize {
    let used_unique: std::collections::HashSet<Generator> = used.iter().copied().collect();
    let needed_count = needed.generator_count();
    used_unique.len().saturating_sub(needed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_generators_total() {
        assert_eq!(Generator::ALL.len(), 5);
    }

    #[test]
    fn classify_regular() {
        assert_eq!(
            classify_generators(&[Generator::Sigma, Generator::BigSigma]),
            ChomskyLevel::Type3Regular
        );
    }

    #[test]
    fn classify_context_free() {
        assert_eq!(
            classify_generators(&[Generator::Sigma, Generator::BigSigma, Generator::Rho]),
            ChomskyLevel::Type2ContextFree
        );
    }

    #[test]
    fn classify_context_sensitive() {
        assert_eq!(
            classify_generators(&[Generator::Sigma, Generator::Kappa]),
            ChomskyLevel::Type1ContextSensitive
        );
    }

    #[test]
    fn classify_unrestricted() {
        assert_eq!(
            classify_generators(&[Generator::Exists]),
            ChomskyLevel::Type0Unrestricted
        );
    }

    #[test]
    fn overengineering_perfect() {
        assert_eq!(
            overengineering(
                &[Generator::Sigma, Generator::BigSigma],
                ChomskyLevel::Type3Regular
            ),
            0
        );
    }

    #[test]
    fn overengineering_detected() {
        assert_eq!(
            overengineering(Generator::ALL, ChomskyLevel::Type3Regular),
            3 // 5 used - 2 needed
        );
    }

    #[test]
    fn levels_are_ordered() {
        assert!(ChomskyLevel::Type3Regular < ChomskyLevel::Type2ContextFree);
        assert!(ChomskyLevel::Type2ContextFree < ChomskyLevel::Type1ContextSensitive);
        assert!(ChomskyLevel::Type1ContextSensitive < ChomskyLevel::Type0Unrestricted);
    }

    #[test]
    fn generator_count_increases() {
        assert_eq!(ChomskyLevel::Type3Regular.generator_count(), 2);
        assert_eq!(ChomskyLevel::Type2ContextFree.generator_count(), 3);
        assert_eq!(ChomskyLevel::Type1ContextSensitive.generator_count(), 4);
        assert_eq!(ChomskyLevel::Type0Unrestricted.generator_count(), 5);
    }

    #[test]
    fn display_formats() {
        assert_eq!(
            format!("{}", ChomskyLevel::Type3Regular),
            "Type-3 (Regular)"
        );
        assert_eq!(format!("{}", Generator::Sigma), "σ");
    }
}
