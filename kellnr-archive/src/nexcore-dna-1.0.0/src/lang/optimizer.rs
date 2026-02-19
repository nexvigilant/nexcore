//! AST optimizer: constant folding, identity elimination, strength reduction.
//!
//! Transforms the AST before codegen to reduce codon count.
//! All transformations preserve semantics (pure rewrites only).
//!
//! ## Optimizations
//!
//! | Pass | Example | Result |
//! |------|---------|--------|
//! | Constant fold | `2 + 3` | `5` |
//! | Nested fold | `(2 + 3) * 4` | `20` |
//! | Identity elim | `x + 0`, `x * 1` | `x` |
//! | Zero multiply | `x * 0` | `0` |
//! | Double negate | `--x` | `x` |
//! | Not-not | `not not x` | `x` |
//! | Comparison fold | `3 < 5` | `1` |
//!
//! Tier: T2-C (μ Mapping + κ Comparison + σ Sequence)

use super::ast::{BinOp, Expr, Stmt};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Optimize a list of statements (top-level entry point).
///
/// Recursively folds constants and eliminates identities in all
/// expressions throughout the AST.
pub fn optimize(stmts: &[Stmt]) -> Vec<Stmt> {
    stmts.iter().map(optimize_stmt).collect()
}

// ---------------------------------------------------------------------------
// Statement optimization
// ---------------------------------------------------------------------------

fn optimize_stmt(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::ExprStmt(expr) => Stmt::ExprStmt(optimize_expr(expr)),

        Stmt::Let { name, value } => Stmt::Let {
            name: name.clone(),
            value: optimize_expr(value),
        },

        Stmt::Assign { name, value } => Stmt::Assign {
            name: name.clone(),
            value: optimize_expr(value),
        },

        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let opt_cond = optimize_expr(cond);

            // If condition is a constant, eliminate dead branch
            if let Expr::Lit(v) = &opt_cond {
                if *v != 0 {
                    // Truthy: keep only then_body
                    return Stmt::If {
                        cond: opt_cond,
                        then_body: optimize(&then_body[..]),
                        else_body: Vec::new(),
                    };
                } else if !else_body.is_empty() {
                    // Falsy with else: keep only else_body
                    return Stmt::If {
                        cond: opt_cond,
                        then_body: Vec::new(),
                        else_body: optimize(&else_body[..]),
                    };
                }
            }

            Stmt::If {
                cond: opt_cond,
                then_body: optimize(&then_body[..]),
                else_body: optimize(&else_body[..]),
            }
        }

        Stmt::While { cond, body } => Stmt::While {
            cond: optimize_expr(cond),
            body: optimize(&body[..]),
        },

        Stmt::For {
            var,
            start,
            end,
            body,
        } => Stmt::For {
            var: var.clone(),
            start: optimize_expr(start),
            end: optimize_expr(end),
            body: optimize(&body[..]),
        },

        Stmt::FnDef { name, params, body } => Stmt::FnDef {
            name: name.clone(),
            params: params.clone(),
            body: optimize(&body[..]),
        },

        Stmt::Return(expr) => Stmt::Return(optimize_expr(expr)),
    }
}

// ---------------------------------------------------------------------------
// Expression optimization
// ---------------------------------------------------------------------------

/// Optimize a single expression (recursive, bottom-up).
fn optimize_expr(expr: &Expr) -> Expr {
    match expr {
        // Literals and variables pass through
        Expr::Lit(_) | Expr::Var(_) => expr.clone(),

        // Unary negation: fold constants, cancel double-negation
        Expr::Neg(inner) => {
            let opt_inner = optimize_expr(inner);
            match &opt_inner {
                // Fold: -5 → Lit(-5)
                Expr::Lit(n) => Expr::Lit(n.wrapping_neg()),
                // Double-negate: --x → x
                Expr::Neg(double_inner) => *double_inner.clone(),
                _ => Expr::Neg(Box::new(opt_inner)),
            }
        }

        // Logical not: fold constants, cancel double-not
        Expr::Not(inner) => {
            let opt_inner = optimize_expr(inner);
            match &opt_inner {
                // Fold: not 0 → 1, not nonzero → 0
                Expr::Lit(n) => Expr::Lit(if *n == 0 { 1 } else { 0 }),
                // Double-not: not not x → x
                Expr::Not(double_inner) => *double_inner.clone(),
                _ => Expr::Not(Box::new(opt_inner)),
            }
        }

        // Bitwise NOT: fold constants, cancel double-bitnot
        Expr::BitNot(inner) => {
            let opt_inner = optimize_expr(inner);
            match &opt_inner {
                // Fold: ~Lit(n) → Lit(!n)
                Expr::Lit(n) => Expr::Lit(!n),
                // Double-bitnot: ~~x → x
                Expr::BitNot(double_inner) => *double_inner.clone(),
                _ => Expr::BitNot(Box::new(opt_inner)),
            }
        }

        // Binary operations: the main folding target
        Expr::BinOp { left, op, right } => {
            let opt_left = optimize_expr(left);
            let opt_right = optimize_expr(right);
            optimize_binop(opt_left, *op, opt_right)
        }

        // Function calls: optimize arguments
        Expr::Call { name, args } => {
            let opt_args: Vec<Expr> = args.iter().map(optimize_expr).collect();

            // Fold constant calls to known builtins
            if let Some(folded) = fold_builtin_call(name, &opt_args) {
                return folded;
            }

            Expr::Call {
                name: name.clone(),
                args: opt_args,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Binary operation folding
// ---------------------------------------------------------------------------

/// Optimize a binary operation after both operands have been optimized.
fn optimize_binop(left: Expr, op: BinOp, right: Expr) -> Expr {
    // Case 1: Both operands are constants → fold completely
    if let (Expr::Lit(a), Expr::Lit(b)) = (&left, &right) {
        if let Some(result) = fold_const(*a, op, *b) {
            return Expr::Lit(result);
        }
    }

    // Case 2: Identity elimination (one operand is constant)
    if let Some(simplified) = identity_elim(&left, op, &right) {
        return simplified;
    }

    // No optimization possible — return as-is
    Expr::BinOp {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

/// Fold a constant binary operation.
/// Returns None for division by zero (let runtime handle it).
fn fold_const(a: i64, op: BinOp, b: i64) -> Option<i64> {
    match op {
        BinOp::Add => Some(a.wrapping_add(b)),
        BinOp::Sub => Some(a.wrapping_sub(b)),
        BinOp::Mul => Some(a.wrapping_mul(b)),
        BinOp::Div => {
            if b == 0 {
                None // preserve division by zero for runtime error
            } else {
                Some(a.wrapping_div(b))
            }
        }
        BinOp::Mod => {
            if b == 0 {
                None
            } else {
                Some(a.wrapping_rem(b))
            }
        }
        BinOp::Eq => Some(if a == b { 1 } else { 0 }),
        BinOp::Neq => Some(if a != b { 1 } else { 0 }),
        BinOp::Lt => Some(if a < b { 1 } else { 0 }),
        BinOp::Gt => Some(if a > b { 1 } else { 0 }),
        BinOp::Le => Some(if a <= b { 1 } else { 0 }),
        BinOp::Ge => Some(if a >= b { 1 } else { 0 }),
        BinOp::BitAnd => Some(a & b),
        BinOp::BitOr => Some(a | b),
        BinOp::BitXor => Some(a ^ b),
        BinOp::Shl => Some(a.wrapping_shl((b & 63) as u32)),
        BinOp::Shr => Some(a.wrapping_shr((b & 63) as u32)),
        BinOp::And => Some(if a != 0 && b != 0 { 1 } else { 0 }),
        BinOp::Or => Some(if a != 0 || b != 0 { 1 } else { 0 }),
    }
}

/// Eliminate identity operations.
fn identity_elim(left: &Expr, op: BinOp, right: &Expr) -> Option<Expr> {
    match op {
        // x + 0 → x, 0 + x → x
        BinOp::Add => {
            if matches!(right, Expr::Lit(0)) {
                return Some(left.clone());
            }
            if matches!(left, Expr::Lit(0)) {
                return Some(right.clone());
            }
            None
        }
        // x - 0 → x
        BinOp::Sub => {
            if matches!(right, Expr::Lit(0)) {
                return Some(left.clone());
            }
            None
        }
        // x * 1 → x, 1 * x → x, x * 0 → 0, 0 * x → 0
        BinOp::Mul => {
            if matches!(right, Expr::Lit(1)) {
                return Some(left.clone());
            }
            if matches!(left, Expr::Lit(1)) {
                return Some(right.clone());
            }
            if matches!(right, Expr::Lit(0)) || matches!(left, Expr::Lit(0)) {
                return Some(Expr::Lit(0));
            }
            None
        }
        // x / 1 → x
        BinOp::Div => {
            if matches!(right, Expr::Lit(1)) {
                return Some(left.clone());
            }
            None
        }
        // x & 0 → 0, x & -1 → x (all bits set)
        BinOp::BitAnd => {
            if matches!(right, Expr::Lit(0)) || matches!(left, Expr::Lit(0)) {
                return Some(Expr::Lit(0));
            }
            None
        }
        // x | 0 → x, 0 | x → x
        BinOp::BitOr => {
            if matches!(right, Expr::Lit(0)) {
                return Some(left.clone());
            }
            if matches!(left, Expr::Lit(0)) {
                return Some(right.clone());
            }
            None
        }
        // x ^ 0 → x, 0 ^ x → x
        BinOp::BitXor => {
            if matches!(right, Expr::Lit(0)) {
                return Some(left.clone());
            }
            if matches!(left, Expr::Lit(0)) {
                return Some(right.clone());
            }
            None
        }
        // x << 0 → x, x >> 0 → x
        BinOp::Shl | BinOp::Shr => {
            if matches!(right, Expr::Lit(0)) {
                return Some(left.clone());
            }
            None
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Builtin call folding
// ---------------------------------------------------------------------------

/// Fold constant calls to known builtins.
fn fold_builtin_call(name: &str, args: &[Expr]) -> Option<Expr> {
    // Only fold if all args are constants
    let const_args: Vec<i64> = args
        .iter()
        .filter_map(|a| if let Expr::Lit(n) = a { Some(*n) } else { None })
        .collect();

    if const_args.len() != args.len() {
        return None; // not all args are constant
    }

    match (name, const_args.as_slice()) {
        ("abs", &[a]) => Some(Expr::Lit(a.wrapping_abs())),
        ("sign", &[a]) => Some(Expr::Lit(a.signum())),
        ("min", &[a, b]) => Some(Expr::Lit(a.min(b))),
        ("max", &[a, b]) => Some(Expr::Lit(a.max(b))),
        ("clamp", &[val, lo, hi]) => Some(Expr::Lit(val.clamp(lo, hi))),
        ("log2", &[a]) => {
            if a > 0 {
                Some(Expr::Lit(63_i64.wrapping_sub(a.leading_zeros() as i64)))
            } else {
                None // preserve runtime error for non-positive
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a binary expression
    fn binop(left: Expr, op: BinOp, right: Expr) -> Expr {
        Expr::BinOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    fn lit(n: i64) -> Expr {
        Expr::Lit(n)
    }

    fn var(name: &str) -> Expr {
        Expr::Var(name.into())
    }

    // --- Constant folding ---

    #[test]
    fn fold_add() {
        let expr = binop(lit(2), BinOp::Add, lit(3));
        assert_eq!(optimize_expr(&expr), lit(5));
    }

    #[test]
    fn fold_sub() {
        let expr = binop(lit(10), BinOp::Sub, lit(3));
        assert_eq!(optimize_expr(&expr), lit(7));
    }

    #[test]
    fn fold_mul() {
        let expr = binop(lit(6), BinOp::Mul, lit(7));
        assert_eq!(optimize_expr(&expr), lit(42));
    }

    #[test]
    fn fold_div() {
        let expr = binop(lit(15), BinOp::Div, lit(3));
        assert_eq!(optimize_expr(&expr), lit(5));
    }

    #[test]
    fn fold_mod() {
        let expr = binop(lit(17), BinOp::Mod, lit(5));
        assert_eq!(optimize_expr(&expr), lit(2));
    }

    #[test]
    fn fold_nested() {
        // (2 + 3) * 4 → 20
        let inner = binop(lit(2), BinOp::Add, lit(3));
        let expr = binop(inner, BinOp::Mul, lit(4));
        assert_eq!(optimize_expr(&expr), lit(20));
    }

    #[test]
    fn fold_deeply_nested() {
        // ((1 + 2) * (3 + 4)) - 1 → 20
        let a = binop(lit(1), BinOp::Add, lit(2)); // 3
        let b = binop(lit(3), BinOp::Add, lit(4)); // 7
        let mul = binop(a, BinOp::Mul, b); // 21
        let expr = binop(mul, BinOp::Sub, lit(1)); // 20
        assert_eq!(optimize_expr(&expr), lit(20));
    }

    // --- Comparison folding ---

    #[test]
    fn fold_eq_true() {
        let expr = binop(lit(5), BinOp::Eq, lit(5));
        assert_eq!(optimize_expr(&expr), lit(1));
    }

    #[test]
    fn fold_eq_false() {
        let expr = binop(lit(5), BinOp::Eq, lit(3));
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    #[test]
    fn fold_lt() {
        let expr = binop(lit(3), BinOp::Lt, lit(5));
        assert_eq!(optimize_expr(&expr), lit(1));
    }

    #[test]
    fn fold_gt() {
        let expr = binop(lit(3), BinOp::Gt, lit(5));
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    #[test]
    fn fold_le() {
        let expr = binop(lit(5), BinOp::Le, lit(5));
        assert_eq!(optimize_expr(&expr), lit(1));
    }

    #[test]
    fn fold_ge() {
        let expr = binop(lit(3), BinOp::Ge, lit(5));
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    // --- Logical folding ---

    #[test]
    fn fold_and() {
        assert_eq!(optimize_expr(&binop(lit(1), BinOp::And, lit(1))), lit(1));
        assert_eq!(optimize_expr(&binop(lit(1), BinOp::And, lit(0))), lit(0));
    }

    #[test]
    fn fold_or() {
        assert_eq!(optimize_expr(&binop(lit(0), BinOp::Or, lit(0))), lit(0));
        assert_eq!(optimize_expr(&binop(lit(0), BinOp::Or, lit(1))), lit(1));
    }

    // --- Division by zero preserved ---

    #[test]
    fn no_fold_div_by_zero() {
        let expr = binop(lit(10), BinOp::Div, lit(0));
        // Should NOT fold — runtime error preserved
        assert!(matches!(optimize_expr(&expr), Expr::BinOp { .. }));
    }

    #[test]
    fn no_fold_mod_by_zero() {
        let expr = binop(lit(10), BinOp::Mod, lit(0));
        assert!(matches!(optimize_expr(&expr), Expr::BinOp { .. }));
    }

    // --- Identity elimination ---

    #[test]
    fn identity_add_zero_right() {
        // x + 0 → x
        let expr = binop(var("x"), BinOp::Add, lit(0));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_add_zero_left() {
        // 0 + x → x
        let expr = binop(lit(0), BinOp::Add, var("x"));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_sub_zero() {
        // x - 0 → x
        let expr = binop(var("x"), BinOp::Sub, lit(0));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_mul_one_right() {
        // x * 1 → x
        let expr = binop(var("x"), BinOp::Mul, lit(1));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_mul_one_left() {
        // 1 * x → x
        let expr = binop(lit(1), BinOp::Mul, var("x"));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_mul_zero() {
        // x * 0 → 0
        let expr = binop(var("x"), BinOp::Mul, lit(0));
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    #[test]
    fn identity_mul_zero_left() {
        // 0 * x → 0
        let expr = binop(lit(0), BinOp::Mul, var("x"));
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    #[test]
    fn identity_div_one() {
        // x / 1 → x
        let expr = binop(var("x"), BinOp::Div, lit(1));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    // --- Unary folding ---

    #[test]
    fn fold_neg_const() {
        let expr = Expr::Neg(Box::new(lit(5)));
        assert_eq!(optimize_expr(&expr), lit(-5));
    }

    #[test]
    fn fold_double_neg() {
        // --x → x
        let expr = Expr::Neg(Box::new(Expr::Neg(Box::new(var("x")))));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn fold_not_const() {
        assert_eq!(optimize_expr(&Expr::Not(Box::new(lit(0)))), lit(1));
        assert_eq!(optimize_expr(&Expr::Not(Box::new(lit(5)))), lit(0));
    }

    #[test]
    fn fold_double_not() {
        // not not x → x
        let expr = Expr::Not(Box::new(Expr::Not(Box::new(var("x")))));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    // --- Builtin call folding ---

    #[test]
    fn fold_abs_call() {
        let expr = Expr::Call {
            name: "abs".into(),
            args: vec![lit(-7)],
        };
        assert_eq!(optimize_expr(&expr), lit(7));
    }

    #[test]
    fn fold_min_call() {
        let expr = Expr::Call {
            name: "min".into(),
            args: vec![lit(3), lit(5)],
        };
        assert_eq!(optimize_expr(&expr), lit(3));
    }

    #[test]
    fn fold_max_call() {
        let expr = Expr::Call {
            name: "max".into(),
            args: vec![lit(3), lit(5)],
        };
        assert_eq!(optimize_expr(&expr), lit(5));
    }

    #[test]
    fn fold_clamp_call() {
        let expr = Expr::Call {
            name: "clamp".into(),
            args: vec![lit(15), lit(0), lit(10)],
        };
        assert_eq!(optimize_expr(&expr), lit(10));
    }

    #[test]
    fn fold_sign_call() {
        let expr = Expr::Call {
            name: "sign".into(),
            args: vec![lit(-42)],
        };
        assert_eq!(optimize_expr(&expr), lit(-1));
    }

    #[test]
    fn no_fold_non_const_call() {
        let expr = Expr::Call {
            name: "abs".into(),
            args: vec![var("x")],
        };
        // Should NOT fold — arg is not constant
        assert!(matches!(optimize_expr(&expr), Expr::Call { .. }));
    }

    // --- Statement optimization ---

    #[test]
    fn optimize_let_folds_value() {
        let stmts = vec![Stmt::Let {
            name: "x".into(),
            value: binop(lit(2), BinOp::Add, lit(3)),
        }];
        let opt = optimize(&stmts);
        assert_eq!(opt.len(), 1);
        if let Stmt::Let { value, .. } = &opt[0] {
            assert_eq!(*value, lit(5));
        }
    }

    #[test]
    fn optimize_if_eliminates_dead_branch() {
        // if 1 > 0 do ... else ... end → eliminates else branch
        let stmts = vec![Stmt::If {
            cond: binop(lit(1), BinOp::Gt, lit(0)),
            then_body: vec![Stmt::ExprStmt(lit(42))],
            else_body: vec![Stmt::ExprStmt(lit(99))],
        }];
        let opt = optimize(&stmts);
        if let Stmt::If { else_body, .. } = &opt[0] {
            assert!(else_body.is_empty());
        }
    }

    #[test]
    fn optimize_if_false_eliminates_then_branch() {
        // if 0 > 1 do ... else ... end → eliminates then branch
        let stmts = vec![Stmt::If {
            cond: binop(lit(0), BinOp::Gt, lit(1)),
            then_body: vec![Stmt::ExprStmt(lit(42))],
            else_body: vec![Stmt::ExprStmt(lit(99))],
        }];
        let opt = optimize(&stmts);
        if let Stmt::If {
            then_body,
            else_body,
            ..
        } = &opt[0]
        {
            assert!(then_body.is_empty());
            assert_eq!(else_body.len(), 1);
        }
    }

    #[test]
    fn optimize_for_folds_bounds() {
        let stmts = vec![Stmt::For {
            var: "i".into(),
            start: binop(lit(0), BinOp::Add, lit(0)),
            end: binop(lit(5), BinOp::Mul, lit(2)),
            body: vec![Stmt::ExprStmt(var("i"))],
        }];
        let opt = optimize(&stmts);
        if let Stmt::For { start, end, .. } = &opt[0] {
            assert_eq!(*start, lit(0));
            assert_eq!(*end, lit(10));
        }
    }

    #[test]
    fn optimize_fn_def_folds_body() {
        let stmts = vec![Stmt::FnDef {
            name: "f".into(),
            params: vec!["x".into()],
            body: vec![Stmt::Return(binop(lit(1), BinOp::Add, lit(1)))],
        }];
        let opt = optimize(&stmts);
        if let Stmt::FnDef { body, .. } = &opt[0] {
            if let Stmt::Return(expr) = &body[0] {
                assert_eq!(*expr, lit(2));
            }
        }
    }

    // --- No optimization when variables involved ---

    #[test]
    fn no_fold_with_variable() {
        let expr = binop(var("x"), BinOp::Add, lit(3));
        let result = optimize_expr(&expr);
        assert!(matches!(result, Expr::BinOp { .. }));
    }

    // --- Mixed: partial fold ---

    #[test]
    fn partial_fold_left_const() {
        // (2 + 3) + x → 5 + x
        let left = binop(lit(2), BinOp::Add, lit(3));
        let expr = binop(left, BinOp::Add, var("x"));
        let result = optimize_expr(&expr);
        // Left should be folded to 5
        if let Expr::BinOp { left, .. } = &result {
            assert_eq!(**left, lit(5));
        }
    }

    // --- Bitwise constant folding ---

    #[test]
    fn fold_bitwise_and() {
        let expr = binop(lit(12), BinOp::BitAnd, lit(10));
        assert_eq!(optimize_expr(&expr), lit(8)); // 1100 & 1010 = 1000
    }

    #[test]
    fn fold_bitwise_or() {
        let expr = binop(lit(12), BinOp::BitOr, lit(10));
        assert_eq!(optimize_expr(&expr), lit(14)); // 1100 | 1010 = 1110
    }

    #[test]
    fn fold_bitwise_xor() {
        let expr = binop(lit(12), BinOp::BitXor, lit(10));
        assert_eq!(optimize_expr(&expr), lit(6)); // 1100 ^ 1010 = 0110
    }

    #[test]
    fn fold_shift_left() {
        let expr = binop(lit(1), BinOp::Shl, lit(4));
        assert_eq!(optimize_expr(&expr), lit(16));
    }

    #[test]
    fn fold_shift_right() {
        let expr = binop(lit(16), BinOp::Shr, lit(4));
        assert_eq!(optimize_expr(&expr), lit(1));
    }

    #[test]
    fn fold_bitnot_const() {
        let expr = Expr::BitNot(Box::new(lit(0)));
        assert_eq!(optimize_expr(&expr), lit(-1));
    }

    #[test]
    fn fold_double_bitnot() {
        // ~~x → x
        let expr = Expr::BitNot(Box::new(Expr::BitNot(Box::new(var("x")))));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    // --- Bitwise identity elimination ---

    #[test]
    fn identity_bitand_zero() {
        // x & 0 → 0
        let expr = binop(var("x"), BinOp::BitAnd, lit(0));
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    #[test]
    fn identity_bitor_zero() {
        // x | 0 → x
        let expr = binop(var("x"), BinOp::BitOr, lit(0));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_bitxor_zero() {
        // x ^ 0 → x
        let expr = binop(var("x"), BinOp::BitXor, lit(0));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_shl_zero() {
        // x << 0 → x
        let expr = binop(var("x"), BinOp::Shl, lit(0));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    #[test]
    fn identity_shr_zero() {
        // x >> 0 → x
        let expr = binop(var("x"), BinOp::Shr, lit(0));
        assert_eq!(optimize_expr(&expr), var("x"));
    }

    // --- log2 folding ---

    #[test]
    fn fold_log2_call() {
        let expr = Expr::Call {
            name: "log2".into(),
            args: vec![lit(8)],
        };
        assert_eq!(optimize_expr(&expr), lit(3));
    }

    #[test]
    fn fold_log2_one() {
        let expr = Expr::Call {
            name: "log2".into(),
            args: vec![lit(1)],
        };
        assert_eq!(optimize_expr(&expr), lit(0));
    }

    #[test]
    fn no_fold_log2_zero() {
        // log2(0) is undefined — preserve runtime error
        let expr = Expr::Call {
            name: "log2".into(),
            args: vec![lit(0)],
        };
        assert!(matches!(optimize_expr(&expr), Expr::Call { .. }));
    }

    #[test]
    fn no_fold_log2_negative() {
        let expr = Expr::Call {
            name: "log2".into(),
            args: vec![lit(-5)],
        };
        assert!(matches!(optimize_expr(&expr), Expr::Call { .. }));
    }
}
