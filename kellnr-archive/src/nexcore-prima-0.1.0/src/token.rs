// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Token Definitions
//!
//! Tokens for the Prima lexer, mathematically grounded in the 15 Lex Primitiva.
//!
//! ## Mathematical Foundation
//!
//! Every token grounds to the two root constants:
//! - **0** (zero): Absence, identity, origin
//! - **1** (one): Existence, unit, witness
//!
//! ## Token Grounding
//!
//! | Token Class | Dominant Primitive | Grounding Path |
//! |-------------|-------------------|----------------|
//! | Literals | N (Quantity) | N → 0,1 |
//! | Identifiers | λ (Location) | λ → ∃ → → → 1 |
//! | Operators | → (Causality) | → → 1 |
//! | Delimiters | ∂ (Boundary) | ∂ → κ → N → 0,1 |
//!
//! ## Tier: T2-P (σ + Σ)

use nexcore_lex_primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};
use std::fmt;

/// File extension for Prima source files.
///
/// σ (sigma) represents Sequence — a program is a sequence of statements.
pub const FILE_EXTENSION: &str = "σ";

/// Alternative ASCII file extension.
pub const FILE_EXTENSION_ASCII: &str = "prima";

/// Source location span.
///
/// Grounds to: λ (Location) → ∃ (Existence) → → (Causality) → 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset (grounded to N).
    pub start: usize,
    /// End byte offset (grounded to N).
    pub end: usize,
    /// Line number (grounded to N).
    pub line: usize,
}

impl Span {
    /// Create a new span.
    #[must_use]
    pub const fn new(start: usize, end: usize, line: usize) -> Self {
        Self { start, end, line }
    }

    /// Merge two spans (grounded to σ composition).
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
        }
    }

    /// Length of the span (grounded to N).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if span is empty (grounded to ∅).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "λ[{},{}]:L{}", self.start, self.end, self.line + 1)
    }
}

/// Prima token kinds.
///
/// Each variant grounds through the primitive dependency DAG to {0, 1}.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // ═══════════════════════════════════════════════════════════════════════
    // PRIMITIVES — The 15 Lex Primitiva symbols (T1)
    // Direct grounding to mathematical foundations
    // ═══════════════════════════════════════════════════════════════════════
    /// A Lex Primitiva symbol.
    /// Grounding: T1 → directly to {0, 1} via dependency DAG
    Primitive(LexPrimitiva),

    // ═══════════════════════════════════════════════════════════════════════
    // LITERALS — Grounded through N (Quantity)
    // N → 0, 1 (Peano axioms)
    // ═══════════════════════════════════════════════════════════════════════
    /// Integer literal: N → 0, 1
    Int(i64),
    /// Floating-point literal: N → 0, 1, π, e
    Float(f64),
    /// String literal: σ[N] → sequence of quantities
    String(String),
    /// Boolean literal: Σ(0, 1) → sum of root constants
    Bool(bool),

    // ═══════════════════════════════════════════════════════════════════════
    // IDENTIFIERS — Grounded through λ (Location)
    // λ → ∃ → → → 1
    // ═══════════════════════════════════════════════════════════════════════
    /// Identifier: λ (Location) pointing to a binding
    Ident(String),

    // ═══════════════════════════════════════════════════════════════════════
    // KEYWORDS — Reserved identifiers with semantic meaning
    // Each keyword maps to a primitive operation
    // ═══════════════════════════════════════════════════════════════════════
    /// `fn` — Function definition (→ Causality)
    Fn,
    /// `let` — Binding creation (ς State)
    Let,
    /// `type` — Type definition (μ Mapping)
    Type,
    /// `if` — Conditional branch (Σ Sum + κ Comparison)
    If,
    /// `else` — Alternative branch (Σ Sum)
    Else,
    /// `match` — Pattern matching (κ Comparison + Σ Sum)
    Match,
    /// `for` — Iteration (σ Sequence + ρ Recursion)
    For,
    /// `in` — Membership test (∃ Existence + κ Comparison)
    In,
    /// `return` — Early exit (∂ Boundary + → Causality)
    Return,
    /// `true` — Boolean 1
    True,
    /// `false` — Boolean 0
    False,

    // ═══════════════════════════════════════════════════════════════════════
    // OPERATORS — Grounded through → (Causality)
    // Operations that transform values
    // ═══════════════════════════════════════════════════════════════════════
    /// `→` Arrow: direct causality primitive
    Arrow,
    /// `|` Pipe: sum type separator (Σ)
    Pipe,
    /// `:` Colon: type annotation (μ Mapping)
    Colon,
    /// `=` Assignment: state mutation (ς)
    Equal,
    /// `==` Equality: comparison (κ)
    EqualEqual,
    /// `!=` Inequality: negated comparison (κ + ∅)
    NotEqual,
    /// `+` Addition: N × N → N
    Plus,
    /// `-` Subtraction: N × N → N
    Minus,
    /// `*` Multiplication: N × N → N
    Star,
    /// `/` Division: N × N → N (with ∂ boundary for zero)
    Slash,
    /// `%` Modulo: N × N → N
    Percent,
    /// `<` Less than: κ comparison
    Less,
    /// `>` Greater than: κ comparison
    Greater,
    /// `<=` Less or equal: κ comparison
    LessEqual,
    /// `>=` Greater or equal: κ comparison
    GreaterEqual,
    /// `&&` Logical and: Σ(0,1) × Σ(0,1) → Σ(0,1)
    And,
    /// `||` Logical or: Σ(0,1) × Σ(0,1) → Σ(0,1)
    Or,
    /// `!` Logical not: Σ(0,1) → Σ(0,1)
    Not,

    // ═══════════════════════════════════════════════════════════════════════
    // KAPPA OPERATORS — Explicit comparison primitive (κ)
    // Syntactic sugar making primitive grounding explicit
    // ═══════════════════════════════════════════════════════════════════════
    /// `κ=` Kappa equals: explicit comparison
    KappaEq,
    /// `κ<` Kappa less: explicit comparison
    KappaLt,
    /// `κ>` Kappa greater: explicit comparison
    KappaGt,

    // ═══════════════════════════════════════════════════════════════════════
    // DELIMITERS — Grounded through ∂ (Boundary)
    // Structural boundaries in syntax
    // ═══════════════════════════════════════════════════════════════════════
    /// `(` Left paren: boundary start
    LParen,
    /// `)` Right paren: boundary end
    RParen,
    /// `{` Left brace: block boundary start
    LBrace,
    /// `}` Right brace: block boundary end
    RBrace,
    /// `[` Left bracket: sequence boundary start
    LBracket,
    /// `]` Right bracket: sequence boundary end
    RBracket,
    /// `,` Comma: element separator in σ
    Comma,
    /// `;` Semicolon: statement separator in σ
    Semicolon,
    /// `.` Dot: member access (λ Location)
    Dot,
    /// `_` Wildcard: universal pattern (∀ implicit)
    Underscore,

    // ═══════════════════════════════════════════════════════════════════════
    // HOMOICONICITY — Code as Data (ρ self-reference)
    // Required for self-hosting: Symbol + Quote + Quasiquote + Eval
    // ═══════════════════════════════════════════════════════════════════════
    /// `:name` Symbol: interned identifier (λ Location)
    /// Grounding: λ → ∃ → → → 1
    Symbol(String),
    /// `'` Quote prefix: AST as data (ρ Recursion)
    /// Grounding: ρ → σ → N → {0, 1}
    Quote,
    /// `` ` `` Quasiquote: AST with selective unquoting (ρ + σ)
    /// Grounding: ρ → σ → N → {0, 1}
    Quasiquote,
    /// `~` Unquote: evaluate within quasiquote (→ Causality)
    /// Grounding: → → 1
    Unquote,
    /// `~@` Unquote-splice: evaluate and splice (→ + σ)
    /// Grounding: → → σ → N → {0, 1}
    UnquoteSplice,

    // ═══════════════════════════════════════════════════════════════════════
    // SPECIAL — Meta-tokens
    // ═══════════════════════════════════════════════════════════════════════
    /// End of file: ∂ ultimate boundary
    Eof,
    /// Newline: σ statement separator
    Newline,
}

impl TokenKind {
    /// Returns the dominant primitive for this token kind.
    #[must_use]
    pub const fn dominant_primitive(&self) -> LexPrimitiva {
        match self {
            Self::Primitive(p) => *p,
            Self::Int(_) | Self::Float(_) => LexPrimitiva::Quantity,
            Self::String(_) => LexPrimitiva::Sequence,
            Self::Bool(_) => LexPrimitiva::Sum,
            Self::Ident(_) | Self::Symbol(_) => LexPrimitiva::Location, // λ
            Self::Quote | Self::Quasiquote => LexPrimitiva::Recursion,  // ρ (self-reference)
            Self::Unquote | Self::UnquoteSplice => LexPrimitiva::Causality, // → (evaluate)
            Self::Fn | Self::Arrow | Self::Return => LexPrimitiva::Causality,
            Self::Let | Self::Equal => LexPrimitiva::State,
            Self::Type | Self::Colon => LexPrimitiva::Mapping,
            Self::If | Self::Else | Self::Match | Self::Pipe => LexPrimitiva::Sum,
            Self::For => LexPrimitiva::Sequence,
            Self::In
            | Self::EqualEqual
            | Self::NotEqual
            | Self::Less
            | Self::Greater
            | Self::LessEqual
            | Self::GreaterEqual
            | Self::KappaEq
            | Self::KappaLt
            | Self::KappaGt => LexPrimitiva::Comparison,
            Self::True | Self::False => LexPrimitiva::Quantity,
            Self::Plus | Self::Minus | Self::Star | Self::Slash | Self::Percent => {
                LexPrimitiva::Quantity
            }
            Self::And | Self::Or | Self::Not => LexPrimitiva::Sum,
            Self::LParen
            | Self::RParen
            | Self::LBrace
            | Self::RBrace
            | Self::LBracket
            | Self::RBracket
            | Self::Eof => LexPrimitiva::Boundary,
            Self::Comma | Self::Semicolon | Self::Newline => LexPrimitiva::Sequence,
            Self::Dot => LexPrimitiva::Location,
            Self::Underscore => LexPrimitiva::Void,
        }
    }

    /// Returns true if this is a primitive token.
    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    /// Returns true if this is a literal token.
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::Int(_) | Self::Float(_) | Self::String(_) | Self::Bool(_) | Self::Symbol(_)
        )
    }

    /// Returns true if this is a keyword.
    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Fn
                | Self::Let
                | Self::Type
                | Self::If
                | Self::Else
                | Self::Match
                | Self::For
                | Self::In
                | Self::Return
                | Self::True
                | Self::False
        )
    }

    /// Returns true if this is an operator.
    #[must_use]
    pub const fn is_operator(&self) -> bool {
        matches!(
            self,
            Self::Arrow
                | Self::Pipe
                | Self::Colon
                | Self::Equal
                | Self::EqualEqual
                | Self::NotEqual
                | Self::Plus
                | Self::Minus
                | Self::Star
                | Self::Slash
                | Self::Percent
                | Self::Less
                | Self::Greater
                | Self::LessEqual
                | Self::GreaterEqual
                | Self::And
                | Self::Or
                | Self::Not
                | Self::KappaEq
                | Self::KappaLt
                | Self::KappaGt
        )
    }

    /// Returns true if this is a delimiter.
    #[must_use]
    pub const fn is_delimiter(&self) -> bool {
        matches!(
            self,
            Self::LParen
                | Self::RParen
                | Self::LBrace
                | Self::RBrace
                | Self::LBracket
                | Self::RBracket
                | Self::Comma
                | Self::Semicolon
                | Self::Dot
                | Self::Underscore
        )
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(p) => write!(f, "{}", p.symbol()),
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Bool(b) => write!(f, "{}", if *b { "1" } else { "0" }),
            Self::Ident(s) => write!(f, "{}", s),
            Self::Symbol(s) => write!(f, ":{}", s), // λ: interned symbol
            Self::Quote => write!(f, "'"),          // ρ: quote prefix
            Self::Quasiquote => write!(f, "`"),     // ρ+σ: quasiquote
            Self::Unquote => write!(f, "~"),        // →: unquote
            Self::UnquoteSplice => write!(f, "~@"), // →+σ: unquote-splice
            Self::Fn => write!(f, "fn"),
            Self::Let => write!(f, "let"),
            Self::Type => write!(f, "type"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::Match => write!(f, "match"),
            Self::For => write!(f, "for"),
            Self::In => write!(f, "in"),
            Self::Return => write!(f, "return"),
            Self::True => write!(f, "⊤"),  // Mathematical true
            Self::False => write!(f, "⊥"), // Mathematical false
            Self::Arrow => write!(f, "→"),
            Self::Pipe => write!(f, "|"),
            Self::Colon => write!(f, ":"),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => write!(f, "≡"), // Mathematical equivalence
            Self::NotEqual => write!(f, "≢"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "−"), // Mathematical minus
            Self::Star => write!(f, "×"),  // Mathematical times
            Self::Slash => write!(f, "÷"), // Mathematical divide
            Self::Percent => write!(f, "mod"),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::LessEqual => write!(f, "≤"),
            Self::GreaterEqual => write!(f, "≥"),
            Self::And => write!(f, "∧"), // Mathematical and
            Self::Or => write!(f, "∨"),  // Mathematical or
            Self::Not => write!(f, "¬"), // Mathematical not
            Self::KappaEq => write!(f, "κ="),
            Self::KappaLt => write!(f, "κ<"),
            Self::KappaGt => write!(f, "κ>"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::Comma => write!(f, ","),
            Self::Semicolon => write!(f, ";"),
            Self::Dot => write!(f, "."),
            Self::Underscore => write!(f, "_"),
            Self::Eof => write!(f, "∎"), // Mathematical end of proof
            Self::Newline => write!(f, "↵"),
        }
    }
}

/// A token with its span.
///
/// Composition: Σ (token kind) + λ (location)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// Token kind (Σ — one of many variants).
    pub kind: TokenKind,
    /// Source span (λ — location in source).
    pub span: Span,
}

impl Token {
    /// Create a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Check if this is an EOF token.
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    /// Get the dominant primitive.
    #[must_use]
    pub const fn dominant_primitive(&self) -> LexPrimitiva {
        self.kind.dominant_primitive()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.kind, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_extension() {
        assert_eq!(FILE_EXTENSION, "σ");
        assert_eq!(FILE_EXTENSION_ASCII, "prima");
    }

    #[test]
    fn test_span_grounding() {
        let span = Span::new(0, 10, 0);
        // Span grounds to λ (Location)
        assert_eq!(span.start, 0); // 0 is a root constant
        assert_eq!(span.len(), 10); // Length is N
    }

    #[test]
    fn test_span_merge() {
        let a = Span::new(5, 10, 1);
        let b = Span::new(0, 15, 0);
        let merged = a.merge(b);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_span_display_shows_lambda() {
        let span = Span::new(5, 10, 2);
        let s = format!("{}", span);
        assert!(s.contains("λ")); // λ for Location
    }

    #[test]
    fn test_literal_grounding() {
        assert_eq!(
            TokenKind::Int(42).dominant_primitive(),
            LexPrimitiva::Quantity
        );
        assert_eq!(
            TokenKind::Float(3.14).dominant_primitive(),
            LexPrimitiva::Quantity
        );
        assert_eq!(
            TokenKind::String("hi".into()).dominant_primitive(),
            LexPrimitiva::Sequence
        );
        assert_eq!(
            TokenKind::Bool(true).dominant_primitive(),
            LexPrimitiva::Sum
        );
    }

    #[test]
    fn test_operator_grounding() {
        assert_eq!(
            TokenKind::Arrow.dominant_primitive(),
            LexPrimitiva::Causality
        );
        assert_eq!(TokenKind::Plus.dominant_primitive(), LexPrimitiva::Quantity);
        assert_eq!(
            TokenKind::EqualEqual.dominant_primitive(),
            LexPrimitiva::Comparison
        );
    }

    #[test]
    fn test_delimiter_grounding() {
        assert_eq!(
            TokenKind::LParen.dominant_primitive(),
            LexPrimitiva::Boundary
        );
        assert_eq!(TokenKind::Eof.dominant_primitive(), LexPrimitiva::Boundary);
    }

    #[test]
    fn test_keyword_grounding() {
        assert_eq!(TokenKind::Fn.dominant_primitive(), LexPrimitiva::Causality);
        assert_eq!(TokenKind::Let.dominant_primitive(), LexPrimitiva::State);
        assert_eq!(TokenKind::If.dominant_primitive(), LexPrimitiva::Sum);
        assert_eq!(TokenKind::For.dominant_primitive(), LexPrimitiva::Sequence);
    }

    #[test]
    fn test_mathematical_display() {
        // Operators display as mathematical symbols
        assert_eq!(format!("{}", TokenKind::EqualEqual), "≡");
        assert_eq!(format!("{}", TokenKind::And), "∧");
        assert_eq!(format!("{}", TokenKind::Or), "∨");
        assert_eq!(format!("{}", TokenKind::Not), "¬");
        assert_eq!(format!("{}", TokenKind::LessEqual), "≤");
        assert_eq!(format!("{}", TokenKind::Eof), "∎");
    }

    #[test]
    fn test_boolean_display_as_constants() {
        assert_eq!(format!("{}", TokenKind::True), "⊤");
        assert_eq!(format!("{}", TokenKind::False), "⊥");
        assert_eq!(format!("{}", TokenKind::Bool(true)), "1");
        assert_eq!(format!("{}", TokenKind::Bool(false)), "0");
    }

    #[test]
    fn test_all_primitives_have_dominant() {
        // Every token kind must ground to a primitive
        let kinds = vec![
            TokenKind::Int(0),
            TokenKind::Float(0.0),
            TokenKind::String(String::new()),
            TokenKind::Bool(true),
            TokenKind::Ident(String::new()),
            TokenKind::Fn,
            TokenKind::Let,
            TokenKind::Arrow,
            TokenKind::LParen,
            TokenKind::Eof,
        ];
        for kind in kinds {
            let prim = kind.dominant_primitive();
            // All primitives eventually ground to {0, 1}
            assert!(LexPrimitiva::all().contains(&prim));
        }
    }

    #[test]
    fn test_token_classification() {
        assert!(TokenKind::Primitive(LexPrimitiva::Sequence).is_primitive());
        assert!(TokenKind::Int(42).is_literal());
        assert!(TokenKind::Fn.is_keyword());
        assert!(TokenKind::Plus.is_operator());
        assert!(TokenKind::LParen.is_delimiter());
    }
}
