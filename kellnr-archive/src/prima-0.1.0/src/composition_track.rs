// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Composition Tracking
//!
//! Tracks primitive flow through expressions and statements.
//!
//! ## Philosophy
//!
//! Every value in Prima carries its primitive composition.
//! Composition tracking lets us:
//! - Compute transfer confidence between domains
//! - Verify grounding to {0, 1}
//! - Classify tiers automatically
//!
//! ## Tier: T2-C (σ + μ + → + ς)
//!
//! ## Tracking Rules
//!
//! 1. Literals: Composition from their type
//! 2. Operations: Union of operand compositions + operator primitive
//! 3. Functions: Union of body compositions
//! 4. Control flow: Union of branch compositions

use crate::ast::{BinOp, Block, Expr, Literal, Program, Stmt, UnOp};
use crate::token::Span;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition, Tier};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════
// COMPOSITION ENTRY — ς (State at a point)
// ═══════════════════════════════════════════════════════════════════════════

/// A tracked composition with metadata.
///
/// ## Tier: T2-P (ς + σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionEntry {
    /// The composition.
    pub composition: PrimitiveComposition,
    /// Where this was computed.
    pub span: Span,
    /// How this composition was derived.
    pub derivation: Derivation,
}

/// How a composition was derived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Derivation {
    /// From a literal value.
    Literal(String),
    /// From a variable reference.
    Variable(String),
    /// From an operation.
    Operation(String),
    /// From function application.
    Application(String),
    /// From composition union.
    Union(Vec<Box<Derivation>>),
    /// From control flow merge.
    Merge(Vec<Box<Derivation>>),
}

impl CompositionEntry {
    /// Create from a literal.
    #[must_use]
    pub fn from_literal(composition: PrimitiveComposition, desc: &str, span: Span) -> Self {
        Self {
            composition,
            span,
            derivation: Derivation::Literal(desc.to_string()),
        }
    }

    /// Create from a variable.
    #[must_use]
    pub fn from_variable(composition: PrimitiveComposition, name: &str, span: Span) -> Self {
        Self {
            composition,
            span,
            derivation: Derivation::Variable(name.to_string()),
        }
    }

    /// Create from an operation.
    #[must_use]
    pub fn from_operation(composition: PrimitiveComposition, op: &str, span: Span) -> Self {
        Self {
            composition,
            span,
            derivation: Derivation::Operation(op.to_string()),
        }
    }

    /// Get the tier of this composition.
    #[must_use]
    pub fn tier(&self) -> Tier {
        Tier::classify(&self.composition)
    }

    /// Get transfer confidence.
    #[must_use]
    pub fn transfer_confidence(&self) -> f64 {
        self.tier().transfer_multiplier()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPOSITION CONTEXT — μ[name → composition]
// ═══════════════════════════════════════════════════════════════════════════

/// Context for tracking compositions through evaluation.
///
/// ## Tier: T2-C (μ + σ + ς)
#[derive(Debug, Clone, Default)]
pub struct CompositionContext {
    /// Variable bindings.
    bindings: Vec<HashMap<String, CompositionEntry>>,
    /// Function compositions.
    functions: HashMap<String, CompositionEntry>,
    /// All tracked compositions (for analysis).
    history: Vec<CompositionEntry>,
}

impl CompositionContext {
    /// Create a new context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: vec![HashMap::new()],
            functions: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Push a scope.
    pub fn push_scope(&mut self) {
        self.bindings.push(HashMap::new());
    }

    /// Pop a scope.
    pub fn pop_scope(&mut self) {
        self.bindings.pop();
    }

    /// Bind a variable.
    pub fn bind(&mut self, name: String, entry: CompositionEntry) {
        self.history.push(entry.clone());
        if let Some(scope) = self.bindings.last_mut() {
            scope.insert(name, entry);
        }
    }

    /// Lookup a variable.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&CompositionEntry> {
        for scope in self.bindings.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }

    /// Register a function.
    pub fn register_function(&mut self, name: String, entry: CompositionEntry) {
        self.history.push(entry.clone());
        self.functions.insert(name, entry);
    }

    /// Lookup a function.
    #[must_use]
    pub fn lookup_function(&self, name: &str) -> Option<&CompositionEntry> {
        self.functions.get(name)
    }

    /// Get all tracked compositions.
    #[must_use]
    pub fn history(&self) -> &[CompositionEntry] {
        &self.history
    }

    /// Get composition statistics.
    #[must_use]
    pub fn stats(&self) -> CompositionStats {
        let mut unique_primitives = HashSet::new();
        let mut tier_counts = HashMap::new();

        for entry in &self.history {
            for prim in &entry.composition.primitives {
                unique_primitives.insert(*prim);
            }
            *tier_counts.entry(entry.tier()).or_insert(0) += 1;
        }

        CompositionStats {
            total_entries: self.history.len(),
            unique_primitives: unique_primitives.len(),
            tier_distribution: tier_counts,
        }
    }
}

/// Statistics about tracked compositions.
#[derive(Debug, Clone)]
pub struct CompositionStats {
    /// Total composition entries.
    pub total_entries: usize,
    /// Unique primitives used.
    pub unique_primitives: usize,
    /// Distribution by tier.
    pub tier_distribution: HashMap<Tier, usize>,
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPOSITION TRACKER — → (Causality: AST → Compositions)
// ═══════════════════════════════════════════════════════════════════════════

/// Tracks compositions through AST traversal.
///
/// ## Tier: T2-C (→ + σ + μ)
#[derive(Debug, Default)]
pub struct CompositionTracker {
    /// The tracking context.
    context: CompositionContext,
}

impl CompositionTracker {
    /// Create a new tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a program.
    pub fn track_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.track_stmt(stmt);
        }
    }

    /// Track a statement.
    pub fn track_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, span } => {
                let entry = self.track_expr(value);
                let bound_entry = CompositionEntry {
                    composition: entry.composition.clone(),
                    span: *span,
                    derivation: Derivation::Variable(name.clone()),
                };
                self.context.bind(name.clone(), bound_entry);
            }
            Stmt::FnDef {
                name, body, span, ..
            } => {
                self.context.push_scope();
                let body_entry = self.track_block(body);
                self.context.pop_scope();

                let fn_entry = CompositionEntry {
                    composition: merge_with(&body_entry.composition, LexPrimitiva::Causality),
                    span: *span,
                    derivation: Derivation::Application(name.clone()),
                };
                self.context.register_function(name.clone(), fn_entry);
            }
            Stmt::Expr { expr, .. } => {
                self.track_expr(expr);
            }
            Stmt::Return { value, span } => {
                if let Some(v) = value {
                    self.track_expr(v);
                } else {
                    let entry = CompositionEntry::from_literal(
                        PrimitiveComposition::new(vec![LexPrimitiva::Void]),
                        "void return",
                        *span,
                    );
                    self.context.history.push(entry);
                }
            }
            Stmt::TypeDef { .. } => {}
        }
    }

    /// Track a block.
    pub fn track_block(&mut self, block: &Block) -> CompositionEntry {
        self.context.push_scope();

        let mut compositions = Vec::new();
        for stmt in &block.statements {
            self.track_stmt(stmt);
        }

        // Merge all statement compositions
        for entry in self
            .context
            .history()
            .iter()
            .rev()
            .take(block.statements.len())
        {
            compositions.push(entry.composition.clone());
        }

        self.context.pop_scope();

        let merged = merge_all(&compositions);
        CompositionEntry {
            composition: merged,
            span: Span::default(),
            derivation: Derivation::Merge(vec![]),
        }
    }

    /// Track an expression.
    pub fn track_expr(&mut self, expr: &Expr) -> CompositionEntry {
        let entry = match expr {
            Expr::Literal { value, span } => self.track_literal(value, *span),

            Expr::Ident { name, span } => {
                if let Some(entry) = self.context.lookup(name) {
                    entry.clone()
                } else {
                    CompositionEntry::from_variable(
                        PrimitiveComposition::new(vec![LexPrimitiva::Location]),
                        name,
                        *span,
                    )
                }
            }

            Expr::Binary {
                left,
                op,
                right,
                span,
                ..
            } => {
                let left_entry = self.track_expr(left);
                let right_entry = self.track_expr(right);
                let op_prim = binop_primitive(*op);

                let merged = merge_all(&[
                    left_entry.composition,
                    right_entry.composition,
                    PrimitiveComposition::new(vec![op_prim]),
                ]);

                CompositionEntry::from_operation(merged, &format!("{op:?}"), *span)
            }

            Expr::Unary { op, operand, span } => {
                let operand_entry = self.track_expr(operand);
                let op_prim = unop_primitive(*op);

                let merged = merge_with(&operand_entry.composition, op_prim);
                CompositionEntry::from_operation(merged, &format!("{op:?}"), *span)
            }

            Expr::Call { func, args, span } => {
                let mut compositions = Vec::new();

                // Track argument compositions
                for arg in args {
                    let arg_entry = self.track_expr(arg);
                    compositions.push(arg_entry.composition);
                }

                // Add function composition if known
                if let Some(fn_entry) = self.context.lookup_function(func) {
                    compositions.push(fn_entry.composition.clone());
                }

                // Add causality for application
                compositions.push(PrimitiveComposition::new(vec![LexPrimitiva::Causality]));

                let merged = merge_all(&compositions);
                CompositionEntry {
                    composition: merged,
                    span: *span,
                    derivation: Derivation::Application(func.clone()),
                }
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                let cond_entry = self.track_expr(cond);
                let then_entry = self.track_block(then_branch);

                let mut compositions = vec![
                    cond_entry.composition,
                    then_entry.composition,
                    PrimitiveComposition::new(vec![LexPrimitiva::Boundary]),
                ];

                if let Some(else_b) = else_branch {
                    let else_entry = self.track_block(else_b);
                    compositions.push(else_entry.composition);
                }

                let merged = merge_all(&compositions);
                CompositionEntry {
                    composition: merged,
                    span: *span,
                    derivation: Derivation::Merge(vec![]),
                }
            }

            Expr::Lambda { body, span, .. } => {
                let body_entry = self.track_expr(body);
                let merged = merge_with(&body_entry.composition, LexPrimitiva::Causality);
                CompositionEntry::from_operation(merged, "lambda", *span)
            }

            Expr::Sequence { elements, span } => {
                let mut compositions =
                    vec![PrimitiveComposition::new(vec![LexPrimitiva::Sequence])];

                for elem in elements {
                    let elem_entry = self.track_expr(elem);
                    compositions.push(elem_entry.composition);
                }

                let merged = merge_all(&compositions);
                CompositionEntry::from_operation(merged, "sequence", *span)
            }

            Expr::Block { block, .. } => self.track_block(block),

            // Default for other expressions
            _ => CompositionEntry::from_literal(
                PrimitiveComposition::new(vec![LexPrimitiva::Existence]),
                "unknown",
                Span::default(),
            ),
        };

        self.context.history.push(entry.clone());
        entry
    }

    /// Track a literal.
    fn track_literal(&self, lit: &Literal, span: Span) -> CompositionEntry {
        let (composition, desc) = match lit {
            Literal::Int(_) => (
                PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
                "int",
            ),
            Literal::Float(_) => (
                PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
                "float",
            ),
            Literal::Bool(_) => (PrimitiveComposition::new(vec![LexPrimitiva::Sum]), "bool"),
            Literal::String(_) => (
                PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]),
                "string",
            ),
            Literal::Void => (PrimitiveComposition::new(vec![LexPrimitiva::Void]), "void"),
            Literal::Symbol(_) => (
                PrimitiveComposition::new(vec![LexPrimitiva::Location]),
                "symbol",
            ),
        };

        CompositionEntry::from_literal(composition, desc, span)
    }

    /// Get the context.
    #[must_use]
    pub fn context(&self) -> &CompositionContext {
        &self.context
    }

    /// Get statistics.
    #[must_use]
    pub fn stats(&self) -> CompositionStats {
        self.context.stats()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Get primitive for binary operator.
fn binop_primitive(op: BinOp) -> LexPrimitiva {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => LexPrimitiva::Quantity,
        BinOp::Eq
        | BinOp::Ne
        | BinOp::Lt
        | BinOp::Gt
        | BinOp::Le
        | BinOp::Ge
        | BinOp::KappaEq
        | BinOp::KappaNe
        | BinOp::KappaLt
        | BinOp::KappaGt
        | BinOp::KappaLe
        | BinOp::KappaGe => LexPrimitiva::Comparison,
        BinOp::And | BinOp::Or => LexPrimitiva::Sum,
    }
}

/// Get primitive for unary operator.
fn unop_primitive(op: UnOp) -> LexPrimitiva {
    match op {
        UnOp::Neg => LexPrimitiva::Quantity,
        UnOp::Not => LexPrimitiva::Sum,
    }
}

/// Merge a composition with an additional primitive.
fn merge_with(comp: &PrimitiveComposition, prim: LexPrimitiva) -> PrimitiveComposition {
    let mut primitives = comp.primitives.clone();
    if !primitives.contains(&prim) {
        primitives.push(prim);
    }
    PrimitiveComposition::new(primitives)
}

/// Merge multiple compositions.
fn merge_all(compositions: &[PrimitiveComposition]) -> PrimitiveComposition {
    let mut all_prims: HashSet<LexPrimitiva> = HashSet::new();
    for comp in compositions {
        for prim in &comp.primitives {
            all_prims.insert(*prim);
        }
    }
    let primitives: Vec<_> = all_prims.into_iter().collect();
    PrimitiveComposition::new(primitives)
}

/// Get primitive composition for composition tracking.
#[must_use]
pub fn tracking_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![
        LexPrimitiva::Sequence,
        LexPrimitiva::Mapping,
        LexPrimitiva::Causality,
        LexPrimitiva::State,
    ])
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_entry_tier() {
        let entry = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
            "int",
            Span::default(),
        );
        assert_eq!(entry.tier(), Tier::T1Universal);
    }

    #[test]
    fn test_composition_entry_transfer() {
        let t1_entry = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
            "int",
            Span::default(),
        );
        assert_eq!(t1_entry.transfer_confidence(), 1.0);

        let t2p_entry = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]),
            "string",
            Span::default(),
        );
        assert_eq!(t2p_entry.transfer_confidence(), 0.9);
    }

    #[test]
    fn test_context_scope() {
        let mut ctx = CompositionContext::new();

        let entry = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
            "x",
            Span::default(),
        );
        ctx.bind("x".into(), entry);
        assert!(ctx.lookup("x").is_some());

        ctx.push_scope();
        let entry2 = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
            "y",
            Span::default(),
        );
        ctx.bind("y".into(), entry2);
        assert!(ctx.lookup("x").is_some());
        assert!(ctx.lookup("y").is_some());

        ctx.pop_scope();
        assert!(ctx.lookup("x").is_some());
        assert!(ctx.lookup("y").is_none());
    }

    #[test]
    fn test_tracker_literal() {
        let tracker = CompositionTracker::new();
        let entry = tracker.track_literal(&Literal::Int(42), Span::default());

        assert!(
            entry
                .composition
                .primitives
                .contains(&LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn test_binop_primitive() {
        assert_eq!(binop_primitive(BinOp::Add), LexPrimitiva::Quantity);
        assert_eq!(binop_primitive(BinOp::Eq), LexPrimitiva::Comparison);
        assert_eq!(binop_primitive(BinOp::And), LexPrimitiva::Sum);
        assert_eq!(binop_primitive(BinOp::KappaEq), LexPrimitiva::Comparison);
    }

    #[test]
    fn test_unop_primitive() {
        assert_eq!(unop_primitive(UnOp::Neg), LexPrimitiva::Quantity);
        assert_eq!(unop_primitive(UnOp::Not), LexPrimitiva::Sum);
    }

    #[test]
    fn test_merge_with() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let merged = merge_with(&comp, LexPrimitiva::Causality);

        assert!(merged.primitives.contains(&LexPrimitiva::Quantity));
        assert!(merged.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_merge_all() {
        let c1 = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let c2 = PrimitiveComposition::new(vec![LexPrimitiva::Sequence]);
        let c3 = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]); // Duplicate

        let merged = merge_all(&[c1, c2, c3]);
        let unique = merged.unique();

        assert!(unique.contains(&LexPrimitiva::Quantity));
        assert!(unique.contains(&LexPrimitiva::Sequence));
        assert_eq!(unique.len(), 2); // No duplicates
    }

    #[test]
    fn test_context_stats() {
        let mut ctx = CompositionContext::new();

        let entry1 = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
            "int",
            Span::default(),
        );
        let entry2 = CompositionEntry::from_literal(
            PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]),
            "string",
            Span::default(),
        );

        ctx.bind("a".into(), entry1);
        ctx.bind("b".into(), entry2);

        let stats = ctx.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.unique_primitives, 2); // Quantity, Sequence
    }

    #[test]
    fn test_tracking_composition() {
        let comp = tracking_composition();
        let unique = comp.unique();

        assert!(unique.contains(&LexPrimitiva::Sequence));
        assert!(unique.contains(&LexPrimitiva::Mapping));
        assert!(unique.contains(&LexPrimitiva::Causality));
        assert!(unique.contains(&LexPrimitiva::State));
    }
}
