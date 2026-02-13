//! Abstract Syntax Tree for the nexcore-dna high-level language.
//!
//! Phase 4: Extended with statements, variables, control flow, and functions.
//! Every node grounds to T1 primitives.
//!
//! Tier: T3 (σ Sequence + → Causality + N Quantity + ∂ Boundary + ρ Recursion + ς State)

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// A single statement in the language.
///
/// Tier: T2-C (σ + ς + → + ∂)
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement: evaluate and output.
    /// `2 + 3` → outputs 5
    ExprStmt(Expr),

    /// Variable binding: `let x = expr`
    /// Grounds to ς State (new variable) + μ Mapping (name → addr).
    Let { name: String, value: Expr },

    /// Variable assignment: `x = expr`
    /// Grounds to ς State (mutation).
    Assign { name: String, value: Expr },

    /// Conditional: `if cond do body end` or `if cond do body else body end`
    /// Grounds to → Causality + ∂ Boundary.
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },

    /// While loop: `while cond do body end`
    /// Grounds to ρ Recursion + → Causality.
    While { cond: Expr, body: Vec<Stmt> },

    /// Function definition: `fn name(params) do body end`
    /// Grounds to ∂ Boundary (scope) + σ Sequence (body).
    FnDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    /// Return from function: `return expr`
    /// Grounds to → Causality.
    Return(Expr),

    /// For loop: `for var = start to end do body end`
    /// Grounds to σ Sequence + ρ Recursion + ∂ Boundary + N Quantity.
    For {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/// A single expression in the language.
///
/// Tier: T2-C (σ + → + N + Σ)
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal. Grounds to N Quantity.
    Lit(i64),

    /// Variable reference. Grounds to ς State.
    Var(String),

    /// Unary negation: `-expr`. Grounds to → Causality.
    Neg(Box<Expr>),

    /// Logical not: `not expr`. Grounds to → Causality.
    Not(Box<Expr>),

    /// Bitwise NOT: `~expr`. Grounds to → Causality.
    BitNot(Box<Expr>),

    /// Binary operation: `left op right`. Grounds to → Causality.
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    /// Function call: `name(args...)`. Grounds to → Causality + ∂ Boundary.
    Call { name: String, args: Vec<Expr> },
}

/// Binary operator.
///
/// Tier: T2-P (μ Mapping + → Causality)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic (precedence 4-5)
    /// Addition: `+` → GCA (Add)
    Add,
    /// Subtraction: `-` → GCT (Sub)
    Sub,
    /// Multiplication: `*` → GCG (Mul)
    Mul,
    /// Division: `/` → GCC (Div)
    Div,
    /// Modulo: `%` → TCA (Mod)
    Mod,

    // Comparison (precedence 3)
    /// Equal: `==` → GTA (Eq)
    Eq,
    /// Not equal: `!=` → GTC (Neq)
    Neq,
    /// Less than: `<` → GTT (Lt)
    Lt,
    /// Greater than: `>` → GTG (Gt)
    Gt,
    /// Less or equal: `<=` → emits Gt + push0 + Eq
    Le,
    /// Greater or equal: `>=` → emits Lt + push0 + Eq
    Ge,

    // Bitwise (precedence 4-7)
    /// Bitwise AND: `&`
    BitAnd,
    /// Bitwise OR: `|`
    BitOr,
    /// Bitwise XOR: `^`
    BitXor,
    /// Shift left: `<<`
    Shl,
    /// Shift right: `>>`
    Shr,

    // Logical (precedence 1-2)
    /// Logical and: `and` → CAA (And)
    And,
    /// Logical or: `or` → CAG (Or)
    Or,
}

impl BinOp {
    /// Precedence level for the Pratt parser.
    /// Higher number = tighter binding.
    ///
    /// | Level | Operators |
    /// |-------|-----------|
    /// | 1 | `or` |
    /// | 2 | `and` |
    /// | 3 | `==` `!=` `<` `>` `<=` `>=` |
    /// | 4 | `\|` (bitwise OR) |
    /// | 5 | `^` (bitwise XOR) |
    /// | 6 | `&` (bitwise AND) |
    /// | 7 | `<<` `>>` (shifts) |
    /// | 8 | `+` `-` |
    /// | 9 | `*` `/` `%` |
    pub fn precedence(self) -> u8 {
        match self {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => 3,
            BinOp::BitOr => 4,
            BinOp::BitXor => 5,
            BinOp::BitAnd => 6,
            BinOp::Shl | BinOp::Shr => 7,
            BinOp::Add | BinOp::Sub => 8,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 9,
        }
    }
}

impl core::fmt::Display for BinOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
            BinOp::And => write!(f, "and"),
            BinOp::Or => write!(f, "or"),
        }
    }
}

impl core::fmt::Display for Expr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Expr::Lit(n) => write!(f, "{n}"),
            Expr::Var(name) => write!(f, "{name}"),
            Expr::Neg(inner) => write!(f, "(-{inner})"),
            Expr::Not(inner) => write!(f, "(not {inner})"),
            Expr::BitNot(inner) => write!(f, "(~{inner})"),
            Expr::BinOp { left, op, right } => write!(f, "({left} {op} {right})"),
            Expr::Call { name, args } => {
                write!(f, "{name}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_literal() {
        let e = Expr::Lit(42);
        assert_eq!(format!("{e}"), "42");
    }

    #[test]
    fn display_var() {
        let e = Expr::Var("x".into());
        assert_eq!(format!("{e}"), "x");
    }

    #[test]
    fn display_binop() {
        let e = Expr::BinOp {
            left: Box::new(Expr::Lit(2)),
            op: BinOp::Add,
            right: Box::new(Expr::Lit(3)),
        };
        assert_eq!(format!("{e}"), "(2 + 3)");
    }

    #[test]
    fn display_neg() {
        let e = Expr::Neg(Box::new(Expr::Lit(5)));
        assert_eq!(format!("{e}"), "(-5)");
    }

    #[test]
    fn display_not() {
        let e = Expr::Not(Box::new(Expr::Var("flag".into())));
        assert_eq!(format!("{e}"), "(not flag)");
    }

    #[test]
    fn display_call() {
        let e = Expr::Call {
            name: "add".into(),
            args: vec![Expr::Lit(1), Expr::Lit(2)],
        };
        assert_eq!(format!("{e}"), "add(1, 2)");
    }

    #[test]
    fn display_comparison() {
        let e = Expr::BinOp {
            left: Box::new(Expr::Var("x".into())),
            op: BinOp::Le,
            right: Box::new(Expr::Lit(10)),
        };
        assert_eq!(format!("{e}"), "(x <= 10)");
    }

    #[test]
    fn precedence_order() {
        assert!(BinOp::Mul.precedence() > BinOp::Add.precedence());
        assert!(BinOp::Add.precedence() > BinOp::Shl.precedence());
        assert!(BinOp::Shl.precedence() > BinOp::BitAnd.precedence());
        assert!(BinOp::BitAnd.precedence() > BinOp::BitXor.precedence());
        assert!(BinOp::BitXor.precedence() > BinOp::BitOr.precedence());
        assert!(BinOp::BitOr.precedence() > BinOp::Eq.precedence());
        assert!(BinOp::Eq.precedence() > BinOp::And.precedence());
        assert!(BinOp::And.precedence() > BinOp::Or.precedence());
        assert_eq!(BinOp::Add.precedence(), BinOp::Sub.precedence());
        assert_eq!(BinOp::Mul.precedence(), BinOp::Div.precedence());
        assert_eq!(BinOp::Mul.precedence(), BinOp::Mod.precedence());
        assert_eq!(BinOp::Eq.precedence(), BinOp::Neq.precedence());
        assert_eq!(BinOp::Lt.precedence(), BinOp::Gt.precedence());
        assert_eq!(BinOp::Le.precedence(), BinOp::Ge.precedence());
        assert_eq!(BinOp::Shl.precedence(), BinOp::Shr.precedence());
    }

    #[test]
    fn display_bitwise_ops() {
        assert_eq!(format!("{}", BinOp::BitAnd), "&");
        assert_eq!(format!("{}", BinOp::BitOr), "|");
        assert_eq!(format!("{}", BinOp::BitXor), "^");
        assert_eq!(format!("{}", BinOp::Shl), "<<");
        assert_eq!(format!("{}", BinOp::Shr), ">>");
    }

    #[test]
    fn display_bitnot() {
        let e = Expr::BitNot(Box::new(Expr::Var("x".into())));
        assert_eq!(format!("{e}"), "(~x)");
    }
}
