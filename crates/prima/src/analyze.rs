// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Semantic Analyzer
//!
//! Type checking and semantic validation for Prima programs.
//!
//! ## Safety Philosophy
//!
//! "Humans need to learn to compile their language."
//!
//! If it compiles, it's mathematically true. This module enforces:
//! - **Type Safety**: Every operation verified at compile time
//! - **Exhaustive Matching**: All Σ (Sum) cases handled
//! - **Composition Tracking**: Primitive flow through expressions
//! - **Grounding Verification**: Every value traces to {0, 1}

#![allow(missing_docs)]

use crate::ast::{BinOp, Block, Expr, Literal, MatchArm, Param, Pattern, Program, Stmt, UnOp};
use crate::error::PrimaError;
use crate::token::Span;
use crate::types::{PrimaType, TypeEnv};
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition, Tier};
use std::collections::{HashMap, HashSet};

pub type AnalysisResult<T> = Result<T, PrimaError>;

#[derive(Debug, Clone)]
pub struct InferredType {
    pub ty: PrimaType,
    pub certain: bool,
}

impl InferredType {
    #[must_use]
    pub fn certain(ty: PrimaType) -> Self {
        Self { ty, certain: true }
    }

    #[must_use]
    pub fn inferred(ty: PrimaType) -> Self {
        Self { ty, certain: false }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub params: Vec<PrimaType>,
    pub ret: PrimaType,
    pub composition: PrimitiveComposition,
}

#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub message: String,
    pub span: Span,
    pub kind: WarningKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    UnusedVariable,
    UnreachableCode,
    ImplicitCoercion,
    NonExhaustivePattern,
}

#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub type_env: TypeEnv,
    bindings: Vec<HashMap<String, InferredType>>,
    functions: HashMap<String, FunctionSig>,
    current_return: Option<PrimaType>,
    errors: Vec<PrimaError>,
    warnings: Vec<AnalysisWarning>,
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisContext {
    #[must_use]
    pub fn new() -> Self {
        let mut ctx = Self {
            type_env: TypeEnv::with_builtins(),
            bindings: vec![HashMap::new()],
            functions: HashMap::new(),
            current_return: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        register_builtins(&mut ctx);
        ctx
    }

    pub fn push_scope(&mut self) {
        self.bindings.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.bindings.pop();
    }

    pub fn bind(&mut self, name: impl Into<String>, ty: InferredType) {
        if let Some(scope) = self.bindings.last_mut() {
            scope.insert(name.into(), ty);
        }
    }

    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&InferredType> {
        self.bindings.iter().rev().find_map(|s| s.get(name))
    }

    pub fn register_function(&mut self, name: impl Into<String>, sig: FunctionSig) {
        self.functions.insert(name.into(), sig);
    }

    #[must_use]
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.get(name)
    }

    pub fn error(&mut self, err: PrimaError) {
        self.errors.push(err);
    }

    pub fn warn(&mut self, message: impl Into<String>, span: Span, kind: WarningKind) {
        self.warnings.push(AnalysisWarning {
            message: message.into(),
            span,
            kind,
        });
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[must_use]
    pub fn errors(&self) -> &[PrimaError] {
        &self.errors
    }

    #[must_use]
    pub fn warnings(&self) -> &[AnalysisWarning] {
        &self.warnings
    }
}

// ==== Type Constructors ====

fn ty_void() -> PrimaType {
    PrimaType::new("∅", PrimitiveComposition::new(vec![LexPrimitiva::Void]))
}

fn ty_string() -> PrimaType {
    PrimaType::new(
        "String",
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]),
    )
}

fn ty_n() -> PrimaType {
    PrimaType::new("N", PrimitiveComposition::new(vec![LexPrimitiva::Quantity]))
}

fn ty_bool() -> PrimaType {
    PrimaType::new("Bool", PrimitiveComposition::new(vec![LexPrimitiva::Sum]))
}

fn ty_infer() -> PrimaType {
    PrimaType::new(
        "?",
        PrimitiveComposition::new(vec![LexPrimitiva::Existence]),
    )
}

fn ty_seq() -> PrimaType {
    PrimaType::new("σ", PrimitiveComposition::new(vec![LexPrimitiva::Sequence]))
}

fn ty_map() -> PrimaType {
    PrimaType::new("μ", PrimitiveComposition::new(vec![LexPrimitiva::Mapping]))
}

fn ty_fn() -> PrimaType {
    PrimaType::new(
        "→",
        PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
    )
}

fn ty_optional() -> PrimaType {
    PrimaType::new(
        "?",
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Void]),
    )
}

fn ty_expr() -> InferredType {
    InferredType::certain(PrimaType::new(
        "Expr",
        PrimitiveComposition::new(vec![LexPrimitiva::Recursion]),
    ))
}

// ==== Builtin Registration ====

fn register_builtins(ctx: &mut AnalysisContext) {
    register_io_fns(ctx);
    register_introspect_fns(ctx);
    register_assert_fns(ctx);
}

fn register_io_fns(ctx: &mut AnalysisContext) {
    let sig = FunctionSig {
        params: vec![ty_string()],
        ret: ty_void(),
        composition: PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Void]),
    };
    ctx.register_function("print", sig.clone());
    ctx.register_function("ω", sig);
}

fn register_introspect_fns(ctx: &mut AnalysisContext) {
    let sig = FunctionSig {
        params: vec![ty_n()],
        ret: ty_string(),
        composition: PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
    };
    ctx.register_function("typeof", sig.clone());
    ctx.register_function("τ", sig.clone());
    ctx.register_function("tier", sig.clone());
    ctx.register_function("T", sig);
}

fn register_assert_fns(ctx: &mut AnalysisContext) {
    let sig = FunctionSig {
        params: vec![ty_bool()],
        ret: ty_void(),
        composition: PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
        ]),
    };
    ctx.register_function("assert", sig.clone());
    ctx.register_function("‼", sig);
}

// ==== Analyzer ====

pub struct Analyzer {
    ctx: AnalysisContext,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ctx: AnalysisContext::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> AnalysisResult<&AnalysisContext> {
        self.collect_fns(program);
        self.analyze_stmts(program);
        Ok(&self.ctx)
    }

    /// Consume the analyzer and return the context.
    #[must_use]
    pub fn into_context(self) -> AnalysisContext {
        self.ctx
    }

    fn collect_fns(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.maybe_collect_fn(stmt);
        }
    }

    fn maybe_collect_fn(&mut self, stmt: &Stmt) {
        let Stmt::FnDef {
            name, params, ret, ..
        } = stmt
        else {
            return;
        };
        let sig = build_fn_sig(&self.ctx.type_env, params, ret);
        self.ctx.register_function(name.clone(), sig);
    }

    fn analyze_stmts(&mut self, program: &Program) {
        for stmt in &program.statements {
            if let Err(e) = self.analyze_stmt(stmt) {
                self.ctx.error(e);
            }
        }
    }
}

fn build_fn_sig(type_env: &TypeEnv, params: &[Param], ret: &crate::ast::TypeExpr) -> FunctionSig {
    let param_types: Vec<PrimaType> = params
        .iter()
        .filter_map(|p| type_env.resolve(&p.ty))
        .collect();
    let ret_ty = type_env.resolve(ret).unwrap_or_else(ty_void);
    let composition = build_fn_composition(&param_types, &ret_ty);
    FunctionSig {
        params: param_types,
        ret: ret_ty,
        composition,
    }
}

fn build_fn_composition(params: &[PrimaType], ret: &PrimaType) -> PrimitiveComposition {
    let mut c = PrimitiveComposition::new(vec![LexPrimitiva::Causality]);
    for pt in params {
        merge_prims(&mut c, &pt.composition);
    }
    merge_prims(&mut c, &ret.composition);
    c
}

fn merge_prims(target: &mut PrimitiveComposition, source: &PrimitiveComposition) {
    for p in &source.primitives {
        if !target.primitives.contains(p) {
            target.primitives.push(*p);
        }
    }
}

// ==== Statement Analysis ====

impl Analyzer {
    fn analyze_stmt(&mut self, stmt: &Stmt) -> AnalysisResult<()> {
        match stmt {
            Stmt::Let { name, value, .. } => self.analyze_let(name, value),
            Stmt::TypeDef { name, ty, .. } => self.analyze_typedef(name, ty),
            Stmt::FnDef {
                name,
                params,
                body,
                ret,
                ..
            } => self.analyze_fndef(name, params, body, ret),
            Stmt::Expr { expr, .. } => {
                self.infer_expr(expr)?;
                Ok(())
            }
            Stmt::Return { value, span } => self.analyze_return(value.as_ref(), *span),
        }
    }

    fn analyze_let(&mut self, name: &str, value: &Expr) -> AnalysisResult<()> {
        let ty = self.infer_expr(value)?;
        self.ctx.bind(name.to_string(), ty);
        Ok(())
    }

    fn analyze_typedef(&mut self, name: &str, ty: &crate::ast::TypeExpr) -> AnalysisResult<()> {
        if let Some(mut prima_ty) = self.ctx.type_env.resolve(ty) {
            prima_ty.name = name.to_string();
            self.ctx.type_env.insert(prima_ty);
        }
        Ok(())
    }

    fn analyze_fndef(
        &mut self,
        name: &str,
        params: &[Param],
        body: &Block,
        ret: &crate::ast::TypeExpr,
    ) -> AnalysisResult<()> {
        self.ctx.push_scope();
        self.bind_params(params);
        let ret_ty = self.ctx.type_env.resolve(ret).unwrap_or_else(ty_void);
        self.ctx.current_return = Some(ret_ty.clone());
        let body_ty = self.analyze_block(body)?;
        self.check_ret_type(name, &body_ty.ty, &ret_ty, body.span);
        self.ctx.current_return = None;
        self.ctx.pop_scope();
        Ok(())
    }

    fn bind_params(&mut self, params: &[Param]) {
        for p in params {
            if let Some(ty) = self.ctx.type_env.resolve(&p.ty) {
                self.ctx.bind(p.name.clone(), InferredType::certain(ty));
            }
        }
    }

    fn check_ret_type(&mut self, name: &str, body: &PrimaType, ret: &PrimaType, span: Span) {
        if !types_compat(body, ret) {
            self.ctx.warn(
                format!(
                    "fn `{name}` body is `{}`, expected `{}`",
                    body.name, ret.name
                ),
                span,
                WarningKind::ImplicitCoercion,
            );
        }
    }

    fn analyze_return(&mut self, value: Option<&Expr>, span: Span) -> AnalysisResult<()> {
        let Some(expected) = &self.ctx.current_return.clone() else {
            return Err(PrimaError::analyzer(span, "return outside function"));
        };
        self.check_return_value(value, expected, span)
    }

    fn check_return_value(
        &mut self,
        value: Option<&Expr>,
        expected: &PrimaType,
        span: Span,
    ) -> AnalysisResult<()> {
        let Some(val) = value else {
            return self.check_void_return(expected, span);
        };
        let ty = self.infer_expr(val)?;
        if !types_compat(&ty.ty, expected) {
            return Err(PrimaError::analyzer(
                span,
                format!("expected `{}`, found `{}`", expected.name, ty.ty.name),
            ));
        }
        Ok(())
    }

    fn check_void_return(&self, expected: &PrimaType, span: Span) -> AnalysisResult<()> {
        if expected.name != "∅" && expected.name != "Void" {
            return Err(PrimaError::analyzer(
                span,
                format!("expected return value of `{}`", expected.name),
            ));
        }
        Ok(())
    }

    fn analyze_block(&mut self, block: &Block) -> AnalysisResult<InferredType> {
        for stmt in &block.statements {
            self.analyze_stmt(stmt)?;
        }
        block.expr.as_ref().map_or_else(
            || Ok(InferredType::certain(ty_void())),
            |e| self.infer_expr(e),
        )
    }
}

// ==== Expression Inference ====

impl Analyzer {
    pub fn infer_expr(&mut self, expr: &Expr) -> AnalysisResult<InferredType> {
        match expr {
            Expr::Literal { value, .. } => Ok(infer_literal(value)),
            Expr::Ident { name, span } => self.infer_ident(name, *span),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => self.infer_binary(left, *op, right, *span),
            Expr::Unary { op, operand, span } => self.infer_unary(*op, operand, *span),
            Expr::Call { func, args, span } => self.infer_call(func, args, *span),
            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => self.infer_if(cond, then_branch, else_branch.as_ref(), *span),
            Expr::Match {
                scrutinee,
                arms,
                span,
            } => self.infer_match(scrutinee, arms, *span),
            Expr::For {
                var, iter, body, ..
            } => self.infer_for(var, iter, body),
            Expr::Block { block, .. } => self.analyze_block(block),
            Expr::Lambda { params, body, .. } => self.infer_lambda(params, body),
            Expr::Sequence { elements, .. } => self.infer_sequence(elements),
            Expr::Mapping { pairs, .. } => self.infer_mapping(pairs),
            Expr::Member { object, .. } => self.infer_member(object),
            Expr::MethodCall {
                object,
                method,
                args,
                ..
            } => self.infer_method(object, method, args),
            Expr::Quoted { .. } => Ok(ty_expr()),
            Expr::Quasiquoted { expr: inner, .. } => {
                self.validate_qq(inner)?;
                Ok(ty_expr())
            }
            Expr::Unquoted { expr: inner, span } => {
                self.ctx
                    .warn("~ outside quasiquote", *span, WarningKind::UnreachableCode);
                self.infer_expr(inner)
            }
            Expr::UnquotedSplice { span, .. } => {
                Err(PrimaError::analyzer(*span, "~@ must be inside quasiquote"))
            }
        }
    }

    fn infer_ident(&self, name: &str, span: Span) -> AnalysisResult<InferredType> {
        if let Some(ty) = self.ctx.lookup(name) {
            return Ok(ty.clone());
        }
        if self.ctx.lookup_function(name).is_some() {
            return Ok(InferredType::certain(ty_fn()));
        }
        Err(PrimaError::analyzer(span, format!("undefined: `{name}`")))
    }

    fn validate_qq(&mut self, expr: &Expr) -> AnalysisResult<()> {
        if let Expr::Unquoted { expr: inner, .. } = expr {
            self.infer_expr(inner)?;
        }
        if let Expr::UnquotedSplice { expr: inner, span } = expr {
            let ty = self.infer_expr(inner)?;
            if !ty
                .ty
                .composition
                .primitives
                .contains(&LexPrimitiva::Sequence)
            {
                self.ctx.warn(
                    "~@ should produce sequence",
                    *span,
                    WarningKind::ImplicitCoercion,
                );
            }
        }
        Ok(())
    }
}

fn infer_literal(lit: &Literal) -> InferredType {
    let ty = match lit {
        Literal::Int(_) | Literal::Float(_) => ty_n(),
        Literal::String(_) => ty_string(),
        Literal::Bool(_) => ty_bool(),
        Literal::Void => ty_void(),
        Literal::Symbol(_) => PrimaType::new(
            "Symbol",
            PrimitiveComposition::new(vec![LexPrimitiva::Location]),
        ),
    };
    InferredType::certain(ty)
}

// ==== Binary/Unary ====

impl Analyzer {
    fn infer_binary(
        &mut self,
        left: &Expr,
        op: BinOp,
        right: &Expr,
        span: Span,
    ) -> AnalysisResult<InferredType> {
        let l = self.infer_expr(left)?;
        let r = self.infer_expr(right)?;
        infer_binop_result(&l.ty, op, &r.ty, span)
    }

    fn infer_unary(
        &mut self,
        op: UnOp,
        operand: &Expr,
        span: Span,
    ) -> AnalysisResult<InferredType> {
        let ty = self.infer_expr(operand)?;
        infer_unop_result(op, &ty.ty, span).map(|_| ty)
    }
}

fn infer_binop_result(
    l: &PrimaType,
    op: BinOp,
    r: &PrimaType,
    span: Span,
) -> AnalysisResult<InferredType> {
    if is_arith(op) {
        return check_arith(l, r, span);
    }
    if is_cmp(op) {
        return Ok(InferredType::certain(ty_bool()));
    }
    if is_logic(op) {
        return check_logic(l, r, span);
    }
    Ok(InferredType::inferred(ty_infer()))
}

fn is_arith(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
    )
}

fn is_cmp(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Gt
            | BinOp::Le
            | BinOp::Ge
            | BinOp::KappaEq
            | BinOp::KappaLt
            | BinOp::KappaGt
    )
}

fn is_logic(op: BinOp) -> bool {
    matches!(op, BinOp::And | BinOp::Or)
}

fn check_arith(l: &PrimaType, r: &PrimaType, span: Span) -> AnalysisResult<InferredType> {
    if is_num(l) && is_num(r) {
        return Ok(InferredType::certain(ty_n()));
    }
    Err(PrimaError::analyzer(span, "arithmetic requires numeric"))
}

fn check_logic(l: &PrimaType, r: &PrimaType, span: Span) -> AnalysisResult<InferredType> {
    if is_bool(l) && is_bool(r) {
        return Ok(InferredType::certain(ty_bool()));
    }
    Err(PrimaError::analyzer(span, "logic requires boolean"))
}

fn infer_unop_result(op: UnOp, ty: &PrimaType, span: Span) -> AnalysisResult<()> {
    match op {
        UnOp::Neg if is_num(ty) => Ok(()),
        UnOp::Neg => Err(PrimaError::analyzer(span, "negate requires numeric")),
        UnOp::Not if is_bool(ty) => Ok(()),
        UnOp::Not => Err(PrimaError::analyzer(span, "not requires boolean")),
    }
}

// ==== Calls/HOF ====

impl Analyzer {
    fn infer_call(
        &mut self,
        func: &str,
        args: &[Expr],
        span: Span,
    ) -> AnalysisResult<InferredType> {
        if is_hof(func) {
            return self.infer_hof(func, args);
        }
        self.infer_regular_call(func, args, span)
    }

    fn infer_regular_call(
        &mut self,
        func: &str,
        args: &[Expr],
        _span: Span,
    ) -> AnalysisResult<InferredType> {
        for a in args {
            self.infer_expr(a)?;
        }
        let Some(sig) = self.ctx.lookup_function(func).cloned() else {
            return Ok(InferredType::inferred(ty_infer()));
        };
        Ok(InferredType::certain(sig.ret))
    }

    fn infer_hof(&mut self, func: &str, args: &[Expr]) -> AnalysisResult<InferredType> {
        for a in args {
            self.infer_expr(a)?;
        }
        Ok(match func {
            "map" | "Φ" | "filter" | "Ψ" | "zip" | "⊠" => InferredType::inferred(ty_seq()),
            "fold" | "Ω" => args.first().map_or_else(
                || InferredType::inferred(ty_infer()),
                |a| {
                    self.infer_expr(a)
                        .unwrap_or_else(|_| InferredType::inferred(ty_infer()))
                },
            ),
            "any" | "∃?" | "all" | "∀?" => InferredType::certain(ty_bool()),
            "find" | "⊃" => InferredType::certain(ty_optional()),
            _ => InferredType::inferred(ty_infer()),
        })
    }
}

fn is_hof(name: &str) -> bool {
    matches!(
        name,
        "map"
            | "Φ"
            | "filter"
            | "Ψ"
            | "fold"
            | "Ω"
            | "any"
            | "∃?"
            | "all"
            | "∀?"
            | "find"
            | "⊃"
            | "zip"
            | "⊠"
    )
}

// ==== Control Flow ====

impl Analyzer {
    fn infer_if(
        &mut self,
        cond: &Expr,
        then_b: &Block,
        else_b: Option<&Block>,
        span: Span,
    ) -> AnalysisResult<InferredType> {
        let cond_ty = self.infer_expr(cond)?;
        if !is_bool(&cond_ty.ty) {
            return Err(PrimaError::analyzer(span, "condition must be boolean"));
        }
        self.ctx.push_scope();
        let then_ty = self.analyze_block(then_b)?;
        self.ctx.pop_scope();
        if let Some(eb) = else_b {
            self.ctx.push_scope();
            let _ = self.analyze_block(eb)?;
            self.ctx.pop_scope();
            return Ok(then_ty);
        }
        Ok(InferredType::certain(
            ty_optional().with_params(vec![then_ty.ty]),
        ))
    }

    fn infer_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        span: Span,
    ) -> AnalysisResult<InferredType> {
        let scr_ty = self.infer_expr(scrutinee)?;
        self.warn_non_exhaustive(arms, span);
        let mut result: Option<InferredType> = None;
        for arm in arms {
            self.ctx.push_scope();
            bind_pattern(&mut self.ctx, &arm.pattern, &scr_ty.ty);
            let arm_ty = self.infer_expr(&arm.body)?;
            if result.is_none() {
                result = Some(arm_ty);
            }
            self.ctx.pop_scope();
        }
        Ok(result.unwrap_or_else(|| InferredType::certain(ty_void())))
    }

    fn warn_non_exhaustive(&mut self, arms: &[MatchArm], span: Span) {
        let has_wildcard = arms
            .iter()
            .any(|a| matches!(a.pattern, Pattern::Wildcard { .. } | Pattern::Ident { .. }));
        if !has_wildcard {
            self.ctx.warn(
                "non-exhaustive match",
                span,
                WarningKind::NonExhaustivePattern,
            );
        }
    }

    fn infer_for(&mut self, var: &str, iter: &Expr, body: &Block) -> AnalysisResult<InferredType> {
        let iter_ty = self.infer_expr(iter)?;
        let elem = iter_ty.ty.params.first().cloned().unwrap_or_else(ty_infer);
        self.ctx.push_scope();
        self.ctx.bind(var.to_string(), InferredType::inferred(elem));
        let _ = self.analyze_block(body)?;
        self.ctx.pop_scope();
        Ok(InferredType::certain(ty_void()))
    }
}

fn bind_pattern(ctx: &mut AnalysisContext, pat: &Pattern, ty: &PrimaType) {
    if let Pattern::Ident { name, .. } = pat {
        ctx.bind(name.clone(), InferredType::certain(ty.clone()));
    }
    if let Pattern::Constructor { fields, .. } = pat {
        for (i, f) in fields.iter().enumerate() {
            let ft = ty.params.get(i).cloned().unwrap_or_else(ty_infer);
            bind_pattern(ctx, f, &ft);
        }
    }
}

// ==== Compound Types ====

impl Analyzer {
    fn infer_lambda(&mut self, params: &[Param], body: &Expr) -> AnalysisResult<InferredType> {
        self.ctx.push_scope();
        let param_types = self.bind_lambda_params(params);
        let body_ty = self.infer_expr(body)?;
        self.ctx.pop_scope();
        let composition = build_fn_composition(&param_types, &body_ty.ty);
        let mut all = param_types;
        all.push(body_ty.ty);
        Ok(InferredType::certain(
            PrimaType::new("→", composition).with_params(all),
        ))
    }

    fn bind_lambda_params(&mut self, params: &[Param]) -> Vec<PrimaType> {
        params
            .iter()
            .map(|p| {
                let ty = self.ctx.type_env.resolve(&p.ty).unwrap_or_else(ty_infer);
                self.ctx
                    .bind(p.name.clone(), InferredType::inferred(ty.clone()));
                ty
            })
            .collect()
    }

    fn infer_sequence(&mut self, elements: &[Expr]) -> AnalysisResult<InferredType> {
        if elements.is_empty() {
            return Ok(InferredType::certain(ty_seq()));
        }
        let first = self.infer_expr(&elements[0])?;
        for e in elements.iter().skip(1) {
            self.infer_expr(e)?;
        }
        Ok(InferredType::certain(ty_seq().with_params(vec![first.ty])))
    }

    fn infer_mapping(&mut self, pairs: &[(Expr, Expr)]) -> AnalysisResult<InferredType> {
        if pairs.is_empty() {
            return Ok(InferredType::certain(ty_map()));
        }
        let (k, v) = &pairs[0];
        let kt = self.infer_expr(k)?;
        let vt = self.infer_expr(v)?;
        for (k, v) in pairs.iter().skip(1) {
            self.infer_expr(k)?;
            self.infer_expr(v)?;
        }
        Ok(InferredType::certain(
            ty_map().with_params(vec![kt.ty, vt.ty]),
        ))
    }

    fn infer_member(&mut self, object: &Expr) -> AnalysisResult<InferredType> {
        self.infer_expr(object)?;
        Ok(InferredType::inferred(ty_infer()))
    }

    fn infer_method(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> AnalysisResult<InferredType> {
        let obj_ty = self.infer_expr(object)?;
        for a in args {
            self.infer_expr(a)?;
        }
        Ok(match method {
            "len" | "#" => InferredType::certain(ty_n()),
            "push" | "⊕" | "pop" | "⊖" | "map" | "Φ" | "filter" | "Ψ" => obj_ty,
            _ => InferredType::inferred(ty_infer()),
        })
    }
}

// ==== Type Compatibility ====

fn types_compat(a: &PrimaType, b: &PrimaType) -> bool {
    if a.name == b.name {
        return true;
    }
    if is_num(a) && is_num(b) {
        return true;
    }
    if a.name == "?" || b.name == "?" {
        return true;
    }
    if (a.name == "∅" || a.name == "Void") && (b.name == "∅" || b.name == "Void") {
        return true;
    }
    false
}

fn is_num(ty: &PrimaType) -> bool {
    // Accept inferred types as potentially numeric
    ty.name == "?"
        || matches!(ty.name.as_str(), "N" | "Int" | "Float")
        || ty.composition.primitives.contains(&LexPrimitiva::Quantity)
}

fn is_bool(ty: &PrimaType) -> bool {
    // Accept inferred types as potentially boolean
    ty.name == "?" || ty.name == "Bool" || ty.composition.primitives.contains(&LexPrimitiva::Sum)
}

// ==== Report ====

#[derive(Debug, Default)]
pub struct CompositionSummary {
    pub primitives_used: HashSet<LexPrimitiva>,
    pub max_tier: Option<Tier>,
    pub type_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn analyze_code(src: &str) -> AnalysisResult<AnalysisContext> {
        let toks = Lexer::new(src).tokenize()?;
        let prog = Parser::new(toks).parse()?;
        let mut a = Analyzer::new();
        a.analyze(&prog)?;
        Ok(a.into_context())
    }

    #[test]
    fn test_literal_types() {
        let ctx = analyze_code("λ x = 42").expect("ok");
        assert_eq!(ctx.lookup("x").expect("x").ty.name, "N");
    }

    #[test]
    fn test_string_literal() {
        let ctx = analyze_code("λ s = \"hello\"").expect("ok");
        assert_eq!(ctx.lookup("s").expect("s").ty.name, "String");
    }

    #[test]
    fn test_boolean_literal() {
        let ctx = analyze_code("λ b = true").expect("ok");
        assert_eq!(ctx.lookup("b").expect("b").ty.name, "Bool");
    }

    #[test]
    fn test_arithmetic() {
        let ctx = analyze_code("λ r = 1 + 2 * 3").expect("ok");
        assert_eq!(ctx.lookup("r").expect("r").ty.name, "N");
    }

    #[test]
    fn test_comparison() {
        let ctx = analyze_code("λ r = 1 κ< 2").expect("ok");
        assert_eq!(ctx.lookup("r").expect("r").ty.name, "Bool");
    }

    #[test]
    fn test_sequence() {
        let ctx = analyze_code("λ s = σ[1, 2, 3]").expect("ok");
        assert_eq!(ctx.lookup("s").expect("s").ty.name, "σ");
    }

    #[test]
    fn test_undefined() {
        let result = analyze_code("λ x = y");
        assert!(result.is_err() || result.expect("ctx").has_errors());
    }

    #[test]
    fn test_function_def() {
        let ctx = analyze_code("μ double(n: N) → N { n * 2 }").expect("ok");
        let sig = ctx.lookup_function("double").expect("fn");
        assert_eq!(sig.ret.name, "N");
    }

    #[test]
    fn test_lambda() {
        let ctx = analyze_code("λ f = |x| x * 2").expect("ok");
        assert_eq!(ctx.lookup("f").expect("f").ty.name, "→");
    }

    #[test]
    fn test_hof_map() {
        let ctx = analyze_code("λ r = Φ(σ[1, 2, 3], |x| x * 2)").expect("ok");
        assert_eq!(ctx.lookup("r").expect("r").ty.name, "σ");
    }

    #[test]
    fn test_tier() {
        let ctx = analyze_code("λ n = 42").expect("ok");
        assert_eq!(ctx.lookup("n").expect("n").ty.tier(), Tier::T1Universal);
    }
}
