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
use nexcore_lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};

/// A program is σ[Stmt] — a sequence of statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// The statements in the program.
    pub statements: Vec<Stmt>,
}

/// Statement — ς (State) modification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    /// `let x = e` — binding (ς)
    Let {
        /// The name of the variable.
        name: String,
        /// The initial value.
        value: Expr,
        /// The source span.
        span: Span,
    },
    /// `type T = ...` — type definition (μ)
    TypeDef {
        /// The name of the type.
        name: String,
        /// The type expression.
        ty: TypeExpr,
        /// The source span.
        span: Span,
    },
    /// `fn f(params) -> T { body }` — function (→)
    FnDef {
        /// The name of the function.
        name: String,
        /// The function parameters.
        params: Vec<Param>,
        /// The return type.
        ret: TypeExpr,
        /// The function body.
        body: Block,
        /// The source span.
        span: Span,
    },
    /// Expression statement
    Expr {
        /// The expression.
        expr: Expr,
        /// The source span.
        span: Span,
    },
    /// Return statement (∂ + →)
    Return {
        /// The optional return value.
        value: Option<Expr>,
        /// The source span.
        span: Span,
    },
}

/// Expression — produces a value (→).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value (N)
    Literal {
        /// The literal value.
        value: Literal,
        /// The source span.
        span: Span,
    },
    /// Variable reference (λ)
    Ident {
        /// The name of the variable.
        name: String,
        /// The source span.
        span: Span,
    },
    /// Binary operation (→)
    Binary {
        /// The left operand.
        left: Box<Expr>,
        /// The binary operator.
        op: BinOp,
        /// The right operand.
        right: Box<Expr>,
        /// The source span.
        span: Span,
    },
    /// Unary operation (→)
    Unary {
        /// The unary operator.
        op: UnOp,
        /// The operand.
        operand: Box<Expr>,
        /// The source span.
        span: Span,
    },
    /// Function call (→)
    Call {
        /// The name of the function to call.
        func: String,
        /// The arguments to the function.
        args: Vec<Expr>,
        /// The source span.
        span: Span,
    },
    /// Conditional (Σ + κ)
    If {
        /// The condition expression.
        cond: Box<Expr>,
        /// The then branch.
        then_branch: Block,
        /// The optional else branch.
        else_branch: Option<Block>,
        /// The source span.
        span: Span,
    },
    /// Pattern match (Σ + κ)
    Match {
        /// The expression to match against.
        scrutinee: Box<Expr>,
        /// The match arms.
        arms: Vec<MatchArm>,
        /// The source span.
        span: Span,
    },
    /// For loop (σ + ρ)
    For {
        /// The loop variable name.
        var: String,
        /// The iterator expression.
        iter: Box<Expr>,
        /// The loop body.
        body: Block,
        /// The source span.
        span: Span,
    },
    /// Block expression
    Block {
        /// The block of statements.
        block: Block,
        /// The source span.
        span: Span,
    },
    /// Lambda (→ + ρ)
    Lambda {
        /// The lambda parameters.
        params: Vec<Param>,
        /// The lambda body.
        body: Box<Expr>,
        /// The source span.
        span: Span,
    },
    /// Sequence literal σ[...]
    Sequence {
        /// The elements of the sequence.
        elements: Vec<Expr>,
        /// The source span.
        span: Span,
    },
    /// Mapping literal μ(k → v, ...)
    Mapping {
        /// The key-value pairs in the mapping.
        pairs: Vec<(Expr, Expr)>,
        /// The source span.
        span: Span,
    },
    /// Member access (λ)
    Member {
        /// The object expression.
        object: Box<Expr>,
        /// The field name.
        field: String,
        /// The source span.
        span: Span,
    },
    /// Method call (→)
    MethodCall {
        /// The receiver object.
        object: Box<Expr>,
        /// The method name.
        method: String,
        /// The arguments.
        args: Vec<Expr>,
        /// The source span.
        span: Span,
    },
    /// Quoted expression (ρ) — AST as data for homoiconicity
    /// `'expr` syntax: returns the AST node itself, not its evaluated value
    /// Grounding: ρ (Recursion) — self-reference, code-as-data
    Quoted {
        /// The quoted expression.
        expr: Box<Expr>,
        /// The source span.
        span: Span,
    },
    /// Quasiquoted expression (ρ + σ) — AST with selective unquoting
    /// `` `expr `` syntax: template with holes for unquote/unquote-splice
    /// Grounding: ρ (Recursion) + σ (Sequence) — template composition
    Quasiquoted {
        /// The quasiquoted expression.
        expr: Box<Expr>,
        /// The source span.
        span: Span,
    },
    /// Unquote within quasiquote (→) — evaluate expression
    /// `~expr` syntax: evaluate and insert result in quasiquote
    /// Grounding: → (Causality) — triggers evaluation
    Unquoted {
        /// The unquoted expression.
        expr: Box<Expr>,
        /// The source span.
        span: Span,
    },
    /// Unquote-splice within quasiquote (→ + σ) — evaluate and splice
    /// `~@expr` syntax: evaluate list and splice elements into quasiquote
    /// Grounding: → (Causality) + σ (Sequence) — evaluate then flatten
    UnquotedSplice {
        /// The unquote-splice expression.
        expr: Box<Expr>,
        /// The source span.
        span: Span,
    },
}

/// Block — σ[Stmt] with optional final expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// The statements in the block.
    pub statements: Vec<Stmt>,
    /// The optional final expression (implicit return).
    pub expr: Option<Box<Expr>>,
    /// The source span.
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
    /// Addition (N × N → N)
    Add,
    /// Subtraction (N × N → N)
    Sub,
    /// Multiplication (N × N → N)
    Mul,
    /// Division (N × N → N)
    Div,
    /// Modulo (N × N → N)
    Mod,
    /// Equality (κ)
    Eq,
    /// Inequality (κ)
    Ne,
    /// Less than (κ)
    Lt,
    /// Greater than (κ)
    Gt,
    /// Less than or equal (κ)
    Le,
    /// Greater than or equal (κ)
    Ge,
    /// Kappa equality (κ)
    KappaEq,
    /// Kappa less than (κ)
    KappaLt,
    /// Kappa greater than (κ)
    KappaGt,
    /// Logical AND (Σ × Σ → Σ)
    And,
    /// Logical OR (Σ × Σ → Σ)
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
            | Self::KappaLt
            | Self::KappaGt => LexPrimitiva::Comparison,
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
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    pub ty: TypeExpr,
    /// The source span.
    pub span: Span,
}

/// Match arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    /// The pattern to match.
    pub pattern: Pattern,
    /// The body expression to execute if the pattern matches.
    pub body: Expr,
    /// The source span.
    pub span: Span,
}

/// Pattern for matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard: matches anything
    Wildcard {
        /// The source span.
        span: Span,
    },
    /// Literal pattern
    Literal {
        /// The literal value to match.
        value: Literal,
        /// The source span.
        span: Span,
    },
    /// Binding pattern
    Ident {
        /// The name to bind the matched value to.
        name: String,
        /// The source span.
        span: Span,
    },
    /// Constructor pattern
    Constructor {
        /// The name of the constructor.
        name: String,
        /// The fields of the constructor pattern.
        fields: Vec<Pattern>,
        /// The source span.
        span: Span,
    },
}

/// Type expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeExpr {
    /// The kind of the type.
    pub kind: TypeKind,
    /// The source span.
    pub span: Span,
}

/// Type kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    /// Primitive type (T1)
    Primitive(LexPrimitiva),
    /// Named type
    Named(
        /// The name of the type.
        String,
    ),
    /// Sequence type: σ[T]
    Sequence(
        /// The element type.
        Box<TypeExpr>,
    ),
    /// Mapping type: μ[A → B]
    Mapping(
        /// The key type.
        Box<TypeExpr>,
        /// The value type.
        Box<TypeExpr>,
    ),
    /// Sum type: A | B
    Sum(
        /// The variants of the sum type.
        Vec<TypeExpr>,
    ),
    /// Function type: (A, B) → C
    Function(
        /// The parameter types.
        Vec<TypeExpr>,
        /// The return type.
        Box<TypeExpr>,
    ),
    /// Optional type: T | ∅
    Optional(
        /// The underlying type.
        Box<TypeExpr>,
    ),
    /// Void type
    Void,
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
