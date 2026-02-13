// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Exhaustive Pattern Checking
//!
//! Ensures all pattern matches (Σ) cover all possible cases.
//!
//! ## Philosophy
//!
//! "Code that compiles is mathematically true."
//!
//! Exhaustive checking ensures no unhandled cases at runtime.
//! Every Σ (Sum) must decompose completely.
//!
//! ## Tier: T2-C (Σ + κ + ν + ∂)
//!
//! ## Checking Strategy
//!
//! 1. Build a "pattern space" representing all possible values
//! 2. For each arm, subtract the matched patterns from the space
//! 3. If remaining space is non-empty, match is non-exhaustive
//! 4. Report missing patterns as diagnostics

use crate::ast::{Literal, MatchArm, Pattern};
use crate::token::Span;
use crate::types::PrimaType;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ═══════════════════════════════════════════════════════════════════════════
// PATTERN SPACE — Σ (Sum of all possible patterns)
// ═══════════════════════════════════════════════════════════════════════════

/// Represents the space of possible patterns for a type.
///
/// ## Tier: T2-P (Σ + σ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternSpace {
    /// All values of a type (unconstrained).
    All,
    /// No values (empty/contradiction).
    Empty,
    /// Boolean space: {true, false}.
    Bool(BoolSpace),
    /// Integer space (for literal matching).
    Int(IntSpace),
    /// String space (for literal matching).
    String(StringSpace),
    /// Sequence space.
    Sequence(Box<PatternSpace>),
    /// Sum type variants.
    Variants(HashSet<String>),
    /// Wildcard (matches anything).
    Wildcard,
}

/// Boolean pattern space.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoolSpace {
    pub has_true: bool,
    pub has_false: bool,
}

impl BoolSpace {
    /// Full boolean space.
    #[must_use]
    pub const fn full() -> Self {
        Self {
            has_true: true,
            has_false: true,
        }
    }

    /// Check if exhausted.
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        !self.has_true && !self.has_false
    }

    /// Remove a value.
    pub fn remove(&mut self, value: bool) {
        if value {
            self.has_true = false;
        } else {
            self.has_false = false;
        }
    }

    /// Get missing patterns.
    #[must_use]
    pub fn missing(&self) -> Vec<String> {
        let mut result = Vec::new();
        if self.has_true {
            result.push("true".to_string());
        }
        if self.has_false {
            result.push("false".to_string());
        }
        result
    }
}

/// Integer pattern space (simplified for common cases).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntSpace {
    /// Explicitly covered values.
    pub covered: HashSet<i64>,
    /// Whether a wildcard covers the rest.
    pub has_wildcard: bool,
}

impl Default for IntSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl IntSpace {
    /// New uncovered integer space.
    #[must_use]
    pub fn new() -> Self {
        Self {
            covered: HashSet::new(),
            has_wildcard: false,
        }
    }

    /// Check if exhausted (integers are infinite, so only wildcards exhaust).
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.has_wildcard
    }

    /// Cover a specific value.
    pub fn cover(&mut self, value: i64) {
        self.covered.insert(value);
    }

    /// Cover all remaining with wildcard.
    pub fn cover_wildcard(&mut self) {
        self.has_wildcard = true;
    }
}

/// String pattern space (simplified).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringSpace {
    /// Explicitly covered values.
    pub covered: HashSet<String>,
    /// Whether a wildcard covers the rest.
    pub has_wildcard: bool,
}

impl Default for StringSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl StringSpace {
    /// New uncovered string space.
    #[must_use]
    pub fn new() -> Self {
        Self {
            covered: HashSet::new(),
            has_wildcard: false,
        }
    }

    /// Check if exhausted.
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.has_wildcard
    }

    /// Cover a specific value.
    pub fn cover(&mut self, value: String) {
        self.covered.insert(value);
    }

    /// Cover all remaining with wildcard.
    pub fn cover_wildcard(&mut self) {
        self.has_wildcard = true;
    }
}

impl PatternSpace {
    /// Check if the space is exhausted (all patterns covered).
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        match self {
            Self::All => false,
            Self::Empty => true,
            Self::Wildcard => true,
            Self::Bool(b) => b.is_exhausted(),
            Self::Int(i) => i.is_exhausted(),
            Self::String(s) => s.is_exhausted(),
            Self::Sequence(inner) => inner.is_exhausted(),
            Self::Variants(v) => v.is_empty(),
        }
    }

    /// Create pattern space for a type.
    #[must_use]
    pub fn for_type(ty: &PrimaType) -> Self {
        match ty.name.as_str() {
            "Bool" | "Σ(0,1)" => Self::Bool(BoolSpace::full()),
            "N" | "Int" => Self::Int(IntSpace::new()),
            "String" => Self::String(StringSpace::new()),
            "∅" | "Void" => Self::Empty, // Void has no values
            _ if ty.name.starts_with("σ[") => Self::Sequence(Box::new(Self::All)),
            _ => Self::All,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXHAUSTIVENESS RESULT — ∂ (Boundary: complete or not)
// ═══════════════════════════════════════════════════════════════════════════

/// Result of exhaustiveness checking.
///
/// ## Tier: T2-P (∂ + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExhaustivenessResult {
    /// Whether the match is exhaustive.
    pub is_exhaustive: bool,
    /// Missing patterns (if non-exhaustive).
    pub missing: Vec<MissingPattern>,
    /// Redundant arms (covered by earlier patterns).
    pub redundant: Vec<RedundantArm>,
}

/// A missing pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingPattern {
    /// Description of the missing pattern.
    pub description: String,
    /// Example value that would match.
    pub example: Option<String>,
}

/// A redundant arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedundantArm {
    /// Index of the redundant arm.
    pub arm_index: usize,
    /// The pattern that's redundant.
    pub pattern: String,
    /// Which earlier arm covers it.
    pub covered_by: usize,
}

impl ExhaustivenessResult {
    /// Create an exhaustive result.
    #[must_use]
    pub fn exhaustive() -> Self {
        Self {
            is_exhaustive: true,
            missing: Vec::new(),
            redundant: Vec::new(),
        }
    }

    /// Create a non-exhaustive result.
    #[must_use]
    pub fn non_exhaustive(missing: Vec<MissingPattern>) -> Self {
        Self {
            is_exhaustive: false,
            missing,
            redundant: Vec::new(),
        }
    }

    /// Add a redundant arm.
    pub fn add_redundant(&mut self, arm_index: usize, pattern: String, covered_by: usize) {
        self.redundant.push(RedundantArm {
            arm_index,
            pattern,
            covered_by,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXHAUSTIVENESS CHECKER — κ (Comparison of patterns)
// ═══════════════════════════════════════════════════════════════════════════

/// Exhaustiveness checker.
///
/// ## Tier: T2-C (κ + Σ + ν)
#[derive(Debug, Default)]
pub struct ExhaustivenessChecker {
    /// Warnings generated.
    warnings: Vec<ExhaustivenessWarning>,
}

/// Warning from exhaustiveness checking.
#[derive(Debug, Clone)]
pub struct ExhaustivenessWarning {
    pub message: String,
    pub span: Span,
}

impl ExhaustivenessChecker {
    /// Create a new checker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check exhaustiveness of a match expression.
    #[must_use]
    pub fn check_match(
        &mut self,
        scrutinee_type: &PrimaType,
        arms: &[MatchArm],
        span: Span,
    ) -> ExhaustivenessResult {
        let mut space = PatternSpace::for_type(scrutinee_type);

        // Track which arms cover which patterns
        let mut wildcard_arm: Option<usize> = None;

        for (i, arm) in arms.iter().enumerate() {
            if self.pattern_is_wildcard(&arm.pattern) {
                if wildcard_arm.is_some() {
                    // Redundant wildcard
                    self.warnings.push(ExhaustivenessWarning {
                        message: format!("redundant pattern in arm {}", i + 1),
                        span,
                    });
                }
                wildcard_arm = Some(i);
            }

            self.subtract_pattern(&mut space, &arm.pattern);
        }

        // Check if exhausted
        if space.is_exhausted() {
            ExhaustivenessResult::exhaustive()
        } else {
            let missing = self.compute_missing(&space);
            ExhaustivenessResult::non_exhaustive(missing)
        }
    }

    /// Check if a pattern is a wildcard.
    fn pattern_is_wildcard(&self, pattern: &Pattern) -> bool {
        matches!(pattern, Pattern::Wildcard { .. })
    }

    /// Subtract a pattern from the space.
    fn subtract_pattern(&self, space: &mut PatternSpace, pattern: &Pattern) {
        match (space, pattern) {
            (
                PatternSpace::Bool(b),
                Pattern::Literal {
                    value: Literal::Bool(v),
                    ..
                },
            ) => {
                b.remove(*v);
            }
            (
                PatternSpace::Int(i),
                Pattern::Literal {
                    value: Literal::Int(v),
                    ..
                },
            ) => {
                i.cover(*v);
            }
            (
                PatternSpace::String(s),
                Pattern::Literal {
                    value: Literal::String(v),
                    ..
                },
            ) => {
                s.cover(v.clone());
            }
            (PatternSpace::Variants(v), Pattern::Constructor { name, .. }) => {
                v.remove(name);
            }
            (PatternSpace::Bool(b), Pattern::Wildcard { .. }) => {
                b.has_true = false;
                b.has_false = false;
            }
            (PatternSpace::Int(i), Pattern::Wildcard { .. }) => {
                i.cover_wildcard();
            }
            (PatternSpace::String(s), Pattern::Wildcard { .. }) => {
                s.cover_wildcard();
            }
            (PatternSpace::Variants(v), Pattern::Wildcard { .. }) => {
                v.clear();
            }
            (space @ PatternSpace::All, Pattern::Wildcard { .. }) => {
                *space = PatternSpace::Empty;
            }
            _ => {}
        }
    }

    /// Compute missing patterns from remaining space.
    fn compute_missing(&self, space: &PatternSpace) -> Vec<MissingPattern> {
        match space {
            PatternSpace::Empty | PatternSpace::Wildcard => vec![],
            PatternSpace::All => vec![MissingPattern {
                description: "_".to_string(),
                example: Some("any value".to_string()),
            }],
            PatternSpace::Bool(b) => b
                .missing()
                .into_iter()
                .map(|s| MissingPattern {
                    description: s.clone(),
                    example: Some(s),
                })
                .collect(),
            PatternSpace::Int(i) => {
                if i.is_exhausted() {
                    vec![]
                } else {
                    vec![MissingPattern {
                        description: "_".to_string(),
                        example: Some("other integers".to_string()),
                    }]
                }
            }
            PatternSpace::String(s) => {
                if s.is_exhausted() {
                    vec![]
                } else {
                    vec![MissingPattern {
                        description: "_".to_string(),
                        example: Some("other strings".to_string()),
                    }]
                }
            }
            PatternSpace::Sequence(_) => vec![MissingPattern {
                description: "_".to_string(),
                example: Some("sequences".to_string()),
            }],
            PatternSpace::Variants(v) => v
                .iter()
                .map(|name| MissingPattern {
                    description: name.clone(),
                    example: None,
                })
                .collect(),
        }
    }

    /// Get warnings.
    #[must_use]
    pub fn warnings(&self) -> &[ExhaustivenessWarning] {
        &self.warnings
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONVENIENCE FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Check if a match expression is exhaustive.
#[must_use]
pub fn is_exhaustive(scrutinee_type: &PrimaType, arms: &[MatchArm]) -> bool {
    let mut checker = ExhaustivenessChecker::new();
    let result = checker.check_match(scrutinee_type, arms, Span::default());
    result.is_exhaustive
}

/// Get primitive composition for exhaustiveness checking.
#[must_use]
pub fn exhaustive_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![
        LexPrimitiva::Sum,
        LexPrimitiva::Comparison,
        LexPrimitiva::Irreversibility,
        LexPrimitiva::Boundary,
    ])
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    fn ty_bool() -> PrimaType {
        PrimaType::new("Bool", PrimitiveComposition::new(vec![LexPrimitiva::Sum]))
    }

    fn ty_int() -> PrimaType {
        PrimaType::new("N", PrimitiveComposition::new(vec![LexPrimitiva::Quantity]))
    }

    #[test]
    fn test_bool_space_full() {
        let space = BoolSpace::full();
        assert!(space.has_true);
        assert!(space.has_false);
        assert!(!space.is_exhausted());
    }

    #[test]
    fn test_bool_space_remove() {
        let mut space = BoolSpace::full();
        space.remove(true);
        assert!(!space.has_true);
        assert!(space.has_false);

        space.remove(false);
        assert!(space.is_exhausted());
    }

    #[test]
    fn test_bool_space_missing() {
        let mut space = BoolSpace::full();
        space.remove(true);
        let missing = space.missing();
        assert_eq!(missing, vec!["false"]);
    }

    #[test]
    fn test_int_space() {
        let mut space = IntSpace::new();
        assert!(!space.is_exhausted());

        space.cover(1);
        space.cover(2);
        assert!(!space.is_exhausted());

        space.cover_wildcard();
        assert!(space.is_exhausted());
    }

    #[test]
    fn test_pattern_space_for_type() {
        let space = PatternSpace::for_type(&ty_bool());
        assert!(matches!(space, PatternSpace::Bool(_)));

        let space = PatternSpace::for_type(&ty_int());
        assert!(matches!(space, PatternSpace::Int(_)));
    }

    #[test]
    fn test_exhaustive_bool_match() {
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal {
                    value: Literal::Bool(true),
                    span: Span::default(),
                },
                body: Expr::Literal {
                    value: Literal::Int(1),
                    span: Span::default(),
                },
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Literal {
                    value: Literal::Bool(false),
                    span: Span::default(),
                },
                body: Expr::Literal {
                    value: Literal::Int(0),
                    span: Span::default(),
                },
                span: Span::default(),
            },
        ];

        assert!(is_exhaustive(&ty_bool(), &arms));
    }

    #[test]
    fn test_non_exhaustive_bool_match() {
        let arms = vec![MatchArm {
            pattern: Pattern::Literal {
                value: Literal::Bool(true),
                span: Span::default(),
            },
            body: Expr::Literal {
                value: Literal::Int(1),
                span: Span::default(),
            },
            span: Span::default(),
        }];

        assert!(!is_exhaustive(&ty_bool(), &arms));
    }

    #[test]
    fn test_wildcard_exhausts() {
        let arms = vec![MatchArm {
            pattern: Pattern::Wildcard {
                span: Span::default(),
            },
            body: Expr::Literal {
                value: Literal::Int(0),
                span: Span::default(),
            },
            span: Span::default(),
        }];

        assert!(is_exhaustive(&ty_bool(), &arms));
        assert!(is_exhaustive(&ty_int(), &arms));
    }

    #[test]
    fn test_int_needs_wildcard() {
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal {
                    value: Literal::Int(0),
                    span: Span::default(),
                },
                body: Expr::Literal {
                    value: Literal::String("zero".into()),
                    span: Span::default(),
                },
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Literal {
                    value: Literal::Int(1),
                    span: Span::default(),
                },
                body: Expr::Literal {
                    value: Literal::String("one".into()),
                    span: Span::default(),
                },
                span: Span::default(),
            },
        ];

        // Not exhaustive without wildcard
        assert!(!is_exhaustive(&ty_int(), &arms));
    }

    #[test]
    fn test_check_match_result() {
        let mut checker = ExhaustivenessChecker::new();
        let arms = vec![MatchArm {
            pattern: Pattern::Literal {
                value: Literal::Bool(true),
                span: Span::default(),
            },
            body: Expr::Literal {
                value: Literal::Int(1),
                span: Span::default(),
            },
            span: Span::default(),
        }];

        let result = checker.check_match(&ty_bool(), &arms, Span::default());
        assert!(!result.is_exhaustive);
        assert_eq!(result.missing.len(), 1);
        assert_eq!(result.missing[0].description, "false");
    }

    #[test]
    fn test_exhaustive_composition() {
        let comp = exhaustive_composition();
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Sum));
        assert!(unique.contains(&LexPrimitiva::Comparison));
    }
}
