// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Abstract Syntax Tree
//!
//! AST types grounded in primitive compositions.
//!
//! ## Mathematical Foundation
//!
//! The AST is a tree structure grounded in ρ (Recursion):
//! - Nodes are ς (State) — data at a point
//! - Children form σ (Sequence) — ordered list
//! - Variants are Σ (Sum) — one of many possibilities
//!
//! ## Tier: T2-C (σ + ρ + Σ + ς)

use crate::token::Span;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};

/// A program is σ[Stmt] — a sequence of statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// Statement — ς (State) modification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    /// `let x = e` — binding (ς)
    Let {
        name: String,
        value: Expr,
        span: Span,
    },
    /// `type T = ...` — type definition (μ)
    TypeDef {
        name: String,
        ty: TypeExpr,
        span: Span,
    },
    /// `fn f(params) -> T { body }` — function (→)
    FnDef {
        name: String,
        params: Vec<Param>,
        ret: TypeExpr,
        body: Block,
        span: Span,
    },
    /// Expression statement
    Expr { expr: Expr, span: Span },
    /// Return statement (∂ + →)
    Return { value: Option<Expr>, span: Span },
}

/// Expression — produces a value (→).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value (N)
    Literal { value: Literal, span: Span },
    /// Variable reference (λ)
    Ident { name: String, span: Span },
    /// Binary operation (→)
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },
    /// Unary operation (→)
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    /// Function call (→)
    Call {
        func: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// Conditional (Σ + κ)
    If {
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
        span: Span,
    },
    /// Pattern match (Σ + κ)
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// For loop (σ + ρ)
    For {
        var: String,
        iter: Box<Expr>,
        body: Block,
        span: Span,
    },
    /// Block expression
    Block { block: Block, span: Span },
    /// Lambda (→ + ρ)
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
        span: Span,
    },
    /// Sequence literal σ[...]
    Sequence { elements: Vec<Expr>, span: Span },
    /// Mapping literal μ(k → v, ...)
    Mapping {
        pairs: Vec<(Expr, Expr)>,
        span: Span,
    },
    /// Member access (λ)
    Member {
        object: Box<Expr>,
        field: String,
        span: Span,
    },
    /// Method call (→)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// Quoted expression (ρ) — AST as data for homoiconicity
    /// `'expr` syntax: returns the AST node itself, not its evaluated value
    /// Grounding: ρ (Recursion) — self-reference, code-as-data
    Quoted { expr: Box<Expr>, span: Span },
    /// Quasiquoted expression (ρ + σ) — AST with selective unquoting
    /// `` `expr `` syntax: template with holes for unquote/unquote-splice
    /// Grounding: ρ (Recursion) + σ (Sequence) — template composition
    Quasiquoted { expr: Box<Expr>, span: Span },
    /// Unquote within quasiquote (→) — evaluate expression
    /// `~expr` syntax: evaluate and insert result in quasiquote
    /// Grounding: → (Causality) — triggers evaluation
    Unquoted { expr: Box<Expr>, span: Span },
    /// Unquote-splice within quasiquote (→ + σ) — evaluate and splice
    /// `~@expr` syntax: evaluate list and splice elements into quasiquote
    /// Grounding: → (Causality) + σ (Sequence) — evaluate then flatten
    UnquotedSplice { expr: Box<Expr>, span: Span },
}

/// Block — σ[Stmt] with optional final expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

/// Literal values — grounded to root constants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    /// Integer: N → {0, 1}
    Int(i64),
    /// Float: N → {0, 1, π, e}
    Float(f64),
    /// String: σ[N]
    String(String),
    /// Boolean: Σ(0, 1)
    Bool(bool),
    /// Void: ∅
    Void,
    /// Symbol: λ → ∃ → → → 1 (interned identifier for homoiconicity)
    /// `:name` syntax, evaluates to itself
    Symbol(String),
}

impl Literal {
    /// Get the grounding composition.
    #[must_use]
    pub fn composition(&self) -> PrimitiveComposition {
        match self {
            Self::Int(_) | Self::Float(_) => {
                PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            }
            Self::String(_) => {
                PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity])
            }
            Self::Bool(_) => PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
            Self::Void => PrimitiveComposition::new(vec![LexPrimitiva::Void]),
            // Symbol: λ (Location) — interned reference, T1 primitive
            Self::Symbol(_) => PrimitiveComposition::new(vec![LexPrimitiva::Location]),
        }
    }
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic (N × N → N)
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison (κ)
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    KappaEq,
    KappaNe,
    KappaLt,
    KappaGt,
    KappaLe,
    KappaGe,
    // Logical (Σ × Σ → Σ)
    And,
    Or,
}

impl BinOp {
    /// Get the dominant primitive.
    #[must_use]
    pub const fn dominant_primitive(&self) -> LexPrimitiva {
        match self {
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod => LexPrimitiva::Quantity,
            Self::Eq
            | Self::Ne
            | Self::Lt
            | Self::Gt
            | Self::Le
            | Self::Ge
            | Self::KappaEq
            | Self::KappaNe
            | Self::KappaLt
            | Self::KappaGt
            | Self::KappaLe
            | Self::KappaGe => LexPrimitiva::Comparison,
            Self::And | Self::Or => LexPrimitiva::Sum,
        }
    }
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    /// Negation: N → N
    Neg,
    /// Logical not: Σ → Σ
    Not,
}

/// Function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Match arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

/// Pattern for matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard: matches anything
    Wildcard { span: Span },
    /// Literal pattern
    Literal { value: Literal, span: Span },
    /// Binding pattern
    Ident { name: String, span: Span },
    /// Constructor pattern
    Constructor {
        name: String,
        fields: Vec<Pattern>,
        span: Span,
    },
}

/// Type expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeExpr {
    pub kind: TypeKind,
    pub span: Span,
}

/// Type kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    /// Primitive type (T1)
    Primitive(LexPrimitiva),
    /// Named type
    Named(String),
    /// Sequence type: σ[T]
    Sequence(Box<TypeExpr>),
    /// Mapping type: μ[A → B]
    Mapping(Box<TypeExpr>, Box<TypeExpr>),
    /// Sum type: A | B
    Sum(Vec<TypeExpr>),
    /// Function type: (A, B) → C
    Function(Vec<TypeExpr>, Box<TypeExpr>),
    /// Optional type: T | ∅
    Optional(Box<TypeExpr>),
    /// Void type
    Void,
    /// Inferred type (for lambda params)
    Infer,
}

impl TypeKind {
    /// Get the grounding composition.
    #[must_use]
    pub fn composition(&self) -> PrimitiveComposition {
        match self {
            Self::Primitive(p) => PrimitiveComposition::new(vec![*p]),
            Self::Named(_) => PrimitiveComposition::new(vec![LexPrimitiva::Location]),
            Self::Sequence(inner) => {
                let mut c = inner.kind.composition();
                c.primitives.insert(0, LexPrimitiva::Sequence);
                c
            }
            Self::Mapping(_, _) => PrimitiveComposition::new(vec![LexPrimitiva::Mapping]),
            Self::Sum(_) => PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
            Self::Function(_, _) => PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
            Self::Optional(inner) => {
                let mut c = inner.kind.composition();
                c.primitives.push(LexPrimitiva::Void);
                c
            }
            Self::Void => PrimitiveComposition::new(vec![LexPrimitiva::Void]),
            Self::Infer => PrimitiveComposition::new(vec![LexPrimitiva::Existence]),
        }
    }
}

impl Expr {
    /// Get the span of this expression.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Literal { span, .. }
            | Self::Ident { span, .. }
            | Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Call { span, .. }
            | Self::If { span, .. }
            | Self::Match { span, .. }
            | Self::For { span, .. }
            | Self::Block { span, .. }
            | Self::Lambda { span, .. }
            | Self::Sequence { span, .. }
            | Self::Mapping { span, .. }
            | Self::Member { span, .. }
            | Self::MethodCall { span, .. }
            | Self::Quoted { span, .. }
            | Self::Quasiquoted { span, .. }
            | Self::Unquoted { span, .. }
            | Self::UnquotedSplice { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_grounding() {
        let int = Literal::Int(42);
        assert!(
            int.composition()
                .primitives
                .contains(&LexPrimitiva::Quantity)
        );

        let s = Literal::String("hello".into());
        assert!(s.composition().primitives.contains(&LexPrimitiva::Sequence));

        let b = Literal::Bool(true);
        assert!(b.composition().primitives.contains(&LexPrimitiva::Sum));

        let v = Literal::Void;
        assert!(v.composition().primitives.contains(&LexPrimitiva::Void));
    }

    #[test]
    fn test_binop_grounding() {
        assert_eq!(BinOp::Add.dominant_primitive(), LexPrimitiva::Quantity);
        assert_eq!(BinOp::Eq.dominant_primitive(), LexPrimitiva::Comparison);
        assert_eq!(BinOp::And.dominant_primitive(), LexPrimitiva::Sum);
    }

    #[test]
    fn test_type_composition() {
        let seq = TypeKind::Sequence(Box::new(TypeExpr {
            kind: TypeKind::Primitive(LexPrimitiva::Quantity),
            span: Span::default(),
        }));
        assert!(
            seq.composition()
                .primitives
                .contains(&LexPrimitiva::Sequence)
        );
    }
}
