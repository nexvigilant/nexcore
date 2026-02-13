// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Full Type Inference
//!
//! Hindley-Milner style type inference for Prima.
//!
//! ## Philosophy
//!
//! "Code that compiles is mathematically true."
//!
//! Type inference eliminates annotation burden while preserving
//! complete type safety. Every expression has exactly one type.
//!
//! ## Tier: T2-C (μ + → + Σ + ∃)
//!
//! ## Algorithm
//!
//! 1. Generate fresh type variables for unknowns
//! 2. Collect constraints from expression structure
//! 3. Unify constraints to find substitutions
//! 4. Apply substitutions to resolve all types
//!
//! ## Grounding
//!
//! - Type variables: ∃ (Existence) — something exists but unknown
//! - Substitution: μ (Mapping) — type var → concrete type
//! - Unification: κ (Comparison) — structural equality check
//! - Constraints: → (Causality) — expression implies type

use crate::ast::{BinOp, Block, Expr, Literal, Stmt, UnOp};
use crate::error::{PrimaError, PrimaResult};
use crate::token::Span;
use crate::types::PrimaType;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// TYPE VARIABLE — ∃ (Existence: unknown type)
// ═══════════════════════════════════════════════════════════════════════════

/// A type variable (fresh unknown).
///
/// ## Tier: T1 (∃)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeVar(pub u32);

impl TypeVar {
    /// Create a new type variable.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}", self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INFERRED TYPE — Σ (Sum: concrete | variable)
// ═══════════════════════════════════════════════════════════════════════════

/// A type that may be concrete or a variable.
///
/// ## Tier: T2-P (Σ + ∃)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InferType {
    /// Concrete type (known).
    Concrete(PrimaType),
    /// Type variable (unknown).
    Var(TypeVar),
    /// Function type: params → return.
    Function {
        params: Vec<InferType>,
        ret: Box<InferType>,
    },
    /// Sequence type: σ[elem].
    Sequence(Box<InferType>),
    /// Optional type: elem | ∅.
    Optional(Box<InferType>),
}

impl InferType {
    /// Create void type.
    #[must_use]
    pub fn void() -> Self {
        Self::Concrete(PrimaType::new(
            "∅",
            PrimitiveComposition::new(vec![LexPrimitiva::Void]),
        ))
    }

    /// Create integer type.
    #[must_use]
    pub fn int() -> Self {
        Self::Concrete(PrimaType::new(
            "N",
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        ))
    }

    /// Create float type.
    #[must_use]
    pub fn float() -> Self {
        Self::Concrete(PrimaType::new(
            "Float",
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        ))
    }

    /// Create boolean type.
    #[must_use]
    pub fn bool() -> Self {
        Self::Concrete(PrimaType::new(
            "Bool",
            PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
        ))
    }

    /// Create string type.
    #[must_use]
    pub fn string() -> Self {
        Self::Concrete(PrimaType::new(
            "String",
            PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]),
        ))
    }

    /// Check if this is a type variable.
    #[must_use]
    pub const fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    /// Check if this is concrete (fully resolved).
    #[must_use]
    pub fn is_concrete(&self) -> bool {
        match self {
            Self::Concrete(_) => true,
            Self::Var(_) => false,
            Self::Function { params, ret } => {
                params.iter().all(Self::is_concrete) && ret.is_concrete()
            }
            Self::Sequence(elem) | Self::Optional(elem) => elem.is_concrete(),
        }
    }

    /// Apply a substitution.
    #[must_use]
    pub fn apply(&self, subst: &Substitution) -> Self {
        match self {
            Self::Concrete(t) => Self::Concrete(t.clone()),
            Self::Var(v) => subst.get(*v).cloned().unwrap_or(Self::Var(*v)),
            Self::Function { params, ret } => Self::Function {
                params: params.iter().map(|p| p.apply(subst)).collect(),
                ret: Box::new(ret.apply(subst)),
            },
            Self::Sequence(elem) => Self::Sequence(Box::new(elem.apply(subst))),
            Self::Optional(elem) => Self::Optional(Box::new(elem.apply(subst))),
        }
    }

    /// Get free type variables.
    #[must_use]
    pub fn free_vars(&self) -> Vec<TypeVar> {
        match self {
            Self::Concrete(_) => vec![],
            Self::Var(v) => vec![*v],
            Self::Function { params, ret } => {
                let mut vars: Vec<TypeVar> = params.iter().flat_map(Self::free_vars).collect();
                vars.extend(ret.free_vars());
                vars
            }
            Self::Sequence(elem) | Self::Optional(elem) => elem.free_vars(),
        }
    }
}

impl std::fmt::Display for InferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Concrete(t) => write!(f, "{}", t.name),
            Self::Var(v) => write!(f, "{v}"),
            Self::Function { params, ret } => {
                let params_str: Vec<String> = params.iter().map(|p| format!("{p}")).collect();
                write!(f, "({}) → {}", params_str.join(", "), ret)
            }
            Self::Sequence(elem) => write!(f, "σ[{elem}]"),
            Self::Optional(elem) => write!(f, "{elem}?"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SUBSTITUTION — μ (Mapping: TypeVar → InferType)
// ═══════════════════════════════════════════════════════════════════════════

/// A substitution mapping type variables to types.
///
/// ## Tier: T2-P (μ)
#[derive(Debug, Clone, Default)]
pub struct Substitution {
    map: HashMap<TypeVar, InferType>,
}

impl Substitution {
    /// Create empty substitution.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mapping.
    pub fn insert(&mut self, var: TypeVar, ty: InferType) {
        self.map.insert(var, ty);
    }

    /// Get a mapping.
    #[must_use]
    pub fn get(&self, var: TypeVar) -> Option<&InferType> {
        self.map.get(&var)
    }

    /// Compose two substitutions: self ∘ other.
    #[must_use]
    pub fn compose(&self, other: &Self) -> Self {
        let mut result = Self::new();

        // Apply self to other's mappings
        for (var, ty) in &other.map {
            result.insert(*var, ty.apply(self));
        }

        // Add self's mappings (not overwriting)
        for (var, ty) in &self.map {
            result.map.entry(*var).or_insert_with(|| ty.clone());
        }

        result
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONSTRAINT — → (Causality: type relations)
// ═══════════════════════════════════════════════════════════════════════════

/// A type constraint.
///
/// ## Tier: T2-P (→ + κ)
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Two types must be equal.
    Equal(InferType, InferType, Span),
}

impl Constraint {
    /// Create an equality constraint.
    #[must_use]
    pub fn equal(t1: InferType, t2: InferType, span: Span) -> Self {
        Self::Equal(t1, t2, span)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPE INFERENCER — Central inference engine
// ═══════════════════════════════════════════════════════════════════════════

/// Type inference engine.
///
/// ## Tier: T2-C (μ + → + Σ + κ)
#[derive(Debug, Default)]
pub struct TypeInferencer {
    /// Next type variable ID.
    next_var: u32,
    /// Collected constraints.
    constraints: Vec<Constraint>,
    /// Variable bindings: name → type.
    env: Vec<HashMap<String, InferType>>,
    /// Computed substitution.
    subst: Substitution,
}

impl TypeInferencer {
    /// Create a new inferencer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            env: vec![HashMap::new()],
            ..Default::default()
        }
    }

    /// Generate a fresh type variable.
    pub fn fresh(&mut self) -> InferType {
        let var = TypeVar::new(self.next_var);
        self.next_var += 1;
        InferType::Var(var)
    }

    /// Push a scope.
    pub fn push_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    /// Pop a scope.
    pub fn pop_scope(&mut self) {
        self.env.pop();
    }

    /// Bind a variable.
    pub fn bind(&mut self, name: String, ty: InferType) {
        if let Some(scope) = self.env.last_mut() {
            scope.insert(name, ty);
        }
    }

    /// Lookup a variable.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<InferType> {
        for scope in self.env.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    /// Add a constraint.
    pub fn constrain(&mut self, t1: InferType, t2: InferType, span: Span) {
        self.constraints.push(Constraint::equal(t1, t2, span));
    }

    /// Infer type of a literal.
    #[must_use]
    pub fn infer_literal(&self, lit: &Literal) -> InferType {
        match lit {
            Literal::Int(_) => InferType::int(),
            Literal::Float(_) => InferType::float(),
            Literal::Bool(_) => InferType::bool(),
            Literal::String(_) => InferType::string(),
            Literal::Void => InferType::void(),
            Literal::Symbol(_) => InferType::Concrete(PrimaType::new(
                "Symbol",
                PrimitiveComposition::new(vec![LexPrimitiva::Location]),
            )),
        }
    }

    /// Infer type of an expression.
    pub fn infer_expr(&mut self, expr: &Expr) -> PrimaResult<InferType> {
        match expr {
            Expr::Literal { value, .. } => Ok(self.infer_literal(value)),

            Expr::Ident { name, span: _ } => self
                .lookup(name)
                .ok_or_else(|| PrimaError::undefined(name.clone())),

            Expr::Binary {
                left,
                op,
                right,
                span,
                ..
            } => {
                let left_ty = self.infer_expr(left)?;
                let right_ty = self.infer_expr(right)?;

                // Constrain operands to be the same type for most ops
                self.constrain(left_ty.clone(), right_ty.clone(), *span);

                // Result type depends on operator
                Ok(match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => left_ty,
                    BinOp::Eq
                    | BinOp::Ne
                    | BinOp::Lt
                    | BinOp::Le
                    | BinOp::Gt
                    | BinOp::Ge
                    | BinOp::KappaEq
                    | BinOp::KappaNe
                    | BinOp::KappaLt
                    | BinOp::KappaGt
                    | BinOp::KappaLe
                    | BinOp::KappaGe => InferType::bool(),
                    BinOp::And | BinOp::Or => InferType::bool(),
                })
            }

            Expr::Unary { op, operand, span } => {
                let operand_ty = self.infer_expr(operand)?;
                match op {
                    UnOp::Neg => {
                        self.constrain(operand_ty.clone(), InferType::int(), *span);
                        Ok(InferType::int())
                    }
                    UnOp::Not => {
                        self.constrain(operand_ty, InferType::bool(), *span);
                        Ok(InferType::bool())
                    }
                }
            }

            Expr::Call { func, args, span } => {
                // Look up function type
                let func_ty = self
                    .lookup(func)
                    .ok_or_else(|| PrimaError::undefined(func.clone()))?;

                // Infer argument types
                let arg_tys: Vec<InferType> = args
                    .iter()
                    .map(|a| self.infer_expr(a))
                    .collect::<PrimaResult<_>>()?;

                // Fresh return type
                let ret_ty = self.fresh();

                // Constrain function type
                let expected_fn = InferType::Function {
                    params: arg_tys,
                    ret: Box::new(ret_ty.clone()),
                };
                self.constrain(func_ty, expected_fn, *span);

                Ok(ret_ty)
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                let cond_ty = self.infer_expr(cond)?;
                self.constrain(cond_ty, InferType::bool(), *span);

                let then_ty = self.infer_block(then_branch)?;

                if let Some(else_b) = else_branch {
                    let else_ty = self.infer_block(else_b)?;
                    self.constrain(then_ty.clone(), else_ty, *span);
                }

                Ok(then_ty)
            }

            Expr::Lambda {
                params,
                body,
                span: _,
            } => {
                self.push_scope();

                // Fresh types for parameters
                let param_tys: Vec<InferType> = params
                    .iter()
                    .map(|p| {
                        let ty = self.fresh();
                        self.bind(p.name.clone(), ty.clone());
                        ty
                    })
                    .collect();

                let body_ty = self.infer_expr(body)?;

                self.pop_scope();

                Ok(InferType::Function {
                    params: param_tys,
                    ret: Box::new(body_ty),
                })
            }

            Expr::Sequence { elements, .. } => {
                if elements.is_empty() {
                    let elem_ty = self.fresh();
                    Ok(InferType::Sequence(Box::new(elem_ty)))
                } else {
                    let first_ty = self.infer_expr(&elements[0])?;
                    for elem in elements.iter().skip(1) {
                        let elem_ty = self.infer_expr(elem)?;
                        self.constrain(first_ty.clone(), elem_ty, Span::default());
                    }
                    Ok(InferType::Sequence(Box::new(first_ty)))
                }
            }

            Expr::Block { block, .. } => self.infer_block(block),

            // Default for other expressions
            _ => Ok(self.fresh()),
        }
    }

    /// Infer type of a block.
    pub fn infer_block(&mut self, block: &Block) -> PrimaResult<InferType> {
        self.push_scope();
        let mut result = InferType::void();

        for stmt in &block.statements {
            result = self.infer_stmt(stmt)?;
        }

        self.pop_scope();
        Ok(result)
    }

    /// Infer type of a statement.
    pub fn infer_stmt(&mut self, stmt: &Stmt) -> PrimaResult<InferType> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let ty = self.infer_expr(value)?;
                self.bind(name.clone(), ty);
                Ok(InferType::void())
            }
            Stmt::FnDef {
                name, params, body, ..
            } => {
                self.push_scope();

                // Bind parameters with fresh types
                let param_tys: Vec<InferType> = params
                    .iter()
                    .map(|p| {
                        let ty = self.fresh();
                        self.bind(p.name.clone(), ty.clone());
                        ty
                    })
                    .collect();

                let ret_ty = self.infer_block(body)?;

                self.pop_scope();

                let fn_ty = InferType::Function {
                    params: param_tys,
                    ret: Box::new(ret_ty),
                };
                self.bind(name.clone(), fn_ty.clone());

                Ok(InferType::void())
            }
            Stmt::Expr { expr, .. } => self.infer_expr(expr),
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.infer_expr(v)
                } else {
                    Ok(InferType::void())
                }
            }
            Stmt::TypeDef { .. } => Ok(InferType::void()),
        }
    }

    /// Solve constraints via unification.
    pub fn solve(&mut self) -> PrimaResult<()> {
        let constraints = std::mem::take(&mut self.constraints);

        for constraint in constraints {
            match constraint {
                Constraint::Equal(t1, t2, span) => {
                    let t1 = t1.apply(&self.subst);
                    let t2 = t2.apply(&self.subst);
                    let new_subst = unify(&t1, &t2, span)?;
                    self.subst = self.subst.compose(&new_subst);
                }
            }
        }

        Ok(())
    }

    /// Get the computed substitution.
    #[must_use]
    pub fn substitution(&self) -> &Substitution {
        &self.subst
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIFICATION — κ (Comparison for type equality)
// ═══════════════════════════════════════════════════════════════════════════

/// Unify two types, producing a substitution.
pub fn unify(t1: &InferType, t2: &InferType, span: Span) -> PrimaResult<Substitution> {
    match (t1, t2) {
        // Same concrete types
        (InferType::Concrete(a), InferType::Concrete(b)) if a.name == b.name => {
            Ok(Substitution::new())
        }

        // Variable = anything
        (InferType::Var(v), t) | (t, InferType::Var(v)) => {
            if let InferType::Var(v2) = t {
                if v == v2 {
                    return Ok(Substitution::new());
                }
            }
            // Occurs check
            if t.free_vars().contains(v) {
                return Err(PrimaError::type_error(
                    span,
                    format!("infinite type: {v} = {t}"),
                ));
            }
            let mut subst = Substitution::new();
            subst.insert(*v, t.clone());
            Ok(subst)
        }

        // Function types
        (
            InferType::Function {
                params: p1,
                ret: r1,
            },
            InferType::Function {
                params: p2,
                ret: r2,
            },
        ) => {
            if p1.len() != p2.len() {
                return Err(PrimaError::type_error(
                    span,
                    format!("arity mismatch: {} vs {}", p1.len(), p2.len()),
                ));
            }
            let mut subst = Substitution::new();
            for (a, b) in p1.iter().zip(p2.iter()) {
                let s = unify(&a.apply(&subst), &b.apply(&subst), span)?;
                subst = subst.compose(&s);
            }
            let s = unify(&r1.apply(&subst), &r2.apply(&subst), span)?;
            Ok(subst.compose(&s))
        }

        // Sequence types
        (InferType::Sequence(e1), InferType::Sequence(e2)) => unify(e1, e2, span),

        // Optional types
        (InferType::Optional(e1), InferType::Optional(e2)) => unify(e1, e2, span),

        // Type mismatch
        _ => Err(PrimaError::type_error(
            span,
            format!("type mismatch: {t1} vs {t2}"),
        )),
    }
}

/// Get primitive composition for type inference.
#[must_use]
pub fn infer_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![
        LexPrimitiva::Mapping,
        LexPrimitiva::Causality,
        LexPrimitiva::Sum,
        LexPrimitiva::Existence,
    ])
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_var_display() {
        assert_eq!(format!("{}", TypeVar::new(0)), "?0");
        assert_eq!(format!("{}", TypeVar::new(42)), "?42");
    }

    #[test]
    fn test_infer_type_concrete() {
        assert!(InferType::int().is_concrete());
        assert!(InferType::bool().is_concrete());
        assert!(!InferType::Var(TypeVar::new(0)).is_concrete());
    }

    #[test]
    fn test_infer_type_display() {
        assert_eq!(format!("{}", InferType::int()), "N");
        assert_eq!(format!("{}", InferType::bool()), "Bool");
        assert_eq!(format!("{}", InferType::Var(TypeVar::new(0))), "?0");
    }

    #[test]
    fn test_function_type_display() {
        let fn_ty = InferType::Function {
            params: vec![InferType::int(), InferType::int()],
            ret: Box::new(InferType::int()),
        };
        assert_eq!(format!("{fn_ty}"), "(N, N) → N");
    }

    #[test]
    fn test_substitution() {
        let mut subst = Substitution::new();
        let v0 = TypeVar::new(0);
        subst.insert(v0, InferType::int());

        let ty = InferType::Var(v0);
        let applied = ty.apply(&subst);
        assert_eq!(applied, InferType::int());
    }

    #[test]
    fn test_substitution_compose() {
        let mut s1 = Substitution::new();
        s1.insert(TypeVar::new(0), InferType::Var(TypeVar::new(1)));

        let mut s2 = Substitution::new();
        s2.insert(TypeVar::new(1), InferType::int());

        let composed = s2.compose(&s1);
        let ty = InferType::Var(TypeVar::new(0));
        let applied = ty.apply(&composed);
        assert_eq!(applied, InferType::int());
    }

    #[test]
    fn test_unify_same_concrete() {
        let result = unify(&InferType::int(), &InferType::int(), Span::default());
        assert!(result.is_ok());
        assert!(result.unwrap_or_default().is_empty());
    }

    #[test]
    fn test_unify_var_concrete() {
        let v = InferType::Var(TypeVar::new(0));
        let result = unify(&v, &InferType::int(), Span::default());
        assert!(result.is_ok());
        let subst = result.unwrap_or_default();
        assert_eq!(v.apply(&subst), InferType::int());
    }

    #[test]
    fn test_unify_mismatch() {
        let result = unify(&InferType::int(), &InferType::bool(), Span::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_function() {
        let f1 = InferType::Function {
            params: vec![InferType::Var(TypeVar::new(0))],
            ret: Box::new(InferType::int()),
        };
        let f2 = InferType::Function {
            params: vec![InferType::int()],
            ret: Box::new(InferType::Var(TypeVar::new(1))),
        };

        let result = unify(&f1, &f2, Span::default());
        assert!(result.is_ok());
        let subst = result.unwrap_or_default();

        assert_eq!(
            InferType::Var(TypeVar::new(0)).apply(&subst),
            InferType::int()
        );
        assert_eq!(
            InferType::Var(TypeVar::new(1)).apply(&subst),
            InferType::int()
        );
    }

    #[test]
    fn test_inferencer_literal() {
        let inferencer = TypeInferencer::new();
        let ty = inferencer.infer_literal(&Literal::Int(42));
        assert_eq!(ty, InferType::int());
    }

    #[test]
    fn test_inferencer_fresh() {
        let mut inferencer = TypeInferencer::new();
        let v1 = inferencer.fresh();
        let v2 = inferencer.fresh();
        assert!(v1.is_var());
        assert!(v2.is_var());
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_inferencer_scope() {
        let mut inferencer = TypeInferencer::new();
        inferencer.bind("x".into(), InferType::int());
        assert!(inferencer.lookup("x").is_some());

        inferencer.push_scope();
        inferencer.bind("y".into(), InferType::bool());
        assert!(inferencer.lookup("x").is_some());
        assert!(inferencer.lookup("y").is_some());

        inferencer.pop_scope();
        assert!(inferencer.lookup("x").is_some());
        assert!(inferencer.lookup("y").is_none());
    }

    #[test]
    fn test_free_vars() {
        let ty = InferType::Function {
            params: vec![InferType::Var(TypeVar::new(0))],
            ret: Box::new(InferType::Var(TypeVar::new(1))),
        };
        let vars = ty.free_vars();
        assert!(vars.contains(&TypeVar::new(0)));
        assert!(vars.contains(&TypeVar::new(1)));
    }

    #[test]
    fn test_occurs_check() {
        // t = t -> t would be infinite
        let v = TypeVar::new(0);
        let infinite = InferType::Function {
            params: vec![InferType::Var(v)],
            ret: Box::new(InferType::Var(v)),
        };
        let result = unify(&InferType::Var(v), &infinite, Span::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_infer_composition() {
        let comp = infer_composition();
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Mapping));
        assert!(unique.contains(&LexPrimitiva::Causality));
    }
}
