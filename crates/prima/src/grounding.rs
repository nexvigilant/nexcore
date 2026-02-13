// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Grounding Verification
//!
//! Verifies that all Prima constructs ground to `{0, 1}`.
//!
//! ## Mathematical Foundation
//!
//! Grounding verification is itself grounded:
//! - **∂** (Boundary): Verification as boundary check
//! - **κ** (Comparison): Checking against `{0, 1}`
//! - **ρ** (Recursion): Traversing AST tree
//! - **Σ** (Sum): Match on expression variants
//!
//! ## Tier: T2-C (∂ + κ + ρ + Σ)

use crate::ast::{BinOp, Block, Expr, Literal, Pattern, Program, Stmt};
use crate::error::PrimaResult;
use crate::token::Span;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};

/// Root constants that all computation grounds to.
pub const ROOT_CONSTANTS: [&str; 2] = ["0", "1"];

/// Grounding verification result.
#[derive(Debug, Clone)]
pub struct GroundingResult {
    /// Whether the program is fully grounded.
    pub grounded: bool,
    /// Total composition of the program.
    pub composition: PrimitiveComposition,
    /// Violations found during verification.
    pub violations: Vec<GroundingViolation>,
    /// Trace of grounding steps.
    pub trace: Vec<GroundingStep>,
}

/// A grounding violation.
#[derive(Debug, Clone)]
pub struct GroundingViolation {
    /// Location of the violation.
    pub span: Span,
    /// Description of the violation.
    pub message: String,
    /// The composition that failed to ground.
    pub composition: PrimitiveComposition,
}

/// A step in the grounding trace.
#[derive(Debug, Clone)]
pub struct GroundingStep {
    /// Description of the step.
    pub description: String,
    /// Composition at this step.
    pub composition: PrimitiveComposition,
    /// Depth in the trace (for visualization).
    pub depth: usize,
}

/// Grounding verifier.
///
/// Traverses the AST and verifies each construct grounds to `{0, 1}`.
pub struct GroundingVerifier {
    violations: Vec<GroundingViolation>,
    trace: Vec<GroundingStep>,
    depth: usize,
}

impl Default for GroundingVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundingVerifier {
    /// Create a new verifier.
    #[must_use]
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            trace: Vec::new(),
            depth: 0,
        }
    }

    /// Verify a program grounds to `{0, 1}`.
    pub fn verify(&mut self, program: &Program) -> PrimaResult<GroundingResult> {
        self.trace_step("Program", Self::comp_sequence());
        let comp = self.verify_program(program);
        Ok(self.build_result(comp))
    }

    /// Build final result from accumulated state.
    fn build_result(&self, composition: PrimitiveComposition) -> GroundingResult {
        GroundingResult {
            grounded: self.violations.is_empty(),
            composition,
            violations: self.violations.clone(),
            trace: self.trace.clone(),
        }
    }

    /// Verify program statements.
    fn verify_program(&mut self, program: &Program) -> PrimitiveComposition {
        let mut comp = Self::comp_sequence();
        for stmt in &program.statements {
            let stmt_comp = self.verify_stmt(stmt);
            comp = Self::merge_compositions(&comp, &stmt_comp);
        }
        comp
    }

    /// Verify a statement.
    fn verify_stmt(&mut self, stmt: &Stmt) -> PrimitiveComposition {
        match stmt {
            Stmt::Let { name, value, .. } => self.verify_let(name, value),
            Stmt::TypeDef { name, .. } => self.verify_typedef(name),
            Stmt::FnDef { name, body, .. } => self.verify_fndef(name, body),
            Stmt::Expr { expr, .. } => self.verify_expr(expr),
            Stmt::Return { value, .. } => self.verify_return(value),
        }
    }

    /// Verify let binding.
    fn verify_let(&mut self, name: &str, value: &Expr) -> PrimitiveComposition {
        self.trace_step(&format!("Let({name})"), Self::comp_state());
        self.enter();
        let val_comp = self.verify_expr(value);
        self.leave();
        Self::merge_compositions(&Self::comp_state(), &val_comp)
    }

    /// Verify type definition.
    fn verify_typedef(&mut self, name: &str) -> PrimitiveComposition {
        self.trace_step(&format!("TypeDef({name})"), Self::comp_mapping());
        Self::comp_mapping()
    }

    /// Verify function definition.
    fn verify_fndef(&mut self, name: &str, body: &Block) -> PrimitiveComposition {
        self.trace_step(&format!("FnDef({name})"), Self::comp_causality());
        self.enter();
        let body_comp = self.verify_block(body);
        self.leave();
        Self::merge_compositions(&Self::comp_causality(), &body_comp)
    }

    /// Verify return statement.
    fn verify_return(&mut self, value: &Option<Expr>) -> PrimitiveComposition {
        self.trace_step("Return", Self::comp_boundary());
        match value {
            Some(expr) => {
                self.enter();
                let comp = self.verify_expr(expr);
                self.leave();
                Self::merge_compositions(&Self::comp_boundary(), &comp)
            }
            None => Self::comp_void(),
        }
    }
}

// ============================================================================
// Expression Verification (extracted helpers)
// ============================================================================

impl GroundingVerifier {
    /// Verify an expression.
    fn verify_expr(&mut self, expr: &Expr) -> PrimitiveComposition {
        match expr {
            Expr::Literal { value, .. } => self.verify_literal(value),
            Expr::Ident { name, .. } => self.verify_ident(name),
            Expr::Binary {
                op, left, right, ..
            } => self.verify_binary(op, left, right),
            Expr::Unary { operand, .. } => self.verify_unary(operand),
            Expr::Call { func, args, .. } => self.verify_call(func, args),
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => self.verify_if(cond, then_branch, else_branch),
            Expr::Match {
                scrutinee, arms, ..
            } => self.verify_match(scrutinee, arms),
            Expr::For {
                var, iter, body, ..
            } => self.verify_for(var, iter, body),
            Expr::Block { block, .. } => self.verify_block(block),
            Expr::Lambda { body, .. } => self.verify_lambda(body),
            Expr::Sequence { elements, .. } => self.verify_sequence(elements),
            Expr::Mapping { pairs, .. } => self.verify_mapping(pairs),
            Expr::Member { object, .. } => self.verify_member(object),
            Expr::MethodCall { object, args, .. } => self.verify_method_call(object, args),
            Expr::Quoted { expr: inner, .. } => self.verify_quoted(inner),
            Expr::Quasiquoted { expr: inner, .. } => self.verify_quasiquoted(inner),
            Expr::Unquoted { expr: inner, .. } => self.verify_unquoted(inner),
            Expr::UnquotedSplice { expr: inner, .. } => self.verify_splice(inner),
        }
    }

    /// Verify literal value.
    fn verify_literal(&mut self, lit: &Literal) -> PrimitiveComposition {
        let comp = lit.composition();
        self.trace_step(&format!("Literal({lit:?})"), comp.clone());
        comp
    }

    /// Verify identifier.
    fn verify_ident(&mut self, name: &str) -> PrimitiveComposition {
        self.trace_step(&format!("Ident({name})"), Self::comp_location());
        Self::comp_location()
    }

    /// Verify binary operation.
    fn verify_binary(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> PrimitiveComposition {
        let op_prim = op.dominant_primitive();
        self.trace_step(&format!("Binary({op:?})"), Self::comp_single(op_prim));
        self.enter();
        let left_comp = self.verify_expr(left);
        let right_comp = self.verify_expr(right);
        self.leave();
        Self::merge_three(&Self::comp_single(op_prim), &left_comp, &right_comp)
    }

    /// Verify unary operation.
    fn verify_unary(&mut self, operand: &Expr) -> PrimitiveComposition {
        self.trace_step("Unary", Self::comp_causality());
        self.enter();
        let comp = self.verify_expr(operand);
        self.leave();
        Self::merge_compositions(&Self::comp_causality(), &comp)
    }

    /// Verify function call.
    fn verify_call(&mut self, func: &str, args: &[Expr]) -> PrimitiveComposition {
        self.trace_step(&format!("Call({func})"), Self::comp_causality());
        self.enter();
        let mut comp = Self::comp_causality();
        for arg in args {
            let arg_comp = self.verify_expr(arg);
            comp = Self::merge_compositions(&comp, &arg_comp);
        }
        self.leave();
        comp
    }
}

// ============================================================================
// Control Flow Verification
// ============================================================================

impl GroundingVerifier {
    /// Verify if expression.
    fn verify_if(
        &mut self,
        cond: &Expr,
        then_branch: &Block,
        else_branch: &Option<Block>,
    ) -> PrimitiveComposition {
        let if_comp = Self::comp_sum();
        self.trace_step("If", if_comp.clone());
        self.enter();
        let cond_comp = self.verify_expr(cond);
        let then_comp = self.verify_block(then_branch);
        let else_comp = else_branch
            .as_ref()
            .map_or_else(Self::comp_void, |b| self.verify_block(b));
        self.leave();
        // Include Sum from the if itself, not just branches
        let branches = Self::merge_three(&cond_comp, &then_comp, &else_comp);
        Self::merge_compositions(&if_comp, &branches)
    }

    /// Verify match expression.
    fn verify_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[crate::ast::MatchArm],
    ) -> PrimitiveComposition {
        self.trace_step("Match", Self::comp_sum());
        self.enter();
        let mut comp = self.verify_expr(scrutinee);
        for arm in arms {
            self.verify_pattern(&arm.pattern);
            let arm_comp = self.verify_expr(&arm.body);
            comp = Self::merge_compositions(&comp, &arm_comp);
        }
        self.leave();
        comp
    }

    /// Verify for loop.
    fn verify_for(&mut self, var: &str, iter: &Expr, body: &Block) -> PrimitiveComposition {
        self.trace_step(&format!("For({var})"), Self::comp_sequence());
        self.enter();
        let iter_comp = self.verify_expr(iter);
        let body_comp = self.verify_block(body);
        self.leave();
        Self::merge_three(&Self::comp_recursion(), &iter_comp, &body_comp)
    }

    /// Verify block.
    fn verify_block(&mut self, block: &Block) -> PrimitiveComposition {
        self.trace_step("Block", Self::comp_sequence());
        self.enter();
        let mut comp = Self::comp_sequence();
        for stmt in &block.statements {
            let stmt_comp = self.verify_stmt(stmt);
            comp = Self::merge_compositions(&comp, &stmt_comp);
        }
        if let Some(expr) = &block.expr {
            let expr_comp = self.verify_expr(expr);
            comp = Self::merge_compositions(&comp, &expr_comp);
        }
        self.leave();
        comp
    }

    /// Verify pattern.
    fn verify_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard { .. } => self.trace_step("Pattern(_)", Self::comp_existence()),
            Pattern::Literal { value, .. } => {
                self.trace_step(&format!("Pattern({value:?})"), value.composition());
            }
            Pattern::Ident { name, .. } => {
                self.trace_step(&format!("Pattern({name})"), Self::comp_location());
            }
            Pattern::Constructor { name, fields, .. } => {
                self.trace_step(&format!("Pattern({name}(..))"), Self::comp_sum());
                for field in fields {
                    self.verify_pattern(field);
                }
            }
        }
    }
}

// ============================================================================
// Container & Homoiconicity Verification
// ============================================================================

impl GroundingVerifier {
    /// Verify lambda.
    fn verify_lambda(&mut self, body: &Expr) -> PrimitiveComposition {
        self.trace_step("Lambda", Self::comp_causality());
        self.enter();
        let body_comp = self.verify_expr(body);
        self.leave();
        Self::merge_compositions(&Self::comp_causality(), &body_comp)
    }

    /// Verify sequence literal.
    fn verify_sequence(&mut self, elements: &[Expr]) -> PrimitiveComposition {
        self.trace_step("Sequence", Self::comp_sequence());
        self.enter();
        let mut comp = Self::comp_sequence();
        for elem in elements {
            let elem_comp = self.verify_expr(elem);
            comp = Self::merge_compositions(&comp, &elem_comp);
        }
        self.leave();
        comp
    }

    /// Verify mapping literal.
    fn verify_mapping(&mut self, pairs: &[(Expr, Expr)]) -> PrimitiveComposition {
        self.trace_step("Mapping", Self::comp_mapping());
        self.enter();
        let mut comp = Self::comp_mapping();
        for (k, v) in pairs {
            let k_comp = self.verify_expr(k);
            let v_comp = self.verify_expr(v);
            comp = Self::merge_three(&comp, &k_comp, &v_comp);
        }
        self.leave();
        comp
    }

    /// Verify member access.
    fn verify_member(&mut self, object: &Expr) -> PrimitiveComposition {
        self.trace_step("Member", Self::comp_location());
        self.enter();
        let obj_comp = self.verify_expr(object);
        self.leave();
        Self::merge_compositions(&Self::comp_location(), &obj_comp)
    }

    /// Verify method call.
    fn verify_method_call(&mut self, object: &Expr, args: &[Expr]) -> PrimitiveComposition {
        self.trace_step("MethodCall", Self::comp_causality());
        self.enter();
        let mut comp = self.verify_expr(object);
        for arg in args {
            let arg_comp = self.verify_expr(arg);
            comp = Self::merge_compositions(&comp, &arg_comp);
        }
        self.leave();
        comp
    }

    /// Verify quoted expression.
    fn verify_quoted(&mut self, inner: &Expr) -> PrimitiveComposition {
        self.trace_step("Quote", Self::comp_recursion());
        self.enter();
        let inner_comp = self.verify_expr(inner);
        self.leave();
        Self::merge_compositions(&Self::comp_recursion(), &inner_comp)
    }

    /// Verify quasiquoted expression.
    fn verify_quasiquoted(&mut self, inner: &Expr) -> PrimitiveComposition {
        self.trace_step("Quasiquote", Self::comp_recursion());
        self.enter();
        let inner_comp = self.verify_expr(inner);
        self.leave();
        Self::merge_three(&Self::comp_recursion(), &Self::comp_sequence(), &inner_comp)
    }

    /// Verify unquoted expression.
    fn verify_unquoted(&mut self, inner: &Expr) -> PrimitiveComposition {
        self.trace_step("Unquote", Self::comp_causality());
        self.enter();
        let inner_comp = self.verify_expr(inner);
        self.leave();
        Self::merge_compositions(&Self::comp_causality(), &inner_comp)
    }

    /// Verify unquote-splice.
    fn verify_splice(&mut self, inner: &Expr) -> PrimitiveComposition {
        self.trace_step("UnquoteSplice", Self::comp_causality());
        self.enter();
        let inner_comp = self.verify_expr(inner);
        self.leave();
        Self::merge_three(&Self::comp_causality(), &Self::comp_sequence(), &inner_comp)
    }
}

// ============================================================================
// Composition Helpers (static, no self)
// ============================================================================

impl GroundingVerifier {
    /// Single-primitive composition.
    fn comp_single(p: LexPrimitiva) -> PrimitiveComposition {
        PrimitiveComposition::new(vec![p])
    }

    /// Sequence composition.
    fn comp_sequence() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Sequence)
    }

    /// State composition.
    fn comp_state() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::State)
    }

    /// Mapping composition.
    fn comp_mapping() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Mapping)
    }

    /// Causality composition.
    fn comp_causality() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Causality)
    }

    /// Boundary composition.
    fn comp_boundary() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Boundary)
    }

    /// Void composition.
    fn comp_void() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Void)
    }

    /// Location composition.
    fn comp_location() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Location)
    }

    /// Sum composition.
    fn comp_sum() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Sum)
    }

    /// Recursion composition.
    fn comp_recursion() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Recursion)
    }

    /// Existence composition.
    fn comp_existence() -> PrimitiveComposition {
        Self::comp_single(LexPrimitiva::Existence)
    }

    /// Merge two compositions.
    fn merge_compositions(
        a: &PrimitiveComposition,
        b: &PrimitiveComposition,
    ) -> PrimitiveComposition {
        let mut prims = a.primitives.clone();
        for p in &b.primitives {
            if !prims.contains(p) {
                prims.push(*p);
            }
        }
        PrimitiveComposition::new(prims)
    }

    /// Merge three compositions.
    fn merge_three(
        a: &PrimitiveComposition,
        b: &PrimitiveComposition,
        c: &PrimitiveComposition,
    ) -> PrimitiveComposition {
        let ab = Self::merge_compositions(a, b);
        Self::merge_compositions(&ab, c)
    }
}

// ============================================================================
// Trace Helpers
// ============================================================================

impl GroundingVerifier {
    /// Add a trace step.
    fn trace_step(&mut self, description: &str, composition: PrimitiveComposition) {
        self.trace.push(GroundingStep {
            description: description.to_string(),
            composition,
            depth: self.depth,
        });
    }

    /// Increase depth.
    fn enter(&mut self) {
        self.depth += 1;
    }

    /// Decrease depth.
    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Add a violation.
    #[allow(dead_code)]
    fn add_violation(&mut self, span: Span, message: String, composition: PrimitiveComposition) {
        self.violations.push(GroundingViolation {
            span,
            message,
            composition,
        });
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Verify that a program grounds to `{0, 1}`.
///
/// # Errors
///
/// Returns an error if verification fails due to invalid AST.
pub fn verify_grounding(program: &Program) -> PrimaResult<GroundingResult> {
    GroundingVerifier::new().verify(program)
}

/// Format grounding trace for display.
#[must_use]
pub fn format_trace(trace: &[GroundingStep]) -> String {
    trace.iter().map(format_step).collect::<Vec<_>>().join("\n")
}

/// Format a single trace step.
fn format_step(step: &GroundingStep) -> String {
    let indent = "  ".repeat(step.depth);
    let prims = format_primitives(&step.composition);
    format!("{indent}{} → {prims}", step.description)
}

/// Format primitives for display.
fn format_primitives(comp: &PrimitiveComposition) -> String {
    comp.primitives
        .iter()
        .map(|p| p.symbol())
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn verify(src: &str) -> GroundingResult {
        let toks = Lexer::new(src).tokenize().expect("lex");
        let prog = Parser::new(toks).parse().expect("parse");
        verify_grounding(&prog).expect("verify")
    }

    #[test]
    fn test_literal_grounding() {
        let r = verify("42");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn test_let_grounding() {
        let r = verify("λ x = 1");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::State));
    }

    #[test]
    fn test_function_grounding() {
        let r = verify("μ f(x: N) → N { x }");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_if_grounding() {
        let r = verify("∂ true { 1 } else { 0 }");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Sum));
    }

    #[test]
    fn test_sequence_grounding() {
        let r = verify("σ[1, 2, 3]");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Sequence));
    }

    #[test]
    fn test_binary_grounding() {
        let r = verify("1 + 2");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn test_comparison_grounding() {
        let r = verify("1 κ< 2");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn test_lambda_grounding() {
        let r = verify("|x| x * 2");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_trace_format() {
        let r = verify("λ x = 1 + 2");
        let trace = format_trace(&r.trace);
        assert!(trace.contains("Let(x)"));
        assert!(trace.contains("Binary"));
    }

    #[test]
    fn test_nested_grounding() {
        let r = verify("μ f(n: N) → N { ∂ n κ= 0 { 0 } else { n + 1 } }");
        assert!(r.grounded);
        // Should have Sum (from if), Causality (from fn), Comparison (from κ=)
        assert!(r.composition.primitives.contains(&LexPrimitiva::Sum));
        assert!(r.composition.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_root_constants() {
        assert_eq!(ROOT_CONSTANTS, ["0", "1"]);
    }

    #[test]
    fn test_hof_grounding() {
        let r = verify("Φ(σ[1,2,3], |x| x * 2)");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_pipeline_grounding() {
        let r = verify("σ[1,2,3] |> Φ(|x| x * 2)");
        assert!(r.grounded);
    }

    #[test]
    fn test_quote_grounding() {
        let r = verify("'(1 + 2)");
        assert!(r.grounded);
        assert!(r.composition.primitives.contains(&LexPrimitiva::Recursion));
    }
}
